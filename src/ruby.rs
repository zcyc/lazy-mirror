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
    crate::run(
        "gem",
        &[
            "sources",
            "--add",
            "https://rubygems.org/",
            "--remove",
            "https://gems.ruby-china.com/",
        ],
    )
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
    Ok(crate::ToolStatus {
        configured: sources.contains(expected),
        detail: sources.replace('\n', "; "),
        version,
    })
}

pub fn bundle_status(expected: &str) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("bundle")?;
    let mirror = crate::command_output(
        "bundle",
        &["config", "get", "--global", "mirror.https://rubygems.org"],
    )
    .unwrap_or_else(|_| "not configured".to_owned());
    Ok(crate::ToolStatus {
        configured: mirror.contains(expected),
        detail: mirror.replace('\n', "; "),
        version,
    })
}
