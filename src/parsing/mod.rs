pub mod ast;
pub mod error;

use std::collections::HashMap;

use crate::consts::*;

use crate::parsing::ast::{Literal, Param, ParseExpr, ParseExprKind, Type};
use chumsky::{extra::Err, prelude::*};

fn operator<'a>() -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone {
    one_of(OPERATORS)
        .map(String::from)
        .map(ParseExprKind::Identifier)
}

fn identifier<'a>() -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone {
    let ident_part = any()
        .filter(|c: &char| c.is_alphanumeric() || *c == '_')
        .repeated()
        .at_least(1);
    text::ident()
        .then(just('-').then(ident_part).repeated())
        .to_slice()
        .map(String::from)
        .map(ParseExprKind::Identifier)
}

fn member_access<'a>() -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone {
    identifier()
        .spanned()
        .then(
            just(MEMBER_ACCESS_SEPARATOR)
                .ignore_then(identifier())
                .spanned()
                .repeated()
                .collect::<Vec<_>>(),
        )
        .map(|(first, rest)| {
            let mut parts = vec![ParseExpr {
                kind: first.inner.clone(),
                span: first.span,
                id: None,
            }];
            parts.extend(rest.iter().map(|member| ParseExpr {
                kind: member.inner.clone(),
                span: member.span,
                id: None,
            }));
            ParseExprKind::MemberAccess(parts)
        })
}

fn identifier_annotated<'a>() -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone
{
    identifier()
        .then_ignore(just(':'))
        .then(type_keyword())
        .try_map(|(ident, ty), span| match ident {
            ParseExprKind::Identifier(name) => Ok(ParseExprKind::IdentifierAnnotated((name, ty))),
            _ => Err(Rich::custom(span, "invalid identifier")),
        })
}

fn integer<'a>() -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone {
    just('-')
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
        })
}

fn float<'a>() -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone {
    just('-')
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
        })
}

fn string<'a>() -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone {
    just('"')
        .ignore_then(none_of('"').repeated().collect::<String>())
        .then_ignore(just('"'))
        .map(|s| ParseExprKind::Literal(Literal::Str(s)))
}

fn boolean<'a>() -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone {
    choice((just("true").to(true), just("false").to(false)))
        .map(|b| ParseExprKind::Literal(Literal::Bool(b)))
}

fn map_literal<'a, P>(
    expr: P,
) -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone
where
    P: Parser<'a, &'a str, ParseExpr, Err<Rich<'a, char>>> + Clone,
{
    just(':')
        .ignore_then(identifier())
        .padded()
        .then(expr.clone())
        .padded()
        .repeated()
        .collect::<Vec<_>>()
        .delimited_by(just('{'), just('}'))
        .try_map(|entries, span| {
            let mut map = HashMap::new();
            for (key, value) in entries {
                let ParseExprKind::Identifier(name) = key else {
                    return Err(Rich::custom(span, "invalid map key"));
                };
                map.insert(name, value);
            }
            Ok(ParseExprKind::Literal(Literal::Map(map)))
        })
}

fn quote_literal<'a, P>(
    expr: P,
) -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone
where
    P: Parser<'a, &'a str, ParseExpr, Err<Rich<'a, char>>> + Clone,
{
    just(QUOTE_MODIFIER)
        .ignore_then(expr)
        .map(|expr| ParseExprKind::Literal(Literal::Quote(Box::new(expr))))
}

