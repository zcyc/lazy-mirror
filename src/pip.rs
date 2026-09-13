use std::io;

fn executable() -> io::Result<&'static str> {
    ["pip", "pip3"]
        .into_iter()
        .find(|name| crate::command_exists(name))
        .ok_or_else(|| crate::missing_commands(&["pip", "pip3"]))
}

pub fn set(mirror: &str) -> io::Result<()> {
    crate::run(
        executable()?,
        &["config", "--user", "set", "global.index-url", mirror],
    )
}

pub fn unset() -> io::Result<()> {
    crate::run(
        executable()?,
        &["config", "--user", "unset", "global.index-url"],
    )
}

pub fn status(expected: &str) -> io::Result<crate::ToolStatus> {
    let executable = executable()?;
    let version = crate::command_version(executable)?;
    let index_url =
        crate::command_output(executable, &["config", "--user", "get", "global.index-url"])
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
