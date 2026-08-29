use std::collections::{BTreeMap, BTreeSet};
use std::io;
use std::path::PathBuf;
use std::process;
use std::thread;

use clap::{Parser, Subcommand, ValueEnum};

use lm::config::{Config, Scope};

#[derive(Debug, Parser)]
#[command(name = "lm", version, about = "Change package and software sources")]
struct Cli {
    #[arg(long, global = true, value_name = "FILE")]
    config: Option<PathBuf>,
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(about = "List targets and mirror sources", visible_aliases = ["ls", "l"])]
    List {
        query: Option<String>,
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
    #[command(about = "Measure mirror availability and latency", visible_aliases = ["m", "cesu"])]
    Measure {
        target: Target,
        mirror: Option<String>,
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
        #[arg(long)]
        cache_ttl: Option<u64>,
        #[arg(long)]
        no_cache: bool,
        #[arg(long)]
        only_installed: bool,
        #[arg(long)]
        parallelism: Option<usize>,
    },
    #[command(about = "Check mirror protocol endpoints", visible_alias = "verify")]
    Check {
        target: Target,
        mirror: Option<String>,
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
        #[arg(long)]
        cache_ttl: Option<u64>,
        #[arg(long)]
        no_cache: bool,
        #[arg(long)]
        only_installed: bool,
        #[arg(long)]
        parallelism: Option<usize>,
    },
    #[command(about = "Show the current source", visible_alias = "g")]
    Get {
        target: Target,
        #[arg(long, value_enum, default_value = "user")]
        scope: Scope,
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
        #[arg(long)]
        only_installed: bool,
    },
    #[command(about = "Set a source, mirror name, or URL", visible_alias = "s")]
    Set {
        target: Target,
        mirror: Option<String>,
        #[arg(long, value_enum, default_value = "user")]
        scope: Scope,
        #[arg(long, visible_alias = "dry")]
        dry_run: bool,
        #[arg(long)]
        atomic: bool,
    },
    #[command(about = "Reset to the upstream source", visible_alias = "r")]
    Reset {
        target: Target,
        #[arg(long, value_enum, default_value = "user")]
        scope: Scope,
        #[arg(long, visible_alias = "dry")]
        dry_run: bool,
    },
    #[command(about = "Validate or show the effective TOML configuration")]
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    #[command(about = "Check tools, configuration and selected mirror")]
    Doctor {
        target: Target,
        mirror: Option<String>,
        #[arg(long, value_enum, default_value = "user")]
        scope: Scope,
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
        #[arg(long)]
        cache_ttl: Option<u64>,
        #[arg(long)]
        no_cache: bool,
        #[arg(long)]
        only_installed: bool,
        #[arg(long)]
        parallelism: Option<usize>,
    },
    #[command(about = "Show the exact source change plan", visible_alias = "diff")]
    Plan {
        target: Target,
        mirror: Option<String>,
        #[arg(long)]
        reset: bool,
        #[arg(long, value_enum, default_value = "user")]
        scope: Scope,
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
        #[arg(long)]
        only_installed: bool,
    },
}

