use crate::parsing::ast::Node;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub struct SymbolInfo {}

pub struct Scope {
    pub symbols: HashMap<String, SymbolInfo>,
    pub parent: Option<Arc<Mutex<Scope>>>,
    pub children: Vec<Arc<Mutex<Scope>>>,
}
impl Scope {
    pub fn new(parent: Option<Arc<Mutex<Scope>>>) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self {
            symbols: HashMap::new(),
            parent,
            children: Vec::new(),
        }))
    }
}

pub fn build_scope_tree(ast: &Vec<Node>) -> Arc<Mutex<Scope>> {
    let root = Scope::new(None);
    let mut current = Arc::clone(&root);
    for node in ast {
        walk_node(node, &mut current);
    }
    root
}

fn walk_node(node: &Node, current: &mut Arc<Mutex<Scope>>) {
    match node {
        Node::List(elements, _) => {
            if let Some(Node::TypedSymbol(name, annotation, info)) = elements.get(0) {
                match name.as_str() {
                    "fn" => {}
                    _ => {
                        for inner in elements {
                            walk_node(inner, current);
                        }
                    }
                }
            }
        }
        _ => {}
    }
}
