use crate::structure::helpers::*;
use crisp::parsing::ast::validation::validate_let;
use crisp::parsing::{CrispParser, Rule};
use pest::Parser;
use proptest::prelude::ProptestConfig;
use proptest::{prop_assert, proptest};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[test]
    fn valid(f in gen_let()) {
        let source = format!("{}", f);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        prop_assert!(validate_let(&pair, ""));
    }
    #[test]
    fn invalid(f in gen_bad_let()) {
        let source = format!("{}", f);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        prop_assert!(!validate_let(&pair, ""));
    }
}
