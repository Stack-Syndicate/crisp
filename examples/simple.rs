use chumsky::Parser;
use crisp::{
    parsing::{crip_parser, error::print_parse_errors},
    semantics::{analyze_semantics, error::print_semantic_errors},
};

fn main() {
    let crisp_txt = "\
        (defn foo[x: i32, y: i32] -> i32 (\
            (+ 10 z)\
        ))";
    let (ast_opt, errors) = crip_parser().parse(crisp_txt).into_output_errors();
    if !errors.is_empty() {
        print_parse_errors(crisp_txt, &errors);
    }
    if let Some(ast) = ast_opt {
        let semantic_errors = analyze_semantics(&ast);
        print_semantic_errors(crisp_txt, &semantic_errors);
    }
}
