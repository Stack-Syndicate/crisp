use crate::{diagnostics::print_error, parsing::SourceFile};
use std::{str::FromStr, sync::Arc};

use crate::parsing::{ast::Rule, ast::validation::*};
use log::trace;
use pest::iterators::Pair;

#[derive(Debug)]
pub enum Literal {
    String(String),
    Number { literal: Number },
    Boolean(bool),
}
impl Literal {
    pub fn to_test_string(&self) -> String {
        match self {
            Literal::String(s) => format!("String({:?})", s),
            Literal::Number { literal } => literal.to_test_string(),
            Literal::Boolean(b) => format!("Boolean({})", b),
        }
    }
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
impl FromStr for Number {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, String> {
        if s.contains('.') || s.contains('e') || s.contains('E') {
            if let Ok(f) = s.parse::<f32>() {
                return Ok(Number::F32(f));
            }
            return Ok(Number::F64(
                s.parse::<f64>()
                    .unwrap_or_else(|_| core::panic!("Invalid float literal: {}", s)),
            ));
        }

        if let Ok(v) = s.parse::<u8>() {
            return Ok(Number::U8(v));
        }
        if let Ok(v) = s.parse::<u16>() {
            return Ok(Number::U16(v));
        }
        if let Ok(v) = s.parse::<u32>() {
            return Ok(Number::U32(v));
        }
        if let Ok(v) = s.parse::<u64>() {
            return Ok(Number::U64(v));
        }

        if let Ok(v) = s.parse::<i8>() {
            return Ok(Number::I8(v));
        }
        if let Ok(v) = s.parse::<i16>() {
            return Ok(Number::I16(v));
        }
        if let Ok(v) = s.parse::<i32>() {
            return Ok(Number::I32(v));
        }
        if let Ok(v) = s.parse::<i64>() {
            return Ok(Number::I64(v));
        }

        if let Ok(f) = s.parse::<f64>() {
            return Ok(Number::F64(f));
        }

