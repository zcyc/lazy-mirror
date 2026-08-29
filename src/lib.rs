use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

pub mod catalog;
pub mod conda;
pub mod config;
pub mod container;
pub mod dart;
pub mod docker;
pub mod go;
pub mod helm;
pub mod huggingface;
pub mod java;
pub mod node;
pub mod nuget;
pub mod pdm;
pub mod php;
pub mod platform;
pub mod poetry;
pub mod probe;
pub mod python;
pub mod r;
pub mod ruby;
pub mod rust;
pub mod sbt;
pub mod uv;

pub const JSON_SCHEMA: &str = "lm/v1";

pub(crate) fn run(program: &str, args: &[&str]) -> io::Result<()> {
    let output = Command::new(program).args(args).output()?;
    if output.status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "{program} exited with {}",
            output.status
        )))
    }
}

pub(crate) fn command_output(program: &str, args: &[&str]) -> io::Result<String> {
    let output = Command::new(program).args(args).output()?;
    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stderr).trim().to_owned();
        let message = if detail.is_empty() {
            format!("{program} exited with {}", output.status)
        } else {
            format!("{program} exited with {}: {detail}", output.status)
        };
        return Err(io::Error::other(message));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

pub(crate) fn command_version(program: &str) -> io::Result<String> {
    Ok(command_output(program, &["--version"])?
        .lines()
        .next()
        .unwrap_or_default()
        .to_owned())
}

pub fn command_exists(program: &str) -> bool {
    Command::new(program)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

pub struct ToolStatus {
    pub version: String,
    pub configured: bool,
    pub source: Option<String>,
    pub path: Option<PathBuf>,
    pub detail: String,
}

impl ToolStatus {
    pub fn new(
        version: String,
        configured: bool,
        source: Option<String>,
        path: Option<PathBuf>,
        detail: impl Into<String>,
    ) -> Self {
        Self {
            version,
            configured,
            source,
            path,
            detail: detail.into(),
        }
    }
}

pub(crate) struct FileLock {
    path: Option<PathBuf>,
}

impl FileLock {
    fn acquire(path: &Path) -> io::Result<Self> {
        let lock_path = lock_path(path);
        if lock_path.parent().is_some_and(|parent| !parent.exists()) {
            return Ok(Self { path: None });
        }
        let started = Instant::now();
        loop {
            match OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&lock_path)
            {
                Ok(mut file) => {
                    let _ = writeln!(file, "pid={}", std::process::id());
                    return Ok(Self {
                        path: Some(lock_path),
                    });
                }
                Err(error)
                    if error.kind() == io::ErrorKind::AlreadyExists
                        && started.elapsed() < Duration::from_secs(10) =>
                {
                    if fs::metadata(&lock_path)
                        .and_then(|metadata| metadata.modified())
                        .ok()
                        .and_then(|modified| modified.elapsed().ok())
                        .is_some_and(|age| age > Duration::from_secs(60))
                    {
                        let _ = fs::remove_file(&lock_path);
                        continue;
                    }
                    thread::sleep(Duration::from_millis(50));
                }
                Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
                    return Err(io::Error::new(
                        io::ErrorKind::WouldBlock,
                        format!("timed out waiting for {}", lock_path.display()),
                    ))
                }
                Err(error) => return Err(error),
            }
        }
    }
}

impl Drop for FileLock {
    fn drop(&mut self) {
        if let Some(path) = &self.path {
            let _ = fs::remove_file(path);
        }
    }
}

pub(crate) fn lock(path: &Path) -> io::Result<FileLock> {
    FileLock::acquire(path)
}

pub(crate) fn atomic_write(path: &Path, content: &str) -> io::Result<()> {
    atomic_write_bytes(path, content.as_bytes())
}

fn atomic_write_bytes(path: &Path, content: &[u8]) -> io::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let mode = fs::metadata(path).ok().map(file_mode);
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let mut temporary = path.as_os_str().to_os_string();
    temporary.push(format!(".lazy-mirror.tmp-{}-{nonce}", std::process::id()));
    let temporary = PathBuf::from(temporary);
    let result = (|| {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        if let Some(mode) = mode {
            set_file_mode(&file, mode)?;
        }
        file.write_all(content)?;
        file.sync_all()?;
        replace_file(&temporary, path)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temporary);
    }
    result
}

