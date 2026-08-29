use std::io;

use crate::config::Scope;

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    let path = profile_path(scope)?;
    crate::update_named_managed_block(
        &path,
        "huggingface",
        &crate::shell_env_assignment("HF_ENDPOINT", mirror),
    )
}

pub fn unset(scope: Scope) -> io::Result<()> {
    crate::remove_named_managed_block(&profile_path(scope)?, "huggingface")
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = command_version()?;
    let path = profile_path(scope)?;
    let source = std::env::var("HF_ENDPOINT").ok().or_else(|| {
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|content| env_value(&content, "HF_ENDPOINT"))
    });
    let configured = source.as_deref().is_some_and(|value| value == expected);
    Ok(crate::ToolStatus::new(
        version,
        configured,
        source.clone(),
        Some(path.clone()),
        format!(
            "source={}; profile={}",
            source.unwrap_or_else(|| "not configured".to_owned()),
            path.display()
        ),
    ))
}

fn env_value(content: &str, variable: &str) -> Option<String> {
    content
        .lines()
        .find_map(|line| crate::shell_env_value(line, variable))
}

fn command_version() -> io::Result<String> {
    crate::command_version("hf").or_else(|_| crate::command_version("huggingface-cli"))
}

fn profile_path(scope: Scope) -> io::Result<std::path::PathBuf> {
    match scope {
        Scope::Project => std::env::current_dir().map(|path| path.join(".env")),
        Scope::User => {
            if let Some(path) = std::env::var_os("LM_SHELL_PROFILE") {
                return Ok(path.into());
            }
            #[cfg(windows)]
            {
                return crate::powershell_profile_path();
            }
            let shell = std::env::var_os("SHELL")
                .map(|value| value.to_string_lossy().into_owned())
                .unwrap_or_default();
            if shell.ends_with("/fish") {
                crate::home_file(".config/fish/config.fish")
            } else if shell.ends_with("/bash") {
                crate::home_file(".bashrc")
            } else {
                crate::home_file(".zshrc")
            }
        }
        Scope::System => {
            #[cfg(windows)]
            {
                crate::powershell_system_profile_path()
            }
            #[cfg(not(windows))]
            {
                Ok("/etc/profile".into())
            }
        }
    }
}
