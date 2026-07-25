use chumsky::Parser;

fn main() {
    let crisp_txt = "(def f (fn [x:i32, y:i32]->i32 10))";
    println!("{:#?}", crisp::crip_parser().parse(crisp_txt));
}
