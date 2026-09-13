use std::io;
use std::path::{Path, PathBuf};

use crate::config::Scope;

pub(crate) fn file_status<F>(
    command: &str,
    path: &Path,
    expected: &str,
    source: F,
) -> io::Result<crate::ToolStatus>
where
    F: Fn(&str) -> Option<String>,
{
    let content = crate::read_optional(path)?;
    let source = content.as_deref().and_then(source);
    let version = crate::command_version(command)?;
    Ok(crate::ToolStatus::new(
        version,
        source.as_deref().is_some_and(|value| {
            expected.is_empty() || value.trim_end_matches('/') == expected.trim_end_matches('/')
        }),
        source.clone(),
        Some(path.to_path_buf()),
        format!(
            "source={}; config={}",
            source.unwrap_or_else(|| "not configured".to_owned()),
            path.display()
        ),
    ))
}

pub(crate) fn config_path(name: &str, scope: Scope) -> io::Result<PathBuf> {
    if let Some(path) = std::env::var_os(format!("LM_{}_CONFIG", name.to_uppercase())) {
        return Ok(path.into());
    }
    match scope {
        Scope::Project => {
            let relative = match name {
                "clojure" => ".clojure/deps.edn".to_owned(),
                "emacs" => ".emacs".to_owned(),
                _ => format!(".{name}/config"),
            };
            std::env::current_dir().map(|path| path.join(relative))
        }
        Scope::User => crate::home_file(match name {
            "clojure" => ".clojure/deps.edn",
            "cabal" => ".cabal/config",
            "stack" => ".stack/config.yaml",
            "emacs" => ".emacs",
            _ => ".config/lazy-mirror/config",
        }),
        Scope::System => Ok(PathBuf::from(format!("/etc/{name}/lazy-mirror.conf"))),
    }
}

pub(crate) fn profile_set(name: &str, scope: Scope, block: &str) -> io::Result<()> {
    crate::update_named_managed_block(&profile_path(scope)?, name, block)
}

pub(crate) fn profile_unset(name: &str, scope: Scope) -> io::Result<()> {
    crate::remove_named_managed_block(&profile_path(scope)?, name)
}

pub(crate) fn profile_status(
    command: &str,
    name: &str,
    variable: &str,
    scope: Scope,
) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version(command)?;
    let path = profile_path(scope)?;
    let marker = format!("# >>> lazy-mirror:{name} >>>");
    let content = crate::read_optional(&path)?;
    let source = content.as_deref().and_then(|content| {
        content
            .contains(&marker)
            .then(|| content.lines().find(|line| line.contains(variable)))
            .flatten()
            .and_then(|line| crate::shell_env_value(line, variable))
    });
    Ok(crate::ToolStatus {
        version,
        configured: source.is_some(),
        source: source.clone(),
        path: Some(path.clone()),
        detail: format!(
            "{variable}={}; profile={}",
            source.as_deref().unwrap_or("not configured"),
            path.display()
        ),
    })
}

pub fn source_for_restore(name: &str, source: &str) -> String {
    let source = if name == "brew" {
        source.strip_suffix("/homebrew-bottles").unwrap_or(source)
    } else {
        source
    };
    source.trim_end_matches('/').to_owned()
}

pub(crate) fn sdk_profile_path(scope: Scope) -> io::Result<PathBuf> {
    match scope {
        Scope::Project => std::env::current_dir().map(|path| path.join(".env")),
        Scope::User => {
            if let Some(path) = std::env::var_os("LM_SHELL_PROFILE") {
                return Ok(path.into());
            }
            #[cfg(windows)]
            {
                crate::powershell_profile_path()
            }
            #[cfg(not(windows))]
            {
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
        }
        Scope::System => {
            #[cfg(windows)]
            {
                crate::powershell_system_profile_path()
            }
            #[cfg(not(windows))]
            {
                Ok(PathBuf::from("/etc/profile"))
            }
        }
    }
}

fn profile_path(scope: Scope) -> io::Result<PathBuf> {
    match scope {
        Scope::Project => std::env::current_dir().map(|path| path.join(".env")),
        Scope::User => {
            if let Some(path) = std::env::var_os("LM_SHELL_PROFILE") {
                Ok(path.into())
            } else {
                #[cfg(windows)]
                {
                    crate::powershell_profile_path()
                }
                #[cfg(not(windows))]
                {
                    crate::home_file(profile_relative_path(&shell()))
                }
            }
        }
        Scope::System => {
            #[cfg(windows)]
            {
                crate::powershell_system_profile_path()
            }
            #[cfg(not(windows))]
            {
                Ok(PathBuf::from("/etc/profile"))
            }
        }
    }
}

#[cfg(not(windows))]
fn shell() -> String {
    std::env::var_os("SHELL")
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(not(windows))]
fn profile_relative_path(shell: &str) -> &'static str {
    if shell.ends_with("/fish") {
        ".config/fish/config.fish"
    } else {
        ".profile"
    }
}

pub(crate) fn require_system(name: &str, scope: Scope) -> io::Result<()> {
    if scope == Scope::System {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{name} supports system scope only"),
        ))
    }
}

pub(crate) fn require_user(name: &str, scope: Scope) -> io::Result<()> {
    if scope == Scope::User {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{name} supports user scope only"),
        ))
    }
}

pub(crate) fn apt_distribution() -> String {
    if let Ok(distribution) = std::env::var("LM_APT_DISTRIBUTION") {
        return distribution;
    }
    std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            content.lines().find_map(|line| {
                line.strip_prefix("VERSION_CODENAME=")
                    .or_else(|| line.strip_prefix("UBUNTU_CODENAME="))
                    .map(|value| value.trim_matches('"').to_owned())
            })
        })
        .unwrap_or_else(|| "stable".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{file_status, profile_relative_path, source_for_restore};

    #[test]
    fn file_status_reports_missing_tool() {
        let path = std::env::temp_dir().join(format!(
            "lm-file-status-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::write(&path, "mirror").unwrap();
        let error = match file_status("lm-command-does-not-exist", &path, "mirror", |_| {
            Some("mirror".to_owned())
        }) {
            Ok(_) => panic!("missing tool should be reported"),
            Err(error) => error,
        };
        assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn rendered_homebrew_source_is_normalized_before_restore() {
        assert_eq!(
            source_for_restore("brew", "https://mirror.example/homebrew-bottles"),
            "https://mirror.example"
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn fish_uses_its_startup_configuration_file() {
        assert_eq!(
            profile_relative_path("/usr/bin/fish"),
            ".config/fish/config.fish"
        );
        assert_eq!(profile_relative_path("/bin/bash"), ".profile");
    }
}
