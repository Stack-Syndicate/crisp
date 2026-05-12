use crate::structure::helpers::*;
use crisp::parsing::{CrispParser, Rule, ast::validation::validate_for};
use pest::Parser;
use proptest::prelude::ProptestConfig;
use proptest::{prop_assert, proptest};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[test]
    fn valid_form(f in gen_for()) {
        let source = format!("{}", f);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        prop_assert!(validate_for(&pair, ""));
    }
    #[test]
    fn invalid_form(f in gen_bad_for()) {
        let source = format!("{}", f);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        prop_assert!(!validate_for(&pair, ""));
    }
}
