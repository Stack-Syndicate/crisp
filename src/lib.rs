pub mod ast;

use chumsky::prelude::*;

use crate::ast::{Expr, Literal, Param, Type};

pub fn crip_parser<'a>() -> impl Parser<'a, &'a str, Vec<Expr>> {
    let expr = recursive(|expr| {
        let reserved = [
            "def", "fn", "i32", "i64", "f32", "f64", "u32", "u64", "bool",
        ];
        let identifier = text::ident()
            .map(String::from)
            .filter(move |s| !reserved.contains(&s.as_str()))
            .map(Expr::Identifier);
        let literal = {
            let string = just('"')
                .ignore_then(none_of("\"").repeated().collect::<String>())
                .then_ignore(just('"'))
                .map(|s| Expr::Literal(Literal::Str(s)));
            let number = {
                let integer = text::digits(10)
                    .to_slice()
                    .map(|s: &str| Expr::Literal(Literal::Int32(s.parse::<i32>().unwrap())));
                let float = text::digits(10)
                    .then(just('.'))
                    .then(text::digits(10).or_not())
                    .to_slice()
                    .map(|s: &str| s.parse::<f32>().unwrap())
                    .map(|f| Expr::Literal(Literal::Float32(f)));
                choice((float, integer))
            };
            choice((string, number))
        };
        let type_keyword = choice((
            just("i32").to(Type::I32),
            just("i64").to(Type::I64),
            just("f32").to(Type::F32),
            just("f64").to(Type::F64),
            just("u32").to(Type::U32),
            just("u64").to(Type::U64),
            just("bool").to(Type::Bool),
        ));
        let type_annotation = just(':').padded().ignore_then(type_keyword.clone());
        let typed_identifier = text::ident()
            .map(String::from)
            .then(type_annotation.clone());
        let special_forms = {
            let define = just("def")
                .padded()
                .ignore_then(text::ident().map(String::from))
                .then(type_annotation.or_not())
                .then(expr.clone().padded())
                .delimited_by(just('(').padded(), just(')').padded())
                .map(|((name, type_annotation), value)| Expr::Def {
                    name,
                    type_annotation,
                    value: Box::new(value),
                });
            let func_params = typed_identifier
                .padded()
                .separated_by(just(',').padded())
                .collect::<Vec<_>>()
                .delimited_by(just('[').padded(), just(']').padded())
                .map(|params| {
                    params
                        .into_iter()
                        .map(|(name, type_annotation)| Param {
                            name,
                            type_annotation,
                        })
                        .collect::<Vec<Param>>()
                });
            let func_return_type = just("->").padded().ignore_then(type_keyword.clone());
            let func = just("fn")
                .padded()
                .ignore_then(func_params)
                .then(func_return_type)
                .then(expr.clone().padded())
                .delimited_by(just('(').padded(), just(')').padded())
                .map(|((params, return_type), value)| Expr::Fn {
                    params,
                    return_type,
                    body: Box::new(value),
                });
            choice((define, func))
        };
        let block = expr
            .clone()
            .repeated()
            .collect::<Vec<_>>()
            .delimited_by(just("("), just(")"))
            .map(Expr::Block);
        choice((special_forms, block, literal, identifier)).padded()
    });
    expr.repeated().collect()
}
