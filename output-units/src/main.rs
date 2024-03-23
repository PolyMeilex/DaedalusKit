use std::io::Write;

use bstr::{BString, ByteSlice};
use lexer::{DaedalusLexer, Token, TokenError, TokenErrorKind};
use output_units::OutputUnits;

fn main() {
    let mut units = OutputUnits::new();

    let mut files = codespan_reporting::files::SimpleFiles::new();
    let mut errors = Vec::new();

    let src = src_file::load("/home/poly/Downloads/G2MDK-PolishScripts-master/Content/Gothic.src");
    let files_bytes: Vec<_> = src
        .iter()
        .map(|path| (path, std::fs::read(path).unwrap()))
        .collect();
    let files_utf8: Vec<_> = files_bytes
        .iter()
        .map(|(path, bytes)| (path, encoding_rs::WINDOWS_1250.decode(bytes).0))
        .collect();

    for (path, src) in files_utf8.iter() {
        let name = path.file_name().unwrap().to_str().unwrap();
        let id = files.add(name, src.as_ref());

        if let Err(err) = load_file(src, &mut units) {
            errors.push((id, err));
        }
    }

    let mut out = Vec::new();
    units.encode(&mut out).unwrap();

    let str = BString::new(out);

    if errors.is_empty() {
        std::io::stdout().write_all(str.as_slice()).unwrap();
    } else {
        for (id, err) in errors {
            emit_error(&files, id, &err);
        }
    }
}

fn load_file(str: &str, units: &mut OutputUnits) -> Result<(), TokenError> {
    let mut lexer = lexer::DaedalusLexer::new(str);

    loop {
        let token = lexer.peek();

        let token = match token {
            Ok(token) => token,
            Err(err) => {
                if let TokenErrorKind::EOF = err.kind {
                    return Ok(());
                } else {
                    return Err(err);
                }
            }
        };

        match token {
            Token::Ident => {
                let ident = lexer.eat_ident().unwrap();

                if ident.to_uppercase() != "AI_OUTPUT" {
                    continue;
                }

                parse_ai_output(&mut lexer, units)?;
            }
            Token::Instance => {
                lexer.eat_token(Token::Instance).unwrap();
                lexer.eat_ident().unwrap();
                lexer.eat_token(Token::OpenParen).unwrap();
                let ident = lexer.eat_ident().unwrap();
                lexer.eat_token(Token::CloseParen).unwrap();

                if ident.to_uppercase() != "C_SVM" {
                    continue;
                }

                parse_svm_block(&mut lexer, units)?;
            }
            _ => {
                lexer.eat_one().ok();
            }
        }
    }
}

fn parse_ai_output(lexer: &mut DaedalusLexer, units: &mut OutputUnits) -> Result<(), TokenError> {
    lexer.eat_while(|token| *token != Token::Comma);
    lexer.eat_token(Token::Comma).unwrap();

    lexer.eat_while(|token| *token != Token::Comma);
    lexer.eat_token(Token::Comma).unwrap();

    let id = lexer.eat_string().unwrap();

    lexer.eat_token(Token::CloseParen).unwrap();

    lexer.eat_while(|token| *token != Token::Semi);
    lexer.eat_token(Token::Semi).unwrap();

    let text = if lexer.peek_with_comments().ok() == Some(Token::LineComment) {
        Some(lexer.eat_line_comment()?)
    } else {
        None
    };

    let (id, _, _) = encoding_rs::WINDOWS_1250.encode(id);
    let (text, _, _) = encoding_rs::WINDOWS_1250.encode(text.unwrap_or(""));

    units.push(id.as_bstr(), text.as_bstr());

    Ok(())
}

fn parse_svm_block(lexer: &mut DaedalusLexer, units: &mut OutputUnits) -> Result<(), TokenError> {
    lexer.eat_token(Token::OpenBrace).unwrap();

    fn handle_nest_level(level: &mut usize, token: &Token) {
        match token {
            Token::OpenBrace => {
                *level += 1;
            }
            Token::CloseBrace => {
                *level -= 1;
            }
            _ => {}
        }
    }

    let mut nest_level = 1;
    loop {
        if nest_level == 0 {
            break;
        }

        let token = lexer.eat_one().unwrap();
        handle_nest_level(&mut nest_level, &token);
        if token != Token::Ident {
            continue;
        }

        let token = lexer.eat_one().unwrap();
        handle_nest_level(&mut nest_level, &token);
        if token != Token::Eq {
            continue;
        }

        if lexer.peek()? != Token::String {
            continue;
        }

        let id = lexer.eat_string()?;

        lexer.eat_token(Token::Semi).unwrap();

        let text = if lexer.peek_with_comments().ok() == Some(Token::LineComment) {
            Some(lexer.eat_line_comment()?)
        } else {
            None
        };

        let (id, _, _) = encoding_rs::WINDOWS_1250.encode(id);
        let (text, _, _) = encoding_rs::WINDOWS_1250.encode(text.unwrap_or(""));

        units.push(id.as_bstr(), text.as_bstr());
    }

    Ok(())
}

fn emit_error(
    files: &codespan_reporting::files::SimpleFiles<&str, &str>,
    file_id: usize,
    err: &TokenError,
) {
    use codespan_reporting::diagnostic::{Diagnostic, Label};
    use codespan_reporting::term;
    use codespan_reporting::term::termcolor::{ColorChoice, StandardStream};

    let labels = vec![Label::primary(file_id, err.span().clone()).with_message(err.to_string())];

    let diagnostic = Diagnostic::error()
        .with_message(err.to_string())
        .with_labels(labels);

    let writer = StandardStream::stderr(ColorChoice::Always);
    let config = codespan_reporting::term::Config::default();

    term::emit(&mut writer.lock(), &config, files, &diagnostic).unwrap();
}
