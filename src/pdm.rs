use std::io;

pub fn set(mirror: &str) -> io::Result<()> {
    crate::run("pdm", &["config", "pypi.url", mirror])
}

pub fn unset() -> io::Result<()> {
    crate::run("pdm", &["config", "--delete", "pypi.url"])
}

pub fn status(expected: &str) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("pdm")?;
    let value = crate::command_output("pdm", &["config", "pypi.url"])
        .unwrap_or_else(|_| "not configured".to_owned());
    let source = (value != "not configured").then(|| value.clone());
    Ok(crate::ToolStatus::new(
        version,
        value.trim_end_matches('/') == expected.trim_end_matches('/'),
        source,
        None,
        format!("pypi.url={value}"),
    ))
}
