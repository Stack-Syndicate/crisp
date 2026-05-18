use crate::structure::helpers::*;
use crisp::parsing::{CrispParser, Rule, ast::validation::validate_if};
use pest::Parser;
use proptest::{prop_assert, proptest};

proptest! {
    #[test]
    fn valid(predicate in gen_body(), then_block in gen_body(), else_block in gen_body()) {
        let source = format!("(if {} {} {})", predicate, then_block, else_block);
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        prop_assert!(validate_if(&pair, ""));
    }
    #[test]
    fn invalid(if_statement in gen_bad_if()) {
        let source = if_statement.to_string();
        let mut pairs = CrispParser::parse(Rule::file, &source).unwrap();
        let pair = pairs.next().unwrap();
        prop_assert!(!validate_if(&pair, ""));
    }
}
