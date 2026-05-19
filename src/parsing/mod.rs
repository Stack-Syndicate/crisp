pub mod ast;
pub mod rir;

#[derive(Default, Debug, Clone)]
pub struct SourceFile {
    pub path: String,
    pub source: String,
}
