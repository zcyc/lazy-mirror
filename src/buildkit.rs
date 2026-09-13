use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::config::Scope;
use crate::{command_exists, command_output, command_version, ToolStatus};

const CONFIG_ENV: &str = "LM_BUILDKIT_CONFIG";
const CURRENT_SUFFIX: &str = ".lazy-mirror.buildkit.current";

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    let mirror = validate_mirror(mirror)?;
    set_at(&config_path(scope)?, &mirror)
}

fn set_at(path: &Path, mirror: &str) -> io::Result<()> {
    let previous_content = fs::read_to_string(path).ok();
    let had_backup = crate::backup_path(path).exists();
    let had_created_marker = crate::created_marker_path(path).exists();
    let previous_marker = fs::read_to_string(current_marker_path(path)).ok();
    crate::update_toml(
        path,
        |document| set_mirror(document, mirror),
        |document| is_managed(path, document),
    )?;
    let result = fs::read_to_string(path)
        .map(|content| state(mirror, &content))
        .and_then(|state| crate::atomic_write(&current_marker_path(path), &state));
    if let Err(error) = result {
        if let Some(content) = previous_content {
            let _ = crate::atomic_write(path, &content);
        } else {
            let _ = fs::remove_file(path);
        }
        if !had_backup {
            let _ = fs::remove_file(crate::backup_path(path));
        }
        if !had_created_marker {
            let _ = fs::remove_file(crate::created_marker_path(path));
        }
        match previous_marker {
            Some(marker) => {
                let _ = crate::atomic_write(&current_marker_path(path), &marker);
            }
            None => {
                let _ = fs::remove_file(current_marker_path(path));
            }
        }
        return Err(error);
    }
    Ok(())
}

fn set_mirror(document: &mut toml::Table, mirror: &str) -> io::Result<()> {
    let registry = document
        .entry("registry")
        .or_insert_with(|| toml::Value::Table(toml::Table::new()))
        .as_table_mut()
        .ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "registry must be a TOML table")
        })?;
    let docker = registry
        .entry("docker.io")
        .or_insert_with(|| toml::Value::Table(toml::Table::new()))
        .as_table_mut()
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                "registry.docker.io must be a TOML table",
            )
        })?;
    docker.insert(
        "mirrors".to_owned(),
        toml::Value::Array(vec![toml::Value::String(mirror.to_owned())]),
    );
    Ok(())
}

fn is_managed(path: &Path, document: &toml::Table) -> bool {
    let Some(mirror) = mirror(document) else {
        return false;
    };
    let Some(current) = fs::read_to_string(current_marker_path(path)).ok() else {
        return false;
    };
    let Some((current_mirror, current_fingerprint)) = current.trim_end().split_once('\n') else {
        return false;
    };
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    current_mirror == mirror
        && current_fingerprint == fingerprint(&content)
        && validate_mirror(mirror).is_ok()
}

fn state(mirror: &str, content: &str) -> String {
    format!("{mirror}\n{}\n", fingerprint(content))
}

fn fingerprint(content: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in content.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

pub fn unset(scope: Scope) -> io::Result<()> {
    unset_at(&config_path(scope)?)
}

fn unset_at(path: &Path) -> io::Result<()> {
    if !path.exists()
        && !crate::backup_path(path).exists()
        && !crate::created_marker_path(path).exists()
    {
        return Ok(());
    }
    if !crate::backup_path(path).exists() && !crate::created_marker_path(path).exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "refusing to restore unmanaged BuildKit config {}",
                path.display()
            ),
        ));
    }
    let result = crate::remove_toml_with_backup(path, |document| is_managed(path, document));
    if result.is_ok() {
        let _ = fs::remove_file(current_marker_path(path));
    }
    result
}

pub fn status(scope: Scope) -> io::Result<ToolStatus> {
    let version = backend_version()?;
    let path = config_path(scope)?;
    let source = fs::read_to_string(&path)
        .ok()
        .and_then(|content| content.parse::<toml::Table>().ok())
        .and_then(|document| mirror(&document).map(str::to_owned));
    Ok(ToolStatus::new(
        version,
        source.is_some(),
        source.clone(),
        Some(path.clone()),
        format!(
            "registry.docker.io.mirrors={}; config={}",
            source.unwrap_or_else(|| "not configured".to_owned()),
            path.display()
        ),
    ))
}

