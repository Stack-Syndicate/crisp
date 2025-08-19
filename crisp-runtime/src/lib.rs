pub mod parsing;
use std::{ops::Add, sync::Arc};
use parsing::*;
use pest::iterators::{Pair, Pairs};
use proc_macro2::TokenStream;
use quote::quote;

#[derive(Clone)]
pub enum Value {
    String(String),
    Number(f32),
    Bool(bool),
    List(Vec<Value>),
    Function(Arc<dyn Fn(&[Value]) -> Value>),
}
impl Value {
    fn as_string(&self) -> Result<String, String> {
        if let Value::String(s) = self {
            Ok(s.clone())
        } else {
            Err("Not a string".to_string())
        }
    }
    fn as_number(&self) -> Result<f32, String> {
        if let Value::Number(f) = self {
            Ok(*f)
        } else {
            Err("Not a number".to_string())
        }
    }
    fn as_bool(&self) -> Result<bool, String> {
        if let Value::Bool(b) = self {
            Ok(*b)
        } else {
            Err("Not a bool".to_string())
        }
    }
    fn as_list(&self) -> Result<Vec<Value>, String> {
        if let Value::List(f) = self {
            Ok(f.clone())
        } else {
            Err("Not a list".to_string())
        }
    }
    fn as_function(&self) -> Result<Arc<dyn Fn(&[Value]) -> Value>, String> {
        if let Value::Function(f) = self {
            Ok(f.clone())
        } else {
            Err("Not a function".to_string())
        }
    }
}
impl From<Value> for String {
    fn from(value: Value) -> Self {
        return value.as_string().unwrap();
    }
}
impl From<Value> for f32 {
    fn from(value: Value) -> Self {
        return value.as_number().unwrap();
    }
}
impl From<f32> for Value {
    fn from(value: f32) -> Self {
        return Value::Number(value);
    }
}
impl From<String> for Value {
    fn from(value: String) -> Self {
        return Value::String(value);
    }
}
impl Add<f32> for Value {
    type Output = Value;
    fn add(self, other: f32) -> Self::Output {
        match self {
            Value::Number(s) => Value::Number(s + other),
            _ => {
                panic!("Cannot add these types")
            }
        }
    }
}
impl Add<Value> for f32 {
    type Output = f32;
    fn add(self, other: Value) -> Self::Output {
        match other {
            Value::Number(s) => s + other,
            _ => {
                panic!("Cannot add these types")
            }
        }
    }
}
impl Add<String> for Value {
    type Output = Value;
    fn add(self, other: String) -> Self::Output {
        match self {
            Value::String(s) => Value::String(s + other.as_str()),
            _ => {
                panic!("Cannot add these types")
            }
        }
    }
}
impl Add<Value> for String {
    type Output = Value;
    fn add(self, other: Value) -> Self::Output {
        match other {
            Value::String(s) => Value::String(self + s.as_str()),
            _ => {
                panic!("Cannot add these types")
            }
        }
    }
}
impl Add<Value> for Value {
    type Output = Value;
    fn add(self, other: Value) -> Self::Output {
        match self {
            Value::Number(s) => Value::Number(s + other.as_number().unwrap()),
            Value::String(s) => Value::String(s + other.as_string().unwrap().as_str()),
            _ => {
                panic!("Cannot add these types")
            }
        }
    }
}

pub fn transpile_to_rust(pairs: Pairs<'_, Rule>) -> TokenStream {
    let mut result = TokenStream::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::fun => {
                let mut inner = pair.into_inner();
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
                result.extend(quote! {Value::Function(Arc::new(|args: &[Value]| {
                     #(let #params = args[#indices].clone().into();)*
                     #body 
                }));});
            }
            Rule::def => {
                let mut inner = pair.into_inner();
                let var_name =
                    syn::parse_str::<syn::Ident>(inner.next().unwrap().as_str()).unwrap();
                let var_def = transpile_to_rust(inner.next().unwrap().into_inner());
                result.extend(quote! {let #var_name: Value = #var_def;});
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
                result.extend(quote! {let #var_name = Value::Function(Arc::new(|args: &[Value]| {
                     #(let #params = args[#indices].clone().into();)*
                     (#body).into() 
                }));});
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
                result.extend(quote! {Value::Function(Arc::new(|args: &[Value]| {
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
                result.extend(quote! {let #var_name = Value::Function(Arc::new(Box::new(|args: &[Value]| {
                     #(let #params = args[#indices].clone().into();)*
                     (#body).into() 
                })));});
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
            Rule::list => {
                let inner = pair.into_inner();
                let transpiled = transpile_to_rust(inner);
                result.extend(quote! {{#transpiled}});
            }
            Rule::symbol => {
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

fn parse_typed_identifier(ident: Pair<'_, Rule>) -> TokenStream {
    let mut inner = ident.into_inner();
    let symbol =
        syn::parse_str::<syn::Ident>(inner.next().expect("Expected typed symbol").as_str())
            .expect("Could not parse typed symbol");
    let t = inner.next().expect("Expected type").as_str();
    let t_token = match t {
        "String" => {
            quote! {String}
        }
        "Number" => {
            quote! {f32}
        }
        "Bool" => {
            quote! {bool}
        }
        "List" => {
            quote! {Vec<Value>}
        }
        "Function" => {
            quote! {Box<dyn Fn(&[Value]) -> Value>}
        }
        _ => quote! { #t },
    };
    quote! {#symbol: #t_token}
}
