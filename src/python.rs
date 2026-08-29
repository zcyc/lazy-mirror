use std::io;

pub fn set(name: &str, mirror: &str) -> io::Result<()> {
    crate::run(
        name,
        &["config", "--user", "set", "global.index-url", mirror],
    )
}

pub fn unset(name: &str) -> io::Result<()> {
    crate::run(name, &["config", "--user", "unset", "global.index-url"])
}

pub fn status(name: &str, expected: &str) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version(name)?;
    let index_url = crate::command_output(name, &["config", "--user", "get", "global.index-url"])
        .unwrap_or_else(|_| "not configured".to_owned());
    Ok(crate::ToolStatus {
        configured: index_url == expected,
        detail: format!("global.index-url={index_url}"),
        version,
    })
}
