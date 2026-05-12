use crate::parsing::{
    Rule,
    ast::{print_error, validation::*},
};
use log::trace;
use pest::{Span, iterators::Pair};

#[derive(Debug)]
pub enum Literal {
    String(String),
    Number { literal: Number },
    Boolean(bool),
}

#[derive(Debug)]
pub enum Number {
    F32(f32),
    F64(f64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
}
impl Number {
    pub fn from_str(s: &str) -> Self {
        if s.contains('.') || s.contains('e') || s.contains('E') {
            if let Ok(f) = s.parse::<f32>() {
                return Number::F32(f);
            }
            return Number::F64(
                s.parse::<f64>()
                    .unwrap_or_else(|_| core::panic!("Invalid float literal: {}", s)),
            );
        }

        if let Ok(v) = s.parse::<u8>() {
            return Number::U8(v);
        }
        if let Ok(v) = s.parse::<u16>() {
            return Number::U16(v);
        }
        if let Ok(v) = s.parse::<u32>() {
            return Number::U32(v);
        }
        if let Ok(v) = s.parse::<u64>() {
            return Number::U64(v);
        }

        if let Ok(v) = s.parse::<i8>() {
            return Number::I8(v);
        }
        if let Ok(v) = s.parse::<i16>() {
            return Number::I16(v);
        }
        if let Ok(v) = s.parse::<i32>() {
            return Number::I32(v);
        }
        if let Ok(v) = s.parse::<i64>() {
            return Number::I64(v);
        }

        if let Ok(f) = s.parse::<f64>() {
            return Number::F64(f);
        }

        core::panic!("Numeric literal out of range or invalid: {}", s);
    }
}

#[derive(Debug)]
pub enum Symbol {
    Typed { name: String, annotation: String },
    Untyped { name: String },
}
impl Symbol {
    pub fn from_pair(pair: &Pair<Rule>) -> Symbol {
        let pair_str = pair.as_str();
        if pair_str.len() < 2 {
            return Symbol::Untyped {
                name: pair_str.to_string(),
            };
        }
        let last_potential_index = pair_str.len() - 1;
        let split_index = pair_str[..last_potential_index].rfind(':');
        match split_index {
            Some(i) if i > 0 => {
                let (name, annotation) = pair_str.split_at(i);
                Self::Typed {
                    name: name.to_string(),
                    annotation: annotation.to_string(),
                }
            }
            _ => Symbol::Untyped {
                name: pair_str.to_string(),
            },
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceInfo<'a> {
    pub line: usize,
    pub col: usize,
    pub span: Span<'a>,
    pub path: &'a str,
}
impl<'a> SourceInfo<'a> {
    pub fn from_pair(pair: &Pair<'a, Rule>, path: &'static str) -> SourceInfo<'a> {
        SourceInfo {
            line: pair.line_col().0,
            col: pair.line_col().1,
            span: pair.as_span(),
            path,
        }
    }
}

#[derive(Debug)]
pub enum Node {
    Fn {
        name: Option<Symbol>,
        params: Vec<Symbol>,
        body: Box<Node>,
    },
    If {
        predicate: Box<Node>,
        yes: Box<Node>,
        no: Option<Box<Node>>,
    },
    Let {
        symbol: Symbol,
        value: Box<Node>,
    },
    For {
        dummy: Symbol,
        iterator: Box<Node>,
        body: Box<Node>,
    },
    Given {
        predicate: Box<Node>,
        cases: Box<Node>,
    },
    Return {
        value: Box<Node>,
    },
    Identifier {
        symbol: Symbol,
    },
    Literal(Literal),
    Call {
        name: Symbol,
        args: Vec<Node>,
    },
    Block {
        expressions: Vec<Node>,
    },
    Invalid,
}
impl Node {
    pub fn from_pair(pair: Pair<Rule>, path: &'static str) -> Node {
        match pair.as_rule() {
            Rule::file => parse_program(pair, path),
            Rule::list => parse_list(pair, path),
            Rule::symbol => Node::Identifier {
                symbol: Symbol::from_pair(&pair),
            },
            Rule::number => Node::Literal(Literal::Number {
                literal: Number::from_str(pair.as_str()),
            }),
            Rule::string => {
                Node::Literal(Literal::String(pair.as_str().trim_matches('"').to_string()))
            }
            Rule::boolean => Node::Literal(Literal::Boolean(pair.as_str().parse().unwrap())),
            _ => {
                print_error("Unexpected syntax", &SourceInfo::from_pair(&pair, path));
                core::panic!("AST construction failed: rule {:?}", pair.as_rule());
            }
        }
    }
}

fn parse_program(pair: Pair<Rule>, path: &'static str) -> Node {
    let mut expressions = Vec::new();
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::EOI => continue,
            _ => expressions.push(Node::from_pair(inner_pair, path)),
        }
    }
    Node::Block { expressions }
}

