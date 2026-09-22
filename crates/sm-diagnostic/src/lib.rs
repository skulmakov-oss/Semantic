#![no_std]

extern crate alloc;

use alloc::borrow::Cow;
use alloc::boxed::Box;
use alloc::string::String;
use alloc::vec::Vec;

/// Error returned when attempting to construct a [`DiagnosticCode`] from an empty or whitespace-only string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EmptyCodeError;

/// Opaque textual diagnostic identifier owned strictly by the producer stage.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DiagnosticCode(Cow<'static, str>);

impl DiagnosticCode {
    /// Attempts to construct a [`DiagnosticCode`] from a static string slice.
    ///
    /// Rejects empty or whitespace-only inputs.
    /// Preserves exact bytes and case without trimming, normalizing, or case-folding.
    pub fn try_from_static(value: &'static str) -> Result<Self, EmptyCodeError> {
        if value.trim().is_empty() {
            return Err(EmptyCodeError);
        }
        Ok(Self(Cow::Borrowed(value)))
    }

    /// Attempts to construct a [`DiagnosticCode`] from an owned [`String`].
    ///
    /// Rejects empty or whitespace-only inputs.
    /// Preserves exact bytes and case without trimming, normalizing, or case-folding.
    pub fn try_from_string(value: String) -> Result<Self, EmptyCodeError> {
        if value.trim().is_empty() {
            return Err(EmptyCodeError);
        }
        Ok(Self(Cow::Owned(value)))
    }

    /// Returns the diagnostic code as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Two-state diagnostic severity vocabulary representing compiler/verifier findings
/// and runtime execution failures explicitly admitted as canonical diagnostics.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticSeverity {
    Error,
    Warning,
}

/// Identifies the originating canonical compiler or runtime stage.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum DiagnosticFamily {
    Frontend,
    Semantic,
    Verification,
    Runtime,
}

/// Opaque internal session token identifying an input source.
///
/// Direct external construction is prohibited; minting authority belongs exclusively
/// to a future canonical source registry/context.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceId(u64);

/// Error returned when a machine-sized offset cannot be converted into a 64-bit [`ByteOffset`].
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct OffsetOverflow;

/// Exact zero-based UTF-8 byte offset within a source.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ByteOffset(u64);

impl ByteOffset {
    /// Attempts to construct a [`ByteOffset`] from a `usize`.
    ///
    /// Checked via `u64::try_from`; no truncating, wrapping, or saturating conversions.
    pub fn try_from_usize(value: usize) -> Result<Self, OffsetOverflow> {
        let val = u64::try_from(value).map_err(|_| OffsetOverflow)?;
        Ok(Self(val))
    }

    /// Returns the raw 64-bit byte offset value.
    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

/// Half-open UTF-8 byte span `[start, end)` within a source.
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceRange {
    start: ByteOffset,
    end: ByteOffset,
}

impl SourceRange {
    /// Constructs a [`SourceRange`] if `start <= end`, returning `None` if inverted (`start > end`).
    pub fn new(start: ByteOffset, end: ByteOffset) -> Option<Self> {
        if start.as_u64() <= end.as_u64() {
            Some(Self { start, end })
        } else {
            None
        }
    }

    /// Returns the starting byte offset.
    pub const fn start(&self) -> ByteOffset {
        self.start
    }

    /// Returns the ending byte offset.
    pub const fn end(&self) -> ByteOffset {
        self.end
    }

    /// Returns `true` if the range is zero-width (`start == end`).
    pub const fn is_empty(&self) -> bool {
        self.start.as_u64() == self.end.as_u64()
    }
}

/// Canonical location binding an opaque [`SourceId`] to an optional byte range.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct SourceContext {
    pub source: SourceId,
    pub range: Option<SourceRange>,
}

/// Error returned when constructing a [`DiagnosticMessage`] from an empty or whitespace-only string.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct EmptyMessageError;

/// Human-readable semantic diagnostic text.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DiagnosticMessage(String);

impl DiagnosticMessage {
    /// Constructs a [`DiagnosticMessage`], rejecting empty or whitespace-only inputs.
    ///
    /// Preserves input bytes exactly without trimming or altering formatting.
    pub fn new(msg: impl Into<String>) -> Result<Self, EmptyMessageError> {
        let msg = msg.into();
        if msg.trim().is_empty() {
            return Err(EmptyMessageError);
        }
        Ok(Self(msg))
    }

    /// Returns the diagnostic message as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Structured secondary source location relevant to the diagnosis.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct RelatedLocation {
    pub context: SourceContext,
    pub message: Option<String>,
}

/// Structured supplemental explanation or contextual hint.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct DiagnosticNote {
    pub message: String,
}

/// Structured guidance providing human or procedural guidance.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct FixProposal {
    pub message: String,
}

/// Causal link preserving nested lower-level diagnostics structurally.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DiagnosticCause {
    Diagnostic(Box<Diagnostic>),
    Report(Vec<Diagnostic>),
}

