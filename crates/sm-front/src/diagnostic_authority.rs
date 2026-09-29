//! SSF-09 C1A/C1B: `sm-front`'s native diagnostic authority and its
//! producer-owned projection into the canonical `sm-diagnostic` carrier.
//!
//! `sm-front` owns four frontend failure stages - lexing, grammar parsing,
//! cross-grammar surface resolution, and the RustLike type checker - and this
//! module is the single place that states, per stage, the frontend's own:
//!
//! - diagnostic code (never inferred from message text downstream),
//! - severity (every frontend failure is an `Error`; the frontend emits no
//!   warnings),
//! - genuine UTF-8 byte range, or its explicit absence.
//!
//! Range authority is deliberately narrow (SSF-09 Decision D, carrier
//! contract section 8): a legacy point `FrontendError.pos` is never widened
//! into a zero-width span. A parse error only receives a range when the
//! reported byte offset is the start of a lexical token of the *same* token
//! stream the parser consumed, and that token's text is byte-identical to the
//! source at that offset; the range is then exactly that token's extent.
//! Type-checker errors carry no positional authority at all (the checker
//! reports offset `0` for every error) and therefore never carry a range.

use crate::lexer::LexFailure;
use crate::types::{FrontendError, FrontendErrorKind, Token, TokenKind};
use alloc::format;
use alloc::string::String;
use alloc::vec::Vec;
use core::ops::Range;
use sm_diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticFamily, DiagnosticMessage, DiagnosticNote,
    DiagnosticSeverity, RelatedLocation, SourceContext, SourceId, SourceRange,
};

/// `E0005`: the frontend grammar parser rejected the input.
pub const FRONTEND_SYNTAX_CODE: &str = "E0005";
/// `E0006`: the frontend grammar parser rejected a construct the active
/// parser profile does not admit.
pub const FRONTEND_POLICY_VIOLATION_CODE: &str = "E0006";
/// `E0007`: cross-grammar surface resolution found conflicting claims.
pub const FRONTEND_AMBIGUOUS_SURFACE_CODE: &str = "E0007";
/// `E0008`: no grammar established a surface claim for the input.
pub const FRONTEND_NO_SURFACE_CLAIM_CODE: &str = "E0008";
/// `E0201`: the RustLike type checker rejected an admitted program.
pub const FRONTEND_TYPE_CHECK_CODE: &str = "E0201";

/// Every diagnostic code this module itself assigns, in ascending order.
/// Logos grammar codes (`E02xx`) are assigned by the Logos parser at the
/// error site and relayed unchanged through [`FrontendErrorDetail`]
/// (crate::types::FrontendErrorDetail); registry-completeness guards cover
/// both sets.
pub const FRONTEND_DIAGNOSTIC_CODES: [&str; 9] = [
    "E0001",
    "E0002",
    "E0004",
    FRONTEND_SYNTAX_CODE,
    FRONTEND_POLICY_VIOLATION_CODE,
    FRONTEND_AMBIGUOUS_SURFACE_CODE,
    FRONTEND_NO_SURFACE_CLAIM_CODE,
    "E0101",
    FRONTEND_TYPE_CHECK_CODE,
];

/// The frontend stage that produced a failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FrontendStage {
    Lex,
    Parse,
    SurfaceResolution,
    TypeCheck,
}

/// A frontend failure carrying the frontend's own code, bare message, and
/// genuine optional byte range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendDiagnostic {
    pub stage: FrontendStage,
    pub code: &'static str,
    pub message: String,
    pub range: Option<Range<usize>>,
    /// Further errors the same parse recovered past, in producer order,
    /// each with its own code, bare message and optional range.
    pub related: Vec<FrontendRelated>,
}

/// One additional recovered error of a [`FrontendDiagnostic`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendRelated {
    pub code: &'static str,
    pub message: String,
    pub range: Option<Range<usize>>,
}

