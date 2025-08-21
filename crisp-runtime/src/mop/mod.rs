use std::sync::Arc;
use std::collections::HashMap;

type Method = Arc<dyn Fn(&mut Object, Vec<Object>) -> Object>;

pub enum Type {
    F32(f32), F64(f64),
    I8(i8), I16(i16), I32(i32), I64(i64), I128(i128),
    BYTE(u8),
    U16(u16), U32(u32), U64(u64), U128(u128),
    STRING(String),
    LIST(Vec<Object>),
    FUNCTION(Method)
}

pub struct Object {
    pub data: Arc<Type>,
    pub meta: Arc<dyn MetaObject>
}
impl Object {
    pub fn call(&mut self, method_name: &str, args: Vec<Object>) -> Option<Object> {
        self.meta.get_method(method_name).map(|m| m(self, args))
    }
}

pub trait MetaObject {
    fn add_method(&mut self, name: &str, method: Method);
    fn add_delegate(&mut self, delegate: Arc<dyn MetaObject>);
    fn get_method(&self, name: &str) -> Option<Method>;
    fn instantiate(&self) -> Object;
}

#[derive(Clone)]
pub struct BaseMetaObject {
    pub name: String,
    pub methods: HashMap<String, Method>,
    pub delegates: Vec<Arc<dyn MetaObject>>
}
impl BaseMetaObject {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            methods: HashMap::new(),
            delegates: Vec::new()
        }
    }
}
impl MetaObject for BaseMetaObject {
    fn add_method(&mut self, name: &str, method: Method) {
        self.methods.insert(name.to_string(), method);
    }
    fn add_delegate(&mut self, delegate: Arc<dyn MetaObject>) {
        self.delegates.push(delegate);
    }
    fn get_method(&self, name: &str) -> Option<Method> {
        if let Some(m) = self.methods.get(name) {
            return Some(m.clone())
        }
        for d in &self.delegates {
            if let Some(m) = d.get_method(name) {
                return Some(m)
            }
        }
        return None
    }
    fn instantiate(&self) -> Object {
        Object {
            data: Arc::new(Type::BYTE(0u8)),
            meta: Arc::new(self.clone())
        }
    }
}
