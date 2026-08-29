use std::collections::BTreeSet;
use std::io;
use std::path::PathBuf;
use std::process;

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
    List { query: Option<String> },
    #[command(about = "Measure mirror availability and latency", visible_aliases = ["m", "cesu"])]
    Measure {
        target: Target,
        mirror: Option<String>,
    },
    #[command(about = "Show the current source", visible_alias = "g")]
    Get { target: Target },
    #[command(about = "Set a source, mirror name, or URL", visible_alias = "s")]
    Set {
        target: Target,
        mirror: Option<String>,
        #[arg(long, value_enum, default_value = "user")]
        scope: Scope,
        #[arg(long)]
        dry_run: bool,
    },
    #[command(about = "Reset to the upstream source", visible_alias = "r")]
    Reset {
        target: Target,
        #[arg(long, value_enum, default_value = "user")]
        scope: Scope,
        #[arg(long)]
        dry_run: bool,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
#[value(rename_all = "lower")]
enum Target {
    All,
    Npm,
    Pnpm,
    Yarn,
    Bun,
    Node,
    Go,
    Pip,
    Pip3,
    Python,
    Uv,
    Pdm,
    Poetry,
    Composer,
    Php,
    Gem,
    Bundle,
    Ruby,
    Maven,
    Gradle,
    Sbt,
    Java,
    Cargo,
    Rust,
    Docker,
    Containerd,
    Nerdctl,
    Podman,
    Conda,
    Mamba,
    Dart,
    Flutter,
    Nuget,
    Dotnet,
    Cran,
    R,
    #[value(alias = "hf", alias = "huggingface-hub")]
    Huggingface,
}

const ALL_TARGETS: &[Target] = &[
    Target::Npm,
    Target::Pnpm,
    Target::Yarn,
    Target::Bun,
    Target::Go,
    Target::Pip3,
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
];

#[derive(Clone, Copy)]
enum Action {
    Set,
    Reset,
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
        Target::Npm | Target::Pnpm | Target::Yarn | Target::Bun | Target::Node => {
            let name = if target == Target::Node {
                "npm"
            } else {
                target_name(target)
            };
            match action {
                Action::Set => lm::node::set(name, mirror.unwrap(), scope),
                Action::Reset => lm::node::unset(name, scope),
            }
        }
        Target::Go => match action {
            Action::Set => lm::go::set(mirror.unwrap()),
            Action::Reset => lm::go::unset(),
        },
        Target::Pip | Target::Pip3 | Target::Python => {
            let name = if target == Target::Pip { "pip" } else { "pip3" };
            match action {
                Action::Set => lm::python::set(name, mirror.unwrap()),
                Action::Reset => lm::python::unset(name),
            }
        }
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
        Target::Maven | Target::Java => match action {
            Action::Set => lm::java::maven_set(mirror.unwrap()),
            Action::Reset => lm::java::maven_unset(),
        },
        Target::Gradle => match action {
            Action::Set => lm::java::gradle_set(mirror.unwrap()),
            Action::Reset => lm::java::gradle_unset(),
        },
        Target::Sbt => match action {
            Action::Set => lm::sbt::set(mirror.unwrap()),
            Action::Reset => lm::sbt::unset(),
        },
        Target::Cargo | Target::Rust => match action {
            Action::Set => lm::rust::set(mirror.unwrap(), scope),
            Action::Reset => lm::rust::unset(scope),
        },
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
        Target::Dart => match action {
            Action::Set => lm::dart::dart_set(mirror.unwrap(), scope),
            Action::Reset => lm::dart::unset(scope),
        },
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
        Target::All => unreachable!(),
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
    if dry_run {
        match action {
            Action::Set => println!(
                "would set {} mirror to {} (scope={scope:?})",
                target_name(target),
                mirror.as_deref().unwrap_or_default()
            ),
            Action::Reset => println!(
                "would reset {} mirror (scope={scope:?})",
                target_name(target)
            ),
        }
        return Ok(());
    }
    run_action(target, action, mirror.as_deref(), scope)?;
    let verb = match action {
        Action::Set => "set",
        Action::Reset => "reset",
    };
    println!("{verb} {} mirror", target_name(target));
    Ok(())
}

fn validate_scope(target: Target, scope: Scope) -> io::Result<()> {
    let supported = match scope {
        Scope::User => true,
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
        ),
        Scope::System => matches!(
            target,
            Target::Cargo
                | Target::Rust
                | Target::Dart
                | Target::Flutter
                | Target::Huggingface
                | Target::Docker
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

fn execute_all(
    action: Action,
    selector: Option<&str>,
    scope: Scope,
    dry_run: bool,
    config: &Config,
) -> io::Result<()> {
    let mut failed = 0;
    for &target in ALL_TARGETS {
        if let Err(error) = execute(target, action, selector, scope, dry_run, config) {
            failed += 1;
            eprintln!("{}: {error}", target_name(target));
        }
    }
    if failed == 0 {
        Ok(())
    } else {
        Err(io::Error::other(format!("{failed} target(s) failed")))
    }
}

fn inspect(target: Target, config: &Config, scope: Scope) -> io::Result<lm::ToolStatus> {
    let expected = lm::catalog::resolve(catalog_name(target), None, config).unwrap_or_default();
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
            let name = if target == Target::Pip { "pip" } else { "pip3" };
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
        Target::All => unreachable!(),
    }
}

fn print_status(target: Target, config: &Config, scope: Scope) -> bool {
    match inspect(target, config, scope) {
        Ok(status) => {
            let state = if status.configured {
                "configured"
            } else {
                "not configured"
            };
            println!(
                "{}: {state}; {}; {}",
                target_name(target),
                status.version,
                status.detail
            );
            status.configured
        }
        Err(error) => {
            eprintln!("{}: unavailable; {error}", target_name(target));
            false
        }
    }
}

fn list(query: Option<&str>, config: &Config) -> io::Result<()> {
    println!("config: {}", config.path.display());
    if matches!(query, Some("mirror")) {
        let mut names = BTreeSet::new();
        for target in lm::catalog::targets() {
            for mirror in target.mirrors {
                names.insert(mirror.name);
            }
        }
        for (name, _) in config.custom_mirrors() {
            names.insert(name);
        }
        for name in names {
            println!("{name}");
        }
        return Ok(());
    }
    if matches!(query, Some("target" | "os" | "lang" | "ware")) || query.is_none() {
        let mut names = BTreeSet::new();
        for target in lm::catalog::targets() {
            names.insert(target.name);
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
        println!("{}\t{}", mirror.name, mirror.url);
    }
    for (name, url) in config.custom_mirrors() {
        println!("{name}\t{url} (config)");
    }
    Ok(())
}

fn measure_one(target: &str, selector: Option<&str>, config: &Config) -> io::Result<bool> {
    let specs = lm::catalog::builtin_mirrors(target)?;
    let candidates = if let Some(selector) = selector {
        vec![(
            selector.to_owned(),
            lm::catalog::resolve(target, Some(selector), config)?,
        )]
    } else if specs.is_empty() {
        vec![(
            "configured".to_owned(),
            lm::catalog::resolve(target, None, config)?,
        )]
    } else {
        specs
            .iter()
            .map(|mirror| (mirror.name.to_owned(), mirror.url.to_owned()))
            .collect()
    };
    let mut all_available = true;
    for (name, url) in candidates {
        let probe_url = url.split_once(',').map_or(url.as_str(), |(url, _)| url);
        match lm::probe::probe(probe_url) {
            Ok(result) => {
                let available = result.code != "404" && !result.code.starts_with('5');
                all_available &= available;
                println!("{name}\t{url}\t{}\t{}ms", result.code, result.milliseconds);
            }
            Err(error) => {
                all_available = false;
                println!("{name}\t{url}\tfailed\t{error}");
            }
        }
    }
    Ok(all_available)
}

fn measure(target: Target, selector: Option<&str>, config: &Config) -> io::Result<()> {
    let mut all_available = true;
    if target == Target::All {
        for &target in ALL_TARGETS {
            all_available &= measure_one(catalog_name(target), selector, config)?;
        }
    } else {
        all_available = measure_one(catalog_name(target), selector, config)?;
    }
    if all_available {
        Ok(())
    } else {
        Err(io::Error::other("one or more mirrors are unavailable"))
    }
}

fn run() -> io::Result<()> {
    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref())?;
    match cli.command {
        Commands::List { query } => list(query.as_deref(), &config),
        Commands::Measure { target, mirror } => measure(target, mirror.as_deref(), &config),
        Commands::Get { target } => {
            let targets: &[Target] = if target == Target::All {
                ALL_TARGETS
            } else {
                std::slice::from_ref(&target)
            };
            let mut healthy = true;
            for &target in targets {
                healthy &= print_status(target, &config, Scope::User);
            }
            if healthy {
                Ok(())
            } else {
                Err(io::Error::other("one or more targets are unavailable"))
            }
        }
        Commands::Set {
            target,
            mirror,
            scope,
            dry_run,
        } => {
            if target == Target::All {
                execute_all(Action::Set, mirror.as_deref(), scope, dry_run, &config)
            } else {
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
                execute_all(Action::Reset, None, scope, dry_run, &config)
            } else {
                execute(target, Action::Reset, None, scope, dry_run, &config)
            }
        }
    }
}

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_matches_chsrc_shaped_commands() {
        assert!(Cli::try_parse_from(["lm", "list", "docker"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "ls", "docker"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "measure", "docker"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "m", "docker"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "get", "go"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "g", "go"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "set", "docker", "daocloud", "--dry-run"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "set", "huggingface", "hf-mirror"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "s", "docker", "daocloud"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "reset", "docker"]).is_ok());
        assert!(Cli::try_parse_from(["lm", "r", "docker"]).is_ok());
    }
}