#[derive(Debug, Subcommand)]
enum ConfigCommand {
    #[command(about = "Validate the TOML configuration")]
    Validate,
    #[command(about = "Show effective configuration")]
    Show {
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
#[value(rename_all = "lower")]
enum OutputFormat {
    Table,
    Json,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
#[value(rename_all = "lower")]
enum Target {
    All,
    Npm,
    Pnpm,
    Yarn,
    Bun,
    #[value(alias = "nodejs")]
    Node,
    Go,
    #[value(alias = "py", alias = "pypi")]
    Pip,
    Pip3,
    Python,
    Uv,
    Pdm,
    Poetry,
    Composer,
    Php,
    #[value(alias = "rb", alias = "rubygems")]
    Gem,
    #[value(alias = "bundler")]
    Bundle,
    Ruby,
    #[value(alias = "mvn", alias = "maven-daemon", alias = "mvnd")]
    Maven,
    Gradle,
    Sbt,
    Java,
    #[value(alias = "crate")]
    Cargo,
    Rust,
    #[value(alias = "dockerhub")]
    Docker,
    Containerd,
    Nerdctl,
    Podman,
    #[value(alias = "anaconda")]
    Conda,
    Mamba,
    #[value(alias = "pub")]
    Dart,
    Flutter,
    Nuget,
    Dotnet,
    Cran,
    R,
    #[value(alias = "hf", alias = "huggingface-hub")]
    Huggingface,
    #[value(alias = "debian", alias = "ubuntu")]
    Apt,
    #[value(alias = "alpine")]
    Apk,
    #[value(alias = "homebrew")]
    Brew,
    Rustup,
    #[value(alias = "mix")]
    Hex,
    Julia,
    #[value(alias = "perl")]
    Cpan,
    Winget,
    Opam,
    #[value(alias = "python-rye")]
    Rye,
    Nvm,
    #[value(alias = "lua")]
    Luarocks,
    #[value(alias = "clojars")]
    Clojure,
    Haskell,
    Hackage,
    Cabal,
    Stack,
    #[value(alias = "ocaml")]
    Ocaml,
    #[value(alias = "cocoa", alias = "pod")]
    Cocoapods,
    #[value(alias = "flatpak")]
    Flathub,
    Nix,
    Guix,
    #[value(alias = "elpa")]
    Emacs,
    #[value(alias = "ctan", alias = "latex", alias = "texlive", alias = "miktex")]
    Tex,
    #[value(alias = "mint", alias = "zorinos")]
    Linuxmint,
    Fedora,
    #[value(alias = "suse")]
    Opensuse,
    Kali,
    #[value(alias = "archlinux")]
    Arch,
    Archlinuxcn,
    Manjaro,
    Gentoo,
    #[value(alias = "rockylinux")]
    Rocky,
    #[value(alias = "almalinux")]
    Alma,
    #[value(alias = "void")]
    Voidlinux,
    Solus,
    #[value(alias = "ros2")]
    Ros,
    Trisquel,
    #[value(alias = "lite")]
    Linuxlite,
    #[value(alias = "raspberrypi")]
    Raspi,
    Armbian,
    Openwrt,
    Openeuler,
    #[value(alias = "anolis")]
    Openanolis,
    Openkylin,
    Deepin,
    #[value(alias = "msys")]
    Msys2,
    Termux,
    Freebsd,
    Openbsd,
    Netbsd,
}

const ALL_TARGETS: &[Target] = &[
    Target::Npm,
    Target::Pnpm,
    Target::Yarn,
    Target::Bun,
    Target::Go,
    Target::Pip,
    Target::Uv,
    Target::Pdm,
    Target::Poetry,
    Target::Composer,
    Target::Maven,
    Target::Gradle,
    Target::Sbt,
    Target::Gem,
    Target::Bundle,
    Target::Cargo,
    Target::Docker,
    Target::Containerd,
    Target::Podman,
    Target::Conda,
    Target::Dart,
    Target::Flutter,
    Target::Cran,
    Target::Huggingface,
    Target::Nuget,
    Target::Apt,
    Target::Apk,
    Target::Brew,
    Target::Rustup,
    Target::Hex,
    Target::Julia,
    Target::Cpan,
    Target::Winget,
    Target::Opam,
    Target::Rye,
    Target::Nvm,
    Target::Luarocks,
    Target::Clojure,
    Target::Haskell,
    Target::Cabal,
    Target::Stack,
    Target::Cocoapods,
    Target::Flathub,
    Target::Nix,
    Target::Guix,
    Target::Emacs,
    Target::Tex,
    Target::Linuxmint,
    Target::Fedora,
    Target::Opensuse,
    Target::Kali,
    Target::Arch,
    Target::Archlinuxcn,
    Target::Manjaro,
    Target::Gentoo,
    Target::Rocky,
    Target::Alma,
    Target::Voidlinux,
    Target::Solus,
    Target::Ros,
    Target::Trisquel,
    Target::Linuxlite,
    Target::Raspi,
    Target::Armbian,
    Target::Openwrt,
    Target::Openeuler,
    Target::Openanolis,
    Target::Openkylin,
    Target::Deepin,
    Target::Msys2,
    Target::Termux,
    Target::Freebsd,
    Target::Openbsd,
    Target::Netbsd,
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Action {
    Set,
    Reset,
}

#[derive(Debug)]
struct StatusRecord {
    target: String,
    scope: String,
    configured: bool,
    version: Option<String>,
    source: Option<String>,
    path: Option<String>,
    origin: String,
    detail: Option<String>,
    error: Option<String>,
}

#[derive(Debug)]
struct MeasureRecord {
    target: String,
    mirror: String,
    url: String,
    probe_url: Option<String>,
    code: Option<String>,
    state: String,
    detail: Option<String>,
    milliseconds: Option<u128>,
    cached: bool,
    error: Option<String>,
}

#[derive(Clone, Copy)]
struct ProbeOptions {
    format: OutputFormat,
    cache_ttl: Option<u64>,
    no_cache: bool,
    only_installed: bool,
    parallelism: Option<usize>,
}

fn target_name(target: Target) -> &'static str {
    match target {
        Target::All => "all",
        Target::Npm => "npm",
        Target::Pnpm => "pnpm",
        Target::Yarn => "yarn",
        Target::Bun => "bun",
        Target::Node => "node",
        Target::Go => "go",
        Target::Pip => "pip",
        Target::Pip3 => "pip3",
        Target::Python => "python",
        Target::Uv => "uv",
        Target::Pdm => "pdm",
        Target::Poetry => "poetry",
        Target::Composer => "composer",
        Target::Php => "php",
        Target::Gem => "gem",
        Target::Bundle => "bundle",
        Target::Ruby => "ruby",
        Target::Maven => "maven",
        Target::Gradle => "gradle",
        Target::Sbt => "sbt",
        Target::Java => "java",
        Target::Cargo => "cargo",
        Target::Rust => "rust",
        Target::Docker => "docker",
        Target::Containerd => "containerd",
        Target::Nerdctl => "nerdctl",
        Target::Podman => "podman",
        Target::Conda => "conda",
        Target::Mamba => "mamba",
        Target::Dart => "dart",
        Target::Flutter => "flutter",
        Target::Nuget => "nuget",
        Target::Dotnet => "dotnet",
        Target::Cran => "cran",
        Target::R => "r",
        Target::Huggingface => "huggingface",
        Target::Apt => "apt",
        Target::Apk => "apk",
        Target::Brew => "brew",
        Target::Rustup => "rustup",
        Target::Hex => "hex",
        Target::Julia => "julia",
        Target::Cpan => "cpan",
        Target::Winget => "winget",
        Target::Opam => "opam",
        Target::Rye => "rye",
        Target::Nvm => "nvm",
        Target::Luarocks => "luarocks",
        Target::Clojure => "clojure",
        Target::Haskell => "haskell",
        Target::Hackage => "hackage",
        Target::Cabal => "cabal",
        Target::Stack => "stack",
        Target::Ocaml => "ocaml",
        Target::Cocoapods => "cocoapods",
        Target::Flathub => "flathub",
        Target::Nix => "nix",
        Target::Guix => "guix",
        Target::Emacs => "emacs",
        Target::Tex => "tex",
        Target::Linuxmint => "linuxmint",
        Target::Fedora => "fedora",
        Target::Opensuse => "opensuse",
        Target::Kali => "kali",
        Target::Arch => "arch",
        Target::Archlinuxcn => "archlinuxcn",
        Target::Manjaro => "manjaro",
        Target::Gentoo => "gentoo",
        Target::Rocky => "rocky",
        Target::Alma => "alma",
        Target::Voidlinux => "voidlinux",
        Target::Solus => "solus",
        Target::Ros => "ros",
        Target::Trisquel => "trisquel",
        Target::Linuxlite => "linuxlite",
        Target::Raspi => "raspi",
        Target::Armbian => "armbian",
        Target::Openwrt => "openwrt",
        Target::Openeuler => "openeuler",
        Target::Openanolis => "openanolis",
        Target::Openkylin => "openkylin",
        Target::Deepin => "deepin",
        Target::Msys2 => "msys2",
        Target::Termux => "termux",
        Target::Freebsd => "freebsd",
        Target::Openbsd => "openbsd",
        Target::Netbsd => "netbsd",
    }
}

fn catalog_name(target: Target) -> &'static str {
    match target {
        Target::Node => "npm",
        Target::Pip3 | Target::Python => "pip",
        Target::Php => "composer",
        Target::Ruby => "gem",
        Target::Java => "maven",
        Target::Rust => "cargo",
        Target::Nerdctl => "containerd",
        Target::Mamba => "conda",
        Target::Dotnet => "nuget",
        Target::R => "cran",
        Target::Huggingface => "huggingface",
        Target::Clojure => "clojure",
        Target::Haskell | Target::Hackage | Target::Cabal => "cabal",
        Target::Ocaml => "opam",
        Target::Luarocks => "luarocks",
        target @ (Target::Linuxmint
        | Target::Fedora
        | Target::Opensuse
        | Target::Kali
        | Target::Arch
        | Target::Archlinuxcn
        | Target::Manjaro
        | Target::Gentoo
        | Target::Rocky
        | Target::Alma
        | Target::Voidlinux
        | Target::Solus
        | Target::Ros
        | Target::Trisquel
        | Target::Linuxlite
        | Target::Raspi
        | Target::Armbian
        | Target::Openwrt
        | Target::Openeuler
        | Target::Openanolis
        | Target::Openkylin
        | Target::Deepin
        | Target::Msys2
        | Target::Termux
        | Target::Freebsd
        | Target::Openbsd
        | Target::Netbsd) => target_name(target),
        target => target_name(target),
    }
}

fn run_action(
    target: Target,
    action: Action,
    mirror: Option<&str>,
    scope: Scope,
) -> io::Result<()> {
    match target {
        Target::Npm | Target::Pnpm | Target::Yarn | Target::Bun => {
            let name = target_name(target);
            match action {
                Action::Set => lm::node::set(name, mirror.unwrap(), scope),
                Action::Reset => lm::node::unset(name, scope),
            }
        }
        Target::Node => run_group(
            action,
            mirror,
            scope,
            &[Target::Npm, Target::Pnpm, Target::Yarn],
        ),
        Target::Go => match action {
            Action::Set => lm::go::set(mirror.unwrap()),
            Action::Reset => lm::go::unset(),
        },
        Target::Pip | Target::Pip3 => {
            let name = if target == Target::Pip { "pip" } else { "pip3" };
            match action {
                Action::Set => lm::python::set(name, mirror.unwrap()),
                Action::Reset => lm::python::unset(name),
            }
        }
        Target::Python => run_group(
            action,
            mirror,
            scope,
            &[Target::Pip, Target::Uv, Target::Pdm, Target::Poetry],
        ),
        Target::Uv => match action {
            Action::Set => lm::uv::set(mirror.unwrap(), scope),
            Action::Reset => lm::uv::unset(scope),
        },
        Target::Pdm => match action {
            Action::Set => lm::pdm::set(mirror.unwrap()),
            Action::Reset => lm::pdm::unset(),
        },
        Target::Poetry => match action {
            Action::Set => lm::poetry::set(mirror.unwrap(), scope),
            Action::Reset => lm::poetry::unset(scope),
        },
        Target::Composer | Target::Php => match action {
            Action::Set => lm::php::set(mirror.unwrap()),
            Action::Reset => lm::php::unset(),
        },
        Target::Gem | Target::Ruby => match action {
            Action::Set => lm::ruby::gem_set(mirror.unwrap()),
            Action::Reset => lm::ruby::gem_unset(),
        },
        Target::Bundle => match action {
            Action::Set => lm::ruby::bundle_set(mirror.unwrap()),
            Action::Reset => lm::ruby::bundle_unset(),
        },
        Target::Maven => match action {
            Action::Set => lm::java::maven_set(mirror.unwrap()),
            Action::Reset => lm::java::maven_unset(),
        },
        Target::Java => run_group(
            action,
            mirror,
            scope,
            &[Target::Maven, Target::Gradle, Target::Sbt],
        ),
        Target::Gradle => match action {
            Action::Set => lm::java::gradle_set(mirror.unwrap()),
            Action::Reset => lm::java::gradle_unset(),
        },
        Target::Sbt => match action {
            Action::Set => lm::sbt::set(mirror.unwrap()),
            Action::Reset => lm::sbt::unset(),
        },
        Target::Cargo => match action {
            Action::Set => lm::rust::set(mirror.unwrap(), scope),
            Action::Reset => lm::rust::unset(scope),
        },
        Target::Rust => run_group(action, mirror, scope, &[Target::Cargo, Target::Rustup]),
        Target::Docker => match action {
            Action::Set => lm::docker::set(mirror.unwrap()),
            Action::Reset => lm::docker::unset(),
        },
        Target::Containerd | Target::Nerdctl => match action {
            Action::Set => lm::container::containerd_set(mirror.unwrap()),
            Action::Reset => lm::container::containerd_unset(),
        },
        Target::Podman => match action {
            Action::Set => lm::container::podman_set(mirror.unwrap()),
            Action::Reset => lm::container::podman_unset(),
        },
        Target::Conda | Target::Mamba => {
            let name = target_name(target);
            match action {
                Action::Set => lm::conda::set(name, mirror.unwrap()),
                Action::Reset => lm::conda::unset(name),
            }
        }
        Target::Dart => run_dart_group(action, mirror, scope),
        Target::Flutter => match action {
            Action::Set => lm::dart::flutter_set(mirror.unwrap(), scope),
            Action::Reset => lm::dart::unset(scope),
        },
        Target::Nuget | Target::Dotnet => match action {
            Action::Set => lm::nuget::set(mirror.unwrap(), scope),
            Action::Reset => lm::nuget::unset(scope),
        },
        Target::Cran | Target::R => match action {
            Action::Set => lm::r::set(mirror.unwrap()),
            Action::Reset => lm::r::unset(),
        },
        Target::Huggingface => match action {
            Action::Set => lm::huggingface::set(mirror.unwrap(), scope),
            Action::Reset => lm::huggingface::unset(scope),
        },
        Target::Apt => match action {
            Action::Set => lm::platform::apt_set(mirror.unwrap(), scope),
            Action::Reset => lm::platform::apt_unset(scope),
        },
        Target::Apk => match action {
            Action::Set => lm::platform::apk_set(mirror.unwrap(), scope),
            Action::Reset => lm::platform::apk_unset(scope),
        },
        Target::Brew => match action {
            Action::Set => lm::platform::brew_set(mirror.unwrap(), scope),
            Action::Reset => lm::platform::brew_unset(scope),
        },
        Target::Rustup => match action {
            Action::Set => lm::platform::rustup_set(mirror.unwrap(), scope),
            Action::Reset => lm::platform::rustup_unset(scope),
        },
        Target::Hex => match action {
            Action::Set => lm::platform::hex_set(mirror.unwrap(), scope),
            Action::Reset => lm::platform::hex_unset(scope),
        },
        Target::Julia => match action {
            Action::Set => lm::platform::julia_set(mirror.unwrap(), scope),
            Action::Reset => lm::platform::julia_unset(scope),
        },
        Target::Cpan => match action {
            Action::Set => lm::platform::cpan_set(mirror.unwrap(), scope),
            Action::Reset => lm::platform::cpan_unset(scope),
        },
        Target::Winget => match action {
            Action::Set => lm::platform::winget_set(mirror.unwrap(), scope),
            Action::Reset => lm::platform::winget_unset(scope),
        },
        Target::Opam => match action {
            Action::Set => lm::platform::opam_set(mirror.unwrap(), scope),
            Action::Reset => lm::platform::opam_unset(scope),
        },
        Target::Rye => match action {
            Action::Set => lm::platform::env_set("rye", "RYE_PYPI_MIRROR", mirror.unwrap(), scope),
            Action::Reset => lm::platform::env_unset("rye", scope),
        },
        Target::Nvm => match action {
            Action::Set => {
                lm::platform::env_set("nvm", "NVM_NODEJS_ORG_MIRROR", mirror.unwrap(), scope)
            }
            Action::Reset => lm::platform::env_unset("nvm", scope),
        },
        Target::Luarocks => match action {
            Action::Set => lm::platform::luarocks_set(mirror.unwrap(), scope),
            Action::Reset => lm::platform::luarocks_unset(scope),
        },
        Target::Clojure => match action {
            Action::Set => lm::platform::clojure_set(mirror.unwrap(), scope),
            Action::Reset => lm::platform::clojure_unset(scope),
        },
        Target::Haskell => run_group(action, mirror, scope, &[Target::Cabal, Target::Stack]),
        Target::Hackage | Target::Cabal => match action {
            Action::Set => lm::platform::cabal_set(mirror.unwrap(), scope),
            Action::Reset => lm::platform::cabal_unset(scope),
        },
        Target::Stack => match action {
            Action::Set => lm::platform::stack_set(mirror.unwrap(), scope),
            Action::Reset => lm::platform::stack_unset(scope),
        },
        Target::Ocaml => match action {
            Action::Set => lm::platform::opam_set(mirror.unwrap(), scope),
            Action::Reset => lm::platform::opam_unset(scope),
        },
        Target::Cocoapods => match action {
            Action::Set => lm::platform::cocoapods_set(mirror.unwrap(), scope),
            Action::Reset => lm::platform::cocoapods_unset(scope),
        },
        Target::Flathub => match action {
            Action::Set => lm::platform::flatpak_set(mirror.unwrap(), scope),
            Action::Reset => lm::platform::flatpak_unset(scope),
        },
        Target::Nix => match action {
            Action::Set => lm::platform::env_set("nix", "NIX_CONFIG", mirror.unwrap(), scope),
            Action::Reset => lm::platform::env_unset("nix", scope),
        },
        Target::Guix => match action {
            Action::Set => {
                lm::platform::env_set("guix", "GUIX_SUBSTITUTE_URLS", mirror.unwrap(), scope)
            }
            Action::Reset => lm::platform::env_unset("guix", scope),
        },
        Target::Emacs => match action {
            Action::Set => lm::platform::emacs_set(mirror.unwrap(), scope),
            Action::Reset => lm::platform::emacs_unset(scope),
        },
        Target::Tex => match action {
            Action::Set => lm::platform::tex_set(mirror.unwrap(), scope),
            Action::Reset => lm::platform::tex_unset(scope),
        },
        target @ (Target::Linuxmint
        | Target::Fedora
        | Target::Opensuse
        | Target::Kali
        | Target::Arch
        | Target::Archlinuxcn
        | Target::Manjaro
        | Target::Gentoo
        | Target::Rocky
        | Target::Alma
        | Target::Voidlinux
        | Target::Solus
        | Target::Ros
        | Target::Trisquel
        | Target::Linuxlite
        | Target::Raspi
        | Target::Armbian
        | Target::Openwrt
        | Target::Openeuler
        | Target::Openanolis
        | Target::Openkylin
        | Target::Deepin
        | Target::Msys2
        | Target::Termux
        | Target::Freebsd
        | Target::Openbsd
        | Target::Netbsd) => match action {
            Action::Set => lm::platform::os_set(target_name(target), mirror.unwrap(), scope),
            Action::Reset => lm::platform::os_unset(target_name(target), scope),
        },
        Target::All => unreachable!(),
    }
}

fn run_group(
    action: Action,
    mirror: Option<&str>,
    scope: Scope,
    targets: &[Target],
) -> io::Result<()> {
    let mut applied = false;
    for &target in targets {
        if is_installed(target) && validate_scope(target, scope).is_ok() {
            run_action(target, action, mirror, scope)?;
            applied = true;
        }
    }
    if applied {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "none of the grouped tools are installed",
        ))
    }
}

