use crisp_runtime::{parsing::*, transpile_to_rust};
use pest::Parser;
use proc_macro::TokenStream;
use quote::quote;

#[proc_macro]
pub fn crisp(input: TokenStream) -> TokenStream {
    let ts_str = input.to_string();
    let parsed_ts = CrispParser::parse(Rule::program, &ts_str).expect("Parsing failed.");
    let parsed_crisp = transpile_to_rust(parsed_ts);
    TokenStream::from(quote! {
        #parsed_crisp
    })
}
