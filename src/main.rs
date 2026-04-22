use pest::Parser;
use pest_derive::Parser;

use crate::types::{
    Structure::*,
    Type::{self, *},
    Value::*,
};

pub(crate) mod types;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CrispParser;

fn parse_value(pair: pest::iterators::Pair<Rule>) -> Type {
    match pair.as_rule() {
        Rule::list => {
            let items = pair.into_inner().map(parse_value).collect();
            Structure(List(items))
        }
        Rule::typed_symbol => {
            let mut inner = pair.into_inner();
            let identifier = inner.next().unwrap().as_str().to_string();
            let annotation = inner
                .next()
                .unwrap()
                .into_inner()
                .next()
                .unwrap()
                .as_str()
                .to_string();
            Structure(TypedSymbol {
                identifier,
                annotation,
            })
        }
        Rule::symbol => Structure(Symbol(pair.as_str().to_string())),
        Rule::number => {
            let s = pair.as_str();
            let value = if let Ok(u) = s.parse::<u32>() {
                U32(u)
            } else if let Ok(i) = s.parse::<i32>() {
                I32(i)
            } else if let Ok(u64_val) = s.parse::<u64>() {
                U64(u64_val)
            } else if let Ok(i64_val) = s.parse::<i64>() {
                I64(i64_val)
            } else {
                let f = s.parse::<f64>().unwrap_or(0.0);
                if f.abs() <= f32::MAX as f64 && f.abs() >= f32::MIN_POSITIVE as f64 || f == 0.0 {
                    F32(f as f32)
                } else {
                    F64(f)
                }
            };
            Primitive(value)
        }
        Rule::string => {
            let raw = pair.as_str();
            Primitive(STR(raw[1..raw.len() - 1].to_string()))
        }
        _ => unreachable!("Unexpected rule: {:?}", pair.as_rule()),
    }
}

fn main() {
    let input = "(defun add-one (x: int) (+ x 1))";
    let successful_parse = CrispParser::parse(Rule::file, input);
    let ast: Vec<Type>;
    match successful_parse {
        Ok(mut pairs) => {
            let file_pair = pairs.next().unwrap();
            ast = file_pair
                .into_inner()
                .filter(|pair| pair.as_rule() != Rule::EOI) // Skip the end of input
                .map(parse_value)
                .collect();
            println!("{:#?}", ast);
        }
        Err(e) => eprintln!("Parse failed: {}", e),
    }
}
