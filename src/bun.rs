use std::io;

use crate::config::Scope;

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    if scope == Scope::System {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "bun does not expose a system mirror scope",
        ));
    }
    crate::run("bun", &["config", "set", "registry", mirror])
}

pub fn unset(scope: Scope) -> io::Result<()> {
    if scope == Scope::System {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "bun does not expose a system mirror scope",
        ));
    }
    crate::run("bun", &["config", "delete", "registry"])
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    if scope == Scope::System {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "bun does not expose a system mirror scope",
        ));
    }
    let version = crate::command_version("bun")?;
    let registry = crate::command_output("bun", &["config", "get", "registry"])
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
