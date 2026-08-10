use crate::{
    OPERATORS,
    parsing::ast::{ExprVisitor, Param, ParseExpr, Type},
};
use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Symbol {
    pub name: String,
    pub type_annotation: Option<Type>,
}

#[derive(Debug, Clone, Default)]
pub struct Scope {
    pub parent: Option<usize>,
    pub children: Vec<usize>,
    pub symbols: HashSet<Symbol>,
}

#[derive(Debug, Default)]
pub struct ScopeTree {
    pub scopes: Vec<Scope>,
}

impl ScopeTree {
    pub fn new() -> Self {
        let mut toplevel_scope = Scope::default();
        for c in OPERATORS.chars() {
            toplevel_scope.symbols.insert(Symbol {
                name: c.to_string(),
                type_annotation: None,
            });
        }
        Self {
            scopes: vec![toplevel_scope],
        }
    }
    pub fn create_scope(&mut self, parent: usize) -> usize {
        let child_index = self.scopes.len();
        self.scopes.push(Scope {
            parent: Some(parent),
            children: Vec::new(),
            symbols: HashSet::new(),
        });
        self.scopes[parent].children.push(child_index);
        child_index
    }
    pub fn insert_symbol(
        &mut self,
        scope_index: usize,
        name: String,
        type_annotation: Option<Type>,
    ) {
        self.scopes[scope_index].symbols.insert(Symbol {
            name,
            type_annotation,
        });
    }
    pub fn lookup(&self, name: &str, start_scope: usize) -> Option<(usize, &Symbol)> {
        let mut current_scope = Some(start_scope);
        while let Some(sid) = current_scope {
            let scope = &self.scopes[sid];
            if let Some(sym) = scope.symbols.iter().find(|s| s.name == name) {
                return Some((sid, sym));
            }
            current_scope = scope.parent;
        }
        None
    }
}

pub struct ScopeBuilderPass<'a> {
    pub tree: &'a mut ScopeTree,
}
impl<'a> ExprVisitor for ScopeBuilderPass<'a> {
    fn visit_def(
        &mut self,
        name: &str,
        type_ann: Option<&Type>,
        value: &ParseExpr,
        scope_index: usize,
    ) {
        self.tree
            .insert_symbol(scope_index, name.to_string(), type_ann.cloned());
        self.visit_expr(value, scope_index);
    }
    fn visit_block(&mut self, ast: &[ParseExpr], _expr: &ParseExpr, scope_index: usize) {
        let block_scope = self.tree.create_scope(scope_index);
        for expr in ast {
            self.visit_expr(expr, block_scope);
        }
    }
    fn visit_fn(
        &mut self,
        params: &[Param],
        body: &ParseExpr,
        _expr: &ParseExpr,
        scope_index: usize,
    ) {
        let fn_scope = self.tree.create_scope(scope_index);
        for param in params {
            self.tree.insert_symbol(
                fn_scope,
                param.name.clone(),
                Some(param.type_annotation.clone()),
            );
        }
        self.visit_expr(body, fn_scope);
    }
}
