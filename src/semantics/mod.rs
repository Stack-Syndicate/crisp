use std::collections::HashMap;

use crate::parsing::ast::ExprVisitor;
use crate::semantics::error::SemanticError;
use crate::{
    parsing::ast::ParseExpr,
    semantics::{
        names::NameResolverPass,
        scope::{ScopeBuilderPass, ScopeTree},
    },
};

pub mod error;
pub mod names;
pub mod scope;

pub fn analyze_semantics(ast: &[ParseExpr]) -> Vec<SemanticError> {
    let mut errors = Vec::new();
    let mut tree = ScopeTree::new();
    let mut scope_builder = ScopeBuilderPass { tree: &mut tree };
    for expr in ast {
        scope_builder.visit_expr(expr, 0);
    }
    let mut name_resolver = NameResolverPass {
        tree: &tree,
        child_indices: HashMap::new(),
        errors: Vec::new(),
    };
    for expr in ast {
        name_resolver.visit_expr(expr, 0);
    }
    name_resolver
        .errors
        .iter()
        .for_each(|error| errors.push(error.clone()));
    errors
}
