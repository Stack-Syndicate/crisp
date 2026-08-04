/// - [x] Variables
/// - [x] Functions
/// - [ ] Loops (while)
pub mod ast;
use crate::ast::{Expr, Literal, Param, Type};
use chumsky::prelude::*;

pub fn crip_parser<'a>() -> impl Parser<'a, &'a str, Vec<Expr>> {
    // parse expressions
    let expr = recursive(|expr| {
        // reserved keywords
        let reserved = [
            "def", "fn", "i32", "i64", "f32", "f64", "u32", "u64", "bool", "loop",
        ];
        // built-in operators
        let operator = one_of("+-*/=<>!&|")
            .repeated()
            .at_least(1)
            .collect::<String>(); // generic identifiers (function/variable names and the like)
        let identifier = text::ident()
            .map(String::from)
            .or(operator)
            .filter(move |s| !reserved.contains(&s.as_str()))
            .map(Expr::Identifier); // text or numerical data typed in at comptime
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
        // keywords referring to types (type hints etc)
        let type_keyword = choice((
            just("i32").to(Type::I32),
            just("i64").to(Type::I64),
            just("f32").to(Type::F32),
            just("f64").to(Type::F64),
            just("u32").to(Type::U32),
            just("u64").to(Type::U64),
            just("bool").to(Type::Bool),
        ));
        // specific type hint form (: i32)
        let type_annotation = just(':').padded().ignore_then(type_keyword.clone());
        // identifier with a type hint (abc: i32)
        let typed_identifier = text::ident()
            .map(String::from)
            .then(type_annotation.clone());
        // syntactic forms with specific functionality
        let special_forms = {
            // using the def keywords to define a new variable or function
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
            // parameter list for functions; basically a bunch of annotated identifiers
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
            // function return type, self explanatory
            let func_return_type = just("->").padded().ignore_then(type_keyword.clone());
            // annoymous function definitions
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
            // while condition is true do body
            let while_loop = just("loop")
                .padded()
                .ignore_then(expr.clone())
                .then(expr.clone())
                .delimited_by(just('(').padded(), just(")").padded())
                .map(|(condition, body)| Expr::Loop {
                    condition: Box::new(condition),
                    body: Box::new(body),
                });
            choice((define, func, while_loop))
        };
        // creation of a new scope; important for certain special forms
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
