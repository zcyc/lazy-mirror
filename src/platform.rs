use std::io;
use std::path::PathBuf;

use crate::config::Scope;

const APT_PREFIX: &str = "# managed by lazy-mirror\n";

pub fn apt_set(mirror: &str, scope: Scope) -> io::Result<()> {
    require_system("apt", scope)?;
    let path = apt_path()?;
    let content = if mirror.starts_with("deb ") || mirror.starts_with("deb-src ") {
        format!("{APT_PREFIX}{mirror}\n")
    } else {
        let suites = std::env::var("LM_APT_SUITES")
            .map(|value| value.split_whitespace().map(str::to_owned).collect())
            .unwrap_or_else(|_| vec![apt_distribution()]);
        let components = std::env::var("LM_APT_COMPONENTS").unwrap_or_else(|_| "main".to_owned());
        let lines = suites
            .iter()
            .map(|suite| format!("deb {mirror} {suite} {components}"))
            .collect::<Vec<_>>()
            .join("\n");
        format!("{APT_PREFIX}{lines}\n")
    };
    crate::write_with_backup_if(&path, &content, |content| content.starts_with(APT_PREFIX))
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
    Ok(crate::ToolStatus::new(
        crate::command_output("apt", &["--version"]).unwrap_or_else(|_| "apt".to_owned()),
        source.is_some(),
        source.clone(),
        Some(path.clone()),
        format!(
            "source={}; config={}",
            source.unwrap_or_else(|| "not configured".to_owned()),
            path.display()
        ),
    ))
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
    Ok(crate::ToolStatus::new(
        crate::command_output("apk", &["--version"]).unwrap_or_else(|_| "apk".to_owned()),
        source.is_some(),
        source.clone(),
        Some(path.clone()),
        format!(
            "repositories={}; config={}",
            source.unwrap_or_else(|| "not configured".to_owned()),
            path.display()
        ),
    ))
}

pub fn env_set(name: &str, variable: &str, mirror: &str, scope: Scope) -> io::Result<()> {
    profile_set(name, scope, &crate::shell_env_assignment(variable, mirror))
}

pub fn env_unset(name: &str, scope: Scope) -> io::Result<()> {
    profile_unset(name, scope)
}

pub fn env_status(
    command: &str,
    name: &str,
    variable: &str,
    expected: &str,
    scope: Scope,
) -> io::Result<crate::ToolStatus> {
    let mut status = profile_status(command, name, variable, scope)?;
    status.configured = status.source.as_deref() == Some(expected);
    Ok(status)
}

pub fn nix_set(mirror: &str, scope: Scope) -> io::Result<()> {
    env_set(
        "nix",
        "NIX_CONFIG",
        &format!("substituters = {mirror}"),
        scope,
    )
}

pub fn nix_unset(scope: Scope) -> io::Result<()> {
    env_unset("nix", scope)
}

pub fn nix_status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let mut status = profile_status("nix", "nix", "NIX_CONFIG", scope)?;
    let source = status
        .source
        .as_deref()
        .and_then(nix_mirror)
        .map(str::to_owned);
    status.configured = source
        .as_deref()
        .is_some_and(|value| value.trim_end_matches('/') == expected.trim_end_matches('/'));
    status.source = source;
    Ok(status)
}

pub fn luarocks_set(mirror: &str, scope: Scope) -> io::Result<()> {
    let path = config_path("luarocks", scope)?;
    crate::write_with_backup_if(
        &path,
        &format!(
            "-- managed by lazy-mirror\nrocks_servers = {{ [\"lazy-mirror\"] = \"{mirror}\" }}\n"
        ),
        |content| content.starts_with("-- managed by lazy-mirror\n"),
    )
}

pub fn luarocks_unset(scope: Scope) -> io::Result<()> {
    crate::remove_with_backup_if(&config_path("luarocks", scope)?, |content| {
        content.starts_with("-- managed by lazy-mirror\n")
    })
}

pub fn luarocks_status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let path = config_path("luarocks", scope)?;
    file_status("luarocks", &path, expected, |content| {
        content
            .split_once("[\"lazy-mirror\"] = \"")
            .and_then(|(_, value)| value.split_once('"').map(|(value, _)| value.to_owned()))
    })
}

