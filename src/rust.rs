use std::io;
use std::path::PathBuf;

use crate::config::Scope;

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    let path = config_path(scope)?;
    let registry = if mirror.starts_with("sparse+") {
        mirror.to_owned()
    } else {
        format!("sparse+{}", mirror.trim_end_matches('/'))
    };
    crate::update_toml(
        &path,
        |document| {
            let source = document
                .entry("source")
                .or_insert_with(|| toml::Value::Table(toml::Table::new()))
                .as_table_mut()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "source must be a TOML table")
                })?;
            let crates_io = source
                .entry("crates-io")
                .or_insert_with(|| toml::Value::Table(toml::Table::new()))
                .as_table_mut()
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "source.crates-io must be a TOML table",
                    )
                })?;
            crates_io.insert(
                "replace-with".to_owned(),
                toml::Value::String("lazy-mirror".to_owned()),
            );
            let mirror_source = source
                .entry("lazy-mirror")
                .or_insert_with(|| toml::Value::Table(toml::Table::new()))
                .as_table_mut()
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "source.lazy-mirror must be a TOML table",
                    )
                })?;
            mirror_source.insert("registry".to_owned(), toml::Value::String(registry));
            Ok(())
        },
        is_managed,
    )
}

pub fn unset(scope: Scope) -> io::Result<()> {
    crate::remove_toml_with_backup(&config_path(scope)?, is_managed)
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("cargo")?;
    let path = config_path(scope)?;
    let source = std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| content.parse::<toml::Table>().ok())
        .and_then(|document| registry(&document).map(str::to_owned));
    let configured = source
        .as_deref()
        .is_some_and(|value| value.trim_start_matches("sparse+") == expected.trim_end_matches('/'));
    Ok(crate::ToolStatus::new(
        version,
        configured,
        source.clone(),
        Some(path.clone()),
        format!(
            "source={}; config={}",
            source.unwrap_or_else(|| "not configured".to_owned()),
            path.display()
        ),
    ))
}

fn config_path(scope: Scope) -> io::Result<PathBuf> {
    match scope {
        Scope::Project => {
            let current = std::env::current_dir()?;
            Ok(project_config_path(&current))
        }
        Scope::User => Ok(preferred_config_path(
            &cargo_home()?,
            "config.toml",
            "config",
        )),
        Scope::System => Ok(preferred_config_path(
            std::path::Path::new("/etc/cargo"),
            "config.toml",
            "config",
        )),
    }
}

fn project_config_path(start: &std::path::Path) -> PathBuf {
    crate::nearest_existing_file(start, &[".cargo/config.toml", ".cargo/config"])
        .unwrap_or_else(|| start.join(".cargo/config.toml"))
}

fn preferred_config_path(directory: &std::path::Path, modern: &str, legacy: &str) -> PathBuf {
    let modern = directory.join(modern);
    if modern.is_file() {
        modern
    } else if directory.join(legacy).is_file() {
        directory.join(legacy)
    } else {
        modern
    }
}

fn cargo_home() -> io::Result<PathBuf> {
    if let Some(path) = std::env::var_os("CARGO_HOME") {
        return Ok(PathBuf::from(path));
    }

    dirs::home_dir()
        .map(|home| home.join(".cargo"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cannot determine home directory"))
}

fn is_managed(document: &toml::Table) -> bool {
    registry(document).is_some_and(|_| {
        document
            .get("source")
            .and_then(toml::Value::as_table)
            .and_then(|source| source.get("crates-io"))
            .and_then(toml::Value::as_table)
            .and_then(|crates_io| crates_io.get("replace-with"))
            .and_then(toml::Value::as_str)
            == Some("lazy-mirror")
    })
}

fn registry(document: &toml::Table) -> Option<&str> {
    document
        .get("source")
        .and_then(toml::Value::as_table)
        .and_then(|source| source.get("lazy-mirror"))
        .and_then(toml::Value::as_table)
        .and_then(|mirror| mirror.get("registry"))
        .and_then(toml::Value::as_str)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_config_uses_nearest_cargo_config_and_prefers_toml() {
        let root = std::env::temp_dir().join(format!(
            "lm-cargo-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let nested = root.join("packages/example");
        std::fs::create_dir_all(&nested).unwrap();
        let legacy = root.join(".cargo/config");
        std::fs::create_dir_all(legacy.parent().unwrap()).unwrap();
        std::fs::write(&legacy, "").unwrap();
        assert_eq!(project_config_path(&nested), legacy);

        let modern = root.join(".cargo/config.toml");
        std::fs::write(&modern, "").unwrap();
        assert_eq!(project_config_path(&nested), modern);
        std::fs::remove_dir_all(root).unwrap();
    }
}
