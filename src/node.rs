use std::io;

use crate::config::Scope;

pub fn set(name: &str, mirror: &str, scope: Scope) -> io::Result<()> {
    if scope == Scope::System {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{name} does not expose a system mirror scope"),
        ));
    }
    if name == "yarn" && yarn_berry()? {
        if scope == Scope::User {
            crate::run(
                name,
                &["config", "set", "npmRegistryServer", mirror, "--home"],
            )
        } else {
            crate::run(name, &["config", "set", "npmRegistryServer", mirror])
        }
    } else if name == "npm" {
        let location = match scope {
            Scope::Project => "project",
            Scope::User => "user",
            Scope::System => unreachable!(),
        };
        crate::run(
            name,
            &["config", "set", "registry", mirror, "--location", location],
        )
    } else if name == "pnpm" && scope == Scope::User {
        crate::run(name, &["config", "set", "registry", mirror, "--global"])
    } else {
        crate::run(name, &["config", "set", "registry", mirror])
    }
}

pub fn unset(name: &str, scope: Scope) -> io::Result<()> {
    if scope == Scope::System {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{name} does not expose a system mirror scope"),
        ));
    }
    if name == "yarn" && yarn_berry()? {
        if scope == Scope::User {
            crate::run(name, &["config", "unset", "npmRegistryServer", "--home"])
        } else {
            crate::run(name, &["config", "unset", "npmRegistryServer"])
        }
    } else if name == "npm" {
        let location = match scope {
            Scope::Project => "project",
            Scope::User => "user",
            Scope::System => unreachable!(),
        };
        crate::run(
            name,
            &["config", "delete", "registry", "--location", location],
        )
    } else if name == "pnpm" && scope == Scope::User {
        crate::run(name, &["config", "delete", "registry", "--global"])
    } else {
        crate::run(name, &["config", "delete", "registry"])
    }
}

pub fn status(name: &str, expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version(name)?;
    let registry = if name == "yarn" && yarn_berry()? {
        if scope == Scope::User {
            crate::command_output(name, &["config", "get", "npmRegistryServer", "--home"])
        } else {
            crate::command_output(name, &["config", "get", "npmRegistryServer"])
        }
    } else if name == "npm" {
        let location = match scope {
            Scope::Project => "project",
            Scope::User => "user",
            Scope::System => "global",
        };
        crate::command_output(name, &["config", "get", "registry", "--location", location])
    } else if name == "pnpm" && scope == Scope::User {
        crate::command_output(name, &["config", "get", "registry", "--global"])
    } else {
        crate::command_output(name, &["config", "get", "registry"])
    }
    .unwrap_or_else(|_| "not configured".to_owned());
    Ok(crate::ToolStatus {
        configured: registry == expected,
        detail: format!("registry={registry}"),
        version,
    })
}

fn yarn_berry() -> io::Result<bool> {
    let version = crate::command_output("yarn", &["--version"])?;
    Ok(version
        .split('.')
        .next()
        .and_then(|major| major.parse::<u32>().ok())
        .is_some_and(|major| major >= 2))
}
