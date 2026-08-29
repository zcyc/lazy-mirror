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
    let source = std::fs::read_to_string(&path).ok().and_then(|content| {
        content
            .strip_prefix(PREFIX)
            .map(|value| value.trim().to_owned())
    });
    let configured = source.as_deref().is_some_and(|value| value == expected);
    Ok(crate::ToolStatus {
        configured,
        detail: format!(
            "source={}; config={}",
            source.unwrap_or_else(|| "not configured".to_owned()),
            path.display()
        ),
        version,
    })
}

fn config_path() -> io::Result<PathBuf> {
    if let Some(path) = std::env::var_os("LM_SBT_REPOSITORIES") {
        return Ok(PathBuf::from(path));
    }
    crate::home_file(".sbt/repositories")
}
