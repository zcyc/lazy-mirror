use std::io;

use crate::config::Scope;

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    let key = if yarn_berry()? {
        "npmRegistryServer"
    } else {
        "registry"
    };
    match scope {
        Scope::User if key == "npmRegistryServer" => {
            crate::run("yarn", &["config", "set", key, mirror, "--home"])
        }
        Scope::User | Scope::Project => crate::run("yarn", &["config", "set", key, mirror]),
        Scope::System => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "yarn does not expose a system mirror scope",
        )),
    }
}

pub fn unset(scope: Scope) -> io::Result<()> {
    let key = if yarn_berry()? {
        "npmRegistryServer"
    } else {
        "registry"
    };
    match scope {
        Scope::User if key == "npmRegistryServer" => {
            crate::run("yarn", &["config", "unset", key, "--home"])
        }
        Scope::Project if key == "npmRegistryServer" => {
            crate::run("yarn", &["config", "unset", key])
        }
        Scope::User => crate::run("yarn", &["config", "delete", key]),
        Scope::Project => crate::run("yarn", &["config", "delete", key]),
        Scope::System => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "yarn does not expose a system mirror scope",
        )),
    }
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("yarn")?;
    let registry = if yarn_berry()? {
        match scope {
            Scope::User => {
                crate::command_output("yarn", &["config", "get", "npmRegistryServer", "--home"])
            }
            Scope::Project => {
                crate::command_output("yarn", &["config", "get", "npmRegistryServer"])
            }
            Scope::System => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "yarn does not expose a system mirror scope",
                ));
            }
        }
    } else {
        match scope {
            Scope::User | Scope::Project => {
                crate::command_output("yarn", &["config", "get", "registry"])
            }
            Scope::System => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "yarn does not expose a system mirror scope",
                ));
            }
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

fn yarn_berry() -> io::Result<bool> {
    let version = crate::command_output("yarn", &["--version"])?;
    Ok(version
        .split('.')
        .next()
        .and_then(|major| major.parse::<u32>().ok())
        .is_some_and(|major| major >= 2))
}
