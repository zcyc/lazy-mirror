use std::{process::Command, string};

use clap::{Parser, Subcommand};

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "lm")]
#[command(about = "A mirror setting cli for lazy", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(arg_required_else_help = true)]
    Set {
        name: String,
    },
    Unset {
        name: String,
    },
}

fn all(action: i32) {
    todo!()
}

fn npm(commands: &Commands) {
    match commands {
        Commands::Set { name: _ } => {
            lm::npm::set();
        }
        Commands::Unset { name: _ } => {
            lm::npm::unset();
        }
    };
}

fn pnpm(commands: &Commands) {
    match commands {
        Commands::Set { name: _ } => {
            lm::pnpm::set();
        }
        Commands::Unset { name: _ } => {
            lm::pnpm::unset();
        }
    };
}

fn yarn(commands: &Commands) {
    match commands {
        Commands::Set { name: _ } => {
            lm::yarn::set();
        }
        Commands::Unset { name: _ } => {
            lm::yarn::unset();
        }
    };
}

fn go(commands: &Commands) {
    match commands {
        Commands::Set { name: _ } => {
            lm::go::set();
        }
        Commands::Unset { name: _ } => {
            lm::go::unset();
        }
    };
}

fn pip(name: &String, commands: &Commands) {
    match commands {
        Commands::Set { name: _ } => {
            lm::pip::set(name);
        }
        Commands::Unset { name: _ } => {
            lm::pip::unset(name);
        }
    };
}

fn composer(commands: &Commands) {
    match commands {
        Commands::Set { name: _ } => {
            lm::composer::set();
        }
        Commands::Unset { name: _ } => {
            lm::composer::unset();
        }
    };
}

fn gem(commands: &Commands) {
    match commands {
        Commands::Set { name: _ } => {
            lm::gem::set();
        }
        Commands::Unset { name: _ } => {
            lm::gem::unset();
        }
    };
}

fn maven(action: i32) {
    todo!()
}

fn gradle(action: i32) {
    todo!()
}

fn cargo(action: i32) {
    let zshrc = r#"# set by lirror begin #
export RUSTUP_DIST_SERVER="https://rsproxy.cn"
export RUSTUP_UPDATE_ROOT="https://rsproxy.cn/rustup"
# set by lirror end #"#;
    // let new_zshrc = zshrc.replace(zshrc, "new string");
    // println!("new_zshrc: {new_zshrc}");
    let cmd = format!("echo '{zshrc}' >> ~/kk");
    println!("cmd: {cmd}");
    let output = Command::new("sh")
        .args(["-c", &cmd])
        .output()
        .expect("set cargo error");
    println!("output: {output:?}");
}

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Set { name } => match name.as_str() {
            "all" => all(1),
            // node
            "npm" => npm(&args.command),
            "pnpm" => pnpm(&args.command),
            "yarn" => yarn(&args.command),
            "node" => npm(&args.command),
            // go
            "go" => go(&args.command),
            // python
            "pip" => pip(name, &args.command),
            "pip3" => pip(name, &args.command),
            "python" => pip(&String::from("pip3"), &args.command),
            // php
            "composer" => composer(&args.command),
            "php" => composer(&args.command),
            // ruby
            "gem" => gem(&args.command),
            "ruby" => gem(&args.command),
            "gems" => gem(&args.command),
            "rubygems" => gem(&args.command),
            // java
            "maven" => maven(1),
            "gradle" => gradle(1),
            "java" => maven(1),
            // rust
            "cargo" => cargo(1),
            "rust" => cargo(1),
            // other
            _ => println!("not support it"),
        },
        Commands::Unset { name } => match name.as_str() {
            "all" => all(2),
            // node
            "npm" => npm(&args.command),
            "pnpm" => pnpm(&args.command),
            "yarn" => yarn(&args.command),
            "node" => npm(&args.command),
            // go
            "go" => go(&args.command),
            // python
            "pip" => pip(name, &args.command),
            "pip3" => pip(name, &args.command),
            "python" => pip(&String::from("pip3"), &args.command),
            // php
            "composer" => composer(&args.command),
            "php" => composer(&args.command),
            // ruby
            "gem" => gem(&args.command),
            "ruby" => gem(&args.command),
            "gems" => gem(&args.command),
            "rubygems" => gem(&args.command),
            // java
            "maven" => maven(2),
            "gradle" => gradle(2),
            "java" => maven(2),
            // rust
            "cargo" => cargo(2),
            "rust" => cargo(2),
            // other
            _ => println!("not support it"),
        },
    }
}