fn run_dart_group(action: Action, mirror: Option<&str>, scope: Scope) -> io::Result<()> {
    let mut applied = false;
    if is_installed(Target::Dart) {
        match action {
            Action::Set => lm::dart::dart_set(mirror.unwrap(), scope)?,
            Action::Reset => lm::dart::unset(scope)?,
        }
        applied = true;
    }
    if is_installed(Target::Flutter) {
        match action {
            Action::Set => lm::dart::flutter_set(mirror.unwrap(), scope)?,
            Action::Reset => lm::dart::unset(scope)?,
        }
        applied = true;
    }
    if applied {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "neither dart nor flutter is installed",
        ))
    }
}

fn execute(
    target: Target,
    action: Action,
    selector: Option<&str>,
    scope: Scope,
    dry_run: bool,
    config: &Config,
) -> io::Result<()> {
    validate_scope(target, scope)?;
    let mirror = match action {
        Action::Set => Some(lm::catalog::resolve(
            catalog_name(target),
            selector,
            config,
        )?),
        Action::Reset => None,
    };
    execute_resolved(target, action, mirror.as_deref(), scope, dry_run)
}

fn execute_resolved(
    target: Target,
    action: Action,
    mirror: Option<&str>,
    scope: Scope,
    dry_run: bool,
) -> io::Result<()> {
    if dry_run {
        match action {
            Action::Set => println!(
                "would set {} mirror to {} (scope={scope:?})",
                target_name(target),
                redact_url(mirror.unwrap_or_default())
            ),
            Action::Reset => println!(
                "would reset {} mirror (scope={scope:?})",
                target_name(target)
            ),
        }
        return Ok(());
    }
    run_action(target, action, mirror, scope)?;
    let verb = match action {
        Action::Set => "set",
        Action::Reset => "reset",
    };
    println!("{verb} {} mirror", target_name(target));
    Ok(())
}

