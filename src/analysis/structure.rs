use crate::{analysis::print_error, parsing::ast::Node};
use colored::Colorize;
use log::trace;

static VALID_TYPES: &[&str] = &[
    "i8", "i16", "i32", "i64", "u8", "u16", "u32", "u64", "f16", "f32", "f64", "void", "string",
];

const OPERATORS: &[&str] = &[
    "+", "-", "/", "*", "|", "&", "!", ">", "<", "%", ">=", "<=", "==", "!=", "+=", "-=", "/=",
    "*=", "|=", "&=",
];

pub fn validate_annotations(ast: &Vec<Node>) -> bool {
    let mut is_valid = true;
    for node in ast {
        match node {
            Node::List(nodes, _) => is_valid &= validate_annotations(nodes),
            Node::TypedSymbol(_, annotation, info) => {
                if !VALID_TYPES.contains(&annotation.as_str()) {
                    print_error("Invalid type annotation", info);
                } else {
                    trace!(
                        "{} {} {} {} Valid type annotation: {}",
                        "Line:".cyan(),
                        info.line.to_string().cyan(),
                        "Column:".cyan(),
                        info.col.to_string().cyan(),
                        annotation.to_string().cyan()
                    );
                }
            }
            _ => {}
        }
    }
    return is_valid;
}

pub fn validate_fn_definitions(ast: &Vec<Node>) -> bool {
    let mut is_fn = false;
    let mut is_valid = true;
    // Is it an fn definition?
    match &ast[0] {
        Node::TypedSymbol(name, _, _) => {
            is_fn = name == "fn";
        }
        Node::Symbol(name, _) => {
            is_fn = name == "fn";
        }
        _ => {}
    }
    // If it is an fn definition, is it valid?
    if is_fn {
        // Is the size wrong?
        if ast.len() < 3 {
            is_valid &= false;
            print_error("Fn definition has too few parts", &ast[0].get_info());
        }
        if ast.len() > 4 {
            is_valid &= false;
            print_error("Fn definition has too many parts", &ast[0].get_info());
        // Is it anonymous?
        } else if ast.len() == 3 {
            let params = &ast[1];
            if let Node::List(sub_ast, _) = params {
                // Are params all typed identifiers?
                for node in sub_ast {
                    match node {
                        Node::TypedSymbol(_, _, _) => {}
                        _ => {
                            print_error("Not a typed parameter", node.get_info());
                        }
                    }
                }
            } else {
                print_error("Anonymous fn params is not a list", params.get_info());
                is_valid &= false;
            }
            let body = &ast[2];
            match body {
                Node::List(sub_ast, _) => is_valid &= validate_fn_definitions(sub_ast),
                Node::Symbol(_, _) => {}
                _ => {
                    print_error("Invalid anonymous fn body", body.get_info());
                }
            }
        // Is it a normal function?
        } else if ast.len() == 4 {
            let fn_name = &ast[1];
            if !matches!(fn_name, Node::Symbol(_, _)) {
                print_error("Name is not a valid identifier", fn_name.get_info());
            }
            let params = &ast[2];
            if let Node::List(sub_ast, _) = params {
                // Are params all typed identifiers?
                for node in sub_ast {
                    match node {
                        Node::TypedSymbol(_, _, _) => {}
                        _ => {
                            print_error("Not a typed parameter", node.get_info());
                        }
                    }
                }
            } else {
                print_error("Fn params are not a list", params.get_info());
                is_valid &= false;
            }
            let body = &ast[3];
            match body {
                Node::List(sub_ast, _) => is_valid &= validate_fn_definitions(sub_ast),
                Node::Symbol(name, info) => {
                    if OPERATORS.contains(&name.as_str()) {
                        print_error("Invalid fn body (lone operators are not allowed)", info);
                    }
                }
                _ => {
                    print_error("Invalid fn body", body.get_info());
                }
            }
        }
    // Else, maybe it's a list?
    } else {
        for node in ast {
            match node {
                Node::List(sub_ast, _) => is_valid &= validate_fn_definitions(sub_ast),
                _ => {}
            }
        }
    }
    is_valid
}
