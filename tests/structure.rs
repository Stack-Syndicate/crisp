use crisp::parsing::{CrispParser, Rule, ast::validation::validate_fn};
use pest::Parser;
use proptest::prelude::ProptestConfig;
use proptest::string::string_regex;
use proptest::{collection::vec, prelude::Just, prop_oneof, proptest, strategy::Strategy};

fn gen_atom() -> impl Strategy<Value = String> {
    prop_oneof![
        gen_ident(),
        Just("true".to_string()),
        Just("false".to_string()),
        string_regex("[0-9]+").unwrap(),
        string_regex("\"[a-z]+\"").unwrap(),
    ]
}

fn gen_ident() -> impl Strategy<Value = String> {
    let first = "[a-zA-Z_]";
    let rest = "[a-zA-Z0-9_!$%&*+\\-./<=>?@^~]*";
    string_regex(&format!("{}{}", first, rest)).unwrap()
}

fn gen_expr() -> impl Strategy<Value = String> {
    gen_atom().prop_recursive(4, 64, 10, |inner| {
        vec(inner, 0..5).prop_map(|parts| format!("({})", parts.join(" ")))
    })
}

fn gen_body() -> impl Strategy<Value = String> {
    vec(gen_expr(), 1..3).prop_map(|exprs| format!("({})", exprs.join(" ")))
}

fn gen_type() -> impl Strategy<Value = String> {
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

fn gen_param() -> impl Strategy<Value = String> {
    (gen_ident(), gen_type()).prop_map(|(name, ty)| format!("{}:{}", name, ty))
}

fn gen_params() -> impl Strategy<Value = String> {
    vec(gen_param(), 0..3).prop_map(|params| format!("({})", params.join(" ")))
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[test]
    fn fn_named(name in gen_ident(), params in gen_params(), body in gen_body()) {
        let source = format!("(fn {} {} {})", name, params, body);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        assert!(validate_fn(&pair, "test"));
    }
    #[test]
    fn fn_named_missing_body(name in gen_ident(), params in gen_params()) {
        let source = format!("(fn {} {})", name, params);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        assert!(!validate_fn(&pair, "test"));
    }
    #[test]
    fn fn_named_missing_params(name in gen_ident(), body in gen_body()) {
        let source = format!("(fn {} {})", name, body);
        let mut pairs =CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        assert!(!validate_fn(&pair, "test"));
    }
    #[test]
    fn fn_named_extra_body_parts(name in gen_ident(), params in gen_params(), body in gen_body()) {
        let source = format!("(fn {} {} {} {})", name, params, body, body);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        assert!(!validate_fn(&pair, "test"));
    }
    #[test]
    fn fn_anon(params in gen_params(), body in gen_body()) {
        let source = format!("(fn {} {})", params, body);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        assert!(validate_fn(&pair, "test"));
    }
    #[test]
    fn fn_anon_missing_body(params in gen_params()) {
        let source = format!("(fn {})", params);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        assert!(!validate_fn(&pair, "test"));
    }
    #[test]
    fn fn_anon_missing_params(body in gen_body()) {
        let source = format!("(fn {})", body);
        let mut pairs =CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        assert!(!validate_fn(&pair, "test"));
    }
    #[test]
    fn fn_anon_extra_body_parts(params in gen_params(), body in gen_body()) {
        let source = format!("(fn {} {} {})", params, body, body);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        assert!(!validate_fn(&pair, "test"));
    }
}
