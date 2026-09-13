use std::io;

pub fn set(mirror: &str) -> io::Result<()> {
    crate::run(
        "bundle",
        &[
            "config",
            "set",
            "--global",
            "mirror.https://rubygems.org",
            mirror,
        ],
    )
}

pub fn unset() -> io::Result<()> {
    crate::run(
        "bundle",
        &["config", "unset", "--global", "mirror.https://rubygems.org"],
    )
}

pub fn status(expected: &str) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("bundle")?;
    let mirror = crate::command_output(
        "bundle",
        &["config", "get", "--global", "mirror.https://rubygems.org"],
    )
    .unwrap_or_else(|_| "not configured".to_owned());
    let source = first_url(&mirror);
    Ok(crate::ToolStatus::new(
        version,
        mirror.contains(expected),
        source,
        None,
        mirror.replace('\n', "; "),
    ))
}

fn first_url(value: &str) -> Option<String> {
    value
        .split_whitespace()
        .find(|value| value.starts_with("http://") || value.starts_with("https://"))
        .map(str::to_owned)
}
