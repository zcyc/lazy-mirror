use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{Map, Value};

use crate::config::Scope;
use crate::{command_exists, command_output, command_version, ToolStatus};

const REGISTRY_MIRRORS: &str = "registry-mirrors";
const CONFIG_ENV: &str = "LM_DOCKER_DAEMON_CONFIG";
const BUILDKIT_CONFIG_ENV: &str = "LM_BUILDKIT_CONFIG";
const CREATED_SUFFIX: &str = ".lazy-mirror.created";
const CURRENT_SUFFIX: &str = ".lazy-mirror.current";
const BUILDKIT_CURRENT_SUFFIX: &str = ".lazy-mirror.buildkit.current";

const MIRRORS: &[(&str, &str)] = &[("daocloud", "https://docker.m.daocloud.io")];

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    let mirror = validate_mirror(mirror)?;
    set_at(&config_path(scope)?, &mirror)
}

pub fn unset(scope: Scope) -> io::Result<()> {
    unset_at(&config_path(scope)?)
}

pub fn buildkit_set(mirror: &str, scope: Scope) -> io::Result<()> {
    let mirror = validate_mirror(mirror)?;
    buildkit_set_at(&buildkit_config_path(scope)?, &mirror)
}

fn buildkit_set_at(path: &Path, mirror: &str) -> io::Result<()> {
    let previous_content = fs::read_to_string(path).ok();
    let had_backup = crate::backup_path(path).exists();
    let had_created_marker = crate::created_marker_path(path).exists();
    let previous_marker = fs::read_to_string(buildkit_current_marker_path(path)).ok();
    crate::update_toml(
        path,
        |document| set_buildkit_mirror(document, mirror),
        |document| is_buildkit_managed(path, document),
    )?;
    let result = fs::read_to_string(path)
        .map(|content| buildkit_state(mirror, &content))
        .and_then(|state| crate::atomic_write(&buildkit_current_marker_path(path), &state));
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
                let _ = crate::atomic_write(&buildkit_current_marker_path(path), &marker);
            }
            None => {
                let _ = fs::remove_file(buildkit_current_marker_path(path));
            }
        }
        return Err(error);
    }
    Ok(())
}

fn set_buildkit_mirror(document: &mut toml::Table, mirror: &str) -> io::Result<()> {
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

fn is_buildkit_managed(path: &Path, document: &toml::Table) -> bool {
    let Some(mirror) = buildkit_mirror(document) else {
        return false;
    };
    let Some(current) = fs::read_to_string(buildkit_current_marker_path(path)).ok() else {
        return false;
    };
    let Some((current_mirror, current_fingerprint)) = current.trim_end().split_once('\n') else {
        return false;
    };
    let Ok(content) = fs::read_to_string(path) else {
        return false;
    };
    if current_mirror != mirror
        || current_fingerprint != buildkit_fingerprint(&content)
        || validate_mirror(mirror).is_err()
    {
        return false;
    }
    true
}

fn buildkit_state(mirror: &str, content: &str) -> String {
    format!("{mirror}\n{}\n", buildkit_fingerprint(content))
}

fn buildkit_fingerprint(content: &str) -> String {
    let mut hash = 0xcbf29ce484222325_u64;
    for byte in content.bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }
    format!("{hash:016x}")
}

pub fn buildkit_unset(scope: Scope) -> io::Result<()> {
    buildkit_unset_at(&buildkit_config_path(scope)?)
}

fn buildkit_unset_at(path: &Path) -> io::Result<()> {
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
    let result =
        crate::remove_toml_with_backup(path, |document| is_buildkit_managed(path, document));
    if result.is_ok() {
        let _ = fs::remove_file(buildkit_current_marker_path(path));
    }
    result
}

pub fn buildkit_status(scope: Scope) -> io::Result<ToolStatus> {
    let version = buildkit_backend_version()?;
    let path = buildkit_config_path(scope)?;
    let source = fs::read_to_string(&path)
        .ok()
        .and_then(|content| content.parse::<toml::Table>().ok())
        .and_then(|document| buildkit_mirror(&document).map(str::to_owned));
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

pub fn buildkit_available() -> bool {
    buildkit_backend_version().is_ok()
}

fn buildkit_backend_version() -> io::Result<String> {
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
            format!("Docker mirror must be an HTTP(S) root URL without a path: {mirror}"),
        ));
    }
    Ok(mirror.to_owned())
}

