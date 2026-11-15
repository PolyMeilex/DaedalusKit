use std::{collections::HashMap, io::IsTerminal, path::Path};

use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFiles,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};
use daedalus_compiler::{CompilationUnit, CompileError, SourceFile};

const USAGE: &str = "\
usage: daedalus-compiler --unit <gothic|camera> -o <out.DAT> <input>...

Inputs are compiled in order. A `.src` input is expanded into the files it lists.
The unit selects the builtin definitions the engine provides to the scripts.";

fn main() {
    let Some(Args {
        unit,
        output,
        inputs,
    }) = Args::parse(std::env::args().skip(1))
    else {
        eprintln!("{USAGE}");
        std::process::exit(2);
    };

    let mut sources = Vec::new();
    for input in inputs.iter().map(Path::new) {
        let read = if input
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("src"))
        {
            SourceFile::read_src(input)
        } else {
            SourceFile::read(input).map(|file| vec![file])
        };

        match read {
            Ok(mut files) => sources.append(&mut files),
            Err(err) => fail(format_args!("failed to read {}: {err}", input.display())),
        }
    }

    let dat = match daedalus_compiler::compile(unit, &sources) {
        Ok(dat) => dat,
        Err(errors) => {
            report(unit, &sources, &errors);
            fail(format_args!(
                "could not compile due to {} errors",
                errors.len()
            ));
        }
    };

    if let Err(err) = std::fs::write(&output, dat) {
        fail(format_args!("failed to write {output}: {err}"));
    }
}

struct Args {
    unit: CompilationUnit,
    output: String,
    inputs: Vec<String>,
}

impl Args {
    /// Flags and a non-empty list of inputs, in any order
    fn parse(mut args: impl Iterator<Item = String>) -> Option<Self> {
        let mut unit = None;
        let mut output = None;
        let mut inputs = Vec::new();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--unit" => unit = Some(args.next()?.parse().ok()?),
                "-o" => output = Some(args.next()?),
                _ => inputs.push(arg),
            }
        }

        Some(Self {
            unit: unit?,
            output: output?,
            inputs,
        })
        .filter(|args| !args.inputs.is_empty())
    }
}

/// Print errors with the source code they point to
fn report(unit: CompilationUnit, sources: &[SourceFile], errors: &[CompileError]) {
    let mut files = SimpleFiles::new();
    let mut file_ids = HashMap::new();

    let prelude = unit
        .prelude()
        .map(|prelude| (Path::new(CompilationUnit::PRELUDE_PATH), prelude));
    let sources = sources
        .iter()
        .map(|file| (file.path.as_path(), file.source.as_str()));

    for (path, source) in prelude.into_iter().chain(sources) {
        file_ids.insert(path, files.add(path.display().to_string(), source));
    }

    let writer = StandardStream::stderr(if std::io::stderr().is_terminal() {
        ColorChoice::Auto
    } else {
        ColorChoice::Never
    });
    let config = term::Config::default();

    for error in errors {
        let diagnostic = Diagnostic::error().with_message(error.kind.to_string());
        let diagnostic = match file_ids.get(error.path.as_path()) {
            Some(&file_id) => {
                diagnostic.with_labels(vec![Label::primary(file_id, error.span.clone())])
            }
            None => diagnostic.with_notes(vec![format!("in {}", error.path.display())]),
        };

        term::emit(&mut writer.lock(), &config, &files, &diagnostic)
            .expect("failed to write to stderr");
    }
}

fn fail(message: impl std::fmt::Display) -> ! {
    eprintln!("error: {message}");
    std::process::exit(1);
}
