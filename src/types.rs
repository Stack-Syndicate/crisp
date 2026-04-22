#![allow(dead_code)]

#[derive(Clone, Debug)]
pub enum Type {
    Primitive(Value),
    Structure(Structure),
}

#[derive(Clone, Debug)]
pub enum Value {
    Byte(u8),
    I32(i32),
    I64(i64),
    U32(u32),
    U64(u64),
    F32(f32),
    F64(f64),
    STR(String),
}

#[derive(Clone, Debug)]
pub enum Structure {
    Symbol(String),
    TypedSymbol {
        identifier: String,
        annotation: String,
    },
    List(Vec<Type>),
}

