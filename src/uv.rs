use std::io;
use std::path::PathBuf;

use crate::config::Scope;

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    let path = config_path(scope)?;
    let url = mirror.trim_end_matches('/').to_owned() + "/";
    crate::update_toml(
        &path,
        |document| {
            let indexes = document
                .entry("index")
                .or_insert_with(|| toml::Value::Array(Vec::new()))
                .as_array_mut()
                .ok_or_else(|| {
                    io::Error::new(io::ErrorKind::InvalidData, "uv index must be an array")
                })?;
            indexes.retain(|value| {
                value.get("name").and_then(toml::Value::as_str) != Some("lazy-mirror")
            });
            let mut index = toml::Table::new();
            index.insert(
                "name".to_owned(),
                toml::Value::String("lazy-mirror".to_owned()),
            );
            index.insert("url".to_owned(), toml::Value::String(url));
            index.insert("default".to_owned(), toml::Value::Boolean(true));
            indexes.push(toml::Value::Table(index));
            Ok(())
        },
        is_managed,
    )
}

pub fn unset(scope: Scope) -> io::Result<()> {
    crate::remove_toml_with_backup(&config_path(scope)?, is_managed)
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("uv")?;
    let path = config_path(scope)?;
    let source = std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| content.parse::<toml::Table>().ok())
        .and_then(|document| index_url(&document).map(str::to_owned));
    let configured = source
        .as_deref()
        .is_some_and(|url| url.trim_end_matches('/') == expected.trim_end_matches('/'));
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
        Scope::Project => std::env::current_dir().map(|path| path.join("uv.toml")),
        Scope::User => crate::home_file(".config/uv/uv.toml"),
        Scope::System => Ok(PathBuf::from("/etc/uv/uv.toml")),
    }
}

fn is_managed(document: &toml::Table) -> bool {
    index_url(document).is_some()
}

fn index_url(document: &toml::Table) -> Option<&str> {
    document
        .get("index")
        .and_then(toml::Value::as_array)
        .and_then(|indexes| {
            indexes.iter().find_map(|index| {
                (index.get("name").and_then(toml::Value::as_str) == Some("lazy-mirror"))
                    .then(|| index.get("url").and_then(toml::Value::as_str))
                    .flatten()
            })
        })
}