/// Canonical internal diagnostic carrier.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Diagnostic {
    pub code: DiagnosticCode,
    pub severity: DiagnosticSeverity,
    pub family: DiagnosticFamily,
    pub message: DiagnosticMessage,
    pub source_context: Option<SourceContext>,
    pub related_locations: Vec<RelatedLocation>,
    pub notes: Vec<DiagnosticNote>,
    pub fix_proposal: Option<FixProposal>,
    pub cause: Option<Box<DiagnosticCause>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::string::ToString;
    use alloc::vec;

    fn test_source_id(id: u64) -> SourceId {
        SourceId(id)
    }

    #[test]
    fn diagnostic_code_rejects_empty_and_whitespace() {
        assert_eq!(DiagnosticCode::try_from_static(""), Err(EmptyCodeError));
        assert_eq!(DiagnosticCode::try_from_static("   "), Err(EmptyCodeError));
        assert_eq!(
            DiagnosticCode::try_from_static("\t\n "),
            Err(EmptyCodeError)
        );
        assert_eq!(
            DiagnosticCode::try_from_string("".to_string()),
            Err(EmptyCodeError)
        );
        assert_eq!(
            DiagnosticCode::try_from_string(" \t ".to_string()),
            Err(EmptyCodeError)
        );
    }

    #[test]
    fn diagnostic_code_accepts_non_empty_and_preserves_exact_bytes_and_case() {
        let code1 = DiagnosticCode::try_from_static("E0101").unwrap();
        assert_eq!(code1.as_str(), "E0101");

        let code2 = DiagnosticCode::try_from_static("e0101").unwrap();
        assert_eq!(code2.as_str(), "e0101");
        assert_ne!(code1, code2);

        // Does not trim valid non-whitespace tokens
        let untrimmed = DiagnosticCode::try_from_static(" E0101 ").unwrap();
        assert_eq!(untrimmed.as_str(), " E0101 ");

        let owned = DiagnosticCode::try_from_string("VERIFY_042".to_string()).unwrap();
        assert_eq!(owned.as_str(), "VERIFY_042");
    }

    #[test]
    fn diagnostic_message_rejects_empty_and_preserves_bytes() {
        assert_eq!(DiagnosticMessage::new(""), Err(EmptyMessageError));
        assert_eq!(DiagnosticMessage::new("   \n"), Err(EmptyMessageError));

        let msg = DiagnosticMessage::new(" type mismatch: expected i32 ").unwrap();
        assert_eq!(msg.as_str(), " type mismatch: expected i32 ");
    }

    #[test]
    fn byte_offset_conversion_and_access() {
        let offset = ByteOffset::try_from_usize(1234).unwrap();
        assert_eq!(offset.as_u64(), 1234);
    }

    #[test]
    fn source_range_invariants() {
        let o10 = ByteOffset::try_from_usize(10).unwrap();
        let o20 = ByteOffset::try_from_usize(20).unwrap();

        // start < end
        let range = SourceRange::new(o10, o20).unwrap();
        assert_eq!(range.start(), o10);
        assert_eq!(range.end(), o20);
        assert!(!range.is_empty());

        // start == end (zero-width)
        let zero_width = SourceRange::new(o10, o10).unwrap();
        assert_eq!(zero_width.start(), o10);
        assert_eq!(zero_width.end(), o10);
        assert!(zero_width.is_empty());

        // start > end (inversion rejected)
        assert!(SourceRange::new(o20, o10).is_none());
    }

    #[test]
    fn source_context_variants() {
        let src = test_source_id(42);
        let range = SourceRange::new(
            ByteOffset::try_from_usize(0).unwrap(),
            ByteOffset::try_from_usize(10).unwrap(),
        );

        let ctx_with_range = SourceContext { source: src, range };
        assert_eq!(ctx_with_range.source, src);
        assert!(ctx_with_range.range.is_some());

        let ctx_no_range = SourceContext {
            source: src,
            range: None,
        };
        assert_eq!(ctx_no_range.source, src);
        assert!(ctx_no_range.range.is_none());
    }

    #[test]
    fn structured_cause_preserves_nesting_and_order() {
        let child1 = Diagnostic {
            code: DiagnosticCode::try_from_static("V0001").unwrap(),
            severity: DiagnosticSeverity::Error,
            family: DiagnosticFamily::Verification,
            message: DiagnosticMessage::new("opcode invalid").unwrap(),
            source_context: None,
            related_locations: Vec::new(),
            notes: Vec::new(),
            fix_proposal: None,
            cause: None,
        };

        let child2 = Diagnostic {
            code: DiagnosticCode::try_from_static("V0002").unwrap(),
            severity: DiagnosticSeverity::Warning,
            family: DiagnosticFamily::Verification,
            message: DiagnosticMessage::new("stack depth warning").unwrap(),
            source_context: None,
            related_locations: Vec::new(),
            notes: Vec::new(),
            fix_proposal: None,
            cause: None,
        };

        // Single nested diagnostic
        let single_cause = DiagnosticCause::Diagnostic(Box::new(child1.clone()));
        match &single_cause {
            DiagnosticCause::Diagnostic(inner) => {
                assert_eq!(inner.as_ref(), &child1);
            }
            _ => panic!("expected single diagnostic"),
        }

        // Report of nested diagnostics
        let report_cause = DiagnosticCause::Report(vec![child1.clone(), child2.clone()]);
        match &report_cause {
            DiagnosticCause::Report(items) => {
                assert_eq!(items.len(), 2);
                assert_eq!(&items[0], &child1);
                assert_eq!(&items[1], &child2);
            }
            _ => panic!("expected report diagnostic"),
        }
    }

    #[test]
    fn multiplicity_retains_duplicates() {
        let diag = Diagnostic {
            code: DiagnosticCode::try_from_static("E0101").unwrap(),
            severity: DiagnosticSeverity::Error,
            family: DiagnosticFamily::Frontend,
            message: DiagnosticMessage::new("syntax error").unwrap(),
            source_context: None,
            related_locations: Vec::new(),
            notes: Vec::new(),
            fix_proposal: None,
            cause: None,
        };

        let mut collection = Vec::new();
        for _ in 0..2 {
            collection.push(diag.clone());
        }
        assert_eq!(collection.len(), 2);
        assert_eq!(collection[0], collection[1]);
    }
}
