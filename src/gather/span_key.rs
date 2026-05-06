//! Stable cross-compilation key for `Span`s.
//!
//! `FirstPassInfo` is built in compilation pass 1 and consulted in pass 2.
//! Raw `Span` values carry a `SyntaxContext` and reference the active
//! `SourceMap`'s `BytePos` interning, both of which are recreated each
//! invocation. They happen to match across passes today because the file
//! loader registers source files identically, but that's an unwritten
//! invariant. `SpanKey` is a `(file, lo, hi)` triple computed by resolving
//! the span through the `SourceMap` to the *file-local* byte offsets,
//! which are a pure function of the source bytes.
//!
//! Spans inside macro expansions / desugarings are normalized via
//! `source_callsite()` so a query on a syntactic AST node still hits the
//! recorded entry — see also task #3.

use std::path::PathBuf;

use rustc_span::{FileName, RemapPathScopeComponents, Span};
use rustc_span::source_map::SourceMap;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpanKey {
    /// Path-string identifier for the source file. Derived from `FileName`
    /// in a way that is invariant across the local/remapped/scopes flags
    /// of `RealFileName`, since pass 1 (rustc's normal file loading) and
    /// pass 2 (our `from_virtual_path`-based parser) populate those
    /// differently for the same underlying file.
    file: FileKey,
    lo: u32,
    hi: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum FileKey {
    Real(PathBuf),
    Other(String),
}

impl FileKey {
    fn from_filename(name: &FileName) -> Self {
        match name {
            // `path(MACRO)` returns the same `name` PathBuf regardless of
            // whether `local` is populated and whether the file went through
            // remapping — both pass 1 (rustc-loaded) and pass 2 (our
            // `from_virtual_path`-loaded) yield the file-relative path here.
            FileName::Real(rfn) => FileKey::Real(rfn.path(RemapPathScopeComponents::MACRO).to_path_buf()),
            other => FileKey::Other(format!("{other:?}")),
        }
    }
}

impl SpanKey {
    /// Resolve `span` to a stable `(file, lo, hi)` key. Collapses
    /// macro-expansion / desugaring spans to their call site so pass 2
    /// queries on the syntactic AST find the entry pass 1 recorded.
    /// Returns `None` for dummy spans or spans whose call site cannot
    /// be located in any registered source file.
    pub fn from_span(span: Span, sm: &SourceMap) -> Option<Self> {
        let span = span.source_callsite();
        if span.is_dummy() {
            return None;
        }
        let lo = sm.lookup_byte_offset(span.lo());
        let hi = sm.lookup_byte_offset(span.hi());
        Some(Self {
            file: FileKey::from_filename(&lo.sf.name),
            lo: lo.pos.0,
            hi: hi.pos.0,
        })
    }
}