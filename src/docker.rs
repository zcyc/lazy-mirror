use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::{command_version, ToolStatus};

const REGISTRY_MIRRORS: &str = "registry-mirrors";
const CONFIG_ENV: &str = "LM_DOCKER_DAEMON_CONFIG";
const CREATED_SUFFIX: &str = ".lazy-mirror.created";
const CURRENT_SUFFIX: &str = ".lazy-mirror.current";

const MIRRORS: &[(&str, &str)] = &[("daocloud", "https://docker.m.daocloud.io")];

pub fn set(mirror: &str) -> io::Result<()> {
    if !(mirror.starts_with("http://") || mirror.starts_with("https://"))
        || mirror.chars().any(char::is_whitespace)
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("Docker mirror must be an HTTP(S) URL: {mirror}"),
        ));
    }
    set_at(&config_path()?, mirror)
}

pub fn unset() -> io::Result<()> {
    unset_at(&config_path()?)
}

pub fn status() -> io::Result<ToolStatus> {
    let version = command_version("docker")?;
    let path = config_path()?;
    let config = read_config(&path)?;
    let (configured, detail) = match config {
        None => (
            false,
            format!("registry-mirrors not set; path={}", path.display()),
        ),
        Some(config) => {
            let mirrors = configured_mirrors(&config)?;
            let names = mirrors
                .iter()
                .map(|url| mirror_name(url).unwrap_or("custom"))
                .collect::<Vec<_>>();
            if mirrors.is_empty() {
                (
                    false,
                    format!("registry-mirrors not set; path={}", path.display()),
                )
            } else {
                let configured = true;
                (
                    configured,
                    format!(
                        "registry-mirrors={}; source={}; path={}",
                        names.join(","),
                        mirrors.join(","),
                        path.display()
                    ),
                )
            }
        }
    };
    Ok(ToolStatus {
        version,
        configured,
        detail,
    })
}

fn set_at(path: &Path, url: &str) -> io::Result<()> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    let _lock = crate::lock(path)?;
    let backup = backup_path(path);
    let created = created_marker_path(path);
    let had_file = path.exists();
    let mut config = match read_config(path)? {
        Some(config) => config,
        None => Value::Object(Map::new()),
    };

    if !config.is_object() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!(
                "Docker config {} must contain a JSON object",
                path.display()
            ),
        ));
    }

    if had_file {
        if backup.exists() || created.exists() {
            if !is_managed(path, &config)? {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!(
                        "refusing to overwrite unmanaged Docker config {}; unset it before changing the mirror",
                        path.display()
                    ),
                ));
            }
        } else {
            fs::copy(path, &backup)?;
        }
    } else {
        if backup.exists() || created.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "Docker mirror state exists for missing config {}",
                    path.display()
                ),
            ));
        }
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)?;
        }
        fs::write(&created, [])?;
    }

    let object = config.as_object_mut().expect("checked above");
    object.insert(
        REGISTRY_MIRRORS.to_owned(),
        Value::Array(vec![Value::String(url.to_owned())]),
    );

    if let Err(error) = write_config(path, &config) {
        if had_file {
            if backup.exists() {
                let _ = fs::copy(&backup, path);
                let _ = fs::remove_file(backup);
            }
        } else {
            let _ = fs::remove_file(path);
            let _ = fs::remove_file(created);
        }
        return Err(error);
    }
    if let Err(error) = crate::atomic_write(&current_marker_path(path), url) {
        if had_file {
            if backup.exists() {
                let _ = restore_file(&backup, path);
                let _ = fs::remove_file(&backup);
            }
        } else {
            let _ = fs::remove_file(path);
            let _ = fs::remove_file(&created);
        }
        return Err(error);
    }
    Ok(())
}

fn unset_at(path: &Path) -> io::Result<()> {
    let _lock = crate::lock(path)?;
    let Some(config) = read_config(path)? else {
        return Ok(());
    };
    if !is_managed(path, &config)? {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!(
                "refusing to remove unmanaged Docker config {}; the file was modified",
                path.display()
            ),
        ));
    }

    let backup = backup_path(path);
    if backup.exists() {
        restore_file(&backup, path)?;
        fs::remove_file(backup)?;
    } else {
        fs::remove_file(path)?;
        fs::remove_file(created_marker_path(path))?;
    }
    let _ = fs::remove_file(current_marker_path(path));
    Ok(())
}

fn write_config(path: &Path, config: &Value) -> io::Result<()> {
    let content = serde_json::to_string_pretty(config)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?
        + "\n";
    crate::atomic_write(path, &content)
}

fn restore_file(backup: &Path, path: &Path) -> io::Result<()> {
    let content = fs::read_to_string(backup)?;
    crate::atomic_write(path, &content)
}

fn read_config(path: &Path) -> io::Result<Option<Value>> {
    match fs::read_to_string(path) {
        Ok(content) => serde_json::from_str(&content)
            .map(Some)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error),
    }
}

