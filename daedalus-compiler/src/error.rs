use std::{fmt, ops::Range, path::PathBuf};

use crate::files::{FileId, Files};

#[derive(Debug)]
pub enum ErrorKind {
    Parse(daedalus_parser::ParseError),
    UndefinedSymbol(String),
    UnknownType(String),
    /// Extern function the engine does not provide
    UnknownExtern(String),
    /// Valid Daedalus that the compiler can not compile yet
    Unsupported(String),
    Invalid(String),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::Parse(err) => write!(f, "{err}"),
            ErrorKind::UndefinedSymbol(name) => write!(f, "undefined symbol `{name}`"),
            ErrorKind::UnknownType(name) => write!(f, "unknown type `{name}`"),
            ErrorKind::UnknownExtern(name) => {
                write!(f, "the engine does not provide extern function `{name}`")
            }
            ErrorKind::Unsupported(what) => write!(f, "not supported yet: {what}"),
            ErrorKind::Invalid(what) => write!(f, "{what}"),
        }
    }
}

/// Error at a byte span of the file currently being compiled
#[derive(Debug)]
pub struct Error {
    pub span: Range<usize>,
    pub kind: ErrorKind,
}

impl Error {
    pub fn new(span: &Range<usize>, kind: ErrorKind) -> Self {
        Self {
            span: span.clone(),
            kind,
        }
    }

    pub fn undefined_symbol(span: &Range<usize>, name: impl Into<String>) -> Self {
        Self::new(span, ErrorKind::UndefinedSymbol(name.into()))
    }

    pub fn unsupported(span: &Range<usize>, what: impl Into<String>) -> Self {
        Self::new(span, ErrorKind::Unsupported(what.into()))
    }

    pub fn invalid(span: &Range<usize>, what: impl Into<String>) -> Self {
        Self::new(span, ErrorKind::Invalid(what.into()))
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct CompileError {
    pub path: PathBuf,
    /// Byte span in the file at `path`
    pub span: Range<usize>,
    pub kind: ErrorKind,
}

impl CompileError {
    pub(crate) fn new(files: &Files, file: FileId, error: Error) -> Self {
        Self {
            path: files.name(file).into(),
            span: error.span,
            kind: error.kind,
        }
    }
}

impl fmt::Display for CompileError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.path.display(), self.kind)
    }
}

impl std::error::Error for CompileError {}

/// Errors collected while compiling, so that compilation can go on and report all of them
pub(crate) struct Diagnostics<'a> {
    files: &'a Files<'a>,
    errors: Vec<CompileError>,
}

impl<'a> Diagnostics<'a> {
    pub fn new(files: &'a Files<'a>) -> Self {
        Self {
            files,
            errors: Vec::new(),
        }
    }

    pub fn report(&mut self, file: FileId, error: Error) {
        self.errors.push(CompileError::new(self.files, file, error));
    }

    pub fn into_errors(self) -> Vec<CompileError> {
        self.errors
    }
}