fn validate_scope(target: Target, scope: Scope) -> io::Result<()> {
    let supported = match scope {
        Scope::User => !is_system_os(target),
        Scope::Project => matches!(
            target,
            Target::Npm
                | Target::Pnpm
                | Target::Yarn
                | Target::Bun
                | Target::Node
                | Target::Uv
                | Target::Poetry
                | Target::Cargo
                | Target::Rust
                | Target::Dart
                | Target::Flutter
                | Target::Huggingface
                | Target::Nuget
                | Target::Dotnet
                | Target::Brew
                | Target::Rustup
                | Target::Julia
                | Target::Cpan
                | Target::Rye
                | Target::Nvm
                | Target::Luarocks
                | Target::Clojure
                | Target::Haskell
                | Target::Hackage
                | Target::Cabal
                | Target::Stack
        ),
        Scope::System => matches!(
            target,
            Target::Cargo
                | Target::Rust
                | Target::Dart
                | Target::Flutter
                | Target::Huggingface
                | Target::Docker
                | Target::Apt
                | Target::Apk
                | Target::Brew
                | Target::Rustup
                | Target::Julia
                | Target::Cpan
                | Target::Nix
                | Target::Guix
                | Target::Luarocks
                | Target::Clojure
                | Target::Haskell
                | Target::Hackage
                | Target::Cabal
                | Target::Stack
                | Target::Linuxmint
                | Target::Fedora
                | Target::Opensuse
                | Target::Kali
                | Target::Arch
                | Target::Archlinuxcn
                | Target::Manjaro
                | Target::Gentoo
                | Target::Rocky
                | Target::Alma
                | Target::Voidlinux
                | Target::Solus
                | Target::Ros
                | Target::Trisquel
                | Target::Linuxlite
                | Target::Raspi
                | Target::Armbian
                | Target::Openwrt
                | Target::Openeuler
                | Target::Openanolis
                | Target::Openkylin
                | Target::Deepin
        ),
    };
    if supported {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} does not support {scope:?} scope", target_name(target)),
        ))
    }
}

fn is_system_os(target: Target) -> bool {
    matches!(
        target,
        Target::Apt
            | Target::Apk
            | Target::Linuxmint
            | Target::Fedora
            | Target::Opensuse
            | Target::Kali
            | Target::Arch
            | Target::Archlinuxcn
            | Target::Manjaro
            | Target::Gentoo
            | Target::Rocky
            | Target::Alma
            | Target::Voidlinux
            | Target::Solus
            | Target::Ros
            | Target::Trisquel
            | Target::Linuxlite
            | Target::Raspi
            | Target::Armbian
            | Target::Openwrt
            | Target::Openeuler
            | Target::Openanolis
            | Target::Openkylin
            | Target::Deepin
    )
}

fn execute_all(
    action: Action,
    selector: Option<&str>,
    scope: Scope,
    dry_run: bool,
    atomic: bool,
    config: &Config,
) -> io::Result<()> {
    if atomic && action != Action::Set {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--atomic is supported only for set all",
        ));
    }
    let mut plan = Vec::new();
    for &target in ALL_TARGETS {
        if !config.enabled(catalog_name(target)) {
            continue;
        }
        if let Err(error) = validate_scope(target, scope) {
            eprintln!("{}: skipped; {error}", target_name(target));
            continue;
        }
        if atomic && !atomic_supported(target) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "{} cannot participate in --atomic; use a single target or omit --atomic",
                    target_name(target)
                ),
            ));
        }
        if atomic && !is_installed(target) {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!(
                    "{} is not installed; --atomic refuses a partial plan",
                    target_name(target)
                ),
            ));
        }
        let mirror = match action {
            Action::Set => match lm::catalog::resolve(catalog_name(target), selector, config) {
                Ok(mirror) => Some(mirror),
                Err(_error)
                    if selector.is_none()
                        && lm::catalog::builtin_mirrors(catalog_name(target))?.is_empty() =>
                {
                    eprintln!(
                        "{}: skipped; configure a default or pass a URL",
                        target_name(target)
                    );
                    continue;
                }
                Err(error) => return Err(error),
            },
            Action::Reset => None,
        };
        let previous = if atomic {
            let status = inspect(target, config, scope)?;
            if status.configured && status.source.is_none() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "{} has no structured previous source; --atomic cannot restore it",
                        target_name(target)
                    ),
                ));
            }
            status.source
        } else {
            None
        };
        plan.push((target, mirror, previous));
    }
    let mut applied: Vec<(Target, Option<String>)> = Vec::new();
    for (target, mirror, previous) in plan {
        if let Err(error) = execute_resolved(target, action, mirror.as_deref(), scope, dry_run) {
            if atomic && !dry_run {
                let mut rollback_error = None;
                for (applied_target, previous) in applied.into_iter().rev() {
                    let result = match previous {
                        Some(source) => {
                            run_action(applied_target, Action::Set, Some(&source), scope)
                        }
                        None => run_action(applied_target, Action::Reset, None, scope),
                    };
                    if let Err(rollback) = result {
                        rollback_error = Some(format!(
                            "rollback {} failed: {rollback}",
                            target_name(applied_target)
                        ));
                    }
                }
                return Err(io::Error::other(match rollback_error {
                    Some(rollback) => format!("{error}; {rollback}"),
                    None => format!("{error}; changes rolled back"),
                }));
            }
            return Err(error);
        }
        if atomic && !dry_run {
            applied.push((target, previous));
        }
    }
    Ok(())
}

fn atomic_supported(target: Target) -> bool {
    matches!(
        target,
        Target::Uv
            | Target::Cargo
            | Target::Rust
            | Target::Docker
            | Target::Containerd
            | Target::Nerdctl
            | Target::Podman
            | Target::Maven
            | Target::Java
            | Target::Gradle
            | Target::Sbt
            | Target::Dart
            | Target::Flutter
            | Target::Huggingface
            | Target::Apt
            | Target::Apk
            | Target::Brew
            | Target::Rustup
            | Target::Julia
            | Target::Cpan
            | Target::Cran
            | Target::R
            | Target::Rye
            | Target::Nvm
            | Target::Luarocks
            | Target::Clojure
            | Target::Haskell
            | Target::Hackage
            | Target::Cabal
            | Target::Stack
            | Target::Nix
            | Target::Guix
            | Target::Emacs
            | Target::Linuxmint
            | Target::Fedora
            | Target::Opensuse
            | Target::Kali
            | Target::Arch
            | Target::Archlinuxcn
            | Target::Manjaro
            | Target::Gentoo
            | Target::Rocky
            | Target::Alma
            | Target::Voidlinux
            | Target::Solus
            | Target::Ros
            | Target::Trisquel
            | Target::Linuxlite
            | Target::Raspi
            | Target::Armbian
            | Target::Openwrt
            | Target::Openeuler
            | Target::Openanolis
            | Target::Openkylin
            | Target::Deepin
    )
}

