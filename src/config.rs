use std::collections::BTreeMap;
use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use clap::ValueEnum;

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
#[value(rename_all = "lower")]
pub enum Scope {
    Project,
    User,
    System,
}

#[derive(Debug, Default)]
pub struct Config {
    pub path: PathBuf,
    mirrors: BTreeMap<String, String>,
    defaults: BTreeMap<String, String>,
    targets: BTreeMap<String, TargetConfig>,
    settings: Settings,
}

#[derive(Debug, Clone, Copy)]
pub struct Settings {
    pub timeout_seconds: u64,
    pub retries: u32,
    pub cache_ttl_seconds: u64,
    pub parallelism: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            timeout_seconds: 10,
            retries: 1,
            cache_ttl_seconds: 0,
            parallelism: 4,
        }
    }
}

#[derive(Debug, Default)]
struct TargetConfig {
    default: Option<String>,
    enabled: bool,
}

impl Config {
    pub fn load(override_path: Option<&Path>) -> io::Result<Self> {
        let path = override_path
            .map(Path::to_path_buf)
            .unwrap_or(default_path()?);
        let Some(content) = read_optional(&path)? else {
            return Ok(Self {
                path,
                ..Self::default()
            });
        };
        let document = content.parse::<toml::Table>().map_err(|error| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("invalid TOML config {}: {error}", path.display()),
            )
        })?;
        for key in document.keys() {
            if !matches!(key.as_str(), "mirrors" | "defaults" | "targets" | "options") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unknown TOML section [{key}]"),
                ));
            }
        }
        let defaults = string_table(&document, "defaults")?;
        let targets = target_table(&document)?;
        let defaults = if defaults.is_empty() {
            targets
                .iter()
                .filter_map(|(name, target)| {
                    target
                        .default
                        .as_ref()
                        .map(|value| (name.clone(), value.clone()))
                })
                .collect()
        } else {
            defaults
        };
        let mirrors = string_table(&document, "mirrors")?;
        for (name, url) in &mirrors {
            if !is_url(url) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("TOML mirror {name} must be an HTTP(S) URL"),
                ));
            }
        }
        Ok(Self {
            path,
            mirrors,
            defaults,
            settings: settings_table(&document)?,
            targets,
        })
    }

    pub fn mirror(&self, name: &str) -> Option<&str> {
        self.mirrors.get(name).map(String::as_str)
    }

    pub fn default_for(&self, target: &str) -> Option<&str> {
        self.defaults
            .get(target)
            .or_else(|| {
                self.targets
                    .get(target)
                    .and_then(|target| target.default.as_ref())
            })
            .map(String::as_str)
    }

    pub fn enabled(&self, target: &str) -> bool {
        self.targets.get(target).is_none_or(|target| target.enabled)
    }

    pub fn settings(&self) -> Settings {
        self.settings
    }

    pub fn custom_mirrors(&self) -> impl Iterator<Item = (&str, &str)> {
        self.mirrors
            .iter()
            .map(|(name, url)| (name.as_str(), url.as_str()))
    }
}

fn target_table(document: &toml::Table) -> io::Result<BTreeMap<String, TargetConfig>> {
    let Some(value) = document.get("targets") else {
        return Ok(BTreeMap::new());
    };
    let table = value.as_table().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "TOML section [targets] must be a table",
        )
    })?;
    table
        .iter()
        .map(|(key, value)| {
            if let Some(default) = value.as_str() {
                return Ok((
                    key.clone(),
                    TargetConfig {
                        default: Some(default.to_owned()),
                        enabled: true,
                    },
                ));
            }
            let target = value.as_table().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("TOML value [targets].{key} must be a string or table"),
                )
            })?;
            for field in target.keys() {
                if !matches!(field.as_str(), "default" | "enabled") {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("unknown field [targets.{key}].{field}"),
                    ));
                }
            }
            let default = target
                .get("default")
                .map(|value| {
                    value.as_str().map(str::to_owned).ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("TOML field [targets.{key}].default must be a string"),
                        )
                    })
                })
                .transpose()?;
            let enabled = target.get("enabled").map_or(Ok(true), |value| {
                value.as_bool().ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("TOML field [targets.{key}].enabled must be a boolean"),
                    )
                })
            })?;
            Ok((key.clone(), TargetConfig { default, enabled }))
        })
        .collect()
}

