use std::io;

use crate::config::Scope;
use crate::shared::require_user;

pub fn set(mirror: &str, scope: Scope) -> io::Result<()> {
    require_user("flathub", scope)?;
    crate::run(
        "flatpak",
        &args(&["remote-modify", "--url", mirror, "flathub"]),
    )
    .or_else(|_| {
        crate::run(
            "flatpak",
            &args(&["remote-add", "--if-not-exists", "flathub", mirror]),
        )
    })
}

pub fn unset(scope: Scope) -> io::Result<()> {
    require_user("flathub", scope)?;
    crate::run(
        "flatpak",
        &args(&[
            "remote-modify",
            "--url",
            "https://dl.flathub.org/repo/",
            "flathub",
        ]),
    )
}

pub fn status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    require_user("flathub", scope)?;
    let version = crate::command_version("flatpak")?;
    let detail = crate::command_output("flatpak", &args(&["remotes", "--columns=name,url"]))
        .unwrap_or_else(|_| "not configured".to_owned());
    let source = detail.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        (fields.next() == Some("flathub"))
            .then(|| fields.next())
            .flatten()
    });
    Ok(crate::ToolStatus::new(
        version,
        source == Some(expected),
        source.map(str::to_owned),
        None,
        detail,
    ))
}

fn args<'a>(args: &[&'a str]) -> Vec<&'a str> {
    let mut scoped = Vec::with_capacity(args.len() + 1);
    scoped.push("--user");
    scoped.extend_from_slice(args);
    scoped
}

#[cfg(test)]
mod tests {
    use super::args;

    #[test]
    fn commands_use_user_scope() {
        assert_eq!(
            args(&[
                "remote-modify",
                "--url",
                "https://mirror.example",
                "flathub"
            ]),
            vec![
                "--user",
                "remote-modify",
                "--url",
                "https://mirror.example",
                "flathub"
            ]
        );
    }
}
