use pest_derive::Parser;

pub mod ast;
pub mod types;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CrispParser;
