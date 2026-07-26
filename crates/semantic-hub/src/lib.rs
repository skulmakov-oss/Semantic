//! Semantic Hub v0: the governed execution boundary between Semantic and
//! external computational tools.
//!
//! Semantic Hub governs tool identity, request admission, dispatch,
//! capabilities, privacy classification, resource budgets, worker
//! lifecycle, failure containment, supervision, output validation, audit,
//! and provenance. It does not own Semantic language meaning, verifier
//! admission, VM execution, capability truth, canonical audit truth,
//! project state, or release truth -- those remain owned by Semantic Core,
//! the verifier, the VM, and the existing PROMETHEUS boundary crates.
//!
//! A tool result returned through the Hub is untrusted computational
//! evidence, never Semantic truth by itself.

#![forbid(unsafe_code)]

pub mod admission;
pub mod audit;
pub mod capability;
pub mod descriptor;
pub mod envelope;
pub mod execution;
pub mod fault;
pub mod ids;
pub mod provenance;
pub mod registry;
pub mod resource;
pub mod runtime;
pub mod worker;

pub use admission::{admit, AdmissionAmbient, AdmittedInvocation};
pub use audit::{AuditParseError, HubAuditRecord, HubAuditTrail, HUB_AUDIT_FORMAT_VERSION};
pub use capability::{HubCapability, HubCapabilitySet, HubCapabilitySetVersion};
pub use descriptor::{DescriptorError, HubOperationDescriptor, HubToolDescriptor};
pub use envelope::{HubReply, HubReplyStatus, HubRequest};
pub use envelope::{HUB_ENVELOPE_SCHEMA_VERSION, MAX_PAYLOAD_BYTES};
pub use execution::{HubDeterminismClass, HubExecutionMode, HubPrivacyClass, HubTrustClass};
pub use fault::{HubFault, HubToolError};
pub use ids::{HubApiVersion, HubCallerIdentity, HubOperationId, HubRequestId};
pub use ids::{HubSessionId, HubToolId, HubToolVersion, IdSyntaxError};
pub use provenance::{content_digest, HubDigest, HubProvenance};
pub use registry::{RegistryError, ToolRegistry};
pub use resource::{check_budget, HubBudgetExceeded, HubResourceBudget};
pub use resource::{HubResourceKind, HubResourceUsage};
pub use runtime::{Hub, HubTool, RestrictedHubContext};
pub use worker::{HubRestartPolicy, HubSupervisionPolicy, HubWorkerHealth};
pub use worker::{HubWorkerState, IllegalWorkerTransition};
