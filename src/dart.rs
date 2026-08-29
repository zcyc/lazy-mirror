use std::io;
use std::path::PathBuf;

use crate::config::Scope;

pub fn dart_set(mirror: &str, scope: Scope) -> io::Result<()> {
    let path = profile_path(scope)?;
    crate::update_named_managed_block(
        &path,
        "dart",
        &format!("export PUB_HOSTED_URL=\"{mirror}\""),
    )
}

pub fn flutter_set(mirror: &str, scope: Scope) -> io::Result<()> {
    let path = profile_path(scope)?;
    let pub_url = pub_url(mirror);
    crate::update_named_managed_block(
        &path,
        "flutter",
        &format!(
            "export PUB_HOSTED_URL=\"{pub_url}\"\nexport FLUTTER_STORAGE_BASE_URL=\"{mirror}\""
        ),
    )
}

pub fn unset(scope: Scope) -> io::Result<()> {
    crate::remove_named_managed_block(&profile_path(scope)?, "dart")?;
    crate::remove_named_managed_block(&profile_path(scope)?, "flutter")
}

pub fn dart_status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("dart")?;
    let path = profile_path(scope)?;
    let source = std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| env_value(&content, "PUB_HOSTED_URL"));
    let configured = source.as_deref().is_some_and(|value| value == expected);
    Ok(crate::ToolStatus {
        configured,
        detail: format!(
            "source={}; profile={}",
            source.unwrap_or_else(|| "not configured".to_owned()),
            path.display()
        ),
        version,
    })
}

pub fn flutter_status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("flutter")?;
    let path = profile_path(scope)?;
    let source = std::fs::read_to_string(&path)
        .ok()
        .and_then(|content| env_value(&content, "FLUTTER_STORAGE_BASE_URL"));
    let configured = source.as_deref().is_some_and(|value| value == expected);
    Ok(crate::ToolStatus {
        configured,
        detail: format!(
            "source={}; profile={}",
            source.unwrap_or_else(|| "not configured".to_owned()),
            path.display()
        ),
        version,
    })
}

fn env_value(content: &str, variable: &str) -> Option<String> {
    content.lines().find_map(|line| {
        let value = line.split_once(&format!("{variable}=\""))?.1;
        value.split_once('"').map(|(value, _)| value.to_owned())
    })
}

fn pub_url(mirror: &str) -> String {
    if mirror.ends_with("/flutter") {
        return format!("{}/dart-pub", mirror.trim_end_matches("/flutter"));
    }
    if mirror == "https://storage.flutter-io.cn" {
        return "https://pub.flutter-io.cn".to_owned();
    }
    mirror.to_owned()
}

fn profile_path(scope: Scope) -> io::Result<PathBuf> {
    match scope {
        Scope::Project => std::env::current_dir().map(|path| path.join(".env")),
        Scope::User => {
            if let Some(path) = std::env::var_os("LM_SHELL_PROFILE") {
                return Ok(PathBuf::from(path));
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
        Scope::System => Ok(PathBuf::from("/etc/profile")),
    }
}
