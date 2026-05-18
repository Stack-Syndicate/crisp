use pest::Parser;
use std::{fs::File, io::Read};

use log::error;
use pest::iterators::Pair;
use pest_derive::Parser;

use crate::{diagnostics::print_pest_error, parsing::ast::nodes::Node};

pub mod nodes;
pub mod validation;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CrispParser;

pub fn cst_to_ast<'a>(pair: Pair<'a, Rule>, path: &'static str) -> Node {
    Node::from_pair(pair, path)
}

pub fn parse_file(path: &'static str) -> Result<Node, String> {
    let mut source = "".to_string();
    File::open(path)
        .unwrap()
        .read_to_string(&mut source)
        .expect("Could not read the file as source code");
    parse_str(source, path)
}

pub fn parse_str(source: String, path: &'static str) -> Result<Node, String> {
    if source.is_empty() {
        error!("Source file is empty!");
        return Err("Source file is empty".to_string());
    }
    let pest_cst = CrispParser::parse(Rule::file, &source);

    match pest_cst {
        Ok(mut pairs) => Ok(cst_to_ast(pairs.next().unwrap(), path)),
        Err(e) => {
            print_pest_error(e, path, &source);
            Err("Parse failed; see logs for details.".to_string())
        }
    }
}
