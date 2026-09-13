use std::io;

use crate::config::Scope;
use crate::shared::{profile_set, profile_status, profile_unset};

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    profile_set(
        "nix",
        scope,
        &crate::shell_env_assignment("NIX_CONFIG", &format!("substituters = {mirror}")),
    )
}

pub fn unset(scope: Scope) -> io::Result<()> {
    profile_unset("nix", scope)
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let mut status = profile_status("nix", "nix", "NIX_CONFIG", scope)?;
    let source = status.source.as_deref().and_then(mirror).map(str::to_owned);
    status.configured = source
        .as_deref()
        .is_some_and(|value| value.trim_end_matches('/') == expected.trim_end_matches('/'));
    status.source = source;
    Ok(status)
}

fn mirror(value: &str) -> Option<&str> {
    value.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        (key.trim() == "substituters")
            .then(|| value.split_whitespace().next())
            .flatten()
    })
}

#[cfg(test)]
mod tests {
    use super::mirror;

    #[test]
    fn config_source_extracts_the_first_substituter() {
        assert_eq!(
            mirror("substituters = https://cache.example"),
            Some("https://cache.example")
        );
        assert_eq!(mirror("https://cache.example"), None);
    }
}
