use std::io;

use crate::config::Scope;
use crate::shared::{profile_set, profile_status, profile_unset};

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    let base = mirror.trim_end_matches('/');
    profile_set(
        "homebrew",
        scope,
        &[
            crate::shell_env_assignment(
                "HOMEBREW_API_DOMAIN",
                &format!("{base}/homebrew-bottles/api"),
            ),
            crate::shell_env_assignment(
                "HOMEBREW_BOTTLE_DOMAIN",
                &format!("{base}/homebrew-bottles"),
            ),
            crate::shell_env_assignment(
                "HOMEBREW_BREW_GIT_REMOTE",
                &format!("{base}/git/homebrew/brew.git"),
            ),
            crate::shell_env_assignment(
                "HOMEBREW_CORE_GIT_REMOTE",
                &format!("{base}/git/homebrew/homebrew-core.git"),
            ),
        ]
        .join("\n"),
    )
}

pub fn unset(scope: Scope) -> io::Result<()> {
    profile_unset("homebrew", scope)
}

pub fn status(scope: Scope) -> io::Result<crate::ToolStatus> {
    profile_status("brew", "homebrew", "HOMEBREW_BOTTLE_DOMAIN", scope)
}
