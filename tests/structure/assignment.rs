use crate::structure::helpers::*;
use crisp::parsing::ast::validation::validate_let;
use crisp::parsing::ast::{CrispParser, Rule};
use pest::Parser;
use proptest::{prop_assert, proptest};

proptest! {
    #[test]
    fn valid(f in gen_let()) {
        let source = f.to_string();
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        prop_assert!(validate_let(&pair, ""));
    }
    #[test]
    fn invalid(f in gen_bad_let()) {
        let source = f.to_string();
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        prop_assert!(!validate_let(&pair, ""));
    }
}
