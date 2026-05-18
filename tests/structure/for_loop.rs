use crate::structure::helpers::*;
use crisp::parsing::ast::{CrispParser, Rule, validation::validate_for};
use pest::Parser;
use proptest::{prop_assert, proptest};

proptest! {
    #[test]
    fn valid(f in gen_for()) {
        let source = f.to_string();
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        prop_assert!(validate_for(&pair, ""));
    }
    #[test]
    fn invalid(f in gen_bad_for()) {
        let source = f.to_string();
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        prop_assert!(!validate_for(&pair, ""));
    }
}
