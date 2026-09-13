use std::io;

use crate::config::Scope;

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    match scope {
        Scope::User => crate::run("pnpm", &["config", "set", "registry", mirror, "--global"]),
        Scope::Project => crate::run("pnpm", &["config", "set", "registry", mirror]),
        Scope::System => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "pnpm does not expose a system mirror scope",
        )),
    }
}

pub fn unset(scope: Scope) -> io::Result<()> {
    match scope {
        Scope::User => crate::run("pnpm", &["config", "delete", "registry", "--global"]),
        Scope::Project => crate::run("pnpm", &["config", "delete", "registry"]),
        Scope::System => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "pnpm does not expose a system mirror scope",
        )),
    }
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("pnpm")?;
    let registry = match scope {
        Scope::User => crate::command_output("pnpm", &["config", "get", "registry", "--global"]),
        Scope::Project => crate::command_output("pnpm", &["config", "get", "registry"]),
        Scope::System => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "pnpm does not expose a system mirror scope",
            ));
        }
    }
    .unwrap_or_else(|_| "not configured".to_owned());
    let source = (registry != "not configured").then(|| registry.clone());
    Ok(crate::ToolStatus::new(
        version,
        registry == expected,
        source,
        None,
        format!("registry={registry}"),
    ))
}
