//! Retained non-owning TON618 compatibility primitive crate for `v1`.
//!
//! Canonical ownership of public platform contracts lives in the `sm-*` crates.
//! This crate keeps the historical `ton618-core` name only as part of the
//! retained compatibility perimeter and must not become a second public owner.

#![allow(clippy::len_without_is_empty)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "alloc")]
pub mod arena;
pub mod diagnostics;
pub mod ids;
#[cfg(feature = "alloc")]
pub mod sigtable;
pub mod source;

#[cfg(feature = "alloc")]
pub use arena::{Arena, ArenaId};
pub use diagnostics::{DiagLevel, Diagnostic};
pub use ids::{ExprId, StmtId, SymbolId};
#[cfg(feature = "alloc")]
pub use sigtable::SigTable;
pub use source::{FileId, SourceMark, Span};
#[cfg(feature = "alloc")]
pub use source::{SourceFile, SourceMap};
