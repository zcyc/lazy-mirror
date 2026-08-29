use std::io;

use crate::config::Scope;

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    if scope != Scope::Project {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Poetry sources are project-scoped; use --scope project",
        ));
    }
    crate::run(
        "poetry",
        &[
            "source",
            "add",
            "--priority",
            "primary",
            "lazy-mirror",
            mirror,
        ],
    )
}

pub fn unset(scope: Scope) -> io::Result<()> {
    if scope != Scope::Project {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Poetry sources are project-scoped; use --scope project",
        ));
    }
    crate::run("poetry", &["source", "remove", "lazy-mirror"])
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("poetry")?;
    if scope != Scope::Project {
        return Ok(crate::ToolStatus {
            configured: false,
            detail: "Poetry sources are project-scoped".to_owned(),
            version,
        });
    }
    let sources = crate::command_output("poetry", &["source", "show"])
        .unwrap_or_else(|_| "not configured".to_owned());
    Ok(crate::ToolStatus {
        configured: sources.contains(expected),
        detail: sources,
        version,
    })
}