fn buildkit_mirror(document: &toml::Table) -> Option<&str> {
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

pub fn status(scope: Scope) -> io::Result<ToolStatus> {
    let version = command_version("docker")?;
    let path = config_path(scope)?;
    let config = read_config(&path)?;
    let source = config.as_ref().and_then(configured_mirror);
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
    Ok(ToolStatus::new(
        version,
        configured,
        source,
        Some(path),
        detail,
    ))
}

fn configured_mirror(config: &Value) -> Option<String> {
    configured_mirrors(config)
        .ok()?
        .into_iter()
        .next()
        .map(str::to_owned)
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
        let backup = backup_path(path);
        if backup.exists() {
            restore_file(&backup, path)?;
            fs::remove_file(backup)?;
        } else {
            let _ = fs::remove_file(created_marker_path(path));
        }
        let _ = fs::remove_file(current_marker_path(path));
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

fn buildkit_current_marker_path(path: &Path) -> PathBuf {
    let mut marker = path.as_os_str().to_os_string();
    marker.push(BUILDKIT_CURRENT_SUFFIX);
    PathBuf::from(marker)
}

fn mirror_name(url: &str) -> Option<&'static str> {
    MIRRORS
        .iter()
        .find_map(|(name, candidate)| (*candidate == url).then_some(*name))
}

fn config_path(scope: Scope) -> io::Result<PathBuf> {
    if scope == Scope::Project {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Docker does not support project scope",
        ));
    }
    if let Some(path) = env::var_os(CONFIG_ENV) {
        if path.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{CONFIG_ENV} cannot be empty"),
            ));
        }
        return Ok(PathBuf::from(path));
    }

    match scope {
        Scope::Project => unreachable!(),
        Scope::User => {
            #[cfg(target_os = "linux")]
            {
                let config_home = env::var_os("XDG_CONFIG_HOME")
                    .map(PathBuf::from)
                    .or_else(|| dirs::home_dir().map(|home| home.join(".config")))
                    .ok_or_else(|| {
                        io::Error::new(io::ErrorKind::NotFound, "cannot determine home directory")
                    })?;
                Ok(config_home.join("docker/daemon.json"))
            }
            #[cfg(not(target_os = "linux"))]
            {
                crate::home_file(".docker/daemon.json")
            }
        }
        Scope::System => {
            #[cfg(target_os = "windows")]
            {
                Ok(PathBuf::from(r"C:\ProgramData\docker\config\daemon.json"))
            }
            #[cfg(not(target_os = "windows"))]
            {
                Ok(PathBuf::from("/etc/docker/daemon.json"))
            }
        }
    }
}

fn buildkit_config_path(scope: Scope) -> io::Result<PathBuf> {
    if scope == Scope::Project {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Docker BuildKit does not support project scope",
        ));
    }
    if let Some(path) = env::var_os(BUILDKIT_CONFIG_ENV) {
        if path.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{BUILDKIT_CONFIG_ENV} cannot be empty"),
            ));
        }
        return Ok(PathBuf::from(path));
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

    #[test]
    fn buildkit_mirror_is_merged_and_restored() {
        let path = temp_path("buildkit").with_file_name("buildkitd.toml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "debug = true\n").unwrap();

        buildkit_set_at(&path, mirror_url("daocloud").unwrap()).unwrap();
        let document: toml::Table = fs::read_to_string(&path).unwrap().parse().unwrap();
        assert_eq!(
            buildkit_mirror(&document),
            Some(mirror_url("daocloud").unwrap())
        );
        buildkit_unset_at(&path).unwrap();
        assert_eq!(fs::read_to_string(&path).unwrap(), "debug = true\n");
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn docker_mirrors_must_be_root_urls() {
        assert!(validate_mirror("https://mirror.example").is_ok());
        assert!(validate_mirror("https://mirror.example/").is_ok());
        assert!(validate_mirror("https://mirror.example/v2").is_err());
        assert!(validate_mirror("https://mirror.example?token=x").is_err());
    }

    #[test]
    fn buildkit_reset_never_removes_unmanaged_config() {
        let path = temp_path("buildkit-unmanaged").with_file_name("buildkitd.toml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        let content = "[registry.\"docker.io\"]\nmirrors = [\"https://user.example\"]\n";
        fs::write(&path, content).unwrap();

        assert!(buildkit_unset_at(&path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), content);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn buildkit_reset_refuses_external_changes() {
        let path = temp_path("buildkit-modified").with_file_name("buildkitd.toml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "debug = true\n").unwrap();

        buildkit_set_at(&path, mirror_url("daocloud").unwrap()).unwrap();
        let content = fs::read_to_string(&path)
            .unwrap()
            .replace("debug = true", "debug = false");
        fs::write(&path, &content).unwrap();

        assert!(buildkit_unset_at(&path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), content);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }

    #[test]
    fn buildkit_reset_refuses_external_comments() {
        let path = temp_path("buildkit-comment").with_file_name("buildkitd.toml");
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(&path, "debug = true\n").unwrap();

        buildkit_set_at(&path, mirror_url("daocloud").unwrap()).unwrap();
        let content = format!("{}# external note\n", fs::read_to_string(&path).unwrap());
        fs::write(&path, &content).unwrap();

        assert!(buildkit_unset_at(&path).is_err());
        assert_eq!(fs::read_to_string(&path).unwrap(), content);
        fs::remove_dir_all(path.parent().unwrap()).unwrap();
    }
}
