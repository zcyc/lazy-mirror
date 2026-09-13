use std::io;

use crate::config::Scope;

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    let location = location(scope)?;
    crate::run(
        "npm",
        &["config", "set", "registry", mirror, "--location", location],
    )
}

pub fn unset(scope: Scope) -> io::Result<()> {
    let location = location(scope)?;
    crate::run(
        "npm",
        &["config", "delete", "registry", "--location", location],
    )
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("npm")?;
    let location = match scope {
        Scope::Project => "project",
        Scope::User => "user",
        Scope::System => "global",
    };
    let registry = crate::command_output(
        "npm",
        &["config", "get", "registry", "--location", location],
    )
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

fn location(scope: Scope) -> io::Result<&'static str> {
    match scope {
        Scope::Project => Ok("project"),
        Scope::User => Ok("user"),
        Scope::System => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "npm does not expose a system mirror scope",
        )),
    }
}
