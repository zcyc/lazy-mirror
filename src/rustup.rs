use std::io;

use crate::config::Scope;
use crate::shared::{profile_set, profile_status, profile_unset};

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    let base = mirror.trim_end_matches('/');
    profile_set(
        "rustup",
        scope,
        &[
            crate::shell_env_assignment("RUSTUP_DIST_SERVER", base),
            crate::shell_env_assignment("RUSTUP_UPDATE_ROOT", &format!("{base}/rustup")),
        ]
        .join("\n"),
    )
}

pub fn unset(scope: Scope) -> io::Result<()> {
    profile_unset("rustup", scope)
}

pub fn status(scope: Scope) -> io::Result<crate::ToolStatus> {
    profile_status("rustup", "rustup", "RUSTUP_DIST_SERVER", scope)
}
