use chumsky::Parser;
use crisp::compilation::compile;
use crisp::parsing::{crip_parser, error::print_parse_errors};

fn main() {
    let crisp_txt = "((defn foo[x: i32, y: i32]->i32 (+ 10 20)))";
    let (crisp_parsed, errors) = crip_parser().parse(crisp_txt).into_output_errors();
    if !errors.is_empty() {
        print_parse_errors(crisp_txt, &errors);
    } else {
        compile(crisp_parsed.unwrap(), "test").unwrap();
    }
}
