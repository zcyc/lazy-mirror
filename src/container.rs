use std::io;
use std::path::PathBuf;

use crate::config::Scope;

const CONTAINERD_PREFIX: &str =
    "# managed by lazy-mirror\nserver = \"https://registry-1.docker.io\"\n\n[host.\"";
const CONTAINERD_SUFFIX: &str = "\"]\n  capabilities = [\"pull\"]\n";
const PODMAN_PREFIX: &str = "# managed by lazy-mirror\nunqualified-search-registries = [\"docker.io\"]\n\n[[registry]]\nlocation = \"docker.io\"\n\n[[registry.mirror]]\nlocation = \"";

pub fn containerd_set(mirror: &str, scope: Scope) -> io::Result<()> {
    let path = containerd_path(scope)?;
    crate::write_with_backup_if(
        &path,
        &format!("{CONTAINERD_PREFIX}{mirror}{CONTAINERD_SUFFIX}"),
        |content| content.starts_with(CONTAINERD_PREFIX),
    )
}

pub fn containerd_unset(scope: Scope) -> io::Result<()> {
    crate::remove_with_backup_if(&containerd_path(scope)?, |content| {
        content.starts_with(CONTAINERD_PREFIX)
    })
}

pub fn containerd_status(command: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version(command)?;
    let path = containerd_path(scope)?;
    let content = std::fs::read_to_string(&path).ok();
    let configured = content
        .as_deref()
        .is_some_and(|content| content.starts_with(CONTAINERD_PREFIX));
    let source = content
        .as_deref()
        .and_then(|content| content.strip_prefix(CONTAINERD_PREFIX))
        .and_then(|content| content.split_once("\"]").map(|(mirror, _)| mirror));
    Ok(crate::ToolStatus::new(
        version,
        configured,
        source.map(str::to_owned),
        Some(path.clone()),
        format!(
            "source={}; config={}",
            source.unwrap_or("not configured"),
            path.display()
        ),
    ))
}

pub fn podman_set(mirror: &str, scope: Scope) -> io::Result<()> {
    let path = podman_path(scope)?;
    crate::write_with_backup_if(&path, &format!("{PODMAN_PREFIX}{mirror}\"\n"), |content| {
        content.starts_with(PODMAN_PREFIX)
    })
}

pub fn podman_unset(scope: Scope) -> io::Result<()> {
    crate::remove_with_backup_if(&podman_path(scope)?, |content| {
        content.starts_with(PODMAN_PREFIX)
    })
}

pub fn podman_status(scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("podman")?;
    let path = podman_path(scope)?;
    let content = std::fs::read_to_string(&path).ok();
    let configured = content
        .as_deref()
        .is_some_and(|content| content.starts_with(PODMAN_PREFIX));
    let source = content
        .as_deref()
        .and_then(|content| content.strip_prefix(PODMAN_PREFIX))
        .and_then(|content| content.split_once('"').map(|(mirror, _)| mirror));
    Ok(crate::ToolStatus::new(
        version,
        configured,
        source.map(str::to_owned),
        Some(path.clone()),
        format!(
            "source={}; config={}",
            source.unwrap_or("not configured"),
            path.display()
        ),
    ))
}

fn containerd_path(scope: Scope) -> io::Result<PathBuf> {
    if scope == Scope::Project {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "containerd does not support project scope",
        ));
    }
    if let Some(path) = std::env::var_os("LM_CONTAINERD_HOSTS_FILE") {
        return Ok(PathBuf::from(path));
    }
    match scope {
        Scope::Project => unreachable!(),
        Scope::User => crate::home_file(".config/containerd/certs.d/docker.io/hosts.toml"),
        Scope::System => Ok(crate::system_file(
            r"C:\ProgramData\containerd\certs.d\docker.io\hosts.toml",
            "/etc/containerd/certs.d/docker.io/hosts.toml",
        )),
    }
}

fn podman_path(scope: Scope) -> io::Result<PathBuf> {
    if scope == Scope::Project {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Podman does not support project scope",
        ));
    }
    if let Some(path) = std::env::var_os("LM_PODMAN_CONFIG") {
        return Ok(PathBuf::from(path));
    }
    match scope {
        Scope::Project => unreachable!(),
        Scope::User => crate::home_file(".config/containers/registries.conf"),
        Scope::System => Ok(crate::system_file(
            r"C:\ProgramData\containers\registries.conf",
            "/etc/containers/registries.conf",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn podman_uses_the_registries_v2_mirror_table() {
        let content = format!("{PODMAN_PREFIX}https://mirror.example\"\n");
        assert!(content.contains("[[registry.mirror]]\nlocation = \""));
        assert!(!content.contains("\nmirror = \""));
    }
}
