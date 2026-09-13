use std::io;
use std::path::PathBuf;

const PREFIX: &str = r#"allprojects {
  repositories {
    maven {
      url '"#;
const SUFFIX: &str = r#"'
    }
    mavenLocal()
    mavenCentral()
  }
}"#;

pub fn set(mirror: &str) -> io::Result<()> {
    let path = path()?;
    let content = format!("{PREFIX}{mirror}{SUFFIX}");
    crate::write_with_backup_if(&path, &content, |current| {
        current.starts_with(PREFIX) && current.ends_with(SUFFIX)
    })
}

pub fn unset() -> io::Result<()> {
    crate::remove_with_backup_if(&path()?, |content| {
        content.starts_with(PREFIX) && content.ends_with(SUFFIX)
    })
}

pub fn status(expected: &str) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("gradle")?;
    let path = path()?;
    let source = value(&path, "url '", "'")?;
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
    std::env::var_os("GRADLE_USER_HOME")
        .map(PathBuf::from)
        .or_else(dirs::home_dir)
        .map(|home| home.join("init.d/lazy-mirror.init.gradle"))
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "cannot determine home directory"))
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
