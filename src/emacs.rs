use std::io;

use crate::config::Scope;
use crate::shared::{config_path, file_status, require_user};

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    require_user("emacs", scope)?;
    let path = config_path("emacs", scope)?;
    crate::update_named_block(
        &path,
        "emacs",
        ";;",
        &format!("(setq package-archives '((\"mirror\" . \"{mirror}\")))"),
    )
}

pub fn unset(scope: Scope) -> io::Result<()> {
    require_user("emacs", scope)?;
    crate::remove_named_block(&config_path("emacs", scope)?, "emacs", ";;")
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    require_user("emacs", scope)?;
    let path = config_path("emacs", scope)?;
    file_status("emacs", &path, expected, |content| {
        content
            .split_once("(\"mirror\" . \"")
            .and_then(|(_, value)| value.split_once('"').map(|(value, _)| value.to_owned()))
    })
}
