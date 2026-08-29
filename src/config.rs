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

#[derive(Debug, Clone)]
pub struct ConfigSource {
    pub path: PathBuf,
    pub active: bool,
    pub loaded: bool,
}

#[derive(Debug, Default)]
pub struct Config {
    pub path: PathBuf,
    mirrors: BTreeMap<String, String>,
    defaults: BTreeMap<String, String>,
    targets: BTreeMap<String, TargetConfig>,
    settings: Settings,
    sources: Vec<ConfigSource>,
    origins: BTreeMap<String, PathBuf>,
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
    enabled: Option<bool>,
    mirrors: Option<Vec<String>>,
}

impl TargetConfig {
    fn merge(&mut self, overlay: Self) {
        if overlay.default.is_some() {
            self.default = overlay.default;
        }
        if overlay.enabled.is_some() {
            self.enabled = overlay.enabled;
        }
        if overlay.mirrors.is_some() {
            self.mirrors = overlay.mirrors;
        }
    }
}

#[derive(Debug, Default)]
struct SettingsPatch {
    timeout_seconds: Option<u64>,
    retries: Option<u32>,
    cache_ttl_seconds: Option<u64>,
    parallelism: Option<usize>,
}

impl Config {
    pub fn load(override_path: Option<&Path>) -> io::Result<Self> {
        Self::load_with_options(override_path, false)
    }

    pub fn load_with_options(override_path: Option<&Path>, no_config: bool) -> io::Result<Self> {
        let primary = match override_path {
            Some(path) => path.to_owned(),
            None if no_config => platform_default_path()?,
            None => default_path()?,
        };
        let paths = if no_config || override_path.is_some() || env::var_os("LM_CONFIG").is_some() {
            vec![primary.clone()]
        } else {
            discovered_paths(&primary)?
        };
        Self::load_paths(paths, primary, !no_config)
    }

