#![allow(clippy::approx_constant)]
use chumsky::{Parser, span::SimpleSpan};
use crisp::parsing::ast::*;
use crisp::{
    consts::*,
    parsing::{
        ast::{ParseExpr, ParseExprKind},
        crisp_parser,
    },
};
use std::collections::HashMap;

fn expr(kind: ParseExprKind) -> ParseExpr {
    ParseExpr {
        kind,
        span: SimpleSpan::default(),
        id: None,
    }
}

fn strip_spans(mut parse_expr: ParseExpr) -> ParseExpr {
    parse_expr.span = SimpleSpan::default();
    parse_expr.id = None;
    parse_expr.kind = match parse_expr.kind {
        ParseExprKind::MemberAccess(parts) => {
            ParseExprKind::MemberAccess(parts.into_iter().map(strip_spans).collect())
        }
        ParseExprKind::Def {
            name,
            type_annotation,
            value,
        } => ParseExprKind::Def {
            name,
            type_annotation,
            value: Box::new(strip_spans(*value)),
        },
        ParseExprKind::Fn {
            params,
            return_type,
            body,
        } => ParseExprKind::Fn {
            params,
            return_type,
            body: Box::new(strip_spans(*body)),
        },
        ParseExprKind::Call { callee, args } => ParseExprKind::Call {
            callee: Box::new(strip_spans(*callee)),
            args: args.into_iter().map(strip_spans).collect(),
        },
        ParseExprKind::If {
            condition,
            then_branch,
            else_branch,
        } => ParseExprKind::If {
            condition: Box::new(strip_spans(*condition)),
            then_branch: Box::new(strip_spans(*then_branch)),
            else_branch: else_branch.map(|b| Box::new(strip_spans(*b))),
        },
        ParseExprKind::Loop { condition, body } => ParseExprKind::Loop {
            condition: Box::new(strip_spans(*condition)),
            body: Box::new(strip_spans(*body)),
        },
        ParseExprKind::Block(stmts) => {
            ParseExprKind::Block(stmts.into_iter().map(strip_spans).collect())
        }
        ParseExprKind::Protocol { params, fns } => ParseExprKind::Protocol {
            params,
            fns: fns.into_iter().map(strip_spans).collect(),
        },
        ParseExprKind::Literal(Literal::Quote(inner)) => {
            ParseExprKind::Literal(Literal::Quote(Box::new(strip_spans(*inner))))
        }
        ParseExprKind::Literal(Literal::Map(map)) => ParseExprKind::Literal(Literal::Map(
            map.into_iter().map(|(k, v)| (k, strip_spans(v))).collect(),
        )),
        other => other,
    };
    parse_expr
}

fn parse_ast(input: &str) -> ParseExprKind {
    let parsed = crisp_parser()
        .parse(input)
        .into_result()
        .unwrap_or_else(|errs| panic!("Failed to parse '{input}': {errs:?}"));
    strip_spans(parsed[0].clone()).kind
}

#[test]
fn test_literals() {
    assert_eq!(
        parse_ast("123"),
        ParseExprKind::Literal(Literal::Int32(123))
    );
    assert_eq!(
        parse_ast("-456"),
        ParseExprKind::Literal(Literal::Int32(-456))
    );
    assert_eq!(
        parse_ast("3.14"),
        ParseExprKind::Literal(Literal::Float64(3.14))
    );
    assert_eq!(
        parse_ast("-0.001"),
        ParseExprKind::Literal(Literal::Float64(-0.001))
    );
    assert_eq!(
        parse_ast("\"hello world\""),
        ParseExprKind::Literal(Literal::Str("hello world".into()))
    );
    assert_eq!(
        parse_ast("true"),
        ParseExprKind::Literal(Literal::Bool(true))
    );
    assert_eq!(
        parse_ast("false"),
        ParseExprKind::Literal(Literal::Bool(false))
    );
}

#[test]
fn test_identifiers_and_access() {
    assert_eq!(
        parse_ast("my_var"),
        ParseExprKind::Identifier("my_var".into())
    );
    assert_eq!(
        parse_ast("my-var"),
        ParseExprKind::Identifier("my-var".into())
    );
    assert_eq!(
        parse_ast("my_var:i32"),
        ParseExprKind::IdentifierAnnotated(("my_var".into(), Type::I32))
    );
    assert_eq!(
        parse_ast(&format!("{}my_placeholder", PLACEHOLDER_MODIFIER)),
        ParseExprKind::IdentifierPlaceholder("my_placeholder".into())
    );
    assert_eq!(
        parse_ast(&format!(
            "obj{}prop{}sub",
            MEMBER_ACCESS_SEPARATOR, MEMBER_ACCESS_SEPARATOR
        )),
        ParseExprKind::MemberAccess(vec![
            expr(ParseExprKind::Identifier("obj".into())),
            expr(ParseExprKind::Identifier("prop".into())),
            expr(ParseExprKind::Identifier("sub".into())),
        ])
    );
}