impl FrontendDiagnostic {
    /// Lexer failure: code, detail, and range are the lexer's own.
    pub fn from_lex_failure(failure: &LexFailure) -> Self {
        Self {
            stage: FrontendStage::Lex,
            code: failure.code,
            message: failure.detail.clone(),
            range: failure.range.clone(),
            related: Vec::new(),
        }
    }

    /// Grammar parse failure over `tokens`, which must be the exact token
    /// stream the failing parser consumed for `source`.
    ///
    /// When the parser attached structured detail (the Logos grammar), the
    /// first underlying error supplies the code, bare message and range and
    /// every further recovered error becomes a [`FrontendRelated`] entry;
    /// otherwise the error kind selects `E0005`/`E0006` and `message` is
    /// already bare.
    pub fn from_parse_error(source: &str, tokens: &[Token], error: &FrontendError) -> Self {
        if let Some(detail) = &error.detail {
            if let Some((first, rest)) = detail.items.split_first() {
                return Self {
                    stage: FrontendStage::Parse,
                    code: first.code,
                    message: first.message.clone(),
                    range: token_anchor_range(source, tokens, first.pos),
                    related: rest
                        .iter()
                        .map(|item| FrontendRelated {
                            code: item.code,
                            message: item.message.clone(),
                            range: token_anchor_range(source, tokens, item.pos),
                        })
                        .collect(),
                };
            }
        }
        let code = match error.kind() {
            FrontendErrorKind::Syntax => FRONTEND_SYNTAX_CODE,
            FrontendErrorKind::PolicyViolation => FRONTEND_POLICY_VIOLATION_CODE,
        };
        Self {
            stage: FrontendStage::Parse,
            code,
            message: error.message.clone(),
            range: token_anchor_range(source, tokens, error.pos),
            related: Vec::new(),
        }
    }

    /// RustLike type-checker failure. The type checker has no positional
    /// authority, so the range is always absent.
    pub fn from_type_check_error(error: &FrontendError) -> Self {
        Self {
            stage: FrontendStage::TypeCheck,
            code: FRONTEND_TYPE_CHECK_CODE,
            message: error.message.clone(),
            range: None,
            related: Vec::new(),
        }
    }

    /// Cross-grammar surface-resolution failure (`E0007`/`E0008`). The
    /// classification concerns the whole input, not a sub-range of it.
    pub fn surface_resolution(code: &'static str, message: String) -> Self {
        Self {
            stage: FrontendStage::SurfaceResolution,
            code,
            message,
            range: None,
            related: Vec::new(),
        }
    }

    /// Every frontend failure is an error.
    pub const fn severity(&self) -> DiagnosticSeverity {
        DiagnosticSeverity::Error
    }

    /// Producer-owned lossless projection into the canonical carrier.
    ///
    /// `source` is the session token of the text this diagnostic was
    /// produced from; `None` leaves the whole source context absent.
    /// Returns `None` only if the code or message violates the carrier's
    /// non-empty invariants - never a substituted placeholder.
    pub fn to_canonical(&self, source: Option<SourceId>) -> Option<Diagnostic> {
        let code = DiagnosticCode::try_from_static(self.code).ok()?;
        let message = DiagnosticMessage::new(self.message.clone()).ok()?;
        let mut diagnostic =
            Diagnostic::new(code, self.severity(), DiagnosticFamily::Frontend, message);
        diagnostic.source_context = source.map(|source| SourceContext {
            source,
            range: canonical_range(&self.range),
        });
        for related in &self.related {
            let message = format!("{}: {}", related.code, related.message);
            match source {
                Some(source) => diagnostic.related_locations.push(RelatedLocation {
                    context: SourceContext {
                        source,
                        range: canonical_range(&related.range),
                    },
                    message: Some(message),
                }),
                // Without a source token a related location cannot exist;
                // the recovered error is kept as a note instead of dropped.
                None => diagnostic.notes.push(DiagnosticNote { message }),
            }
        }
        Some(diagnostic)
    }
}

