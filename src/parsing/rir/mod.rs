use std::collections::HashMap;

use log::info;

use crate::{diagnostics::print_error, parsing::ast::nodes::Node};
pub type ScopeID = usize;
pub type SymbolID = usize;

#[derive(Debug)]
pub struct Symbol {
    name: String,
    scope: ScopeID,
}

#[derive(Debug)]
pub struct Scope {
    parent: Option<ScopeID>,
    symbols: HashMap<String, SymbolID>,
}

#[derive(Debug)]
pub struct ScopeStack {
    scopes: Vec<Scope>,
    symbols: Vec<Symbol>,
    current_scope_id: ScopeID,
}
impl ScopeStack {
    pub fn new() -> Self {
        let mut stack = Self {
            scopes: vec![Scope {
                parent: None,
                symbols: HashMap::new(),
            }],
            symbols: vec![],
            current_scope_id: 0,
        };
        let builtins = vec!["+", "-", "*", "/", "print", "void", "i32"];
        for builtin in builtins {
            stack.declare(builtin.to_string());
        }

        stack
    }
    fn exit_scope(&mut self) {
        let parent_scope_id = self.scopes[self.current_scope_id]
            .parent
            .expect("Cannot exit root scope");
        self.current_scope_id = parent_scope_id;
    }
    fn enter_scope(&mut self) {
        self.scopes.push(Scope {
            parent: Some(self.current_scope_id),
            symbols: HashMap::new(),
        });
        self.current_scope_id = self.scopes.len() - 1;
    }
    fn declare(&mut self, name: String) -> SymbolID {
        self.symbols.push(Symbol {
            name: name.clone(),
            scope: self.current_scope_id,
        });
        let symbol_id = self.symbols.len() - 1;
        self.scopes[self.current_scope_id]
            .symbols
            .insert(name, symbol_id);
        symbol_id
    }
    fn resolve(&self, name: &str) -> Option<SymbolID> {
        let mut current_scope_id = self.current_scope_id;
        loop {
            let scope = &self
                .scopes
                .get(current_scope_id)
                .expect("Scope doesn't exist");
            if let Some(id) = scope.symbols.get(name) {
                return Some(*id);
            }
            match scope.parent {
                Some(parent) => current_scope_id = parent,
                None => break,
            }
        }
        None
    }
    fn exists_in_scope(&self, name: &str) -> bool {
        let current_scope = &self.scopes[self.current_scope_id];
        current_scope.symbols.contains_key(name)
    }
}

impl Default for ScopeStack {
    fn default() -> Self {
        Self::new()
    }
}

pub fn ast_to_scopes(node: &Node, scopes: &mut ScopeStack) -> bool {
    match node {
        Node::Fn {
            name,
            params,
            body,
            info,
        } => {
            if let Some(name_symbol) = name {
                let name_str = name_symbol.get_name();
                if scopes.exists_in_scope(&name_str) {
                    print_error("Symbol already registered in scope", info);
                    return false;
                } else {
                    scopes.declare(name_str);
                }
            }
            scopes.enter_scope();
            for param in params {
                scopes.declare(param.get_name());
            }
            let res = ast_to_scopes(body, scopes);
            scopes.exit_scope();
            if !res {
                return false;
            }
            true
        }

        Node::If {
            predicate,
            yes,
            no,
            info,
        } => {
            if !ast_to_scopes(predicate, scopes) {
                return false;
            }
            if !ast_to_scopes(yes, scopes) {
                return false;
            }
            if let Some(no_block) = no {
                return ast_to_scopes(no_block, scopes);
            }
            true
        }

        Node::Let {
            symbol,
            value,
            info,
        } => {
            let name = symbol.get_name();
            scopes.declare(name);
            ast_to_scopes(value, scopes)
        }

        Node::For {
            dummy,
            iterator,
            body,
            info,
        } => {
            scopes.enter_scope();
            scopes.declare(dummy.get_name());
            if !ast_to_scopes(iterator, scopes) {
                return false;
            }
            if !ast_to_scopes(body, scopes) {
                return false;
            }
            scopes.exit_scope();
            true
        }
        Node::Given {
            predicate,
            cases,
            info,
        } => {
            if !ast_to_scopes(predicate, scopes) {
                print_error("Invalid predicate detected", info);
                return false;
            }
            scopes.enter_scope();
            let res = ast_to_scopes(cases, scopes);
            scopes.exit_scope();
            res
        }
        Node::Return { value, .. } => ast_to_scopes(value, scopes),
        Node::Identifier { symbol, info } => {
            if scopes.resolve(&symbol.get_name()).is_none() {
                print_error("Unknown symbol detected", info);
                return false;
            }
            true
        }
        Node::Literal { inner, info } => true,
        Node::Call { name, args, info } => {
            if scopes.resolve(&name.get_name()).is_none() {
                print_error("Unknown symbol detected", info);
                return false;
            }
            for arg in args {
                if !ast_to_scopes(arg, scopes) {
                    print_error("Invalid argument detected", info);
                    return false;
                }
            }
            true
        }
        Node::Block { expressions, info } => {
            scopes.enter_scope();
            for expr in expressions {
                if !ast_to_scopes(expr, scopes) {
                    scopes.exit_scope();
                    return false;
                }
            }
            scopes.exit_scope();
            true
        }
        Node::Invalid => false,
    }
}