fn replace_file(temporary: &Path, path: &Path) -> io::Result<()> {
    match fs::rename(temporary, path) {
        Ok(()) => Ok(()),
        #[cfg(windows)]
        Err(error) if error.kind() == io::ErrorKind::AlreadyExists => {
            fs::remove_file(path)?;
            fs::rename(temporary, path)
        }
        Err(error) => Err(error),
    }
}

#[cfg(unix)]
fn file_mode(metadata: fs::Metadata) -> u32 {
    use std::os::unix::fs::PermissionsExt;

    metadata.permissions().mode()
}

#[cfg(not(unix))]
fn file_mode(metadata: fs::Metadata) -> bool {
    metadata.permissions().readonly()
}

#[cfg(unix)]
fn set_file_mode(file: &fs::File, mode: u32) -> io::Result<()> {
    use std::os::unix::fs::PermissionsExt;

    file.set_permissions(fs::Permissions::from_mode(mode))
}

#[cfg(not(unix))]
fn set_file_mode(file: &fs::File, readonly: bool) -> io::Result<()> {
    let mut permissions = file.metadata()?.permissions();
    permissions.set_readonly(readonly);
    file.set_permissions(permissions)
}

pub(crate) fn home_file(relative_path: &str) -> io::Result<std::path::PathBuf> {
    dirs::home_dir()
        .map(|home| home.join(relative_path))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cannot determine home directory"))
}

pub(crate) fn nearest_existing_file(start: &Path, names: &[&str]) -> Option<PathBuf> {
    let mut directory = start;
    loop {
        for name in names {
            let path = directory.join(name);
            if path.is_file() {
                return Some(path);
            }
        }
        directory = directory.parent()?;
    }
}

pub(crate) fn system_file(windows: &str, unix: &str) -> PathBuf {
    #[cfg(windows)]
    {
        let _ = unix;
        PathBuf::from(windows)
    }
    #[cfg(not(windows))]
    {
        let _ = windows;
        PathBuf::from(unix)
    }
}

pub(crate) fn write_with_backup_if<F>(path: &Path, content: &str, managed: F) -> io::Result<()>
where
    F: Fn(&str) -> bool,
{
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let _lock = lock(path)?;
    let marker = created_marker_path(path);
    let had_file = path.exists();
    match fs::read_to_string(path) {
        Ok(existing) if existing == content => return Ok(()),
        Ok(existing) => {
            let backup = backup_path(path);
            if (backup.exists() || marker.exists()) && !managed(&existing) {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "{} is already managed; unset it before changing the mirror",
                        path.display()
                    ),
                ));
            }
            if !backup.exists() && !marker.exists() {
                fs::copy(path, &backup)?;
            }
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if backup_path(path).exists() || marker.exists() {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("backup exists for missing file {}", path.display()),
                ));
            }
        }
        Err(error) => return Err(error),
    }

    if !had_file {
        fs::write(&marker, [])?;
    }

    if let Err(error) = atomic_write(path, content) {
        if had_file {
            let backup = backup_path(path);
            if backup.exists() {
                let _ = restore_file(&backup, path);
            }
        } else {
            let _ = fs::remove_file(path);
        }
        let _ = fs::remove_file(marker);
        return Err(error);
    }

    Ok(())
}

pub(crate) fn update_named_managed_block(path: &Path, name: &str, block: &str) -> io::Result<()> {
    update_named_block(path, name, "#", block)
}

pub(crate) fn update_named_block(
    path: &Path,
    name: &str,
    comment: &str,
    block: &str,
) -> io::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let _lock = lock(path)?;
    let start = format!("{comment} >>> lazy-mirror:{name} >>>");
    let end_marker = format!("{comment} <<< lazy-mirror:{name} <<<");
    let existing = read_optional(path)?.unwrap_or_default();
    let replacement = format!("{start}\n{block}\n{end_marker}");
    let content = match (existing.find(&start), existing.find(&end_marker)) {
        (Some(start), Some(end)) if end >= start => {
            let end = end + end_marker.len();
            format!("{}{}{}", &existing[..start], replacement, &existing[end..])
        }
        (None, None) => {
            if existing.is_empty() {
                replacement
            } else {
                format!("{}\n\n{replacement}", existing.trim_end())
            }
        }
        _ => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("incomplete lazy-mirror block in {}", path.display()),
            ))
        }
    };
    atomic_write(path, &format!("{content}\n"))
}

