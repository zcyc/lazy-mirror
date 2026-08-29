use std::io;
use std::path::PathBuf;

use crate::config::Scope;

pub fn dart_set(mirror: &str, scope: Scope) -> io::Result<()> {
    let path = profile_path(scope)?;
    crate::update_named_managed_block(
        &path,
        "dart",
        &crate::shell_env_assignment("PUB_HOSTED_URL", mirror),
    )
}

pub fn flutter_set(mirror: &str, scope: Scope) -> io::Result<()> {
    let (pub_url, storage_url) = flutter_urls(mirror);
    let path = profile_path(scope)?;
    crate::update_named_managed_block(
        &path,
        "flutter",
        &[
            crate::shell_env_assignment("PUB_HOSTED_URL", &pub_url),
            crate::shell_env_assignment("FLUTTER_STORAGE_BASE_URL", &storage_url),
        ]
        .join("\n"),
    )
}

pub fn dart_unset(scope: Scope) -> io::Result<()> {
    crate::remove_named_managed_block(&profile_path(scope)?, "dart")
}

pub fn flutter_unset(scope: Scope) -> io::Result<()> {
    crate::remove_named_managed_block(&profile_path(scope)?, "flutter")
}

pub fn unset(scope: Scope) -> io::Result<()> {
    dart_unset(scope)?;
    flutter_unset(scope)
}

pub fn dart_status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("dart")?;
    let path = profile_path(scope)?;
    let source = std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| env_value(&content, "PUB_HOSTED_URL"));
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

pub fn flutter_status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("flutter")?;
    let path = profile_path(scope)?;
    let source = std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| env_value(&content, "FLUTTER_STORAGE_BASE_URL"));
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

pub fn flutter_urls(mirror: &str) -> (String, String) {
    let mirror = mirror.trim_end_matches('/');
    if let Some(base) = mirror.strip_suffix("/dart-pub") {
        let storage = if base == "https://mirrors.tuna.tsinghua.edu.cn" {
            format!("{base}/flutter")
        } else {
            base.to_owned()
        };
        return (mirror.to_owned(), storage);
    }
    if let Some(base) = mirror.strip_suffix("/flutter") {
        return (format!("{base}/dart-pub"), mirror.to_owned());
    }
    if mirror == "https://pub.flutter-io.cn" {
        return (
            mirror.to_owned(),
            "https://storage.flutter-io.cn".to_owned(),
        );
    }
    if mirror == "https://storage.flutter-io.cn" {
        return ("https://pub.flutter-io.cn".to_owned(), mirror.to_owned());
    }
    if mirror == "https://mirror.sjtu.edu.cn" {
        return (format!("{mirror}/dart-pub"), mirror.to_owned());
    }
    (mirror.to_owned(), mirror.to_owned())
}

pub fn flutter_mirror(mirror: &str) -> String {
    flutter_urls(mirror).1
}

fn profile_path(scope: Scope) -> io::Result<PathBuf> {
    match scope {
        Scope::Project => std::env::current_dir().map(|path| path.join(".env")),
        Scope::User => {
            if let Some(path) = std::env::var_os("LM_SHELL_PROFILE") {
                return Ok(PathBuf::from(path));
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
                Ok(PathBuf::from("/etc/profile"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{flutter_mirror, flutter_urls};

    #[test]
    fn dart_mirror_maps_to_flutter_storage_mirror() {
        assert_eq!(
            flutter_mirror("https://mirror.sjtu.edu.cn/dart-pub"),
            "https://mirror.sjtu.edu.cn"
        );
        assert_eq!(
            flutter_mirror("https://pub.flutter-io.cn"),
            "https://storage.flutter-io.cn"
        );
        assert_eq!(
            flutter_urls("https://mirror.sjtu.edu.cn/dart-pub"),
            (
                "https://mirror.sjtu.edu.cn/dart-pub".to_owned(),
                "https://mirror.sjtu.edu.cn".to_owned()
            )
        );
        assert_eq!(
            flutter_urls("https://mirrors.tuna.tsinghua.edu.cn/dart-pub"),
            (
                "https://mirrors.tuna.tsinghua.edu.cn/dart-pub".to_owned(),
                "https://mirrors.tuna.tsinghua.edu.cn/flutter".to_owned()
            )
        );
    }
}
