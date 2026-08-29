use std::collections::{BTreeMap, BTreeSet};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;
use std::thread;

use clap::{Args, Parser, Subcommand, ValueEnum};

use lm::config::{Config, Scope};

const STARTER_CONFIG: &str = r#"# lazy-mirror configuration

[mirrors]
# company = "https://packages.example.com/simple"

[defaults]
# pip = "company"
# docker = "https://registry.example.com"

[options]
timeout_seconds = 10
retries = 1
cache_ttl_seconds = 300
parallelism = 4
"#;

#[derive(Debug, Parser)]
#[command(name = "lm", version, about = "Change package and software sources")]
struct Cli {
    #[arg(long, global = true, value_name = "FILE")]
    config: Option<PathBuf>,
    #[arg(long, global = true, conflicts_with = "config")]
    no_config: bool,
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
        #[command(flatten)]
        ip: IpOptions,
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
        #[command(flatten)]
        ip: IpOptions,
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
        #[arg(long, conflicts_with = "scope")]
        all_scopes: bool,
        #[arg(long)]
        explain: bool,
    },
    #[command(about = "Set a source, mirror name, or URL", visible_alias = "s")]
    Set {
        target: Target,
        mirror: Option<String>,
        #[arg(long, conflicts_with = "mirror")]
        best: bool,
        #[arg(long, value_enum, default_value = "user")]
        scope: Scope,
        #[arg(long, visible_alias = "dry")]
        dry_run: bool,
        #[arg(long)]
        verify: bool,
        #[arg(long)]
        atomic: bool,
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
    #[command(about = "Reset to the upstream source", visible_alias = "r")]
    Reset {
        target: Target,
        #[arg(long, value_enum, default_value = "user")]
        scope: Scope,
        #[arg(long, visible_alias = "dry")]
        dry_run: bool,
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
    #[command(about = "Validate or show the effective TOML configuration")]
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },
    #[command(about = "Validate the built-in target and mirror catalog")]
    Catalog {
        #[command(subcommand)]
        command: CatalogCommand,
    },
    #[command(about = "Generate shell completion script")]
    Completions {
        #[arg(value_enum)]
        shell: CompletionShell,
    },
    #[command(about = "Print mirror environment assignments for the current shell")]
    Env {
        target: Target,
        mirror: Option<String>,
        #[arg(long, value_enum, default_value = "sh")]
        shell: EnvShell,
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
        #[arg(long)]
        explain: bool,
        #[command(flatten)]
        ip: IpOptions,
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
    #[command(about = "Create a starter TOML configuration")]
    Init,
    #[command(about = "Validate the TOML configuration")]
    Validate,
    #[command(about = "Show effective configuration")]
    Show {
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
    #[command(about = "Show configuration files and precedence")]
    Sources {
        #[arg(long, value_enum, default_value = "table")]
        format: OutputFormat,
    },
}

#[derive(Debug, Subcommand)]
enum CatalogCommand {
    #[command(about = "Validate target names, aliases and mirror URLs")]
    Lint {
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

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "lower")]
enum CompletionShell {
    Bash,
    Zsh,
    Fish,
    Powershell,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
#[value(rename_all = "lower")]
enum EnvShell {
    Sh,
    Fish,
    Powershell,
}

#[derive(Debug, Clone, Copy, Args)]
struct IpOptions {
    #[arg(long, conflicts_with = "ipv6")]
    ipv4: bool,
    #[arg(long, conflicts_with = "ipv4")]
    ipv6: bool,
}

impl IpOptions {
    fn version(self) -> lm::probe::IpVersion {
        if self.ipv4 {
            lm::probe::IpVersion::V4
        } else if self.ipv6 {
            lm::probe::IpVersion::V6
        } else {
            lm::probe::IpVersion::Any
        }
    }
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
    #[value(alias = "docker-buildkit", alias = "buildx")]
    Buildkit,
    Containerd,
    Nerdctl,
    Podman,
    Helm,
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
    Target::Helm,
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
    metrics: Option<lm::probe::ProbeMetrics>,
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
    ip_version: lm::probe::IpVersion,
}

#[derive(Clone, Copy)]
struct ExecuteOptions {
    best: bool,
    verify: bool,
    scope: Scope,
    dry_run: bool,
    atomic: bool,
    format: OutputFormat,
}

#[derive(Debug)]
struct ChangeRecord {
    target: String,
    action: &'static str,
    scope: &'static str,
    before: Option<String>,
    desired: Option<String>,
    after: Option<String>,
    path: Option<String>,
    changed: bool,
    dry_run: bool,
    verified: bool,
}

#[derive(Clone, Copy)]
struct TargetCapabilities {
    project: bool,
    user: bool,
    system: bool,
    atomic: bool,
    commands: &'static [&'static str],
}

impl TargetCapabilities {
    const fn new(
        project: bool,
        user: bool,
        system: bool,
        atomic: bool,
        commands: &'static [&'static str],
    ) -> Self {
        Self {
            project,
            user,
            system,
            atomic,
            commands,
        }
    }

    fn supports(self, scope: Scope) -> bool {
        match scope {
            Scope::Project => self.project,
            Scope::User => self.user,
            Scope::System => self.system,
        }
    }
}

fn target_capabilities(target: Target) -> TargetCapabilities {
    match target {
        Target::Npm => TargetCapabilities::new(true, true, false, false, &["npm"]),
        Target::Pnpm => TargetCapabilities::new(true, true, false, false, &["pnpm"]),
        Target::Yarn => TargetCapabilities::new(true, true, false, false, &["yarn"]),
        Target::Bun => TargetCapabilities::new(true, true, false, false, &["bun"]),
        Target::Node => {
            TargetCapabilities::new(true, true, false, false, &["npm", "pnpm", "yarn", "bun"])
        }
        Target::Go => TargetCapabilities::new(false, true, false, false, &["go"]),
        Target::Pip => TargetCapabilities::new(false, true, false, false, &["pip"]),
        Target::Pip3 => TargetCapabilities::new(false, true, false, false, &["pip3"]),
        Target::Python => {
            TargetCapabilities::new(false, true, false, false, &["pip", "uv", "pdm", "poetry"])
        }
        Target::Uv => TargetCapabilities::new(true, true, false, true, &["uv"]),
        Target::Pdm => TargetCapabilities::new(false, true, false, false, &["pdm"]),
        Target::Poetry => TargetCapabilities::new(true, true, false, false, &["poetry"]),
        Target::Composer | Target::Php => {
            TargetCapabilities::new(false, true, false, false, &["composer"])
        }
        Target::Gem | Target::Bundle | Target::Ruby => {
            TargetCapabilities::new(false, true, false, false, &["gem"])
        }
        Target::Maven => TargetCapabilities::new(false, true, false, true, &["mvn"]),
        Target::Gradle => TargetCapabilities::new(false, true, false, true, &["gradle"]),
        Target::Sbt => TargetCapabilities::new(false, true, false, true, &["sbt"]),
        Target::Java => {
            TargetCapabilities::new(false, true, false, true, &["mvn", "gradle", "sbt"])
        }
        Target::Cargo => TargetCapabilities::new(true, true, true, true, &["cargo"]),
        Target::Rust => TargetCapabilities::new(true, true, true, true, &["cargo", "rustup"]),
        Target::Docker => TargetCapabilities::new(false, true, true, true, &["docker"]),
        Target::Buildkit => {
            TargetCapabilities::new(false, true, true, true, &["docker", "buildctl"])
        }
        Target::Containerd | Target::Nerdctl => {
            TargetCapabilities::new(false, true, true, true, &["containerd", "nerdctl"])
        }
        Target::Podman => TargetCapabilities::new(false, true, true, true, &["podman"]),
        Target::Helm => TargetCapabilities::new(false, true, false, false, &["helm"]),
        Target::Conda => TargetCapabilities::new(false, true, false, false, &["conda"]),
        Target::Mamba => TargetCapabilities::new(false, true, false, false, &["mamba"]),
        Target::Dart => TargetCapabilities::new(true, true, true, true, &["dart", "flutter"]),
        Target::Flutter => TargetCapabilities::new(true, true, true, true, &["flutter"]),
        Target::Nuget | Target::Dotnet => {
            TargetCapabilities::new(true, true, false, false, &["dotnet"])
        }
        Target::Cran | Target::R => TargetCapabilities::new(false, true, false, true, &["R"]),
        Target::Huggingface => {
            TargetCapabilities::new(true, true, true, true, &["hf", "huggingface-cli"])
        }
        Target::Apt => TargetCapabilities::new(false, false, true, true, &["apt"]),
        Target::Apk => TargetCapabilities::new(false, false, true, true, &["apk"]),
        Target::Brew => TargetCapabilities::new(true, true, true, true, &["brew"]),
        Target::Rustup => TargetCapabilities::new(true, true, true, true, &["rustup"]),
        Target::Hex => TargetCapabilities::new(false, true, false, false, &["mix"]),
        Target::Julia => TargetCapabilities::new(true, true, true, true, &["julia"]),
        Target::Cpan => TargetCapabilities::new(true, true, true, true, &["cpan"]),
        Target::Winget => TargetCapabilities::new(false, true, false, false, &["winget"]),
        Target::Opam => TargetCapabilities::new(false, true, false, false, &["opam"]),
        Target::Rye => TargetCapabilities::new(true, true, false, true, &["rye"]),
        Target::Nvm => TargetCapabilities::new(true, true, false, true, &["node"]),
        Target::Luarocks => TargetCapabilities::new(true, true, true, true, &["luarocks"]),
        Target::Clojure => TargetCapabilities::new(true, true, true, true, &["clojure"]),
        Target::Haskell => TargetCapabilities::new(true, true, true, true, &["cabal", "stack"]),
        Target::Hackage | Target::Cabal => {
            TargetCapabilities::new(true, true, true, true, &["cabal"])
        }
        Target::Stack => TargetCapabilities::new(true, true, true, true, &["stack"]),
        Target::Ocaml => TargetCapabilities::new(false, true, false, false, &["opam"]),
        Target::Cocoapods => TargetCapabilities::new(false, true, false, false, &["pod"]),
        Target::Flathub => TargetCapabilities::new(false, true, false, false, &["flatpak"]),
        Target::Nix => TargetCapabilities::new(false, true, true, true, &["nix"]),
        Target::Guix => TargetCapabilities::new(false, true, true, true, &["guix"]),
        Target::Emacs => TargetCapabilities::new(false, true, false, true, &["emacs"]),
        Target::Tex => TargetCapabilities::new(false, true, false, false, &["tlmgr"]),
        Target::Linuxmint
        | Target::Kali
        | Target::Trisquel
        | Target::Linuxlite
        | Target::Raspi
        | Target::Armbian
        | Target::Deepin => TargetCapabilities::new(false, false, true, true, &["apt"]),
        Target::Fedora | Target::Rocky | Target::Alma | Target::Openeuler | Target::Openanolis => {
            TargetCapabilities::new(false, false, true, true, &["dnf", "yum"])
        }
        Target::Opensuse => TargetCapabilities::new(false, false, true, true, &["zypper"]),
        Target::Arch | Target::Archlinuxcn | Target::Manjaro | Target::Msys2 => {
            TargetCapabilities::new(
                false,
                target == Target::Msys2,
                target != Target::Msys2,
                false,
                &["pacman"],
            )
        }
        Target::Gentoo => TargetCapabilities::new(false, false, true, true, &["emerge"]),
        Target::Voidlinux => TargetCapabilities::new(false, false, true, true, &["xbps-install"]),
        Target::Solus => TargetCapabilities::new(false, false, true, true, &["eopkg"]),
        Target::Ros => TargetCapabilities::new(false, false, true, true, &["apt"]),
        Target::Openwrt => TargetCapabilities::new(false, false, true, true, &["opkg"]),
        Target::Openkylin => TargetCapabilities::new(false, false, true, true, &["apt"]),
        Target::Termux => TargetCapabilities::new(false, true, false, false, &["pkg"]),
        Target::Freebsd => TargetCapabilities::new(false, true, false, false, &["pkg"]),
        Target::Openbsd => TargetCapabilities::new(false, true, false, false, &["pkg_add"]),
        Target::Netbsd => TargetCapabilities::new(false, true, false, false, &["pkgin"]),
        Target::All => TargetCapabilities::new(false, false, false, false, &[]),
    }
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
        Target::Buildkit => "buildkit",
        Target::Containerd => "containerd",
        Target::Nerdctl => "nerdctl",
        Target::Podman => "podman",
        Target::Helm => "helm",
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
            &[Target::Npm, Target::Pnpm, Target::Yarn, Target::Bun],
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
            Action::Set => lm::docker::set(mirror.unwrap(), scope),
            Action::Reset => lm::docker::unset(scope),
        },
        Target::Buildkit => match action {
            Action::Set => lm::docker::buildkit_set(mirror.unwrap(), scope),
            Action::Reset => lm::docker::buildkit_unset(scope),
        },
        Target::Containerd | Target::Nerdctl => match action {
            Action::Set => lm::container::containerd_set(mirror.unwrap(), scope),
            Action::Reset => lm::container::containerd_unset(scope),
        },
        Target::Podman => match action {
            Action::Set => lm::container::podman_set(mirror.unwrap(), scope),
            Action::Reset => lm::container::podman_unset(scope),
        },
        Target::Helm => match action {
            Action::Set => lm::helm::set(mirror.unwrap(), scope),
            Action::Reset => lm::helm::unset(scope),
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

fn select_mirror(
    target: Target,
    selector: Option<&str>,
    best: bool,
    config: &Config,
    cache: Option<&mut lm::probe::HealthCache>,
) -> io::Result<String> {
    if !best {
        return lm::catalog::resolve(catalog_name(target), selector, config);
    }
    let cache = cache.ok_or_else(|| io::Error::other("best mirror selection requires a cache"))?;
    let candidates = measure_one(
        target,
        None,
        config,
        cache,
        Some(config.settings().parallelism),
        lm::probe::IpVersion::Any,
    )?
    .into_iter()
    .collect::<Vec<_>>();
    fastest_mirror(candidates).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            format!("no usable mirror found for {}", target_name(target)),
        )
    })
}

