use std::io;

use crate::config::Scope;
use crate::shared::require_user;

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    require_user("cocoapods", scope)?;
    let _ = crate::run("pod", &["repo", "remove", "lazy-mirror"]);
    crate::run("pod", &["repo", "add", "lazy-mirror", mirror])
}

pub fn unset(scope: Scope) -> io::Result<()> {
    require_user("cocoapods", scope)?;
    crate::run("pod", &["repo", "remove", "lazy-mirror"])
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    require_user("cocoapods", scope)?;
    let version = crate::command_version("pod")?;
    let detail = crate::command_output("pod", &["repo", "list"])
        .unwrap_or_else(|_| "not configured".to_owned());
    Ok(crate::ToolStatus::new(
        version,
        detail.contains("lazy-mirror") && detail.contains(expected),
        None,
        None,
        detail,
    ))
}
