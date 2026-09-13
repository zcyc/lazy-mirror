use std::io;

pub fn set(mirror: &str) -> io::Result<()> {
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

pub fn unset() -> io::Result<()> {
    crate::run("gem", &["sources", "--add", "https://rubygems.org/"])
}

pub fn status(expected: &str) -> io::Result<crate::ToolStatus> {
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

fn first_url(value: &str) -> Option<String> {
    value
        .split_whitespace()
        .find(|value| value.starts_with("http://") || value.starts_with("https://"))
        .map(str::to_owned)
}
