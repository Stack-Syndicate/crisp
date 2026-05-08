use pest::Span;
#[allow(unused)]
use pest::iterators::{Pair, Pairs};

use crate::parsing::{
    Rule,
    types::{List, Number},
};

#[derive(Debug, Clone)]
pub struct SourceInfo<'a> {
    pub line: usize,
    pub col: usize,
    pub span: Span<'a>,
    pub path: &'a str,
}

#[derive(Debug)]
pub enum ASTNode<'a> {
    List(List, SourceInfo<'a>),
    Symbol(String, SourceInfo<'a>),
    TypedSymbol(String, String, SourceInfo<'a>, SourceInfo<'a>),
    Number(Number, SourceInfo<'a>),
    String(String, SourceInfo<'a>),
    Pointer(String, SourceInfo<'a>),
    Boolean(bool, SourceInfo<'a>),
}
impl<'a> ASTNode<'a> {
    pub fn from_pair(pair: Pair<'a, Rule>, path: &'a str) -> Self {
        let line = pair.line_col().0;
        let col = pair.line_col().1;
        match pair.as_rule() {
            Rule::string => ASTNode::String(
                pair.as_str().to_string(),
                SourceInfo {
                    line,
                    col,
                    span: pair.as_span(),
                    path,
                },
            ),
            Rule::symbol => ASTNode::Symbol(
                pair.as_str().to_string(),
                SourceInfo {
                    line,
                    col,
                    span: pair.as_span(),
                    path,
                },
            ),
            Rule::pointer_ref => ASTNode::Pointer(
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
                ASTNode::TypedSymbol(
                    symbol.as_str().to_string(),
                    annotation.as_str().to_string(),
                    SourceInfo {
                        line: symbol.line_col().0,
                        col: symbol.line_col().1,
                        span: symbol.as_span(),
                        path,
                    },
                    SourceInfo {
                        line: annotation.line_col().0,
                        col: annotation.line_col().1,
                        span: annotation.as_span(),
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
                ASTNode::Number(
                    n,
                    SourceInfo {
                        line,
                        col,
                        span: pair.as_span(),
                        path,
                    },
                )
            }
            Rule::boolean => ASTNode::Boolean(
                pair.as_str().parse().expect("Could not parse boolean."),
                SourceInfo {
                    line,
                    col,
                    span: pair.as_span(),
                    path,
                },
            ),
            // Lists are just a Vec of Nodes.... OR ARE THEY?
            Rule::file | Rule::list => {
                let mut list = Vec::new();
                let pairs = pair.clone().into_inner();
                for pair in pairs {
                    list.push(ASTNode::from_pair(pair, path));
                }
                ASTNode::List(
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
            ASTNode::List(_, info)
            | ASTNode::Symbol(_, info)
            | ASTNode::TypedSymbol(_, _, info, _)
            | ASTNode::Number(_, info)
            | ASTNode::String(_, info)
            | ASTNode::Pointer(_, info)
            | ASTNode::Boolean(_, info) => info,
        }
    }
}

pub fn cst_to_ast<'a>(pairs: Pairs<'a, Rule>, path: &'a str) -> Vec<ASTNode<'a>> {
    pairs
        .filter(|pair| pair.as_rule() != Rule::EOI)
        .map(|pair| ASTNode::from_pair(pair, path))
        .collect()
}
