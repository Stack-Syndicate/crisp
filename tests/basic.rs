use crisp::*;

#[test]
fn basic_arithmetic() {
    assert_eq!(crisp!((+ 1 2 3)), (1 + 2 + 3).into());
    assert_eq!(crisp!((- 1 2 3)), (1 - 2 - 3).into());
    assert_eq!(crisp!((* 1 (- 2 3))), (1 * (2 - 3)).into());
    assert_eq!(crisp!((/ 4 2)), 2.into());
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

#[test]
fn if_statements() {
    crisp!((if (= 1 1) true false));
    assert_eq!(crisp!((if (= 1 1) true false)), true.into());
    assert_eq!(crisp!((if (> 1 1) true false)), false.into());
    assert_eq!(crisp!((if (< 3 10) true false)), true.into());
}