fn settings_table(document: &toml::Table) -> io::Result<Settings> {
    let Some(value) = document.get("options") else {
        return Ok(Settings::default());
    };
    let table = value.as_table().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "TOML section [options] must be a table",
        )
    })?;
    for field in table.keys() {
        if !matches!(
            field.as_str(),
            "timeout_seconds" | "retries" | "cache_ttl_seconds" | "parallelism"
        ) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown field [options].{field}"),
            ));
        }
    }
    let mut settings = Settings::default();
    if let Some(value) = table.get("timeout_seconds") {
        settings.timeout_seconds = positive_integer(value, "timeout_seconds")?;
    }
    if let Some(value) = table.get("retries") {
        settings.retries = integer(value, "retries")? as u32;
    }
    if let Some(value) = table.get("cache_ttl_seconds") {
        settings.cache_ttl_seconds = integer(value, "cache_ttl_seconds")?;
    }
    if let Some(value) = table.get("parallelism") {
        settings.parallelism = positive_integer(value, "parallelism")? as usize;
    }
    Ok(settings)
}

fn positive_integer(value: &toml::Value, name: &str) -> io::Result<u64> {
    let value = integer(value, name)?;
    if value == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("[options].{name} must be greater than zero"),
        ));
    }
    Ok(value)
}

fn integer(value: &toml::Value, name: &str) -> io::Result<u64> {
    value
        .as_integer()
        .and_then(|value| u64::try_from(value).ok())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("[options].{name} must be a non-negative integer"),
            )
        })
}

fn is_url(value: &str) -> bool {
    (value.starts_with("http://") || value.starts_with("https://"))
        && !value.contains(char::is_whitespace)
}

fn string_table(document: &toml::Table, name: &str) -> io::Result<BTreeMap<String, String>> {
    let Some(value) = document.get(name) else {
        return Ok(BTreeMap::new());
    };
    let table = value.as_table().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("TOML section [{name}] must be a table"),
        )
    })?;
    table
        .iter()
        .map(|(key, value)| {
            let value = if let Some(value) = value.as_str() {
                value
            } else if let Some(table) = value.as_table() {
                if table.keys().any(|field| field != "url") {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("unknown field [{name}.{key}]"),
                    ));
                }
                table
                    .get("url")
                    .and_then(toml::Value::as_str)
                    .ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("TOML value [{name}].{key}.url must be a string"),
                        )
                    })?
            } else {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("TOML value [{name}].{key} must be a string"),
                ));
            };
            Ok((key.clone(), value.to_owned()))
        })
        .collect()
}

fn default_path() -> io::Result<PathBuf> {
    if let Some(path) = env::var_os("LM_CONFIG") {
        if path.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "LM_CONFIG cannot be empty",
            ));
        }
        return Ok(PathBuf::from(path));
    }
    dirs::config_dir()
        .map(|path| path.join("lazy-mirror/config.toml"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cannot determine config directory"))
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
    fn reads_named_mirrors_and_target_defaults() {
        let path = env::temp_dir().join(format!("lm-config-test-{}.toml", std::process::id()));
        fs::write(
            &path,
            "[mirrors]\ninternal = \"https://mirror.example/simple\"\n[defaults]\npip = \"internal\"\n",
        )
        .unwrap();
        let config = Config::load(Some(&path)).unwrap();
        assert_eq!(
            config.mirror("internal"),
            Some("https://mirror.example/simple")
        );
        assert_eq!(config.default_for("pip"), Some("internal"));
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn reads_options_and_nested_target_settings() {
        let path = env::temp_dir().join(format!("lm-config-options-{}.toml", std::process::id()));
        fs::write(
            &path,
            "[mirrors]\ncorp = \"https://mirror.example/simple\"\n[targets.pip]\ndefault = \"corp\"\nenabled = false\n[options]\ntimeout_seconds = 3\nretries = 2\ncache_ttl_seconds = 60\nparallelism = 2\n",
        )
        .unwrap();
        let config = Config::load(Some(&path)).unwrap();
        assert_eq!(config.default_for("pip"), Some("corp"));
        assert!(!config.enabled("pip"));
        assert_eq!(config.settings().timeout_seconds, 3);
        assert_eq!(config.settings().retries, 2);
        assert_eq!(config.settings().cache_ttl_seconds, 60);
        assert_eq!(config.settings().parallelism, 2);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_unknown_config_sections() {
        let path = env::temp_dir().join(format!("lm-config-invalid-{}.toml", std::process::id()));
        fs::write(&path, "[unknown]\nvalue = true\n").unwrap();
        assert!(Config::load(Some(&path)).is_err());
        fs::remove_file(path).unwrap();
    }
}
