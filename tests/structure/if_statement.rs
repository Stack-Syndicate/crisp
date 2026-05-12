use crate::structure::helpers::*;
use crisp::parsing::{CrispParser, Rule, ast::validation::validate_if};
use pest::Parser;
use proptest::{
    collection::vec, prelude::ProptestConfig, prop_assert, proptest, strategy::Strategy,
};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(10000))]
    #[test]
    fn valid_form(predicate in gen_body(), then_block in gen_body(), else_block in gen_body()) {
        let source = format!("(if {} {} {})", predicate, then_block, else_block);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        prop_assert!(validate_if(&pair, ""));
    }
    #[test]
    fn wrong_arity(args in vec(gen_body(), 0..10).prop_filter("Filter out correct arities", |v| v.len() != 2 && v.len() != 3)) {
        let content = args.join(" ");
        let source = format!("(if {})", content);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        prop_assert!(!validate_if(&pair, ""));
    }
    #[test]
    fn invalid_form(if_statement in gen_bad_if()) {
        let source = format!("{}", if_statement);
        let mut pairs = CrispParser::parse(Rule::file, &source).unwrap();
        let pair = pairs.next().unwrap();
        prop_assert!(!validate_if(&pair, ""));
    }
}
