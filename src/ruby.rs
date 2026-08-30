use std::io;

pub fn gem_set(mirror: &str) -> io::Result<()> {
    crate::run(
        "gem",
        &[
            "sources",
            "--add",
            mirror,
            "--remove",
            "https://rubygems.org/",
        ],
    )
}

pub fn gem_unset() -> io::Result<()> {
    crate::run("gem", &["sources", "--add", "https://rubygems.org/"])
}

pub fn bundle_set(mirror: &str) -> io::Result<()> {
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

pub fn bundle_unset() -> io::Result<()> {
    crate::run(
        "bundle",
        &["config", "unset", "--global", "mirror.https://rubygems.org"],
    )
}

pub fn gem_status(expected: &str) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("gem")?;
    let sources = crate::command_output("gem", &["sources", "--list"])?;
    let source = first_url(&sources);
    Ok(crate::ToolStatus::new(
        version,
        sources.contains(expected),
        source,
        None,
        sources.replace('\n', "; "),
    ))
}

pub fn bundle_status(expected: &str) -> io::Result<crate::ToolStatus> {
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
