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
mod files;
mod span;
mod symbol_indices;

use compiler::Compiler;
use const_eval::ConstValues;
use files::{File, Files};
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

#[derive(Debug)]
pub enum CompileError {
    Parse {
        path: PathBuf,
        error: daedalus_parser::ParseError,
    },
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CompileError::Parse { path, error } => {
                write!(f, "{}: {error}", path.display())
            }
        }
    }
}

impl std::error::Error for CompileError {}

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
    /// Declarations the engine registers before compiling the unit
    fn prelude(self) -> Option<&'static str> {
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
pub fn compile(unit: CompilationUnit, sources: &[SourceFile]) -> Result<Vec<u8>, CompileError> {
    const PRELUDE_PATH: &str = "<builtin>";

    let mut files_store = Files::new();
    let parse_error = |path: &Path| {
        let path = path.to_path_buf();
        move |error| CompileError::Parse { path, error }
    };

    let prelude = unit
        .prelude()
        .map(|prelude| {
            files_store
                .parse_builtins(PRELUDE_PATH, prelude)
                .map_err(parse_error(Path::new(PRELUDE_PATH)))
        })
        .transpose()?;

    let sources = sources
        .iter()
        .map(|file| {
            files_store
                .parse(&file.path, &file.source)
                .map_err(parse_error(&file.path))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let files: Vec<File> = prelude.into_iter().chain(sources).collect();

    let symbol_map = SymbolIndices::build(&files);
    let const_values = ConstValues::build(&files, &symbol_map);
    Ok(Compiler::new(symbol_map, const_values).build(&files, &files_store))
}
