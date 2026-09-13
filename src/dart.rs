use std::io;

use crate::config::Scope;
use crate::shared::sdk_profile_path;

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    crate::update_named_managed_block(
        &sdk_profile_path(scope)?,
        "dart",
        &crate::shell_env_assignment("PUB_HOSTED_URL", mirror),
    )
}

pub fn unset(scope: Scope) -> io::Result<()> {
    crate::remove_named_managed_block(&sdk_profile_path(scope)?, "dart")
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("dart")?;
    let path = sdk_profile_path(scope)?;
    let content = crate::read_optional(&path)?;
    let source = content
        .as_deref()
        .and_then(|content| env_value(content, "PUB_HOSTED_URL"));
    Ok(crate::ToolStatus::new(
        version,
        source.as_deref() == Some(expected),
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