pub(crate) fn remove_named_managed_block(path: &Path, name: &str) -> io::Result<()> {
    remove_named_block(path, name, "#")
}

pub(crate) fn remove_named_block(path: &Path, name: &str, comment: &str) -> io::Result<()> {
    let _lock = lock(path)?;
    let start_marker = format!("{comment} >>> lazy-mirror:{name} >>>");
    let end_marker = format!("{comment} <<< lazy-mirror:{name} <<<");
    let Some(existing) = read_optional(path)? else {
        return Ok(());
    };
    let Some(start) = existing.find(&start_marker) else {
        return Ok(());
    };
    let Some(relative_end) = existing[start..].find(&end_marker) else {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("incomplete lazy-mirror block in {}", path.display()),
        ));
    };
    let end = start + relative_end + end_marker.len();
    let content = format!("{}{}", &existing[..start], &existing[end..]);
    atomic_write(path, content.trim_start_matches('\n'))
}

pub(crate) fn remove_with_backup_if<F>(path: &Path, managed: F) -> io::Result<()>
where
    F: Fn(&str) -> bool,
{
    let _lock = lock(path)?;
    let backup = backup_path(path);
    let marker = created_marker_path(path);
    if !backup.exists() && !marker.exists() {
        return Ok(());
    }
    match fs::read_to_string(path) {
        Ok(existing) if managed(&existing) => {
            if backup.exists() {
                restore_file(&backup, path)?;
                fs::remove_file(backup)?;
            } else {
                fs::remove_file(path)?;
            }
            let _ = fs::remove_file(created_marker_path(path));
            Ok(())
        }
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "refusing to restore {}; the managed file was modified",
                path.display()
            ),
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            if backup.exists() {
                restore_file(&backup, path)?;
                fs::remove_file(backup)?;
            }
            let _ = fs::remove_file(marker);
            Ok(())
        }
        Err(error) => Err(error),
    }
}

pub(crate) fn update_toml<F, M>(path: &Path, update: F, managed: M) -> io::Result<()>
where
    F: FnOnce(&mut toml::Table) -> io::Result<()>,
    M: Fn(&toml::Table) -> bool,
{
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let _lock = lock(path)?;
    let backup = backup_path(path);
    let marker = created_marker_path(path);
    let had_file = path.exists();
    let mut new_backup = false;
    let mut new_marker = false;
    let mut document = match fs::read_to_string(path) {
        Ok(content) => content.parse::<toml::Table>().map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid TOML file {}: {error}", path.display()),
            )
        })?,
        Err(error) if error.kind() == io::ErrorKind::NotFound => toml::Table::new(),
        Err(error) => return Err(error),
    };

    if had_file {
        if backup.exists() || marker.exists() {
            if !managed(&document) {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "refusing to overwrite unmanaged TOML file {}; reset it before changing the mirror",
                        path.display()
                    ),
                ));
            }
        } else {
            fs::copy(path, &backup)?;
            new_backup = true;
        }
    } else {
        if backup.exists() || marker.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "mirror state exists for missing TOML file {}",
                    path.display()
                ),
            ));
        }
        fs::write(&marker, [])?;
        new_marker = true;
    }

    if let Err(error) = update(&mut document) {
        if new_backup {
            let _ = fs::remove_file(&backup);
        }
        if new_marker {
            let _ = fs::remove_file(&marker);
        }
        return Err(error);
    }
    let content = match toml::to_string_pretty(&document) {
        Ok(content) => content,
        Err(error) => {
            if new_backup {
                let _ = fs::remove_file(&backup);
            }
            if new_marker {
                let _ = fs::remove_file(&marker);
            }
            return Err(io::Error::new(io::ErrorKind::InvalidData, error));
        }
    };
    if let Err(error) = atomic_write(path, &content) {
        if had_file {
            if backup.exists() {
                let _ = restore_file(&backup, path);
            }
        } else {
            let _ = fs::remove_file(path);
        }
        if new_backup {
            let _ = fs::remove_file(&backup);
        }
        if new_marker {
            let _ = fs::remove_file(&marker);
        }
        return Err(error);
    }
    Ok(())
}

