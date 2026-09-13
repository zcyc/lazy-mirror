use std::io;

use crate::config::Scope;
use crate::shared::sdk_profile_path;

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    let (pub_url, storage_url) = urls(mirror);
    crate::update_named_managed_block(
        &sdk_profile_path(scope)?,
        "flutter",
        &[
            crate::shell_env_assignment("PUB_HOSTED_URL", &pub_url),
            crate::shell_env_assignment("FLUTTER_STORAGE_BASE_URL", &storage_url),
        ]
        .join("\n"),
    )
}

pub fn unset(scope: Scope) -> io::Result<()> {
    crate::remove_named_managed_block(&sdk_profile_path(scope)?, "flutter")
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("flutter")?;
    let path = sdk_profile_path(scope)?;
    let content = crate::read_optional(&path)?;
    let source = content
        .as_deref()
        .and_then(|content| env_value(content, "FLUTTER_STORAGE_BASE_URL"));
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

pub fn urls(mirror: &str) -> (String, String) {
    let mirror = mirror.trim_end_matches('/');
    if let Some(base) = mirror.strip_suffix("/dart-pub") {
        return (mirror.to_owned(), base.to_owned());
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

pub fn mirror(mirror: &str) -> String {
    urls(mirror).1
}

fn env_value(content: &str, variable: &str) -> Option<String> {
    content
        .lines()
        .find_map(|line| crate::shell_env_value(line, variable))
}

#[cfg(test)]
mod tests {
    use super::{mirror, urls};

    #[test]
    fn dart_mirror_maps_to_flutter_storage_mirror() {
        assert_eq!(
            mirror("https://mirror.sjtu.edu.cn/dart-pub"),
            "https://mirror.sjtu.edu.cn"
        );
        assert_eq!(
            mirror("https://pub.flutter-io.cn"),
            "https://storage.flutter-io.cn"
        );
        assert_eq!(
            urls("https://mirror.sjtu.edu.cn/dart-pub"),
            (
                "https://mirror.sjtu.edu.cn/dart-pub".to_owned(),
                "https://mirror.sjtu.edu.cn".to_owned()
            )
        );
    }
}
