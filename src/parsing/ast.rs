#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Identifier(String),
    Def {
        name: String,
        type_annotation: Option<Type>,
        value: Box<Expr>,
    },
    Fn {
        params: Vec<Param>,
        return_type: Type,
        body: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    If {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Option<Box<Expr>>,
    },
    Loop {
        condition: Box<Expr>,
        body: Box<Expr>,
    },
    Block(Vec<Expr>),
    Error,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_annotation: Type,
}

#[derive(Debug, Clone, PartialEq)]
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
        params: Vec<Type>,
        return_type: Box<Type>,
    },
    Unit,
}

#[derive(Debug, Clone)]
pub enum Literal {
    Int32(i32),
    Int64(i64),
    Uint32(u32),
    Uint64(u64),
    Float32(f32),
    Float64(f64),
    Str(String),
    Bool(bool),
    Unit,
}