fn inspect(target: Target, config: &Config, scope: Scope) -> io::Result<lm::ToolStatus> {
    let name = catalog_name(target);
    let expected =
        if lm::catalog::builtin_mirrors(name)?.is_empty() && config.default_for(name).is_none() {
            String::new()
        } else {
            lm::catalog::resolve(name, None, config)?
        };
    match target {
        Target::Npm | Target::Pnpm | Target::Yarn | Target::Bun | Target::Node => {
            let name = if target == Target::Node {
                "npm"
            } else {
                target_name(target)
            };
            lm::node::status(name, &expected, scope)
        }
        Target::Go => lm::go::status(&expected),
        Target::Pip | Target::Pip3 | Target::Python => {
            let name = if target == Target::Pip3 {
                "pip3"
            } else {
                "pip"
            };
            lm::python::status(name, &expected)
        }
        Target::Uv => lm::uv::status(&expected, scope),
        Target::Pdm => lm::pdm::status(&expected),
        Target::Poetry => lm::poetry::status(&expected, scope),
        Target::Composer | Target::Php => lm::php::status(&expected),
        Target::Gem | Target::Ruby => lm::ruby::gem_status(&expected),
        Target::Bundle => lm::ruby::bundle_status(&expected),
        Target::Maven | Target::Java => lm::java::maven_status(&expected),
        Target::Gradle => lm::java::gradle_status(&expected),
        Target::Sbt => lm::sbt::status(&expected),
        Target::Cargo | Target::Rust => lm::rust::status(&expected, scope),
        Target::Docker => lm::docker::status(),
        Target::Containerd | Target::Nerdctl => {
            lm::container::containerd_status(target_name(target))
        }
        Target::Podman => lm::container::podman_status(),
        Target::Conda | Target::Mamba => lm::conda::status(target_name(target), &expected),
        Target::Dart => lm::dart::dart_status(&expected, scope),
        Target::Flutter => lm::dart::flutter_status(&expected, scope),
        Target::Nuget | Target::Dotnet => lm::nuget::status(scope),
        Target::Cran | Target::R => lm::r::status(&expected),
        Target::Huggingface => lm::huggingface::status(&expected, scope),
        Target::Apt => lm::platform::apt_status(scope),
        Target::Apk => lm::platform::apk_status(scope),
        Target::Brew => lm::platform::brew_status(scope),
        Target::Rustup => lm::platform::rustup_status(scope),
        Target::Hex => lm::platform::hex_status(scope),
        Target::Julia => lm::platform::julia_status(scope),
        Target::Cpan => lm::platform::cpan_status(scope),
        Target::Winget => lm::platform::winget_status(scope),
        Target::Opam => lm::platform::opam_status(scope),
        Target::Rye => lm::platform::env_status("rye", "rye", "RYE_PYPI_MIRROR", &expected, scope),
        Target::Nvm => {
            lm::platform::env_status("node", "nvm", "NVM_NODEJS_ORG_MIRROR", &expected, scope)
        }
        Target::Luarocks => lm::platform::luarocks_status(&expected, scope),
        Target::Clojure => lm::platform::clojure_status(&expected, scope),
        Target::Haskell | Target::Hackage | Target::Cabal => {
            lm::platform::cabal_status(&expected, scope)
        }
        Target::Stack => lm::platform::stack_status(&expected, scope),
        Target::Ocaml => lm::platform::opam_status(scope),
        Target::Cocoapods => lm::platform::cocoapods_status(&expected, scope),
        Target::Flathub => lm::platform::flatpak_status(&expected, scope),
        Target::Nix => lm::platform::env_status("nix", "nix", "NIX_CONFIG", &expected, scope),
        Target::Guix => {
            lm::platform::env_status("guix", "guix", "GUIX_SUBSTITUTE_URLS", &expected, scope)
        }
        Target::Emacs => lm::platform::emacs_status(&expected, scope),
        Target::Tex => lm::platform::tex_status(&expected, scope),
        target @ (Target::Linuxmint
        | Target::Fedora
        | Target::Opensuse
        | Target::Kali
        | Target::Arch
        | Target::Archlinuxcn
        | Target::Manjaro
        | Target::Gentoo
        | Target::Rocky
        | Target::Alma
        | Target::Voidlinux
        | Target::Solus
        | Target::Ros
        | Target::Trisquel
        | Target::Linuxlite
        | Target::Raspi
        | Target::Armbian
        | Target::Openwrt
        | Target::Openeuler
        | Target::Openanolis
        | Target::Openkylin
        | Target::Deepin
        | Target::Msys2
        | Target::Termux
        | Target::Freebsd
        | Target::Openbsd
        | Target::Netbsd) => lm::platform::os_status(target_name(target), scope),
        Target::All => unreachable!(),
    }
}

fn status_record(target: Target, config: &Config, scope: Scope) -> StatusRecord {
    match inspect(target, config, scope) {
        Ok(status) => StatusRecord {
            target: target_name(target).to_owned(),
            scope: format!("{scope:?}").to_lowercase(),
            configured: status.configured,
            version: Some(status.version),
            source: status.source.map(|source| redact_url(&source)),
            path: status.path.map(|path| path.display().to_string()),
            origin: format!("{scope:?}").to_lowercase(),
            detail: Some(redact_text(&status.detail)),
            error: None,
        },
        Err(error) => StatusRecord {
            target: target_name(target).to_owned(),
            scope: format!("{scope:?}").to_lowercase(),
            configured: false,
            version: None,
            source: None,
            path: None,
            origin: format!("{scope:?}").to_lowercase(),
            detail: None,
            error: Some(error.to_string()),
        },
    }
}

fn get(
    target: Target,
    config: &Config,
    scope: Scope,
    format: OutputFormat,
    only_installed: bool,
) -> io::Result<()> {
    let targets: &[Target] = if target == Target::All {
        ALL_TARGETS
    } else {
        std::slice::from_ref(&target)
    };
    let records: Vec<_> = targets
        .iter()
        .copied()
        .filter(|target| config.enabled(catalog_name(*target)))
        .filter(|target| validate_scope(*target, scope).is_ok())
        .filter(|target| !only_installed || is_installed(*target))
        .map(|target| status_record(target, config, scope))
        .collect();
    if format == OutputFormat::Json {
        print_json(&serde_json::Value::Array(
            records.iter().map(status_json).collect(),
        ))?;
    } else {
        for record in &records {
            if let Some(error) = &record.error {
                eprintln!("{}: unavailable; {error}", record.target);
            } else {
                let state = if record.configured {
                    "configured"
                } else {
                    "not configured"
                };
                println!(
                    "{}: {state}; {}; {}",
                    record.target,
                    record.version.as_deref().unwrap_or_default(),
                    record.detail.as_deref().unwrap_or_default()
                );
            }
        }
    }
    if !records.is_empty()
        && records
            .iter()
            .all(|record| record.configured && record.error.is_none())
    {
        Ok(())
    } else {
        Err(io::Error::other(
            "one or more targets are unavailable or unconfigured",
        ))
    }
}

