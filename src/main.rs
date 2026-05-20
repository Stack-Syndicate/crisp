#![feature(path_absolute_method)]

use std::fs::File;
use std::io::Read;

use clap::Parser as CLIParser;
use crisp::{
    cli::{Args, Command},
    parsing::{
        SourceFile,
        ast::parse_file as parse_file_to_ast,
        rir::{ScopeStack, ast_to_scopes},
    },
};
use log::{debug, error, info};

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
    let mut raw_source = String::new();
    File::open(path)
        .unwrap()
        .read_to_string(&mut raw_source)
        .expect("Could not read the file as source code");
    let source = SourceFile {
        path: path.to_string(),
        source: raw_source,
    };
    debug!("Parsing input");
    let parse_result = parse_file_to_ast(source).expect("Pest to AST Parsing failed.");
    println!("{:?}", parse_result);
    debug!("Doing scope pass");
    let mut scope_stack = ScopeStack::new();
    let _scopes_are_good = ast_to_scopes(&parse_result, &mut scope_stack);
}
