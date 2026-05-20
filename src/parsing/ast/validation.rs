use crate::{diagnostics::print_error, parsing::SourceFile};
use pest::iterators::Pair;
use std::collections::HashSet;

use crate::parsing::ast::{
    Rule,
    nodes::{SourceInfo, Symbol},
};

pub fn validate_fn(pair: &Pair<Rule>, source: &SourceFile) -> bool {
    let inner: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    // check if fn block has too many or too few parts
    if inner.len() != 3 && inner.len() != 4 {
        print_error(
            "Function definition has too many/too few parts",
            &SourceInfo::from_pair(pair, source),
        );
        return false;
    }
    // check if op name is 'fn'
    let op_name = &inner[0];
    if !op_name.as_str().starts_with("fn:") {
        print_error(
            "Function definition must start with a typed fn symbol",
            &SourceInfo::from_pair(op_name, source),
        );
        return false;
    }
    // is the fn definition anonymous or not?
    let is_anonymous = inner.len() == 3;
    let params = if is_anonymous { &inner[1] } else { &inner[2] };
    let body = if is_anonymous { &inner[2] } else { &inner[3] };
    // if not anonymous, is name a symbol?
    if !is_anonymous && !matches!(&inner[1].as_rule(), Rule::symbol) {
        print_error(
            "Fn name must be a valid symbol",
            &SourceInfo::from_pair(&inner[1], source),
        );
        return false;
    }
    // is params a list?
    if !matches!(params.as_rule(), Rule::list) {
        print_error(
            "Parameters must be a list",
            &SourceInfo::from_pair(params, source),
        );
        return false;
    } else {
        // is each param typed?
        for param in params.clone().into_inner() {
            if !matches!(param.as_rule(), Rule::symbol) {
                print_error(
                    "Parameter must be a direct typed symbol (e.g. x:i8), no nesting allowed",
                    &SourceInfo::from_pair(&param, source),
                );
                return false;
            }
            let type_symbol = Symbol::from_pair(&param);
            if let Symbol::Untyped { .. } = type_symbol {
                print_error(
                    "Parameter must be typed (e.g. x:i32)",
                    &SourceInfo::from_pair(&param, source),
                );
                return false;
            }
        }
    }
    // is body a list?
    if !matches!(body.as_rule(), Rule::list) {
        print_error("Body must be a list", &SourceInfo::from_pair(body, source));
        return false;
    }
    true
}

pub fn validate_if(pair: &Pair<Rule>, source: &SourceFile) -> bool {
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    // if must be a list in the first place
    if !matches!(pair.as_rule(), Rule::list) {
        print_error(
            "If statement must be a list",
            &SourceInfo::from_pair(pair, source),
        );
        return false;
    }
    // if must be 3 or 4 elements
    if pairs.len() != 3 && pairs.len() != 4 {
        println!("{}", pairs.len());
        print_error("Invalid if statement", &SourceInfo::from_pair(pair, source));
        return false;
    }
    // keyword is a symbol and strictly "if"
    let head = &pairs[0];
    if head.as_rule() != Rule::symbol || head.as_str() != "if" {
        print_error(
            "If statement must start with the \"if\" keyword",
            &SourceInfo::from_pair(head, source),
        );
        return false;
    }
    // predicate must be list, boolean or symbol
    if !matches!(
        pairs[1].as_rule(),
        Rule::list | Rule::boolean | Rule::symbol
    ) {
        print_error(
            "Predicate must be a list",
            &SourceInfo::from_pair(&pairs[1], source),
        );
        return false;
    }
    // then block must be list
    if !matches!(pairs[2].as_rule(), Rule::list | Rule::symbol) {
        print_error(
            "Then block must be a list or a symbol",
            &SourceInfo::from_pair(&pairs[2], source),
        );
        return false;
    }
    if pairs.len() == 4 && !matches!(pairs[3].as_rule(), Rule::list | Rule::symbol) {
        print_error(
            "Else block must be a list or a symbol",
            &SourceInfo::from_pair(&pairs[3], source),
        );
        return false;
    }
    true
}

pub fn validate_for(pair: &Pair<Rule>, source: &SourceFile) -> bool {
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    if pairs.len() != 4 {
        print_error("Invalid for loop", &SourceInfo::from_pair(pair, source));
        return false;
    }
    let for_op = &pairs[0];
    if !matches!(for_op.as_rule(), Rule::symbol) {
        print_error(
            "For loop must start with a for symbol",
            &SourceInfo::from_pair(for_op, source),
        );
        return false;
    }
    if for_op.as_str() != "for" {
        print_error(
            "For loop must start with a for symbol",
            &SourceInfo::from_pair(for_op, source),
        );
        return false;
    }
    if !matches!(pairs[1].as_rule(), Rule::symbol) {
        print_error(
            "Dummy index is not a symbol",
            &SourceInfo::from_pair(&pairs[1], source),
        );
        return false;
    }
    if !matches!(
        pairs[2].as_rule(),
        Rule::list | Rule::symbol | Rule::boolean
    ) {
        print_error(
            "Iterator is invalid",
            &SourceInfo::from_pair(&pairs[2], source),
        );
        return false;
    }
    if !matches!(pairs[3].as_rule(), Rule::list) {
        print_error(
            "Body is not a block",
            &SourceInfo::from_pair(&pairs[3], source),
        );
        return false;
    }
    true
}

