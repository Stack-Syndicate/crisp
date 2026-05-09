use std::collections::HashSet;

use colored::Colorize;
use log::error;
use pest::iterators::Pair;

use crate::parsing::{Rule, ast::nodes::SourceInfo};

pub fn validate_fn(pair: &Pair<Rule>, path: &'static str) -> bool {
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    let len = pairs.len();
    if len != 3 && len != 4 {
        print_ast_error(
            "Function definition is missing components; expected (fn [name] (params) (body))",
            &SourceInfo::from_pair(pair, path),
        );
        return false;
    }
    let is_named = len > 1 && matches!(pairs[1].as_rule(), Rule::symbol);
    if is_named {
        if len != 4 {
            print_ast_error(
                "Named function must have 4 parts: (fn name (params) (body))",
                &SourceInfo::from_pair(pair, path),
            );
            return false;
        }
    } else {
        if len != 3 {
            print_ast_error(
                "Anonymous function must have 3 parts: (fn (params) (body))",
                &SourceInfo::from_pair(pair, path),
            );
            return false;
        }
    }
    let name_idx = 1;
    let params_idx = if is_named { 2 } else { 1 };
    let body_idx = if is_named { 3 } else { 2 };
    if is_named && !matches!(pairs[name_idx].as_rule(), Rule::symbol) {
        print_ast_error(
            "Function name must be a symbol",
            &SourceInfo::from_pair(&pairs[name_idx], path),
        );
        return false;
    }
    let params_pair = &pairs[params_idx];
    if !matches!(params_pair.as_rule(), Rule::list) {
        print_ast_error(
            "Parameters must be enclosed in a list: (x:type ...)",
            &SourceInfo::from_pair(params_pair, path),
        );
        return false;
    }
    for pair in params_pair.clone().into_inner() {
        if !matches!(pair.as_rule(), Rule::typed_symbol) {
            print_ast_error(
                "Every parameter must have a type annotation (e.g., name:type)",
                &SourceInfo::from_pair(&pair, path),
            );
            return false;
        }
    }
    let body_pair = &pairs[body_idx];
    if !matches!(body_pair.as_rule(), Rule::list) {
        print_ast_error(
            "Function body must be a list of expressions",
            &SourceInfo::from_pair(body_pair, path),
        );
        return false;
    }
    true
}
pub fn validate_if(pair: &Pair<Rule>, path: &'static str) -> bool {
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    // if must be 3 or 4 elements
    if pairs.len() != 3 && pairs.len() != 4 {
        println!("{}", pairs.len());
        print_ast_error("Invalid if statement", &SourceInfo::from_pair(pair, path));
        return false;
    }
    // predicate must be list, boolean or symbol
    if !matches!(
        pairs[1].as_rule(),
        Rule::list | Rule::boolean | Rule::symbol
    ) {
        print_ast_error(
            "Predicate must be a list",
            &SourceInfo::from_pair(&pairs[1], path),
        );
        return false;
    }
    // then block must be list
    if !matches!(pairs[2].as_rule(), Rule::list) {
        print_ast_error(
            "Then block must be a list",
            &SourceInfo::from_pair(&pairs[2], path),
        );
        return false;
    }
    if pairs.len() == 4 {
        if !matches!(pairs[3].as_rule(), Rule::list) {
            print_ast_error(
                "Invalid else block",
                &SourceInfo::from_pair(&pairs[3], path),
            );
            return false;
        }
    }
    true
}

pub fn validate_for(pair: &Pair<Rule>, path: &'static str) -> bool {
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    // for must be 4 elements
    if pairs.len() != 4 {
        print_ast_error("Invalid for loop", &SourceInfo::from_pair(pair, path));
        return false;
    }
    // dummy must be symbol or typed symbol
    if !matches!(pairs[1].as_rule(), Rule::symbol | Rule::typed_symbol) {
        print_ast_error(
            "Dummy index is not a symbol",
            &SourceInfo::from_pair(&pairs[1], path),
        );
        return false;
    }
    // iterator must be list
    if !matches!(pairs[2].as_rule(), Rule::list | Rule::symbol) {
        print_ast_error(
            "Iterator is invalid",
            &SourceInfo::from_pair(&pairs[2], path),
        );
        return false;
    }
    // body must be a block
    if !matches!(pairs[3].as_rule(), Rule::list) {
        print_ast_error(
            "Body is not a block",
            &SourceInfo::from_pair(&pairs[3], path),
        );
        return false;
    }
    true
}

