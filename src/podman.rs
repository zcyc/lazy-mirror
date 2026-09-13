use std::io;
use std::path::PathBuf;

use crate::config::Scope;

const PREFIX: &str = "# managed by lazy-mirror\nunqualified-search-registries = [\"docker.io\"]\n\n[[registry]]\nlocation = \"docker.io\"\n\n[[registry.mirror]]\nlocation = \"";

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    crate::write_with_backup_if(&path(scope)?, &format!("{PREFIX}{mirror}\"\n"), |content| {
        content.starts_with(PREFIX)
    })
}

pub fn unset(scope: Scope) -> io::Result<()> {
    crate::remove_with_backup_if(&path(scope)?, |content| content.starts_with(PREFIX))
}

pub fn status(scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("podman")?;
    let path = path(scope)?;
    let content = crate::read_optional(&path)?;
    let configured = content
        .as_deref()
        .is_some_and(|content| content.starts_with(PREFIX));
    let source = content
        .as_deref()
        .and_then(|content| content.strip_prefix(PREFIX))
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

fn path(scope: Scope) -> io::Result<PathBuf> {
    if scope == Scope::Project {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Podman does not support project scope",
        ));
    }
    if let Some(path) = std::env::var_os("LM_PODMAN_CONFIG") {
        return Ok(path.into());
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
    use super::PREFIX;

    #[test]
    fn uses_the_registries_v2_mirror_table() {
        let content = format!("{PREFIX}https://mirror.example\"\n");
        assert!(content.contains("[[registry.mirror]]\nlocation = \""));
        assert!(!content.contains("\nmirror = \""));
    }
}