fn plan(
    target: Target,
    selector: Option<&str>,
    reset: bool,
    scope: Scope,
    config: &Config,
    format: OutputFormat,
    only_installed: bool,
) -> io::Result<()> {
    let targets: &[Target] = if target == Target::All {
        ALL_TARGETS
    } else {
        std::slice::from_ref(&target)
    };
    let mut records = Vec::new();
    for &target in targets {
        if !config.enabled(catalog_name(target))
            || validate_scope(target, scope).is_err()
            || (only_installed && !is_installed(target))
        {
            continue;
        }
        let current = status_record(target, config, scope);
        let desired = if reset {
            Ok(None)
        } else {
            lm::catalog::resolve(catalog_name(target), selector, config).map(Some)
        };
        let (desired, error) = match desired {
            Ok(desired) => (desired, current.error.clone()),
            Err(error) => (None, Some(error.to_string())),
        };
        records.push(serde_json::json!({
            "target": target_name(target),
            "scope": scope_name(scope),
            "action": if reset { "reset" } else { "set" },
            "current": current.source,
            "desired": desired.as_deref().map(redact_url),
            "path": current.path,
            "strategy": if current.path.is_some() { "file" } else { "command" },
            "installed": is_installed(target),
            "error": error,
        }));
    }
    if format == OutputFormat::Json {
        print_json(&serde_json::Value::Array(records.clone()))?;
    } else {
        for record in &records {
            println!(
                "{}\t{}\t{}\t{}\t{}",
                record["target"].as_str().unwrap_or_default(),
                record["action"].as_str().unwrap_or_default(),
                record["desired"].as_str().unwrap_or("upstream"),
                record["path"].as_str().unwrap_or("external command"),
                record["error"].as_str().unwrap_or("ready")
            );
        }
    }
    if records.iter().any(|record| !record["error"].is_null()) {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "one or more source changes cannot be planned",
        ))
    } else {
        Ok(())
    }
}

fn doctor(
    target: Target,
    selector: Option<&str>,
    config: &Config,
    scope: Scope,
    options: ProbeOptions,
) -> io::Result<()> {
    validate_parallelism(options.parallelism)?;
    let targets: &[Target] = if target == Target::All {
        ALL_TARGETS
    } else {
        std::slice::from_ref(&target)
    };
    let ttl = if options.no_cache {
        0
    } else {
        options
            .cache_ttl
            .unwrap_or(config.settings().cache_ttl_seconds)
    };
    let mut cache = lm::probe::HealthCache::load(ttl)?;
    let mut records = Vec::new();
    for &target in targets {
        if !config.enabled(catalog_name(target))
            || (options.only_installed && !is_installed(target))
            || validate_scope(target, scope).is_err()
        {
            continue;
        }
        let status = status_record(target, config, scope);
        let desired = lm::catalog::resolve(catalog_name(target), selector, config);
        let health = desired.as_ref().ok().and_then(|url| {
            measure_one(target, Some(url), config, &mut cache, options.parallelism)
                .ok()
                .and_then(|items| items.into_iter().next())
        });
        let error = status
            .error
            .clone()
            .or_else(|| desired.as_ref().err().map(std::string::ToString::to_string));
        records.push(serde_json::json!({
            "target": target_name(target),
            "installed": is_installed(target),
            "configured": status.configured,
            "source": status.source,
            "path": status.path,
            "health": health.as_ref().map(|record| record.state.clone()),
            "code": health.as_ref().and_then(|record| record.code.clone()),
            "latency_ms": health.as_ref().and_then(|record| record.milliseconds),
            "error": error,
        }));
    }
    cache.save()?;
    if options.format == OutputFormat::Json {
        print_json(&serde_json::Value::Array(records.clone()))?;
    } else {
        for record in &records {
            println!(
                "{}\t{}\t{}\t{}",
                record["target"].as_str().unwrap_or_default(),
                if record["configured"].as_bool().unwrap_or(false) {
                    "configured"
                } else {
                    "not configured"
                },
                record["health"].as_str().unwrap_or("not checked"),
                record["error"].as_str().unwrap_or("ok")
            );
        }
    }
    if !records.is_empty()
        && records.iter().all(|record| {
            record["configured"].as_bool() == Some(true)
                && record["error"].is_null()
                && matches!(
                    record["health"].as_str(),
                    Some("healthy") | Some("auth-required")
                )
        })
    {
        Ok(())
    } else {
        Err(io::Error::other("doctor found one or more problems"))
    }
}

fn scope_name(scope: Scope) -> &'static str {
    match scope {
        Scope::Project => "project",
        Scope::User => "user",
        Scope::System => "system",
    }
}

fn validate_parallelism(parallelism: Option<usize>) -> io::Result<()> {
    if parallelism.is_some_and(|value| !(1..=64).contains(&value)) {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "parallelism must be between 1 and 64",
        ))
    } else {
        Ok(())
    }
}

fn list(query: Option<&str>, config: &Config, format: OutputFormat) -> io::Result<()> {
    if format == OutputFormat::Json {
        return list_json(query, config);
    }
    println!("config: {}", config.path.display());
    if matches!(query, Some("mirror")) {
        let mut mirrors = BTreeSet::new();
        for target in lm::catalog::targets() {
            for mirror in target.mirrors {
                mirrors.insert(mirror.name);
            }
        }
        for (name, _) in config.custom_mirrors() {
            mirrors.insert(name);
        }
        for name in mirrors {
            println!("{name}");
        }
        return Ok(());
    }
    if query.is_none() || matches!(query, Some("target" | "os" | "lang" | "ware")) {
        let category = query.unwrap_or("target");
        let mut names = BTreeSet::new();
        for target in lm::catalog::targets() {
            if category == "target" || target_category(target.name) == category {
                names.insert(target.name);
            }
        }
        for name in names {
            println!("{name}");
        }
        return Ok(());
    }
    let target = query.unwrap();
    let spec = lm::catalog::find(target).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("unsupported target: {target}"),
        )
    })?;
    println!("target: {}", spec.name);
    for mirror in spec.mirrors {
        println!("{}\t{}", mirror.name, redact_url(mirror.url));
    }
    for (name, url) in config.custom_mirrors() {
        println!("{}\t{} (config)", name, redact_url(url));
    }
    Ok(())
}

fn list_json(query: Option<&str>, config: &Config) -> io::Result<()> {
    let output = if matches!(query, Some("mirror")) {
        let mut mirrors = BTreeMap::new();
        for target in lm::catalog::targets() {
            for mirror in target.mirrors {
                mirrors.insert(mirror.name.to_owned(), (mirror.url.to_owned(), "builtin"));
            }
        }
        for (name, url) in config.custom_mirrors() {
            mirrors.insert(name.to_owned(), (url.to_owned(), "config"));
        }
        serde_json::json!({
            "config": config.path,
            "mirrors": mirrors.into_iter().map(|(name, (url, origin))| serde_json::json!({"name": name, "url": redact_url(&url), "origin": origin})).collect::<Vec<_>>()
        })
    } else if let Some(query) =
        query.filter(|query| !matches!(*query, "target" | "os" | "lang" | "ware"))
    {
        let spec = lm::catalog::find(query).ok_or_else(|| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unsupported target: {query}"),
            )
        })?;
        serde_json::json!({
            "config": config.path,
            "target": spec.name,
            "aliases": spec.aliases,
            "mirrors": spec.mirrors.iter().map(|mirror| serde_json::json!({"name": mirror.name, "url": redact_url(mirror.url), "origin": "builtin"})).collect::<Vec<_>>(),
            "custom_mirrors": config.custom_mirrors().map(|(name, url)| serde_json::json!({"name": name, "url": redact_url(url), "origin": "config"})).collect::<Vec<_>>()
        })
    } else {
        let category = query.unwrap_or("target");
        serde_json::json!({
            "config": config.path,
            "targets": lm::catalog::targets().iter().filter(|target| category == "target" || target_category(target.name) == category).map(|target| serde_json::json!({"name": target.name, "category": target_category(target.name), "aliases": target.aliases, "mirrors": target.mirrors.len(), "enabled": config.enabled(target.name)})).collect::<Vec<_>>()
        })
    };
    print_json(&output)
}

