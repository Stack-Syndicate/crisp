use crate::structure::helpers::*;
use crisp::parsing::CrispParser;
use crisp::parsing::Rule;
use crisp::parsing::ast::validation::validate_if;
use pest::Parser;
use proptest::collection::vec;
use proptest::prelude::ProptestConfig;
use proptest::prop_assert;
use proptest::proptest;
use proptest::strategy::Strategy;

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1000))]
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
    fn bad_if(if_statement in gen_bad_if()) {
        let source = format!("{}", if_statement);
        let mut pairs = CrispParser::parse(Rule::file, &source).unwrap();
        let pair = pairs.next().unwrap();
        prop_assert!(!validate_if(&pair, ""));
    }
}