pub fn clojure_set(mirror: &str, scope: Scope) -> io::Result<()> {
    let path = config_path("clojure", scope)?;
    crate::write_with_backup_if(
        &path,
        &format!(
            ";; managed by lazy-mirror\n{{:mvn/repos {{:central {{:url \"https://repo.maven.apache.org/maven2/\"}} :clojars {{:url \"{mirror}\"}}}}}}\n"
        ),
        |content| content.starts_with(";; managed by lazy-mirror\n"),
    )
    .map_err(|error| {
        if error.kind() == io::ErrorKind::AlreadyExists {
            io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!(
                    "{} already contains unmanaged EDN; set LM_CLOJURE_CONFIG to a dedicated file",
                    path.display()
                ),
            )
        } else {
            error
        }
    })
}

pub fn clojure_unset(scope: Scope) -> io::Result<()> {
    crate::remove_with_backup_if(&config_path("clojure", scope)?, |content| {
        content.starts_with(";; managed by lazy-mirror\n")
    })
}

pub fn clojure_status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let path = config_path("clojure", scope)?;
    file_status("clojure", &path, expected, |content| {
        content
            .split_once(":clojars {:url \"")
            .and_then(|(_, value)| value.split_once('"').map(|(value, _)| value.to_owned()))
    })
}

pub fn cabal_set(mirror: &str, scope: Scope) -> io::Result<()> {
    let path = config_path("cabal", scope)?;
    crate::write_with_backup_if(
        &path,
        &format!(
            "-- managed by lazy-mirror\nrepository hackage.haskell.org\n  url: {mirror}\n  secure: True\n"
        ),
        |content| content.starts_with("-- managed by lazy-mirror\n"),
    )
}

pub fn cabal_unset(scope: Scope) -> io::Result<()> {
    crate::remove_with_backup_if(&config_path("cabal", scope)?, |content| {
        content.starts_with("-- managed by lazy-mirror\n")
    })
}

pub fn cabal_status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let path = config_path("cabal", scope)?;
    file_status("cabal", &path, expected, |content| {
        content
            .lines()
            .find_map(|line| line.trim().strip_prefix("url:").map(str::trim))
            .map(str::to_owned)
    })
}

pub fn stack_set(mirror: &str, scope: Scope) -> io::Result<()> {
    let path = config_path("stack", scope)?;
    crate::write_with_backup_if(
        &path,
        &format!("# managed by lazy-mirror\npackage-indices:\n- download-prefix: {mirror}\n"),
        |content| content.starts_with("# managed by lazy-mirror\n"),
    )
}

pub fn stack_unset(scope: Scope) -> io::Result<()> {
    crate::remove_with_backup_if(&config_path("stack", scope)?, |content| {
        content.starts_with("# managed by lazy-mirror\n")
    })
}

pub fn stack_status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    let path = config_path("stack", scope)?;
    file_status("stack", &path, expected, |content| {
        content
            .lines()
            .find_map(|line| {
                line.trim()
                    .strip_prefix("- download-prefix:")
                    .map(str::trim)
            })
            .map(str::to_owned)
    })
}

pub fn cocoapods_set(mirror: &str, scope: Scope) -> io::Result<()> {
    require_user("cocoapods", scope)?;
    let _ = crate::run("pod", &["repo", "remove", "lazy-mirror"]);
    crate::run("pod", &["repo", "add", "lazy-mirror", mirror])
}

pub fn cocoapods_unset(scope: Scope) -> io::Result<()> {
    require_user("cocoapods", scope)?;
    crate::run("pod", &["repo", "remove", "lazy-mirror"])
}

pub fn cocoapods_status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    require_user("cocoapods", scope)?;
    let version = crate::command_version("pod")?;
    let detail = crate::command_output("pod", &["repo", "list"])
        .unwrap_or_else(|_| "not configured".to_owned());
    Ok(crate::ToolStatus::new(
        version,
        detail.contains("lazy-mirror") && detail.contains(expected),
        None,
        None,
        detail,
    ))
}

pub fn flatpak_set(mirror: &str, scope: Scope) -> io::Result<()> {
    require_user("flatpak", scope)?;
    crate::run(
        "flatpak",
        &flatpak_args(&["remote-modify", "--url", mirror, "flathub"]),
    )
    .or_else(|_| {
        crate::run(
            "flatpak",
            &flatpak_args(&["remote-add", "--if-not-exists", "flathub", mirror]),
        )
    })
}

pub fn flatpak_unset(scope: Scope) -> io::Result<()> {
    require_user("flatpak", scope)?;
    crate::run(
        "flatpak",
        &flatpak_args(&[
            "remote-modify",
            "--url",
            "https://dl.flathub.org/repo/",
            "flathub",
        ]),
    )
}

