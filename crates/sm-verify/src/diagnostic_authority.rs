//! SSF-09 C3A/C3B: `sm-verify`'s native diagnostic authority and its
//! producer-owned projection into the canonical `sm-diagnostic` carrier.
//!
//! C3A: every [`VerificationCode`] owns an explicit, stable textual code token
//! (`V0001`..). Tokens are assigned by an exhaustive `match` - never by
//! `Debug` formatting - and are append-only: a new variant receives the next
//! unused token, and an existing token is never reassigned. Every verifier
//! finding is an admission rejection, so its producer-owned severity is
//! `Error`.
//!
//! C3B: a verification diagnostic never carries a canonical source range.
//! `VerificationDiagnostic.offset` is a SemCode artifact byte offset, not a
//! source position (carrier contract section 8, items 4-5); the function
//! name and artifact offset are preserved as structured notes instead.

use crate::{RejectReport, VerificationCode, VerificationDiagnostic};
use sm_diagnostic::{
    Diagnostic, DiagnosticCode, DiagnosticFamily, DiagnosticMessage, DiagnosticNote,
    DiagnosticSeverity,
};
use std::format;
use std::vec::Vec;

/// Every verification code, in token order (`V0001` first).
pub const ALL_VERIFICATION_CODES: [VerificationCode; 32] = [
    VerificationCode::BadHeader,
    VerificationCode::UnsupportedVersion,
    VerificationCode::TruncatedFunction,
    VerificationCode::InvalidFunctionName,
    VerificationCode::DuplicateFunction,
    VerificationCode::InvalidStringTable,
    VerificationCode::InvalidDebugSection,
    VerificationCode::InvalidOwnershipSection,
    VerificationCode::UnknownOpcode,
    VerificationCode::OperandOutOfBounds,
    VerificationCode::InvalidJumpTarget,
    VerificationCode::InvalidStringReference,
    VerificationCode::InvalidRegisterReference,
    VerificationCode::UnknownCallTarget,
    VerificationCode::ResourceLimitExceeded,
    VerificationCode::CapabilityViolation,
    VerificationCode::AmbiguousInstructionFraming,
    VerificationCode::OpcodeRequiresNewerHeader,
    VerificationCode::ReachableFunctionFallthrough,
    VerificationCode::InvalidSignatureSection,
    VerificationCode::CallArgumentCountMismatch,
    VerificationCode::UndefinedRegisterRead,
    VerificationCode::AnalysisStateLimitExceeded,
    VerificationCode::AnalysisWorkLimitExceeded,
    VerificationCode::InvalidOwnershipAnchor,
    VerificationCode::InvalidAdtDescriptorSection,
    VerificationCode::AdtRequiresDescriptorHeader,
    VerificationCode::UnknownAdtType,
    VerificationCode::InvalidAdtDiscriminant,
    VerificationCode::AdtVariantNameMismatch,
    VerificationCode::AdtPayloadArityMismatch,
    VerificationCode::AdtPayloadIndexOutOfRange,
];

