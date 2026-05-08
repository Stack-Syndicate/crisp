use crate::parsing::ast::ASTNode;

#[allow(unused)]
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

#[derive(Debug)]
pub enum List {
    FN {
        name: Symbol,
        params: Params,
        body: List,
    },
    GIVEN {
        predicate: List,
        conditions: Vec<List>,
    },
    IF {
        predicate: List,
        then: List,
        r#else: Option<List>,
    },
    FOR {
        dummy: Symbol,
        iterator: Symbol,
        body: List,
    },
    SET {
        name: Symbol,
        value: List,
    },
    LET {
        name: Symbol,
        value: List,
    },
    RAW {
        items: Vec<List>,
    },
}

#[derive(Debug)]
pub enum Symbol {
    TYPED { name: String, annotation: String },
    UNTYPED { name: String },
}

pub type Params = Vec<Symbol>;
