use colored::Colorize;
use log::{debug, error};

use crate::{
    analysis::{
        scope::build_scope_tree,
        structure::{validate_annotations, validate_fn_definitions},
    },
    parsing::ast::{Node, SourceInfo},
};

pub(crate) mod scope;
pub(crate) mod structure;

pub fn analyze_ast(ast: Vec<Node>) {
    debug!("Validating type annotations");
    let valid_annotations = validate_annotations(&ast);
    debug!("Validating fn definitions");
    let valid_fns = validate_fn_definitions(&ast);
    debug!("Building scope tree");
    let scope_tree = build_scope_tree(&ast);
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
        "{}\n  --> {}[{}|{}]\n{:>4} |\n{:>4} | {}\n     | {}{}",
        msg.bold(),
        info.path.blue(), // Blue makes it look like a link
        info.line.to_string().red(),
        info.col.to_string().red(),
        "|",
        info.line.to_string().red(),
        line_text,
        indent,
        pointer
    );
}

pub fn print_debug(msg: &str, extra: &str, info: &SourceInfo) {
    debug!(
        "[{}:{}|{}:{}|-> {} {}",
        "L".to_string().blue(),
        info.line.to_string().blue(),
        "C".to_string().blue(),
        info.col.to_string().blue(),
        msg,
        extra.to_string().blue()
    )
}
