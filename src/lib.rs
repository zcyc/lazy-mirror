use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

pub mod catalog;
pub mod conda;
pub mod config;
pub mod container;
pub mod dart;
pub mod docker;
pub mod go;
pub mod huggingface;
pub mod java;
pub mod node;
pub mod nuget;
pub mod pdm;
pub mod php;
pub mod poetry;
pub mod probe;
pub mod python;
pub mod r;
pub mod ruby;
pub mod rust;
pub mod sbt;
pub mod uv;

pub(crate) fn run(program: &str, args: &[&str]) -> io::Result<()> {
    let status = Command::new(program).args(args).status()?;
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("{program} exited with {status}")))
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

pub struct ToolStatus {
    pub version: String,
    pub configured: bool,
    pub detail: String,
}

pub(crate) fn home_file(relative_path: &str) -> io::Result<std::path::PathBuf> {
    dirs::home_dir()
        .map(|home| home.join(relative_path))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cannot determine home directory"))
}

pub(crate) fn write_with_backup_if<F>(path: &Path, content: &str, managed: F) -> io::Result<()>
where
    F: Fn(&str) -> bool,
{
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

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if !had_file {
        fs::write(&marker, [])?;
    }

    if let Err(error) = fs::write(path, content) {
        if had_file {
            let backup = backup_path(path);
            if backup.exists() {
                let _ = fs::copy(&backup, path);
            }
        } else {
            let _ = fs::remove_file(path);
        }
        let _ = fs::remove_file(marker);
        return Err(error);
    }

    Ok(())
}

pub(crate) fn remove_owned_if<F>(path: &Path, managed: F) -> io::Result<()>
where
    F: Fn(&str) -> bool,
{
    match fs::read_to_string(path) {
        Ok(existing) if managed(&existing) => fs::remove_file(path),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("refusing to remove unmanaged file {}", path.display()),
        )),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub(crate) fn update_named_managed_block(path: &Path, name: &str, block: &str) -> io::Result<()> {
    let start = format!("# >>> lazy-mirror:{name} >>>");
    let end_marker = format!("# <<< lazy-mirror:{name} <<<");
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
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, format!("{content}\n"))
}

pub(crate) fn remove_named_managed_block(path: &Path, name: &str) -> io::Result<()> {
    let start_marker = format!("# >>> lazy-mirror:{name} >>>");
    let end_marker = format!("# <<< lazy-mirror:{name} <<<");
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
    fs::write(path, content.trim_start_matches('\n'))
}

pub(crate) fn remove_with_backup_if<F>(path: &Path, managed: F) -> io::Result<()>
where
    F: Fn(&str) -> bool,
{
    let backup = backup_path(path);
    let marker = created_marker_path(path);
    if !backup.exists() && !marker.exists() {
        return Ok(());
    }
    match fs::read_to_string(path) {
        Ok(existing) if managed(&existing) => {
            if backup.exists() {
                fs::copy(&backup, path)?;
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
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

pub(crate) fn update_toml<F, M>(path: &Path, update: F, managed: M) -> io::Result<()>
where
    F: FnOnce(&mut toml::Table) -> io::Result<()>,
    M: Fn(&toml::Table) -> bool,
{
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
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
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
    if let Err(error) = fs::write(path, content) {
        if had_file {
            if backup.exists() {
                let _ = fs::copy(&backup, path);
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
    let Some(content) = read_optional(path)? else {
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
        fs::copy(&backup, path)?;
        fs::remove_file(backup)?;
    } else {
        fs::remove_file(path)?;
        let _ = fs::remove_file(created_marker_path(path));
    }
    Ok(())
}

fn backup_path(path: &Path) -> PathBuf {
    let mut backup = path.as_os_str().to_os_string();
    backup.push(".lazy-mirror.bak");
    PathBuf::from(backup)
}

fn created_marker_path(path: &Path) -> PathBuf {
    let mut marker = path.as_os_str().to_os_string();
    marker.push(".lazy-mirror.created");
    PathBuf::from(marker)
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
}
