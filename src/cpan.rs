use std::io;

use crate::config::Scope;
use crate::shared::{profile_set, profile_status, profile_unset};

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    profile_set(
        "cpan",
        scope,
        &crate::shell_env_assignment("PERL_CPAN_MIRROR", mirror),
    )
}

pub fn unset(scope: Scope) -> io::Result<()> {
    profile_unset("cpan", scope)
}

pub fn status(scope: Scope) -> io::Result<crate::ToolStatus> {
    profile_status("cpan", "cpan", "PERL_CPAN_MIRROR", scope)
}
