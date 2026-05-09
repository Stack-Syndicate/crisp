use colored::Colorize;
use std::{fs::File, io::Read};

use anyhow::Result;
use log::error;
use pest::{
    Parser, Position,
    error::{Error, ErrorVariant, InputLocation, LineColLocation},
};

use crate::parsing::{
    CrispParser, Rule,
    ast::{cst_to_ast, nodes::Node},
};

pub mod cli;
pub mod parsing;

pub fn parse_file<'a>(path: &'static str) -> Result<Node, String> {
    let mut source = "".to_string();
    File::open(&path)
        .unwrap()
        .read_to_string(&mut source)
        .expect("Could not read the file as source code");
    return parse_str(source, path);
}

pub fn parse_str(source: String, path: &'static str) -> Result<Node, String> {
    if source.is_empty() {
        error!("Source file is empty!");
        return Err("Source file is empty".to_string());
    }
    let pest_cst = CrispParser::parse(Rule::file, &source);

    match pest_cst {
        Ok(mut pairs) => Ok(cst_to_ast(pairs.next().unwrap(), path)),
        Err(e) => {
            print_pest_error(e, path, &source);
            Err("Parse failed; see logs for details.".to_string())
        }
    }
}

pub fn print_pest_error(err: Error<Rule>, path: &str, source: &str) {
    let (mut line, mut col) = match err.line_col {
        LineColLocation::Pos((l, c)) => (l, c),
        LineColLocation::Span(start_lc, _) => start_lc,
    };
    let (mut start, mut end) = match err.location {
        InputLocation::Pos(pos) => (pos, pos),
        InputLocation::Span((s, e)) => (s, e),
    };

    let msg = match &err.variant {
        ErrorVariant::ParsingError { positives, .. } => {
            if positives.contains(&Rule::list) && start >= source.len() {
                "Syntax Error: Unclosed parenthesis".to_string()
            } else {
                format!("Syntax Error: Expected one of {:?}", positives)
            }
        }
        ErrorVariant::CustomError { message } => message.clone(),
    };

    if msg.contains("Unclosed") {
        let mut bracket_count = 0;
        for (i, ch) in source.char_indices().rev() {
            if ch == ')' {
                bracket_count += 1;
            } else if ch == '(' {
                if bracket_count == 0 {
                    // Found the unmatched bracket
                    start = i;
                    end = i + 1;

                    // Update line/col for the actual bracket position
                    let pos = Position::new(source, i).unwrap();
                    let (l, c) = pos.line_col();
                    line = l;
                    col = c;
                    break;
                }
                bracket_count -= 1;
            }
        }
    }

    let line_start = source[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = source[start..]
        .find('\n')
        .map(|i| start + i)
        .unwrap_or(source.len());
    let line_text = &source[line_start..line_end];

    let span_len = (end - start).max(1);
    let indent = " ".repeat(col.saturating_sub(1));
    let pointer = "~".repeat(span_len).red();

    error!(
        "{}\n--> {}[{}|{}]\n{:>4} |\n{:>4} | {}\n     | {}{}",
        msg.bold(),
        path.blue(),
        line.to_string().red(),
        col.to_string().red(),
        "",
        line.to_string().red(),
        line_text,
        indent,
        pointer
    );
}
