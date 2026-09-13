use std::io;
use std::path::PathBuf;

use crate::config::Scope;
use crate::shared::require_system;

const PREFIX: &str = "# managed by lazy-mirror\n";

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    require_system("apk", scope)?;
    crate::write_with_backup_if(&path()?, &format!("{PREFIX}{mirror}\n"), |content| {
        content.starts_with(PREFIX)
    })
}

pub fn unset(scope: Scope) -> io::Result<()> {
    require_system("apk", scope)?;
    crate::remove_with_backup_if(&path()?, |content| content.starts_with(PREFIX))
}

pub fn status(scope: Scope) -> io::Result<crate::ToolStatus> {
    require_system("apk", scope)?;
    let path = path()?;
    let content = crate::read_optional(&path)?;
    let source = content.as_deref().and_then(|content| {
        content
            .strip_prefix(PREFIX)
            .map(str::trim)
            .map(str::to_owned)
    });
    Ok(crate::ToolStatus::new(
        crate::command_version("apk")?,
        source.is_some(),
        source.clone(),
        Some(path.clone()),
        format!(
            "repositories={}; config={}",
            source.unwrap_or_else(|| "not configured".to_owned()),
            path.display()
        ),
    ))
}

fn path() -> io::Result<PathBuf> {
    Ok(std::env::var_os("LM_APK_REPOSITORIES_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/etc/apk/repositories")))
}
