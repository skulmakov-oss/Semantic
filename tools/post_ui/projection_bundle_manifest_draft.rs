// Fixture-facing ProjectionBundle manifest draft types.
// This file is compile-checked as test/fixture evidence only.
// It is not a parser.
// It is not a loader.
// It is not final serialization.
// It is not runtime activation code.
// It does not verify bundles.
// It does not authorize production UI wiring.
#![forbid(unsafe_code)]
#![allow(dead_code)]

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProjectionBundleManifestDraft<'a> {
    pub bundle_id: &'a str,
    pub bundle_version: &'a str,
    pub projection_id: &'a str,
    pub source_refs: &'a [&'a str],
    pub artifacts: ManifestArtifactsDraft<'a>,
    pub compatibility: ManifestCompatibilityDraft<'a>,
    pub safety: ManifestSafetyDraft<'a>,
    pub trust: ManifestTrustDraft<'a>,
    pub activation_policy: ManifestActivationPolicyDraft,
    pub update_policy: ManifestUpdatePolicyDraft,
    pub diagnostics: ManifestDiagnosticsDraft<'a>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ManifestArtifactsDraft<'a> {
    pub ui_ir_ref: &'a str,
    pub binding_graph_ref: &'a str,
    pub action_ir_ref: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ManifestCompatibilityDraft<'a> {
    pub role_dictionary_version: &'a str,
    pub renderer_profile: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ManifestSafetyDraft<'a> {
    pub safety_class: ManifestSafetyClassDraft,
    pub criticality: ManifestCriticalityDraft,
    pub required_capabilities: &'a [&'a str],
    pub freshness_policy: ManifestFreshnessPolicyDraft,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ManifestTrustDraft<'a> {
    pub hash: &'a str,
    pub signature: &'a str,
    pub created_by: &'a str,
    pub created_at: &'a str,
    pub compiler_identity: &'a str,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ManifestActivationPolicyDraft {
    pub require_verification: bool,
    pub allow_runtime_tree_streaming: bool,
    pub allow_production_activation: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ManifestUpdatePolicyDraft {
    pub require_safe_update_boundary: bool,
    pub allow_critical_update_during_pending_unknown: bool,
    pub allow_critical_update_during_quarantine: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ManifestDiagnosticsDraft<'a> {
    pub expected: &'a [&'a str],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManifestSafetyClassDraft {
    CriticalPinned,
    VerifiedDynamic,
    DiagnosticOnly,
    WorkbenchExperimental,
    ReadOnlyDashboard,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManifestCriticalityDraft {
    NonCritical,
    Guarded,
    Critical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ManifestFreshnessPolicyDraft {
    FreshForControl,
    ReadOnlyWhenStale,
    ObservationOnly,
}

pub const MINIMAL_SKETCH_MANIFEST_DRAFT: ProjectionBundleManifestDraft<'static> =
    ProjectionBundleManifestDraft {
        bundle_id: "bundle.example.minimal",
        bundle_version: "0-sketch",
        projection_id: "ExampleMinimalProjection",
        source_refs: &["semantic.source.example", "projection.source.example"],
        artifacts: ManifestArtifactsDraft {
            ui_ir_ref: "ui_ir.example.minimal",
            binding_graph_ref: "binding_graph.example.minimal",
            action_ir_ref: "action_ir.example.minimal",
        },
        compatibility: ManifestCompatibilityDraft {
            role_dictionary_version: "ui-roles.0-sketch",
            renderer_profile: "semantic-shell.reference-sketch",
        },
        safety: ManifestSafetyDraft {
            safety_class: ManifestSafetyClassDraft::VerifiedDynamic,
            criticality: ManifestCriticalityDraft::NonCritical,
            required_capabilities: &[],
            freshness_policy: ManifestFreshnessPolicyDraft::FreshForControl,
        },
        trust: ManifestTrustDraft {
            hash: "sha256:SKETCH-NOT-A-REAL-HASH",
            signature: "signature:SKETCH-NOT-A-REAL-SIGNATURE",
            created_by: "semantic-projection-compiler.SKETCH",
            created_at: "not-a-real-timestamp",
            compiler_identity: "semantic-projection-compiler.0-sketch",
        },
        activation_policy: ManifestActivationPolicyDraft {
            require_verification: true,
            allow_runtime_tree_streaming: false,
            allow_production_activation: false,
        },
        update_policy: ManifestUpdatePolicyDraft {
            require_safe_update_boundary: true,
            allow_critical_update_during_pending_unknown: false,
            allow_critical_update_during_quarantine: false,
        },
        diagnostics: ManifestDiagnosticsDraft { expected: &[] },
    };