        core::panic!("Numeric literal out of range or invalid: {}", s);
    }
}
impl Number {
    pub fn to_test_string(&self) -> String {
        match self {
            Number::F32(v) => format!("F32({})", v),
            Number::F64(v) => format!("F64({})", v),
            Number::I8(v) => format!("I8({})", v),
            Number::I16(v) => format!("I16({})", v),
            Number::I32(v) => format!("I32({})", v),
            Number::I64(v) => format!("I64({})", v),
            Number::U8(v) => format!("U8({})", v),
            Number::U16(v) => format!("U16({})", v),
            Number::U32(v) => format!("U32({})", v),
            Number::U64(v) => format!("U64({})", v),
        }
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
    pub fn get_name(&self) -> String {
        match self {
            Symbol::Untyped { name } => name.clone(),
            Symbol::Typed { name, .. } => name.clone(),
        }
    }
    pub fn get_annotation(&self) -> Option<String> {
        match self {
            Symbol::Untyped { .. } => None,
            Symbol::Typed { annotation, .. } => {
                Some(annotation.to_string().trim_start_matches(":").to_string())
            }
        }
    }
    pub fn to_test_string(&self) -> String {
        match self {
            Symbol::Typed { name, annotation } => {
                let clean_type = annotation.trim_start_matches(':');
                format!("{}:{}", name, clean_type)
            }
            Symbol::Untyped { name } => name.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SourceInfo {
    pub line: usize,
    pub col: usize,
    pub start: usize,
    pub end: usize,
    pub file: Arc<SourceFile>,
}
impl SourceInfo {
    pub fn from_pair(pair: &Pair<Rule>, source: &SourceFile) -> SourceInfo {
        let span = pair.as_span();
        SourceInfo {
            line: pair.line_col().0,
            col: pair.line_col().1,
            start: span.start(),
            end: span.end(),
            file: Arc::new(source.clone()),
        }
    }
}

#[derive(Debug)]
pub enum Node {
    Fn {
        name: Option<Symbol>,
        return_type: Symbol,
        params: Vec<Symbol>,
        body: Box<Node>,
        info: SourceInfo,
    },
    If {
        predicate: Box<Node>,
        yes: Box<Node>,
        no: Option<Box<Node>>,
        info: SourceInfo,
    },
    Let {
        symbol: Symbol,
        value: Box<Node>,
        info: SourceInfo,
    },
    For {
        dummy: Symbol,
        iterator: Box<Node>,
        body: Box<Node>,
        info: SourceInfo,
    },
    Given {
        predicate: Box<Node>,
        cases: Box<Node>,
        info: SourceInfo,
    },
    Return {
        value: Box<Node>,
        info: SourceInfo,
    },
    Identifier {
        symbol: Symbol,
        info: SourceInfo,
    },
    Literal {
        inner: Literal,
        info: SourceInfo,
    },
    Call {
        name: Symbol,
        args: Vec<Node>,
        info: SourceInfo,
    },
    Block {
        expressions: Vec<Node>,
        info: SourceInfo,
    },
    Invalid,
}
impl Node {
    pub fn from_pair(pair: Pair<Rule>, source: &SourceFile) -> Node {
        let info = SourceInfo::from_pair(&pair, source);
        match pair.as_rule() {
            Rule::file => parse_program(pair, source),
            Rule::list => parse_list(pair, source),
            Rule::symbol => Node::Identifier {
                symbol: Symbol::from_pair(&pair),
                info,
            },
            Rule::number => Node::Literal {
                inner: Literal::Number {
                    literal: Number::from_str(pair.as_str())
                        .expect("Could not parse string as a Number type"),
                },
                info,
            },
            Rule::string => Node::Literal {
                inner: Literal::String(pair.as_str().trim_matches('"').to_string()),
                info,
            },
            Rule::boolean => Node::Literal {
                inner: Literal::Boolean(pair.as_str().parse().unwrap()),
                info,
            },
            _ => {
                print_error("Unexpected syntax", &SourceInfo::from_pair(&pair, source));
                core::panic!("AST construction failed: rule {:?}", pair.as_rule());
            }
        }
    }
    pub fn to_test_string(&self) -> String {
        self.to_test_string_impl(0)
    }
    fn to_test_string_impl(&self, level: usize) -> String {
        let indent = "\t".repeat(level);
        let next_indent = "\t".repeat(level + 1);
        match self {
            Node::Literal { inner, .. } => format!("Literal({})", inner.to_test_string()),
            Node::Identifier { symbol, .. } => format!("Identifier({})", symbol.to_test_string()),
            Node::Let { symbol, value, .. } => {
                format!(
                    "Let({}, {})",
                    symbol.to_test_string(),
                    value.to_test_string_impl(level)
                )
            }
            Node::Block { expressions, .. } => {
                if expressions.is_empty() {
                    return "Block([])".to_string();
                }
                let exprs: Vec<String> = expressions
                    .iter()
                    .map(|e| format!("{}{}", next_indent, e.to_test_string_impl(level + 1)))
                    .collect();
                format!("Block([\n{}\n{}])", exprs.join(",\n"), indent)
            }
            Node::If {
                predicate, yes, no, ..
            } => {
                let no_str = match no {
                    Some(no_node) => format!(
                        ",\n{}Some({})",
                        next_indent,
                        no_node.to_test_string_impl(level + 1)
                    ),
                    None => format!(",\n{}None", next_indent),
                };
                format!(
                    "If(\n{}{},\n{}{}{}\n{})",
                    next_indent,
                    predicate.to_test_string_impl(level + 1),
                    next_indent,
                    yes.to_test_string_impl(level + 1),
                    no_str,
                    indent
                )
            }
            Node::Call { name, args, .. } => {
                if args.is_empty() {
                    return format!("Call({}, [])", name.to_test_string());
                }
                let args_str: Vec<String> =
                    args.iter().map(|e| e.to_test_string_impl(level)).collect();
                format!("Call({}, [{}])", name.to_test_string(), args_str.join(", "))
            }
            Node::Fn {
                name,
                return_type,
                params,
                body,
                ..
            } => {
                let param_names: Vec<String> = params.iter().map(|p| p.get_name()).collect();
                let param_types: Vec<Option<String>> =
                    params.iter().map(|p| p.get_annotation()).collect();
                let mut param_strings = Vec::new();
                for (name, annotation) in param_names.iter().zip(param_types) {
                    let mut param_string = String::new();
                    param_string += format!("{} {}", name, annotation.unwrap()).as_str();
                    param_strings.push(param_string);
                }
                let fn_name = match name {
                    Some(symbol) => symbol.get_name(),
                    None => "anon".to_string(),
                };
                format!(
                    "Fn({}->{},\n{}Params([{}]),\n{}{}\n{})",
                    fn_name,
                    return_type.to_test_string(),
                    next_indent,
                    param_strings.join(", "),
                    next_indent,
                    body.to_test_string_impl(level + 1),
                    indent
                )
            }
            Node::Invalid => "Invalid".to_string(),
            Node::Return { value, info } => {
                format!("Return({})", value.to_test_string())
            }
            other => format!("{:?}", other),
        }
    }
}

fn parse_program(pair: Pair<Rule>, source: &SourceFile) -> Node {
    let info = SourceInfo::from_pair(&pair, source);
    let mut expressions = Vec::new();
    for inner_pair in pair.into_inner() {
        match inner_pair.as_rule() {
            Rule::EOI => continue,
            _ => expressions.push(Node::from_pair(inner_pair, source)),
        }
    }
    Node::Block { expressions, info }
}

fn parse_list(pair: Pair<Rule>, source: &SourceFile) -> Node {
    if !validate_list(&pair, source) {
        return Node::Invalid;
    }

    let info = SourceInfo::from_pair(&pair, source);
    let inner = pair.clone().into_inner();
    if inner.is_empty() {
        return Node::Block {
            expressions: vec![],
            info,
        };
    }

    let mut special_form_inner = pair.clone().into_inner();
    let first_element = special_form_inner.next().unwrap();

    if matches!(first_element.as_rule(), Rule::symbol) {
        let first_symbol = Symbol::from_pair(&first_element);
        if let Symbol::Typed {
            ref name,
            annotation: _,
        } = first_symbol
        {
            let name_str = name.as_str();
            if name_str == "fn" {
                return parse_fn(pair, source);
            }
        }
        if let Symbol::Untyped { ref name } = first_symbol {
            let name_str = name.as_str();
            match name_str {
                "if" => return parse_if(pair, source),
                "for" => return parse_for(pair, source),
                "let" => return parse_let(pair, source),
                "given" => return parse_given(pair, source),
                "ret" => return parse_ret(pair, source),
                _ => return parse_call(pair, source),
            }
        }
    }

    let mut expressions = vec![];
    for p in inner {
        expressions.push(Node::from_pair(p, source));
    }
    Node::Block { expressions, info }
}
fn parse_fn(pair: Pair<Rule>, source: &SourceFile) -> Node {
    if !validate_fn(&pair, source) {
        return Node::Invalid;
    }
    let info = SourceInfo::from_pair(&pair, source);
    let mut pairs = pair.clone().into_inner().peekable();
    let fn_op = pairs.next().unwrap();
    let return_type = Symbol::Untyped {
        name: Symbol::from_pair(&fn_op).get_annotation().unwrap(),
    };
    let mut name = None;
    if let Some(p) = pairs.peek()
        && p.as_rule() == Rule::symbol
    {
        name = Some(Symbol::from_pair(&pairs.next().unwrap()));
    }
    let params_pair = pairs.next().unwrap_or_else(|| {
        core::panic!("Function missing parameter list at {:?}", source);
    });
    if !validate_params(&params_pair, source) {
        return Node::Invalid;
    }
    let params = params_pair
        .into_inner()
        .map(|pair| Symbol::from_pair(&pair))
        .collect();
    let body_pair = pairs.next().unwrap_or_else(|| {
        core::panic!("Function missing body at {:?}", source);
    });
    if !validate_block(&body_pair, source) {
        return Node::Invalid;
    }
    let body = Box::new(Node::from_pair(body_pair, source));
    trace!("Function definition detected");
    Node::Fn {
        name,
        return_type,
        params,
        body,
        info,
    }
}

fn parse_if(pair: Pair<Rule>, source: &SourceFile) -> Node {
    if !validate_if(&pair, source) {
        return Node::Invalid;
    }
    let info = SourceInfo::from_pair(&pair, source);
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    let predicate = Box::new(Node::from_pair(pairs[1].clone(), source));
    let yes = Box::new(Node::from_pair(pairs[2].clone(), source));
    let no = pairs
        .get(3)
        .map(|pair| Box::new(Node::from_pair(pair.clone(), source)));
    trace!("If statement detected\n{}", pair.as_str());
    Node::If {
        predicate,
        yes,
        no,
        info,
    }
}

fn parse_let(pair: Pair<Rule>, source: &SourceFile) -> Node {
    if !validate_let(&pair, source) {
        print_error("Invalid assignment", &SourceInfo::from_pair(&pair, source));
        return Node::Invalid;
    }
    let info = SourceInfo::from_pair(&pair, source);
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    let symbol = Symbol::from_pair(&pairs[1]);
    let value = Box::new(Node::from_pair(pairs[2].clone(), source));
    trace!("Let statement detected\n{}", pair.as_str());
    Node::Let {
        symbol,
        value,
        info,
    }
}

fn parse_for(pair: Pair<Rule>, source: &SourceFile) -> Node {
    if !validate_for(&pair, source) {
        print_error("Invalid for loop", &SourceInfo::from_pair(&pair, source));
        return Node::Invalid;
    }
    let info = SourceInfo::from_pair(&pair, source);
    let pairs: Vec<Pair<Rule>> = pair.clone().into_inner().collect();
    let dummy = Symbol::from_pair(&pairs[1]);
    let iterator = Box::new(Node::from_pair(pairs[2].clone(), source));
    let body = Box::new(Node::from_pair(pairs[3].clone(), source));
    trace!("For loop detected\n{}", pair.as_str());
    Node::For {
        dummy,
        iterator,
        body,
        info,
    }
}

fn parse_given(pair: Pair<Rule>, source: &SourceFile) -> Node {
    if !validate_given(&pair, source) {
        return Node::Invalid;
    }
    let info = SourceInfo::from_pair(&pair, source);
    let mut inner = pair.into_inner();
    inner.next();
    let predicate_pair = inner.next().unwrap();
    let predicate = Box::new(Node::from_pair(predicate_pair, source));
    let cases_nodes: Vec<Node> = inner
        .map(|case_pair| {
            let info = SourceInfo::from_pair(&case_pair, source);
            let mut case_inner = case_pair.into_inner();
            let pattern = Node::from_pair(case_inner.next().unwrap(), source);
            let body = Node::from_pair(case_inner.next().unwrap(), source);
            Node::Block {
                expressions: vec![pattern, body],
                info,
            }
        })
        .collect();
    Node::Given {
        predicate,
        cases: Box::new(Node::Block {
            expressions: cases_nodes,
            info: info.clone(),
        }),
        info,
    }
}

fn parse_ret(pair: Pair<Rule>, source: &SourceFile) -> Node {
    if !validate_ret(&pair, source) {
        return Node::Invalid;
    }
    trace!("Return keyword detected\n{}", pair.as_str());
    let pairs = pair.clone().into_inner().collect::<Vec<_>>();
    let info = SourceInfo::from_pair(&pair, source);
    Node::Return {
        value: Box::new(Node::from_pair(pairs[1].clone(), source)),
        info,
    }
}

fn parse_call(pair: Pair<Rule>, source: &SourceFile) -> Node {
    let mut pairs = pair.clone().into_inner();
    let name_pair = pairs.next().unwrap();
    let name = Symbol::from_pair(&name_pair);
    let args = pairs.map(|p| Node::from_pair(p, source)).collect();
    let info = SourceInfo::from_pair(&pair, source);
    trace!("Function call detected\n{}", pair.as_str());
    Node::Call { name, args, info }
}
