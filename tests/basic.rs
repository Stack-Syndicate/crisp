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
    crisp!((define f1 [x Number y Number] (+ x y)));
    crisp!((define f2 [x Number y] (+ x y)));
    crisp!((define f3 [x y] (+ x y)));
}

#[test]
fn if_statements() {
    assert_eq!(crisp!((if (= 1 1) true false)), true.into());
    assert_eq!(crisp!((if (> 1 1) true false)), false.into());
    assert_eq!(crisp!((if (< 3 10) true false)), true.into());
}
