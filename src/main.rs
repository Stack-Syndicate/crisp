use anyhow::Error;
use chumsky::Parser as ChumskyParser;
use clap::{
    Parser, Subcommand,
    builder::styling::{AnsiColor, Effects, Styles},
};
use colored::Colorize;
use crisp::{
    logging::setup_logging,
    parsing::error::print_parse_errors,
    project::{check_project, init_project, new_project},
};
use std::{fs, path::PathBuf};

fn help_styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default() | Effects::BOLD)
        .usage(AnsiColor::Green.on_default() | Effects::BOLD)
        .literal(AnsiColor::Cyan.on_default() | Effects::BOLD)
        .placeholder(AnsiColor::Yellow.on_default())
}

#[derive(Parser)]
#[command(
    name = "crisp",
    styles = help_styles(),
    color = clap::ColorChoice::Auto
)]
#[command(about = "A small, expressive Lisp-inspired programming language", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the REPL
    Repl,
    /// Parse a single Crisp file
    Parse {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
    /// Create a new project in an existing directory
    Init {
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },
    /// Create a new project
    New {
        #[arg(value_name = "PATH")]
        path: PathBuf,
    },
    /// Build project in current directory
    Build,
}

fn main() -> Result<(), Error> {
    let multi_progress = setup_logging();
    let cli = Cli::parse();
    match cli.command {
        Commands::Repl => {
            println!("{} {}", "Starting".cyan(), "Crisp REPL".yellow());
        }
        Commands::Parse { file } => match fs::read_to_string(&file) {
            Ok(source) => {
                println!(
                    "{} {}",
                    "Loaded file:".cyan(),
                    file.display().to_string().yellow()
                );
                let crisp_parsed = crisp::parsing::crisp_parser()
                    .parse(&source)
                    .into_output_errors();
                if !crisp_parsed.1.is_empty() {
                    print_parse_errors(&source, &crisp_parsed.1[0..1]);
                } else {
                    println!("{}", "No errors detected".green());
                }
                println!("{:#?}", crisp_parsed.0);
            }
            Err(err) => {
                eprintln!(
                    "{} Failed to read '{}': {}",
                    "Error:".red().bold(),
                    file.display().to_string().yellow(),
                    err
                );
                std::process::exit(1);
            }
        },
        Commands::Init { path } => {
            init_project(&path, multi_progress)?;
        }
        Commands::New { path } => {
            new_project(&path, multi_progress)?;
        }
        Commands::Build => {
            let current_path = std::env::current_dir().expect("Cannot get current directory");
            check_project(&current_path, multi_progress)?;
        }
    }
    Ok(())
}
