use pest_derive::Parser;

pub mod ast;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CrispParser;