fn fastest_mirror(mut candidates: Vec<MeasureRecord>) -> Option<String> {
    candidates.retain(probe_is_usable);
    candidates.sort_by_key(|record| record.milliseconds.unwrap_or(u128::MAX));
    candidates.into_iter().next().map(|record| record.url)
}

fn verify_mirror(target: Target, mirror: &str, config: &Config) -> io::Result<()> {
    let result = lm::probe::probe_target(
        catalog_name(target),
        mirror,
        config.settings().timeout_seconds,
        config.settings().retries,
        lm::probe::IpVersion::Any,
    )?;
    if state_is_usable(catalog_name(target), &result.state) {
        Ok(())
    } else {
        Err(io::Error::other(format!(
            "mirror verification failed for {}: {} ({})",
            target_name(target),
            result.state,
            result.detail
        )))
    }
}

type Snapshot = Vec<(Target, Option<String>)>;

#[derive(Clone, Copy)]
enum RollbackMode {
    Exact,
    Attempted,
}

impl RollbackMode {
    fn label(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Attempted => "attempted",
        }
    }
}

fn verification_targets(target: Target) -> Vec<Target> {
    match target {
        Target::Node => vec![Target::Npm, Target::Pnpm, Target::Yarn, Target::Bun],
        Target::Python => vec![Target::Pip, Target::Uv, Target::Pdm, Target::Poetry],
        Target::Java => vec![Target::Maven, Target::Gradle, Target::Sbt],
        Target::Rust => vec![Target::Cargo, Target::Rustup],
        Target::Dart => vec![Target::Dart, Target::Flutter],
        Target::Haskell => vec![Target::Cabal, Target::Stack],
        target => vec![target],
    }
}

