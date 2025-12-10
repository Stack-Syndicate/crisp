use pest::{Parser, error::Error, iterators::{Pairs}};
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CrispParser;

pub fn parse(src: &str) -> anyhow::Result<Pairs<'_, Rule>, Error<Rule>> {
    let pairs = CrispParser::parse(Rule::program, src)?;
    Ok(pairs)
}

fn main() {
    println!("Hello, world!");
}