fn target_category(target: &str) -> &'static str {
    if matches!(
        target,
        "apt"
            | "apk"
            | "linuxmint"
            | "fedora"
            | "opensuse"
            | "kali"
            | "arch"
            | "archlinuxcn"
            | "manjaro"
            | "gentoo"
            | "rocky"
            | "alma"
            | "voidlinux"
            | "solus"
            | "ros"
            | "trisquel"
            | "linuxlite"
            | "raspi"
            | "armbian"
            | "openwrt"
            | "openeuler"
            | "openanolis"
            | "openkylin"
            | "deepin"
            | "msys2"
            | "termux"
            | "freebsd"
            | "openbsd"
            | "netbsd"
    ) {
        "os"
    } else if matches!(
        target,
        "brew"
            | "docker"
            | "containerd"
            | "podman"
            | "winget"
            | "opam"
            | "cocoapods"
            | "flathub"
            | "nix"
            | "guix"
            | "emacs"
            | "tex"
    ) {
        "ware"
    } else {
        "lang"
    }
}

fn is_installed(target: Target) -> bool {
    command_candidates(target)
        .iter()
        .any(|command| lm::command_exists(command))
}

fn command_candidates(target: Target) -> &'static [&'static str] {
    match target {
        Target::Npm | Target::Node => &["npm"],
        Target::Pnpm => &["pnpm"],
        Target::Yarn => &["yarn"],
        Target::Bun => &["bun"],
        Target::Go => &["go"],
        Target::Pip => &["pip"],
        Target::Pip3 => &["pip3"],
        Target::Python => &["pip", "uv", "pdm", "poetry"],
        Target::Uv => &["uv"],
        Target::Pdm => &["pdm"],
        Target::Poetry => &["poetry"],
        Target::Composer | Target::Php => &["composer"],
        Target::Gem | Target::Bundle | Target::Ruby => &["gem"],
        Target::Maven | Target::Java => &["mvn"],
        Target::Gradle => &["gradle"],
        Target::Sbt => &["sbt"],
        Target::Cargo | Target::Rust => &["cargo"],
        Target::Docker => &["docker"],
        Target::Containerd | Target::Nerdctl => &["containerd", "nerdctl"],
        Target::Podman => &["podman"],
        Target::Conda => &["conda"],
        Target::Mamba => &["mamba"],
        Target::Dart => &["dart"],
        Target::Flutter => &["flutter"],
        Target::Nuget | Target::Dotnet => &["dotnet"],
        Target::Cran | Target::R => &["R"],
        Target::Huggingface => &["hf", "huggingface-cli"],
        Target::Apt => &["apt"],
        Target::Apk => &["apk"],
        Target::Brew => &["brew"],
        Target::Rustup => &["rustup"],
        Target::Hex => &["mix"],
        Target::Julia => &["julia"],
        Target::Cpan => &["cpan"],
        Target::Winget => &["winget"],
        Target::Opam => &["opam"],
        Target::Rye => &["rye"],
        Target::Nvm => &["node"],
        Target::Luarocks => &["luarocks"],
        Target::Clojure => &["clojure"],
        Target::Haskell | Target::Hackage | Target::Cabal => &["cabal"],
        Target::Stack => &["stack"],
        Target::Ocaml => &["opam"],
        Target::Cocoapods => &["pod"],
        Target::Flathub => &["flatpak"],
        Target::Nix => &["nix"],
        Target::Guix => &["guix"],
        Target::Emacs => &["emacs"],
        Target::Tex => &["tlmgr"],
        Target::Linuxmint | Target::Kali | Target::Ros | Target::Trisquel | Target::Linuxlite => {
            &["apt"]
        }
        Target::Fedora | Target::Rocky | Target::Alma | Target::Openeuler | Target::Openanolis => {
            &["dnf", "yum"]
        }
        Target::Opensuse => &["zypper"],
        Target::Arch | Target::Archlinuxcn | Target::Manjaro | Target::Msys2 => &["pacman"],
        Target::Gentoo => &["emerge"],
        Target::Voidlinux => &["xbps-install"],
        Target::Solus => &["eopkg"],
        Target::Raspi | Target::Armbian | Target::Openkylin | Target::Deepin => &["apt"],
        Target::Openwrt => &["opkg"],
        Target::Termux | Target::Freebsd => &["pkg"],
        Target::Openbsd => &["pkg_add"],
        Target::Netbsd => &["pkgin"],
        Target::All => &[],
    }
}

fn measure(
    target: Target,
    selector: Option<&str>,
    config: &Config,
    options: ProbeOptions,
) -> io::Result<()> {
    validate_parallelism(options.parallelism)?;
    let targets: &[Target] = if target == Target::All {
        ALL_TARGETS
    } else {
        std::slice::from_ref(&target)
    };
    let ttl = if options.no_cache {
        0
    } else {
        options
            .cache_ttl
            .unwrap_or(config.settings().cache_ttl_seconds)
    };
    let mut cache = lm::probe::HealthCache::load(ttl)?;
    let mut records = Vec::new();
    for &target in targets {
        if !config.enabled(catalog_name(target)) {
            continue;
        }
        if options.only_installed && !is_installed(target) {
            continue;
        }
        records.extend(measure_one(
            target,
            selector,
            config,
            &mut cache,
            options.parallelism,
        )?);
    }
    cache.save()?;
    if options.format == OutputFormat::Json {
        print_json(&serde_json::Value::Array(
            records.iter().map(measure_json).collect(),
        ))?;
    } else {
        for record in &records {
            if let Some(error) = &record.error {
                println!(
                    "{}\t{}\t{}\tfailed\t{}",
                    record.target,
                    record.mirror,
                    redact_url(&record.url),
                    error
                );
            } else {
                println!(
                    "{}\t{}\t{}\t{}\t{}\t{}ms{}",
                    record.target,
                    record.mirror,
                    redact_url(&record.url),
                    record.code.as_deref().unwrap_or_default(),
                    record.state,
                    record.milliseconds.unwrap_or_default(),
                    if record.cached { "\tcached" } else { "" }
                );
            }
        }
    }
    if !records.is_empty() && records.iter().all(probe_is_usable) {
        Ok(())
    } else {
        Err(io::Error::other("one or more mirrors are unavailable"))
    }
}

fn probe_is_usable(record: &MeasureRecord) -> bool {
    record.state == "healthy"
        || (record.state == "auth-required"
            && matches!(record.target.as_str(), "docker" | "containerd" | "podman"))
}

fn measure_one(
    target: Target,
    selector: Option<&str>,
    config: &Config,
    cache: &mut lm::probe::HealthCache,
    parallelism: Option<usize>,
) -> io::Result<Vec<MeasureRecord>> {
    let name = catalog_name(target);
    let specs = lm::catalog::builtin_mirrors(name)?;
    let candidates = if let Some(selector) = selector {
        vec![(
            selector.to_owned(),
            lm::catalog::resolve(name, Some(selector), config)?,
        )]
    } else if specs.is_empty() {
        match lm::catalog::resolve(name, None, config) {
            Ok(url) => vec![("configured".to_owned(), url)],
            Err(error) => {
                return Ok(vec![MeasureRecord {
                    target: name.to_owned(),
                    mirror: "configured".to_owned(),
                    url: String::new(),
                    probe_url: None,
                    code: None,
                    state: "unavailable".to_owned(),
                    detail: None,
                    milliseconds: None,
                    cached: false,
                    error: Some(error.to_string()),
                }])
            }
        }
    } else {
        specs
            .iter()
            .map(|mirror| (mirror.name.to_owned(), mirror.url.to_owned()))
            .collect()
    };
    let mut settings = config.settings();
    if let Some(parallelism) = parallelism {
        settings.parallelism = parallelism;
    }
    let mut records: Vec<Option<MeasureRecord>> = (0..candidates.len()).map(|_| None).collect();
    let mut pending = Vec::new();
    for (index, (mirror, url)) in candidates.iter().enumerate() {
        if let Some(result) = cache.get(name, url) {
            records[index] = Some(MeasureRecord {
                target: name.to_owned(),
                mirror: mirror.clone(),
                url: url.clone(),
                probe_url: Some(result.probe_url.clone()),
                code: Some(result.code.clone()),
                state: result.state.clone(),
                detail: Some(result.detail.clone()),
                milliseconds: Some(result.milliseconds),
                cached: true,
                error: None,
            });
        } else {
            pending.push(index);
        }
    }
    for chunk in pending.chunks(settings.parallelism.max(1)) {
        let results = thread::scope(|scope| {
            chunk
                .iter()
                .map(|&index| {
                    let url = candidates[index].1.clone();
                    scope.spawn(move || {
                        lm::probe::probe_target(
                            name,
                            &url,
                            settings.timeout_seconds,
                            settings.retries,
                        )
                    })
                })
                .collect::<Vec<_>>()
                .into_iter()
                .map(|handle| {
                    handle
                        .join()
                        .unwrap_or_else(|_| Err(io::Error::other("probe thread panicked")))
                })
                .collect::<Vec<_>>()
        });
        for (&index, result) in chunk.iter().zip(results) {
            let (mirror, url) = &candidates[index];
            match result {
                Ok(result) => {
                    cache.put(name, url, result.clone());
                    records[index] = Some(MeasureRecord {
                        target: name.to_owned(),
                        mirror: mirror.clone(),
                        url: url.clone(),
                        probe_url: Some(result.probe_url),
                        code: Some(result.code),
                        state: result.state,
                        detail: Some(result.detail),
                        milliseconds: Some(result.milliseconds),
                        cached: false,
                        error: None,
                    });
                }
                Err(error) => {
                    records[index] = Some(MeasureRecord {
                        target: name.to_owned(),
                        mirror: mirror.clone(),
                        url: url.clone(),
                        probe_url: None,
                        code: None,
                        state: "error".to_owned(),
                        detail: None,
                        milliseconds: None,
                        cached: false,
                        error: Some(error.to_string()),
                    });
                }
            }
        }
    }
    Ok(records.into_iter().flatten().collect())
}