    fn load_paths(paths: Vec<PathBuf>, primary: PathBuf, read_files: bool) -> io::Result<Self> {
        let mut config = Self {
            path: primary,
            sources: paths
                .into_iter()
                .map(|path| ConfigSource {
                    path,
                    active: read_files,
                    loaded: false,
                })
                .collect(),
            ..Self::default()
        };
        if read_files {
            for index in 0..config.sources.len() {
                let path = config.sources[index].path.clone();
                let Some(content) = read_optional(&path)? else {
                    continue;
                };
                config.sources[index].loaded = true;
                let document = content.parse::<toml::Table>().map_err(|error| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("invalid TOML config {}: {error}", path.display()),
                    )
                })?;
                config.apply_document(&document, &path)?;
            }
        }
        let fallbacks = config
            .targets
            .iter()
            .filter_map(|(name, target)| {
                target
                    .default
                    .as_ref()
                    .filter(|_| !config.defaults.contains_key(name))
                    .map(|value| {
                        (
                            name.clone(),
                            value.clone(),
                            config
                                .origins
                                .get(&format!("target.default:{name}"))
                                .cloned(),
                        )
                    })
            })
            .collect::<Vec<_>>();
        for (name, value, origin) in fallbacks {
            config.defaults.insert(name.clone(), value);
            if let Some(origin) = origin {
                config.origins.insert(format!("default:{name}"), origin);
            }
        }
        validate_references(&config.defaults, &config.targets, &config.mirrors)?;
        Ok(config)
    }

    fn apply_document(&mut self, document: &toml::Table, source: &Path) -> io::Result<()> {
        for key in document.keys() {
            if !matches!(key.as_str(), "mirrors" | "defaults" | "targets" | "options") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("unknown TOML section [{key}]"),
                ));
            }
        }

        let mut defaults = BTreeMap::new();
        for (name, value) in string_table(document, "defaults")? {
            let canonical = canonical_target(&name)?;
            if defaults.insert(canonical.clone(), value).is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("duplicate configuration for target {canonical}"),
                ));
            }
        }
        for (name, value) in defaults {
            self.defaults.insert(name.clone(), value);
            self.origins
                .insert(format!("default:{name}"), source.to_owned());
        }

        let mut targets = BTreeMap::new();
        for (name, target) in target_table(document)? {
            let canonical = canonical_target(&name)?;
            if targets.insert(canonical.clone(), target).is_some() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("duplicate configuration for target {canonical}"),
                ));
            }
        }
        for (name, target) in targets {
            if target.default.is_some() {
                self.origins
                    .insert(format!("target.default:{name}"), source.to_owned());
            }
            if target.enabled.is_some() {
                self.origins
                    .insert(format!("target.enabled:{name}"), source.to_owned());
            }
            if target.mirrors.is_some() {
                self.origins
                    .insert(format!("target.mirrors:{name}"), source.to_owned());
            }
            self.targets.entry(name).or_default().merge(target);
        }

        let mirrors = string_table(document, "mirrors")?;
        for (name, url) in &mirrors {
            if !is_config_mirror_url(url) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("TOML mirror {name} must be a valid HTTP(S) or Cargo sparse URL"),
                ));
            }
        }
        for (name, url) in mirrors {
            self.mirrors.insert(name.clone(), url);
            self.origins
                .insert(format!("mirror:{name}"), source.to_owned());
        }

        let patch = settings_table(document)?;
        if let Some(value) = patch.timeout_seconds {
            self.settings.timeout_seconds = value;
            self.origins
                .insert("option:timeout_seconds".to_owned(), source.to_owned());
        }
        if let Some(value) = patch.retries {
            self.settings.retries = value;
            self.origins
                .insert("option:retries".to_owned(), source.to_owned());
        }
        if let Some(value) = patch.cache_ttl_seconds {
            self.settings.cache_ttl_seconds = value;
            self.origins
                .insert("option:cache_ttl_seconds".to_owned(), source.to_owned());
        }
        if let Some(value) = patch.parallelism {
            self.settings.parallelism = value;
            self.origins
                .insert("option:parallelism".to_owned(), source.to_owned());
        }
        Ok(())
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
        self.targets
            .get(target)
            .and_then(|target| target.enabled)
            .unwrap_or(true)
    }

    pub fn mirrors_for(&self, target: &str) -> Option<&[String]> {
        let target = crate::catalog::find(target).map_or(target, |spec| spec.name);
        self.targets
            .get(target)
            .and_then(|target| target.mirrors.as_deref())
    }

    pub fn settings(&self) -> Settings {
        self.settings
    }

    pub fn custom_mirrors(&self) -> impl Iterator<Item = (&str, &str)> {
        self.mirrors
            .iter()
            .map(|(name, url)| (name.as_str(), url.as_str()))
    }

    pub fn sources(&self) -> &[ConfigSource] {
        &self.sources
    }

    pub fn default_source(&self, target: &str) -> Option<&Path> {
        let target = crate::catalog::find(target).map_or(target, |spec| spec.name);
        self.origins
            .get(&format!("default:{target}"))
            .map(PathBuf::as_path)
    }

    pub fn target_source(&self, target: &str) -> Option<&Path> {
        let target = crate::catalog::find(target).map_or(target, |spec| spec.name);
        ["target.mirrors", "target.enabled", "target.default"]
            .iter()
            .find_map(|kind| self.origins.get(&format!("{kind}:{target}")))
            .map(PathBuf::as_path)
    }

    pub fn effective_json(&self) -> serde_json::Value {
        let targets = self
            .targets
            .iter()
            .map(|(name, target)| {
                let default = target.default.as_ref().map(|value| {
                    if redactable_url(value) {
                        redact_url(value)
                    } else {
                        value.clone()
                    }
                });
                let mirrors = target.mirrors.as_ref().map(|items| {
                    items
                        .iter()
                        .map(|value| {
                            if redactable_url(value) {
                                redact_url(value)
                            } else {
                                value.clone()
                            }
                        })
                        .collect::<Vec<_>>()
                });
                (
                    name.clone(),
                    serde_json::json!({
                        "default": default,
                        "default_source": self.default_source(name),
                        "enabled": target.enabled.unwrap_or(true),
                        "target_source": self.target_source(name),
                        "mirrors": mirrors,
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
                    serde_json::Value::String(if redactable_url(value) {
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
            "sources": self.sources.iter().map(|source| serde_json::json!({
                "path": source.path,
                "active": source.active,
                "loaded": source.loaded,
            })).collect::<Vec<_>>(),
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
    targets: &BTreeMap<String, TargetConfig>,
    mirrors: &BTreeMap<String, String>,
) -> io::Result<()> {
    for (target, selection) in defaults {
        let spec = crate::catalog::find(target).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown target {target} in TOML configuration"),
            )
        })?;
        validate_selection(target, selection, spec, mirrors)?;
    }
    for (target, settings) in targets {
        let Some(selections) = &settings.mirrors else {
            continue;
        };
        let spec = crate::catalog::find(target).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidData,
                format!("unknown target {target} in TOML configuration"),
            )
        })?;
        for selection in selections {
            validate_selection(target, selection, spec, mirrors)?;
        }
    }
    Ok(())
}

fn validate_selection(
    target: &str,
    selection: &str,
    spec: &crate::catalog::TargetSpec,
    mirrors: &BTreeMap<String, String>,
) -> io::Result<()> {
    let valid = mirrors.get(selection).map_or_else(
        || is_selection_url(target, selection),
        |url| is_selection_url(target, url),
    ) || (selection == "first" && !spec.mirrors.is_empty())
        || spec.mirrors.iter().any(|mirror| mirror.name == selection);
    if valid {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("unknown mirror {selection} for target {target}"),
        ))
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
                        enabled: None,
                        mirrors: None,
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
                if !matches!(field.as_str(), "default" | "enabled" | "mirrors") {
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
            let enabled = target.get("enabled").map_or(Ok(None), |value| {
                value.as_bool().ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("TOML field [targets.{key}].enabled must be a boolean"),
                    )
                }).map(Some)
            })?;
            let mirrors = target
                .get("mirrors")
                .map(|value| {
                    let values = value.as_array().ok_or_else(|| {
                        io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("TOML field [targets.{key}].mirrors must be an array of strings"),
                        )
                    })?;
                    if values.is_empty() {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!("TOML field [targets.{key}].mirrors cannot be empty"),
                        ));
                    }
                    values
                        .iter()
                        .map(|value| {
                            value.as_str().map(str::to_owned).ok_or_else(|| {
                                io::Error::new(
                                    io::ErrorKind::InvalidData,
                                    format!("TOML field [targets.{key}].mirrors must be an array of strings"),
                                )
                            })
                        })
                        .collect()
                })
                .transpose()?;
            Ok((
                key.clone(),
                TargetConfig {
                    default,
                    enabled,
                    mirrors,
                },
            ))
        })
        .collect()
}