pub fn validate_let(pair: &Pair<Rule>, path: &'static str) -> bool {
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    // let must be 3 elements
    if pairs.len() != 3 {
        print_ast_error("Invalid let statement", &SourceInfo::from_pair(pair, path));
        return false;
    }
    // variable name must be symbol or typed symbol
    if !matches!(pairs[1].as_rule(), Rule::symbol | Rule::typed_symbol) {
        print_ast_error(
            "Variable name is not a symbol or type annotated symbol",
            &SourceInfo::from_pair(&pairs[1], path),
        );
        return false;
    }
    // value must be literal, symbol or list
    if !matches!(
        pairs[2].as_rule(),
        Rule::number | Rule::string | Rule::boolean | Rule::symbol | Rule::list
    ) {
        print_ast_error(
            "Value is not a literal, untyped symbol or list",
            &SourceInfo::from_pair(&pairs[2], path),
        );
        return false;
    }
    true
}

pub fn validate_given(pair: &Pair<Rule>, path: &'static str) -> bool {
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    if pairs.len() < 2 {
        print_ast_error(
            "Given statement is missing a predicate",
            &SourceInfo::from_pair(pair, path),
        );
        return false;
    }
    let predicate = &pairs[1];
    match predicate.as_rule() {
        Rule::list | Rule::symbol | Rule::boolean | Rule::number => {}
        _ => {
            print_ast_error(
                "Predicate must be an expression (atom or list)",
                &SourceInfo::from_pair(predicate, path),
            );
            return false;
        }
    }
    for case in &pairs[2..] {
        if !matches!(case.as_rule(), Rule::list) {
            print_ast_error(
                "Each case in a given statement must be a list: (pattern (body))",
                &SourceInfo::from_pair(case, path),
            );
            return false;
        }
        if case.clone().into_inner().count() != 2 {
            print_ast_error(
                "Each case must have exactly a pattern and a body",
                &SourceInfo::from_pair(case, path),
            );
            return false;
        }
    }
    true
}

pub fn validate_ret(pair: &Pair<Rule>, path: &'static str) -> bool {
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    // ret must be 2 elements
    if pairs.len() != 2 {
        print_ast_error("Invalid return call", &SourceInfo::from_pair(pair, path));
        return false;
    }
    // value must be symbol, literal or list
    if !matches!(
        pairs[1].as_rule(),
        Rule::string | Rule::number | Rule::boolean | Rule::list | Rule::symbol
    ) {
        print_ast_error(
            "Invalid return call value",
            &SourceInfo::from_pair(pair, path),
        );
        return false;
    }
    true
}

pub fn validate_call(pair: &Pair<Rule>, path: &'static str) -> bool {
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    // call must be 1 or more elements
    if pairs.len() < 1 {
        print_ast_error("Invalid call", &SourceInfo::from_pair(pair, path));
        return false;
    }
    // first element should be a symbol
    if !matches!(pairs[0].as_rule(), Rule::symbol) {
        print_ast_error(
            "Invalid call identifier",
            &SourceInfo::from_pair(&pairs[0], path),
        );
        return false;
    }
    // every other element should be a symbol, list or literal
    for pair in pairs[1..].iter() {
        if !matches!(
            pair.as_rule(),
            Rule::symbol | Rule::list | Rule::boolean | Rule::string | Rule::number
        ) {
            print_ast_error("Invalid call argument", &SourceInfo::from_pair(pair, path));
            return false;
        }
    }
    true
}

pub fn validate_block(pair: &Pair<Rule>, path: &'static str) -> bool {
    let inner = pair.clone().into_inner();
    if inner.len() == 0 {
        print_ast_error(
            "Empty blocks are not allowed",
            &SourceInfo::from_pair(pair, path),
        );
        return false;
    }
    for pair in inner {
        match pair.as_rule() {
            Rule::list | Rule::symbol | Rule::number | Rule::string => continue,
            _ => {
                print_ast_error(
                    "Invalid expression inside block",
                    &SourceInfo::from_pair(&pair, path),
                );
                return false;
            }
        }
    }
    true
}

pub fn validate_params(pair: &Pair<Rule>, path: &'static str) -> bool {
    let inner = pair.clone().into_inner();
    let mut names = HashSet::new();
    for param in inner {
        let name = match param.as_rule() {
            Rule::typed_symbol => param.clone().into_inner().next().unwrap().as_str(),
            Rule::symbol => param.as_str(),
            _ => {
                print_ast_error(
                    "Parameter must be a symbol",
                    &SourceInfo::from_pair(&param, path),
                );
                return false;
            }
        };
        if !names.insert(name) {
            print_ast_error(
                &format!("Duplicate parameter name: {}", name),
                &SourceInfo::from_pair(&param, path),
            );
            return false;
        }
    }
    true
}

pub fn print_ast_error(msg: &str, info: &SourceInfo) {
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
