use crate::structure::helpers::*;
use crisp::parsing::{CrispParser, Rule, ast::validation::validate_fn};
use pest::Parser;
use proptest::{prelude::ProptestConfig, proptest};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[test]
    fn named(ret_type in gen_return_type(), name in gen_ident(), params in gen_params(), body in gen_body()) {
        let source = format!("(fn:{} {} {} {})", ret_type, name, params, body);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        assert!(validate_fn(&pair, "test"));
    }
    #[test]
    fn named_missing_body(ret_type in gen_return_type(), name in gen_ident(), params in gen_params()) {
        let source = format!("(fn:{} {} {})", ret_type, name, params);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        assert!(!validate_fn(&pair, "test"));
    }
    #[test]
    fn named_missing_params(ret_type in gen_return_type(), name in gen_ident(), body in gen_body()) {
        let source = format!("(fn:{} {} {})", ret_type, name, body);
        let mut pairs =CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        assert!(!validate_fn(&pair, "test"));
    }
    #[test]
    fn named_extra_body_parts(ret_type in gen_return_type(), name in gen_ident(), params in gen_params(), body in gen_body()) {
        let source = format!("(fn:{} {} {} {} {})", ret_type, name, params, body, body);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        assert!(!validate_fn(&pair, "test"));
    }
    #[test]
    fn anonymous(ret_type in gen_return_type(), params in gen_params(), body in gen_body()) {
        let source = format!("(fn:{} {} {})", ret_type, params, body);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        assert!(validate_fn(&pair, "test"));
    }
    #[test]
    fn anonymous_missing_body(ret_type in gen_return_type(), params in gen_params()) {
        let source = format!("(fn:{} {})", ret_type, params);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        assert!(!validate_fn(&pair, "test"));
    }
    #[test]
    fn anonymous_missing_params(ret_type in gen_return_type(), body in gen_body()) {
        let source = format!("(fn:{} {})", ret_type, body);
        let mut pairs =CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        assert!(!validate_fn(&pair, "test"));
    }
    #[test]
    fn anonymous_extra_body_parts(ret_type in gen_return_type(), params in gen_params(), body in gen_body()) {
        let source = format!("(fn:{} {} {} {})", ret_type, params, body, body);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        assert!(!validate_fn(&pair, "test"));
    }
    #[test]
    fn named_bad_params(
        ret_type in gen_return_type(),
        name in gen_ident(),
        bad_params in gen_bad_params(),
        body in gen_body(),
    ) {
        let source =
            format!("(fn:{} {} {} {})",
                ret_type,
                name,
                bad_params,
                body,
            );
        let mut pairs =
            CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        assert!(!validate_fn(&pair, "test"));
    }
}
