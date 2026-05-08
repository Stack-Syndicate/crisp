use crate::{analysis::print_error, parsing::Rule};
use pest::{
    Span,
    iterators::{Pair, Pairs},
};

pub struct Block<'a>(pub Vec<Expr<'a>>);

pub enum Literal<'a> {
    String(String),
    Number {
        literal: Number,
        info: SourceInfo<'a>,
    },
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

#[derive(Debug, Clone)]
pub struct SourceInfo<'a> {
    pub line: usize,
    pub col: usize,
    pub span: Span<'a>,
    pub path: &'a str,
}

pub enum Expr<'a> {
    Program {
        exprs: Block<'a>,
    },
    Fn {
        name: Option<Symbol<'a>>,
        body: Block<'a>,
    },
    If {
        predicate: Box<Expr<'a>>,
        yes: Block<'a>,
        no: Option<Block<'a>>,
    },
    Let {
        name: Symbol<'a>,
        value: Block<'a>,
    },
    For {
        dummy: Symbol<'a>,
        iterator: Block<'a>,
        body: Block<'a>,
    },
    Given {
        predicate: Block<'a>,
        conditions: Block<'a>,
    },
    Set {
        variable: Symbol<'a>,
        value: Block<'a>,
    },
    Return {
        value: Block<'a>,
    },
    Reference {
        name: Symbol<'a>,
    },
    Literal(Literal<'a>),
    Invalid,
}
impl<'a> Expr<'a> {
    pub fn from_pair(pair: Pair<Rule>) -> Expr<'a> {
        match pair.as_rule() {
            Rule::file => parse_program(pair),
            Rule::list => parse_list(pair),
            Rule::symbol => parse_symbol(pair),
            Rule::typed_symbol => parse_typed_symbol(pair),
            Rule::number => parse_number(pair),
            Rule::string => parse_string(pair),
            Rule::pointer_ref => parse_pointer_ref(pair),
            _ => {
                panic!("AST construction failed: unexpected syntax")
            }
        }
    }
}

fn parse_program<'a>(pair: Pair<Rule>) -> Expr<'a> {
    let mut exprs = Block(Vec::new());
    for pair in pair.into_inner() {
        exprs.0.push(Expr::from_pair(pair));
    }
    Expr::Program { exprs }
}

fn parse_list<'a>(pair: Pair<Rule>) -> Expr<'a> {
    let mut pairs: Vec<Pair<Rule>> = pair.into_inner().collect();
    let op = pairs[0].as_str();
    match op {
        "fn" => parse_fn(pair),
        "if" => parse_if(pair),
        "let" => parse_let(pair),
        "for" => parse_for(pair),
        "given" => parse_given(pair),
        "set" => parse_set(pair),
        "ret" => parse_ret(pair),
    }
}

fn parse_fn<'a>(pair: Pair<Rule>) -> Expr<'a> {
    if !validate_fn(pair) {
        print_error(
            "Invalid fn definition",
            SourceInfo {
                line: pair.line_col().0,
                col: pair.line_col().1,
                span: pair.as_span(),
                path: "",
            },
        );
        return Expr::Invalid;
    }
    let pairs: Vec<Pair<Rule>> = pair.into_inner().collect();
    // Fn name
    let mut name = None;
    if matches!(pairs[1].as_rule(), Rule::symbol) {
        name = Some(Symbol::Untyped {
            name: pairs[1].to_string(),
            info: SourceInfo {
                line: pairs[1].line_col().0,
                col: pairs[1].line_col().1,
                span: pairs[1].as_span(),
                path: "",
            },
        })
    }
    // Fn body
    let mut body = Block(Vec::new());
    let body_pairs;
    if name.is_some() {
        body_pairs = &pairs[2..];
    } else {
        body_pairs = &pairs[1..];
    }
    for pair in body_pairs {
        body.0.push(Expr::from_pair(pair.clone()));
    }
    Expr::Fn { name, body }
}

fn parse_if<'a>(pair: Pair<Rule>) -> Expr<'a> {
    if !validate_if(pair) {
        print_error(
            "Invalid if statement",
            SourceInfo {
                line: pair.line_col().0,
                col: pair.line_col().1,
                span: pair.as_span(),
                path: "",
            },
        );
        return Expr::Invalid;
    }
    let pairs: Vec<Pair<Rule>> = pair.into_inner().collect();
    let predicate = Box::new(Expr::from_pair(pairs[1].clone()));
    let mut yes = Block(Vec::new());
    for pair in pairs[2].clone().into_inner() {
        yes.0.push(Expr::from_pair(pair));
    }
    let no;
    if let Some(pair) = pairs.get(3) {
        let mut no_block = Block(Vec::new());
        for pair in pair.into_inner() {
            no_block.0.push(Expr::from_pair(pair));
        }
        no = Some(no_block);
    } else {
        no = None;
    }
    Expr::If { predicate, yes, no }
}

fn parse_let<'a>(pair: Pair<Rule>) -> Expr<'a> {
    if !validate_let(pair) {
        print_error(
            "Invalid assignment",
            SourceInfo {
                line: pair.line_col().0,
                col: pair.line_col().1,
                span: pair.as_span(),
                path: "",
            },
        );
        return Expr::Invalid;
    }

    todo!()
}
