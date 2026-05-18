use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;
use clap::builder::Styles;
use clap::builder::styling::AnsiColor;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "A minimal CRISP-to-C transpiler",
    arg_required_else_help = true,
    styles = styles()
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Runs the CRISP-to-C transpiler.
    T {
        #[arg(value_name = "FILE", value_parser = clap::value_parser!(std::path::PathBuf))]
        input: PathBuf,
    },
}

fn styles() -> Styles {
    Styles::styled()
        .header(AnsiColor::Green.on_default().bold())
        .usage(AnsiColor::Cyan.on_default().bold())
        .literal(AnsiColor::Yellow.on_default())
        .placeholder(AnsiColor::BrightBlack.on_default())
        .error(AnsiColor::Red.on_default().bold())
}
