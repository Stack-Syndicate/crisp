use crisp_macro::crisp;

#[test]
fn basic_arithmetic() {
    assert_eq!(crisp!((+ 1 2 3)), 1 + 2 + 3);
    assert_eq!(crisp!((- 1 2 3)), 1 - 2 - 3);
    assert_eq!(crisp!((* 1 (- 2 3))), 1 * (2 - 3));
    assert_eq!(crisp!((/ 4 2)), 2);
}

#[test]
fn functions() {
    let f = crisp!((fn [x y] (+ x y)));
    println!("{}", f(1, 2));
}