pub mod parsing;
use pest::iterators::Pairs;
use proc_macro2::TokenStream;
use quote::quote;
use parsing::*;

enum Value {
    String(String),
    Number(f32),
    Bool(bool),
    List(Vec<Value>),
    Function(Box<dyn Fn(&[Value]) -> Value>)
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
                    params.push(syn::parse_str::<syn::Ident>(param.as_str()).unwrap());
                }
                let body = transpile_to_rust(inner.next().unwrap().into_inner());
                result.extend(quote! {Box::new(|#(#params),*| { #body })});
            }
            Rule::def => {
                let mut inner = pair.into_inner();
                println!("{:?}", inner);
                let var_name = syn::parse_str::<syn::Ident>(inner.next().unwrap().as_str()).unwrap();
                let var_def = transpile_to_rust(inner.next().unwrap().into_inner());
                result.extend(quote! {let #var_name = #var_def;});
            }
            Rule::ifb => {
                let mut inner = pair.into_inner();
                let cond = transpile_to_rust(inner.next().unwrap().into_inner());
                let if_true = transpile_to_rust(inner.next().unwrap().into_inner());
                let if_false =transpile_to_rust(inner.next().unwrap().into_inner());
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
                let inner = syn::parse_str::<syn::Ident>(pair.as_str()).unwrap();
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
            Rule::EOI => {
                return result
            }
            _ => {
                let inner = pair.into_inner();
                result.extend(transpile_to_rust(inner));
            }
        }
    }    result
}