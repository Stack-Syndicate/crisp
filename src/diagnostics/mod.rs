use crate::parsing::{ast::Rule, ast::nodes::SourceInfo};
use colored::Colorize;
use log::error;
use pest::error::{Error, ErrorVariant, InputLocation};

struct Diagnostic {
    title: String,
    path: String,
    line: usize,
    column: usize,
    line_text: String,
    underline_start: usize,
    underline_end: usize,
    hints: Vec<String>,
}

pub fn print_pest_error(err: Error<Rule>, path: &str, source: &str) {
    let (start, end) = match err.location {
        InputLocation::Pos(pos) => (pos, pos),
        InputLocation::Span((s, e)) => (s, e),
    };
    let title = match &err.variant {
        ErrorVariant::ParsingError { .. } => "Pest parsing error".to_string(),
        ErrorVariant::CustomError { message } => message.clone(),
    };
    let (line, line_start, column) = resolve_position(source, start);
    let line_text = extract_line(source, line_start);
    let hints = detect_common_parse_issues(source);
    let diag = Diagnostic {
        title,
        path: path.to_string(),
        line,
        column,
        line_text,
        underline_start: start - line_start,
        underline_end: (end - line_start).max((start - line_start) + 1),
        hints,
    };
    render_diagnostic(diag);
}

pub fn print_error(msg: &str, info: &SourceInfo) {
    let start = info.start;
    let (line, line_start, column) = resolve_position(&info.file.source, start);
    let line_text = extract_line(&info.file.source, line_start);
    let span_len = (info.end - info.start).max(1);
    let diag = Diagnostic {
        title: msg.to_string(),
        path: info.file.path.to_string(),
        line,
        column,
        line_text,
        underline_start: info.col.saturating_sub(1),
        underline_end: info.col.saturating_sub(1) + span_len,
        hints: Vec::new(),
    };
    render_diagnostic(diag);
}

fn render_diagnostic(diag: Diagnostic) {
    let mut output = String::new();
    output.push_str(&format!("{}\n", diag.title.red().bold()));
    output.push_str(&format!(
        "  --> {}:{}:{}\n",
        diag.path.blue(),
        diag.line,
        diag.column
    ));
    output.push_str(&format!("   {}\n", "|").blue());
    output.push_str(&format!(
        "{:>2} {} {}\n",
        diag.line.to_string().blue(),
        "|".blue(),
        diag.line_text
    ));
    output.push_str(&format!(
        "   {} {}\n",
        "|".blue(),
        underline(&diag.line_text, diag.underline_start, diag.underline_end)
    ));
    if !diag.hints.is_empty() {
        output.push_str(&format!("   {}\n", "|").blue());
        for hint in diag.hints {
            output.push_str(&format!(
                "   {} {} {}",
                "=>".blue(),
                "Hint:".bold().yellow(),
                hint
            ));
            output.push('\n');
        }
    }
    error!("{}", output);
}

fn underline(line: &str, start: usize, end: usize) -> String {
    line.chars()
        .enumerate()
        .map(|(i, _)| if i >= start && i < end { '^' } else { ' ' })
        .collect::<String>()
        .red()
        .bold()
        .to_string()
}

fn resolve_position(source: &str, pos: usize) -> (usize, usize, usize) {
    let pos = pos.min(source.len());
    let line_start = source[..pos].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line = source[..line_start].matches('\n').count() + 1;
    let column = pos - line_start + 1;
    (line, line_start, column)
}

fn extract_line(source: &str, start: usize) -> String {
    let slice = &source[start..];
    match slice.find('\n') {
        Some(i) => slice[..i].to_string(),
        None => slice.to_string(),
    }
}

fn detect_common_parse_issues(source: &str) -> Vec<String> {
    let mut hints = Vec::new();
    let mut stack = Vec::new();
    let mut string_start = None;
    let mut is_triple_quote = false;
    let chars: Vec<char> = source.chars().collect();
    let (mut line, mut col) = (1, 1);
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if i + 2 < chars.len() && chars[i..i + 3] == ['"', '"', '"'] {
            if string_start.is_some() && is_triple_quote {
                string_start = None;
                is_triple_quote = false;
            } else {
                string_start = Some((line, col));
                is_triple_quote = true;
            }
            i += 3;
            col += 3;
            continue;
        }
        if c == '"' && (i == 0 || chars[i - 1] != '\\') {
            match string_start {
                Some(_) if !is_triple_quote => string_start = None,
                None => {
                    string_start = Some((line, col));
                    is_triple_quote = false;
                }
                _ => {}
            }
        } else if string_start.is_none() {
            match c {
                '(' => stack.push((line, col)),
                ')' if stack.pop().is_none() => {
                    hints.push(format!("Unexpected ')' at [L{}|C{}]", line, col));
                }
                _ => {}
            }
        }

        if c == '\n' {
            line += 1;
            col = 1;
        } else {
            col += 1;
        }

        i += 1;
    }
    if let Some((l, c)) = string_start {
        let label = if is_triple_quote { "\"\"\"" } else { "\"" };
        hints.push(format!("Unclosed {} starting at [L{}|C{}]", label, l, c));
    }
    while let Some((l, c)) = stack.pop() {
        hints.push(format!("Unclosed '(' at [L{}|C{}]", l, c));
    }
    hints
}
