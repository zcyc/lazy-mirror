use std::process::Command;

use clap::{Parser, Subcommand};

/// A fictional versioning CLI
#[derive(Debug, Parser)] // requires `derive` feature
#[command(name = "lirror")]
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
    println!("WIP")
}

fn npm(action: i32) {
    if action == 1 {
        let output = Command::new("sh")
            .args([
                "-c",
                "npm config set registry https://registry.npmmirror.com/",
            ])
            .output()
            .expect("failed to execute");
        println!("{:?}", output);
    } else if action == 2 {
        let output = Command::new("sh")
            .args(["-c", "npm config set registry https://registry.npmjs.org/"])
            .output()
            .expect("failed to execute");
        println!("{:?}", output);
    }
}

fn pnpm(action: i32) {
    if action == 1 {
        let output = Command::new("sh")
            .args([
                "-c",
                "pnpm config set registry https://registry.npmmirror.com/",
            ])
            .output()
            .expect("failed to execute");
        println!("{:?}", output);
    } else if action == 2 {
        let output = Command::new("sh")
            .args(["-c", "pnpm config set registry https://registry.npmjs.org/"])
            .output()
            .expect("failed to execute");
        println!("{:?}", output);
    }
}

fn yarn(action: i32) {
    if action == 1 {
        let output = Command::new("sh")
            .args([
                "-c",
                "yarn config set registry https://registry.npmmirror.com/",
            ])
            .output()
            .expect("failed to execute");
        println!("{:?}", output);
    } else if action == 2 {
        let output = Command::new("sh")
            .args(["-c", "yarn config set registry https://registry.npmjs.org/"])
            .output()
            .expect("failed to execute");
        println!("{:?}", output);
    }
}

fn go(action: i32) {
    if action == 1 {
        let output = Command::new("sh")
            .args(["-c", "go env -w GOPROXY=https://goproxy.cn,direct"])
            .output()
            .expect("failed to execute");
        println!("{:?}", output);
    } else if action == 2 {
        let output = Command::new("sh")
            .args(["-c", "go env -u GOPROXY"])
            .output()
            .expect("failed to execute");
        println!("{:?}", output);
    }
}

fn pip(action: i32) {
    if action == 1 {
        let output = Command::new("sh")
            .args([
                "-c",
                "pip config set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple",
            ])
            .output()
            .expect("failed to execute");
        println!("{:?}", output);
    } else if action == 2 {
        let output = Command::new("sh")
            .args(["-c", "pip config unset global.index-url"])
            .output()
            .expect("failed to execute");
        println!("{:?}", output);
    }
}

fn pip3(action: i32) {
    if action == 1 {
        let output = Command::new("sh")
            .args([
                "-c",
                "pip3 config set global.index-url https://pypi.tuna.tsinghua.edu.cn/simple",
            ])
            .output()
            .expect("failed to execute");
        println!("{:?}", output);
    } else if action == 2 {
        let output = Command::new("sh")
            .args(["-c", "pip3 config unset global.index-url"])
            .output()
            .expect("failed to execute");
        println!("{:?}", output);
    }
}

fn composer(action: i32) {
    if action == 1 {
        let output = Command::new("sh")
            .args([
                "-c",
                "composer config -g repo.packagist composer https://mirrors.aliyun.com/composer/",
            ])
            .output()
            .expect("failed to execute");
        println!("{:?}", output);
    } else if action == 2 {
        let output = Command::new("sh")
            .args(["-c", "composer config -g --unset repos.packagist"])
            .output()
            .expect("failed to execute");
        println!("{:?}", output);
    }
}

fn gem(action: i32) {
    if action == 1 {
        let output = Command::new("sh")
            .args([
                "-c",
                "gem sources --add https://mirrors.tuna.tsinghua.edu.cn/rubygems/ --remove https://rubygems.org/",
            ])
            .output()
            .expect("failed to execute");
        println!("{:?}", output);
    } else if action == 2 {
        let output = Command::new("sh")
        .args([
            "-c",
            "gem sources --add https://rubygems.org/ --remove https://mirrors.tuna.tsinghua.edu.cn/rubygems/",
        ])
        .output()
        .expect("failed to execute");
        println!("{:?}", output);
    }
}

fn maven(action: i32) {
    println!("WIP")
}

fn gradle(action: i32) {
    println!("WIP")
}

fn cargo(action: i32) {
    println!("WIP")
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Commands::Set { name } => match name.as_str() {
            "all" => all(1),
            // node
            "npm" => npm(1),
            "pnpm" => pnpm(1),
            "yarn" => yarn(1),
            "node" => npm(1),
            // go
            "go" => go(1),
            // python
            "pip" => pip(1),
            "pip3" => pip3(1),
            "python" => pip(1),
            // php
            "composer" => composer(1),
            "php" => composer(1),
            // ruby
            "gem" => gem(1),
            "ruby" => gem(1),
            "gems" => gem(1),
            "rubygems" => gem(1),
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
            "npm" => npm(2),
            "pnpm" => pnpm(2),
            "yarn" => yarn(2),
            "node" => npm(2),
            // go
            "go" => go(2),
            // python
            "pip" => pip(2),
            "python" => pip(2),
            // php
            "composer" => composer(2),
            "php" => composer(2),
            // ruby
            "gem" => gem(2),
            "ruby" => gem(2),
            "gems" => gem(2),
            "rubygems" => gem(2),
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
