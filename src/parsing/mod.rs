use colored::Colorize;
use std::{fs::File, io::Read};

use log::error;
use pest::{
    Parser,
    error::{Error, ErrorVariant, InputLocation},
};
use pest_derive::Parser;

use crate::parsing::ast::{cst_to_ast, nodes::Node};

pub mod ast;

#[derive(Parser)]
#[grammar = "grammar.pest"]
pub struct CrispParser;

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
    let (start, end) = match err.location {
        InputLocation::Pos(pos) => (pos, pos),
        InputLocation::Span((s, e)) => (s, e),
    };

    let msg = match &err.variant {
        ErrorVariant::ParsingError { .. } => "Parsing error".to_string(),
        ErrorVariant::CustomError { message } => message.clone(),
    };

    let line_num = source[..start].chars().filter(|&c| c == '\n').count() + 1;
    let line_start = source[..start].rfind('\n').map(|i| i + 1).unwrap_or(0);
    let line_end = source[start..]
        .find('\n')
        .map(|i| start + i)
        .unwrap_or(source.len());
    let line_text = &source[line_start..line_end];

    let local_start = start - line_start;
    let local_end = (end - line_start).max(local_start + 1);

    let mut output = format!("{}\n", msg.red().bold());

    output.push_str(&format!(
        "  --> {}:{}:{}\n",
        path.blue(),
        line_num,
        local_start + 1
    ));

    output.push_str(&format!("   {}\n", "|").blue());
    output.push_str(&format!(
        "{:>2} {} {}\n",
        line_num.to_string().blue(),
        "|".blue(),
        line_text
    ));

    let underline: String = (0..line_text.len())
        .map(|i| {
            if i >= local_start && i < local_end {
                '^'
            } else {
                ' '
            }
        })
        .collect();
    output.push_str(&format!("   {} {}\n", "|".blue(), underline.red().bold()));

    let hints = detect_common_parse_issues(source);
    if !hints.is_empty() {
        output.push_str(&format!("   {}\n", "|").blue());
        for (index, hint) in hints.iter().enumerate() {
            output.push_str(&format!(
                "   {} {} {}",
                "=>".blue(),
                "Hint:".bold().yellow(),
                hint
            ));
            if index != hints.len() - 1 {
                output.push_str("\n");
            }
        }
    }

    error!("{}", output);
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
            if let Some(..) = string_start {
                if is_triple_quote {
                    string_start = None;
                    is_triple_quote = false;
                }
            } else {
                string_start = Some((line, col));
                is_triple_quote = true;
            }
            i += 3;
            col += 3;
            continue;
        }
        if c == '"' && (i == 0 || chars[i - 1] != '\\') {
            if let Some(_) = string_start {
                if !is_triple_quote {
                    string_start = None;
                }
            } else {
                string_start = Some((line, col));
                is_triple_quote = false;
            }
        } else if string_start.is_none() {
            match c {
                '(' => stack.push((line, col)),
                ')' => {
                    if stack.pop().is_none() {
                        hints.push(format!("Unexpected ')' at [L{}|C{}]", line, col));
                    }
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