fn canonical_range(range: &Option<Range<usize>>) -> Option<SourceRange> {
    range
        .clone()
        .and_then(|r| SourceRange::try_from_bounds(r.start, r.end))
}

/// The extent of the lexical token that starts exactly at byte `offset`,
/// provided the token's text is byte-identical to `source` there. Synthetic
/// layout tokens (`<INDENT>`, `<DEDENT>`, newline markers) never match the
/// source bytes and therefore never produce a range; a newline token anchors
/// exactly its one `\n` byte.
pub fn token_anchor_range(source: &str, tokens: &[Token], offset: usize) -> Option<Range<usize>> {
    let rest = source.get(offset..)?;
    tokens
        .iter()
        .filter(|token| token.pos == offset)
        .find_map(|token| token_source_extent(token, rest))
        .map(|len| offset..offset + len)
}

/// The byte length of `token` at the start of `rest`, when its source bytes
/// are proven there. A newline token's display text is the escaped marker
/// `\n`, so its genuine source extent is the single `\n` byte itself.
fn token_source_extent(token: &Token, rest: &str) -> Option<usize> {
    if token.kind == TokenKind::Newline {
        return rest.starts_with('\n').then_some(1);
    }
    (!token.text.is_empty() && rest.starts_with(token.text.as_str())).then_some(token.text.len())
}

