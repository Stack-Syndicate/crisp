use colored::Colorize;
use log::error;
use pest::iterators::Pair;

use crate::parsing::{
    Rule,
    ast::nodes::{Node, SourceInfo},
};

pub mod nodes;
pub mod validation;

pub fn cst_to_ast<'a>(pair: Pair<'a, Rule>, path: &'static str) -> Node<'a> {
    Node::from_pair(pair, path)
}

pub fn print_error(msg: &str, info: &SourceInfo) {
    let span = info.span;
    let input = span.get_input();
    let start = span.start();

    let line_start = input[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = input[start..]
        .find('\n')
        .map(|i| start + i)
        .unwrap_or(input.len());
    let line_text = &input[line_start..line_end];

    let indent = " ".repeat(info.col - 1);
    let span_len = (span.end() - span.start()).max(1);
    let pointer = "~".repeat(span_len).red();

    error!(
        "{}\n--> {}[{}|{}]\n{:>4} |\n{:>4} | {}\n     | {}{}",
        msg.bold(),
        info.path.blue(),
        info.line.to_string().red(),
        info.col.to_string().red(),
        "|",
        info.line.to_string().red(),
        line_text,
        indent,
        pointer
    );
}
