use crate::structure::helpers::*;
use crisp::parsing::{
    SourceFile,
    ast::{CrispParser, Rule, validation::validate_fn},
};
use pest::Parser;
use proptest::{prop_assert, proptest};

proptest! {
    #[test]
    fn valid(f in gen_fn()) {
        let source = f.to_string();
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        prop_assert!(validate_fn(&pair, &SourceFile::default()));
    }

    #[test]
    fn invalid(f in gen_bad_fn()) {
        let source = f.to_string();
        let mut pairs = CrispParser::parse(Rule::list, &source).unwrap();
        let pair = pairs.next().unwrap();
        prop_assert!(!validate_fn(&pair, &SourceFile::default()));
    }
}
