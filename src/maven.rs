use std::io;
use std::path::PathBuf;

const PREFIX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<settings>
    <mirrors>
        <mirror>
            <id>lazy-mirror</id>
            <mirrorOf>*</mirrorOf>
            <name>lazy-mirror</name>
            <url>"#;
const SUFFIX: &str = r#"</url>
        </mirror>
    </mirrors>
</settings>"#;

pub fn set(mirror: &str) -> io::Result<()> {
    let path = path()?;
    let content = format!("{PREFIX}{mirror}{SUFFIX}");
    crate::write_with_backup_if(&path, &content, |current| {
        current.contains("<id>lazy-mirror</id>")
    })
}

pub fn unset() -> io::Result<()> {
    crate::remove_with_backup_if(&path()?, |content| content.contains("<id>lazy-mirror</id>"))
}

pub fn status(expected: &str) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("mvn")?;
    let path = path()?;
    let source = value(&path, "<url>", "</url>")?;
    Ok(crate::ToolStatus::new(
        version,
        source.as_deref().is_some_and(|value| value == expected),
        source.clone(),
        Some(path.clone()),
        format!(
            "source={}; config={}",
            source.unwrap_or_else(|| "not configured".to_owned()),
            path.display()
        ),
    ))
}

fn path() -> io::Result<PathBuf> {
    crate::home_file(".m2/settings.xml")
}

fn value(path: &std::path::Path, start: &str, end: &str) -> io::Result<Option<String>> {
    let Some(content) = crate::read_optional(path)? else {
        return Ok(None);
    };
    Ok(content.split_once(start).and_then(|(_, value)| {
        value
            .split_once(end)
            .map(|(value, _)| value.trim().to_owned())
    }))
}