impl VerificationCode {
    /// The producer-owned stable textual token of this code.
    pub const fn code_token(self) -> &'static str {
        match self {
            VerificationCode::BadHeader => "V0001",
            VerificationCode::UnsupportedVersion => "V0002",
            VerificationCode::TruncatedFunction => "V0003",
            VerificationCode::InvalidFunctionName => "V0004",
            VerificationCode::DuplicateFunction => "V0005",
            VerificationCode::InvalidStringTable => "V0006",
            VerificationCode::InvalidDebugSection => "V0007",
            VerificationCode::InvalidOwnershipSection => "V0008",
            VerificationCode::UnknownOpcode => "V0009",
            VerificationCode::OperandOutOfBounds => "V0010",
            VerificationCode::InvalidJumpTarget => "V0011",
            VerificationCode::InvalidStringReference => "V0012",
            VerificationCode::InvalidRegisterReference => "V0013",
            VerificationCode::UnknownCallTarget => "V0014",
            VerificationCode::ResourceLimitExceeded => "V0015",
            VerificationCode::CapabilityViolation => "V0016",
            VerificationCode::AmbiguousInstructionFraming => "V0017",
            VerificationCode::OpcodeRequiresNewerHeader => "V0018",
            VerificationCode::ReachableFunctionFallthrough => "V0019",
            VerificationCode::InvalidSignatureSection => "V0020",
            VerificationCode::CallArgumentCountMismatch => "V0021",
            VerificationCode::UndefinedRegisterRead => "V0022",
            VerificationCode::AnalysisStateLimitExceeded => "V0023",
            VerificationCode::AnalysisWorkLimitExceeded => "V0024",
            VerificationCode::InvalidOwnershipAnchor => "V0025",
            VerificationCode::InvalidAdtDescriptorSection => "V0026",
            VerificationCode::AdtRequiresDescriptorHeader => "V0027",
            VerificationCode::UnknownAdtType => "V0028",
            VerificationCode::InvalidAdtDiscriminant => "V0029",
            VerificationCode::AdtVariantNameMismatch => "V0030",
            VerificationCode::AdtPayloadArityMismatch => "V0031",
            VerificationCode::AdtPayloadIndexOutOfRange => "V0032",
        }
    }

    /// Producer-owned severity: every verifier finding rejects admission.
    pub const fn severity(self) -> DiagnosticSeverity {
        DiagnosticSeverity::Error
    }
}

impl VerificationDiagnostic {
    /// Producer-owned lossless projection into the canonical carrier. The
    /// source context is always absent: verified SemCode carries no proven
    /// source provenance.
    pub fn to_canonical(&self) -> Option<Diagnostic> {
        let code = DiagnosticCode::try_from_static(self.code.code_token()).ok()?;
        let message = DiagnosticMessage::new(self.message.clone()).ok()?;
        let mut diagnostic = Diagnostic::new(
            code,
            self.code.severity(),
            DiagnosticFamily::Verification,
            message,
        );
        if let Some(function) = &self.function {
            diagnostic.notes.push(DiagnosticNote {
                message: format!("function: {function}"),
            });
        }
        if let Some(offset) = self.offset {
            diagnostic.notes.push(DiagnosticNote {
                message: format!("artifact byte offset: {offset}"),
            });
        }
        Some(diagnostic)
    }
}

impl RejectReport {
    /// Projects every finding, preserving multiplicity and producer order.
    /// Returns `None` if any single finding cannot be projected, rather than
    /// silently dropping it.
    pub fn to_canonical(&self) -> Option<Vec<Diagnostic>> {
        self.diagnostics
            .iter()
            .map(VerificationDiagnostic::to_canonical)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use std::string::ToString;
    use std::vec;

    #[test]
    fn code_tokens_are_unique_and_never_debug_derived() {
        let mut seen = HashSet::new();
        for (index, code) in ALL_VERIFICATION_CODES.iter().enumerate() {
            let token = code.code_token();
            assert!(seen.insert(token), "duplicate token {token}");
            assert_eq!(token, format!("V{:04}", index + 1));
            assert_ne!(token, format!("{code:?}"));
        }
    }

    #[test]
    fn reject_report_projection_preserves_order_and_multiplicity() {
        let finding = VerificationDiagnostic {
            code: VerificationCode::UnknownOpcode,
            function: Some("main".to_string()),
            offset: Some(12),
            message: "unknown opcode 0xff".to_string(),
        };
        let other = VerificationDiagnostic {
            code: VerificationCode::InvalidJumpTarget,
            function: None,
            offset: None,
            message: "jump out of bounds".to_string(),
        };
        let report = RejectReport {
            diagnostics: vec![finding.clone(), other, finding],
        };
        let canonical = report.to_canonical().unwrap();
        let codes: Vec<&str> = canonical.iter().map(|d| d.code.as_str()).collect();
        assert_eq!(codes, vec!["V0009", "V0011", "V0009"]);
        assert!(canonical.iter().all(|d| d.source_context.is_none()));
        assert!(canonical
            .iter()
            .all(|d| d.family == DiagnosticFamily::Verification
                && d.severity == DiagnosticSeverity::Error));
        assert_eq!(canonical[0].notes.len(), 2);
        assert!(canonical[1].notes.is_empty());
    }
}
