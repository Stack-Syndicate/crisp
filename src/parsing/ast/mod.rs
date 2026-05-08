use colored::Colorize;
use log::error;
use pest::iterators::Pair;

use crate::parsing::{
    Rule,
    ast::nodes::{Expr, SourceInfo},
};

pub(crate) mod nodes;
pub(crate) mod validation;

pub fn cst_to_ast(pair: Pair<Rule>) -> Expr {
    Expr::from_pair(pair)
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
