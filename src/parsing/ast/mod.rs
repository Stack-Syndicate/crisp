use pest::Parser;

use log::error;
use pest::iterators::Pair;
use pest_derive::Parser;

use crate::{
    diagnostics::print_pest_error,
    parsing::{SourceFile, ast::nodes::Node},
};

pub mod nodes;
pub mod validation;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CrispParser;

pub fn cst_to_ast<'a>(pair: Pair<'a, Rule>, source: &SourceFile) -> Node {
    Node::from_pair(pair, source)
}

pub fn parse_file(source: SourceFile) -> Result<Node, String> {
    parse_str(source)
}

pub fn parse_str(source: SourceFile) -> Result<Node, String> {
    if source.source.is_empty() {
        error!("Source file is empty!");
        return Err("Source file is empty".to_string());
    }
    let pest_cst = CrispParser::parse(Rule::file, &source.source);

    match pest_cst {
        Ok(mut pairs) => Ok(cst_to_ast(pairs.next().unwrap(), &source)),
        Err(e) => {
            print_pest_error(e, &source.path, &source.source);
            Err("Parse failed; see logs for details.".to_string())
        }
    }
}