pub(crate) fn remove_toml_with_backup<M>(path: &Path, managed: M) -> io::Result<()>
where
    M: Fn(&toml::Table) -> bool,
{
    let _lock = lock(path)?;
    let Some(content) = read_optional(path)? else {
        let backup = backup_path(path);
        if backup.exists() {
            restore_file(&backup, path)?;
            fs::remove_file(backup)?;
        }
        let _ = fs::remove_file(created_marker_path(path));
        return Ok(());
    };
    let document = content.parse::<toml::Table>().map_err(|error| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("invalid TOML file {}: {error}", path.display()),
        )
    })?;
    if !managed(&document) {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("refusing to restore unmanaged TOML file {}", path.display()),
        ));
    }
    let backup = backup_path(path);
    if backup.exists() {
        restore_file(&backup, path)?;
        fs::remove_file(backup)?;
    } else {
        fs::remove_file(path)?;
        let _ = fs::remove_file(created_marker_path(path));
    }
    Ok(())
}

pub(crate) fn backup_path(path: &Path) -> PathBuf {
    let mut backup = path.as_os_str().to_os_string();
    backup.push(".lazy-mirror.bak");
    PathBuf::from(backup)
}

pub(crate) fn created_marker_path(path: &Path) -> PathBuf {
    let mut marker = path.as_os_str().to_os_string();
    marker.push(".lazy-mirror.created");
    PathBuf::from(marker)
}

fn lock_path(path: &Path) -> PathBuf {
    let mut lock = path.as_os_str().to_os_string();
    lock.push(".lazy-mirror.lock");
    PathBuf::from(lock)
}

fn restore_file(backup: &Path, path: &Path) -> io::Result<()> {
    let content = fs::read_to_string(backup)?;
    atomic_write(path, &content)
}

fn read_optional(path: &Path) -> io::Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(content) => Ok(Some(content)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backed_up_files_are_restored() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("lm-backup-{suffix}"));
        let backup = backup_path(&path);

        fs::write(&path, "user config").unwrap();
        write_with_backup_if(&path, "mirror config", |content| content == "mirror config").unwrap();
        assert_eq!(fs::read_to_string(&backup).unwrap(), "user config");
        remove_with_backup_if(&path, |content| content == "mirror config").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "user config");
        assert!(!backup.exists());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn named_blocks_can_coexist_and_be_removed_independently() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("lm-block-{suffix}"));

        update_named_managed_block(&path, "dart", "export PUB_HOSTED_URL=\"https://dart\"")
            .unwrap();
        update_named_managed_block(
            &path,
            "flutter",
            "export FLUTTER_STORAGE_BASE_URL=\"https://flutter\"",
        )
        .unwrap();
        remove_named_managed_block(&path, "dart").unwrap();
        let content = fs::read_to_string(&path).unwrap();
        assert!(!content.contains("PUB_HOSTED_URL"));
        assert!(content.contains("FLUTTER_STORAGE_BASE_URL"));
        remove_named_managed_block(&path, "flutter").unwrap();
        assert!(fs::read_to_string(&path).unwrap().is_empty());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn managed_file_can_switch_mirrors_without_losing_original() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("lm-switch-{suffix}"));

        fs::write(&path, "user config").unwrap();
        write_with_backup_if(&path, "mirror one", |content| {
            content.starts_with("mirror ")
        })
        .unwrap();
        write_with_backup_if(&path, "mirror two", |content| {
            content.starts_with("mirror ")
        })
        .unwrap();
        remove_with_backup_if(&path, |content| content == "mirror two").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "user config");
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn missing_managed_file_restores_its_backup() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("lm-missing-{suffix}"));

        fs::write(&path, "user config").unwrap();
        write_with_backup_if(&path, "mirror config", |content| content == "mirror config").unwrap();
        fs::remove_file(&path).unwrap();
        remove_with_backup_if(&path, |content| content == "mirror config").unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "user config");
        fs::remove_file(path).unwrap();
    }
}
