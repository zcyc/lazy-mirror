use std::io;
use std::path::PathBuf;

use crate::config::Scope;

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    let path = config_path(scope)?;
    let url = mirror.trim_end_matches('/').to_owned() + "/";
    crate::update_toml(
        &path,
        |document| set_index(document, &path, &url),
        |document| is_managed(document, &path),
    )
}

fn set_index(document: &mut toml::Table, path: &std::path::Path, url: &str) -> io::Result<()> {
    let document = uv_document_mut(document, path)?;
    let indexes = document
        .entry("index")
        .or_insert_with(|| toml::Value::Array(Vec::new()))
        .as_array_mut()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "uv index must be an array"))?;
    indexes.retain(|value| value.get("name").and_then(toml::Value::as_str) != Some("lazy-mirror"));
    for index in indexes.iter_mut().filter_map(toml::Value::as_table_mut) {
        if index
            .get("default")
            .and_then(toml::Value::as_bool)
            .unwrap_or(false)
        {
            index.insert("default".to_owned(), toml::Value::Boolean(false));
        }
    }
    let mut index = toml::Table::new();
    index.insert(
        "name".to_owned(),
        toml::Value::String("lazy-mirror".to_owned()),
    );
    index.insert("url".to_owned(), toml::Value::String(url.to_owned()));
    index.insert("default".to_owned(), toml::Value::Boolean(true));
    indexes.push(toml::Value::Table(index));
    Ok(())
}

pub fn unset(scope: Scope) -> io::Result<()> {
    let path = config_path(scope)?;
    crate::remove_toml_with_backup(&path, |document| is_managed(document, &path))
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("uv")?;
    let path = config_path(scope)?;
    let source = std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| content.parse::<toml::Table>().ok())
        .and_then(|document| index_url(&document, &path).map(str::to_owned));
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
        Scope::Project => {
            let current = std::env::current_dir()?;
            Ok(project_config_path(&current))
        }
        Scope::User => crate::home_file(".config/uv/uv.toml"),
        Scope::System => Ok(crate::system_file(
            r"C:\ProgramData\uv\uv.toml",
            "/etc/uv/uv.toml",
        )),
    }
}

fn project_config_path(start: &std::path::Path) -> PathBuf {
    let root = workspace_root(start).unwrap_or_else(|| start.to_owned());
    let mut directory = root.as_path();
    loop {
        let uv = directory.join("uv.toml");
        if uv.is_file() {
            return uv;
        }
        let path = directory.join("pyproject.toml");
        if has_uv_table(&path) {
            return path;
        }
        let Some(parent) = directory.parent().filter(|parent| *parent != root) else {
            return root.join("uv.toml");
        };
        directory = parent;
    }
}

fn workspace_root(start: &std::path::Path) -> Option<PathBuf> {
    let mut directory = start;
    loop {
        let path = directory.join("pyproject.toml");
        if let Ok(content) = std::fs::read_to_string(&path) {
            if let Ok(document) = content.parse::<toml::Table>() {
                let is_workspace = uv_document(&document, &path)
                    .and_then(|document| document.get("workspace"))
                    .is_some_and(toml::Value::is_table);
                if is_workspace {
                    return Some(directory.to_owned());
                }
            }
        }
        let parent = directory.parent()?;
        directory = parent;
    }
}

fn has_uv_table(path: &std::path::Path) -> bool {
    std::fs::read_to_string(path)
        .ok()
        .and_then(|content| content.parse::<toml::Table>().ok())
        .is_some_and(|document| uv_document(&document, path).is_some())
}

fn is_pyproject(path: &std::path::Path) -> bool {
    path.file_name().and_then(|name| name.to_str()) == Some("pyproject.toml")
}

fn uv_document<'a>(document: &'a toml::Table, path: &std::path::Path) -> Option<&'a toml::Table> {
    if is_pyproject(path) {
        document
            .get("tool")
            .and_then(toml::Value::as_table)
            .and_then(|tool| tool.get("uv"))
            .and_then(toml::Value::as_table)
    } else {
        Some(document)
    }
}

fn uv_document_mut<'a>(
    document: &'a mut toml::Table,
    path: &std::path::Path,
) -> io::Result<&'a mut toml::Table> {
    if is_pyproject(path) {
        document
            .entry("tool")
            .or_insert_with(|| toml::Value::Table(toml::Table::new()))
            .as_table_mut()
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "tool must be a TOML table"))?
            .entry("uv")
            .or_insert_with(|| toml::Value::Table(toml::Table::new()))
            .as_table_mut()
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "tool.uv must be a TOML table")
            })
    } else {
        Ok(document)
    }
}

