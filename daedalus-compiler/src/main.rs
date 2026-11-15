use std::path::Path;

use daedalus_compiler::{CompilationUnit, SourceFile};

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

    let dat = daedalus_compiler::compile(unit, &sources).unwrap_or_else(|err| fail(err));

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

fn fail(message: impl std::fmt::Display) -> ! {
    eprintln!("error: {message}");
    std::process::exit(1);
}