fn settings_table(document: &toml::Table) -> io::Result<SettingsPatch> {
    let Some(value) = document.get("options") else {
        return Ok(SettingsPatch::default());
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
    let mut settings = SettingsPatch::default();
    if let Some(value) = table.get("timeout_seconds") {
        settings.timeout_seconds = Some(positive_integer(value, "timeout_seconds")?);
    }
    if let Some(value) = table.get("retries") {
        settings.retries = Some(bounded_integer(value, "retries", 10)? as u32);
    }
    if let Some(value) = table.get("cache_ttl_seconds") {
        settings.cache_ttl_seconds = Some(integer(value, "cache_ttl_seconds")?);
    }
    if let Some(value) = table.get("parallelism") {
        settings.parallelism = Some(bounded_integer(value, "parallelism", 64)? as usize);
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

pub(crate) fn is_selection_url(target: &str, value: &str) -> bool {
    let url = value.strip_prefix("sparse+").unwrap_or(value);
    is_url(url)
        && (target == "cargo" || !value.starts_with("sparse+"))
        && (target != "cargo" || !url.contains(['?', '#']))
}

fn is_config_mirror_url(value: &str) -> bool {
    value
        .strip_prefix("sparse+")
        .map_or_else(|| is_url(value), |_| is_selection_url("cargo", value))
}

fn redactable_url(value: &str) -> bool {
    let value = value.strip_prefix("sparse+").unwrap_or(value);
    value.starts_with("http://") || value.starts_with("https://")
}

pub fn redact_selection(value: &str) -> String {
    if redactable_url(value) {
        redact_url(value)
    } else {
        value.to_owned()
    }
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
    platform_default_path()
}

fn platform_default_path() -> io::Result<PathBuf> {
    dirs::config_dir()
        .map(|path| path.join("lazy-mirror/config.toml"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cannot determine config directory"))
}

fn discovered_paths(primary: &Path) -> io::Result<Vec<PathBuf>> {
    let mut paths = vec![system_config_path(), primary.to_owned()];
    paths.push(project_config_path(&env::current_dir()?));
    paths.dedup();
    Ok(paths)
}

fn project_config_path(start: &Path) -> PathBuf {
    let mut directory = start;
    loop {
        let candidate = directory.join(".lazy-mirror/config.toml");
        if candidate.is_file() {
            return candidate;
        }
        let Some(parent) = directory.parent() else {
            return start.join(".lazy-mirror/config.toml");
        };
        directory = parent;
    }
}

#[cfg(windows)]
fn system_config_path() -> PathBuf {
    PathBuf::from(r"C:\ProgramData\lazy-mirror\config.toml")
}

#[cfg(not(windows))]
fn system_config_path() -> PathBuf {
    PathBuf::from("/etc/lazy-mirror/config.toml")
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
    fn reads_named_cargo_sparse_mirrors() {
        let path = env::temp_dir().join(format!(
            "lm-config-cargo-sparse-{}.toml",
            std::process::id()
        ));
        fs::write(
            &path,
            "[mirrors]\ninternal = \"sparse+https://mirror.example/index/\"\n[defaults]\ncargo = \"internal\"\n",
        )
        .unwrap();
        let config = Config::load(Some(&path)).unwrap();
        assert_eq!(
            config.mirror("internal"),
            Some("sparse+https://mirror.example/index/")
        );
        fs::remove_file(path).unwrap();
    }

    #[test]
    fn reads_options_and_nested_target_settings() {
        let path = env::temp_dir().join(format!("lm-config-options-{}.toml", std::process::id()));
        fs::write(
            &path,
            "[mirrors]\ncorp = \"https://mirror.example/simple\"\n[targets.pip]\ndefault = \"corp\"\nenabled = false\nmirrors = [\"corp\", \"tuna\"]\n[options]\ntimeout_seconds = 3\nretries = 2\ncache_ttl_seconds = 60\nparallelism = 2\n",
        )
        .unwrap();
        let config = Config::load(Some(&path)).unwrap();
        assert_eq!(config.default_for("pip"), Some("corp"));
        assert!(!config.enabled("pip"));
        assert_eq!(
            config.mirrors_for("pip"),
            Some(["corp".to_owned(), "tuna".to_owned()].as_slice())
        );
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
    fn cargo_sparse_urls_are_strict_and_redactable() {
        assert!(is_selection_url(
            "cargo",
            "sparse+https://mirror.example/index/"
        ));
        assert!(!is_selection_url(
            "cargo",
            "sparse+https://mirror.example/index?token=secret"
        ));
        assert!(redactable_url(
            "sparse+https://mirror.example/index?token=secret"
        ));
        assert_eq!(
            redact_url("sparse+https://mirror.example/index?token=secret"),
            "sparse+https://mirror.example/index"
        );
    }

    #[test]
    fn invalid_url_selections_are_still_redacted() {
        assert_eq!(
            redact_selection("https://user:secret@example.com/index?token=secret"),
            "https://example.com/index"
        );
        assert_eq!(
            redact_selection("sparse+https://user:secret@example.com/index?token=secret"),
            "sparse+https://example.com/index"
        );
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

    #[test]
    fn layered_configs_override_only_values_they_define() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let lower = env::temp_dir().join(format!("lm-config-layer-lower-{suffix}.toml"));
        let upper = env::temp_dir().join(format!("lm-config-layer-upper-{suffix}.toml"));
        fs::write(
            &lower,
            "[mirrors]\ncorp = \"https://lower.example/simple\"\n[defaults]\npip = \"corp\"\n[targets.pip]\nenabled = false\n[options]\ntimeout_seconds = 3\nparallelism = 2\n",
        )
        .unwrap();
        fs::write(
            &upper,
            "[mirrors]\ncorp = \"https://upper.example/simple\"\n[targets.pip]\nenabled = true\n[options]\nretries = 4\n",
        )
        .unwrap();

        let config =
            Config::load_paths(vec![lower.clone(), upper.clone()], upper.clone(), true).unwrap();
        assert_eq!(config.mirror("corp"), Some("https://upper.example/simple"));
        assert_eq!(config.default_for("pip"), Some("corp"));
        assert_eq!(config.default_source("pip"), Some(lower.as_path()));
        assert_eq!(config.target_source("pip"), Some(upper.as_path()));
        assert!(config.enabled("pip"));
        assert_eq!(config.settings().timeout_seconds, 3);
        assert_eq!(config.settings().retries, 4);
        assert_eq!(config.settings().parallelism, 2);
        assert!(config.sources().iter().all(|source| source.loaded));
        fs::remove_file(lower).unwrap();
        fs::remove_file(upper).unwrap();
    }

    #[test]
    fn project_config_uses_the_nearest_parent() {
        let root = env::temp_dir().join(format!(
            "lm-config-project-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let nested = root.join("packages/example");
        fs::create_dir_all(&nested).unwrap();
        fs::create_dir_all(root.join(".lazy-mirror")).unwrap();
        let root_config = root.join(".lazy-mirror/config.toml");
        fs::write(&root_config, "[defaults]\npip = \"tuna\"\n").unwrap();
        assert_eq!(project_config_path(&nested), root_config);

        let nested_config = nested.join(".lazy-mirror/config.toml");
        fs::create_dir_all(nested_config.parent().unwrap()).unwrap();
        fs::write(&nested_config, "[defaults]\npip = \"ustc\"\n").unwrap();
        assert_eq!(project_config_path(&nested), nested_config);
        fs::remove_dir_all(root).unwrap();
    }
}