fn parse_list(pair: Pair<Rule>, path: &'static str) -> Node {
    if !validate_list(&pair, path) {
        return Node::Invalid;
    }
    let mut inner = pair.clone().into_inner();

    if inner.is_empty() {
        return Node::Block {
            expressions: vec![],
        };
    }

    let first_element = inner.next().unwrap();
    if matches!(first_element.as_rule(), Rule::symbol) {
        let first_symbol = Symbol::from_pair(&first_element);
        if let Symbol::Typed {
            ref name,
            annotation: _,
        } = first_symbol
        {
            let name_str = name.as_str();
            if name_str == "fn" {
                return parse_fn(pair, path);
            }
        }
        if let Symbol::Untyped { ref name } = first_symbol {
            let name_str = name.as_str();
            match name_str {
                "if" => return parse_if(pair, path),
                "for" => return parse_for(pair, path),
                "let" => return parse_let(pair, path),
                "given" => return parse_given(pair, path),
                "ret" => return parse_ret(pair, path),
                _ => return parse_call(pair, path),
            }
        }
    }

    let mut expressions = vec![];
    for pair in inner.clone().into_iter() {
        expressions.push(Node::from_pair(pair, path));
    }
    return Node::Block { expressions };
}

fn parse_fn(pair: Pair<Rule>, path: &'static str) -> Node {
    if !validate_fn(&pair, path) {
        return Node::Invalid;
    }
    let mut pairs = pair.clone().into_inner().peekable();
    pairs.next();
    let mut name = None;
    if let Some(p) = pairs.peek() {
        if p.as_rule() == Rule::symbol {
            name = Some(Symbol::from_pair(&pairs.next().unwrap()));
        }
    }
    let params_pair = pairs.next().unwrap_or_else(|| {
        core::panic!("Function missing parameter list at {}", path);
    });
    if !validate_params(&params_pair, path) {
        return Node::Invalid;
    }
    let params = params_pair
        .into_inner()
        .map(|pair| Symbol::from_pair(&pair))
        .collect();
    let body_pair = pairs.next().unwrap_or_else(|| {
        core::panic!("Function missing body at {}", path);
    });
    if !validate_block(&body_pair, path) {
        return Node::Invalid;
    }
    let body = Box::new(block_from_pairs(body_pair.into_inner(), path));
    trace!("Function definition detected");
    Node::Fn { name, params, body }
}

fn parse_if(pair: Pair<Rule>, path: &'static str) -> Node {
    if !validate_if(&pair, path) {
        return Node::Invalid;
    }
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    let predicate = Box::new(Node::from_pair(pairs[1].clone(), path));
    let yes = Box::new(block_from_pairs(pairs[2].clone().into_inner(), path));
    let no = pairs.get(3).map(|pair| {
        Box::new(Node::Block {
            expressions: pair
                .clone()
                .into_inner()
                .map(|pair| Node::from_pair(pair, path))
                .collect(),
        })
    });
    trace!("If statement detected\n{}", pair.as_str());
    Node::If { predicate, yes, no }
}

fn parse_let(pair: Pair<Rule>, path: &'static str) -> Node {
    if !validate_let(&pair, path) {
        print_error("Invalid assignment", &SourceInfo::from_pair(&pair, path));
        return Node::Invalid;
    }
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    let symbol = Symbol::from_pair(&pairs[1]);
    let value = Box::new(block_from_pairs(pairs[2].clone().into_inner(), path));
    trace!("Let statement detected\n{}", pair.as_str());
    Node::Let { symbol, value }
}

fn parse_for(pair: Pair<Rule>, path: &'static str) -> Node {
    if !validate_for(&pair, path) {
        print_error("Invalid for loop", &SourceInfo::from_pair(&pair, path));
        return Node::Invalid;
    }
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    let dummy = Symbol::from_pair(&pairs[1]);
    let iterator = Box::new(block_from_pairs(pairs[2].clone().into_inner(), path));
    let body = Box::new(block_from_pairs(pairs[3].clone().into_inner(), path));
    trace!("For loop detected\n{}", pair.as_str());
    Node::For {
        dummy,
        iterator,
        body,
    }
}

fn parse_given(pair: Pair<Rule>, path: &'static str) -> Node {
    if !validate_given(&pair, path) {
        return Node::Invalid;
    }
    let mut inner = pair.into_inner();
    inner.next();
    let predicate_pair = inner.next().unwrap();
    let predicate = Box::new(Node::from_pair(predicate_pair, path));
    let cases_nodes: Vec<Node> = inner
        .map(|case_pair| {
            let mut case_inner = case_pair.into_inner();
            let pattern = Node::from_pair(case_inner.next().unwrap(), path);
            let body = Node::from_pair(case_inner.next().unwrap(), path);
            Node::Block {
                expressions: vec![pattern, body],
            }
        })
        .collect();
    Node::Given {
        predicate,
        cases: Box::new(Node::Block {
            expressions: cases_nodes,
        }),
    }
}

fn parse_ret(pair: Pair<Rule>, path: &'static str) -> Node {
    if !validate_ret(&pair, path) {
        return Node::Invalid;
    }
    trace!("Return keyword detected\n{}", pair.as_str());
    Node::Return {
        value: Box::new(block_from_pairs(pair.into_inner().skip(1), path)),
    }
}

fn parse_call(pair: Pair<Rule>, path: &'static str) -> Node {
    let mut pairs = pair.clone().into_inner();
    let name_pair = pairs.next().unwrap();
    let name = Symbol::from_pair(&name_pair);
    let args = pairs.map(|p| Node::from_pair(p, path)).collect();
    trace!("Function call detected\n{}", pair.as_str());
    Node::Call { name, args }
}

fn block_from_pairs<'a>(pairs: impl Iterator<Item = Pair<'a, Rule>>, path: &'static str) -> Node {
    Node::Block {
        expressions: pairs.map(|p| Node::from_pair(p, path)).collect(),
    }
}
