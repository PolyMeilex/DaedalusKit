use std::path::Path;

use daedalus_compiler::SourceFile;

const USAGE: &str = "\
usage: daedalus-compiler -o <out.DAT> <input>...

Inputs are compiled in order. A `.src` input is expanded into the files it lists.";

fn main() {
    let Some((output, inputs)) = parse_args(std::env::args().skip(1)) else {
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

    let dat = daedalus_compiler::compile(&sources).unwrap_or_else(|err| fail(err));

    if let Err(err) = std::fs::write(&output, dat) {
        fail(format_args!("failed to write {output}: {err}"));
    }
}

/// `-o <output>` and a non-empty list of inputs, in any order
fn parse_args(mut args: impl Iterator<Item = String>) -> Option<(String, Vec<String>)> {
    let mut output = None;
    let mut inputs = Vec::new();

    while let Some(arg) = args.next() {
        if arg == "-o" {
            output = Some(args.next()?);
        } else {
            inputs.push(arg);
        }
    }

    Some((output?, inputs)).filter(|(_, inputs)| !inputs.is_empty())
}

fn fail(message: impl std::fmt::Display) -> ! {
    eprintln!("error: {message}");
    std::process::exit(1);
}
