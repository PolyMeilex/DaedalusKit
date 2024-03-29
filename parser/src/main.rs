#![allow(clippy::single_match)]

use std::process::exit;

use fmt::DaedalusFormatter;
use lexer::DaedalusLexer;

use crate::parse::Item;

mod fmt;
pub mod parse;

pub type ParseError = lexer::TokenError;

fn main() {
    // let bytes = include_bytes!("../DIA_vlk_439_vatras.d");
    let bytes = include_bytes!("../../DIA_bau_950_lobart.d");
    // let bytes = include_bytes!("../../lobart.d");

    let (src, enc, _) = encoding_rs::WINDOWS_1250.decode(bytes);
    dbg!(enc);

    let mut lexer = DaedalusLexer::new(&src);

    let emit_error = |err: &ParseError| emit_error(&src, err);
    let mut formatter = DaedalusFormatter::default();

    let file = match parse::File::parse(&mut lexer) {
        Ok(file) => file,
        Err(err) => {
            emit_error(&err);
            exit(1);
        }
    };

    for item in file.items {
        match item {
            Item::Class(v) => {
                formatter.format(v).unwrap();
            }
            Item::Instance(v) => {
                formatter.format(v).unwrap();
            }
            Item::Var(v) => {
                formatter.format(v).unwrap();
            }
            Item::Func(v) => {
                formatter.format(v).unwrap();
            }
            Item::Const(v) => {
                formatter.format(v).unwrap();
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

    if err.backtrace().status() == std::backtrace::BacktraceStatus::Captured {
        labels.push(
            Label::secondary(file_id, err.span().clone()).with_message(err.backtrace().to_string()),
        );
    }

    // match &err.kind {
    //     TokenErrorKind::ExpectedToken {
    //         expected: Token::Semi,
    //         ..
    //     } => {
    //         let mut secondary = err.span().clone();
    //
    //         let slice = src.get(0..secondary.start).unwrap();
    //
    //         let mut offset = secondary.start;
    //         for (id, ch) in slice.char_indices().rev() {
    //             if !ch.is_whitespace() {
    //                 offset = id + 1;
    //                 break;
    //             }
    //         }
    //
    //         secondary.start = offset;
    //         secondary.end = offset;
    //
    //         labels
    //             .push(Label::secondary(file_id, secondary).with_message("Try to insert ',' here"));
    //     }
    //     _ => {}
    // }

    let diagnostic = Diagnostic::error()
        .with_message(err.to_string())
        .with_labels(labels);

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
}
