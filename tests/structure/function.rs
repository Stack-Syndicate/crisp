use crate::structure::helpers::*;
use crisp::parsing::{CrispParser, Rule, ast::validation::validate_fn};
use pest::Parser;
use proptest::prop_assert;
use proptest::{prelude::ProptestConfig, proptest};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[test]
    fn valid(f in gen_fn()) {
        let source = format!("{}", f);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        prop_assert!(validate_fn(&pair, "test"));
    }

    #[test]
    fn invalid(f in gen_bad_fn()) {
        let source = format!("{}", f);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        prop_assert!(!validate_fn(&pair, "test"));
    }
}