fn configured_mirrors(config: &Value) -> io::Result<Vec<&str>> {
    let Some(value) = config.get(REGISTRY_MIRRORS) else {
        return Ok(Vec::new());
    };
    let mirrors = value.as_array().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("Docker config key {REGISTRY_MIRRORS} must be an array"),
        )
    })?;
    mirrors
        .iter()
        .map(|value| {
            value.as_str().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("Docker config key {REGISTRY_MIRRORS} must contain strings"),
                )
            })
        })
        .collect()
}

fn is_managed(path: &Path, config: &Value) -> io::Result<bool> {
    let mirrors = configured_mirrors(config)?;
    if mirrors.len() != 1
        || !(mirrors[0].starts_with("http://") || mirrors[0].starts_with("https://"))
    {
        return Ok(false);
    }
    let current = fs::read_to_string(current_marker_path(path)).ok();
    if current.as_deref() != Some(mirrors[0]) && mirror_name(mirrors[0]).is_none() {
        return Ok(false);
    }

    let backup = backup_path(path);
    if backup.exists() {
        let Some(original) = read_config(&backup)? else {
            return Ok(false);
        };
        let Some(mut expected) = original.as_object().cloned() else {
            return Ok(false);
        };
        expected.insert(
            REGISTRY_MIRRORS.to_owned(),
            config[REGISTRY_MIRRORS].clone(),
        );
        return Ok(Value::Object(expected) == *config);
    }

    Ok(created_marker_path(path).exists()
        && config
            .as_object()
            .is_some_and(|object| object.len() == 1 && object.contains_key(REGISTRY_MIRRORS)))
}

#[cfg(test)]
fn mirror_url(name: &str) -> Option<&'static str> {
    MIRRORS
        .iter()
        .find_map(|(candidate, url)| (*candidate == name).then_some(*url))
}

fn current_marker_path(path: &Path) -> PathBuf {
    let mut marker = path.as_os_str().to_os_string();
    marker.push(CURRENT_SUFFIX);
    PathBuf::from(marker)
}

fn mirror_name(url: &str) -> Option<&'static str> {
    MIRRORS
        .iter()
        .find_map(|(name, candidate)| (*candidate == url).then_some(*name))
}

fn config_path() -> io::Result<PathBuf> {
    if let Some(path) = env::var_os(CONFIG_ENV) {
        if path.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{CONFIG_ENV} cannot be empty"),
            ));
        }
        return Ok(PathBuf::from(path));
    }

    #[cfg(target_os = "linux")]
    {
        if env::var_os("DOCKER_HOST")
            .is_some_and(|host| host.to_string_lossy().contains("/run/user/"))
        {
            let config_home = env::var_os("XDG_CONFIG_HOME")
                .map(PathBuf::from)
                .or_else(|| dirs::home_dir().map(|home| home.join(".config")))
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, "cannot determine home directory")
                })?;
            return Ok(config_home.join("docker/daemon.json"));
        }
        return Ok(PathBuf::from("/etc/docker/daemon.json"));
    }
    #[cfg(target_os = "windows")]
    {
        return Ok(PathBuf::from(r"C:\ProgramData\docker\config\daemon.json"));
    }
    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        crate::home_file(".docker/daemon.json")
    }
}

fn backup_path(path: &Path) -> PathBuf {
    let mut backup = path.as_os_str().to_os_string();
    backup.push(".lazy-mirror.bak");
    PathBuf::from(backup)
}

fn created_marker_path(path: &Path) -> PathBuf {
    let mut marker = path.as_os_str().to_os_string();
    marker.push(CREATED_SUFFIX);
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
        env::temp_dir().join(format!("lm-docker-{name}-{suffix}/daemon.json"))
    }

    #[test]
    fn selecting_mirrors_preserves_and_restores_user_config() {
        let path = temp_path("backup");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, r#"{"debug":true}"#).unwrap();

        set_at(&path, mirror_url("daocloud").unwrap()).unwrap();
        set_at(&path, "https://custom.example").unwrap();
        assert!(is_managed(&path, &read_config(&path).unwrap().unwrap()).unwrap());
        unset_at(&path).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), r#"{"debug":true}"#);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn unset_removes_a_config_created_by_lm() {
        let path = temp_path("created");
        set_at(&path, mirror_url("daocloud").unwrap()).unwrap();
        unset_at(&path).unwrap();
        assert!(!path.exists());
        assert!(!created_marker_path(&path).exists());
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn changed_managed_config_is_never_overwritten_or_removed() {
        let path = temp_path("changed");
        set_at(&path, mirror_url("daocloud").unwrap()).unwrap();
        fs::write(&path, r#"{"registry-mirrors":["https://example.com"]}"#).unwrap();

        assert!(set_at(&path, "https://custom.example").is_err());
        assert!(unset_at(&path).is_err());
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn custom_mirror_can_be_switched_and_restored() {
        let path = temp_path("custom");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, r#"{"debug":true}"#).unwrap();

        set_at(&path, "https://first.example").unwrap();
        set_at(&path, "https://second.example").unwrap();
        unset_at(&path).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), r#"{"debug":true}"#);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
