use std::fs;
use std::io;
use std::path::PathBuf;

const MAVEN_CONFIG_PREFIX: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<settings>
    <mirrors>
        <mirror>
            <id>lazy-mirror</id>
            <mirrorOf>*</mirrorOf>
            <name>lazy-mirror</name>
            <url>"#;
const MAVEN_CONFIG_SUFFIX: &str = r#"</url>
        </mirror>
    </mirrors>
</settings>"#;

const GRADLE_CONFIG_PREFIX: &str = r#"allprojects {
  repositories {
    maven {
      url '"#;
const GRADLE_CONFIG_SUFFIX: &str = r#"'
    }
    mavenLocal()
    mavenCentral()
  }
}"#;

pub fn maven_set(mirror: &str) -> io::Result<()> {
    let path = crate::home_file(".m2/settings.xml")?;
    let content = format!("{MAVEN_CONFIG_PREFIX}{mirror}{MAVEN_CONFIG_SUFFIX}");
    crate::write_with_backup_if(&path, &content, |current| {
        current.contains("<id>lazy-mirror</id>")
    })?;
    Ok(())
}

pub fn maven_unset() -> io::Result<()> {
    let path = crate::home_file(".m2/settings.xml")?;
    crate::remove_with_backup_if(&path, |content| content.contains("<id>lazy-mirror</id>"))?;
    Ok(())
}

pub fn gradle_set(mirror: &str) -> io::Result<()> {
    let path = gradle_config_path()?;
    let content = format!("{GRADLE_CONFIG_PREFIX}{mirror}{GRADLE_CONFIG_SUFFIX}");
    crate::write_with_backup_if(&path, &content, |current| {
        current.starts_with(GRADLE_CONFIG_PREFIX) && current.ends_with(GRADLE_CONFIG_SUFFIX)
    })?;
    Ok(())
}

pub fn gradle_unset() -> io::Result<()> {
    crate::remove_owned_if(&gradle_config_path()?, |content| {
        content.starts_with(GRADLE_CONFIG_PREFIX) && content.ends_with(GRADLE_CONFIG_SUFFIX)
    })?;
    Ok(())
}

pub fn maven_status(expected: &str) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("mvn")?;
    let path = crate::home_file(".m2/settings.xml")?;
    let source = file_value(&path, "<url>", "</url>")?;
    let configured = source.as_deref().is_some_and(|value| value == expected);
    Ok(crate::ToolStatus::new(
        version,
        configured,
        source.clone(),
        Some(path.clone()),
        format!(
            "source={}; config={}",
            source.unwrap_or_else(|| "not configured".to_owned()),
            path.display()
        ),
    ))
}

pub fn gradle_status(expected: &str) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("gradle")?;
    let path = gradle_config_path()?;
    let source = file_value(&path, "url '", "'")?;
    let configured = source.as_deref().is_some_and(|value| value == expected);
    Ok(crate::ToolStatus::new(
        version,
        configured,
        source.clone(),
        Some(path.clone()),
        format!(
            "source={}; config={}",
            source.unwrap_or_else(|| "not configured".to_owned()),
            path.display()
        ),
    ))
}

fn gradle_config_path() -> io::Result<PathBuf> {
    std::env::var_os("GRADLE_USER_HOME")
        .map(PathBuf::from)
        .or_else(dirs::home_dir)
        .map(|home| home.join("init.d/lazy-mirror.init.gradle"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cannot determine home directory"))
}

fn file_value(path: &std::path::Path, start: &str, end: &str) -> io::Result<Option<String>> {
    let Some(content) = fs::read_to_string(path).ok() else {
        return Ok(None);
    };
    Ok(content.split_once(start).and_then(|(_, value)| {
        value
            .split_once(end)
            .map(|(value, _)| value.trim().to_owned())
    }))
}
