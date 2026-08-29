use std::io;
use std::path::PathBuf;

use crate::config::Scope;

const SOURCE_NAME: &str = "lazy-mirror";

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    let path = config_path(scope)?;
    let path = path.to_string_lossy().into_owned();
    let _ = run_dotnet(&[
        "nuget",
        "remove",
        "source",
        SOURCE_NAME,
        "--configfile",
        &path,
    ]);
    run_dotnet(&[
        "nuget",
        "add",
        "source",
        mirror,
        "--name",
        SOURCE_NAME,
        "--configfile",
        &path,
    ])
}

pub fn unset(scope: Scope) -> io::Result<()> {
    let path = config_path(scope)?;
    let path = path.to_string_lossy().into_owned();
    run_dotnet(&[
        "nuget",
        "remove",
        "source",
        SOURCE_NAME,
        "--configfile",
        &path,
    ])
}

pub fn status(scope: Scope) -> io::Result<crate::ToolStatus> {
    let version = crate::command_output("dotnet", &["--version"])?;
    let path = config_path(scope)?;
    let path_string = path.to_string_lossy().into_owned();
    let detail = run_dotnet_output(&[
        "nuget",
        "list",
        "source",
        "--format",
        "detailed",
        "--configfile",
        &path_string,
    ])
    .unwrap_or_else(|_| "not configured".to_owned());
    let source = detail
        .lines()
        .skip_while(|line| !line.contains(SOURCE_NAME))
        .find_map(|line| {
            let value = line.trim();
            (value.starts_with("http://") || value.starts_with("https://"))
                .then(|| value.to_owned())
        });
    Ok(crate::ToolStatus::new(
        version,
        source.is_some(),
        source,
        Some(path.clone()),
        format!("config={}; {detail}", path.display()),
    ))
}

fn config_path(scope: Scope) -> io::Result<PathBuf> {
    match scope {
        Scope::Project => std::env::current_dir().map(|path| path.join("NuGet.Config")),
        Scope::User => crate::home_file(".nuget/NuGet/NuGet.Config"),
        Scope::System => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "NuGet supports project and user scopes only",
        )),
    }
}

fn run_dotnet(args: &[&str]) -> io::Result<()> {
    crate::run("dotnet", args)
}

fn run_dotnet_output(args: &[&str]) -> io::Result<String> {
    crate::command_output("dotnet", args)
}
