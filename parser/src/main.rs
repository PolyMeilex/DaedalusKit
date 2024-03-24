#![allow(clippy::single_match)]

use std::process::exit;

use fmt::DaedalusFormatter;
use lexer::{DaedalusLexer, Token, TokenErrorKind};

mod fmt;
pub mod parse;

pub type ParseError = lexer::TokenError;

fn main() {
    // let bytes = include_bytes!("../DIA_vlk_439_vatras.d");
    let bytes = include_bytes!("../../DIA_bau_950_lobart.d");
    // let bytes = include_bytes!("../../lobart.d");

    // let bytes = include_bytes!("../a1.d");
    let (src, enc, _) = encoding_rs::WINDOWS_1250.decode(bytes);
    dbg!(enc);

    let mut lexer = DaedalusLexer::new(&src);

    let emit_error = |err: &ParseError| emit_error(&src, err);
    let mut formatter = DaedalusFormatter::default();

    while let Ok(token) = lexer.peek() {
        match token {
            Token::Class => {
                if let Ok(instance) = parse::Class::parse(&mut lexer).inspect_err(emit_error) {
                    formatter.format(instance).unwrap();
                } else {
                    exit(1);
                }
            }
            Token::Instance => {
                if let Ok(instance) = parse::Instance::parse(&mut lexer).inspect_err(emit_error) {
                    formatter.format(instance).unwrap();
                } else {
                    exit(1);
                }
            }
            Token::Var => {
                if let Ok(var) = parse::VarDeclaration::parse(&mut lexer).inspect_err(emit_error) {
                    formatter.format(var).unwrap();
                } else {
                    exit(1);
                }
            }
            Token::Func => {
                if let Ok(func) =
                    parse::FunctionDefinition::parse(&mut lexer).inspect_err(emit_error)
                {
                    formatter.format(func).unwrap();
                } else {
                    exit(1);
                }
            }
            _ => {
                lexer.eat_any().unwrap();
            }
        }
    }
}

fn emit_error(src: &str, err: &ParseError) {
    use codespan_reporting::diagnostic::{Diagnostic, Label};
    use codespan_reporting::files::SimpleFiles;
    use codespan_reporting::term;
    use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

    let mut files = SimpleFiles::new();

    let file_id = files.add("file.d", src);

    let mut labels =
        vec![Label::primary(file_id, err.span().clone()).with_message(err.to_string())];

    match &err.kind {
        TokenErrorKind::ExpectedToken {
            expected: Token::Semi,
            ..
        } => {
            let mut secondary = err.span().clone();

            let slice = src.get(0..secondary.start).unwrap();

            let mut offset = secondary.start;
            for (id, ch) in slice.char_indices().rev() {
                if !ch.is_whitespace() {
                    offset = id + 1;
                    break;
                }
            }

            secondary.start = offset;
            secondary.end = offset;

            labels
                .push(Label::secondary(file_id, secondary).with_message("Try to insert ',' here"));
        }
        _ => {}
    }

    let diagnostic = Diagnostic::error()
        .with_message(err.to_string())
        .with_labels(labels);

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
}
