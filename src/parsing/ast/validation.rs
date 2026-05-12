use std::collections::HashSet;

use colored::Colorize;
use log::error;
use pest::iterators::Pair;

use crate::parsing::{
    Rule,
    ast::nodes::{SourceInfo, Symbol},
};

pub fn validate_fn(pair: &Pair<Rule>, path: &'static str) -> bool {
    let mut inner = pair.clone().into_inner();
    let fn_op = inner.next().unwrap();
    // function definition has to start with fn:type
    if !matches!(fn_op.as_rule(), Rule::symbol) {
        print_ast_error(
            "Function definition must start with a typed fn symbol",
            &SourceInfo::from_pair(&fn_op, path),
        );
        return false;
    }
    // peak ahead at the second part
    let second = match inner.next() {
        Some(pair) => pair,
        None => {
            print_ast_error(
                "Function definition must be of the shape (fn:type name (param:type...) (body))",
                &SourceInfo::from_pair(pair, path),
            );
            return false;
        }
    };
    // second part might be a name or a parameter list
    let params;
    match second.as_rule() {
        Rule::symbol => {
            params = match inner.next() {
                Some(pair) => pair,
                None => {
                    print_ast_error(
                        "Function definition must be of the shape (fn:type name (param:type...) (body))",
                        &SourceInfo::from_pair(pair, path),
                    );
                    return false;
                }
            };
            match inner.next() {
                Some(pair) => {
                    if !matches!(pair.as_rule(), Rule::list) {
                        print_ast_error("Body must be a list", &SourceInfo::from_pair(&pair, path));
                        return false;
                    }
                }
                None => {
                    print_ast_error(
                        "Function definition must be of the shape (fn:type name (param:type...) (body))",
                        &SourceInfo::from_pair(pair, path),
                    );
                    return false;
                }
            };
        }
        Rule::list => {
            params = second;
            match inner.next() {
                Some(pair) => {
                    if !matches!(pair.as_rule(), Rule::list) {
                        print_ast_error("Body must be a list", &SourceInfo::from_pair(&pair, path));
                        return false;
                    }
                }
                None => {
                    print_ast_error(
                        "Function definition must include a body which is a list",
                        &SourceInfo::from_pair(pair, path),
                    );
                    return false;
                }
            };
        }
        _ => {
            print_ast_error(
                "Unexpected function structure",
                &SourceInfo::from_pair(&second, path),
            );
            return false;
        }
    }
    if inner.next().is_some() {
        print_ast_error(
            "Too many components in function definition",
            &SourceInfo::from_pair(pair, path),
        );
        return false;
    }
    // check if params is actually a list
    if !matches!(params.as_rule(), Rule::list) {
        print_ast_error(
            "Parameters must be a list",
            &SourceInfo::from_pair(&params, path),
        );
        return false;
    }
    // do a quick check to see if param symbols are all typed
    let mut is_params_valid = true;
    for pair in params.into_inner() {
        if !matches!(pair.as_rule(), Rule::symbol) {
            print_ast_error(
                "Parameter must be a symbol",
                &SourceInfo::from_pair(&pair, path),
            );
            is_params_valid &= false;
            continue;
        }
        let param_symbol = Symbol::from_pair(&pair);
        if let Symbol::Untyped { name: _ } = param_symbol {
            print_ast_error(
                "Parameter must be typed",
                &SourceInfo::from_pair(&pair, path),
            );
            is_params_valid &= false;
        }
    }
    if !is_params_valid {
        return false;
    }
    true
}

pub fn validate_if(pair: &Pair<Rule>, path: &'static str) -> bool {
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    // if must be a list in the first place
    if !matches!(pair.as_rule(), Rule::list) {
        print_ast_error(
            "If statement must be a list",
            &SourceInfo::from_pair(&pair, path),
        );
        return false;
    }
    // if must be 3 or 4 elements
    if pairs.len() != 3 && pairs.len() != 4 {
        println!("{}", pairs.len());
        print_ast_error("Invalid if statement", &SourceInfo::from_pair(pair, path));
        return false;
    }
    // keyword is a symbol and strictly "if"
    let head = &pairs[0];
    if head.as_rule() != Rule::symbol || head.as_str() != "if" {
        print_ast_error(
            "If statement must start with the \"if\" keyword",
            &SourceInfo::from_pair(head, path),
        );
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
    if pairs.len() != 4 {
        print_ast_error("Invalid for loop", &SourceInfo::from_pair(pair, path));
        return false;
    }
    let for_op = &pairs[0];
    if !matches!(for_op.as_rule(), Rule::symbol) {
        print_ast_error(
            "For loop must start with a for symbol",
            &SourceInfo::from_pair(&for_op, path),
        );
        return false;
    }
    if for_op.as_str() != "for" {
        print_ast_error(
            "For loop must start with a for symbol",
            &SourceInfo::from_pair(&for_op, path),
        );
        return false;
    }
    if !matches!(pairs[1].as_rule(), Rule::symbol) {
        print_ast_error(
            "Dummy index is not a symbol",
            &SourceInfo::from_pair(&pairs[1], path),
        );
        return false;
    }
    if !matches!(
        pairs[2].as_rule(),
        Rule::list | Rule::symbol | Rule::boolean
    ) {
        print_ast_error(
            "Iterator is invalid",
            &SourceInfo::from_pair(&pairs[2], path),
        );
        return false;
    }
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
    if !matches!(pairs[1].as_rule(), Rule::symbol) {
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

pub fn validate_list(pair: &Pair<Rule>, path: &'static str) -> bool {
    let span = pair.as_span();
    let content = span.as_str().trim();
    if !content.starts_with('(') || !content.ends_with(')') {
        print_ast_error(
            "Missing surrounding parentheses",
            &SourceInfo::from_pair(pair, path),
        );
        return false;
    }
    for inner_pair in pair.clone().into_inner() {
        match inner_pair.as_rule() {
            Rule::symbol => {
                let _ = Symbol::from_pair(&inner_pair);
            }
            Rule::list => {
                if !validate_list(&inner_pair, path) {
                    return false;
                }
            }
            Rule::EOI => {}
            _ => {
                print_ast_error(
                    "Unexpected token in list",
                    &SourceInfo::from_pair(&inner_pair, path),
                );
                return false;
            }
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
