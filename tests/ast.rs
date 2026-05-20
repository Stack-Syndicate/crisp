use crisp::parsing::{
    SourceFile,
    ast::{CrispParser, Rule, nodes::Node},
};
use pest::Parser;

fn parse_success(source_code: &str) -> String {
    let source_file = SourceFile::default();
    let mut pairs = CrispParser::parse(Rule::file, source_code)
        .unwrap_or_else(|e| panic!("Expected successful parse for '{}': {}", source_code, e));
    let top_pair = pairs.next().unwrap();
    let node_representation = Node::from_pair(top_pair, &source_file).to_test_string();
    format!(
        "--- INPUT ---\n{}\n\n--- SUCCESS ---\n{}",
        source_code, node_representation
    )
}

fn parse_failure(source_code: &str) -> String {
    let source_file = SourceFile::default();
    let node_representation = match CrispParser::parse(Rule::file, source_code) {
        Ok(mut pairs) => {
            if let Some(top_pair) = pairs.next() {
                let ast = Node::from_pair(top_pair, &source_file);
                ast.to_test_string()
            } else {
                "Parsing Succeeded but returned no tokens".to_string()
            }
        }
        Err(e) => e.to_string(),
    };
    format!(
        "--- INPUT ---\n{}\n\n--- FAILURE ---\n{}",
        source_code, node_representation
    )
}

#[test]
fn parse_let_statement() {
    let result = parse_success("(let x 5)");
    insta::assert_snapshot!("happy_let_statement", result);
}

#[test]
fn parse_function_with_args() {
    let result = parse_success("(fn:void add (a:i32 b:i32) (ret (+ a b)))");
    insta::assert_snapshot!("happy_function_with_args", result);
}

#[test]
fn parse_function_no_args() {
    let result = parse_success("(fn:i32 compute () (ret 42))");
    insta::assert_snapshot!("happy_function_no_args", result);
}

#[test]
fn parse_if_else() {
    let result = parse_success("(if (> x 1) (ret true) (ret false))");
    insta::assert_snapshot!("happy_if_else", result);
}

#[test]
fn parse_if_flat() {
    let result = parse_success("(if cond yes-flat None)");
    insta::assert_snapshot!("happy_if_flat", result);
}

#[test]
fn parse_binary_op_precedence() {
    let result = parse_success("(+ (* 2 3) (/ 10 5))");
    insta::assert_snapshot!("precedence_binary_ops", result);
}

#[test]
fn parse_mixed_comparison_precedence() {
    let result = parse_success("(> (+ x 1) (* y 2))");
    insta::assert_snapshot!("precedence_mixed_comparisons", result);
}

#[test]
fn parse_deeply_nested_calls() {
    let result = parse_success("(let total (add (multiply 2 5) (subtract 10 3)))");
    insta::assert_snapshot!("nesting_deep_calls", result);
}

#[test]
fn parse_nested_if_statements() {
    let result = parse_success("(if cond1 (if cond2 yes1 no2) no1)");
    insta::assert_snapshot!("nesting_conditions", result);
}

#[test]
fn parse_whitespace_horizontal_spacing() {
    let result = parse_success("(let    x      5)");
    insta::assert_snapshot!("whitespace_horizontal", result);
}

#[test]
fn parse_whitespace_new_lines() {
    let result = parse_success("(let x \n  (+ 5 5))");
    insta::assert_snapshot!("whitespace_newlines", result);
}

#[test]
fn parse_whitespace_multiline_block() {
    let result = parse_success(
        "(fn:void process (a:i32)\n  (if (> a 0)\n    ((ret true))\n    ((ret false))\n  )\n)",
    );
    insta::assert_snapshot!("whitespace_multiline_block", result);
}

#[test]
fn error_let_missing_value() {
    let result = parse_failure("(let x)");
    insta::assert_snapshot!("error_let_missing_value", result);
}

#[test]
fn error_let_empty() {
    let result = parse_failure("(let)");
    insta::assert_snapshot!("error_let_empty", result);
}

#[test]
fn error_fn_missing_argument_types() {
    let result = parse_failure("(fn:void add (a b) (ret (+ a b)))");
    insta::assert_snapshot!("error_fn_missing_arg_types", result);
}

#[test]
fn error_fn_missing_return_type() {
    let result = parse_failure("(fn add (a:i32 b:i32) (ret 0))");
    insta::assert_snapshot!("error_fn_missing_return_type", result);
}

#[test]
fn error_if_empty() {
    let result = parse_failure("(if)");
    insta::assert_snapshot!("error_if_empty", result);
}

#[test]
fn error_if_missing_branches() {
    let result = parse_failure("(if cond)");
    insta::assert_snapshot!("error_if_missing_branches", result);
}
