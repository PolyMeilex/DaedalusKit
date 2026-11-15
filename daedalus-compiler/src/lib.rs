use std::{
    io,
    path::{Path, PathBuf},
    str::FromStr,
};

mod builtin;
mod codegen;
mod compiler;
mod const_eval;
mod dat_symbol_table;
mod error;
mod files;
mod span;
mod symbol_indices;

use compiler::Compiler;
use const_eval::ConstValues;
use error::Diagnostics;
pub use error::{CompileError, ErrorKind};
use files::Files;
use symbol_indices::SymbolIndices;

/// Script source file, decoded from the Windows-1250 encoding used by Gothic scripts
#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub source: String,
}

impl SourceFile {
    pub fn read(path: impl Into<PathBuf>) -> io::Result<Self> {
        let path = path.into();
        let bytes = std::fs::read(&path)?;
        let (source, _, _) = encoding_rs::WINDOWS_1250.decode(&bytes);

        Ok(Self {
            path,
            source: source.into_owned(),
        })
    }

    /// Read every file listed in a `.src` file, in order
    pub fn read_src(src_path: impl AsRef<Path>) -> io::Result<Vec<Self>> {
        src_file::load(src_path)
            .into_iter()
            .map(Self::read)
            .collect()
    }
}

/// Each `.src` file is compiled by a separate engine parser, which comes with its own set of
/// builtin definitions
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompilationUnit {
    /// `Content/Gothic.src`
    Gothic,
    /// `System/Camera.src`
    Camera,
}

impl CompilationUnit {
    /// Path under which errors in the prelude are reported
    pub const PRELUDE_PATH: &'static str = "<builtin>";

    /// Declarations the engine registers before compiling the unit
    pub fn prelude(self) -> Option<&'static str> {
        match self {
            CompilationUnit::Gothic => Some(builtin::GOTHIC_PRELUDE),
            CompilationUnit::Camera => None,
        }
    }
}

impl FromStr for CompilationUnit {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "gothic" => Ok(Self::Gothic),
            "camera" => Ok(Self::Camera),
            _ => Err(()),
        }
    }
}

/// Compile source files, in order, into the encoded `.DAT` file
pub fn compile(
    unit: CompilationUnit,
    sources: &[SourceFile],
) -> Result<Vec<u8>, Vec<CompileError>> {
    let mut files_store = Files::new();
    let mut parse_errors = Vec::new();
    let mut parse_error = |path: &Path, error: daedalus_parser::ParseError| {
        parse_errors.push(CompileError {
            path: path.to_path_buf(),
            span: error.span().clone(),
            kind: ErrorKind::Parse(error),
        });
    };

    let mut files = Vec::with_capacity(sources.len() + 1);
    if let Some(prelude) = unit.prelude() {
        match files_store.parse_builtins(CompilationUnit::PRELUDE_PATH, prelude) {
            Ok(file) => files.push(file),
            Err(error) => parse_error(Path::new(CompilationUnit::PRELUDE_PATH), error),
        }
    }
    for source in sources {
        match files_store.parse(&source.path, &source.source) {
            Ok(file) => files.push(file),
            Err(error) => parse_error(&source.path, error),
        }
    }

    // Name resolution relies on every file being present
    if !parse_errors.is_empty() {
        return Err(parse_errors);
    }

    let mut diagnostics = Diagnostics::new(&files_store);
    let symbol_map = SymbolIndices::build(&files);
    let const_values = ConstValues::build(&files, &symbol_map, &mut diagnostics);
    let dat = Compiler::new(symbol_map, const_values).build(&files, &files_store, &mut diagnostics);

    let errors = diagnostics.into_errors();
    if errors.is_empty() {
        Ok(dat)
    } else {
        Err(errors)
    }
}