pub fn flatpak_status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    require_user("flatpak", scope)?;
    let version = crate::command_version("flatpak")?;
    let detail =
        crate::command_output("flatpak", &flatpak_args(&["remotes", "--columns=name,url"]))
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

fn flatpak_args<'a>(args: &[&'a str]) -> Vec<&'a str> {
    let mut scoped = Vec::with_capacity(args.len() + 1);
    scoped.push("--user");
    scoped.extend_from_slice(args);
    scoped
}

pub fn emacs_set(mirror: &str, scope: Scope) -> io::Result<()> {
    require_user("emacs", scope)?;
    let path = config_path("emacs", scope)?;
    crate::update_named_block(
        &path,
        "emacs",
        ";;",
        &format!("(setq package-archives '((\"mirror\" . \"{mirror}\")))"),
    )
}

pub fn emacs_unset(scope: Scope) -> io::Result<()> {
    require_user("emacs", scope)?;
    crate::remove_named_block(&config_path("emacs", scope)?, "emacs", ";;")
}

pub fn emacs_status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    require_user("emacs", scope)?;
    let path = config_path("emacs", scope)?;
    file_status("emacs", &path, expected, |content| {
        content
            .split_once("(\"mirror\" . \"")
            .and_then(|(_, value)| value.split_once('"').map(|(value, _)| value.to_owned()))
    })
}

pub fn tex_set(mirror: &str, scope: Scope) -> io::Result<()> {
    require_user("tex", scope)?;
    crate::run("tlmgr", &["option", "repository", mirror])
}

pub fn tex_unset(scope: Scope) -> io::Result<()> {
    require_user("tex", scope)?;
    crate::run(
        "tlmgr",
        &[
            "option",
            "repository",
            "https://mirror.ctan.org/systems/texlive/tlnet",
        ],
    )
}

pub fn tex_status(expected: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    require_user("tex", scope)?;
    let version = crate::command_version("tlmgr")?;
    let detail = crate::command_output("tlmgr", &["option", "repository"])
        .unwrap_or_else(|_| "not configured".to_owned());
    let source = detail
        .lines()
        .find_map(|line| line.find("http").map(|index| &line[index..]))
        .map(str::trim)
        .map(str::to_owned);
    Ok(crate::ToolStatus::new(
        version,
        source.as_deref() == Some(expected),
        source,
        None,
        detail,
    ))
}

pub fn os_set(name: &str, mirror: &str, scope: Scope) -> io::Result<()> {
    validate_os_scope(name, scope)?;
    let path = os_path(name)?;
    if name == "gentoo" {
        return crate::update_named_managed_block(
            &path,
            "gentoo",
            &format!("GENTOO_MIRRORS=\"{mirror}\""),
        );
    }
    let content = os_content(name, mirror);
    crate::write_with_backup_if(&path, &content, |current| {
        current.starts_with(&format!("# managed by lazy-mirror:{name}\n"))
    })
}

pub fn os_unset(name: &str, scope: Scope) -> io::Result<()> {
    validate_os_scope(name, scope)?;
    if name == "gentoo" {
        return crate::remove_named_managed_block(&os_path(name)?, "gentoo");
    }
    crate::remove_with_backup_if(&os_path(name)?, |content| {
        content.starts_with(&format!("# managed by lazy-mirror:{name}\n"))
    })
}

pub fn os_status(name: &str, scope: Scope) -> io::Result<crate::ToolStatus> {
    validate_os_scope(name, scope)?;
    let path = os_path(name)?;
    if name == "gentoo" {
        return file_status("emerge", &path, "", |content| {
            content.lines().find_map(|line| {
                line.trim()
                    .strip_prefix("GENTOO_MIRRORS=\"")
                    .and_then(|value| value.strip_suffix('"'))
                    .map(str::to_owned)
            })
        });
    }
    file_status(name, &path, "", |content| {
        content.lines().find_map(|line| {
            line.split_whitespace()
                .find(|value| value.starts_with("http://") || value.starts_with("https://"))
                .map(str::to_owned)
        })
    })
}

