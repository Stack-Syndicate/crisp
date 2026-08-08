use std::collections::HashMap;

use crate::{
    parsing::ast::{ExprVisitor, Param, ParseExpr},
    semantics::{error::SemanticError, scope::ScopeTree},
};

pub struct NameResolverPass<'a> {
    pub tree: &'a ScopeTree,
    pub child_indices: HashMap<usize, usize>,
    pub errors: Vec<SemanticError>,
}

impl<'a> ExprVisitor for NameResolverPass<'a> {
    fn visit_identifier(&mut self, name: &str, expr: &ParseExpr, scope_index: usize) {
        if self.tree.lookup(name, scope_index).is_none() {
            self.errors.push(SemanticError {
                span: expr.span,
                message: format!("Unresolved identifier `{}`", name),
                label_message: Some("Unresolved identifier. Maybe a typo?".to_string()),
            });
        }
    }
    fn visit_block(&mut self, ast: &[ParseExpr], _expr: &ParseExpr, scope_index: usize) {
        let child_index = self.child_indices.entry(scope_index).or_insert(0);
        let block_scope = self.tree.scopes[scope_index].children[*child_index];
        *child_index += 1;
        for expr in ast {
            self.visit_expr(expr, block_scope);
        }
    }
    fn visit_fn(
        &mut self,
        _params: &[Param],
        body: &ParseExpr,
        _expr: &ParseExpr,
        scope_index: usize,
    ) {
        let child_index = self.child_indices.entry(scope_index).or_insert(0);
        let fn_scope = self.tree.scopes[scope_index].children[*child_index];
        *child_index += 1;
        self.visit_expr(body, fn_scope);
    }
}
