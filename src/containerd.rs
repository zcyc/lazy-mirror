use std::io;
use std::path::PathBuf;

use crate::config::Scope;

const PREFIX: &str =
    "# managed by lazy-mirror\nserver = \"https://registry-1.docker.io\"\n\n[host.\"";
const SUFFIX: &str = "\"]\n  capabilities = [\"pull\"]\n";

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    crate::write_with_backup_if(
        &path(scope)?,
        &format!("{PREFIX}{mirror}{SUFFIX}"),
        |content| content.starts_with(PREFIX),
    )
}

pub fn unset(scope: Scope) -> io::Result<()> {
    crate::remove_with_backup_if(&path(scope)?, |content| content.starts_with(PREFIX))
}

pub fn status(scope: Scope) -> io::Result<crate::ToolStatus> {
    let command = ["containerd", "nerdctl"]
        .into_iter()
        .find(|command| crate::command_exists(command))
        .ok_or_else(|| crate::missing_commands(&["containerd", "nerdctl"]))?;
    let version = crate::command_version(command)?;
    let path = path(scope)?;
    let content = crate::read_optional(&path)?;
    let configured = content
        .as_deref()
        .is_some_and(|content| content.starts_with(PREFIX));
    let source = content
        .as_deref()
        .and_then(|content| content.strip_prefix(PREFIX))
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

fn path(scope: Scope) -> io::Result<PathBuf> {
    if scope == Scope::Project {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "containerd does not support project scope",
        ));
    }
    if let Some(path) = std::env::var_os("LM_CONTAINERD_HOSTS_FILE") {
        return Ok(path.into());
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
