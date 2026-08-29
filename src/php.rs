use std::io;

pub fn set(mirror: &str) -> io::Result<()> {
    crate::run(
        "composer",
        &["config", "-g", "repos.packagist", "composer", mirror],
    )
}

pub fn unset() -> io::Result<()> {
    crate::run("composer", &["config", "-g", "--unset", "repos.packagist"])
}

pub fn status(expected: &str) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version("composer")?;
    let repository = crate::command_output("composer", &["config", "-g", "repos.packagist"])
        .unwrap_or_else(|_| "not configured".to_owned());
    Ok(crate::ToolStatus::new(
        version,
        repository.contains(expected),
        Some(repository.clone()),
        None,
        format!("repos.packagist={repository}"),
    ))
}
