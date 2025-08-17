use crisp::crisp;

#[test]
fn basic_arithmetic() {
    assert_eq!(crisp!((+ 1 2 3)), 1 + 2 + 3);
    assert_eq!(crisp!((- 1 2 3)), 1 - 2 - 3);
    assert_eq!(crisp!((- 1 (- 2 3))), 1 - (2 - 3));
    assert_eq!(crisp!((/ 4 2)), 2);
}