fn file_status<F>(
    command: &str,
    path: &std::path::Path,
    expected: &str,
    source: F,
) -> io::Result<crate::ToolStatus>
where
    F: Fn(&str) -> Option<String>,
{
    let content = std::fs::read_to_string(path).ok();
    let source = content.as_deref().and_then(source);
    let version = crate::command_version(command).unwrap_or_else(|_| command.to_owned());
    Ok(crate::ToolStatus::new(
        version,
        source.as_deref().is_some_and(|value| {
            expected.is_empty() || value.trim_end_matches('/') == expected.trim_end_matches('/')
        }),
        source.clone(),
        Some(path.to_path_buf()),
        format!(
            "source={}; config={}",
            source.unwrap_or_else(|| "not configured".to_owned()),
            path.display()
        ),
    ))
}

fn config_path(name: &str, scope: Scope) -> io::Result<PathBuf> {
    if let Some(path) = std::env::var_os(format!("LM_{}_CONFIG", name.to_uppercase())) {
        return Ok(path.into());
    }
    match scope {
        Scope::Project => {
            let relative = match name {
                "clojure" => ".clojure/deps.edn".to_owned(),
                "emacs" => ".emacs".to_owned(),
                _ => format!(".{name}/config"),
            };
            std::env::current_dir().map(|path| path.join(relative))
        }
        Scope::User => crate::home_file(match name {
            "luarocks" => ".luarocks/config.lua",
            "clojure" => ".clojure/deps.edn",
            "cabal" => ".cabal/config",
            "stack" => ".stack/config.yaml",
            "emacs" => ".emacs",
            _ => ".config/lazy-mirror/config",
        }),
        Scope::System => Ok(PathBuf::from(format!("/etc/{name}/lazy-mirror.conf"))),
    }
}

fn os_path(name: &str) -> io::Result<PathBuf> {
    if let Some(path) = std::env::var_os("LM_OS_SOURCES_FILE") {
        if path.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "LM_OS_SOURCES_FILE cannot be empty",
            ));
        }
        return Ok(path.into());
    }
    if name == "termux" {
        let prefix = std::env::var_os("PREFIX").ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::NotFound,
                "PREFIX is required to configure Termux repositories",
            )
        })?;
        return Ok(PathBuf::from(prefix).join("etc/apt/sources.list"));
    }
    let path = match name {
        "fedora" | "rocky" | "alma" | "openeuler" | "openanolis" => {
            "/etc/yum.repos.d/lazy-mirror.repo"
        }
        "opensuse" => "/etc/zypp/repos.d/lazy-mirror.repo",
        "arch" | "archlinuxcn" | "manjaro" => "/etc/pacman.d/mirrorlist",
        "gentoo" => "/etc/portage/make.conf",
        "voidlinux" => "/etc/xbps.d/00-lazy-mirror.conf",
        "solus" => "/etc/solus/repo.conf",
        "ros" => "/etc/apt/sources.list.d/lazy-mirror-ros.list",
        "openwrt" => "/etc/opkg/distfeeds.conf",
        "freebsd" => "/etc/pkg/FreeBSD.conf",
        "openbsd" => "/etc/installurl",
        "netbsd" => "/etc/pkgin/repositories.conf",
        _ => "/etc/apt/sources.list.d/lazy-mirror.list",
    };
    if name == "msys2" {
        return msys2_path();
    }
    Ok(PathBuf::from(path))
}

fn os_content(name: &str, mirror: &str) -> String {
    let prefix = format!("# managed by lazy-mirror:{name}\n");
    match name {
        "fedora" | "rocky" | "alma" | "openeuler" | "openanolis" => {
            format!("{prefix}[lazy-mirror]\nname=lazy-mirror\nbaseurl={mirror}\nenabled=1\ngpgcheck=1\n")
        }
        "opensuse" => format!("{prefix}[lazy-mirror]\ntype=rpm-md\nbaseurl={mirror}\nenabled=1\n"),
        "arch" | "archlinuxcn" | "manjaro" => {
            format!("{prefix}Server = {mirror}/$repo/os/$arch\n")
        }
        "msys2" => format!("{prefix}Server = {mirror}/$repo/$arch\n"),
        "voidlinux" => format!("{prefix}repository={mirror}/current/$XBPS_ARCH\n"),
        "solus" => format!("{prefix}SolusURL = {mirror}\n"),
        "openwrt" => format!("{prefix}src/gz lazy-mirror {mirror}\n"),
        "freebsd" => format!("{prefix}FreeBSD: {{ url: \"{mirror}\" }}\n"),
        "openbsd" => format!("{prefix}{mirror}\n"),
        "netbsd" => format!("{prefix}{mirror}\n"),
        "termux" => format!("{prefix}{mirror}\n"),
        "ros" => {
            let distribution = std::env::var("LM_ROS_DISTRIBUTION")
                .or_else(|_| std::env::var("ROS_DISTRO"))
                .unwrap_or_else(|_| apt_distribution());
            format!("{prefix}deb {mirror} {distribution} main\n")
        }
        _ => {
            let distribution =
                std::env::var("LM_APT_DISTRIBUTION").unwrap_or_else(|_| "stable".to_owned());
            let components =
                std::env::var("LM_APT_COMPONENTS").unwrap_or_else(|_| "main".to_owned());
            format!("{prefix}deb {mirror} {distribution} {components}\n")
        }
    }
}

