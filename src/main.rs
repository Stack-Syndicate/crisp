#![feature(path_absolute_method)]
use std::{fs::File, io::Read};

use anyhow::Result;
use clap::Parser as CLIParser;
use crisp::{
    cli::{Args, Command},
    parse_file,
    parsing::{
        CrispParser, Rule,
        ast::{cst_to_ast, nodes::Node},
    },
};
use log::{debug, error, info, warn};
use pest::Parser;

fn main() {
    // Parse CLI args and set up logging env
    let args = Args::parse();
    let log_level = match args.verbose {
        0 => log::LevelFilter::Info,
        1 => log::LevelFilter::Debug,
        _ => log::LevelFilter::Trace,
    };
    env_logger::Builder::new()
        .filter_level(log_level)
        .format_timestamp(None)
        .write_style(env_logger::WriteStyle::Always)
        .init();
    info!("Log level: {}", log_level.to_string().to_uppercase());
    std::panic::set_hook(Box::new(|info| {
        log::error!("Exiting because: {info}");
    }));
    let cmd = args.command;
    let path: &'static str;
    match cmd {
        Command::T { input } => {
            debug!("Opening file: {:?}", input);
            // Check if the file exists
            if !input.exists() {
                error!("File {:?} not found, exiting", input);
                return;
            } else {
                let path_str = input.absolute().unwrap().to_string_lossy().into_owned();
                path = Box::leak(path_str.into_boxed_str());
            }
        }
    }

    debug!("Parsing input");
    let parse_result = parse_file(path);
}
