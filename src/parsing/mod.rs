pub mod ast;
pub mod error;

use crate::OPERATORS;
use crate::parsing::ast::{Literal, Param, ParseExpr, ParseExprKind, Type};
use chumsky::{extra::Err, prelude::*};

pub fn crisp_parser<'a>() -> impl Parser<'a, &'a str, Vec<ParseExpr>, Err<Rich<'a, char>>> {
    let expr = recursive(|expr| {
        let operator = one_of(OPERATORS)
            .map(String::from)
            .map(ParseExprKind::Identifier);
        let identifier = text::ident()
            .then(
                just('-')
                    .then(
                        any()
                            .filter(|c: &char| c.is_alphanumeric() || *c == '_')
                            .repeated()
                            .at_least(1),
                    )
                    .repeated(),
            )
            .to_slice()
            .map(String::from)
            .map(ParseExprKind::Identifier);
        let literal = {
            let boolean = choice((just("true").to(true), just("false").to(false)))
                .map(|b| ParseExprKind::Literal(Literal::Bool(b)));
            let string = just('"')
                .ignore_then(none_of('"').repeated().collect::<String>())
                .then_ignore(just('"'))
                .map(|s| ParseExprKind::Literal(Literal::Str(s)));
            let number = {
                let integer = just('-')
                    .or_not()
                    .then(text::digits(10).at_least(1).to_slice())
                    .to_slice()
                    .try_map(|s: &str, span| {
                        if let Ok(value) = s.parse::<i32>() {
                            Ok(ParseExprKind::Literal(Literal::Int32(value)))
                        } else if let Ok(value) = s.parse::<i64>() {
                            Ok(ParseExprKind::Literal(Literal::Int64(value)))
                        } else {
                            Err(Rich::custom(span, "integer literal is too large"))
                        }
                    });
                let float = just('-')
                    .or_not()
                    .then(text::digits(10).at_least(1).to_slice())
                    .then(just('.'))
                    .then(text::digits(10).at_least(1).to_slice())
                    .to_slice()
                    .try_map(|s: &str, span| {
                        if let Ok(value) = s.parse::<f32>() {
                            Ok(ParseExprKind::Literal(Literal::Float32(value)))
                        } else if let Ok(value) = s.parse::<f64>() {
                            Ok(ParseExprKind::Literal(Literal::Float64(value)))
                        } else {
                            Err(Rich::custom(span, "float literal is too large"))
                        }
                    });
                choice((float, integer))
            };
            choice((string, number, boolean))
        };
        let type_keyword = choice((
            just("i32").to(Type::I32),
            just("i64").to(Type::I64),
            just("f32").to(Type::F32),
            just("f64").to(Type::F64),
            just("u32").to(Type::U32),
            just("u64").to(Type::U64),
            just("bool").to(Type::Bool),
            just("str").to(Type::Str),
            just("void").to(Type::Void),
            identifier.map(|kind| match kind {
                ParseExprKind::Identifier(name) => Type::Custom(name),
                _ => unreachable!(),
            }),
        ));
        let identifier_annotated = identifier
            .then_ignore(just(':'))
            .then(type_keyword.clone())
            .try_map(|(ident, t), span| match ident {
                ParseExprKind::Identifier(name) => {
                    Ok(ParseExprKind::IdentifierAnnotated((name, t)))
                }
                _ => Err(Rich::custom(span, "invalid identifier")),
            });
        let special_form = {
            let call = identifier
                .or(operator)
                .padded()
                .then(expr.clone().repeated().collect::<Vec<_>>())
                .delimited_by(just('('), just(')'))
                .try_map(|(name, args), span| {
                    Ok(ParseExprKind::Call {
                        callee: Box::new(ParseExpr {
                            kind: name,
                            span,
                            id: None,
                        }),
                        args,
                    })
                });
            let define_variable = text::keyword("def")
                .padded()
                .ignore_then(identifier_annotated.clone().or(identifier))
                .padded()
                .then(expr.clone())
                .delimited_by(just('('), just(')'))
                .try_map(|(ident, value), span| {
                    let (name, type_annotation) = match ident {
                        ParseExprKind::Identifier(name) => (name, None),
                        ParseExprKind::IdentifierAnnotated((name, ty)) => (name, Some(ty)),
                        _ => return Err(Rich::custom(span, "invalid identifier")),
                    };

                    Ok(ParseExprKind::Def {
                        name,
                        type_annotation,
                        value: Box::new(value),
                    })
                });
            let define_function = text::keyword("defn")
                .padded()
                .ignore_then(identifier)
                .padded()
                .then_ignore(just('['))
                .then(
                    identifier_annotated
                        .clone()
                        .padded()
                        .repeated()
                        .collect::<Vec<_>>(),
                )
                .padded()
                .then_ignore(just(']'))
                .padded()
                .then(just("->").padded().ignore_then(type_keyword.clone()))
                .then(expr.clone().repeated().collect::<Vec<_>>())
                .delimited_by(just('('), just(')'))
                .try_map(|(((name, params), return_type), body), span| {
                    Ok(ParseExprKind::Def {
                        name: match name {
                            ParseExprKind::Identifier(name) => name,
                            _ => return Err(Rich::custom(span, "invalid identifier")),
                        },
                        type_annotation: None,
                        value: Box::new(ParseExpr {
                            kind: ParseExprKind::Fn {
                                params: params
                                    .iter()
                                    .map(|param| {
                                        if let ParseExprKind::IdentifierAnnotated((
                                            name,
                                            type_annotation,
                                        )) = param
                                        {
                                            Param {
                                                name: name.clone(),
                                                type_annotation: type_annotation.clone(),
                                            }
                                        } else {
                                            panic!("parser broke")
                                        }
                                    })
                                    .collect::<Vec<_>>(),
                                return_type,
                                body: Box::new(ParseExpr {
                                    kind: ParseExprKind::Block(
                                        body.iter().map(|expr| (*expr).clone()).collect::<Vec<_>>(),
                                    ),
                                    span,
                                    id: None,
                                }),
                            },
                            id: None,
                            span,
                        }),
                    })
                });
            let define_map = text::keyword("defm")
                .padded()
                .ignore_then(identifier)
                .padded()
                .then(
                    identifier_annotated
                        .clone()
                        .padded()
                        .repeated()
                        .collect::<Vec<_>>()
                        .delimited_by(just('[').padded(), just(']').padded()),
                )
                .delimited_by(just('(').padded(), just(')').padded())
                .try_map(|(name, params), span| {
                    let name = match name {
                        ParseExprKind::Identifier(name) => name,
                        _ => return Err(Rich::custom(span, "invalid identifier")),
                    };
                    let params = params
                        .into_iter()
                        .map(|param| match param {
                            ParseExprKind::IdentifierAnnotated((name, type_annotation)) => {
                                Ok(Param {
                                    name,
                                    type_annotation,
                                })
                            }
                            _ => Err(Rich::custom(span, "invalid annotated identifier")),
                        })
                        .collect::<Result<Vec<_>, _>>()?;
                    Ok(ParseExprKind::Def {
                        name,
                        type_annotation: None,
                        value: Box::new(ParseExpr {
                            kind: ParseExprKind::Map { params },
                            id: None,
                            span,
                        }),
                    })
                });
            let anonymous_function = text::keyword("fn")
                .padded()
                .ignore_then(just('['))
                .ignore_then(
                    identifier_annotated
                        .clone()
                        .padded()
                        .repeated()
                        .collect::<Vec<_>>(),
                )
                .padded()
                .then_ignore(just(']'))
                .padded()
                .then(just("->").padded().ignore_then(type_keyword.clone()))
                .then(expr.clone().repeated().collect::<Vec<_>>())
                .delimited_by(just('('), just(')'))
                .try_map(|((params, return_type), body), span| {
                    Ok(ParseExprKind::Fn {
                        params: params
                            .iter()
                            .map(|param| {
                                if let ParseExprKind::IdentifierAnnotated((name, type_annotation)) =
                                    param
                                {
                                    Param {
                                        name: name.clone(),
                                        type_annotation: type_annotation.clone(),
                                    }
                                } else {
                                    panic!("parser broke")
                                }
                            })
                            .collect::<Vec<_>>(),
                        return_type,
                        body: Box::new(ParseExpr {
                            kind: ParseExprKind::Block(
                                body.iter().map(|expr| (*expr).clone()).collect::<Vec<_>>(),
                            ),
                            span,
                            id: None,
                        }),
                    })
                });

            choice((
                define_variable,
                define_function,
                define_map,
                anonymous_function,
                call,
            ))
        };
        choice((
            special_form,
            literal,
            identifier_annotated,
            identifier,
            operator,
        ))
        .padded()
        .map_with(|kind, extra| ParseExpr {
            kind,
            span: extra.span(),
            id: None,
        })
    });
    expr.recover_with(chumsky::recovery::via_parser(
        chumsky::recovery::nested_delimiters('(', ')', [('[', ']'), ('{', '}')], |span| {
            ParseExpr {
                kind: ParseExprKind::Identifier(String::from("parse_error")),
                span,
                id: None,
            }
        }),
    ))
    .repeated()
    .collect()
}