fn msys2_path() -> io::Result<PathBuf> {
    if let Some(path) = std::env::var_os("LM_MSYS2_MIRRORLIST") {
        if path.is_empty() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "LM_MSYS2_MIRRORLIST cannot be empty",
            ));
        }
        return Ok(path.into());
    }
    let environment = std::env::var("MSYSTEM").unwrap_or_else(|_| "MSYS".to_owned());
    let specific = match environment.as_str() {
        "MSYS" => "mirrorlist.msys",
        "MINGW32" => "mirrorlist.mingw32",
        "MINGW64" => "mirrorlist.mingw64",
        "UCRT64" => "mirrorlist.ucrt64",
        "CLANG64" => "mirrorlist.clang64",
        "CLANGARM64" => "mirrorlist.clangarm64",
        _ => "mirrorlist.msys",
    };
    let specific = PathBuf::from("/etc/pacman.d").join(specific);
    if specific.exists() || environment == "MSYS" {
        return Ok(specific);
    }
    Ok(PathBuf::from("/etc/pacman.d/mirrorlist.mingw"))
}

fn validate_os_scope(name: &str, scope: Scope) -> io::Result<()> {
    let user_scope = matches!(name, "msys2" | "termux");
    if (user_scope && scope == Scope::User) || (!user_scope && scope == Scope::System) {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "{name} supports {} scope only",
                if user_scope { "user" } else { "system" }
            ),
        ))
    }
}

