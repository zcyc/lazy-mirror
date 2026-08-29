use std::io;

pub fn set(name: &str, mirror: &str) -> io::Result<()> {
    let base = mirror.trim_end_matches('/');
    let executable = if name == "mamba" { "mamba" } else { "conda" };
    crate::run(executable, &["config", "--set", "channel_alias", base])?;
    for channel in ["main", "r", "msys2"] {
        crate::run(
            executable,
            &[
                "config",
                "--add",
                "default_channels",
                &format!("{base}/pkgs/{channel}"),
            ],
        )?;
    }
    Ok(())
}

pub fn unset(name: &str) -> io::Result<()> {
    let executable = if name == "mamba" { "mamba" } else { "conda" };
    crate::run(executable, &["config", "--remove-key", "channel_alias"])?;
    crate::run(executable, &["config", "--remove-key", "default_channels"])
}

pub fn status(name: &str, expected: &str) -> io::Result<crate::ToolStatus> {
    let executable = if name == "mamba" { "mamba" } else { "conda" };
    let version = crate::command_version(executable)?;
    let detail = crate::command_output(executable, &["config", "--show", "channel_alias"])
        .unwrap_or_else(|_| "not configured".to_owned());
    let source = detail
        .lines()
        .find_map(|line| line.trim().strip_prefix("channel_alias:").map(str::trim))
        .filter(|value| !value.is_empty())
        .map(str::to_owned);
    Ok(crate::ToolStatus::new(
        version,
        detail.contains(expected.trim_end_matches('/')),
        source,
        None,
        detail,
    ))
}
