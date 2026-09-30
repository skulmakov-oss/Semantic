//! SSF-09 C4: explicit runtime diagnostic admission.
//!
//! Not every [`RuntimeError`] is a canonical diagnostic (SSF-08 runtime
//! failure ownership stays authoritative). This module is the single,
//! exhaustive admission table: a failure the table does not admit projects
//! to `None`, never to a placeholder diagnostic.
//!
//! Admitted (producer-owned code, severity `Error`, family `Runtime`):
//!
//! | failure | code |
//! |---|---|
//! | `Trap(AssertionFailed)` | `R0001` |
//! | `Trap(BorrowWriteConflict)` | `R0002` |
//! | `Trap(DivisionByZero)` | `R0003` |
//! | `Trap(ArithmeticOverflow)` | `R0004` |
//! | `QuotaExceeded(_)` | `R0010` |
//! | `CapabilityDenied(_)` | `R0020` |
//! | `VerifierRejected(report)` | `R0030`, with the verifier findings preserved as a structured `DiagnosticCause::Report` |
//!
//! Not admitted: header/format/version failures of raw artifacts and every
//! VM internal-integrity fault on already-verified code (stack under/overflow,
//! unknown function/variable/string id, invalid jump, runtime type mismatch,
//! host ABI failure). Runtime diagnostics never carry a source range: a VM
//! program counter is not a source position.

use crate::RuntimeError;
use sm_diagnostic::{
    Diagnostic, DiagnosticCause, DiagnosticCode, DiagnosticFamily, DiagnosticMessage,
    DiagnosticSeverity,
};
use sm_runtime_core::RuntimeTrap;
use std::boxed::Box;
use std::string::ToString;

/// Every runtime code the admission table can emit, in ascending order.
pub const RUNTIME_DIAGNOSTIC_CODES: [&str; 7] = [
    "R0001", "R0002", "R0003", "R0004", "R0010", "R0020", "R0030",
];

impl RuntimeError {
    /// The producer-owned code of an admitted runtime failure, or `None`.
    pub const fn admitted_code(&self) -> Option<&'static str> {
        match self {
            RuntimeError::Trap(RuntimeTrap::AssertionFailed) => Some("R0001"),
            RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict) => Some("R0002"),
            RuntimeError::Trap(RuntimeTrap::DivisionByZero) => Some("R0003"),
            RuntimeError::Trap(RuntimeTrap::ArithmeticOverflow) => Some("R0004"),
            RuntimeError::QuotaExceeded(_) => Some("R0010"),
            RuntimeError::CapabilityDenied(_) => Some("R0020"),
            RuntimeError::VerifierRejected(_) => Some("R0030"),
            RuntimeError::BadHeader
            | RuntimeError::UnsupportedBytecodeVersion { .. }
            | RuntimeError::BadFormat(_)
            | RuntimeError::UnknownFunction(_)
            | RuntimeError::InvalidJumpAddress { .. }
            | RuntimeError::TypeMismatchRuntime(_)
            | RuntimeError::StackUnderflow
            | RuntimeError::StackOverflow
            | RuntimeError::UnknownVariable(_)
            | RuntimeError::InvalidStringId(_)
            | RuntimeError::HostAbi(_) => None,
        }
    }

    /// Projects an admitted runtime failure into the canonical carrier.
    pub fn to_canonical(&self) -> Option<Diagnostic> {
        let code = DiagnosticCode::try_from_static(self.admitted_code()?).ok()?;
        let message = DiagnosticMessage::new(self.to_string()).ok()?;
        let mut diagnostic = Diagnostic::new(
            code,
            DiagnosticSeverity::Error,
            DiagnosticFamily::Runtime,
            message,
        );
        if let RuntimeError::VerifierRejected(report) = self {
            diagnostic.cause = Some(Box::new(DiagnosticCause::Report(report.to_canonical()?)));
        }
        Some(diagnostic)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use sm_runtime_core::{QuotaExceeded, QuotaKind};
    use sm_verify::{RejectReport, VerificationCode, VerificationDiagnostic};
    use std::vec;

    #[test]
    fn traps_are_admitted_with_producer_codes() {
        let cases = [
            (RuntimeTrap::AssertionFailed, "R0001"),
            (RuntimeTrap::BorrowWriteConflict, "R0002"),
            (RuntimeTrap::DivisionByZero, "R0003"),
            (RuntimeTrap::ArithmeticOverflow, "R0004"),
        ];
        for (trap, code) in cases {
            let d = RuntimeError::Trap(trap).to_canonical().unwrap();
            assert_eq!(d.code.as_str(), code);
            assert_eq!(d.family, DiagnosticFamily::Runtime);
            assert!(d.source_context.is_none());
        }
    }

    #[test]
    fn internal_vm_faults_are_not_admitted() {
        for err in [
            RuntimeError::StackUnderflow,
            RuntimeError::StackOverflow,
            RuntimeError::BadHeader,
            RuntimeError::UnknownVariable("x".into()),
        ] {
            assert_eq!(err.admitted_code(), None);
            assert!(err.to_canonical().is_none());
        }
    }

    #[test]
    fn quota_is_admitted() {
        let err = RuntimeError::QuotaExceeded(QuotaExceeded {
            kind: QuotaKind::Steps,
            limit: 1,
            used: 2,
        });
        assert_eq!(err.to_canonical().unwrap().code.as_str(), "R0010");
    }

    #[test]
    fn verifier_rejection_keeps_structured_report_cause() {
        let report = RejectReport {
            diagnostics: vec![
                VerificationDiagnostic {
                    code: VerificationCode::BadHeader,
                    function: None,
                    offset: None,
                    message: "bad header".into(),
                },
                VerificationDiagnostic {
                    code: VerificationCode::UnknownOpcode,
                    function: None,
                    offset: Some(4),
                    message: "unknown opcode".into(),
                },
            ],
        };
        let d = RuntimeError::VerifierRejected(report)
            .to_canonical()
            .unwrap();
        assert_eq!(d.code.as_str(), "R0030");
        match d.cause.as_deref() {
            Some(DiagnosticCause::Report(items)) => {
                assert_eq!(items.len(), 2);
                assert_eq!(items[0].code.as_str(), "V0001");
                assert_eq!(items[1].code.as_str(), "V0009");
            }
            other => panic!("expected structured report cause, got {other:?}"),
        }
    }
}
