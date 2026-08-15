pub mod ast;
pub mod error;

use crate::OPERATORS;
use crate::parsing::ast::{Literal, Param, ParseExpr, ParseExprKind, Type};
use chumsky::{extra::Err, prelude::*};

fn operator<'a>() -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone {
    one_of(OPERATORS)
        .map(String::from)
        .map(ParseExprKind::Identifier)
}

fn identifier<'a>() -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone {
    text::ident()
        .then(just('-').then(text::ident()).repeated())
        .to_slice()
        .map(String::from)
        .map(ParseExprKind::Identifier)
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

fn type_keyword<'a>() -> impl Parser<'a, &'a str, Type, Err<Rich<'a, char>>> + Clone {
    choice((
        just("i32").to(Type::I32),
        just("i64").to(Type::I64),
        just("f32").to(Type::F32),
        just("f64").to(Type::F64),
        just("u32").to(Type::U32),
        just("u64").to(Type::U64),
        just("bool").to(Type::Bool),
        just("str").to(Type::Str),
        just("void").to(Type::Void),
        identifier().map(|kind| match kind {
            ParseExprKind::Identifier(name) => Type::Custom(name),
            _ => unreachable!(),
        }),
    ))
}

fn call<'a, P>(expr: P) -> impl Parser<'a, &'a str, ParseExprKind, Err<Rich<'a, char>>> + Clone
where
    P: Parser<'a, &'a str, ParseExpr, Err<Rich<'a, char>>> + Clone,
{
    identifier()
        .or(operator())
        .padded()
        .then(expr.repeated().collect::<Vec<_>>())
        .delimited_by(just('('), just(')'))
        .map_with(|(name, args), extra| ParseExprKind::Call {
            callee: Box::new(ParseExpr {
                kind: name,
                span: extra.span(),
                id: None,
            }),
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
        .then(just("->").padded().ignore_then(type_keyword()))
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
        .then(just("->").padded().ignore_then(type_keyword()))
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
        let operator = operator();
        let identifier = identifier();
        let boolean = choice((just("true").to(true), just("false").to(false)))
            .map(|b| ParseExprKind::Literal(Literal::Bool(b)));
        let string = just('"')
            .ignore_then(none_of('"').repeated().collect::<String>())
            .then_ignore(just('"'))
            .map(|s| ParseExprKind::Literal(Literal::Str(s)));
        let number = choice((float(), integer()));
        let literal = choice((string, number, boolean));
        let identifier_annotated = identifier_annotated();
        let call = call(expr.clone());
        let define_variable = define_variable(expr.clone());
        let define_function = define_function(expr.clone());
        let define_map = define_map();
        let anonymous_function = anonymous_function(expr.clone());
        let special_form = choice((
            define_variable,
            define_function,
            define_map,
            anonymous_function,
            call,
        ));
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
