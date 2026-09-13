use std::io;

use crate::config::Scope;
use crate::shared::{config_path, file_status};

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    let path = config_path("cabal", scope)?;
    crate::write_with_backup_if(
        &path,
        &format!(
            "-- managed by lazy-mirror\nrepository hackage.haskell.org\n  url: {mirror}\n  secure: True\n"
        ),
        |content| content.starts_with("-- managed by lazy-mirror\n"),
    )
}

pub fn unset(scope: Scope) -> io::Result<()> {
    crate::remove_with_backup_if(&config_path("cabal", scope)?, |content| {
        content.starts_with("-- managed by lazy-mirror\n")
    })
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let path = config_path("cabal", scope)?;
    file_status("cabal", &path, expected, |content| {
        content
            .lines()
            .find_map(|line| line.trim().strip_prefix("url:").map(str::trim))
            .map(str::to_owned)
    })
}
