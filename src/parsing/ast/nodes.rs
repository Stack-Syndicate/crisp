use core::panic;

use crate::parsing::{
    Rule,
    ast::{print_error, validation::*},
};
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
pub enum Symbol<'a> {
    Typed {
        name: String,
        annotation: String,
        info: SourceInfo<'a>,
    },
    Untyped {
        name: String,
        info: SourceInfo<'a>,
    },
}
impl<'a> Symbol<'a> {
    pub fn from_pair(pair: &Pair<'a, Rule>, path: &'static str) -> Symbol<'a> {
        match pair.as_rule() {
            Rule::typed_symbol => {
                let mut name_pairs = pair.clone().into_inner();
                let name = name_pairs.next().unwrap().to_string();
                let annotation = name_pairs.next().unwrap().to_string();
                return Symbol::Typed {
                    name,
                    annotation,
                    info: SourceInfo::from_pair(pair, path),
                };
            }
            Rule::symbol => {
                return Symbol::Untyped {
                    name: pair.to_string(),
                    info: SourceInfo::from_pair(pair, path),
                };
            }
            Rule::pointer_ref => {
                return Symbol::Untyped {
                    name: pair.to_string(),
                    info: SourceInfo::from_pair(pair, path),
                };
            }
            _ => {
                panic!("Unexpected rule")
            }
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
pub enum Node<'a> {
    Program {
        expressions: Vec<Node<'a>>,
    },
    Fn {
        name: Option<Symbol<'a>>,
        params: Box<Node<'a>>,
        body: Box<Node<'a>>,
    },
    If {
        predicate: Box<Node<'a>>,
        yes: Box<Node<'a>>,
        no: Option<Box<Node<'a>>>,
    },
    Let {
        name: Symbol<'a>,
        value: Box<Node<'a>>,
    },
    For {
        dummy: Symbol<'a>,
        iterator: Box<Node<'a>>,
        body: Box<Node<'a>>,
    },
    Given {
        predicate: Box<Node<'a>>,
        cases: Box<Node<'a>>,
    },
    Set {
        variable: Symbol<'a>,
        value: Box<Node<'a>>,
    },
    Return {
        value: Box<Node<'a>>,
    },
    Identifier {
        name: Symbol<'a>,
    },
    Reference {
        name: Symbol<'a>,
    },
    Literal(Literal),
    Call {
        identifier: Box<Node<'a>>,
        args: Vec<Node<'a>>,
    },
    Block {
        expressions: Vec<Node<'a>>,
    },
    Params {
        parameters: Vec<Symbol<'a>>,
    },
    Invalid,
}
impl<'a> Node<'a> {
    pub fn from_pair(pair: Pair<'a, Rule>, path: &'static str) -> Node<'a> {
        match pair.as_rule() {
            Rule::file => parse_program(pair, path),
            Rule::list => parse_list(pair, path),
            Rule::symbol | Rule::typed_symbol => Node::Identifier {
                name: Symbol::from_pair(&pair, path),
            },
            Rule::pointer_ref => Node::Reference {
                name: Symbol::from_pair(&pair, path),
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

fn parse_program<'a>(pair: Pair<'a, Rule>, path: &'static str) -> Node<'a> {
    let mut expressions = Vec::new();
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::EOI => continue,
            _ => expressions.push(Node::from_pair(inner_pair, path)),
        }
    }
    Node::Program { expressions }
}

fn parse_list<'a>(pair: Pair<'a, Rule>, path: &'static str) -> Node<'a> {
    let mut inner = pair.clone().into_inner();
    let first_pair = match inner.next() {
        Some(p) => p,
        None => {
            return Node::Block {
                expressions: vec![],
            };
        }
    };
    let name = match first_pair.as_rule() {
        Rule::symbol => first_pair.as_str(),
        _ => "",
    };
    match name {
        "fn" => return parse_fn(pair, path),
        "if" => return parse_if(pair, path),
        "let" => return parse_let(pair, path),
        "for" => return parse_for(pair, path),
        "given" => return parse_given(pair, path),
        "set" => return parse_set(pair, path),
        "ret" => return parse_ret(pair, path),
        _ => return parse_call(pair, path),
    }
}

fn parse_fn<'a>(pair: Pair<'a, Rule>, path: &'static str) -> Node<'a> {
    if !validate_fn(&pair, path) {
        return Node::Invalid;
    }
    let mut pairs = pair.into_inner().peekable();
    pairs.next();
    let mut name = None;
    if let Some(p) = pairs.peek() {
        if p.as_rule() == Rule::symbol {
            name = Some(Symbol::from_pair(&pairs.next().unwrap(), path));
        }
    }
    let params_pair = pairs.next().unwrap_or_else(|| {
        core::panic!("Function missing parameter list at {}", path);
    });
    if !validate_params(&params_pair, path) {
        return Node::Invalid;
    }
    let params = Box::new(parse_params(params_pair, path));
    let body_pair = pairs.next().unwrap_or_else(|| {
        core::panic!("Function missing body at {}", path);
    });
    if !validate_block(&body_pair, path) {
        return Node::Invalid;
    }
    let expressions = body_pair
        .into_inner()
        .map(|p| Node::from_pair(p, path))
        .collect();
    let body = Box::new(Node::Block { expressions });
    Node::Fn { name, params, body }
}

fn parse_if<'a>(pair: Pair<'a, Rule>, path: &'static str) -> Node<'a> {
    if !validate_if(&pair, path) {
        return Node::Invalid;
    }
    let pairs: Vec<Pair<Rule>> = pair.into_inner().collect();
    let predicate = Box::new(Node::from_pair(pairs[1].clone(), path));
    let mut yes_expressions = Vec::new();
    for pair in pairs[2].clone().into_inner() {
        yes_expressions.push(Node::from_pair(pair, path));
    }
    let yes = Box::new(Node::Block {
        expressions: yes_expressions,
    });
    let no = pairs.get(3).map(|pair| {
        Box::new(Node::Block {
            expressions: pair
                .clone()
                .into_inner()
                .map(|pair| Node::from_pair(pair, path))
                .collect(),
        })
    });
    Node::If { predicate, yes, no }
}

fn parse_let<'a>(pair: Pair<'a, Rule>, path: &'static str) -> Node<'a> {
    if !validate_let(&pair, path) {
        print_error("Invalid assignment", &SourceInfo::from_pair(&pair, path));
        return Node::Invalid;
    }
    let pairs: Vec<Pair<Rule>> = pair.into_inner().collect();
    let name = Symbol::from_pair(&pairs[1], path);
    let mut value_expressions = Vec::new();
    for pair in pairs[2].clone().into_inner() {
        value_expressions.push(Node::from_pair(pair, path));
    }
    let value = Box::new(Node::Block {
        expressions: value_expressions,
    });
    Node::Let { name, value }
}

fn parse_for<'a>(pair: Pair<'a, Rule>, path: &'static str) -> Node<'a> {
    if !validate_for(&pair, path) {
        print_error("Invalid for loop", &SourceInfo::from_pair(&pair, path));
        return Node::Invalid;
    }
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    let dummy = Symbol::from_pair(&pairs[1], path);
    let iterator_pair = pairs[2].clone();
    let mut iterator_expressions = Vec::new();
    for pair in iterator_pair.into_inner() {
        iterator_expressions.push(Node::from_pair(pair, path));
    }
    let iterator = Box::new(Node::Block {
        expressions: iterator_expressions,
    });
    let body_pair = pairs[3].clone();
    let mut body_expressions = Vec::new();
    for pair in body_pair.into_inner() {
        body_expressions.push(Node::from_pair(pair, path));
    }
    let body = Box::new(Node::Block {
        expressions: body_expressions,
    });
    Node::For {
        dummy,
        iterator,
        body,
    }
}

fn parse_given<'a>(pair: Pair<'a, Rule>, path: &'static str) -> Node<'a> {
    if !validate_given(&pair, path) {
        // validate_given already prints the specific error, so we just return Invalid
        return Node::Invalid;
    }
    let mut inner = pair.into_inner();
    inner.next(); // Skip the "given" symbol
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

fn parse_set<'a>(pair: Pair<'a, Rule>, path: &'static str) -> Node<'a> {
    if !validate_set(&pair, path) {
        return Node::Invalid;
    }
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    let variable = Symbol::from_pair(&pairs[1], path);
    let mut value_expressions = Vec::new();
    for pair in pairs[2].clone().into_inner() {
        value_expressions.push(Node::from_pair(pair, path));
    }
    let value = Box::new(Node::Block {
        expressions: value_expressions,
    });
    Node::Set { variable, value }
}

fn parse_ret<'a>(pair: Pair<'a, Rule>, path: &'static str) -> Node<'a> {
    if !validate_ret(&pair, path) {
        return Node::Invalid;
    }
    let expressions = pair
        .into_inner()
        .skip(1)
        .map(|pair| Node::from_pair(pair, path))
        .collect();
    Node::Return {
        value: Box::new(Node::Block { expressions }),
    }
}

fn parse_call<'a>(pair: Pair<'a, Rule>, path: &'static str) -> Node<'a> {
    let mut pairs = pair.into_inner();
    let identifier = Box::new(Node::from_pair(pairs.next().unwrap(), path));
    let args = pairs.map(|p| Node::from_pair(p, path)).collect();

    Node::Call { identifier, args }
}

fn parse_block<'a>(pair: Pair<'a, Rule>, path: &'static str) -> Node<'a> {
    if !validate_block(&pair, path) {
        return Node::Invalid;
    }
    let pairs: Vec<Pair<Rule>> = pair.into_inner().collect();
    let mut expressions = Vec::new();
    for pair in pairs {
        expressions.push(Node::from_pair(pair, path));
    }
    Node::Block { expressions }
}

fn parse_params<'a>(pair: Pair<'a, Rule>, path: &'static str) -> Node<'a> {
    let parameters = pair
        .into_inner()
        .map(|p| Symbol::from_pair(&p, path))
        .collect();
    Node::Params { parameters }
}
