use std::io;

const PREFIX: &str = "# managed by lazy-mirror\noptions(repos = c(CRAN = \"";
const SUFFIX: &str = "\"))\n";

pub fn set(mirror: &str) -> io::Result<()> {
    let path = crate::home_file(".Rprofile")?;
    crate::write_with_backup_if(&path, &format!("{PREFIX}{mirror}{SUFFIX}"), |content| {
        content.starts_with(PREFIX)
    })
}

pub fn unset() -> io::Result<()> {
    crate::remove_with_backup_if(&crate::home_file(".Rprofile")?, |content| {
        content.starts_with(PREFIX)
    })
}

pub fn status(expected: &str) -> io::Result<crate::ToolStatus> {
    let version = crate::command_output("R", &["--version"])?;
    let path = crate::home_file(".Rprofile")?;
    let source = std::fs::read_to_string(&path).ok().and_then(|content| {
        content
            .strip_prefix(PREFIX)
            .and_then(|value| value.strip_suffix(SUFFIX))
            .map(str::to_owned)
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
