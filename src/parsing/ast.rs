use std::collections::HashMap;

use chumsky::span::SimpleSpan;

#[derive(Debug, Clone, PartialEq)]
pub struct ParseExpr {
    pub kind: ParseExprKind,
    pub span: SimpleSpan,
    pub id: Option<usize>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParseExprKind {
    Literal(Literal),
    Identifier(String),
    IdentifierAnnotated((String, Type)),
    IdentifierPlaceholder(String),
    MemberAccess(Vec<ParseExpr>),
    Def {
        name: String,
        type_annotation: Option<Type>,
        value: Box<ParseExpr>,
    },
    Fn {
        params: Vec<Param>,
        return_type: Type,
        body: Box<ParseExpr>,
    },
    Call {
        callee: Box<ParseExpr>,
        args: Vec<ParseExpr>,
    },
    If {
        condition: Box<ParseExpr>,
        then_branch: Box<ParseExpr>,
        else_branch: Option<Box<ParseExpr>>,
    },
    Loop {
        condition: Box<ParseExpr>,
        body: Box<ParseExpr>,
    },
    Block(Vec<ParseExpr>),
    Map {
        params: Vec<Param>,
    },
    Protocol {
        params: Vec<Param>,
        fns: Vec<ParseExpr>,
    },
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Param {
    pub name: String,
    pub type_annotation: Type,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    I32,
    I64,
    U32,
    U64,
    F32,
    F64,
    Str,
    Bool,
    Function {
        params: Vec<Param>,
        return_type: Box<Type>,
    },
    Unit,
    Void,
    Option(Box<Type>),
    Result(Box<Type>),
    Protocol(Box<Type>),
    Code,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Literal {
    Int32(i32),
    Int64(i64),
    Uint32(u32),
    Uint64(u64),
    Float32(f32),
    Float64(f64),
    Str(String),
    Bool(bool),
    Map(HashMap<String, ParseExpr>),
    Quote(Box<ParseExpr>),
    Unit,
}

pub trait ExprVisitor {
    fn visit_expr(&mut self, expr: &ParseExpr, scope_id: usize) {
        self.walk_expr(expr, scope_id);
    }
    fn walk_expr(&mut self, expr: &ParseExpr, scope_id: usize) {
        match &expr.kind {
            ParseExprKind::Identifier(name) => {
                self.visit_identifier(name, expr, scope_id);
            }
            ParseExprKind::IdentifierAnnotated((name, _)) => {
                self.visit_identifier(name, expr, scope_id);
            }
            ParseExprKind::IdentifierPlaceholder(name) => {
                self.visit_identifier(name, expr, scope_id);
            }
            ParseExprKind::MemberAccess(parts) => {
                for part in parts {
                    self.visit_expr(part, scope_id);
                }
            }
            ParseExprKind::Def {
                name,
                type_annotation,
                value,
            } => {
                self.visit_def(name, type_annotation.as_ref(), value, scope_id);
            }
            ParseExprKind::Block(statements) => {
                self.visit_block(statements, expr, scope_id);
            }
            ParseExprKind::Fn { params, body, .. } => {
                self.visit_fn(params, body, expr, scope_id);
            }
            ParseExprKind::Map { params } => {
                self.visit_map(params, expr, scope_id);
            }
            ParseExprKind::Protocol { params, fns } => {
                self.visit_protocol(params, expr, scope_id);
                for f in fns {
                    self.visit_expr(f, scope_id);
                }
            }
            ParseExprKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                self.visit_expr(condition, scope_id);
                self.visit_expr(then_branch, scope_id);
                if let Some(eb) = else_branch {
                    self.visit_expr(eb, scope_id);
                }
            }
            ParseExprKind::Call { callee, args } => {
                self.visit_expr(callee, scope_id);
                for arg in args {
                    self.visit_expr(arg, scope_id);
                }
            }
            ParseExprKind::Loop { condition, body } => {
                self.visit_expr(condition, scope_id);
                self.visit_expr(body, scope_id);
            }
            ParseExprKind::Literal(_) | ParseExprKind::Error => {}
        }
    }
    fn visit_identifier(&mut self, _name: &str, _expr: &ParseExpr, _scope_id: usize) {}
    fn visit_map(&mut self, _params: &[Param], _expr: &ParseExpr, _scope_id: usize) {}
    fn visit_protocol(&mut self, _params: &[Param], _expr: &ParseExpr, _scope_id: usize) {}
    fn visit_def(
        &mut self,
        _name: &str,
        _type_annotation: Option<&Type>,
        value: &ParseExpr,
        scope_id: usize,
    ) {
        self.visit_expr(value, scope_id);
    }
    fn visit_block(&mut self, ast: &[ParseExpr], _expr: &ParseExpr, scope_id: usize) {
        for expr in ast {
            self.visit_expr(expr, scope_id);
        }
    }
    fn visit_fn(
        &mut self,
        _params: &[Param],
        body: &ParseExpr,
        _expr: &ParseExpr,
        scope_id: usize,
    ) {
        self.visit_expr(body, scope_id);
    }
}
