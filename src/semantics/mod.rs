#[salsa::db]
#[derive(Default)]
struct SemanticAnalysisDb {
    storage: salsa::Storage<Self>,
}
#[salsa::db]
impl salsa::Database for SemanticAnalysisDb {}

#[salsa::input]
struct SourceFile {
    #[returns(deref)]
    text: String,
}
