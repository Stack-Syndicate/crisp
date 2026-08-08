use ariadne::{Color, Label, Report, ReportKind, Source};
use chumsky::error::Rich;

pub fn print_parse_errors(src: &str, errors: &[Rich<char>]) {
    for err in errors {
        let span = err.span().into_range();
        Report::build(ReportKind::Error, ("src", span.clone()))
            .with_message(err.to_string())
            .with_label(
                Label::new(("src", span))
                    .with_message(format!("{}", err.reason()))
                    .with_color(Color::BrightBlue),
            )
            .finish()
            .eprint(("src", Source::from(src)))
            .unwrap();
    }
}
