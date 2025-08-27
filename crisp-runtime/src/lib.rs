pub mod parsing;
use num_iter::{range, range_step};
use parsing::*;
use pest::iterators::{Pair, Pairs};
use proc_macro2::TokenStream;
use quote::quote;
use std::{
    fmt::Debug,
    ops::{Add, Div, Mul, Sub},
    sync::Arc,
};

pub type FnPtr = Arc<dyn Fn(&[Value]) -> Value>;

#[derive(Clone)]
pub enum Value {
    STR(String),
    F32(f32),
    F64(f64),
    I32(i32),
    I64(i64),
    BYTE(u8),
    BOOL(bool),
    LIST(Vec<Value>),
    FN(FnPtr),
}
impl Value {
    pub fn call(&self, args: &[Value]) -> Value {
        match self {
            Value::FN(f) => (f)(args),
            _ => panic!("Cannot call a non-function."),
        }
    }
    fn as_string(&self) -> Result<String, String> {
        if let Value::STR(s) = self {
            Ok(s.clone())
        } else {
            Err("Not a string".to_string())
        }
    }
    fn as_f32(&self) -> Result<f32, String> {
        if let Value::F32(f) = self {
            Ok(*f)
        } else {
            Err("Not an f32".to_string())
        }
    }
    fn as_f64(&self) -> Result<f64, String> {
        if let Value::F64(f) = self {
            Ok(*f)
        } else {
            Err("Not an f64".to_string())
        }
    }
    fn as_i32(&self) -> Result<i32, String> {
        if let Value::I32(f) = self {
            Ok(*f)
        } else {
            Err("Not an i32".to_string())
        }
    }
    fn as_i64(&self) -> Result<i64, String> {
        if let Value::I64(f) = self {
            Ok(*f)
        } else {
            Err("Not an i64".to_string())
        }
    }
    fn as_bool(&self) -> Result<bool, String> {
        if let Value::BOOL(b) = self {
            Ok(*b)
        } else {
            Err("Not a bool".to_string())
        }
    }
    fn as_list(&self) -> Result<Vec<Value>, String> {
        if let Value::LIST(f) = self {
            Ok(f.clone())
        } else {
            Err("Not a list".to_string())
        }
    }
    fn as_function(&self) -> Result<Arc<dyn Fn(&[Value]) -> Value>, String> {
        if let Value::FN(f) = self {
            Ok(f.clone())
        } else {
            Err("Not a function".to_string())
        }
    }
}
impl Add for Value {
    type Output = Value;
    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::F32(x), Value::F32(y)) => Value::F32(x + y),
            (Value::F32(x), Value::F64(y)) => Value::F64(x as f64 + y),
            (Value::F32(x), Value::I32(y)) => Value::F32(x + y as f32),
            (Value::F32(x), Value::I64(y)) => Value::F64(x as f64 + y as f64),

            (Value::F64(x), Value::F64(y)) => Value::F64(x + y),
            (Value::F64(x), Value::F32(y)) => Value::F64(x + y as f64),
            (Value::F64(x), Value::I32(y)) => Value::F64(x + y as f64),
            (Value::F64(x), Value::I64(y)) => Value::F64(x + y as f64),

            (Value::I32(x), Value::I32(y)) => Value::I32(x + y),
            (Value::I32(x), Value::I64(y)) => Value::I64(x as i64 + y),
            (Value::I32(x), Value::F32(y)) => Value::F32(x as f32 + y),
            (Value::I32(x), Value::F64(y)) => Value::F64(x as f64 + y),

            (Value::I64(x), Value::I64(y)) => Value::I64(x + y),
            (Value::I64(x), Value::I32(y)) => Value::I64(x + y as i64),
            (Value::I64(x), Value::F32(y)) => Value::F64(x as f64 + y as f64),
            (Value::I64(x), Value::F64(y)) => Value::F64(x as f64 + y),

            (Value::STR(x), Value::STR(y)) => Value::STR(x + &y),
            (Value::BYTE(x), Value::BYTE(y)) => Value::BYTE(x + y),
            _ => {
                panic!("Incompatible types for adding.")
            }
        }
    }
}
impl Sub for Value {
    type Output = Value;
    fn sub(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (Value::F32(x), Value::F32(y)) => Value::F32(x - y),
            (Value::F32(x), Value::F64(y)) => Value::F64(x as f64 - y),
            (Value::F32(x), Value::I32(y)) => Value::F32(x - y as f32),
            (Value::F32(x), Value::I64(y)) => Value::F64(x as f64 - y as f64),

            (Value::F64(x), Value::F64(y)) => Value::F64(x - y),
            (Value::F64(x), Value::F32(y)) => Value::F64(x - y as f64),
            (Value::F64(x), Value::I32(y)) => Value::F64(x - y as f64),
            (Value::F64(x), Value::I64(y)) => Value::F64(x - y as f64),

            (Value::I32(x), Value::I32(y)) => Value::I32(x - y),
            (Value::I32(x), Value::I64(y)) => Value::I64(x as i64 - y),
            (Value::I32(x), Value::F32(y)) => Value::F32(x as f32 - y),
            (Value::I32(x), Value::F64(y)) => Value::F64(x as f64 - y),

            (Value::I64(x), Value::I64(y)) => Value::I64(x - y),
            (Value::I64(x), Value::I32(y)) => Value::I64(x - y as i64),
            (Value::I64(x), Value::F32(y)) => Value::F64(x as f64 - y as f64),
            (Value::I64(x), Value::F64(y)) => Value::F64(x as f64 - y),

            (Value::BYTE(x), Value::BYTE(y)) => Value::BYTE(x - y),
            _ => {
                panic!("Incompatible types for adding.")
            }
        }
    }
}
impl From<isize> for Value {
    fn from(value: isize) -> Self {
        Value::I32(value as i32)
    }
}
impl From<i64> for Value {
    fn from(value: i64) -> Self {
        Value::I64(value)
    }
}
impl From<i32> for Value {
    fn from(value: i32) -> Self {
        Value::I32(value)
    }
}
impl From<Value> for i32 {
    fn from(value: Value) -> Self {
        match value {
            Value::I32(x) => x,
            Value::I64(x) => x as i32,
            _ => panic!("Must be an integer type!"),
        }
    }
}
impl Add<Value> for i64 {
    type Output = i64;
    fn add(self, rhs: Value) -> Self::Output {
        match rhs {
            Value::I32(x) => self + x as i64,
            Value::I64(x) => self + x,
            _ => panic!("Must be an integer type!"),
        }
    }
}
impl From<Value> for i64 {
    fn from(value: Value) -> Self {
        match value {
            Value::I64(x) => x,
            Value::I32(x) => x as i64,
            _ => panic!("Must be an integer type!"),
        }
    }
}