/// The extent of the lexical token whose legacy `(line, col)` mark equals
/// `mark`, under the same byte-identity rule as [`token_anchor_range`].
pub fn token_anchor_range_at_mark(
    source: &str,
    tokens: &[Token],
    line: u32,
    col: u32,
) -> Option<Range<usize>> {
    tokens
        .iter()
        .filter(|token| token.mark.line == line && token.mark.col == col)
        .find_map(|token| token_anchor_range(source, core::slice::from_ref(token), token.pos))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::{lex_tokens, lex_tokens_with_authority};
    use crate::parse_program_with_profile;
    use sm_diagnostic::SourceRegistry;
    use sm_profile::ParserProfile;

    #[test]
    fn lex_failure_keeps_lexer_code_detail_and_char_range() {
        let src = "fn main() {\n    let x: i32 = 1 @ 2;\n}\n";
        let failure = lex_tokens_with_authority(src).unwrap_err();
        let diag = FrontendDiagnostic::from_lex_failure(&failure);
        assert_eq!(diag.code, "E0001");
        assert_eq!(diag.message, "unexpected character '@'");
        let range = diag.range.clone().unwrap();
        assert_eq!(&src[range], "@");
    }

    #[test]
    fn lex_failure_unterminated_string_spans_to_line_end() {
        let src = "fn main() {\n    let s: text = \"abc\n}\n";
        let failure = lex_tokens_with_authority(src).unwrap_err();
        assert_eq!(failure.code, "E0004");
        assert_eq!(&src[failure.range.clone().unwrap()], "\"abc");
    }

    #[test]
    fn parse_error_range_is_offending_token_extent() {
        let src = "fn main() {\n    let = 1;\n}\n";
        let tokens = lex_tokens(src).unwrap();
        let err =
            parse_program_with_profile(src, &ParserProfile::foundation_default()).unwrap_err();
        let diag = FrontendDiagnostic::from_parse_error(src, &tokens, &err);
        assert_eq!(diag.code, FRONTEND_SYNTAX_CODE);
        let range = diag.range.clone().expect("anchored at a real token");
        let anchored = &src[range.clone()];
        assert!(!anchored.is_empty());
        assert!(tokens
            .iter()
            .any(|t| t.pos == range.start && t.text == anchored));
        assert_eq!(range.start, err.pos);
    }

    #[test]
    fn point_without_source_token_never_becomes_a_range() {
        let src = "fn main() { return; }\n";
        let tokens = lex_tokens(src).unwrap();
        assert_eq!(token_anchor_range(src, &tokens, 2), None);
        assert_eq!(token_anchor_range(src, &tokens, src.len()), None);
        assert_eq!(token_anchor_range(src, &tokens, 10_000), None);
    }

    #[test]
    fn newline_token_anchors_exactly_its_newline_byte() {
        let src = "fn main() {\n    let value: i32 =\n}\n";
        let tokens = lex_tokens(src).unwrap();
        let nl = src.find("=\n").unwrap() + 1;
        assert_eq!(token_anchor_range(src, &tokens, nl), Some(nl..nl + 1));
    }

    #[test]
    fn logos_parse_error_uses_structured_code_message_and_range() {
        let src = "Entity A:\n    state x: quad\nLaw \"L\" [priority x]:\n    When N ->\n        Pulse.emit(\"x\")\n";
        let tokens = lex_tokens(src).unwrap();
        let err = crate::parse_logos_program(src).unwrap_err();
        assert!(err.message.contains("-->"), "legacy message stays rendered");
        let diag = FrontendDiagnostic::from_parse_error(src, &tokens, &err);
        assert_eq!(diag.code, "E0224");
        assert_eq!(diag.message, "expected priority number");
        assert_eq!(&src[diag.range.clone().unwrap()], "x");
        assert!(diag.related.is_empty());
    }

    #[test]
    fn logos_recovered_errors_become_related_locations_in_order() {
        let src = "Entity Broken\nEntity AlsoBroken\nEntity Good:\n    state hp: quad\n";
        let tokens = lex_tokens(src).unwrap();
        let err = crate::parse_logos_program(src).unwrap_err();
        let diag = FrontendDiagnostic::from_parse_error(src, &tokens, &err);
        assert_eq!(diag.related.len(), 1);
        let mut registry = SourceRegistry::new();
        let id = registry.mint();
        let canonical = diag.to_canonical(Some(id)).unwrap();
        assert_eq!(canonical.related_locations.len(), 1);
        assert_eq!(canonical.related_locations[0].context.source, id);
        let unbound = diag.to_canonical(None).unwrap();
        assert!(unbound.related_locations.is_empty());
        assert_eq!(unbound.notes.len(), 1);
    }

    #[test]
    fn type_check_error_never_carries_range() {
        let err = FrontendError {
            detail: None,
            pos: 0,
            message: "program must define fn main()".into(),
        };
        let diag = FrontendDiagnostic::from_type_check_error(&err);
        assert_eq!(diag.code, FRONTEND_TYPE_CHECK_CODE);
        assert_eq!(diag.range, None);
    }

    #[test]
    fn canonical_projection_preserves_identity_and_range() {
        let src = "fn main() {\n    let x: i32 = 1 @ 2;\n}\n";
        let failure = lex_tokens_with_authority(src).unwrap_err();
        let mut registry = SourceRegistry::new();
        let id = registry.mint();
        let canonical = FrontendDiagnostic::from_lex_failure(&failure)
            .to_canonical(Some(id))
            .unwrap();
        assert_eq!(canonical.code.as_str(), "E0001");
        assert_eq!(canonical.severity, DiagnosticSeverity::Error);
        assert_eq!(canonical.family, DiagnosticFamily::Frontend);
        let ctx = canonical.source_context.unwrap();
        assert_eq!(ctx.source, id);
        let range = ctx.range.unwrap();
        assert_eq!(range.end().as_u64() - range.start().as_u64(), 1);
    }

    #[test]
    fn canonical_projection_without_source_has_no_context() {
        let diag = FrontendDiagnostic::surface_resolution(
            FRONTEND_NO_SURFACE_CLAIM_CODE,
            "no surface claim".into(),
        );
        let canonical = diag.to_canonical(None).unwrap();
        assert!(canonical.source_context.is_none());
    }

    #[test]
    fn frontend_code_registry_is_sorted_and_unique() {
        for pair in FRONTEND_DIAGNOSTIC_CODES.windows(2) {
            assert!(pair[0] < pair[1], "{} then {}", pair[0], pair[1]);
        }
    }
}
