use proptest::{
    arbitrary::any,
    collection::vec,
    prop_oneof,
    strategy::{Just, Strategy},
    string::string_regex,
};

pub fn gen_bool() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("true".to_string()),
        Just("false".to_string().to_string())
    ]
}

fn list<S>(inner: S, range: std::ops::Range<usize>) -> impl Strategy<Value = String>
where
    S: Strategy<Value = String>,
{
    vec(inner, range).prop_map(|xs| format!("({})", xs.join(" ")))
}

pub fn gen_ident() -> impl Strategy<Value = String> {
    string_regex(r"[a-zA-Z_][a-zA-Z0-9_!$%&*+\-./<=>?@^~]*").unwrap()
}

pub fn gen_number() -> impl Strategy<Value = String> {
    string_regex(r"[0-9]+").unwrap()
}

pub fn gen_atom() -> impl Strategy<Value = String> {
    prop_oneof![
        gen_ident(),
        gen_bool(),
        gen_number(),
        string_regex(r#""[a-z]+""#).unwrap(),
    ]
}

pub fn gen_expr() -> impl Strategy<Value = String> {
    gen_atom().prop_recursive(10, 512, 16, |inner| list(inner, 1..6))
}

pub fn gen_body() -> impl Strategy<Value = String> {
    list(gen_expr(), 1..3)
}

pub fn gen_type() -> impl Strategy<Value = String> {
    prop_oneof![
        Just("i8".to_string()),
        Just("i16".to_string()),
        Just("i32".to_string()),
        Just("i64".to_string()),
        Just("u8".to_string()),
        Just("u16".to_string()),
        Just("u32".to_string()),
        Just("u64".to_string()),
        Just("bool".to_string()),
        Just("string".to_string()),
    ]
}

pub fn gen_return_type() -> impl Strategy<Value = String> {
    prop_oneof![gen_type(), Just("void".to_string()),]
}

pub fn gen_param() -> impl Strategy<Value = String> {
    (gen_ident(), gen_type()).prop_map(|(name, ty)| format!("{name}:{ty}"))
}

pub fn gen_params() -> impl Strategy<Value = String> {
    list(gen_param(), 0..3)
}

pub fn gen_bad_params() -> impl Strategy<Value = String> {
    let stripped = gen_params().prop_map(|s| s.replace(['(', ')'], "X"));
    let glued = (gen_ident(), gen_ident()).prop_map(|(a, b)| format!("{a}__{b}"));
    let broken_type = gen_type().prop_map(|t| format!("{t}:::"));
    let over_nested = gen_params().prop_map(|s| format!("(((({s}))))"));
    prop_oneof![stripped, glued, broken_type, over_nested,]
}

pub fn gen_bad_if() -> impl Strategy<Value = String> {
    let wrong_keyword =
        (gen_expr(), gen_expr(), gen_expr()).prop_map(|(a, b, c)| format!("(iff {a} {b} {c})"));
    let broken_keyword =
        (gen_expr(), gen_expr(), gen_expr()).prop_map(|(a, b, c)| format!("(i f {a} {b} {c})"));
    let too_few = (gen_expr(), gen_expr()).prop_map(|(a, b)| format!("(if {a} {b})"));
    let too_many = (gen_expr(), gen_expr(), gen_expr(), gen_expr(), gen_expr())
        .prop_map(|(a, b, c, d, e)| format!("(if {a} {b} {c} {d} {e})"));
    let not_list =
        (gen_expr(), gen_expr(), gen_expr()).prop_map(|(a, b, c)| format!("if {a} {b} {c}"));
    prop_oneof![wrong_keyword, broken_keyword, too_few, too_many, not_list,]
}

pub fn gen_for() -> impl Strategy<Value = String> {
    (
        gen_ident(),
        prop_oneof![gen_ident(), gen_body(),],
        gen_body(),
    )
        .prop_map(|(dummy, iter, body)| format!("(for {dummy} {iter} {body})"))
}

pub fn gen_bad_for() -> impl Strategy<Value = String> {
    let bad_dummy = (
        prop_oneof![gen_body(), gen_number(), gen_bool()],
        prop_oneof![gen_ident(), gen_expr()],
        gen_body(),
    )
        .prop_map(|(a, b, c)| format!("(for {a} {b} {c})"));
    let bad_iter_string =
        (gen_ident(), gen_ident()).prop_map(|(a, s)| format!("(for {a} \"{s}\" (a))"));
    let bad_iter_number =
        (gen_ident(), any::<u64>()).prop_map(|(a, n)| format!("(for {a} {n} (a))"));
    let bad_body = (
        gen_ident(),
        prop_oneof![gen_ident(), gen_expr()],
        gen_atom(),
    )
        .prop_map(|(a, b, c)| format!("(for {a} {b} {c})"));
    let too_few =
        prop_oneof![(gen_ident(), gen_expr()).prop_map(|(a, b)| format!("(for {a} {b})")),];
    let too_many = (gen_ident(), gen_expr(), gen_body(), gen_expr())
        .prop_map(|(a, b, c, d)| format!("(for {a} {b} {c} {d})"));
    prop_oneof![
        bad_dummy,
        bad_iter_string,
        bad_iter_number,
        bad_body,
        too_few,
        too_many,
    ]
}
