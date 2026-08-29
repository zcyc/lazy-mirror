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
    let source = (index_url != "not configured").then(|| index_url.clone());
    Ok(crate::ToolStatus::new(
        version,
        index_url == expected,
        source,
        None,
        format!("global.index-url={index_url}"),
    ))
}
