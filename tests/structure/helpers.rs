use proptest::collection::vec;
use proptest::prelude::*;
use proptest::string::string_regex;

pub fn gen_bool() -> impl Strategy<Value = String> {
    prop_oneof![Just("true".to_string()), Just("false".to_string())]
}

fn list<S>(inner: S, range: std::ops::Range<usize>) -> impl Strategy<Value = String>
where
    S: Strategy<Value = String>,
{
    vec(inner, range).prop_map(|xs| format!("({})", xs.join(" ")))
}

pub fn gen_ident() -> impl Strategy<Value = String> {
    string_regex(r"[a-zA-Z_][a-zA-Z0-9_!$%&*+\-./<=>?@^~]{0,31}").unwrap()
}

pub fn gen_number() -> impl Strategy<Value = String> {
    any::<u64>().prop_map(|n| n.to_string())
}

pub fn gen_atom() -> impl Strategy<Value = String> {
    prop_oneof![
        gen_ident(),
        gen_bool(),
        gen_number(),
        string_regex(r#""[a-z]{1,10}""#).unwrap(),
    ]
}

pub fn gen_expr() -> impl Strategy<Value = String> {
    gen_atom().prop_recursive(8, 256, 10, |inner| list(inner, 1..6))
}

pub fn gen_body() -> impl Strategy<Value = String> {
    list(gen_expr(), 0..5)
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
    prop_oneof![gen_type(), Just("void".to_string())]
}

pub fn gen_param() -> impl Strategy<Value = String> {
    (gen_ident(), gen_type()).prop_map(|(name, ty)| format!("{name}:{ty}"))
}

pub fn gen_params() -> impl Strategy<Value = String> {
    list(gen_param(), 0..3)
}

pub fn gen_bad_params() -> impl Strategy<Value = String> {
    prop_oneof![
        gen_params().prop_map(|s| s.replace('(', "X").replace(')', "X")),
        (gen_ident(), gen_ident()).prop_map(|(a, b)| format!("{a}__{b}")),
        gen_type().prop_map(|t| format!("{t}:::")),
        gen_params().prop_map(|s| format!("(((({s}))))")),
    ]
}

pub fn gen_bad_if() -> impl Strategy<Value = String> {
    prop_oneof![
        (gen_expr(), gen_expr(), gen_expr()).prop_map(|(a, b, c)| format!("(iff {a} {b} {c})")),
        (gen_expr(), gen_expr(), gen_expr()).prop_map(|(a, b, c)| format!("(i f {a} {b} {c})")),
        (gen_expr(), gen_expr()).prop_map(|(a, b)| format!("(if {a} {b})")),
        (gen_expr(), gen_expr(), gen_expr(), gen_expr(), gen_expr())
            .prop_map(|(a, b, c, d, e)| format!("(if {a} {b} {c} {d} {e})")),
        (gen_expr(), gen_expr(), gen_expr()).prop_map(|(a, b, c)| format!("if {a} {b} {c}")),
    ]
}

pub fn gen_bad_for() -> impl Strategy<Value = String> {
    prop_oneof![
        (
            prop_oneof![gen_body(), gen_number(), gen_bool()],
            gen_expr(),
            gen_body()
        )
            .prop_map(|(a, b, c)| format!("(for {a} {b} {c})")),
        gen_ident().prop_map(|a| format!("(for {a} \"invalid\" (a))")),
        (gen_ident(), any::<u64>()).prop_map(|(a, n)| format!("(for {a} {n} (a))")),
        (gen_ident(), gen_expr(), gen_atom()).prop_map(|(a, b, c)| format!("(for {a} {b} {c})")),
        (gen_ident(), gen_expr()).prop_map(|(a, b)| format!("(for {a} {b})")),
        (gen_ident(), gen_expr(), gen_body(), gen_expr())
            .prop_map(|(a, b, c, d)| format!("(for {a} {b} {c} {d})")),
    ]
}

pub fn gen_for() -> impl Strategy<Value = String> {
    (
        gen_ident(),
        prop_oneof![gen_ident(), gen_body()],
        gen_body(),
    )
        .prop_map(|(dummy, iter, body)| format!("(for {dummy} {iter} {body})"))
}

pub fn gen_let() -> impl Strategy<Value = String> {
    (
        gen_ident(),
        prop_oneof![gen_body(), gen_number(), gen_bool(), gen_ident()],
    )
        .prop_map(|(a, b)| format!("(let {a} {b})"))
}

pub fn gen_bad_let() -> impl Strategy<Value = String> {
    prop_oneof![
        gen_ident().prop_map(|a| format!("(let {a})")),
        (gen_ident(), gen_body(), gen_body()).prop_map(|(a, b, c)| format!("(let {a} {b} {c})")),
        (
            prop_oneof![gen_body(), gen_bool(), gen_number()],
            gen_body()
        )
            .prop_map(|(a, b)| format!("(let {a} {b})")),
    ]
}

pub fn gen_fn() -> impl Strategy<Value = String> {
    prop_oneof![
        (gen_return_type(), gen_ident(), gen_params(), gen_body())
            .prop_map(|(ret, name, params, body)| { format!("(fn:{ret} {name} {params} {body})") }),
        (gen_return_type(), gen_params(), gen_body())
            .prop_map(|(ret, params, body)| { format!("(fn:{ret} {params} {body})") }),
    ]
}

pub fn gen_bad_fn() -> impl Strategy<Value = String> {
    prop_oneof![
        (gen_return_type(), gen_ident(), gen_params())
            .prop_map(|(r, n, p)| format!("(fn:{r} {n} {p})")),
        (gen_return_type(), gen_ident(), gen_body())
            .prop_map(|(r, n, b)| format!("(fn:{r} {n} {b})")),
        (
            gen_return_type(),
            gen_ident(),
            gen_params(),
            gen_body(),
            gen_body()
        )
            .prop_map(|(r, n, p, b1, b2)| format!("(fn:{r} {n} {p} {b1} {b2})")),
        (gen_return_type(), gen_params()).prop_map(|(r, p)| format!("(fn:{r} {p})")),
        (gen_return_type(), gen_body()).prop_map(|(r, b)| format!("(fn:{r} {b})")),
        (gen_return_type(), gen_params(), gen_body(), gen_body())
            .prop_map(|(r, p, b1, b2)| format!("(fn:{r} {p} {b1} {b2})")),
        (gen_return_type(), gen_ident(), gen_bad_params(), gen_body())
            .prop_map(|(r, n, bp, b)| format!("(fn:{r} {n} {bp} {b})")),
        (gen_return_type(), gen_bad_params(), gen_body())
            .prop_map(|(r, bp, b)| format!("(fn:{r} {bp} {b})")),
    ]
}
