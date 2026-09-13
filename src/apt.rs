use std::io;
use std::path::PathBuf;

use crate::config::Scope;
use crate::shared::{apt_distribution, require_system};

const PREFIX: &str = "# managed by lazy-mirror\n";

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    require_system("apt", scope)?;
    let path = path()?;
    let content = if mirror.starts_with("deb ") || mirror.starts_with("deb-src ") {
        format!("{PREFIX}{mirror}\n")
    } else {
        let suites = std::env::var("LM_APT_SUITES")
            .map(|value| value.split_whitespace().map(str::to_owned).collect())
            .unwrap_or_else(|_| vec![apt_distribution()]);
        let components = std::env::var("LM_APT_COMPONENTS").unwrap_or_else(|_| "main".to_owned());
        let lines = suites
            .iter()
            .map(|suite| format!("deb {mirror} {suite} {components}"))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{PREFIX}{lines}\n")
    };
    crate::write_with_backup_if(&path, &content, |content| content.starts_with(PREFIX))
}

pub fn unset(scope: Scope) -> io::Result<()> {
    require_system("apt", scope)?;
    crate::remove_with_backup_if(&path()?, |content| content.starts_with(PREFIX))
}

pub fn status(scope: Scope) -> io::Result<crate::ToolStatus> {
    require_system("apt", scope)?;
    let path = path()?;
    let content = crate::read_optional(&path)?;
    let source = content.as_deref().and_then(|content| {
        content
            .strip_prefix(PREFIX)
            .map(str::trim)
            .map(str::to_owned)
    });
    Ok(crate::ToolStatus::new(
        crate::command_version("apt")?,
        source.is_some(),
        source.clone(),
        Some(path.clone()),
        format!(
            "source={}; config={}",
            source.unwrap_or_else(|| "not configured".to_owned()),
            path.display()
        ),
    ))
}

fn path() -> io::Result<PathBuf> {
    Ok(std::env::var_os("LM_APT_SOURCES_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/etc/apt/sources.list.d/lazy-mirror.list")))
}
