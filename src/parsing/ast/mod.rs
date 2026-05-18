use pest::iterators::Pair;

use crate::parsing::{Rule, ast::nodes::Node};

pub mod nodes;
pub mod validation;

pub fn cst_to_ast<'a>(pair: Pair<'a, Rule>, path: &'static str) -> Node {
    Node::from_pair(pair, path)
}
