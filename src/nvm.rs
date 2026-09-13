use std::io;

use crate::config::Scope;
use crate::shared::{profile_set, profile_status, profile_unset};

const VARIABLE: &str = "NVM_NODEJS_ORG_MIRROR";

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    profile_set("nvm", scope, &crate::shell_env_assignment(VARIABLE, mirror))
}

pub fn unset(scope: Scope) -> io::Result<()> {
    profile_unset("nvm", scope)
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let mut status = profile_status("node", "nvm", VARIABLE, scope)?;
    status.configured = status.source.as_deref() == Some(expected);
    Ok(status)
}
