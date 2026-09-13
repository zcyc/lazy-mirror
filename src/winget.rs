use std::io;

use crate::config::Scope;
use crate::shared::require_user;

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    require_user("winget", scope)?;
    let _ = crate::run("winget", &["source", "remove", "--name", "lazy-mirror"]);
    crate::run(
        "winget",
        &["source", "add", "--name", "lazy-mirror", mirror],
    )
}

pub fn unset(scope: Scope) -> io::Result<()> {
    require_user("winget", scope)?;
    crate::run("winget", &["source", "remove", "--name", "lazy-mirror"])
}

pub fn status(scope: Scope) -> io::Result<crate::ToolStatus> {
    require_user("winget", scope)?;
    let version = crate::command_version("winget")?;
    let detail = crate::command_output("winget", &["source", "list"])
        .unwrap_or_else(|_| "not configured".to_owned());
    let source = detail
        .lines()
        .skip_while(|line| !line.contains("lazy-mirror"))
        .skip(1)
        .flat_map(str::split_whitespace)
        .find(|value| value.starts_with("http://") || value.starts_with("https://"))
        .map(str::to_owned);
    Ok(crate::ToolStatus::new(
        version,
        detail.contains("lazy-mirror"),
        source,
        None,
        detail,
    ))
}
