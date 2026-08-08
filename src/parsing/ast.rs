#[derive(Debug, Clone)]
pub enum ParseExpr {
    Literal(Literal),
    Identifier(String),
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
