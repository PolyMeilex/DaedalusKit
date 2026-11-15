//! Code spans of symbols, stored in the `.DAT` file for debugging purposes

use std::ops::Range;

use dat_file::properties::SymbolCodeSpan;

use crate::files::{FileId, Files};

/// What a span belongs to.
///
/// Codespans produced by ZenGin are either hard to understand, or straight up broken, so with
/// the `code-span-compat` feature their quirks are reproduced per kind of symbol, and without it
/// spans are exactly what the parser saw.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpanKind {
    /// Class, instance, prototype or function
    Block,
    Const,
    ClassField,
}

impl SpanKind {
    /// Characters ZenGin adds to the end of the span, for no apparent reason
    fn char_count_padding(self) -> u32 {
        match self {
            SpanKind::Block => 2,
            SpanKind::Const | SpanKind::ClassField => 3,
        }
    }
}

impl Files<'_> {
    /// Span of a symbol defined at `span` bytes of `file`, with 1-based line numbers
    pub fn symbol_span(&self, file: FileId, span: &Range<usize>, kind: SpanKind) -> SymbolCodeSpan {
        let compat = cfg!(feature = "code-span-compat");

        let line_start = self.line_index(file, span.start as u32);
        let line_end = self.line_index(file, span.end as u32);

        let char_start = if compat && kind == SpanKind::ClassField {
            // Class field spans start at the beginning of the line, which is plain wrong if 2
            // fields are on the same line, but that's what ZenGin does
            self.line_span(file, line_start)
                .expect("line index comes from this file")
                .start()
                .0
        } else {
            span.start as u32
        };

        let mut char_count = span.end as u32 - char_start;
        if compat {
            char_count += kind.char_count_padding();
        }

        SymbolCodeSpan::new(
            file.raw(),
            (line_start.0 + 1, line_end.0 - line_start.0 + 1),
            (char_start, char_count),
        )
    }
}
