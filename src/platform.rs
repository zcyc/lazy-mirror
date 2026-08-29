use std::io;
use std::path::PathBuf;

use crate::config::Scope;

const APT_PREFIX: &str = "# managed by lazy-mirror\n";

pub fn apt_set(mirror: &str, scope: Scope) -> io::Result<()> {
    require_system("apt", scope)?;
    let path = apt_path()?;
    let distribution = std::env::var("LM_APT_DISTRIBUTION").unwrap_or_else(|_| "stable".to_owned());
    let components = std::env::var("LM_APT_COMPONENTS").unwrap_or_else(|_| "main".to_owned());
    let line = if mirror.starts_with("deb ") || mirror.starts_with("deb-src ") {
        mirror.to_owned()
    } else {
        format!("deb {mirror} {distribution} {components}")
    };
    crate::write_with_backup_if(&path, &format!("{APT_PREFIX}{line}\n"), |content| {
        content.starts_with(APT_PREFIX)
    })
}

pub fn apt_unset(scope: Scope) -> io::Result<()> {
    require_system("apt", scope)?;
    crate::remove_with_backup_if(&apt_path()?, |content| content.starts_with(APT_PREFIX))
}

pub fn apt_status(scope: Scope) -> io::Result<crate::ToolStatus> {
    require_system("apt", scope)?;
    let path = apt_path()?;
    let source = std::fs::read_to_string(&path).ok().and_then(|content| {
        content
            .strip_prefix(APT_PREFIX)
            .map(str::trim)
            .map(str::to_owned)
    });
    Ok(crate::ToolStatus {
        version: crate::command_output("apt", &["--version"]).unwrap_or_else(|_| "apt".to_owned()),
        configured: source.is_some(),
        detail: format!(
            "source={}; config={}",
            source.unwrap_or_else(|| "not configured".to_owned()),
            path.display()
        ),
    })
}

pub fn apk_set(mirror: &str, scope: Scope) -> io::Result<()> {
    require_system("apk", scope)?;
    let path = apk_path()?;
    crate::write_with_backup_if(
        &path,
        &format!("# managed by lazy-mirror\n{mirror}\n"),
        |content| content.starts_with("# managed by lazy-mirror\n"),
    )
}

pub fn apk_unset(scope: Scope) -> io::Result<()> {
    require_system("apk", scope)?;
    crate::remove_with_backup_if(&apk_path()?, |content| {
        content.starts_with("# managed by lazy-mirror\n")
    })
}

pub fn apk_status(scope: Scope) -> io::Result<crate::ToolStatus> {
    require_system("apk", scope)?;
    let path = apk_path()?;
    let source = std::fs::read_to_string(&path).ok().and_then(|content| {
        content
            .strip_prefix("# managed by lazy-mirror\n")
            .map(str::trim)
            .map(str::to_owned)
    });
    Ok(crate::ToolStatus {
        version: crate::command_output("apk", &["--version"]).unwrap_or_else(|_| "apk".to_owned()),
        configured: source.is_some(),
        detail: format!(
            "repositories={}; config={}",
            source.unwrap_or_else(|| "not configured".to_owned()),
            path.display()
        ),
    })
}

pub fn brew_set(mirror: &str, scope: Scope) -> io::Result<()> {
    let base = mirror.trim_end_matches('/');
    profile_set(
        "homebrew",
        scope,
        &format!(
            "export HOMEBREW_API_DOMAIN=\"{base}/homebrew-bottles/api\"\nexport HOMEBREW_BOTTLE_DOMAIN=\"{base}/homebrew-bottles\"\nexport HOMEBREW_BREW_GIT_REMOTE=\"{base}/git/homebrew/brew.git\"\nexport HOMEBREW_CORE_GIT_REMOTE=\"{base}/git/homebrew/homebrew-core.git\""
        ),
    )
}

pub fn brew_unset(scope: Scope) -> io::Result<()> {
    profile_unset("homebrew", scope)
}

pub fn brew_status(scope: Scope) -> io::Result<crate::ToolStatus> {
    profile_status("brew", "homebrew", "HOMEBREW_BOTTLE_DOMAIN", scope)
}

pub fn rustup_set(mirror: &str, scope: Scope) -> io::Result<()> {
    let base = mirror.trim_end_matches('/');
    profile_set(
        "rustup",
        scope,
        &format!(
            "export RUSTUP_DIST_SERVER=\"{base}\"\nexport RUSTUP_UPDATE_ROOT=\"{base}/rustup\""
        ),
    )
}

pub fn rustup_unset(scope: Scope) -> io::Result<()> {
    profile_unset("rustup", scope)
}

pub fn rustup_status(scope: Scope) -> io::Result<crate::ToolStatus> {
    profile_status("rustup", "rustup", "RUSTUP_DIST_SERVER", scope)
}

pub fn hex_set(mirror: &str, scope: Scope) -> io::Result<()> {
    if scope != Scope::User {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Hex supports user scope only",
        ));
    }
    crate::run("mix", &["hex.config", "mirror_url", mirror])
}

pub fn hex_unset(scope: Scope) -> io::Result<()> {
    if scope != Scope::User {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Hex supports user scope only",
        ));
    }
    crate::run("mix", &["hex.config", "mirror_url", "--delete"])
}