pub fn transpile_to_rust(pairs: Pairs<'_, Rule>) -> TokenStream {
    let mut result = TokenStream::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::forloop => {
                let mut inner = pair.into_inner();
                let dummy = syn::parse_str::<syn::Ident>(inner.next().unwrap().as_str()).unwrap();
                let iterator = transpile_to_rust(inner.next().unwrap().into_inner());
                let expression = transpile_to_rust(inner.next().unwrap().into_inner());
                result.extend(quote! {for #dummy in #iterator {#expression}});
            }
            Rule::range => {
                let mut inner = pair.into_inner();
                let min = syn::parse_str::<syn::Lit>(inner.next().unwrap().as_str()).unwrap();
                let max = syn::parse_str::<syn::Lit>(inner.next().unwrap().as_str()).unwrap();
                let step = inner.next();
                if let Some(s) = step {
                    let s_trans = transpile_to_rust(s.into_inner());
                    result.extend(quote! {range_step(#min, #max, #s_trans)})
                } else {
                    result.extend(quote! {range(#min, #max)})
                }
            }
            Rule::set => {
                let mut inner = pair.into_inner();
                let var = syn::parse_str::<syn::Ident>(inner.next().unwrap().as_str()).unwrap();
                let val = transpile_to_rust(inner.next().unwrap().into_inner());
                result.extend(quote! {#var = #val;});
            }
            Rule::fun => {
                result.extend(parse_function(pair));
            }
            Rule::def => {
                let mut inner = pair.into_inner();
                let var_name =
                    syn::parse_str::<syn::Ident>(inner.next().unwrap().as_str()).unwrap();
                let var_def = transpile_to_rust(inner.next().unwrap().into_inner());
                result.extend(quote! {let #var_name: Value = #var_def.into();});
            }
            Rule::defn => {
                let mut inner = pair.into_inner();
                let var_name =
                    syn::parse_str::<syn::Ident>(inner.next().unwrap().as_str()).unwrap();
                let mut params = Vec::new();
                while inner.len() > 1 {
                    let param = inner.next().unwrap();
                    params.push(match param.as_rule() {
                        Rule::symbol_type => parse_typed_identifier(param),
                        Rule::symbol => {
                            let symbol = syn::parse_str::<syn::Ident>(param.as_str())
                                .expect("Invalid symbol");
                            quote! {#symbol: Value}
                        }
                        _ => {
                            panic!("Invalid function parameter")
                        }
                    });
                }
                let body = transpile_to_rust(inner.next().unwrap().into_inner());
                let indices = 0..params.len();
                result.extend(
                    quote! {let #var_name = Value::FN(Arc::new(|args: &[Value]| {
                         #(let #params = args[#indices].clone().into();)*
                         (#body).into()
                    }));},
                );
            }
            Rule::tfun => {
                let mut inner = pair.into_inner();
                let mut params = Vec::new();
                while inner.len() > 1 {
                    let param = inner.next().unwrap();
                    params.push(syn::parse_str::<syn::Ident>(param.as_str()).unwrap());
                }
                let body = transpile_to_rust(inner.next().unwrap().into_inner());
                let indices = 0..params.len();
                result.extend(quote! {Value::FN(Arc::new(|args: &[Value]| {
                     #(let #params = args[#indices].clone().into();)*
                     #body
                }));});
            }
            Rule::tdef => {
                let mut inner = pair.into_inner();
                let var_name = syn::parse_str::<syn::Ident>(inner.next().unwrap().as_str())
                    .expect("Invalid typed variable name");
                let var_def = transpile_to_rust(inner.next().unwrap().into_inner());
                result.extend(quote! {let #var_name = #var_def;});
            }
            Rule::tdefn => {
                let mut inner = pair.into_inner();
                let var_name = syn::parse_str::<syn::Ident>(
                    inner.next().expect("Expected typed function name").as_str(),
                )
                .expect("Invalid typed function name");
                let mut params = Vec::new();
                while inner.len() > 1 {
                    let param = inner.next().expect("Expected typed function parameter");
                    params.push(parse_typed_identifier(param.clone()));
                }
                let body =
                    transpile_to_rust(inner.next().expect("Error parsing body").into_inner());
                let indices = 0..params.len();
                result.extend(
                    quote! {let #var_name = Value::FN(Arc::new(Box::new(|args: &[Value]| {
                         #(let #params = args[#indices].clone().into();)*
                         (#body).into()
                    })));},
                );
            }
            Rule::ifb => {
                let mut inner = pair.into_inner();
                let cond = transpile_to_rust(inner.next().unwrap().into_inner());
                let if_true = transpile_to_rust(inner.next().unwrap().into_inner());
                let if_false = transpile_to_rust(inner.next().unwrap().into_inner());
                result.extend(quote! {if #cond {#if_true} else {#if_false}});
            }
            Rule::opvar => {
                let mut inner = pair.into_inner();
                let operator = transpile_to_rust(inner.next().unwrap().into_inner());
                let mut operands = Vec::new();
                for pair in inner {
                    operands.push(transpile_to_rust(pair.into_inner()));
                }
                let first_operand = operands.first().unwrap();
                result.extend(quote! {#first_operand});
                for op in operands.iter().skip(1) {
                    result.extend(quote! {#operator #op});
                }
            }
            Rule::fnvar => {
                let mut inner = pair.into_inner();
                let fn_sym =
                    syn::parse_str::<syn::Ident>(inner.next().unwrap().as_span().as_str()).unwrap();
                let mut args = Vec::new();
                for pair in inner {
                    args.push(transpile_to_rust(pair.into_inner()))
                }
                result.extend(quote! {#fn_sym.call(&[#(#args.into()),*])});
            }
            Rule::list => {
                let inner = pair.into_inner();
                let transpiled = transpile_to_rust(inner);
                result.extend(quote! {{#transpiled}});
            }
            Rule::symbol => {
                println!("{}", pair.as_str());
                let inner = syn::parse_str::<syn::Ident>(pair.as_str()).expect("Invalid symbol");
                result.extend(quote! {#inner});
            }
            Rule::add => {
                result.extend(quote! {+});
            }
            Rule::sub => {
                result.extend(quote! {-});
            }
            Rule::mul => {
                result.extend(quote! {*});
            }
            Rule::div => {
                result.extend(quote! {/});
            }
            Rule::number => {
                let inner = syn::parse_str::<syn::Lit>(pair.as_str()).unwrap();
                result.extend(quote! {#inner});
            }
            Rule::EOI => return result,
            _ => {
                let inner = pair.into_inner();
                result.extend(transpile_to_rust(inner));
            }
        }
    }
    result
}

fn parse_function(pair: Pair<'_, Rule>) -> TokenStream {
    let mut inner = pair.into_inner();
    let mut params = Vec::new();
    while inner.len() > 1 {
        let param = inner.next().unwrap();
        params.push(match param.as_rule() {
            Rule::symbol_type => parse_typed_identifier(param),
            Rule::symbol => {
                let symbol = syn::parse_str::<syn::Ident>(param.as_str()).expect("Invalid symbol");
                quote! {#symbol: Value}
            }
            _ => {
                panic!("Invalid function parameter")
            }
        });
    }
    let body = transpile_to_rust(inner.next().unwrap().into_inner());
    let indices = 0..params.len();
    quote! {Value::FN(Arc::new(|args: &[Value]| {
        #(let #params = args[#indices].clone().into();)*
        #body
    }));}
}

fn parse_typed_identifier(ident: Pair<'_, Rule>) -> TokenStream {
    let mut inner = ident.into_inner();
    let symbol =
        syn::parse_str::<syn::Ident>(inner.next().expect("Expected typed symbol").as_str())
            .expect("Could not parse typed symbol");
    let t = inner.next().expect("Expected type").as_str();
    let t_token = match t {
        "STR" => {
            quote! {String}
        }
        "F32" => {
            quote! {f32}
        }
        "F64" => {
            quote! {f64}
        }
        "I32" => {
            quote! {i32}
        }
        "I64" => {
            quote! {i64}
        }
        "BOOL" => {
            quote! {bool}
        }
        "LIST" => {
            quote! {Vec<Value>}
        }
        "FN" => {
            quote! {Box<dyn Fn(&[Value]) -> Value>}
        }
        _ => quote! { #t },
    };
    quote! {#symbol: #t_token}
}