#[test]
fn test_collections_and_quote() {
    let mut expected_map = HashMap::new();
    expected_map.insert(
        "a".to_string(),
        expr(ParseExprKind::Literal(Literal::Int32(1))),
    );
    expected_map.insert(
        "b".to_string(),
        expr(ParseExprKind::Literal(Literal::Int32(2))),
    );

    assert_eq!(
        parse_ast("{ :a 1 :b 2 }"),
        ParseExprKind::Literal(Literal::Map(expected_map))
    );
    assert_eq!(
        parse_ast(&format!("{}my_var", QUOTE_MODIFIER)),
        ParseExprKind::Literal(Literal::Quote(Box::new(expr(ParseExprKind::Identifier(
            "my_var".into()
        )))))
    );
    assert_eq!(
        parse_ast(&format!("{}(call 1 2)", QUOTE_MODIFIER)),
        ParseExprKind::Literal(Literal::Quote(Box::new(expr(ParseExprKind::Call {
            callee: Box::new(expr(ParseExprKind::Identifier("call".into()))),
            args: vec![
                expr(ParseExprKind::Literal(Literal::Int32(1))),
                expr(ParseExprKind::Literal(Literal::Int32(2))),
            ],
        }))))
    );
}

#[test]
fn test_definitions() {
    assert_eq!(
        parse_ast("(def x 10)"),
        ParseExprKind::Def {
            name: "x".into(),
            type_annotation: None,
            value: Box::new(expr(ParseExprKind::Literal(Literal::Int32(10)))),
        }
    );
    assert_eq!(
        parse_ast("(def y:str \"text\")"),
        ParseExprKind::Def {
            name: "y".into(),
            type_annotation: Some(Type::Str),
            value: Box::new(expr(ParseExprKind::Literal(Literal::Str("text".into())))),
        }
    );
    assert_eq!(
        parse_ast(&format!(
            "(defn add [a:i32 b:i32] {} i32 (add a b))",
            RETURN_TYPE_STRING
        )),
        ParseExprKind::Def {
            name: "add".into(),
            type_annotation: None,
            value: Box::new(expr(ParseExprKind::Fn {
                params: vec![
                    Param {
                        name: "a".into(),
                        type_annotation: Type::I32,
                    },
                    Param {
                        name: "b".into(),
                        type_annotation: Type::I32,
                    },
                ],
                return_type: Type::I32,
                body: Box::new(expr(ParseExprKind::Block(vec![expr(
                    ParseExprKind::Call {
                        callee: Box::new(expr(ParseExprKind::Identifier("add".to_string()))),
                        args: vec![
                            expr(ParseExprKind::Identifier("a".to_string())),
                            expr(ParseExprKind::Identifier("b".to_string()))
                        ]
                    }
                )])))
            })),
        }
    );
    assert_eq!(
        parse_ast("(defm Point [x:f32 y:f32])"),
        ParseExprKind::Def {
            name: "Point".to_string(),
            type_annotation: None,
            value: Box::new(expr(ParseExprKind::Map {
                params: vec![
                    Param {
                        name: "x".to_string(),
                        type_annotation: Type::F32
                    },
                    Param {
                        name: "y".to_string(),
                        type_annotation: Type::F32
                    }
                ]
            }))
        }
    );
    assert_eq!(
        parse_ast(&format!(
            "(defp Drawable [d:i32] (defn draw [p:@Drawable] {} void d))",
            RETURN_TYPE_STRING
        )),
        ParseExprKind::Def {
            name: "Drawable".to_string(),
            type_annotation: None,
            value: Box::new(expr(ParseExprKind::Protocol {
                params: vec![Param {
                    name: "d".to_string(),
                    type_annotation: Type::I32
                }],
                fns: vec![expr(ParseExprKind::Def {
                    name: "draw".to_string(),
                    type_annotation: None,
                    value: Box::new(expr(ParseExprKind::Fn {
                        params: vec![Param {
                            name: "p".to_string(),
                            type_annotation: Type::Protocol(Box::new(Type::Custom(
                                "Drawable".to_string()
                            )))
                        }],
                        return_type: Type::Void,
                        body: Box::new(expr(ParseExprKind::Block(vec![expr(
                            ParseExprKind::Identifier("d".to_string())
                        )])))
                    }))
                })]
            }))
        }
    );
}

