use std::io;

use crate::config::Scope;
use crate::shared::require_user;

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    require_user("tex", scope)?;
    crate::run("tlmgr", &["option", "repository", mirror])
}

pub fn unset(scope: Scope) -> io::Result<()> {
    require_user("tex", scope)?;
    crate::run(
        "tlmgr",
        &[
            "option",
            "repository",
            "https://mirror.ctan.org/systems/texlive/tlnet",
        ],
    )
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    require_user("tex", scope)?;
    let version = crate::command_version("tlmgr")?;
    let detail = crate::command_output("tlmgr", &["option", "repository"])
        .unwrap_or_else(|_| "not configured".to_owned());
    let source = detail
        .lines()
        .find_map(|line| line.find("http").map(|index| &line[index..]))
        .map(str::trim)
        .map(str::to_owned);
    Ok(crate::ToolStatus::new(
        version,
        source.as_deref() == Some(expected),
        source,
        None,
        detail,
    ))
}
