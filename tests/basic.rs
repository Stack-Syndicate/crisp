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
    crisp!((define f1 [x I64 y I64] (+ x y)));
    crisp!((define f2 [x I64 y] (+ x y)));
    crisp!((define f3 [x y] (+ x y)));
    crisp!((define f4 [x] (
        (define f_inner [a b] (+ a b))
        (f_inner x 2)
    )));
}

#[test]
fn loops() {
    let mut x = 0;
    crisp!((for i in (0, 10) (
        (set x (+ x 1))
    )));
    assert_eq!(x, 10);
}
