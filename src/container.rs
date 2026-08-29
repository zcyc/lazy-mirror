use std::io;
use std::path::PathBuf;

const CONTAINERD_PREFIX: &str =
    "# managed by lazy-mirror\nserver = \"https://registry-1.docker.io\"\n\n[host.\"";
const CONTAINERD_SUFFIX: &str = "\"]\n  capabilities = [\"pull\"]\n";
const PODMAN_PREFIX: &str = "# managed by lazy-mirror\nunqualified-search-registries = [\"docker.io\"]\n\n[[registry]]\nlocation = \"docker.io\"\n";

pub fn containerd_set(mirror: &str) -> io::Result<()> {
    let path = containerd_path()?;
    crate::write_with_backup_if(
        &path,
        &format!("{CONTAINERD_PREFIX}{mirror}{CONTAINERD_SUFFIX}"),
        |content| content.starts_with(CONTAINERD_PREFIX),
    )
}

pub fn containerd_unset() -> io::Result<()> {
    crate::remove_with_backup_if(&containerd_path()?, |content| {
        content.starts_with(CONTAINERD_PREFIX)
    })
}

pub fn containerd_status(command: &str) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version(command)?;
    let path = containerd_path()?;
    let configured = std::fs::read_to_string(&path)
        .map(|content| content.starts_with(CONTAINERD_PREFIX))
        .unwrap_or(false);
    Ok(crate::ToolStatus {
        configured,
        detail: format!("config={}", path.display()),
        version,
    })
}

pub fn podman_set(mirror: &str) -> io::Result<()> {
    let path = podman_path()?;
    crate::write_with_backup_if(
        &path,
        &format!("{PODMAN_PREFIX}mirror = \"{mirror}\"\n"),
        |content| content.starts_with(PODMAN_PREFIX),
    )
}

pub fn podman_unset() -> io::Result<()> {
    crate::remove_with_backup_if(&podman_path()?, |content| {
        content.starts_with(PODMAN_PREFIX)
    })
}

pub fn podman_status() -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("podman")?;
    let path = podman_path()?;
    let configured = std::fs::read_to_string(&path)
        .map(|content| content.starts_with(PODMAN_PREFIX))
        .unwrap_or(false);
    Ok(crate::ToolStatus {
        configured,
        detail: format!("config={}", path.display()),
        version,
    })
}

fn containerd_path() -> io::Result<PathBuf> {
    if let Some(path) = std::env::var_os("LM_CONTAINERD_HOSTS_FILE") {
        return Ok(PathBuf::from(path));
    }
    Ok(PathBuf::from(
        "/etc/containerd/certs.d/docker.io/hosts.toml",
    ))
}

fn podman_path() -> io::Result<PathBuf> {
    if let Some(path) = std::env::var_os("LM_PODMAN_CONFIG") {
        return Ok(PathBuf::from(path));
    }
    Ok(PathBuf::from("/etc/containers/registries.conf"))
}
