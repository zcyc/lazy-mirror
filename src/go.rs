use std::io;

pub fn set(mirror: &str) -> io::Result<()> {
    let setting = format!("GOPROXY={mirror}");
    crate::run("go", &["env", "-w", &setting])
}

pub fn unset() -> io::Result<()> {
    crate::run("go", &["env", "-u", "GOPROXY"])
}

pub fn status(expected: &str) -> io::Result<crate::ToolStatus> {
    let version = crate::command_output("go", &["version"])?
        .lines()
        .next()
        .unwrap_or_default()
        .to_owned();
    let proxy = crate::command_output("go", &["env", "GOPROXY"])?;
    Ok(crate::ToolStatus {
        configured: proxy == expected,
        detail: format!("GOPROXY={proxy}"),
        version,
    })
}