pub fn hex_status(scope: Scope) -> io::Result<crate::ToolStatus> {
    if scope != Scope::User {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Hex supports user scope only",
        ));
    }
    let version = crate::command_version("mix")?;
    let value = crate::command_output("mix", &["hex.config", "mirror_url"])
        .unwrap_or_else(|_| "not configured".to_owned());
    Ok(crate::ToolStatus {
        version,
        configured: value != "not configured",
        detail: format!("mirror_url={value}"),
    })
}

pub fn julia_set(mirror: &str, scope: Scope) -> io::Result<()> {
    profile_set(
        "julia",
        scope,
        &format!("export JULIA_PKG_SERVER=\"{mirror}\""),
    )
}

pub fn julia_unset(scope: Scope) -> io::Result<()> {
    profile_unset("julia", scope)
}

pub fn julia_status(scope: Scope) -> io::Result<crate::ToolStatus> {
    profile_status("julia", "julia", "JULIA_PKG_SERVER", scope)
}

pub fn cpan_set(mirror: &str, scope: Scope) -> io::Result<()> {
    profile_set(
        "cpan",
        scope,
        &format!("export PERL_CPAN_MIRROR=\"{mirror}\""),
    )
}

pub fn cpan_unset(scope: Scope) -> io::Result<()> {
    profile_unset("cpan", scope)
}

pub fn cpan_status(scope: Scope) -> io::Result<crate::ToolStatus> {
    profile_status("cpan", "cpan", "PERL_CPAN_MIRROR", scope)
}

pub fn winget_set(mirror: &str, scope: Scope) -> io::Result<()> {
    require_user("winget", scope)?;
    let _ = crate::run("winget", &["source", "remove", "--name", "lazy-mirror"]);
    crate::run(
        "winget",
        &["source", "add", "--name", "lazy-mirror", mirror],
    )
}

pub fn winget_unset(scope: Scope) -> io::Result<()> {
    require_user("winget", scope)?;
    crate::run("winget", &["source", "remove", "--name", "lazy-mirror"])
}

pub fn winget_status(scope: Scope) -> io::Result<crate::ToolStatus> {
    require_user("winget", scope)?;
    let version = crate::command_version("winget")?;
    let detail = crate::command_output("winget", &["source", "list"])
        .unwrap_or_else(|_| "not configured".to_owned());
    Ok(crate::ToolStatus {
        version,
        configured: detail.contains("lazy-mirror"),
        detail,
    })
}

pub fn opam_set(mirror: &str, scope: Scope) -> io::Result<()> {
    require_user("opam", scope)?;
    let _ = crate::run("opam", &["repository", "remove", "lazy-mirror"]);
    crate::run(
        "opam",
        &["repository", "add", "lazy-mirror", mirror, "--set-default"],
    )
}

pub fn opam_unset(scope: Scope) -> io::Result<()> {
    require_user("opam", scope)?;
    crate::run("opam", &["repository", "remove", "lazy-mirror"])
}

pub fn opam_status(scope: Scope) -> io::Result<crate::ToolStatus> {
    require_user("opam", scope)?;
    let version = crate::command_version("opam")?;
    let detail = crate::command_output("opam", &["repository", "list"])
        .unwrap_or_else(|_| "not configured".to_owned());
    Ok(crate::ToolStatus {
        version,
        configured: detail.contains("lazy-mirror"),
        detail,
    })
}

fn profile_set(name: &str, scope: Scope, block: &str) -> io::Result<()> {
    crate::update_named_managed_block(&profile_path(scope)?, name, block)
}

fn profile_unset(name: &str, scope: Scope) -> io::Result<()> {
    crate::remove_named_managed_block(&profile_path(scope)?, name)
}

fn profile_status(
    command: &str,
    name: &str,
    variable: &str,
    scope: Scope,
) -> io::Result<crate::ToolStatus> {
    let version = crate::command_version(command)?;
    let path = profile_path(scope)?;
    let value = std::env::var(variable).ok();
    let marker = format!("# >>> lazy-mirror:{name} >>>");
    let in_profile = std::fs::read_to_string(&path).ok().and_then(|content| {
        content
            .contains(&marker)
            .then(|| content.lines().find(|line| line.contains(variable)))
            .flatten()
            .map(str::to_owned)
    });
    Ok(crate::ToolStatus {
        version,
        configured: value.is_some() || in_profile.is_some(),
        detail: format!(
            "{variable}={}; profile={}",
            value
                .or(in_profile)
                .unwrap_or_else(|| "not configured".to_owned()),
            path.display()
        ),
    })
}

fn profile_path(scope: Scope) -> io::Result<PathBuf> {
    match scope {
        Scope::Project => std::env::current_dir().map(|path| path.join(".env")),
        Scope::User => {
            if let Some(path) = std::env::var_os("LM_SHELL_PROFILE") {
                Ok(path.into())
            } else {
                crate::home_file(".profile")
            }
        }
        Scope::System => Ok(PathBuf::from("/etc/profile")),
    }
}

fn require_system(name: &str, scope: Scope) -> io::Result<()> {
    if scope == Scope::System {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{name} supports system scope only"),
        ))
    }
}

fn require_user(name: &str, scope: Scope) -> io::Result<()> {
    if scope == Scope::User {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{name} supports user scope only"),
        ))
    }
}

fn apt_path() -> io::Result<PathBuf> {
    Ok(std::env::var_os("LM_APT_SOURCES_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/etc/apt/sources.list.d/lazy-mirror.list")))
}

fn apk_path() -> io::Result<PathBuf> {
    Ok(std::env::var_os("LM_APK_REPOSITORIES_FILE")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/etc/apk/repositories")))
}