pub fn available() -> bool {
    backend_version().is_ok()
}

fn backend_version() -> io::Result<String> {
    let mut versions = Vec::new();
    if command_exists("buildctl") {
        versions.push(command_version("buildctl")?);
    }
    if command_exists("docker") {
        if let Ok(version) = command_output("docker", &["buildx", "version"]) {
            versions.push(version);
        }
    }
    if versions.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "neither buildctl nor docker buildx is installed",
        ));
    }
    Ok(versions.join("; "))
}

fn validate_mirror(mirror: &str) -> io::Result<String> {
    let mirror = mirror.trim_end_matches('/');
    let valid_root = crate::config::is_url(mirror)
        && mirror
            .split_once("://")
            .is_some_and(|(_, authority)| !authority.contains(['/', '?', '#']));
    if !valid_root {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "BuildKit mirror must be an HTTP(S) root URL without a path: {}",
                crate::config::redact_selection(mirror)
            ),
        ));
    }
    Ok(mirror.to_owned())
}

fn mirror(document: &toml::Table) -> Option<&str> {
    document
        .get("registry")
        .and_then(toml::Value::as_table)
        .and_then(|registry| registry.get("docker.io"))
        .and_then(toml::Value::as_table)
        .and_then(|docker| docker.get("mirrors"))
        .and_then(toml::Value::as_array)
        .and_then(|mirrors| mirrors.first())
        .and_then(toml::Value::as_str)
}

fn config_path(scope: Scope) -> io::Result<PathBuf> {
    if scope == Scope::Project {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "BuildKit does not support project scope",
        ));
    }
    if let Some(path) = env::var_os(CONFIG_ENV) {
        if path.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{CONFIG_ENV} cannot be empty"),
            ));
        }
        return Ok(path.into());
    }
    match scope {
        Scope::Project => unreachable!(),
        Scope::User => {
            let config_home = env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .or_else(|| dirs::home_dir().map(|home| home.join(".config")))
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "cannot determine home directory")
                })?;
            Ok(config_home.join("buildkit/buildkitd.toml"))
        }
        Scope::System => {
            #[cfg(windows)]
            {
                Ok(PathBuf::from(r"C:\ProgramData\buildkit\buildkitd.toml"))
            }
            #[cfg(not(windows))]
            {
                Ok(PathBuf::from("/etc/buildkit/buildkitd.toml"))
            }
        }
    }
}

fn current_marker_path(path: &Path) -> PathBuf {
    let mut marker = path.as_os_str().to_os_string();
    marker.push(CURRENT_SUFFIX);
    PathBuf::from(marker)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_path(name: &str) -> PathBuf {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("lm-buildkit-{name}-{suffix}/buildkitd.toml"))
    }

    #[test]
    fn mirror_is_merged_and_restored() {
        let path = temp_path("backup");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "debug = true\n").unwrap();

        set_at(&path, "https://docker.m.daocloud.io").unwrap();
        let document: toml::Table = fs::read_to_string(&path).unwrap().parse().unwrap();
        assert_eq!(mirror(&document), Some("https://docker.m.daocloud.io"));
        unset_at(&path).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "debug = true\n");
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn reset_never_removes_unmanaged_config() {
        let path = temp_path("unmanaged");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let content = "[registry.\"docker.io\"]\nmirrors = [\"https://user.example\"]\n";
        fs::write(&path, content).unwrap();

        assert!(unset_at(&path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), content);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn reset_refuses_external_changes() {
        let path = temp_path("modified");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "debug = true\n").unwrap();

        set_at(&path, "https://docker.m.daocloud.io").unwrap();
        let content = fs::read_to_string(&path)
            .unwrap()
            .replace("debug = true", "debug = false");
        fs::write(&path, &content).unwrap();

        assert!(unset_at(&path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), content);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn reset_refuses_external_comments() {
        let path = temp_path("comment");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "debug = true\n").unwrap();

        set_at(&path, "https://docker.m.daocloud.io").unwrap();
        let content = format!("{}# external note\n", fs::read_to_string(&path).unwrap());
        fs::write(&path, &content).unwrap();

        assert!(unset_at(&path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), content);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