fn type_keyword<'a>() -> impl Parser<'a, &'a str, Type, Err<Rich<'a, char>>> + Clone {
    recursive(|ty| {
        let primitive = choice((
            text::keyword("i32").to(Type::I32),
            text::keyword("i64").to(Type::I64),
            text::keyword("f32").to(Type::F32),
            text::keyword("f64").to(Type::F64),
            text::keyword("u32").to(Type::U32),
            text::keyword("u64").to(Type::U64),
            text::keyword("bool").to(Type::Bool),
            text::keyword("str").to(Type::Str),
            text::keyword("void").to(Type::Void),
            text::keyword("code").to(Type::Code),
            identifier().map(|kind| match kind {
                ParseExprKind::Identifier(name) => Type::Custom(name),
                _ => unreachable!(),
            }),
        ));

        let function = text::keyword("fn")
            .padded()
            .ignore_then(
                just('[')
                    .padded()
                    .ignore_then(
                        identifier()
                            .try_map(|ident, span| match ident {
                                ParseExprKind::Identifier(name) => Ok(name),
                                _ => Err(Rich::custom(span, "invalid parameter name")),
                            })
                            .padded()
                            .then_ignore(just(':'))
                            .padded()
                            .then(ty.clone())
                            .repeated()
                            .collect::<Vec<_>>(),
                    )
                    .then_ignore(just(']').padded()),
            )
            .then_ignore(just(RETURN_TYPE_STRING).padded())
            .then(ty.clone())
            .delimited_by(just('('), just(')'))
            .map(|(params, return_type)| Type::Function {
                params: params
                    .into_iter()
                    .map(|(name, ty)| Param {
                        name,
                        type_annotation: ty,
                    })
                    .collect(),
                return_type: Box::new(return_type),
            });
        let protocol = just(PROTOCOL_MODIFIER)
            .ignore_then(identifier())
            .try_map(|kind, span| match kind {
                ParseExprKind::Identifier(name) => Ok(Type::Protocol(Box::new(Type::Custom(name)))),
                _ => Err(Rich::custom(span, "invalid protocol type keyword")),
            });
        let option =
            just(OPTION_MODIFIER)
                .ignore_then(identifier())
                .try_map(|kind, span| match kind {
                    ParseExprKind::Identifier(name) => {
                        Ok(Type::Option(Box::new(Type::Custom(name))))
                    }
                    _ => Err(Rich::custom(span, "invalid option type keyword")),
                });
        let result =
            just(RESULT_MODIFIER)
                .ignore_then(identifier())
                .try_map(|kind, span| match kind {
                    ParseExprKind::Identifier(name) => {
                        Ok(Type::Result(Box::new(Type::Custom(name))))
                    }
                    _ => Err(Rich::custom(span, "invalid result type keyword")),
                });
        choice((function, protocol, option, result, primitive))
    })
}

fn call<'a, P>(expr: P) -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone
where
    P: Parser<'a, &'a str, ParseExpr, Err<Rich<'a, char>>> + Clone,
{
    just('(')
        .ignore_then(
            expr.clone()
                .padded()
                .then(expr.clone().padded().repeated().collect::<Vec<_>>()),
        )
        .then_ignore(just(')'))
        .map(|(callee, args)| ParseExprKind::Call {
            callee: Box::new(callee),
            args,
        })
}

fn define_variable<'a, P>(
    expr: P,
) -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone
where
    P: Parser<'a, &'a str, ParseExpr, Err<Rich<'a, char>>> + Clone,
{
    text::keyword("def")
        .padded()
        .ignore_then(identifier_annotated().or(identifier()))
        .padded()
        .then(expr)
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
        })
}

fn define_function<'a, P>(
    expr: P,
) -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone
where
    P: Parser<'a, &'a str, ParseExpr, Err<Rich<'a, char>>> + Clone,
{
    text::keyword("defn")
        .padded()
        .ignore_then(identifier())
        .padded()
        .then_ignore(just('['))
        .then(
            identifier_annotated()
                .padded()
                .repeated()
                .collect::<Vec<_>>(),
        )
        .padded()
        .then_ignore(just(']'))
        .padded()
        .then(
            just(RETURN_TYPE_STRING)
                .padded()
                .ignore_then(type_keyword()),
        )
        .then(expr.repeated().collect::<Vec<_>>())
        .delimited_by(just('('), just(')'))
        .try_map(|(((name, params), return_type), body), span| {
            let name = match name {
                ParseExprKind::Identifier(name) => name,
                _ => return Err(Rich::custom(span, "invalid identifier")),
            };
            let params = params
                .into_iter()
                .map(|param| match param {
                    ParseExprKind::IdentifierAnnotated((name, type_annotation)) => Param {
                        name,
                        type_annotation,
                    },
                    _ => panic!("parser broke"),
                })
                .collect::<Vec<_>>();
            Ok(ParseExprKind::Def {
                name,
                type_annotation: None,
                value: Box::new(ParseExpr {
                    kind: ParseExprKind::Fn {
                        params,
                        return_type,
                        body: Box::new(ParseExpr {
                            kind: ParseExprKind::Block(body),
                            span,
                            id: None,
                        }),
                    },
                    span,
                    id: None,
                }),
            })
        })
}

