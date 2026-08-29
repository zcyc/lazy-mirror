use std::io;
use std::path::PathBuf;

const PREFIX: &str = "# managed by lazy-mirror\n[repositories]\n  local\n  lazy-mirror: ";

pub fn set(mirror: &str) -> io::Result<()> {
    let path = config_path()?;
    crate::write_with_backup_if(&path, &format!("{PREFIX}{mirror}\n"), |content| {
        content.starts_with(PREFIX)
    })
}

pub fn unset() -> io::Result<()> {
    crate::remove_with_backup_if(&config_path()?, |content| content.starts_with(PREFIX))
}

pub fn status(expected: &str) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("sbt")?;
    let path = config_path()?;
    let configured = std::fs::read_to_string(&path)
        .map(|content| content.contains(expected))
        .unwrap_or(false);
    Ok(crate::ToolStatus {
        configured,
        detail: format!("config={}", path.display()),
        version,
    })
}

fn config_path() -> io::Result<PathBuf> {
    if let Some(path) = std::env::var_os("LM_SBT_REPOSITORIES") {
        return Ok(PathBuf::from(path));
    }
    crate::home_file(".sbt/repositories")
}