pub fn brew_set(mirror: &str, scope: Scope) -> io::Result<()> {
    let base = mirror.trim_end_matches('/');
    profile_set(
        "homebrew",
        scope,
        &[
            crate::shell_env_assignment(
                "HOMEBREW_API_DOMAIN",
                &format!("{base}/homebrew-bottles/api"),
            ),
            crate::shell_env_assignment(
                "HOMEBREW_BOTTLE_DOMAIN",
                &format!("{base}/homebrew-bottles"),
            ),
            crate::shell_env_assignment(
                "HOMEBREW_BREW_GIT_REMOTE",
                &format!("{base}/git/homebrew/brew.git"),
            ),
            crate::shell_env_assignment(
                "HOMEBREW_CORE_GIT_REMOTE",
                &format!("{base}/git/homebrew/homebrew-core.git"),
            ),
        ]
        .join("\n"),
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
        &[
            crate::shell_env_assignment("RUSTUP_DIST_SERVER", base),
            crate::shell_env_assignment("RUSTUP_UPDATE_ROOT", &format!("{base}/rustup")),
        ]
        .join("\n"),
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
    let source = (value != "not configured").then(|| value.clone());
    Ok(crate::ToolStatus::new(
        version,
        source.is_some(),
        source,
        None,
        format!("mirror_url={value}"),
    ))
}

pub fn julia_set(mirror: &str, scope: Scope) -> io::Result<()> {
    profile_set(
        "julia",
        scope,
        &crate::shell_env_assignment("JULIA_PKG_SERVER", mirror),
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
        &crate::shell_env_assignment("PERL_CPAN_MIRROR", mirror),
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
    let source = detail
        .lines()
        .skip_while(|line| !line.contains("lazy-mirror"))
        .skip(1)
        .flat_map(str::split_whitespace)
        .find(|value| value.starts_with("http://") || value.starts_with("https://"))
        .map(str::to_owned);
    Ok(crate::ToolStatus::new(
        version,
        detail.contains("lazy-mirror"),
        source,
        None,
        detail,
    ))
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
    let source = detail
        .lines()
        .find(|line| line.contains("lazy-mirror"))
        .and_then(|line| {
            line.split_whitespace()
                .find(|value| value.starts_with("http://") || value.starts_with("https://"))
        })
        .map(str::to_owned);
    Ok(crate::ToolStatus::new(
        version,
        detail.contains("lazy-mirror"),
        source,
        None,
        detail,
    ))
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
    let marker = format!("# >>> lazy-mirror:{name} >>>");
    let in_profile = std::fs::read_to_string(&path).ok().and_then(|content| {
        content
            .contains(&marker)
            .then(|| content.lines().find(|line| line.contains(variable)))
            .flatten()
            .and_then(|line| crate::shell_env_value(line, variable))
    });
    let source = in_profile.clone();
    Ok(crate::ToolStatus {
        version,
        configured: source.is_some(),
        source: source.clone(),
        path: Some(path.clone()),
        detail: format!(
            "{variable}={}; profile={}",
            source.as_deref().unwrap_or("not configured"),
            path.display()
        ),
    })
}

fn nix_mirror(value: &str) -> Option<&str> {
    value.lines().find_map(|line| {
        let (key, value) = line.split_once('=')?;
        (key.trim() == "substituters")
            .then(|| value.split_whitespace().next())
            .flatten()
    })
}

pub fn source_for_restore(name: &str, source: &str) -> String {
    let suffix = match name {
        "arch" | "archlinuxcn" | "manjaro" => Some("/$repo/os/$arch"),
        "msys2" => Some("/$repo/$arch"),
        "voidlinux" => Some("/current/$XBPS_ARCH"),
        "brew" => Some("/homebrew-bottles"),
        _ => None,
    };
    suffix
        .and_then(|suffix| source.strip_suffix(suffix))
        .unwrap_or(source)
        .trim_end_matches('/')
        .to_owned()
}

fn profile_path(scope: Scope) -> io::Result<PathBuf> {
    match scope {
        Scope::Project => std::env::current_dir().map(|path| path.join(".env")),
        Scope::User => {
            if let Some(path) = std::env::var_os("LM_SHELL_PROFILE") {
                Ok(path.into())
            } else {
                #[cfg(windows)]
                {
                    return crate::powershell_profile_path();
                }
                crate::home_file(profile_relative_path(&shell()))
            }
        }
        Scope::System => {
            #[cfg(windows)]
            {
                crate::powershell_system_profile_path()
            }
            #[cfg(not(windows))]
            {
                Ok(PathBuf::from("/etc/profile"))
            }
        }
    }
}

fn shell() -> String {
    std::env::var_os("SHELL")
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn profile_relative_path(shell: &str) -> &'static str {
    if shell.ends_with("/fish") {
        ".config/fish/config.fish"
    } else {
        ".profile"
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

pub(crate) fn apt_distribution() -> String {
    if let Ok(distribution) = std::env::var("LM_APT_DISTRIBUTION") {
        return distribution;
    }
    std::fs::read_to_string("/etc/os-release")
        .ok()
        .and_then(|content| {
            content.lines().find_map(|line| {
                line.strip_prefix("VERSION_CODENAME=")
                    .or_else(|| line.strip_prefix("UBUNTU_CODENAME="))
                    .map(|value| value.trim_matches('"').to_owned())
            })
        })
        .unwrap_or_else(|| "stable".to_owned())
}

#[cfg(test)]
mod tests {
    use super::{flatpak_args, nix_mirror, os_content, profile_relative_path, source_for_restore};

    #[test]
    fn msys2_uses_pacman_repository_layout() {
        assert_eq!(
            os_content("msys2", "https://mirror.example"),
            "# managed by lazy-mirror:msys2\nServer = https://mirror.example/$repo/$arch\n"
        );
    }

    #[test]
    fn rendered_sources_are_normalized_before_restore() {
        assert_eq!(
            source_for_restore("arch", "https://mirror.example/$repo/os/$arch"),
            "https://mirror.example"
        );
        assert_eq!(
            source_for_restore("brew", "https://mirror.example/homebrew-bottles"),
            "https://mirror.example"
        );
    }

    #[test]
    fn nix_config_source_extracts_substituter() {
        assert_eq!(
            nix_mirror("substituters = https://cache.example"),
            Some("https://cache.example")
        );
        assert_eq!(nix_mirror("https://cache.example"), None);
    }

    #[test]
    fn fish_uses_its_startup_configuration_file() {
        assert_eq!(
            profile_relative_path("/usr/bin/fish"),
            ".config/fish/config.fish"
        );
        assert_eq!(profile_relative_path("/bin/bash"), ".profile");
    }

    #[test]
    fn flatpak_commands_use_user_scope() {
        assert_eq!(
            flatpak_args(&[
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
