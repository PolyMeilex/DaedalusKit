#![allow(clippy::single_match)]

use std::{path::Path, process::exit};

use fmt::DaedalusFormatter;
use lexer::DaedalusLexer;

mod fmt;
pub mod parse;

pub type ParseError = lexer::TokenError;

fn main() {
    // let bytes = include_bytes!("../../DIA_bau_950_lobart.d");
    // let bytes = include_bytes!("../../my.d");
    // parse(&std::path::PathBuf::from("DIA_bau_950_lobart.d"), bytes);

    let base_path = "./test_data/G2MDK-PolishScripts/Content/";
    let mut src = src_file::load(format!("{base_path}Gothic.src"));
    src.append(&mut src_file::load(format!("{base_path}Fight.src")));

    let len = src.len();

    for (id, path) in src.into_iter().enumerate() {
        let bytes = std::fs::read(&path).unwrap();
        let path = path.strip_prefix(base_path).unwrap();
        println!("{path:?} ({} / {len})", id + 1);
        parse(path, &bytes);
    }
}

fn parse(path: &Path, bytes: &[u8]) {
    let (src, _, _) = encoding_rs::WINDOWS_1250.decode(bytes);

    let mut lexer = DaedalusLexer::new(&src);

    let emit_error = |err: &ParseError| emit_error(path, &src, err);
    let mut formatter = DaedalusFormatter::new(fmt::IoFmt(std::io::stdout()));

    let file = match parse::File::parse(&mut lexer) {
        Ok(file) => file,
        Err(err) => {
            emit_error(&err);
            exit(1);
        }
    };

    formatter.format(file).unwrap();
}

fn emit_error(path: &Path, src: &str, err: &ParseError) {
    use codespan_reporting::diagnostic::{Diagnostic, Label};
    use codespan_reporting::files::SimpleFiles;
    use codespan_reporting::term;
    use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

    let mut files = SimpleFiles::new();

    let file_id = files.add(path.to_string_lossy(), src);

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
