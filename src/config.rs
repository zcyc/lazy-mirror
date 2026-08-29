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
        let raw_defaults = string_table(&document, "defaults")?;
        let raw_targets = target_table(&document)?;
        let mut defaults = BTreeMap::new();
        for (name, value) in raw_defaults {
            let canonical = canonical_target(&name)?;
            if defaults.insert(canonical.clone(), value).is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("duplicate configuration for target {canonical}"),
                ));
            }
        }
        let mut targets = BTreeMap::new();
        for (name, target) in raw_targets {
            let canonical = canonical_target(&name)?;
            if targets.insert(canonical.clone(), target).is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("duplicate configuration for target {canonical}"),
                ));
            }
        }
        for (name, target) in &targets {
            if let Some(value) = &target.default {
                defaults
                    .entry(name.clone())
                    .or_insert_with(|| value.clone());
            }
        }
        let mirrors = string_table(&document, "mirrors")?;
        for (name, url) in &mirrors {
            if !is_url(url) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("TOML mirror {name} must be an HTTP(S) URL"),
                ));
            }
        }
        validate_references(&defaults, &mirrors)?;
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
        let target = crate::catalog::find(target).map_or(target, |spec| spec.name);
        self.defaults.get(target).map(String::as_str).or_else(|| {
            self.targets
                .get(target)
                .and_then(|target| target.default.as_deref())
        })
    }

    pub fn enabled(&self, target: &str) -> bool {
        let target = crate::catalog::find(target).map_or(target, |spec| spec.name);
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

    pub fn effective_json(&self) -> serde_json::Value {
        let targets = self
            .targets
            .iter()
            .map(|(name, target)| {
                let default = target.default.as_ref().map(|value| {
                    if is_url(value) {
                        redact_url(value)
                    } else {
                        value.clone()
                    }
                });
                (
                    name.clone(),
                    serde_json::json!({
                        "default": default,
                        "enabled": target.enabled,
                    }),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        let defaults = self
            .defaults
            .iter()
            .map(|(name, value)| {
                (
                    name.clone(),
                    serde_json::Value::String(if is_url(value) {
                        redact_url(value)
                    } else {
                        value.clone()
                    }),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        let mirrors = self
            .mirrors
            .iter()
            .map(|(name, value)| (name.clone(), serde_json::Value::String(redact_url(value))))
            .collect::<serde_json::Map<_, _>>();
        serde_json::json!({
            "schema": crate::JSON_SCHEMA,
            "config": self.path,
            "mirrors": mirrors,
            "defaults": defaults,
            "targets": targets,
            "options": {
                "timeout_seconds": self.settings.timeout_seconds,
                "retries": self.settings.retries,
                "cache_ttl_seconds": self.settings.cache_ttl_seconds,
                "parallelism": self.settings.parallelism,
            }
        })
    }
}

fn canonical_target(name: &str) -> io::Result<String> {
    crate::catalog::find(name)
        .map(|spec| spec.name.to_owned())
        .ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown target {name} in TOML configuration"),
            )
        })
}

fn validate_references(
    defaults: &BTreeMap<String, String>,
    mirrors: &BTreeMap<String, String>,
) -> io::Result<()> {
    for (target, selection) in defaults {
        let spec = crate::catalog::find(target).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown target {target} in TOML configuration"),
            )
        })?;
        let valid = is_url(selection)
            || mirrors.contains_key(selection)
            || (selection == "first" && !spec.mirrors.is_empty())
            || spec.mirrors.iter().any(|mirror| mirror.name == selection);
        if !valid {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown mirror {selection} for target {target}"),
            ));
        }
    }
    Ok(())
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
        settings.retries = bounded_integer(value, "retries", 10)? as u32;
    }
    if let Some(value) = table.get("cache_ttl_seconds") {
        settings.cache_ttl_seconds = integer(value, "cache_ttl_seconds")?;
    }
    if let Some(value) = table.get("parallelism") {
        settings.parallelism = bounded_integer(value, "parallelism", 64)? as usize;
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

fn bounded_integer(value: &toml::Value, name: &str, maximum: u64) -> io::Result<u64> {
    let value = integer(value, name)?;
    if value > maximum {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("[options].{name} must be at most {maximum}"),
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

pub(crate) fn is_url(value: &str) -> bool {
    let authority = value
        .split_once("://")
        .map(|(_, rest)| rest.split(['/', '?', '#']).next().unwrap_or_default())
        .unwrap_or_default();
    (value.starts_with("http://") || value.starts_with("https://"))
        && !authority.is_empty()
        && !authority.contains('@')
        && !value.chars().any(|character| {
            character.is_control()
                || character.is_whitespace()
                || matches!(character, '"' | '\'' | '\\' | '$' | '`')
        })
}

fn redact_url(value: &str) -> String {
    let Some(scheme) = value.find("://") else {
        return value.to_owned();
    };
    let authority_start = scheme + 3;
    let authority_end = value[authority_start..]
        .find(['/', '?', '#'])
        .map_or(value.len(), |offset| authority_start + offset);
    let authority = &value[authority_start..authority_end];
    let authority = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    let suffix = &value[authority_end..];
    let suffix = suffix
        .find(['?', '#'])
        .map_or(suffix, |offset| &suffix[..offset]);
    format!("{}://{}{}", &value[..scheme], authority, suffix)
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

    #[test]
    fn rejects_urls_that_can_escape_managed_shell_values() {
        let path = env::temp_dir().join(format!(
            "lm-config-unsafe-{}-{}.toml",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(
            &path,
            "[mirrors]\nunsafe = \"https://mirror.example/\\\"\"\n",
        )
        .unwrap();
        assert!(Config::load(Some(&path)).is_err());
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn rejects_unknown_config_references() {
        let path = env::temp_dir().join(format!(
            "lm-config-invalid-{}-{}.toml",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(&path, "[defaults]\npip = \"missing\"\n").unwrap();
        let error = Config::load(Some(&path)).unwrap_err();
        assert_eq!(error.kind(), io::ErrorKind::InvalidData);
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn aliases_share_canonical_target_settings() {
        let path = env::temp_dir().join(format!(
            "lm-config-alias-{}-{}.toml",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        fs::write(
            &path,
            "[mirrors]\ncorp = \"https://mirror.example/simple\"\n[targets.py]\ndefault = \"corp\"\nenabled = false\n",
        )
        .unwrap();
        let config = Config::load(Some(&path)).unwrap();
        assert_eq!(config.default_for("pip"), Some("corp"));
        assert!(!config.enabled("pip"));
        fs::remove_file(path).unwrap();
    }
}
