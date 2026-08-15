pub mod logging;
pub mod parsing;
pub mod project;
pub mod semantics;

pub const OPERATORS: &str = "+-*/=<>!&|";
pub const RESERVED: [&str; 13] = [
    "def", "defn", "defm", "defp", "fn", "i32", "i64", "f32", "f64", "u32", "u64", "bool", "loop",
];