fn define_map<'a>() -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone {
    text::keyword("defm")
        .padded()
        .ignore_then(identifier())
        .padded()
        .then(
            identifier_annotated()
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
                    ParseExprKind::IdentifierAnnotated((name, type_annotation)) => Ok(Param {
                        name,
                        type_annotation,
                    }),
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
        })
}

fn define_protocol<'a, P>(
    expr: P,
) -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone
where
    P: Parser<'a, &'a str, ParseExpr, Err<Rich<'a, char>>> + Clone,
{
    text::keyword("defp")
        .padded()
        .ignore_then(identifier())
        .padded()
        .then(
            identifier_annotated()
                .padded()
                .repeated()
                .collect::<Vec<_>>()
                .delimited_by(just('[').padded(), just(']').padded()),
        )
        .then(
            define_function(expr.clone())
                .padded()
                .repeated()
                .collect::<Vec<_>>(),
        )
        .delimited_by(just('(').padded(), just(')').padded())
        .try_map(|((name, params), fns), span| {
            let ParseExprKind::Identifier(name) = name else {
                return Err(Rich::custom(span, "invalid protocol name"));
            };

            let params = params
                .into_iter()
                .map(|param| match param {
                    ParseExprKind::IdentifierAnnotated((name, ty)) => Ok(Param {
                        name,
                        type_annotation: ty,
                    }),
                    _ => Err(Rich::custom(span, "invalid parameter")),
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(ParseExprKind::Def {
                name,
                type_annotation: None,
                value: Box::new(ParseExpr {
                    kind: ParseExprKind::Protocol {
                        params,
                        fns: fns
                            .iter()
                            .map(|f| ParseExpr {
                                kind: f.clone(),
                                span,
                                id: None,
                            })
                            .collect(),
                    },
                    span,
                    id: None,
                }),
            })
        })
}
fn anonymous_function<'a, P>(
    expr: P,
) -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone
where
    P: Parser<'a, &'a str, ParseExpr, Err<Rich<'a, char>>> + Clone,
{
    text::keyword("fn")
        .padded()
        .ignore_then(just('['))
        .ignore_then(
            identifier_annotated()
                .padded()
                .repeated()
                .collect::<Vec<_>>(),
        )
        .padded()
        .then_ignore(just(']'))
        .padded()
        .then(
            just(RETURN_TYPE_STRING)
                .padded()
                .ignore_then(type_keyword()),
        )
        .then(expr.repeated().collect::<Vec<_>>())
        .delimited_by(just('('), just(')'))
        .try_map(|((params, return_type), body), span| {
            let params = params
                .into_iter()
                .map(|param| match param {
                    ParseExprKind::IdentifierAnnotated((name, type_annotation)) => Param {
                        name,
                        type_annotation,
                    },
                    _ => panic!("parser broke"),
                })
                .collect::<Vec<_>>();

            Ok(ParseExprKind::Fn {
                params,
                return_type,
                body: Box::new(ParseExpr {
                    kind: ParseExprKind::Block(body),
                    span,
                    id: None,
                }),
            })
        })
}

pub fn crisp_parser<'a>() -> impl Parser<'a, &'a str, Vec<ParseExpr>, Err<Rich<'a, char>>> {
    let expr = recursive(|expr| {
        let number = choice((float(), integer()));
        let literal = choice((
            quote_literal(expr.clone()),
            map_literal(expr.clone()),
            string(),
            number,
            boolean(),
        ));
        let call = call(expr.clone());
        let define_variable = define_variable(expr.clone());
        let define_function = define_function(expr.clone());
        let define_map = define_map();
        let define_protocol = define_protocol(expr.clone());
        let anonymous_function = anonymous_function(expr.clone());
        let special_form = choice((
            define_variable,
            define_function,
            define_map,
            define_protocol,
            anonymous_function,
        ));
        choice((
            special_form,
            call,
            member_access(),
            literal,
            identifier_annotated(),
            identifier(),
            operator(),
        ))
        .padded()
        .map_with(|kind, extra| ParseExpr {
            kind,
            span: extra.span(),
            id: None,
        })
    });
    expr.recover_with(via_parser(nested_delimiters(
        '(',
        ')',
        [('[', ']'), ('{', '}')],
        |span| ParseExpr {
            kind: ParseExprKind::Error,
            span,
            id: None,
        },
    )))
    .repeated()
    .collect()
}
