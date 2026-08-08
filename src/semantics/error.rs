use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::span::SimpleSpan;

#[derive(Debug, Clone)]
pub struct SemanticError {
    pub span: SimpleSpan,
    pub message: String,
    pub label_message: Option<String>,
}

pub fn print_semantic_errors(src: &str, errors: &[SemanticError]) {
    for err in errors {
        let span = err.span.into_range();
        let label_text = err.label_message.as_ref().unwrap_or(&err.message);
        Report::build(ReportKind::Error, ("src", span.clone()))
            .with_message(&err.message)
            .with_label(
                Label::new(("src", span))
                    .with_message(label_text)
                    .with_color(Color::BrightRed),
            )
            .finish()
            .eprint(("src", Source::from(src)))
            .unwrap();
    }
}
