use crisp::*;

#[test]
fn basic_arithmetic() {
    assert_eq!(crisp!((+ 1 2 3)), 1 + 2 + 3);
    assert_eq!(crisp!((- 1 2 3)), 1 - 2 - 3);
    assert_eq!(crisp!((* 1 (- 2 3))), 1 * (2 - 3));
    assert_eq!(crisp!((/ 4 2)), 2);
}

#[test]
fn functions() {
    crisp!((define [x y] (+ x y)));
    crisp!((define f1 [x Number y Number] (+ x y)));
    crisp!((define f2 [x Number y] (+ x y)));
    crisp!((define f3 [x y] (+ x y)));
}