fn installed_targets(target: Target, scope: Scope) -> Vec<Target> {
    verification_targets(target)
        .into_iter()
        .filter(|target| validate_scope(*target, scope).is_ok() && is_installed(*target))
        .collect()
}

fn snapshot_targets(target: Target, config: &Config, scope: Scope) -> io::Result<Snapshot> {
    let mut snapshot = Vec::new();
    for target in installed_targets(target, scope) {
        let status = inspect(target, config, scope)?;
        if status.configured && status.source.is_none() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "{} has no structured previous source; --verify cannot restore it",
                    target_name(target)
                ),
            ));
        }
        snapshot.push((target, status.source));
    }
    if snapshot.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "{} has no installed target that can be verified",
                target_name(target)
            ),
        ));
    }
    Ok(snapshot)
}

fn current_sources(target: Target, config: &Config, scope: Scope) -> io::Result<Snapshot> {
    installed_targets(target, scope)
        .into_iter()
        .map(|target| Ok((target, inspect(target, config, scope)?.source)))
        .collect()
}

fn change_records(
    target: Target,
    action: Action,
    mirror: Option<&str>,
    before: &Snapshot,
    options: ExecuteOptions,
    config: &Config,
) -> io::Result<Vec<ChangeRecord>> {
    let mut records = installed_targets(target, options.scope)
        .into_iter()
        .map(|target| {
            let before = before
                .iter()
                .find(|(current, _)| *current == target)
                .and_then(|(_, source)| source.clone());
            let (after, path) = if options.dry_run {
                (before.clone(), None)
            } else {
                inspect(target, config, options.scope)
                    .map(|status| (status.source, status.path))
                    .unwrap_or((None, None))
            };
            Ok(ChangeRecord {
                target: target_name(target).to_owned(),
                action: match action {
                    Action::Set => "set",
                    Action::Reset => "reset",
                },
                scope: scope_name(options.scope),
                changed: match action {
                    Action::Set => !same_source(before.as_deref(), mirror),
                    Action::Reset => before.is_some(),
                },
                before,
                desired: mirror.map(str::to_owned),
                after,
                path: path.map(|path| path.display().to_string()),
                dry_run: options.dry_run,
                verified: options.verify && action == Action::Set,
            })
        })
        .collect::<io::Result<Vec<_>>>()?;
    if records.is_empty() && options.dry_run {
        records.push(ChangeRecord {
            target: target_name(target).to_owned(),
            action: match action {
                Action::Set => "set",
                Action::Reset => "reset",
            },
            scope: scope_name(options.scope),
            before: None,
            desired: mirror.map(str::to_owned),
            after: None,
            path: None,
            changed: true,
            dry_run: true,
            verified: false,
        });
    }
    Ok(records)
}

fn print_change_records(records: &[ChangeRecord]) -> io::Result<()> {
    let output = records
        .iter()
        .map(|record| {
            serde_json::json!({
                "schema": lm::JSON_SCHEMA,
                "target": record.target,
                "action": record.action,
                "scope": record.scope,
                "before": record.before.as_deref().map(redact_url),
                "desired": record.desired.as_deref().map(redact_url),
                "after": record.after.as_deref().map(redact_url),
                "path": record.path,
                "changed": record.changed,
                "dry_run": record.dry_run,
                "verified": record.verified,
            })
        })
        .collect::<Vec<_>>();
    print_json(&serde_json::Value::Array(output))
}

fn same_source(left: Option<&str>, right: Option<&str>) -> bool {
    left.zip(right)
        .is_some_and(|(left, right)| left.trim_end_matches('/') == right.trim_end_matches('/'))
        || left.is_none() && right.is_none()
}

fn print_table_change(
    action: &str,
    target: &str,
    mirror: Option<&str>,
    scope: &str,
    dry_run: bool,
    verified: bool,
) {
    let suffix = if dry_run {
        ""
    } else if verified {
        "; verified"
    } else {
        ""
    };
    match action {
        "set" => println!(
            "{} {} mirror to {}{} (scope={scope})",
            if dry_run { "would set" } else { "set" },
            target,
            redact_url(mirror.unwrap_or_default()),
            suffix
        ),
        _ => println!(
            "{} {} mirror (scope={scope})",
            if dry_run { "would reset" } else { "reset" },
            target
        ),
    }
}

fn restore_snapshots(snapshots: &[&Snapshot], scope: Scope) -> io::Result<RollbackMode> {
    let mode = rollback_mode(snapshots);
    let mut errors = Vec::new();
    for snapshot in snapshots {
        for (target, source) in snapshot.iter().rev() {
            let result = match source {
                Some(source) => run_action(*target, Action::Set, Some(source), scope),
                None => run_action(*target, Action::Reset, None, scope),
            };
            if let Err(error) = result {
                errors.push(format!("rollback {} failed: {error}", target_name(*target)));
            }
        }
    }
    if errors.is_empty() {
        Ok(mode)
    } else {
        Err(io::Error::other(errors.join("; ")))
    }
}

fn rollback_mode(snapshots: &[&Snapshot]) -> RollbackMode {
    if !snapshots.is_empty()
        && snapshots
            .iter()
            .flat_map(|snapshot| snapshot.iter().map(|(target, _)| *target))
            .all(atomic_supported)
    {
        RollbackMode::Exact
    } else {
        RollbackMode::Attempted
    }
}

fn source_matches(target: Target, status: &lm::ToolStatus, expected: &str) -> bool {
    if !status.configured {
        return false;
    }
    let Some(source) = status.source.as_deref() else {
        return target == Target::Cocoapods;
    };
    match target {
        Target::Apt => apt_source_matches(source, expected),
        Target::Brew => url_matches(
            source,
            &format!("{}/homebrew-bottles", expected.trim_end_matches('/')),
        ),
        Target::Arch
        | Target::Archlinuxcn
        | Target::Manjaro
        | Target::Msys2
        | Target::Voidlinux => prefixed_source_matches(source, expected),
        Target::Docker
        | Target::Buildkit
        | Target::Containerd
        | Target::Nerdctl
        | Target::Podman
        | Target::Helm
        | Target::Apk
        | Target::Rustup
        | Target::Hex
        | Target::Julia
        | Target::Cpan
        | Target::Winget
        | Target::Opam
        | Target::Ocaml
        | Target::Nuget
        | Target::Dotnet => url_matches(source, expected),
        Target::Linuxmint
        | Target::Fedora
        | Target::Opensuse
        | Target::Kali
        | Target::Gentoo
        | Target::Rocky
        | Target::Alma
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
        | Target::Termux
        | Target::Freebsd
        | Target::Openbsd
        | Target::Netbsd => url_matches(source, expected),
        _ => true,
    }
}

fn url_matches(source: &str, expected: &str) -> bool {
    source.trim().trim_end_matches('/') == expected.trim().trim_end_matches('/')
}

fn prefixed_source_matches(source: &str, expected: &str) -> bool {
    let source = source.trim().trim_end_matches('/');
    let expected = expected.trim().trim_end_matches('/');
    source == expected
        || source
            .strip_prefix(expected)
            .is_some_and(|suffix| suffix.starts_with('/'))
}

fn apt_source_matches(source: &str, expected: &str) -> bool {
    let mut found = false;
    for line in source.lines() {
        let mut fields = line.split_whitespace();
        if !matches!(fields.next(), Some("deb" | "deb-src")) {
            continue;
        }
        found = true;
        let Some(url) =
            fields.find(|field| field.starts_with("http://") || field.starts_with("https://"))
        else {
            return false;
        };
        if !url_matches(url, expected) {
            return false;
        }
    }
    found
}