pub fn validate_let(pair: &Pair<Rule>, source: &SourceFile) -> bool {
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    // let must be 3 elements
    if pairs.len() != 3 {
        print_error(
            "Invalid let statement",
            &SourceInfo::from_pair(pair, source),
        );
        return false;
    }
    // variable name must be symbol or typed symbol
    if !matches!(pairs[1].as_rule(), Rule::symbol) {
        print_error(
            "Variable name is not a symbol or type annotated symbol",
            &SourceInfo::from_pair(&pairs[1], source),
        );
        return false;
    }
    // value must be literal, symbol or list
    if !matches!(
        pairs[2].as_rule(),
        Rule::number | Rule::string | Rule::boolean | Rule::symbol | Rule::list
    ) {
        print_error(
            "Value is not a literal, untyped symbol or list",
            &SourceInfo::from_pair(&pairs[2], source),
        );
        return false;
    } else if matches!(pairs[2].as_rule(), Rule::list) {
        return validate_list(&pairs[2], source);
    }
    true
}

pub fn validate_given(pair: &Pair<Rule>, source: &SourceFile) -> bool {
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    if pairs.len() < 2 {
        print_error(
            "Given statement is missing a predicate",
            &SourceInfo::from_pair(pair, source),
        );
        return false;
    }
    let predicate = &pairs[1];
    match predicate.as_rule() {
        Rule::list | Rule::symbol | Rule::boolean | Rule::number => {}
        _ => {
            print_error(
                "Predicate must be an expression (atom or list)",
                &SourceInfo::from_pair(predicate, source),
            );
            return false;
        }
    }
    for case in &pairs[2..] {
        if !matches!(case.as_rule(), Rule::list) {
            print_error(
                "Each case in a given statement must be a list: (pattern (body))",
                &SourceInfo::from_pair(case, source),
            );
            return false;
        }
        if case.clone().into_inner().count() != 2 {
            print_error(
                "Each case must have exactly a pattern and a body",
                &SourceInfo::from_pair(case, source),
            );
            return false;
        }
    }
    true
}

pub fn validate_ret(pair: &Pair<Rule>, source: &SourceFile) -> bool {
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    // ret must be 2 elements
    if pairs.len() != 2 {
        print_error("Invalid return call", &SourceInfo::from_pair(pair, source));
        return false;
    }
    // value must be symbol, literal or list
    if !matches!(
        pairs[1].as_rule(),
        Rule::string | Rule::number | Rule::boolean | Rule::list | Rule::symbol
    ) {
        print_error(
            "Invalid return call value",
            &SourceInfo::from_pair(pair, source),
        );
        return false;
    }
    true
}

pub fn validate_call(pair: &Pair<Rule>, source: &SourceFile) -> bool {
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    // call must be 1 or more elements
    if pairs.is_empty() {
        print_error("Invalid call", &SourceInfo::from_pair(pair, source));
        return false;
    }
    // first element should be a symbol
    if !matches!(pairs[0].as_rule(), Rule::symbol) {
        print_error(
            "Invalid call identifier",
            &SourceInfo::from_pair(&pairs[0], source),
        );
        return false;
    }
    // every other element should be a symbol, list or literal
    for pair in pairs[1..].iter() {
        if !matches!(
            pair.as_rule(),
            Rule::symbol | Rule::list | Rule::boolean | Rule::string | Rule::number
        ) {
            print_error(
                "Invalid call argument",
                &SourceInfo::from_pair(pair, source),
            );
            return false;
        }
    }
    true
}

pub fn validate_block(pair: &Pair<Rule>, source: &SourceFile) -> bool {
    let inner = pair.clone().into_inner();
    if inner.is_empty() {
        print_error(
            "Empty blocks are not allowed",
            &SourceInfo::from_pair(pair, source),
        );
        return false;
    }
    for pair in inner {
        match pair.as_rule() {
            Rule::list | Rule::symbol | Rule::number | Rule::string => continue,
            _ => {
                print_error(
                    "Invalid expression inside block",
                    &SourceInfo::from_pair(&pair, source),
                );
                return false;
            }
        }
    }
    true
}

pub fn validate_params(pair: &Pair<Rule>, source: &SourceFile) -> bool {
    let inner = pair.clone().into_inner();
    let mut names = HashSet::new();
    for param in inner {
        let name = match param.as_rule() {
            Rule::symbol => param.as_str(),
            _ => {
                print_error(
                    "Parameter must be a symbol",
                    &SourceInfo::from_pair(&param, source),
                );
                return false;
            }
        };
        if !names.insert(name) {
            print_error(
                &format!("Duplicate parameter name: {}", name),
                &SourceInfo::from_pair(&param, source),
            );
            return false;
        }
    }
    true
}

pub fn validate_list(pair: &Pair<Rule>, source: &SourceFile) -> bool {
    let span = pair.as_span();
    let content = span.as_str().trim();
    if !content.starts_with('(') || !content.ends_with(')') {
        print_error(
            "Missing surrounding parentheses",
            &SourceInfo::from_pair(pair, source),
        );
        return false;
    }
    if pair.clone().into_inner().is_empty() {
        print_error(
            "Empty lists are not allowed",
            &SourceInfo::from_pair(pair, source),
        );
    }
    for inner_pair in pair.clone().into_inner() {
        match inner_pair.as_rule() {
            Rule::symbol => {
                let _ = Symbol::from_pair(&inner_pair);
            }
            Rule::list => {
                if !validate_list(&inner_pair, source) {
                    return false;
                }
            }
            Rule::number | Rule::boolean | Rule::string => {}
            Rule::EOI => {}
            _ => {
                print_error(
                    "Unexpected token in list",
                    &SourceInfo::from_pair(&inner_pair, source),
                );
                return false;
            }
        }
    }

    true
}
