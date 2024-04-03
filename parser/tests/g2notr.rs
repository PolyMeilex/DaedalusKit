use daedalus_lexer::{DaedalusLexer, Token};
use parser::{fmt::DaedalusFormatter, parse::File};
use pretty_assertions::assert_eq;

#[test]
fn parser_g2notr_diff() {
    fn diff(_path: &std::path::Path, src: &str) {
        let ast = File::parse(&mut DaedalusLexer::new(src)).unwrap();
        let mut out = String::new();
        DaedalusFormatter::new(&mut out).format(ast).unwrap();

        let mut src_lex = DaedalusLexer::new(src.trim_end());
        let mut out_lex = DaedalusLexer::new(out.trim_end());

        loop {
            let src_token = match src_lex.eat_any() {
                Ok(Token::Eof) => break,
                Ok(v) => v,
                Err(err) => panic!("{err:?}"),
            };
            let out_token = out_lex.eat_any().unwrap();

            // if src_token != out_token {
            //     emit_error(
            //         &format!("{} (src)", path.to_string_lossy()),
            //         src.trim_end(),
            //         &format!("{out_token:?} != {src_token}"),
            //         src_lex.span(),
            //     );
            //     emit_error(
            //         &format!("{} (out)", path.to_string_lossy()),
            //         out.trim_end(),
            //         &format!("{out_token:?} != {src_token}"),
            //         out_lex.span(),
            //     );
            //     println!();
            // }
            assert_eq!(src_token, out_token);
        }
    }

    let base_path = "../test_data/G2MDK-PolishScripts/Content/";
    let mut src = src_file::load(format!("{base_path}Gothic.src"));
    src.append(&mut src_file::load(format!("{base_path}Fight.src")));

    let len = src.len();

    for (id, path) in src.into_iter().enumerate() {
        let bytes = std::fs::read(&path).unwrap();
        let path = path.strip_prefix(base_path).unwrap();
        println!("{path:?} ({} / {len})", id + 1);

        let (src, _, _) = encoding_rs::WINDOWS_1250.decode(&bytes);
        diff(path, &src);
    }
}

// fn emit_error(path: &str, src: &str, err: &str, span: logos::Span) {
//     use codespan_reporting::diagnostic::{Diagnostic, Label};
//     use codespan_reporting::files::SimpleFiles;
//     use codespan_reporting::term;
//     use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};
//
//     let mut files = SimpleFiles::new();
//
//     let file_id = files.add(path, src);
//
//     let labels = vec![Label::primary(file_id, span).with_message(err.to_string())];
//
//     let diagnostic = Diagnostic::error()
//         .with_message(err.to_string())
//         .with_labels(labels);
//
//     let writer = StandardStream::stderr(ColorChoice::Always);
//     let config = codespan_reporting::term::Config::default();
//
//     term::emit(&mut writer.lock(), &config, &files, &diagnostic).unwrap();
// }
