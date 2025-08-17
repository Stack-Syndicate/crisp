use pest::{Parser, iterators::Pairs};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;

mod parsing {
    use pest_derive::Parser;
    #[derive(Parser)]
    #[grammar = "grammar.pest"]
    pub struct CrispParser;
}

use parsing::*;

#[proc_macro]
pub fn crisp(input: TokenStream) -> TokenStream {
    let ts_str = input.to_string();
    let parsed_ts = CrispParser::parse(Rule::program, &ts_str).expect("Parsing failed.");
    let parsed_crisp = parse_crisp(parsed_ts);
    TokenStream::from(quote! { #parsed_crisp })
}

fn parse_crisp(pairs: Pairs<'_, Rule>) -> TokenStream2 {
    let mut result_ts = TokenStream2::new();
    for pair in pairs {
        match pair.as_rule() {
            Rule::program => {
                let program = pair.into_inner();
                result_ts.extend(parse_crisp(program));
            }
            Rule::s_expr => {
                let mut s_expr = pair.into_inner();
                // Extract identifier
                let ident = s_expr.next().unwrap().into_inner().next().unwrap();
                // Extract arguments
                let args: Vec<TokenStream2> =
                    s_expr.map(|arg| parse_crisp(arg.into_inner())).collect();
                let ts = match ident.as_rule() {
                    Rule::op_ident => {
                        let operator = ident.into_inner().next().unwrap();
                        match operator.as_rule() {
                            Rule::add => quote! { (#(#args)+*) },
                            Rule::sub => quote! { (#(#args)-*) },
                            Rule::mul => quote! { (#(#args)**) },
                            Rule::div => quote! { (#(#args)/ *) },
                            _ => unreachable!(),
                        }
                    }
                    Rule::fn_ident => {
                        let fn_ident = syn::parse_str::<syn::Ident>(ident.as_str()).unwrap();
                        quote! { #fn_ident(#(#args),*) }
                    }
                    _ => panic!("Unexpected identifier {:?}", ident.as_rule()),
                };
                result_ts.extend(ts);
            }
            Rule::literal | Rule::integer | Rule::float | Rule::string | Rule::bool => {
                let lit =
                    syn::parse_str::<syn::Expr>(&pair.as_str()).expect("Failed to parse literal");
                result_ts.extend(quote! { #lit });
            }
            Rule::EOI => {
                break;
            }
            _ => {
                panic!("Parsing failed {}", pair)
            }
        }
    }
    return result_ts;
}