fn verify_applied(target: Target, mirror: &str, scope: Scope) -> io::Result<()> {
    let mut checked = false;
    for target in installed_targets(target, scope) {
        checked = true;
        let status = inspect_with_expected(target, mirror, scope)?;
        if !source_matches(target, &status, mirror) {
            return Err(io::Error::other(format!(
                "post-write verification failed for {}: configured source is {}",
                target_name(target),
                status.source.as_deref().unwrap_or("not configured")
            )));
        }
    }
    if checked {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!(
                "{} has no installed target that can be verified",
                target_name(target)
            ),
        ))
    }
}

fn execute(
    target: Target,
    action: Action,
    selector: Option<&str>,
    options: ExecuteOptions,
    config: &Config,
) -> io::Result<()> {
    validate_scope(target, options.scope)?;
    let mut cache = options
        .best
        .then(|| lm::probe::HealthCache::load(config.settings().cache_ttl_seconds))
        .transpose()?;
    let mirror = match action {
        Action::Set => {
            let mirror = select_mirror(target, selector, options.best, config, cache.as_mut())?;
            if options.verify {
                verify_mirror(target, &mirror, config)?;
            }
            Some(mirror)
        }
        Action::Reset => None,
    };
    let previous = if action == Action::Set && options.verify && !options.dry_run {
        snapshot_targets(target, config, options.scope)?
    } else {
        Vec::new()
    };
    let before = if options.format == OutputFormat::Json {
        if previous.is_empty() {
            current_sources(target, config, options.scope)?
        } else {
            previous.clone()
        }
    } else {
        Vec::new()
    };
    let result = execute_resolved(
        target,
        action,
        mirror.as_deref(),
        options.scope,
        options.dry_run,
        options.verify,
    );
    let result = match result {
        Ok(()) => {
            if options.format == OutputFormat::Json {
                let records =
                    change_records(target, action, mirror.as_deref(), &before, options, config)?;
                print_change_records(&records)
            } else {
                print_table_change(
                    match action {
                        Action::Set => "set",
                        Action::Reset => "reset",
                    },
                    target_name(target),
                    mirror.as_deref(),
                    scope_name(options.scope),
                    options.dry_run,
                    options.verify && action == Action::Set,
                );
                Ok(())
            }
        }
        Err(error) if !previous.is_empty() && !options.dry_run => {
            let mode = rollback_mode(&[&previous]);
            match restore_snapshots(&[&previous], options.scope) {
                Ok(mode) => Err(io::Error::other(format!(
                    "{error}; rollback={}",
                    mode.label()
                ))),
                Err(rollback) => Err(io::Error::other(format!(
                    "{error}; rollback={}-failed: {rollback}",
                    mode.label()
                ))),
            }
        }
        Err(error) => Err(error),
    };
    if let Some(cache) = cache {
        cache.save()?;
    }
    result
}

fn execute_resolved(
    target: Target,
    action: Action,
    mirror: Option<&str>,
    scope: Scope,
    dry_run: bool,
    verify: bool,
) -> io::Result<()> {
    if dry_run {
        return Ok(());
    }
    run_action(target, action, mirror, scope)?;
    if verify && action == Action::Set {
        verify_applied(target, mirror.unwrap_or_default(), scope)?;
    }
    Ok(())
}

fn validate_scope(target: Target, scope: Scope) -> io::Result<()> {
    if target_capabilities(target).supports(scope) {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("{} does not support {scope:?} scope", target_name(target)),
        ))
    }
}

fn execute_all(
    action: Action,
    selector: Option<&str>,
    options: ExecuteOptions,
    config: &Config,
) -> io::Result<()> {
    if options.atomic && action != Action::Set {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "--atomic is supported only for set all",
        ));
    }
    let mut cache = options
        .best
        .then(|| lm::probe::HealthCache::load(config.settings().cache_ttl_seconds))
        .transpose()?;
    let mut plan = Vec::new();
    for &target in ALL_TARGETS {
        if matches!(action, Action::Set | Action::Reset)
            && ((target == Target::Flutter && config.enabled("dart"))
                || (matches!(target, Target::Cabal | Target::Stack) && config.enabled("haskell")))
        {
            continue;
        }
        if !config.enabled(catalog_name(target)) {
            continue;
        }
        if let Err(error) = validate_scope(target, options.scope) {
            eprintln!("{}: skipped; {error}", target_name(target));
            continue;
        }
        if !is_installed(target) {
            if options.atomic {
                return Err(io::Error::new(
                    io::ErrorKind::NotFound,
                    format!(
                        "{} is not installed; --atomic refuses a partial plan",
                        target_name(target)
                    ),
                ));
            }
            eprintln!("{}: skipped; not installed", target_name(target));
            continue;
        }
        if options.atomic && !atomic_supported(target) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!(
                    "{} cannot participate in --atomic; use a single target or omit --atomic",
                    target_name(target)
                ),
            ));
        }
        let mirror = match action {
            Action::Set => {
                match select_mirror(target, selector, options.best, config, cache.as_mut()) {
                    Ok(mirror) => {
                        if options.verify {
                            verify_mirror(target, &mirror, config)?;
                        }
                        Some(mirror)
                    }
                    Err(_error)
                        if selector.is_none()
                            && lm::catalog::builtin_mirrors(catalog_name(target))?.is_empty()
                            && config.default_for(catalog_name(target)).is_none() =>
                    {
                        eprintln!(
                            "{}: skipped; configure a default or pass a URL",
                            target_name(target)
                        );
                        continue;
                    }
                    Err(error) => return Err(error),
                }
            }
            Action::Reset => None,
        };
        let previous = if (options.atomic || options.verify) && !options.dry_run {
            snapshot_targets(target, config, options.scope)?
        } else {
            Vec::new()
        };
        let before = if options.format == OutputFormat::Json {
            if previous.is_empty() {
                current_sources(target, config, options.scope)?
            } else {
                previous.clone()
            }
        } else {
            Vec::new()
        };
        plan.push((target, mirror, previous, before));
    }
    let mut applied: Vec<(Target, Snapshot)> = Vec::new();
    let mut records = Vec::new();
    for (target, mirror, previous, before) in plan {
        if let Err(error) = execute_resolved(
            target,
            action,
            mirror.as_deref(),
            options.scope,
            options.dry_run,
            options.verify,
        ) {
            if (options.atomic || options.verify) && !options.dry_run {
                let snapshots = std::iter::once(&previous)
                    .chain(applied.iter().rev().map(|(_, snapshot)| snapshot))
                    .collect::<Vec<_>>();
                let mode = rollback_mode(&snapshots);
                return Err(io::Error::other(
                    match restore_snapshots(&snapshots, options.scope) {
                        Ok(mode) => format!("{error}; rollback={}", mode.label()),
                        Err(rollback) => {
                            format!("{error}; rollback={}-failed: {rollback}", mode.label())
                        }
                    },
                ));
            }
            return Err(error);
        }
        if (options.atomic || options.verify) && !options.dry_run {
            applied.push((target, previous));
        }
        if options.format == OutputFormat::Json {
            records.extend(change_records(
                target,
                action,
                mirror.as_deref(),
                &before,
                options,
                config,
            )?);
        } else {
            print_table_change(
                match action {
                    Action::Set => "set",
                    Action::Reset => "reset",
                },
                target_name(target),
                mirror.as_deref(),
                scope_name(options.scope),
                options.dry_run,
                options.verify && action == Action::Set,
            );
        }
    }
    if let Some(cache) = cache {
        cache.save()?;
    }
    if options.format == OutputFormat::Json {
        print_change_records(&records)?;
    }
    Ok(())
}

fn atomic_supported(target: Target) -> bool {
    target_capabilities(target).atomic
}

fn inspect(target: Target, config: &Config, scope: Scope) -> io::Result<lm::ToolStatus> {
    let name = catalog_name(target);
    let expected =
        if lm::catalog::builtin_mirrors(name)?.is_empty() && config.default_for(name).is_none() {
            String::new()
        } else {
            lm::catalog::resolve(name, None, config)?
        };
    inspect_with_expected(target, &expected, scope)
}

