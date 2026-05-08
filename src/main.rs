#![feature(path_absolute_method)]
use std::{fs::File, io::Read};

use crate::{
    cli::{Args, Command},
    parsing::{
        CrispParser, Rule,
        ast::{cst_to_ast, nodes::Expr},
    },
};
use clap::Parser as CLIParser;
use log::{debug, error, info, warn};
use pest::Parser;

pub(crate) mod cli;
pub(crate) mod parsing;

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
    let mut source = "".to_string();
    let path: &'static str;
    match cmd {
        Command::T { input } => {
            debug!("Opening file: {:?}", input);
            // Check if the file exists
            if !input.exists() {
                error!("File {:?} not found, exiting", input);
                return;
            } else {
                File::open(&input)
                    .unwrap()
                    .read_to_string(&mut source)
                    .expect("Could not read the file as source code");
                let path_str = input.absolute().unwrap().to_string_lossy().into_owned();
                path = Box::leak(path_str.into_boxed_str());
            }
        }
    }
    if source.is_empty() {
        error!("Source file is empty!");
        return;
    }
    debug!("Parsing input");
    let pest_parse = CrispParser::parse(Rule::file, &source);
    let ast: Expr;
    match pest_parse {
        Ok(mut pairs) => {
            debug!("Constructing AST");
            ast = cst_to_ast(pairs.next().unwrap());
        }
        Err(e) => {
            error!("Parse failed: {}", e)
        }
    }
}
