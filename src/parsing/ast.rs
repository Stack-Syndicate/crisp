#[allow(unused)]
use pest::iterators::{Pair, Pairs};

use crate::parsing::{Rule, types::Number};

#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug)]
pub enum Node {
    List(Vec<Node>, SourceInfo),
    Symbol(String, SourceInfo),
    TypedSymbol(String, String, SourceInfo),
    Number(Number, SourceInfo),
    String(String, SourceInfo),
    Pointer(String, SourceInfo),
    Boolean(bool, SourceInfo),
}
impl Node {
    pub fn from_pair(pair: Pair<Rule>) -> Self {
        let line = pair.line_col().0;
        let col = pair.line_col().1;
        match pair.as_rule() {
            Rule::string => Node::String(pair.as_str().to_string(), SourceInfo { line, col }),
            Rule::symbol => Node::Symbol(pair.as_str().to_string(), SourceInfo { line, col }),
            Rule::pointer_ref => Node::Pointer(pair.as_str().to_string(), SourceInfo { line, col }),
            Rule::typed_symbol => {
                let mut symbol_and_type = pair.into_inner();
                let symbol = symbol_and_type.next().unwrap();
                let annotation = symbol_and_type.next().unwrap();
                Node::TypedSymbol(
                    symbol.as_str().to_string(),
                    annotation.as_str().to_string(),
                    SourceInfo { line, col },
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
                Node::Number(n, SourceInfo { line, col })
            }
            Rule::boolean => Node::Boolean(
                pair.as_str().parse().expect("Could not parse boolean."),
                SourceInfo { line, col },
            ),
            // Lists are just a Vec of Nodes
            Rule::file | Rule::list => {
                let mut list = Vec::new();
                let pairs = pair.into_inner();
                for pair in pairs {
                    list.push(Node::from_pair(pair));
                }
                Node::List(list, SourceInfo { line, col })
            }
            _ => panic!("Unexpected pair {:?}", pair),
        }
    }
    pub fn get_info(&self) -> &SourceInfo {
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

pub fn pest_to_ast(pairs: Pairs<Rule>) -> Vec<Node> {
    pairs
        .filter(|pair| pair.as_rule() != Rule::EOI)
        .map(Node::from_pair)
        .collect()
}
