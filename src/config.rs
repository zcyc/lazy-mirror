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
        let defaults = if document.contains_key("defaults") {
            string_table(&document, "defaults")?
        } else {
            string_table(&document, "targets")?
        };
        Ok(Self {
            path,
            mirrors: string_table(&document, "mirrors")?,
            defaults,
        })
    }

    pub fn mirror(&self, name: &str) -> Option<&str> {
        self.mirrors.get(name).map(String::as_str)
    }

    pub fn default_for(&self, target: &str) -> Option<&str> {
        self.defaults.get(target).map(String::as_str)
    }

    pub fn custom_mirrors(&self) -> impl Iterator<Item = (&str, &str)> {
        self.mirrors
            .iter()
            .map(|(name, url)| (name.as_str(), url.as_str()))
    }
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
            let value = value
                .as_str()
                .or_else(|| {
                    value
                        .as_table()
                        .and_then(|table| table.get("url"))
                        .and_then(toml::Value::as_str)
                })
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        format!("TOML value [{name}].{key} must be a string"),
                    )
                })?;
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
}