fn redact_url(value: &str) -> String {
    let Some(scheme) = value.find("://") else {
        return value.to_owned();
    };
    let authority_start = scheme + 3;
    let authority_end = value[authority_start..]
        .find(['/', '?', '#'])
        .map_or(value.len(), |offset| authority_start + offset);
    let authority = &value[authority_start..authority_end];
    let authority = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    let suffix = &value[authority_end..];
    let suffix = suffix
        .find(['?', '#'])
        .map_or(suffix, |offset| &suffix[..offset]);
    format!("{}://{}{}", &value[..scheme], authority, suffix)
}

fn redact_text(value: &str) -> String {
    value
        .split_whitespace()
        .map(redact_url)
        .collect::<Vec<_>>()
        .join(" ")
}

fn status_json(record: &StatusRecord) -> serde_json::Value {
    serde_json::json!({
        "target": record.target.clone(),
        "scope": record.scope.clone(),
        "configured": record.configured,
        "version": record.version.clone(),
        "source": record.source.clone(),
        "path": record.path.clone(),
        "origin": record.origin.clone(),
        "detail": record.detail.clone(),
        "error": record.error.clone(),
    })
}

fn measure_json(record: &MeasureRecord) -> serde_json::Value {
    serde_json::json!({
        "target": record.target.clone(),
        "mirror": record.mirror.clone(),
        "url": redact_url(&record.url),
        "probe_url": record.probe_url.as_deref().map(redact_url),
        "code": record.code.clone(),
        "state": record.state.clone(),
        "detail": record.detail.clone(),
        "milliseconds": record.milliseconds,
        "cached": record.cached,
        "error": record.error.clone(),
    })
}

fn print_json(value: &serde_json::Value) -> io::Result<()> {
    let output = serde_json::to_string_pretty(value)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    println!("{output}");
    Ok(())
}

fn config_command(config: &Config, command: ConfigCommand) -> io::Result<()> {
    match command {
        ConfigCommand::Validate => {
            println!("valid\t{}", config.path.display());
            Ok(())
        }
        ConfigCommand::Show { format } => {
            let value = config.effective_json();
            if format == OutputFormat::Json {
                print_json(&value)
            } else {
                println!("config: {}", config.path.display());
                println!(
                    "mirrors: {}",
                    value["mirrors"].as_object().map_or(0, |items| items.len())
                );
                println!(
                    "defaults: {}",
                    value["defaults"].as_object().map_or(0, |items| items.len())
                );
                println!(
                    "targets: {}",
                    value["targets"].as_object().map_or(0, |items| items.len())
                );
                let options = &value["options"];
                println!(
                    "options: timeout={} retries={} cache-ttl={} parallelism={}",
                    options["timeout_seconds"].as_u64().unwrap_or_default(),
                    options["retries"].as_u64().unwrap_or_default(),
                    options["cache_ttl_seconds"].as_u64().unwrap_or_default(),
                    options["parallelism"].as_u64().unwrap_or_default()
                );
                Ok(())
            }
        }
    }
}

fn run() -> io::Result<()> {
    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref())?;
    match cli.command {
        Commands::List { query, format } => list(query.as_deref(), &config, format),
        Commands::Measure {
            target,
            mirror,
            format,
            cache_ttl,
            no_cache,
            only_installed,
            parallelism,
        }
        | Commands::Check {
            target,
            mirror,
            format,
            cache_ttl,
            no_cache,
            only_installed,
            parallelism,
        } => measure(
            target,
            mirror.as_deref(),
            &config,
            ProbeOptions {
                format,
                cache_ttl,
                no_cache,
                only_installed,
                parallelism,
            },
        ),
        Commands::Get {
            target,
            scope,
            format,
            only_installed,
        } => get(target, &config, scope, format, only_installed),
        Commands::Set {
            target,
            mirror,
            scope,
            dry_run,
            atomic,
        } => {
            if target == Target::All {
                execute_all(
                    Action::Set,
                    mirror.as_deref(),
                    scope,
                    dry_run,
                    atomic,
                    &config,
                )
            } else {
                if atomic {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "--atomic is supported only for set all",
                    ));
                }
                execute(
                    target,
                    Action::Set,
                    mirror.as_deref(),
                    scope,
                    dry_run,
                    &config,
                )
            }
        }
        Commands::Reset {
            target,
            scope,
            dry_run,
        } => {
            if target == Target::All {
                execute_all(Action::Reset, None, scope, dry_run, false, &config)
            } else {
                execute(target, Action::Reset, None, scope, dry_run, &config)
            }
        }
        Commands::Config { command } => config_command(&config, command),
        Commands::Doctor {
            target,
            mirror,
            scope,
            format,
            cache_ttl,
            no_cache,
            only_installed,
            parallelism,
        } => doctor(
            target,
            mirror.as_deref(),
            &config,
            scope,
            ProbeOptions {
                format,
                cache_ttl,
                no_cache,
                only_installed,
                parallelism,
            },
        ),
        Commands::Plan {
            target,
            mirror,
            reset,
            scope,
            format,
            only_installed,
        } => plan(
            target,
            mirror.as_deref(),
            reset,
            scope,
            &config,
            format,
            only_installed,
        ),
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        process::exit(match error.kind() {
            io::ErrorKind::InvalidInput | io::ErrorKind::InvalidData => 2,
            io::ErrorKind::PermissionDenied => 77,
            io::ErrorKind::NotFound => 127,
            _ => 1,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_supports_checks_json_and_new_platforms() {
        assert!(Cli::try_parse_from(["lm", "check", "docker", "--format", "json"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "list", "apt", "--format", "json"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "set", "rustup", "rsproxy"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "set", "docker", "daocloud", "--dry"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "get", "huggingface", "--scope", "project"]).is_ok());
    }

    #[test]
    fn urls_are_redacted_before_output() {
        assert_eq!(
            redact_url("https://user:secret@example.com/a?token=x"),
            "https://example.com/a"
        );
    }

    #[test]
    fn docker_auth_challenge_is_a_reachable_registry() {
        let record = MeasureRecord {
            target: "docker".to_owned(),
            mirror: "daocloud".to_owned(),
            url: "https://docker.example".to_owned(),
            probe_url: None,
            code: Some("401".to_owned()),
            state: "auth-required".to_owned(),
            detail: None,
            milliseconds: None,
            cached: false,
            error: None,
        };
        assert!(probe_is_usable(&record));
    }
}
