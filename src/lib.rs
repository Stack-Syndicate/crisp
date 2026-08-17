pub mod logging;
pub mod parsing;
pub mod project;
pub mod semantics;

pub mod consts {
    pub const OPERATORS: &str = "+-*/=<>!&|";
    pub const RESERVED: [&str; 13] = [
        "def", "defn", "defm", "defp", "fn", "i32", "i64", "f32", "f64", "u32", "u64", "bool",
        "loop",
    ];
    pub const OPTION_MODIFIER: char = '?';
    pub const RESULT_MODIFIER: char = '!';
    pub const QUOTE_MODIFIER: char = '#';
    pub const PROTOCOL_MODIFIER: char = '@';
    pub const REFERENCE_MODIFIER: char = '&';
    pub const PLACEHOLDER_MODIFIER: char = '$';

    pub const MEMBER_ACCESS_SEPARATOR: char = '/';
    pub const RETURN_TYPE_STRING: &str = ">";
}
