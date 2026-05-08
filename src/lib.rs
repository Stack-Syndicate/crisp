use std::{fs::File, io::Read};

use anyhow::Result;
use log::{debug, error};
use pest::Parser;

use crate::parsing::{
    CrispParser, Rule,
    ast::{cst_to_ast, nodes::Node},
};

pub mod cli;
pub mod parsing;

pub fn parse_file<'a>(path: &'static str) -> Result<Node<'a>, String> {
    let mut source = "".to_string();
    File::open(&path)
        .unwrap()
        .read_to_string(&mut source)
        .expect("Could not read the file as source code");
    return parse_str(source, path);
}

pub fn parse_str<'a>(source: String, path: &'static str) -> Result<Node<'a>, String> {
    if source.is_empty() {
        error!("Source file is empty!");
        return Err("Source file is empty".to_string());
    }
    let pest_cst = CrispParser::parse(Rule::file, &source);
    let ast = match pest_cst {
        Ok(mut pairs) => {
            debug!("Constructing AST");
            cst_to_ast(pairs.next().unwrap(), path)
        }
        Err(e) => return Err(format!("Parse failed: {}", e)),
    };
    return Ok(Node::Invalid);
}
