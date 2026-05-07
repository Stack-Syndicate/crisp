use std::path::PathBuf;

use clap::Parser;
use clap::Subcommand;

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "A minimal Lisp-to-C transpiler",
    arg_required_else_help = true
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