#[test]
fn test_functions_and_calls() {
    assert_eq!(
        parse_ast(&format!(
            "(fn [x:i32] {} i32 (mul x x))",
            RETURN_TYPE_STRING
        )),
        ParseExprKind::Fn {
            params: vec![Param {
                name: "x".into(),
                type_annotation: Type::I32,
            }],
            return_type: Type::I32,
            body: Box::new(expr(ParseExprKind::Block(vec![expr(
                ParseExprKind::Call {
                    callee: Box::new(expr(ParseExprKind::Identifier("mul".to_string()))),
                    args: vec![
                        expr(ParseExprKind::Identifier("x".to_string())),
                        expr(ParseExprKind::Identifier("x".to_string()))
                    ]
                }
            )]))),
        }
    );
    assert_eq!(
        parse_ast("(print \"hello\")"),
        ParseExprKind::Call {
            callee: Box::new(expr(ParseExprKind::Identifier("print".into()))),
            args: vec![expr(ParseExprKind::Literal(Literal::Str("hello".into())))],
        }
    );
    assert_eq!(
        parse_ast("(add 1 (mul 2 3))"),
        ParseExprKind::Call {
            callee: Box::new(expr(ParseExprKind::Identifier("add".into()))),
            args: vec![
                expr(ParseExprKind::Literal(Literal::Int32(1))),
                expr(ParseExprKind::Call {
                    callee: Box::new(expr(ParseExprKind::Identifier("mul".into()))),
                    args: vec![
                        expr(ParseExprKind::Literal(Literal::Int32(2))),
                        expr(ParseExprKind::Literal(Literal::Int32(3))),
                    ],
                }),
            ],
        }
    );
}

#[test]
fn test_type_keywords() {
    assert_eq!(
        parse_ast("(def a:bool true)"),
        ParseExprKind::Def {
            name: "a".into(),
            type_annotation: Some(Type::Bool),
            value: Box::new(expr(ParseExprKind::Literal(Literal::Bool(true)))),
        }
    );
    assert_eq!(
        parse_ast("(def b:MyCustomType 1)"),
        ParseExprKind::Def {
            name: "b".into(),
            type_annotation: Some(Type::Custom("MyCustomType".into())),
            value: Box::new(expr(ParseExprKind::Literal(Literal::Int32(1)))),
        }
    );
    assert_eq!(
        parse_ast(&format!(
            "(def c:(fn [x:i32] {} bool) true)",
            RETURN_TYPE_STRING
        )),
        ParseExprKind::Def {
            name: "c".into(),
            type_annotation: Some(Type::Function {
                params: vec![Param {
                    name: "x".into(),
                    type_annotation: Type::I32,
                }],
                return_type: Box::new(Type::Bool),
            }),
            value: Box::new(expr(ParseExprKind::Literal(Literal::Bool(true)))),
        }
    );
    assert_eq!(
        parse_ast(&format!("(def d:{}MyProtocol 1)", PROTOCOL_MODIFIER)),
        ParseExprKind::Def {
            name: "d".into(),
            type_annotation: Some(Type::Protocol(Box::new(Type::Custom("MyProtocol".into())))),
            value: Box::new(expr(ParseExprKind::Literal(Literal::Int32(1)))),
        }
    );
    assert_eq!(
        parse_ast(&format!("(def e:{}i32 1)", OPTION_MODIFIER)),
        ParseExprKind::Def {
            name: "e".into(),
            type_annotation: Some(Type::Option(Box::new(Type::I32))),
            value: Box::new(expr(ParseExprKind::Literal(Literal::Int32(1)))),
        }
    );
    assert_eq!(
        parse_ast(&format!("(def f:{}str \"\")", RESULT_MODIFIER)),
        ParseExprKind::Def {
            name: "f".into(),
            type_annotation: Some(Type::Result(Box::new(Type::Str))),
            value: Box::new(expr(ParseExprKind::Literal(Literal::Str("".into())))),
        }
    );
}

#[test]
fn test_error_recovery() {
    let result = crisp_parser()
        .parse("(def x 10) (unclosed_paren ")
        .into_result();
    assert!(
        result.is_err(),
        "Expected parsing to fail due to unclosed parenthesis"
    );
}
