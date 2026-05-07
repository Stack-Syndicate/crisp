use pest::Span;
#[allow(unused)]
use pest::iterators::{Pair, Pairs};

use crate::parsing::{Rule, types::Number};

#[derive(Debug, Clone)]
pub struct SourceInfo<'a> {
    pub line: usize,
    pub col: usize,
    pub span: Span<'a>,
    pub path: &'a str,
}

#[derive(Debug)]
pub enum Node<'a> {
    List(Vec<Node<'a>>, SourceInfo<'a>),
    Symbol(String, SourceInfo<'a>),
    TypedSymbol(String, String, SourceInfo<'a>),
    Number(Number, SourceInfo<'a>),
    String(String, SourceInfo<'a>),
    Pointer(String, SourceInfo<'a>),
    Boolean(bool, SourceInfo<'a>),
}
impl<'a> Node<'a> {
    pub fn from_pair(pair: Pair<'a, Rule>, path: &'a str) -> Self {
        let line = pair.line_col().0;
        let col = pair.line_col().1;
        match pair.as_rule() {
            Rule::string => Node::String(
                pair.as_str().to_string(),
                SourceInfo {
                    line,
                    col,
                    span: pair.as_span(),
                    path,
                },
            ),
            Rule::symbol => Node::Symbol(
                pair.as_str().to_string(),
                SourceInfo {
                    line,
                    col,
                    span: pair.as_span(),
                    path,
                },
            ),
            Rule::pointer_ref => Node::Pointer(
                pair.as_str().to_string(),
                SourceInfo {
                    line,
                    col,
                    span: pair.as_span(),
                    path,
                },
            ),
            Rule::typed_symbol => {
                let mut symbol_and_type = pair.clone().into_inner();
                let symbol = symbol_and_type.next().unwrap();
                let annotation = symbol_and_type.next().unwrap();
                Node::TypedSymbol(
                    symbol.as_str().to_string(),
                    annotation.as_str().to_string(),
                    SourceInfo {
                        line,
                        col,
                        span: pair.as_span(),
                        path,
                    },
                )
            }
            // Number literals should occupy the least number of bits
            Rule::number => {
                let s = pair.as_str();
                let n = if s.contains('.') || s.contains('e') || s.contains('E') {
                    let val_f64 = s.parse::<f64>().expect("Invalid float");
                    let val_f32 = val_f64 as f32;
                    if val_f32 as f64 == val_f64 {
                        Number::F32(val_f32)
                    } else {
                        Number::F64(val_f64)
                    }
                } else if !s.starts_with('-') {
                    if let Ok(v) = s.parse::<u8>() {
                        Number::U8(v)
                    } else if let Ok(v) = s.parse::<u16>() {
                        Number::U16(v)
                    } else if let Ok(v) = s.parse::<u32>() {
                        Number::U32(v)
                    } else if let Ok(v) = s.parse::<u64>() {
                        Number::U64(v)
                    } else {
                        Number::I64(s.parse::<i64>().expect("Value out of range"))
                    }
                } else {
                    if let Ok(v) = s.parse::<i8>() {
                        Number::I8(v)
                    } else if let Ok(v) = s.parse::<i16>() {
                        Number::I16(v)
                    } else if let Ok(v) = s.parse::<i32>() {
                        Number::I32(v)
                    } else {
                        Number::I64(s.parse::<i64>().expect("Value out of range"))
                    }
                };
                Node::Number(
                    n,
                    SourceInfo {
                        line,
                        col,
                        span: pair.as_span(),
                        path,
                    },
                )
            }
            Rule::boolean => Node::Boolean(
                pair.as_str().parse().expect("Could not parse boolean."),
                SourceInfo {
                    line,
                    col,
                    span: pair.as_span(),
                    path,
                },
            ),
            // Lists are just a Vec of Nodes
            Rule::file | Rule::list => {
                let mut list = Vec::new();
                let pairs = pair.clone().into_inner();
                for pair in pairs {
                    list.push(Node::from_pair(pair, path));
                }
                Node::List(
                    list,
                    SourceInfo {
                        line,
                        col,
                        span: pair.as_span(),
                        path,
                    },
                )
            }
            _ => panic!("Unexpected pair {:?}", pair),
        }
    }
    pub fn get_info(&self) -> &SourceInfo<'a> {
        match self {
            Node::List(_, info)
            | Node::Symbol(_, info)
            | Node::TypedSymbol(_, _, info)
            | Node::Number(_, info)
            | Node::String(_, info)
            | Node::Pointer(_, info)
            | Node::Boolean(_, info) => info,
        }
    }
}

pub fn pest_to_ast<'a>(pairs: Pairs<'a, Rule>, path: &'a str) -> Vec<Node<'a>> {
    pairs
        .filter(|pair| pair.as_rule() != Rule::EOI)
        .map(|pair| Node::from_pair(pair, path))
        .collect()
}