fn inspect_with_expected(
    target: Target,
    expected: &str,
    scope: Scope,
) -> io::Result<lm::ToolStatus> {
    match target {
        Target::Npm | Target::Pnpm | Target::Yarn | Target::Bun | Target::Node => {
            let name = if target == Target::Node {
                "npm"
            } else {
                target_name(target)
            };
            lm::node::status(name, expected, scope)
        }
        Target::Go => lm::go::status(expected),
        Target::Pip | Target::Pip3 | Target::Python => {
            let name = if target == Target::Pip3 {
                "pip3"
            } else {
                "pip"
            };
            lm::python::status(name, expected)
        }
        Target::Uv => lm::uv::status(expected, scope),
        Target::Pdm => lm::pdm::status(expected),
        Target::Poetry => lm::poetry::status(expected, scope),
        Target::Composer | Target::Php => lm::php::status(expected),
        Target::Gem | Target::Ruby => lm::ruby::gem_status(expected),
        Target::Bundle => lm::ruby::bundle_status(expected),
        Target::Maven | Target::Java => lm::java::maven_status(expected),
        Target::Gradle => lm::java::gradle_status(expected),
        Target::Sbt => lm::sbt::status(expected),
        Target::Cargo | Target::Rust => lm::rust::status(expected, scope),
        Target::Docker => lm::docker::status(scope),
        Target::Buildkit => lm::docker::buildkit_status(scope),
        Target::Containerd | Target::Nerdctl => {
            lm::container::containerd_status(target_name(target), scope)
        }
        Target::Podman => lm::container::podman_status(scope),
        Target::Helm => lm::helm::status(scope),
        Target::Conda | Target::Mamba => lm::conda::status(target_name(target), expected),
        Target::Dart => lm::dart::dart_status(expected, scope),
        Target::Flutter => lm::dart::flutter_status(expected, scope),
        Target::Nuget | Target::Dotnet => lm::nuget::status(scope),
        Target::Cran | Target::R => lm::r::status(expected),
        Target::Huggingface => lm::huggingface::status(expected, scope),
        Target::Apt => lm::platform::apt_status(scope),
        Target::Apk => lm::platform::apk_status(scope),
        Target::Brew => lm::platform::brew_status(scope),
        Target::Rustup => lm::platform::rustup_status(scope),
        Target::Hex => lm::platform::hex_status(scope),
        Target::Julia => lm::platform::julia_status(scope),
        Target::Cpan => lm::platform::cpan_status(scope),
        Target::Winget => lm::platform::winget_status(scope),
        Target::Opam => lm::platform::opam_status(scope),
        Target::Rye => lm::platform::env_status("rye", "rye", "RYE_PYPI_MIRROR", expected, scope),
        Target::Nvm => {
            lm::platform::env_status("node", "nvm", "NVM_NODEJS_ORG_MIRROR", expected, scope)
        }
        Target::Luarocks => lm::platform::luarocks_status(expected, scope),
        Target::Clojure => lm::platform::clojure_status(expected, scope),
        Target::Haskell | Target::Hackage | Target::Cabal => {
            lm::platform::cabal_status(expected, scope)
        }
        Target::Stack => lm::platform::stack_status(expected, scope),
        Target::Ocaml => lm::platform::opam_status(scope),
        Target::Cocoapods => lm::platform::cocoapods_status(expected, scope),
        Target::Flathub => lm::platform::flatpak_status(expected, scope),
        Target::Nix => lm::platform::env_status("nix", "nix", "NIX_CONFIG", expected, scope),
        Target::Guix => {
            lm::platform::env_status("guix", "guix", "GUIX_SUBSTITUTE_URLS", expected, scope)
        }
        Target::Emacs => lm::platform::emacs_status(expected, scope),
        Target::Tex => lm::platform::tex_status(expected, scope),
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
        Ok(status) => {
            let origin = status
                .path
                .as_ref()
                .map(|path| path.display().to_string())
                .unwrap_or_else(|| "external command".to_owned());
            StatusRecord {
                target: target_name(target).to_owned(),
                scope: format!("{scope:?}").to_lowercase(),
                configured: status.configured,
                version: Some(status.version),
                source: status.source.map(|source| redact_text(&source)),
                path: status.path.map(|path| path.display().to_string()),
                origin,
                detail: Some(redact_text(&status.detail)),
                error: None,
            }
        }
        Err(error) => StatusRecord {
            target: target_name(target).to_owned(),
            scope: format!("{scope:?}").to_lowercase(),
            configured: false,
            version: None,
            source: None,
            path: None,
            origin: "unavailable".to_owned(),
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
    all_scopes: bool,
    explain: bool,
) -> io::Result<()> {
    let targets: &[Target] = if target == Target::All {
        ALL_TARGETS
    } else {
        std::slice::from_ref(&target)
    };
    let mut records = Vec::new();
    for target in targets.iter().copied() {
        if !config.enabled(catalog_name(target)) || (only_installed && !is_installed(target)) {
            continue;
        }
        for current_scope in [Scope::Project, Scope::User, Scope::System] {
            if !all_scopes && current_scope != scope {
                continue;
            }
            if validate_scope(target, current_scope).is_ok() {
                records.push(status_record(target, config, current_scope));
            }
        }
    }
    if format == OutputFormat::Json {
        print_json(&serde_json::Value::Array(
            records
                .iter()
                .map(|record| {
                    let mut value = status_json(record);
                    if explain {
                        value["explanation"] = explanation_json(
                            lm::catalog::find(&record.target)
                                .map_or(record.target.as_str(), |spec| spec.name),
                            None,
                            config,
                            scope_from_name(&record.scope),
                            record,
                        );
                    }
                    value
                })
                .collect(),
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
            if explain {
                print_explanation(explanation_json(
                    lm::catalog::find(&record.target)
                        .map_or(record.target.as_str(), |spec| spec.name),
                    None,
                    config,
                    scope_from_name(&record.scope),
                    record,
                ));
            }
        }
    }
    let ok = if all_scopes {
        !records.is_empty() && records.iter().all(|record| record.error.is_none())
    } else {
        !records.is_empty()
            && records
                .iter()
                .all(|record| record.configured && record.error.is_none())
    };
    if ok {
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
        let diff = source_diff(current.source.as_deref(), desired.as_deref());
        records.push(serde_json::json!({
            "schema": lm::JSON_SCHEMA,
            "target": target_name(target),
            "scope": scope_name(scope),
            "action": if reset { "reset" } else { "set" },
            "current": current.source,
            "desired": desired.as_deref().map(redact_url),
            "path": current.path,
            "strategy": if current.path.is_some() { "file" } else { "command" },
            "diff": diff,
            "installed": is_installed(target),
            "error": error,
        }));
    }
    if format == OutputFormat::Json {
        print_json(&serde_json::Value::Array(records.clone()))?;
    } else {
        for record in &records {
            println!(
                "{}\t{}\t{}\t{}\t{}\n{}",
                record["target"].as_str().unwrap_or_default(),
                record["action"].as_str().unwrap_or_default(),
                record["desired"].as_str().unwrap_or("upstream"),
                record["path"].as_str().unwrap_or("external command"),
                record["error"].as_str().unwrap_or("ready"),
                record["diff"].as_str().unwrap_or_default()
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

fn source_diff(current: Option<&str>, desired: Option<&str>) -> String {
    let current = current
        .map(redact_text)
        .unwrap_or_else(|| "upstream".to_owned());
    let desired = desired
        .map(redact_url)
        .unwrap_or_else(|| "upstream".to_owned());
    if current == desired {
        return "no changes".to_owned();
    }
    format!("--- current\n+++ desired\n@@\n- {current}\n+ {desired}")
}

fn doctor(
    target: Target,
    selector: Option<&str>,
    config: &Config,
    scope: Scope,
    options: ProbeOptions,
    explain: bool,
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
            measure_one(
                target,
                Some(url),
                config,
                &mut cache,
                options.parallelism,
                options.ip_version,
            )
            .ok()
            .and_then(|items| items.into_iter().next())
        });
        let error = status
            .error
            .clone()
            .or_else(|| desired.as_ref().err().map(std::string::ToString::to_string));
        let mut record = serde_json::json!({
            "schema": lm::JSON_SCHEMA,
            "target": target_name(target),
            "installed": is_installed(target),
            "configured": status.configured,
            "source": status.source,
            "path": status.path,
            "health": health.as_ref().map(|record| record.state.clone()),
            "code": health.as_ref().and_then(|record| record.code.clone()),
            "latency_ms": health.as_ref().and_then(|record| record.milliseconds),
            "metrics": health
                .as_ref()
                .and_then(|record| record.metrics.as_ref())
                .map(|metrics| serde_json::json!({
                    "remote_ip": metrics.remote_ip,
                    "content_type": metrics.content_type,
                    "dns_milliseconds": metrics.dns_milliseconds,
                    "connect_milliseconds": metrics.connect_milliseconds,
                    "tls_milliseconds": metrics.tls_milliseconds,
                    "ttfb_milliseconds": metrics.ttfb_milliseconds,
                })),
            "health_usable": health.as_ref().is_some_and(probe_is_usable),
            "error": error,
        });
        if explain {
            record["explanation"] =
                explanation_json(catalog_name(target), selector, config, scope, &status);
        }
        records.push(record);
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
            if explain {
                print_explanation(record["explanation"].clone());
            }
        }
    }
    if !records.is_empty()
        && records.iter().all(|record| {
            record["configured"].as_bool() == Some(true)
                && record["error"].is_null()
                && record["health_usable"].as_bool() == Some(true)
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

fn scope_from_name(scope: &str) -> Scope {
    match scope {
        "project" => Scope::Project,
        "system" => Scope::System,
        _ => Scope::User,
    }
}

fn explanation_json(
    target: &str,
    selector: Option<&str>,
    config: &Config,
    scope: Scope,
    status: &StatusRecord,
) -> serde_json::Value {
    let default = config.default_for(target).map(redact_selection);
    let mirrors = config
        .mirrors_for(target)
        .map(|items| {
            items
                .iter()
                .map(|item| redact_selection(item))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    serde_json::json!({
        "config": config.path,
        "config_sources": config
            .sources()
            .iter()
            .map(|source| {
                serde_json::json!({
                    "path": source.path,
                    "active": source.active,
                    "loaded": source.loaded,
                })
            })
            .collect::<Vec<_>>(),
        "target": target,
        "enabled": config.enabled(target),
        "scope": scope_name(scope),
        "requested_selector": selector.map(redact_selection),
        "configured_default": default,
        "default_source": config.default_source(target),
        "target_source": config.target_source(target),
        "mirror_pool": mirrors,
        "selection_order": [
            "CLI selector",
            "[defaults]",
            "[targets.<target>].default",
            "built-in first mirror",
        ],
        "adapter": {
            "path": status.path,
            "source": status.source,
            "detail": status.detail,
            "error": status.error,
        },
    })
}

fn redact_selection(value: &str) -> String {
    if config_url(value) {
        redact_url(value)
    } else {
        value.to_owned()
    }
}

fn config_url(value: &str) -> bool {
    value.starts_with("http://") || value.starts_with("https://")
}

fn print_explanation(value: serde_json::Value) {
    let config = value["config"].as_str().unwrap_or_default();
    let target = value["target"].as_str().unwrap_or_default();
    let scope = value["scope"].as_str().unwrap_or_default();
    let default = value["configured_default"].as_str().unwrap_or("none");
    let pool = value["mirror_pool"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter_map(serde_json::Value::as_str)
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default();
    let adapter = value["adapter"]["path"]
        .as_str()
        .unwrap_or("external command");
    println!(
        "  explain: target={target} scope={scope} config={config} default={default} pool=[{pool}] adapter={adapter}"
    );
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
            "schema": lm::JSON_SCHEMA,
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
            "schema": lm::JSON_SCHEMA,
            "config": config.path,
            "target": spec.name,
            "aliases": spec.aliases,
            "mirrors": spec.mirrors.iter().map(|mirror| serde_json::json!({"name": mirror.name, "url": redact_url(mirror.url), "origin": "builtin"})).collect::<Vec<_>>(),
            "custom_mirrors": config.custom_mirrors().map(|(name, url)| serde_json::json!({"name": name, "url": redact_url(url), "origin": "config"})).collect::<Vec<_>>()
        })
    } else {
        let category = query.unwrap_or("target");
        serde_json::json!({
            "schema": lm::JSON_SCHEMA,
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
            | "buildkit"
            | "containerd"
            | "podman"
            | "helm"
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
    if target == Target::Buildkit {
        return lm::docker::buildkit_available();
    }
    target_capabilities(target)
        .commands
        .iter()
        .any(|command| lm::command_exists(command))
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
            options.ip_version,
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
    state_is_usable(&record.target, &record.state)
}

fn state_is_usable(target: &str, state: &str) -> bool {
    state == "healthy"
        || (state == "auth-required"
            && matches!(target, "docker" | "buildkit" | "containerd" | "podman"))
}

fn measure_one(
    target: Target,
    selector: Option<&str>,
    config: &Config,
    cache: &mut lm::probe::HealthCache,
    parallelism: Option<usize>,
    ip_version: lm::probe::IpVersion,
) -> io::Result<Vec<MeasureRecord>> {
    let name = catalog_name(target);
    let specs = lm::catalog::builtin_mirrors(name)?;
    let candidates = if let Some(selector) = selector {
        vec![(
            selector.to_owned(),
            lm::catalog::resolve(name, Some(selector), config)?,
        )]
    } else {
        let mut candidates = if let Some(selections) = config.mirrors_for(name) {
            selections
                .iter()
                .map(|selection| {
                    Ok((
                        selection.clone(),
                        lm::catalog::resolve(name, Some(selection), config)?,
                    ))
                })
                .collect::<io::Result<Vec<_>>>()?
        } else {
            specs
                .iter()
                .map(|mirror| (mirror.name.to_owned(), mirror.url.to_owned()))
                .collect::<Vec<_>>()
        };
        if config.mirrors_for(name).is_none() && config.default_for(name).is_some() {
            let url = lm::catalog::resolve(name, None, config)?;
            if !candidates.iter().any(|(_, candidate)| candidate == &url) {
                candidates.push(("configured".to_owned(), url));
            }
        }
        if candidates.is_empty() {
            return Ok(vec![MeasureRecord {
                target: name.to_owned(),
                mirror: "configured".to_owned(),
                url: String::new(),
                probe_url: None,
                code: None,
                state: "unavailable".to_owned(),
                detail: None,
                milliseconds: None,
                metrics: None,
                cached: false,
                error: Some(format!("{name} requires a mirror name or URL")),
            }]);
        }
        candidates
    };
    let mut settings = config.settings();
    if let Some(parallelism) = parallelism {
        settings.parallelism = parallelism;
    }
    let mut records: Vec<Option<MeasureRecord>> = (0..candidates.len()).map(|_| None).collect();
    let mut pending = Vec::new();
    for (index, (mirror, url)) in candidates.iter().enumerate() {
        if let Some(result) = cache.get(name, url, ip_version) {
            records[index] = Some(MeasureRecord {
                target: name.to_owned(),
                mirror: mirror.clone(),
                url: url.clone(),
                probe_url: Some(result.probe_url.clone()),
                code: Some(result.code.clone()),
                state: result.state.clone(),
                detail: Some(result.detail.clone()),
                milliseconds: Some(result.milliseconds),
                metrics: Some(result.metrics.clone()),
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
                            ip_version,
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
                    cache.put(name, url, ip_version, result.clone());
                    records[index] = Some(MeasureRecord {
                        target: name.to_owned(),
                        mirror: mirror.clone(),
                        url: url.clone(),
                        probe_url: Some(result.probe_url),
                        code: Some(result.code),
                        state: result.state,
                        detail: Some(result.detail),
                        milliseconds: Some(result.milliseconds),
                        metrics: Some(result.metrics),
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
                        metrics: None,
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
        "schema": lm::JSON_SCHEMA,
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
        "schema": lm::JSON_SCHEMA,
        "target": record.target.clone(),
        "mirror": record.mirror.clone(),
        "url": redact_url(&record.url),
        "probe_url": record.probe_url.as_deref().map(redact_url),
        "code": record.code.clone(),
        "state": record.state.clone(),
        "detail": record.detail.clone(),
        "milliseconds": record.milliseconds,
        "metrics": record.metrics.as_ref().map(|metrics| serde_json::json!({
            "remote_ip": metrics.remote_ip,
            "content_type": metrics.content_type,
            "dns_milliseconds": metrics.dns_milliseconds,
            "connect_milliseconds": metrics.connect_milliseconds,
            "tls_milliseconds": metrics.tls_milliseconds,
            "ttfb_milliseconds": metrics.ttfb_milliseconds,
        })),
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
        ConfigCommand::Init => {
            if config.path.exists() {
                return Err(io::Error::new(
                    io::ErrorKind::AlreadyExists,
                    format!("configuration already exists: {}", config.path.display()),
                ));
            }
            if let Some(parent) = config
                .path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
            {
                std::fs::create_dir_all(parent)?;
            }
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&config.path)?;
            file.write_all(STARTER_CONFIG.as_bytes())?;
            println!("created\t{}", config.path.display());
            Ok(())
        }
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
        ConfigCommand::Sources { format } => {
            let value = serde_json::json!({
                "schema": lm::JSON_SCHEMA,
                "config": config.path,
                "sources": config.sources().iter().map(|source| serde_json::json!({
                    "path": source.path,
                    "active": source.active,
                    "loaded": source.loaded,
                })).collect::<Vec<_>>(),
            });
            if format == OutputFormat::Json {
                print_json(&value)
            } else {
                for source in config.sources() {
                    println!(
                        "{}\t{}",
                        if !source.active {
                            "disabled"
                        } else if source.loaded {
                            "loaded"
                        } else {
                            "not-found"
                        },
                        source.path.display()
                    );
                }
                Ok(())
            }
        }
    }
}

fn catalog_command(command: CatalogCommand) -> io::Result<()> {
    match command {
        CatalogCommand::Lint { format } => {
            lm::catalog::lint()?;
            let target_count = lm::catalog::targets().len();
            let mirror_count = lm::catalog::targets()
                .iter()
                .map(|target| target.mirrors.len())
                .sum::<usize>();
            let value = serde_json::json!({
                "schema": lm::JSON_SCHEMA,
                "valid": true,
                "targets": target_count,
                "mirrors": mirror_count,
            });
            if format == OutputFormat::Json {
                print_json(&value)
            } else {
                println!("valid\ttargets={target_count}\tmirrors={mirror_count}");
                Ok(())
            }
        }
    }
}

fn env_command(
    target: Target,
    selector: Option<&str>,
    shell: EnvShell,
    config: &Config,
) -> io::Result<()> {
    let mirror = lm::catalog::resolve(catalog_name(target), selector, config)?;
    let assignments = environment_values(target, &mirror).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!(
                "{} does not expose shell environment mirror variables",
                target_name(target)
            ),
        )
    })?;
    for (name, value) in assignments {
        let value = match shell {
            EnvShell::Sh | EnvShell::Fish => shell_single_quote(&value),
            EnvShell::Powershell => powershell_single_quote(&value),
        };
        match shell {
            EnvShell::Sh => println!("export {name}={value}"),
            EnvShell::Fish => println!("set -gx {name} {value}"),
            EnvShell::Powershell => println!("$env:{name} = {value}"),
        }
    }
    Ok(())
}

fn environment_values(target: Target, mirror: &str) -> Option<Vec<(&'static str, String)>> {
    let base = mirror.trim_end_matches('/');
    let values = match target {
        Target::Dart => vec![("PUB_HOSTED_URL", mirror.to_owned())],
        Target::Flutter => vec![
            ("PUB_HOSTED_URL", flutter_pub_url(mirror)),
            ("FLUTTER_STORAGE_BASE_URL", mirror.to_owned()),
        ],
        Target::Huggingface => vec![("HF_ENDPOINT", mirror.to_owned())],
        Target::Brew => vec![
            (
                "HOMEBREW_API_DOMAIN",
                format!("{base}/homebrew-bottles/api"),
            ),
            ("HOMEBREW_BOTTLE_DOMAIN", format!("{base}/homebrew-bottles")),
            (
                "HOMEBREW_BREW_GIT_REMOTE",
                format!("{base}/git/homebrew/brew.git"),
            ),
            (
                "HOMEBREW_CORE_GIT_REMOTE",
                format!("{base}/git/homebrew/homebrew-core.git"),
            ),
        ],
        Target::Rustup => vec![
            ("RUSTUP_DIST_SERVER", mirror.to_owned()),
            ("RUSTUP_UPDATE_ROOT", format!("{base}/rustup")),
        ],
        Target::Julia => vec![("JULIA_PKG_SERVER", mirror.to_owned())],
        Target::Cpan => vec![("PERL_CPAN_MIRROR", mirror.to_owned())],
        Target::Rye => vec![("RYE_PYPI_MIRROR", mirror.to_owned())],
        Target::Nvm => vec![("NVM_NODEJS_ORG_MIRROR", mirror.to_owned())],
        Target::Nix => vec![("NIX_CONFIG", mirror.to_owned())],
        Target::Guix => vec![("GUIX_SUBSTITUTE_URLS", mirror.to_owned())],
        _ => return None,
    };
    Some(values)
}

fn flutter_pub_url(mirror: &str) -> String {
    if mirror.ends_with("/flutter") {
        return format!("{}/dart-pub", mirror.trim_end_matches("/flutter"));
    }
    if mirror == "https://storage.flutter-io.cn" {
        return "https://pub.flutter-io.cn".to_owned();
    }
    mirror.to_owned()
}

fn shell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}

fn powershell_single_quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

fn completions(shell: CompletionShell) -> io::Result<()> {
    let mut words = BTreeSet::from([
        "list",
        "measure",
        "check",
        "get",
        "set",
        "reset",
        "config",
        "doctor",
        "plan",
        "diff",
        "completions",
        "env",
        "--help",
        "--version",
        "--format",
        "--scope",
        "--dry-run",
        "--verify",
        "--best",
        "--atomic",
        "--only-installed",
        "--no-cache",
        "--parallelism",
        "--explain",
        "--shell",
    ]);
    for target in lm::catalog::targets() {
        words.insert(target.name);
        words.extend(target.aliases.iter().copied());
    }
    let words = words.into_iter().collect::<Vec<_>>().join(" ");
    let script = match shell {
        CompletionShell::Bash => format!(
            r#"_lm_completions() {{
  local current="${{COMP_WORDS[COMP_CWORD]}}"
  COMPREPLY=( $(compgen -W "{words}" -- "$current") )
}}
complete -F _lm_completions lm
"#
        ),
        CompletionShell::Zsh => format!(
            r#"#compdef lm
_lm_completions() {{
  _arguments '*:argument:({words})'
}}
_lm_completions "$@"
"#
        ),
        CompletionShell::Fish => {
            format!("complete -c lm -f -a '{}'", words)
        }
        CompletionShell::Powershell => format!(
            r#"Register-ArgumentCompleter -Native -CommandName lm -ScriptBlock {{
  param($wordToComplete)
  '{words}'.Split(' ') |
    Where-Object {{ $_ -like "$wordToComplete*" }} |
    ForEach-Object {{ [System.Management.Automation.CompletionResult]::new($_, $_, 'ParameterValue', $_) }}
}}
"#
        ),
    };
    print!("{script}");
    Ok(())
}

fn run() -> io::Result<()> {
    let cli = Cli::parse();
    let skip_config = matches!(&cli.command, Commands::Catalog { .. });
    let config = Config::load_with_options(cli.config.as_deref(), cli.no_config || skip_config)?;
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
            ip,
        }
        | Commands::Check {
            target,
            mirror,
            format,
            cache_ttl,
            no_cache,
            only_installed,
            parallelism,
            ip,
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
                ip_version: ip.version(),
            },
        ),
        Commands::Get {
            target,
            scope,
            format,
            only_installed,
            all_scopes,
            explain,
        } => get(
            target,
            &config,
            scope,
            format,
            only_installed,
            all_scopes,
            explain,
        ),
        Commands::Set {
            target,
            mirror,
            best,
            scope,
            dry_run,
            verify,
            atomic,
            format,
        } => {
            if target == Target::All {
                execute_all(
                    Action::Set,
                    mirror.as_deref(),
                    ExecuteOptions {
                        best,
                        verify,
                        scope,
                        dry_run,
                        atomic,
                        format,
                    },
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
                    ExecuteOptions {
                        best,
                        verify,
                        scope,
                        dry_run,
                        atomic: false,
                        format,
                    },
                    &config,
                )
            }
        }
        Commands::Reset {
            target,
            scope,
            dry_run,
            format,
        } => {
            if target == Target::All {
                execute_all(
                    Action::Reset,
                    None,
                    ExecuteOptions {
                        best: false,
                        verify: false,
                        scope,
                        dry_run,
                        atomic: false,
                        format,
                    },
                    &config,
                )
            } else {
                execute(
                    target,
                    Action::Reset,
                    None,
                    ExecuteOptions {
                        best: false,
                        verify: false,
                        scope,
                        dry_run,
                        atomic: false,
                        format,
                    },
                    &config,
                )
            }
        }
        Commands::Config { command } => config_command(&config, command),
        Commands::Catalog { command } => catalog_command(command),
        Commands::Completions { shell } => completions(shell),
        Commands::Env {
            target,
            mirror,
            shell,
        } => env_command(target, mirror.as_deref(), shell, &config),
        Commands::Doctor {
            target,
            mirror,
            scope,
            format,
            cache_ttl,
            no_cache,
            only_installed,
            parallelism,
            explain,
            ip,
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
                ip_version: ip.version(),
            },
            explain,
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
        assert!(Cli::try_parse_from(["lm", "set", "buildkit", "daocloud", "--dry"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "set", "pip", "--best", "--verify"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "set", "pip", "tuna", "--format", "json"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "reset", "pip", "--format", "json"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "get", "pip", "--all-scopes"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "config", "init"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "completions", "bash"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "get", "huggingface", "--scope", "project"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "get", "pip", "--explain"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "doctor", "pip", "--explain"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "measure", "pip", "--ipv4"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "measure", "pip", "--ipv4", "--ipv6"]).is_err());
        assert!(Cli::try_parse_from(["lm", "config", "sources"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "catalog", "lint", "--format", "json"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "--no-config", "list"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "env", "huggingface", "hf-mirror"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "list", "helm"]).is_ok());
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
            metrics: None,
            cached: false,
            error: None,
        };
        assert!(probe_is_usable(&record));
    }

    #[test]
    fn fastest_mirror_ignores_unusable_sources() {
        let record = |url: &str, state: &str, milliseconds| MeasureRecord {
            target: "pip".to_owned(),
            mirror: url.to_owned(),
            url: url.to_owned(),
            probe_url: None,
            code: None,
            state: state.to_owned(),
            detail: None,
            milliseconds: Some(milliseconds),
            metrics: None,
            cached: false,
            error: None,
        };
        let selected = fastest_mirror(vec![
            record("https://slow.example", "healthy", 80),
            record("https://fast.example", "healthy", 20),
            record("https://broken.example", "unavailable", 1),
        ]);
        assert_eq!(selected.as_deref(), Some("https://fast.example"));
    }

    #[test]
    fn grouped_targets_detect_any_installed_member() {
        assert_eq!(
            target_capabilities(Target::Node).commands,
            &["npm", "pnpm", "yarn", "bun"]
        );
        assert_eq!(
            verification_targets(Target::Node),
            vec![Target::Npm, Target::Pnpm, Target::Yarn, Target::Bun]
        );
        assert_eq!(
            target_capabilities(Target::Java).commands,
            &["mvn", "gradle", "sbt"]
        );
        assert_eq!(
            target_capabilities(Target::Rust).commands,
            &["cargo", "rustup"]
        );
        assert_eq!(
            target_capabilities(Target::Dart).commands,
            &["dart", "flutter"]
        );
        assert_eq!(
            target_capabilities(Target::Haskell).commands,
            &["cabal", "stack"]
        );
        assert_eq!(
            verification_targets(Target::Python),
            vec![Target::Pip, Target::Uv, Target::Pdm, Target::Poetry]
        );
        assert_eq!(verification_targets(Target::Docker), vec![Target::Docker]);
    }

    #[test]
    fn capabilities_describe_scope_and_atomic_support() {
        let docker = target_capabilities(Target::Docker);
        assert!(!docker.supports(Scope::Project));
        assert!(docker.supports(Scope::User));
        assert!(docker.supports(Scope::System));
        assert!(docker.atomic);

        let npm = target_capabilities(Target::Npm);
        assert!(npm.supports(Scope::Project));
        assert!(!npm.supports(Scope::System));
        assert!(!npm.atomic);
    }

    #[test]
    fn post_write_verification_compares_source_only_for_unaware_adapters() {
        let status = lm::ToolStatus::new(
            "helm version".to_owned(),
            true,
            Some("https://charts.example.com/".to_owned()),
            None,
            "",
        );
        assert!(source_matches(
            Target::Helm,
            &status,
            "https://charts.example.com"
        ));
        assert!(!source_matches(
            Target::Helm,
            &status,
            "https://other.example.com"
        ));
    }

    #[test]
    fn post_write_verification_matches_platform_source_shapes() {
        let apt = lm::ToolStatus::new(
            "apt".to_owned(),
            true,
            Some(
                "deb https://mirror.example stable main\ndeb https://mirror.example stable updates"
                    .to_owned(),
            ),
            None,
            "",
        );
        assert!(source_matches(Target::Apt, &apt, "https://mirror.example/"));
        assert!(!source_matches(Target::Apt, &apt, "https://other.example"));

        let brew = lm::ToolStatus::new(
            "brew".to_owned(),
            true,
            Some("https://mirror.example/homebrew-bottles".to_owned()),
            None,
            "",
        );
        assert!(source_matches(
            Target::Brew,
            &brew,
            "https://mirror.example/"
        ));
        assert!(!source_matches(
            Target::Brew,
            &brew,
            "https://other.example"
        ));

        let arch = lm::ToolStatus::new(
            "pacman".to_owned(),
            true,
            Some("https://mirror.example/$repo/os/$arch".to_owned()),
            None,
            "",
        );
        assert!(source_matches(
            Target::Arch,
            &arch,
            "https://mirror.example"
        ));
        assert!(!source_matches(
            Target::Arch,
            &arch,
            "https://other.example"
        ));
    }

    #[test]
    fn env_output_matches_adapter_variables() {
        assert_eq!(
            environment_values(Target::Flutter, "https://mirror.example/flutter"),
            Some(vec![
                (
                    "PUB_HOSTED_URL",
                    "https://mirror.example/dart-pub".to_owned()
                ),
                (
                    "FLUTTER_STORAGE_BASE_URL",
                    "https://mirror.example/flutter".to_owned()
                ),
            ])
        );
        assert_eq!(shell_single_quote("a'b"), "'a'\\''b'");
        assert_eq!(powershell_single_quote("a'b"), "'a''b'");
    }
}
