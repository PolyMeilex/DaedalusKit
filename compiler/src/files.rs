#![allow(unused)]

use std::ffi::{OsStr, OsString};

use codespan::{ByteIndex, LineIndex, Location};
use codespan_reporting::files::Error;

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FileId {
    id: u32,
    inner: codespan::FileId,
}

impl FileId {
    pub fn raw(&self) -> u32 {
        if cfg!(feature = "code-span-compat") {
            // In zengin first file and builtins share 0 as their id
            self.id.saturating_sub(1)
        } else {
            self.id
        }
    }
}

#[derive(Debug, Default)]
pub struct Files<'a> {
    inner: codespan::Files<&'a str>,
    len: usize,
}

impl<'a> Files<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add(&mut self, name: impl Into<OsString>, source: &'a str) -> FileId {
        self.len += 1;
        FileId {
            inner: self.inner.add(name, source),
            id: self.len as u32 - 1,
        }
    }

    /// Get the name of the source file.
    ///
    /// ```rust
    /// use codespan::Files;
    ///
    /// let name = "test";
    ///
    /// let mut files = Files::new();
    /// let file_id = files.add(name, "hello world!");
    ///
    /// assert_eq!(files.name(file_id), name);
    /// ```
    pub fn name(&self, file_id: FileId) -> &OsStr {
        self.inner.name(file_id.inner)
    }

    /// Get the span at the given line index.
    ///
    /// ```rust
    /// use codespan::{Files, LineIndex, Span};
    ///
    /// let mut files = Files::new();
    /// let file_id = files.add("test", "foo\nbar\r\n\nbaz");
    ///
    /// let line_sources = (0..4)
    ///     .map(|line| files.line_span(file_id, line).unwrap())
    ///     .collect::<Vec<_>>();
    ///
    /// assert_eq!(line_sources,
    ///     [
    ///         Span::new(0, 4),    // 0: "foo\n"
    ///         Span::new(4, 9),    // 1: "bar\r\n"
    ///         Span::new(9, 10),   // 2: ""
    ///         Span::new(10, 13),  // 3: "baz"
    ///     ]
    /// );
    /// assert!(files.line_span(file_id, 4).is_err());
    /// ```
    pub fn line_span(
        &self,
        file_id: FileId,
        line_index: impl Into<codespan::LineIndex>,
    ) -> Result<codespan::Span, Error> {
        self.inner.line_span(file_id.inner, line_index)
    }

    /// Get the line index at the given byte in the source file.
    ///
    /// ```rust
    /// use codespan::{Files, LineIndex};
    ///
    /// let mut files = Files::new();
    /// let file_id = files.add("test", "foo\nbar\r\n\nbaz");
    ///
    /// assert_eq!(files.line_index(file_id, 0), LineIndex::from(0));
    /// assert_eq!(files.line_index(file_id, 7), LineIndex::from(1));
    /// assert_eq!(files.line_index(file_id, 8), LineIndex::from(1));
    /// assert_eq!(files.line_index(file_id, 9), LineIndex::from(2));
    /// assert_eq!(files.line_index(file_id, 100), LineIndex::from(3));
    /// ```
    pub fn line_index(&self, file_id: FileId, byte_index: impl Into<ByteIndex>) -> LineIndex {
        self.inner.line_index(file_id.inner, byte_index)
    }

    /// Get the location at the given byte index in the source file.
    ///
    /// ```rust
    /// use codespan::{ByteIndex, Files, Location, Span};
    ///
    /// let mut files = Files::new();
    /// let file_id = files.add("test", "foo\nbar\r\n\nbaz");
    ///
    /// assert_eq!(files.location(file_id, 0).unwrap(), Location::new(0, 0));
    /// assert_eq!(files.location(file_id, 7).unwrap(), Location::new(1, 3));
    /// assert_eq!(files.location(file_id, 8).unwrap(), Location::new(1, 4));
    /// assert_eq!(files.location(file_id, 9).unwrap(), Location::new(2, 0));
    /// assert!(files.location(file_id, 100).is_err());
    /// ```
    pub fn location(
        &self,
        file_id: FileId,
        byte_index: impl Into<ByteIndex>,
    ) -> Result<Location, Error> {
        self.inner.location(file_id.inner, byte_index)
    }

    /// Get the source of the file.
    ///
    /// ```rust
    /// use codespan::Files;
    ///
    /// let source = "hello world!";
    ///
    /// let mut files = Files::new();
    /// let file_id = files.add("test", source);
    ///
    /// assert_eq!(*files.source(file_id), source);
    /// ```
    pub fn source(&self, file_id: FileId) -> &'a str {
        self.inner.source(file_id.inner)
    }

    /// Return the span of the full source.
    ///
    /// ```rust
    /// use codespan::{Files, Span};
    ///
    /// let source = "hello world!";
    ///
    /// let mut files = Files::new();
    /// let file_id = files.add("test", source);
    ///
    /// assert_eq!(files.source_span(file_id), Span::from_str(source));
    /// ```
    pub fn source_span(&self, file_id: FileId) -> codespan::Span {
        self.inner.source_span(file_id.inner)
    }

    /// Return a slice of the source file, given a span.
    ///
    /// ```rust
    /// use codespan::{Files, Span};
    ///
    /// let mut files = Files::new();
    /// let file_id = files.add("test",  "hello world!");
    ///
    /// assert_eq!(files.source_slice(file_id, Span::new(0, 5)).unwrap(), "hello");
    /// assert!(files.source_slice(file_id, Span::new(0, 100)).is_err());
    /// ```
    pub fn source_slice(
        &self,
        file_id: FileId,
        span: impl Into<codespan::Span>,
    ) -> Result<&str, Error> {
        self.inner.source_slice(file_id.inner, span)
    }
}
