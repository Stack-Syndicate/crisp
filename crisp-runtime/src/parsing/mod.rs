use pest::{iterators::{Pair, Pairs}, Parser};
use pest_derive::Parser;
#[derive(Parser)]
#[grammar = "parsing/grammar.pest"]
pub struct CrispParser;