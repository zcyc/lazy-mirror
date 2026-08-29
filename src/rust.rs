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
    let configured = std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| content.parse::<toml::Table>().ok())
        .map(|document| {
            registry(&document).is_some_and(|value| {
                value.trim_start_matches("sparse+") == expected.trim_end_matches('/')
            })
        })
        .unwrap_or(false);
    Ok(crate::ToolStatus {
        configured,
        detail: format!("config={}", path.display()),
        version,
    })
}

fn config_path(scope: Scope) -> io::Result<PathBuf> {
    match scope {
        Scope::Project => std::env::current_dir().map(|path| path.join(".cargo/config.toml")),
        Scope::User => Ok(cargo_home()?.join("config.toml")),
        Scope::System => Ok(PathBuf::from("/etc/cargo/config.toml")),
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
