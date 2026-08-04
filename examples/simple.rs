use chumsky::Parser;

fn main() {
    let crisp_txt = "\
        (\
            (def f (fn [x:i32, y:i32] -> i32\
                (return 45)\
            ))\
            (loop (x < 1) (+ 2 3))\
        )";
    println!("{:#?}", crisp_txt);
    println!("{:#?}", crisp::crip_parser().parse(crisp_txt));
}