fn managed_index_url<'a>(document: &'a toml::Table, path: &std::path::Path) -> Option<&'a str> {
    uv_document(document, path)?
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

fn is_managed(document: &toml::Table, path: &std::path::Path) -> bool {
    managed_index_url(document, path).is_some()
}

fn index_url<'a>(document: &'a toml::Table, path: &std::path::Path) -> Option<&'a str> {
    let document = uv_document(document, path)?;
    let indexes = document.get("index").and_then(toml::Value::as_array);
    indexes
        .and_then(|indexes| {
            indexes
                .iter()
                .filter_map(toml::Value::as_table)
                .find_map(|index| {
                    (index.get("name").and_then(toml::Value::as_str) == Some("lazy-mirror"))
                        .then(|| index.get("url").and_then(toml::Value::as_str))
                        .flatten()
                })
        })
        .or_else(|| {
            indexes.and_then(|indexes| {
                indexes
                    .iter()
                    .filter_map(toml::Value::as_table)
                    .find_map(|index| {
                        index
                            .get("default")
                            .and_then(toml::Value::as_bool)
                            .unwrap_or(false)
                            .then(|| index.get("url").and_then(toml::Value::as_str))
                            .flatten()
                    })
            })
        })
        .or_else(|| {
            indexes.and_then(|indexes| {
                indexes
                    .iter()
                    .filter_map(toml::Value::as_table)
                    .find_map(|index| index.get("url").and_then(toml::Value::as_str))
            })
        })
        .or_else(|| document.get("default-index").and_then(toml::Value::as_str))
        .or_else(|| document.get("index-url").and_then(toml::Value::as_str))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_config_prefers_uv_toml_then_pyproject() {
        let root = std::env::temp_dir().join(format!(
            "lm-uv-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let nested = root.join("packages/example");
        std::fs::create_dir_all(&nested).unwrap();
        let pyproject = root.join("pyproject.toml");
        std::fs::write(&pyproject, "[tool.uv]\n").unwrap();
        assert_eq!(project_config_path(&nested), pyproject);

        let uv = nested.join("uv.toml");
        std::fs::write(&uv, "[[index]]\nurl = \"https://example.com/simple\"\n").unwrap();
        assert_eq!(project_config_path(&nested), uv);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn pyproject_uv_table_is_read_and_written_at_the_right_path() {
        let root = std::env::temp_dir().join(format!(
            "lm-uv-pyproject-{}-{}.toml",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let path = root.join("pyproject.toml");
        let mut document = "[project]\nname = \"demo\"\n[tool.uv]\n[[tool.uv.index]]\nname = \"lazy-mirror\"\nurl = \"https://example.com/simple/\"\n".parse::<toml::Table>().unwrap();
        assert_eq!(
            index_url(&document, &path),
            Some("https://example.com/simple/")
        );
        let uv = uv_document_mut(&mut document, &path).unwrap();
        assert!(uv.contains_key("index"));
        assert_eq!(
            managed_index_url(&document, &path),
            Some("https://example.com/simple/")
        );
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn workspace_config_starts_at_the_workspace_root() {
        let root = std::env::temp_dir().join(format!(
            "lm-uv-workspace-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let member = root.join("packages/example");
        std::fs::create_dir_all(&member).unwrap();
        std::fs::write(
            root.join("pyproject.toml"),
            "[tool.uv.workspace]\nmembers = [\"packages/*\"]\n[tool.uv]\n",
        )
        .unwrap();
        std::fs::write(member.join("uv.toml"), "[[index]]\n").unwrap();
        assert_eq!(project_config_path(&member), root.join("pyproject.toml"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn setting_uv_mirror_preserves_named_indexes_and_replaces_default() {
        let mut document = r#"
            [[index]]
            name = "pytorch"
            url = "https://download.pytorch.org/whl/cpu"
            explicit = true

            [[index]]
            name = "public"
            url = "https://pypi.org/simple"
            default = true
        "#
        .parse::<toml::Table>()
        .unwrap();
        set_index(
            &mut document,
            std::path::Path::new("uv.toml"),
            "https://mirror.example/simple/",
        )
        .unwrap();
        let indexes = document["index"].as_array().unwrap();
        assert_eq!(indexes.len(), 3);
        assert_eq!(indexes[0]["name"].as_str(), Some("pytorch"));
        assert_eq!(indexes[1]["default"].as_bool(), Some(false));
        assert_eq!(indexes[2]["name"].as_str(), Some("lazy-mirror"));
        assert_eq!(indexes[2]["default"].as_bool(), Some(true));
    }
}
