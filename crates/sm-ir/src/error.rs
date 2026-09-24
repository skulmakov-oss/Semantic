#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::string::String;

#[cfg(feature = "std")]
use std::string::String;

use sm_front::FrontendError;

/// Internal non-diagnostic compiler transport.
///
/// Represents compiler-internal invariant failures, optimizer defects,
/// internal register/index limits, and internal emission/format invariants.
/// Has NO diagnostic code, NO severity, NO FrontendErrorKind, NO SourceRange,
/// and NO DiagnosticFamily.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IrError {
    pub message: String,
}

impl core::fmt::Display for IrError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "internal compiler error: {}", self.message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for IrError {}

/// Non-canonical configuration and host transport.
///
/// Represents cargo feature gates, disabled compile profiles, and host/build
/// configuration rejections. Has NO diagnostic code, NO severity, NO FrontendErrorKind,
/// NO SourceRange, and NO DiagnosticFamily.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConfigurationError {
    pub message: String,
}

impl core::fmt::Display for ConfigurationError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "configuration error: {}", self.message)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for ConfigurationError {}

/// Typed pipeline transport envelope.
///
/// Preserves the originating semantic class:
/// - `Frontend`: canonical Frontend-stage diagnostics (lexing, parsing, typecheck, surface admission, lowering)
/// - `InternalIr`: internal compiler invariant defects and internal emission invariants
/// - `Configuration`: host/cargo feature and profile configuration rejections
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CompilePipelineError {
    Frontend(FrontendError),
    InternalIr(IrError),
    Configuration(ConfigurationError),
}

impl core::fmt::Display for CompilePipelineError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::Frontend(e) => core::fmt::Display::fmt(e, f),
            Self::InternalIr(e) => core::fmt::Display::fmt(e, f),
            Self::Configuration(e) => core::fmt::Display::fmt(e, f),
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for CompilePipelineError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Frontend(e) => Some(e),
            Self::InternalIr(e) => Some(e),
            Self::Configuration(e) => Some(e),
        }
    }
}

impl From<FrontendError> for CompilePipelineError {
    fn from(e: FrontendError) -> Self {
        Self::Frontend(e)
    }
}

impl From<IrError> for CompilePipelineError {
    fn from(e: IrError) -> Self {
        Self::InternalIr(e)
    }
}

impl From<ConfigurationError> for CompilePipelineError {
    fn from(e: ConfigurationError) -> Self {
        Self::Configuration(e)
    }
}

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;

    #[test]
    fn test_a_frontend_structural_preservation() {
        let frontend_error = FrontendError {
            pos: 42,
            message: "syntax error test".to_string(),
        };
        let envelope = CompilePipelineError::from(frontend_error.clone());
        match envelope {
            CompilePipelineError::Frontend(preserved) => {
                assert_eq!(preserved, frontend_error);
                assert_eq!(preserved.pos, 42);
                assert_eq!(preserved.message, "syntax error test");
            }
            _ => panic!("expected CompilePipelineError::Frontend variant"),
        }
    }

    #[test]
    fn test_b_internal_ir_preservation() {
        let ir_error = IrError {
            message: "allocator register overflow".to_string(),
        };
        let envelope = CompilePipelineError::from(ir_error.clone());
        match envelope {
            CompilePipelineError::InternalIr(preserved) => {
                assert_eq!(preserved, ir_error);
                assert_eq!(preserved.message, "allocator register overflow");
            }
            _ => panic!("expected CompilePipelineError::InternalIr variant"),
        }
    }

    #[test]
    fn test_c_configuration_preservation() {
        let config_error = ConfigurationError {
            message: "profile-logos feature disabled".to_string(),
        };
        let envelope = CompilePipelineError::from(config_error.clone());
        match envelope {
            CompilePipelineError::Configuration(preserved) => {
                assert_eq!(preserved, config_error);
                assert_eq!(preserved.message, "profile-logos feature disabled");
            }
            _ => panic!("expected CompilePipelineError::Configuration variant"),
        }
    }

    #[test]
    fn test_d_semantic_display_distinction() {
        let ir_err = IrError {
            message: "resource limit reached".to_string(),
        };
        let config_err = ConfigurationError {
            message: "resource limit reached".to_string(),
        };
        let frontend_err = FrontendError {
            pos: 10,
            message: "resource limit reached".to_string(),
        };

        let ir_display = format!("{}", CompilePipelineError::from(ir_err));
        let config_display = format!("{}", CompilePipelineError::from(config_err));
        let frontend_display = format!("{}", CompilePipelineError::from(frontend_err));

        assert!(
            ir_display.starts_with("internal compiler error:"),
            "IrError Display must identify as internal compiler error, got: {}",
            ir_display
        );
        assert!(
            config_display.starts_with("configuration error:"),
            "ConfigurationError Display must identify as configuration error, got: {}",
            config_display
        );
        assert!(
            frontend_display.starts_with("at 10:"),
            "FrontendError Display must delegate to FrontendError, got: {}",
            frontend_display
        );

        assert_ne!(ir_display, config_display);
        assert_ne!(ir_display, frontend_display);
        assert_ne!(config_display, frontend_display);
    }

    #[test]
    fn test_error_source_chain() {
        use std::error::Error;

        let fe = FrontendError {
            pos: 1,
            message: "err".to_string(),
        };
        let envelope_fe = CompilePipelineError::from(fe);
        assert!(envelope_fe.source().is_some());

        let ie = IrError {
            message: "err".to_string(),
        };
        let envelope_ie = CompilePipelineError::from(ie);
        assert!(envelope_ie.source().is_some());

        let ce = ConfigurationError {
            message: "err".to_string(),
        };
        let envelope_ce = CompilePipelineError::from(ce);
        assert!(envelope_ce.source().is_some());
    }
}
