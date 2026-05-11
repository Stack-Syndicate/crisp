use proptest::{
    collection::vec,
    prop_oneof,
    strategy::{Just, Strategy},
    string::string_regex,
};

pub fn gen_atom() -> impl Strategy<Value = String> {
    prop_oneof![
        gen_ident(),
        Just("true".to_string()),
        Just("false".to_string()),
        string_regex("[0-9]+").unwrap(),
        string_regex("\"[a-z]+\"").unwrap(),
    ]
}

pub fn gen_ident() -> impl Strategy<Value = String> {
    let first = "[a-zA-Z_]";
    let rest = "[a-zA-Z0-9_!$%&*+\\-./<=>?@^~]*";
    string_regex(&format!("{}{}", first, rest)).unwrap()
}

pub fn gen_expr() -> impl Strategy<Value = String> {
    gen_atom().prop_recursive(10, 512, 16, |inner| {
        vec(inner, 1..6).prop_map(|parts| format!("({})", parts.join(" ")))
    })
}

pub fn gen_body() -> impl Strategy<Value = String> {
    vec(gen_expr(), 1..3).prop_map(|exprs| format!("({})", exprs.join(" ")))
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
    (gen_ident(), gen_type()).prop_map(|(name, ty)| format!("{}:{}", name, ty))
}

pub fn gen_params() -> impl Strategy<Value = String> {
    vec(gen_param(), 0..3).prop_map(|params| format!("({})", params.join(" ")))
}

pub fn gen_bad_params() -> impl Strategy<Value = String> {
    use proptest::prelude::*;
    let stripped = gen_params().prop_map(|s| s.replace(['(', ')'], "X"));
    let glued = (gen_ident(), gen_ident()).prop_map(|(a, b)| format!("{}__{}", a, b));
    let broken_type = gen_type().prop_map(|t| format!("{}:::", t));
    let over_nested = gen_params().prop_map(|s| format!("(((({}))))", s));
    prop_oneof![stripped, glued, broken_type, over_nested,]
}

pub fn gen_bad_if() -> impl Strategy<Value = String> {
    let wrong_keyword = (gen_expr(), gen_expr(), gen_expr())
        .prop_map(|(a, b, c)| format!("(iff {} {} {})", a, b, c));
    let broken_keyword = (gen_expr(), gen_expr(), gen_expr())
        .prop_map(|(a, b, c)| format!("(i f {} {} {})", a, b, c));
    let too_few = (gen_expr(), gen_expr()).prop_map(|(a, b)| format!("(if {} {})", a, b));
    let too_many = (gen_expr(), gen_expr(), gen_expr(), gen_expr(), gen_expr())
        .prop_map(|(a, b, c, d, e)| format!("(if {} {} {} {} {})", a, b, c, d, e));
    let not_list =
        (gen_expr(), gen_expr(), gen_expr()).prop_map(|(a, b, c)| format!("if {} {} {}", a, b, c));
    prop_oneof![wrong_keyword, broken_keyword, too_few, too_many, not_list,]
}
