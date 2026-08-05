use chumsky::Parser as ChumskyParser;
use clap::builder::styling::{AnsiColor, Effects, Styles};
use clap::{Parser, Subcommand};
use colored::Colorize;
use crisp::parsing::error::print_parse_errors;
use std::fs;
use std::path::PathBuf;

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
    Repl,
    Parse {
        #[arg(value_name = "FILE")]
        file: PathBuf,
    },
}

fn main() {
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
                let crisp_parsed = crisp::parsing::crip_parser()
                    .parse(&source)
                    .into_output_errors();
                if !crisp_parsed.1.is_empty() {
                    print_parse_errors(&source, &crisp_parsed.1);
                } else {
                    println!("{}", "No errors detected".green());
                }
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
    }
}
