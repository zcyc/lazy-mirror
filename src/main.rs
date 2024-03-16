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

fn all(name: &String, commands: &Commands) {
    match commands {
        Commands::Set { name: _ } => {
            lm::go::set();
            lm::python::set(name);
            lm::node::set(name);
            lm::php::set();
            lm::java::maven_set();
            lm::java::gradle_set();
            lm::ruby::gem_set();
            lm::ruby::bundle_set();
            lm::rust::set();
            // lm::brew::set();
        }
        Commands::Unset { name: _ } => {
            lm::go::unset();
            lm::python::unset(name);
            lm::node::unset(name);
            lm::php::unset();
            lm::java::maven_unset();
            lm::java::gradle_unset();
            lm::ruby::gem_unset();
            lm::ruby::bundle_unset();
            lm::rust::unset();
            // lm::brew::unset();
        }
    };
}

fn node(name: &String, commands: &Commands) {
    match commands {
        Commands::Set { name: _ } => {
            lm::node::set(name);
        }
        Commands::Unset { name: _ } => {
            lm::node::unset(name);
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

fn python(name: &String, commands: &Commands) {
    match commands {
        Commands::Set { name: _ } => {
            lm::python::set(name);
        }
        Commands::Unset { name: _ } => {
            lm::python::unset(name);
        }
    };
}

fn php(commands: &Commands) {
    match commands {
        Commands::Set { name: _ } => {
            lm::php::set();
        }
        Commands::Unset { name: _ } => {
            lm::php::unset();
        }
    };
}

fn ruby(commands: &Commands) {
    match commands {
        Commands::Set { name: _ } => {
            lm::ruby::set();
        }
        Commands::Unset { name: _ } => {
            lm::ruby::unset();
        }
    };
}

fn java(name: &str, commands: &Commands) {
    match name {
        "maven" => {
            match commands {
                Commands::Set { name: _ } => {
                    lm::java::maven_set();
                }
                Commands::Unset { name: _ } => {
                    lm::java::maven_unset();
                }
            };
        }
        "gradle" => {
            match commands {
                Commands::Set { name: _ } => {
                    lm::java::gradle_set();
                }
                Commands::Unset { name: _ } => {
                    lm::java::gradle_unset();
                }
            };
        }
        _ => {}
    }
    match commands {
        Commands::Set { name: _ } => {
            lm::java::maven_set();
        }
        Commands::Unset { name: _ } => {
            lm::java::maven_unset();
        }
    };
}

fn rust(commands: &Commands) {
    match commands {
        Commands::Set { name: _ } => {
            lm::rust::set();
        }
        Commands::Unset { name: _ } => {
            lm::rust::unset();
        }
    };
}

fn brew(commands: &Commands) {
    match commands {
        Commands::Set { name: _ } => {
            lm::brew::set();
        }
        Commands::Unset { name: _ } => {
            lm::brew::unset();
        }
    };
}

fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Set { name } => match name.as_str() {
            "all" => all(name, &args.command),
            // node
            "npm" => node(name, &args.command),
            "pnpm" => node(name, &args.command),
            "yarn" => node(name, &args.command),
            "node" => node(&String::from("npm"), &args.command),
            // go
            "go" => go(&args.command),
            // python
            "pip" => python(name, &args.command),
            "pip3" => python(name, &args.command),
            "python" => python(&String::from("pip3"), &args.command),
            // php
            "composer" => php(&args.command),
            "php" => php(&args.command),
            // ruby
            "gem" => ruby(&args.command),
            "ruby" => ruby(&args.command),
            "gems" => ruby(&args.command),
            "rubygems" => ruby(&args.command),
            // java
            "maven" => java(name, &args.command),
            "gradle" => java(name, &args.command),
            "java" => java(&String::from("maven"), &args.command),
            // rust
            "cargo" => rust(&args.command),
            "rust" => rust(&args.command),
            // other
            _ => println!("not support it"),
        },
        Commands::Unset { name } => match name.as_str() {
            "all" => all(name, &args.command),
            // node
            "npm" => node(name, &args.command),
            "pnpm" => node(name, &args.command),
            "yarn" => node(name, &args.command),
            "node" => node(&String::from("npm"), &args.command),
            // go
            "go" => go(&args.command),
            // python
            "pip" => python(name, &args.command),
            "pip3" => python(name, &args.command),
            "python" => python(&String::from("pip3"), &args.command),
            // php
            "composer" => php(&args.command),
            "php" => php(&args.command),
            // ruby
            "gem" => ruby(&args.command),
            "ruby" => ruby(&args.command),
            "gems" => ruby(&args.command),
            "rubygems" => ruby(&args.command),
            // java
            "maven" => java(name, &args.command),
            "gradle" => java(name, &args.command),
            "java" => java(&String::from("maven"), &args.command),
            // rust
            "cargo" => rust(&args.command),
            "rust" => rust(&args.command),
            // brew
            "brew" => brew(&args.command),
            // other
            _ => println!("not support it"),
        },
    }
}
