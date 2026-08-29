use std::io;
use std::path::PathBuf;

use crate::config::Scope;

const REPOSITORY_NAME: &str = "lazy-mirror";
const CONFIG_ENV: &str = "LM_HELM_REPOSITORY_CONFIG";

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    let path = repository_config(scope)?;
    let path = path.to_string_lossy().into_owned();
    validate_repository_url(mirror)?;
    crate::run(
        "helm",
        &[
            "repo",
            "add",
            REPOSITORY_NAME,
            mirror,
            "--force-update",
            "--repository-config",
            &path,
        ],
    )
}

pub fn unset(scope: Scope) -> io::Result<()> {
    let path = repository_config(scope)?;
    let path = path.to_string_lossy().into_owned();
    crate::run(
        "helm",
        &[
            "repo",
            "remove",
            REPOSITORY_NAME,
            "--repository-config",
            &path,
        ],
    )
}

pub fn status(scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("helm")?;
    let path = repository_config(scope)?;
    let path_string = path.to_string_lossy().into_owned();
    let detail = crate::command_output(
        "helm",
        &[
            "repo",
            "list",
            "--output",
            "json",
            "--repository-config",
            &path_string,
        ],
    )
    .unwrap_or_default();
    let source = serde_json::from_str::<serde_json::Value>(&detail)
        .ok()
        .and_then(|items| items.as_array().cloned())
        .and_then(|items| {
            items.into_iter().find_map(|item| {
                (item.get("name").and_then(serde_json::Value::as_str) == Some(REPOSITORY_NAME))
                    .then(|| item.get("url").and_then(serde_json::Value::as_str))
                    .flatten()
                    .map(str::to_owned)
            })
        });
    Ok(crate::ToolStatus::new(
        version,
        source.is_some(),
        source,
        Some(path.clone()),
        format!("repository={REPOSITORY_NAME}; config={}", path.display()),
    ))
}

fn repository_config(scope: Scope) -> io::Result<PathBuf> {
    if scope != Scope::User {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Helm supports user scope only; use a repository config override for another location",
        ));
    }
    if let Some(path) = std::env::var_os(CONFIG_ENV) {
        if path.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("{CONFIG_ENV} cannot be empty"),
            ));
        }
        return Ok(path.into());
    }
    crate::home_file(".config/helm/repositories.yaml")
}

fn validate_repository_url(mirror: &str) -> io::Result<()> {
    if mirror.starts_with("http://") || mirror.starts_with("https://") {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Helm repository mirrors must be HTTP(S) URLs; OCI uses explicit oci:// references",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn helm_accepts_chart_repositories_but_not_oci_references() {
        assert!(validate_repository_url("https://charts.example.com").is_ok());
        assert!(validate_repository_url("oci://registry.example.com/charts").is_err());
    }
}
