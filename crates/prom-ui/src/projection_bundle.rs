//! ProjectionBundle v0 codec: canonical serialization, decoding, structural,
//! cross-artifact, compatibility, and self-consistency verification, and
//! pure in-memory inert loading.
//!
//! This module resolves the concrete representation decisions left open by
//! the UI-DNA2-8A documentation-only contract freeze
//! (`docs/spec/ui/projection_bundle_v0.md`) -- final serialization form,
//! canonical encoding, and per-artifact embedding policy -- exactly as prior
//! UI-DNA2 implementation PRs (#1507, #1508, #1511, #1541) have always
//! resolved analogous "Rust representation unresolved" spec language, under
//! the direct owner authorization recorded in Issue #1543.
//!
//! Stage separation is preserved, not weakened:
//!
//! ```text
//! parser != validator != verifier != loader != activation != production promotion
//! ```
//!
//! # Trust-stage caveat
//!
//! This module's "trust" stage is **deterministic self-consistency
//! verification** (canonicalize the decoded candidate, re-encode it, and
//! compare byte-for-byte against the original input), exactly mirroring the
//! disclaimer already established by the existing Static UI IR Artifact V1
//! precedent (`crates/prom-ui/src/static_ir_artifact.rs`): this proves
//! internal canonical consistency, not authenticity, integrity, or
//! admission proof. It is **not cryptographic digest or signature trust**.
//! No crypto algorithm has been selected by any accepted repository
//! contract (`docs/spec/ui/projection_bundle_v0.md` section 16, unresolved
//! items 4-5), and none is added by this module or its dependencies. Real
//! cryptographic trust remains a distinct, separately-ownable future
//! decision.
#![allow(dead_code)]

use alloc::string::String;
use alloc::vec::Vec;

use crate::action_ir::{
    ActionContextRequirements, ActionIrDocument, ActionRoute, ActionRouteId, ActionRouteOrder,
    ActionTarget, ActionTargetSlot,
};
use crate::binding_graph::{
    BindingDeclaration, BindingDependency, BindingGraphDocument, BindingId, BindingSlot,
    BindingTarget, BindingValueDomain,
};
use crate::collection_anchor_declarations::QualifiedCollectionAnchorDeclarations;
use crate::contract_primitives::{
    ContractVersion, Epoch, Revision, SchemaVersion, SourceId, SourceRef, SourceSpan,
    StaticDocumentId, StaticNodeId,
};
use crate::role_dictionary::{BuiltInRole, RoleDictionary};
use crate::semantic_refs::{CapabilityRef, EpochRef, RevisionRef, SemanticActionRef};
use crate::static_ir::StaticUiDocument;
use crate::static_ir_artifact::{
    verify_static_ui_ir_artifact_v1, StaticUiArtifactV1Error, StaticUiArtifactV1Limits,
    VerifiedStaticUiArtifactV1,
};
use prom_refs::ReferenceToken;

pub(crate) const PROJECTION_BUNDLE_V0_MAGIC: &[u8] = b"UI_DNA2_PROJECTION_BUNDLE_V0";

// ---- Section kinds --------------------------------------------------------

/// Fixed positional section kinds. Canonical bundle order is ascending by
/// this discriminant; every kind is required exactly once for v0. There is
/// no literal "CollectionAnchor declarations" artifact class in the frozen
/// `docs/spec/ui/projection_bundle_v0.md` section 6 list; this
/// implementation adds one as a required section since the reference
/// contour's Shell Player activation needs it, consistent with that
/// section's requirement that bundle composition supply every artifact
/// class the bounded contour actually consumes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[repr(u8)]
pub(crate) enum ProjectionBundleSectionKind {
    StaticUiIr = 0,
    BindingGraph = 1,
    ActionIr = 2,
    CollectionAnchors = 3,
    Accessibility = 4,
    DiagnosticProvenance = 5,
}

const REQUIRED_SECTION_COUNT: usize = 6;

impl ProjectionBundleSectionKind {
    const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::StaticUiIr),
            1 => Some(Self::BindingGraph),
            2 => Some(Self::ActionIr),
            3 => Some(Self::CollectionAnchors),
            4 => Some(Self::Accessibility),
            5 => Some(Self::DiagnosticProvenance),
            _ => None,
        }
    }

    const fn tag(self) -> u8 {
        self as u8
    }
}

// ---- Accessibility evidence (deterministically derived, not fabricated) --

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum AccessibilityRole {
    GenericContainer,
    Label,
    Button,
}

impl AccessibilityRole {
    const fn tag(self) -> u8 {
        match self {
            Self::GenericContainer => 0,
            Self::Label => 1,
            Self::Button => 2,
        }
    }

    const fn from_tag(tag: u8) -> Option<Self> {
        match tag {
            0 => Some(Self::GenericContainer),
            1 => Some(Self::Label),
            2 => Some(Self::Button),
            _ => None,
        }
    }

    /// Deterministic role mapping from the fixed built-in Role Dictionary.
    /// Every `BuiltInRole` maps to exactly one accessibility role; there is
    /// no fallback/default/"latest" resolution.
    const fn from_built_in(role: BuiltInRole) -> Self {
        match role {
            BuiltInRole::Root
            | BuiltInRole::Surface
            | BuiltInRole::EvidencePanel
            | BuiltInRole::Fragment => Self::GenericContainer,
            BuiltInRole::NumericReadout | BuiltInRole::Text => Self::Label,
            BuiltInRole::DangerAction | BuiltInRole::RecoveryOutlet => Self::Button,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct AccessibilityEntry {
    node: StaticNodeId,
    role: AccessibilityRole,
    label: String,
}

impl AccessibilityEntry {
    pub(crate) const fn node(&self) -> StaticNodeId {
        self.node
    }

    pub(crate) const fn role(&self) -> AccessibilityRole {
        self.role
    }

    pub(crate) fn label(&self) -> &str {
        &self.label
    }
}

/// Deterministically derives one accessibility entry per node from the
/// already-compiled Static UI IR and the Role Dictionary it was validated
/// against. This is real, structurally-derived evidence -- not a
/// placeholder -- ascending-ordered by `StaticNodeId`.
pub(crate) fn derive_accessibility_entries(
    document: &StaticUiDocument,
    dictionary: RoleDictionary,
) -> Vec<AccessibilityEntry> {
    let mut entries: Vec<AccessibilityEntry> = document
        .nodes()
        .iter()
        .filter_map(|node| {
            let built_in = dictionary.validate(node.role()).ok()?;
            Some(AccessibilityEntry {
                node: node.id(),
                role: AccessibilityRole::from_built_in(built_in),
                label: String::from(node.role().as_str()),
            })
        })
        .collect();
    entries.sort_by_key(|entry| entry.node);
    entries
}

// ---- Diagnostic / provenance metadata (required container) ---------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct BundleProvenance {
    compiler_identity: String,
    source_id: SourceId,
    source_revision: Revision,
    source_epoch: Epoch,
}

impl BundleProvenance {
    pub(crate) fn new(
        compiler_identity: String,
        source_id: SourceId,
        source_revision: Revision,
        source_epoch: Epoch,
    ) -> Self {
        Self {
            compiler_identity,
            source_id,
            source_revision,
            source_epoch,
        }
    }

    pub(crate) fn compiler_identity(&self) -> &str {
        &self.compiler_identity
    }

    pub(crate) const fn source_id(&self) -> SourceId {
        self.source_id
    }
}

// ---- Whole-bundle candidate ------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ProjectionBundleV0 {
    schema_version: SchemaVersion,
    contract_version: ContractVersion,
    bundle_id: StaticDocumentId,
    bundle_revision: Revision,
    bundle_epoch: Epoch,
    role_dictionary_schema_version: SchemaVersion,
    role_dictionary_contract_version: ContractVersion,
    static_document: StaticUiDocument,
    binding_graph: BindingGraphDocument,
    action_ir: ActionIrDocument,
    collection_anchors: QualifiedCollectionAnchorDeclarations,
    accessibility: Vec<AccessibilityEntry>,
    provenance: BundleProvenance,
}

impl ProjectionBundleV0 {
    /// Assembles one bundle candidate from already-independently-validated
    /// artifacts sharing one document identity. Composition alone does not
    /// verify cross-artifact agreement; call [`verify_projection_bundle_v0`]
    /// on the encoded bytes for the complete fail-closed pipeline.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn assemble(
        bundle_id: StaticDocumentId,
        bundle_revision: Revision,
        bundle_epoch: Epoch,
        dictionary: RoleDictionary,
        static_document: StaticUiDocument,
        binding_graph: BindingGraphDocument,
        action_ir: ActionIrDocument,
        collection_anchors: QualifiedCollectionAnchorDeclarations,
        provenance: BundleProvenance,
    ) -> Self {
        let accessibility = derive_accessibility_entries(&static_document, dictionary);
        Self {
            schema_version: SchemaVersion::CURRENT,
            contract_version: ContractVersion::CURRENT,
            bundle_id,
            bundle_revision,
            bundle_epoch,
            role_dictionary_schema_version: dictionary.schema_version(),
            role_dictionary_contract_version: dictionary.contract_version(),
            static_document,
            binding_graph,
            action_ir,
            collection_anchors,
            accessibility,
            provenance,
        }
    }

    pub(crate) fn static_document(&self) -> &StaticUiDocument {
        &self.static_document
    }

    pub(crate) fn binding_graph(&self) -> &BindingGraphDocument {
        &self.binding_graph
    }

    pub(crate) fn action_ir(&self) -> &ActionIrDocument {
        &self.action_ir
    }

    pub(crate) fn collection_anchors(&self) -> &QualifiedCollectionAnchorDeclarations {
        &self.collection_anchors
    }

    pub(crate) fn accessibility_entries(&self) -> &[AccessibilityEntry] {
        &self.accessibility
    }

    pub(crate) fn provenance(&self) -> &BundleProvenance {
        &self.provenance
    }

    /// Canonical byte encoding. Deterministic: identical logical content
    /// always produces identical bytes.
    pub(crate) fn canonical_bytes(
        &self,
        dictionary: RoleDictionary,
    ) -> Result<Vec<u8>, ProjectionBundleError> {
        let mut bytes = Vec::new();
        push_bytes(&mut bytes, PROJECTION_BUNDLE_V0_MAGIC)?;
        push_u32(&mut bytes, self.schema_version.raw());
        push_u32(&mut bytes, self.contract_version.raw());
        push_u64(&mut bytes, self.bundle_id.raw());
        push_u64(&mut bytes, self.bundle_revision.raw());
        push_u64(&mut bytes, self.bundle_epoch.raw());
        push_u32(&mut bytes, self.role_dictionary_schema_version.raw());
        push_u32(&mut bytes, self.role_dictionary_contract_version.raw());

        push_u32(&mut bytes, REQUIRED_SECTION_COUNT as u32);

        let static_ir_bytes = self
            .static_document
            .canonical_bytes(dictionary)
            .map_err(|_| ProjectionBundleError::new(ProjectionBundleErrorKind::Structural, None))?;
        push_section(
            &mut bytes,
            ProjectionBundleSectionKind::StaticUiIr,
            &static_ir_bytes,
        )?;

        let binding_graph_bytes = encode_binding_graph(&self.binding_graph)?;
        push_section(
            &mut bytes,
            ProjectionBundleSectionKind::BindingGraph,
            &binding_graph_bytes,
        )?;

        let action_ir_bytes = encode_action_ir(&self.action_ir)?;
        push_section(
            &mut bytes,
            ProjectionBundleSectionKind::ActionIr,
            &action_ir_bytes,
        )?;

        let collection_anchor_bytes = encode_collection_anchors(&self.collection_anchors)?;
        push_section(
            &mut bytes,
            ProjectionBundleSectionKind::CollectionAnchors,
            &collection_anchor_bytes,
        )?;

        let accessibility_bytes = encode_accessibility(&self.accessibility)?;
        push_section(
            &mut bytes,
            ProjectionBundleSectionKind::Accessibility,
            &accessibility_bytes,
        )?;

        let provenance_bytes = encode_provenance(&self.provenance)?;
        push_section(
            &mut bytes,
            ProjectionBundleSectionKind::DiagnosticProvenance,
            &provenance_bytes,
        )?;

        Ok(bytes)
    }
}

fn push_section(
    bytes: &mut Vec<u8>,
    kind: ProjectionBundleSectionKind,
    payload: &[u8],
) -> Result<(), ProjectionBundleError> {
    bytes.push(kind.tag());
    push_bytes(bytes, payload)?;
    Ok(())
}

// ---- Per-section canonical encoders ---------------------------------------

fn encode_binding_graph(document: &BindingGraphDocument) -> Result<Vec<u8>, ProjectionBundleError> {
    let mut bytes = Vec::new();
    let mut bindings = document.bindings.clone();
    bindings.sort();
    push_len(&mut bytes, bindings.len())?;
    for binding in &bindings {
        push_u64(&mut bytes, binding.id.0);
        push_u64(&mut bytes, binding.target.node.raw());
        push_u32(&mut bytes, binding.target.slot.0);
        bytes.push(binding_domain_tag(binding.domain));
        push_len(&mut bytes, binding.dependencies.len())?;
        for dependency in &binding.dependencies {
            push_u64(&mut bytes, dependency.target_binding.0);
        }
        push_optional_u64(&mut bytes, binding.collection_key);
        push_optional_u64(&mut bytes, binding.semantic_value.map(|value| value.raw()));
        push_optional_source(&mut bytes, binding.source_ref);
    }
    Ok(bytes)
}

const fn binding_domain_tag(domain: BindingValueDomain) -> u8 {
    match domain {
        BindingValueDomain::Quad => 0,
        BindingValueDomain::Scalar => 1,
        BindingValueDomain::Text => 2,
        BindingValueDomain::Evidence => 3,
        BindingValueDomain::Collection => 4,
        BindingValueDomain::Task => 5,
        BindingValueDomain::Connectivity => 6,
        BindingValueDomain::ActionAvailability => 7,
    }
}

const fn binding_domain_from_tag(tag: u8) -> Option<BindingValueDomain> {
    match tag {
        0 => Some(BindingValueDomain::Quad),
        1 => Some(BindingValueDomain::Scalar),
        2 => Some(BindingValueDomain::Text),
        3 => Some(BindingValueDomain::Evidence),
        4 => Some(BindingValueDomain::Collection),
        5 => Some(BindingValueDomain::Task),
        6 => Some(BindingValueDomain::Connectivity),
        7 => Some(BindingValueDomain::ActionAvailability),
        _ => None,
    }
}

fn encode_action_ir(document: &ActionIrDocument) -> Result<Vec<u8>, ProjectionBundleError> {
    let mut bytes = Vec::new();
    let mut routes = document.routes.clone();
    routes.sort();
    push_len(&mut bytes, routes.len())?;
    for route in &routes {
        push_u64(&mut bytes, route.target.node.raw());
        push_u32(&mut bytes, route.target.slot.0);
        push_u32(&mut bytes, route.order.0);
        push_u64(&mut bytes, route.id.0);
        push_u64(&mut bytes, route.semantic_action.raw());
        push_optional_source(&mut bytes, route.source_ref);
        bytes.push(u8::from(route.context_reqs.requires_actor));
        bytes.push(u8::from(route.context_reqs.requires_session));
        bytes.push(u8::from(route.context_reqs.requires_client));
        push_optional_reference_token(
            &mut bytes,
            route.context_reqs.required_revision.map(|v| v.token()),
        );
        push_optional_reference_token(
            &mut bytes,
            route.context_reqs.required_epoch.map(|v| v.token()),
        );
        push_optional_reference_token(&mut bytes, route.capability.map(|v| v.token()));
    }
    Ok(bytes)
}

fn encode_collection_anchors(
    declarations: &QualifiedCollectionAnchorDeclarations,
) -> Result<Vec<u8>, ProjectionBundleError> {
    let mut bytes = Vec::new();
    push_u64(&mut bytes, declarations.document_id().raw());
    push_u64(&mut bytes, declarations.revision().raw());
    push_u64(&mut bytes, declarations.epoch().raw());
    push_len(&mut bytes, declarations.anchors().len())?;
    for anchor in declarations.anchors() {
        push_u64(&mut bytes, anchor.raw());
    }
    Ok(bytes)
}

fn encode_accessibility(entries: &[AccessibilityEntry]) -> Result<Vec<u8>, ProjectionBundleError> {
    let mut bytes = Vec::new();
    push_len(&mut bytes, entries.len())?;
    for entry in entries {
        push_u64(&mut bytes, entry.node.raw());
        bytes.push(entry.role.tag());
        push_str(&mut bytes, &entry.label)?;
    }
    Ok(bytes)
}

fn encode_provenance(provenance: &BundleProvenance) -> Result<Vec<u8>, ProjectionBundleError> {
    let mut bytes = Vec::new();
    push_str(&mut bytes, &provenance.compiler_identity)?;
    push_u64(&mut bytes, provenance.source_id.raw());
    push_u64(&mut bytes, provenance.source_revision.raw());
    push_u64(&mut bytes, provenance.source_epoch.raw());
    Ok(bytes)
}

// ---- Encoder primitives (mirrors static_ir.rs's private helpers) ---------

fn push_u32(bytes: &mut Vec<u8>, value: u32) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_u64(bytes: &mut Vec<u8>, value: u64) {
    bytes.extend_from_slice(&value.to_le_bytes());
}

fn push_len(bytes: &mut Vec<u8>, value: usize) -> Result<(), ProjectionBundleError> {
    let value = u32::try_from(value).map_err(|_| {
        ProjectionBundleError::new(ProjectionBundleErrorKind::ArithmeticOverflow, None)
    })?;
    push_u32(bytes, value);
    Ok(())
}

fn push_bytes(bytes: &mut Vec<u8>, value: &[u8]) -> Result<(), ProjectionBundleError> {
    push_len(bytes, value.len())?;
    bytes.extend_from_slice(value);
    Ok(())
}

fn push_str(bytes: &mut Vec<u8>, value: &str) -> Result<(), ProjectionBundleError> {
    push_bytes(bytes, value.as_bytes())
}

fn push_optional_u64(bytes: &mut Vec<u8>, value: Option<u64>) {
    match value {
        Some(v) => {
            bytes.push(1);
            push_u64(bytes, v);
        }
        None => bytes.push(0),
    }
}

fn push_optional_source(bytes: &mut Vec<u8>, source: Option<SourceRef>) {
    match source {
        Some(source_ref) => {
            bytes.push(1);
            push_u64(bytes, source_ref.source().raw());
            push_u32(bytes, source_ref.span().start());
            push_u32(bytes, source_ref.span().end());
        }
        None => bytes.push(0),
    }
}

fn push_optional_reference_token(bytes: &mut Vec<u8>, token: Option<ReferenceToken>) {
    match token {
        Some(token) => {
            bytes.push(1);
            push_u64(bytes, token.issuer());
            push_u32(bytes, token.namespace());
            push_u32(bytes, token.generation());
            push_u64(bytes, token.value());
        }
        None => bytes.push(0),
    }
}

// ---- Errors and stages ------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ProjectionBundleStage {
    Resource,
    Decode,
    Header,
    CompleteConsumption,
    Structural,
    CrossArtifact,
    Compatibility,
    Trust,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ProjectionBundleErrorKind {
    InputTooLarge,
    TruncatedPrimitive,
    TruncatedByteString,
    ArithmeticOverflow,
    InvalidUtf8,
    InvalidMagic,
    UnsupportedSchemaVersion,
    UnsupportedContractVersion,
    TrailingBytes,
    MissingSection(ProjectionBundleSectionKind),
    DuplicateSection(ProjectionBundleSectionKind),
    UnknownSectionKind,
    NoncanonicalSectionOrder,
    StaticUiIr,
    Structural,
    CrossArtifactMismatch,
    IncompatibleRoleDictionary,
    ResourceLimitExceeded,
    TrustPlaceholderRejected,
    CanonicalMismatch,
}

impl ProjectionBundleErrorKind {
    const fn stage(self) -> ProjectionBundleStage {
        match self {
            Self::InputTooLarge | Self::ResourceLimitExceeded => ProjectionBundleStage::Resource,
            Self::TruncatedPrimitive
            | Self::TruncatedByteString
            | Self::ArithmeticOverflow
            | Self::InvalidUtf8 => ProjectionBundleStage::Decode,
            Self::InvalidMagic
            | Self::UnsupportedSchemaVersion
            | Self::UnsupportedContractVersion => ProjectionBundleStage::Header,
            Self::TrailingBytes => ProjectionBundleStage::CompleteConsumption,
            Self::MissingSection(_)
            | Self::DuplicateSection(_)
            | Self::UnknownSectionKind
            | Self::NoncanonicalSectionOrder
            | Self::StaticUiIr
            | Self::Structural => ProjectionBundleStage::Structural,
            Self::CrossArtifactMismatch => ProjectionBundleStage::CrossArtifact,
            Self::IncompatibleRoleDictionary => ProjectionBundleStage::Compatibility,
            Self::TrustPlaceholderRejected | Self::CanonicalMismatch => {
                ProjectionBundleStage::Trust
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ProjectionBundleError {
    kind: ProjectionBundleErrorKind,
    byte_offset: Option<usize>,
}

impl ProjectionBundleError {
    const fn new(kind: ProjectionBundleErrorKind, byte_offset: Option<usize>) -> Self {
        Self { kind, byte_offset }
    }

    pub(crate) const fn kind(self) -> ProjectionBundleErrorKind {
        self.kind
    }

    pub(crate) const fn stage(self) -> ProjectionBundleStage {
        self.kind.stage()
    }

    pub(crate) const fn byte_offset(self) -> Option<usize> {
        self.byte_offset
    }
}

// ---- Resource limits (checked before variable-length decoding) -----------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ProjectionBundleLimits {
    pub(crate) max_bundle_bytes: usize,
    pub(crate) max_binding_count: usize,
    pub(crate) max_action_route_count: usize,
    pub(crate) max_collection_anchor_count: usize,
    pub(crate) max_accessibility_entry_count: usize,
    pub(crate) static_ir_limits: StaticUiArtifactV1Limits,
}

// ---- Compatibility context -------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ProjectionBundleCompatContext {
    pub(crate) accepted_schema_version: SchemaVersion,
    pub(crate) accepted_contract_version: ContractVersion,
    pub(crate) role_dictionary: RoleDictionary,
}

// ---- Verified (inert-loaded) bundle ---------------------------------------

/// Opaque verified bundle. Construction is only possible through
/// [`verify_projection_bundle_v0`]. This is pure in-memory inert-loaded
/// evidence -- it is not activation and grants no Shell Player, admission,
/// or Semantic authority by itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct VerifiedProjectionBundleV0 {
    bundle: ProjectionBundleV0,
}

impl VerifiedProjectionBundleV0 {
    pub(crate) fn bundle(&self) -> &ProjectionBundleV0 {
        &self.bundle
    }
}

// ---- Cursor decoder (mirrors static_ir_artifact.rs's checked-offset Cursor)

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn read_exact(&mut self, len: usize) -> Result<&'a [u8], ProjectionBundleError> {
        let end = self
            .offset
            .checked_add(len)
            .ok_or_else(|| self.err(ProjectionBundleErrorKind::ArithmeticOverflow))?;
        if end > self.bytes.len() {
            return Err(self.err(ProjectionBundleErrorKind::TruncatedPrimitive));
        }
        let slice = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(slice)
    }

    fn read_u8(&mut self) -> Result<u8, ProjectionBundleError> {
        Ok(self.read_exact(1)?[0])
    }

    fn read_u32(&mut self) -> Result<u32, ProjectionBundleError> {
        let slice = self.read_exact(4)?;
        Ok(u32::from_le_bytes(slice.try_into().expect("checked len")))
    }

    fn read_u64(&mut self) -> Result<u64, ProjectionBundleError> {
        let slice = self.read_exact(8)?;
        Ok(u64::from_le_bytes(slice.try_into().expect("checked len")))
    }

    fn read_len(&mut self) -> Result<usize, ProjectionBundleError> {
        Ok(self.read_u32()? as usize)
    }

    fn read_byte_string(&mut self) -> Result<&'a [u8], ProjectionBundleError> {
        let len = self.read_len()?;
        let end = self
            .offset
            .checked_add(len)
            .ok_or_else(|| self.err(ProjectionBundleErrorKind::ArithmeticOverflow))?;
        if end > self.bytes.len() {
            return Err(self.err(ProjectionBundleErrorKind::TruncatedByteString));
        }
        self.read_exact(len)
    }

    fn read_str(&mut self) -> Result<String, ProjectionBundleError> {
        let bytes = self.read_byte_string()?;
        core::str::from_utf8(bytes)
            .map(String::from)
            .map_err(|_| self.err(ProjectionBundleErrorKind::InvalidUtf8))
    }

    fn read_optional_u64(&mut self) -> Result<Option<u64>, ProjectionBundleError> {
        match self.read_u8()? {
            0 => Ok(None),
            1 => Ok(Some(self.read_u64()?)),
            _ => Err(self.err(ProjectionBundleErrorKind::TruncatedPrimitive)),
        }
    }

    fn read_optional_source(&mut self) -> Result<Option<SourceRef>, ProjectionBundleError> {
        match self.read_u8()? {
            0 => Ok(None),
            1 => {
                let source = self.read_u64()?;
                let start = self.read_u32()?;
                let end = self.read_u32()?;
                let source_id = SourceId::new(source)
                    .map_err(|_| self.err(ProjectionBundleErrorKind::TruncatedPrimitive))?;
                let span = SourceSpan::new(start, end)
                    .map_err(|_| self.err(ProjectionBundleErrorKind::TruncatedPrimitive))?;
                Ok(Some(SourceRef::new(source_id, span)))
            }
            _ => Err(self.err(ProjectionBundleErrorKind::TruncatedPrimitive)),
        }
    }

    fn read_optional_reference_token(
        &mut self,
    ) -> Result<Option<ReferenceToken>, ProjectionBundleError> {
        match self.read_u8()? {
            0 => Ok(None),
            1 => {
                let issuer = self.read_u64()?;
                let namespace = self.read_u32()?;
                let generation = self.read_u32()?;
                let value = self.read_u64()?;
                Ok(Some(ReferenceToken::new(
                    issuer, namespace, generation, value,
                )))
            }
            _ => Err(self.err(ProjectionBundleErrorKind::TruncatedPrimitive)),
        }
    }

    fn err(&self, kind: ProjectionBundleErrorKind) -> ProjectionBundleError {
        ProjectionBundleError::new(kind, Some(self.offset))
    }
}

// ---- Decoded (not yet structurally/cross-artifact/compat/trust checked) --

struct DecodedBinding {
    id: BindingId,
    target: BindingTarget,
    domain: BindingValueDomain,
    dependencies: Vec<BindingDependency>,
    collection_key: Option<u64>,
    semantic_value: Option<u64>,
    source_ref: Option<SourceRef>,
}

fn decode_binding_graph(bytes: &[u8]) -> Result<Vec<DecodedBinding>, ProjectionBundleError> {
    let mut cursor = Cursor::new(bytes);
    let count = cursor.read_len()?;
    let mut bindings = Vec::with_capacity(count.min(1024));
    for _ in 0..count {
        let id = cursor.read_u64()?;
        let node = cursor.read_u64()?;
        let slot = cursor.read_u32()?;
        let node_id = StaticNodeId::new(node)
            .map_err(|_| cursor.err(ProjectionBundleErrorKind::Structural))?;
        let domain_tag = cursor.read_u8()?;
        let domain = binding_domain_from_tag(domain_tag)
            .ok_or_else(|| cursor.err(ProjectionBundleErrorKind::Structural))?;
        let dep_count = cursor.read_len()?;
        let mut dependencies = Vec::with_capacity(dep_count.min(1024));
        for _ in 0..dep_count {
            dependencies.push(BindingDependency {
                target_binding: BindingId(cursor.read_u64()?),
            });
        }
        let collection_key = cursor.read_optional_u64()?;
        let semantic_value = cursor.read_optional_u64()?;
        let source_ref = cursor.read_optional_source()?;
        bindings.push(DecodedBinding {
            id: BindingId(id),
            target: BindingTarget {
                node: node_id,
                slot: BindingSlot(slot),
            },
            domain,
            dependencies,
            collection_key,
            semantic_value,
            source_ref,
        });
    }
    if cursor.offset != bytes.len() {
        return Err(cursor.err(ProjectionBundleErrorKind::TrailingBytes));
    }
    Ok(bindings)
}

struct DecodedRoute {
    target: ActionTarget,
    order: ActionRouteOrder,
    id: ActionRouteId,
    semantic_action: SemanticActionRef,
    source_ref: Option<SourceRef>,
    context_reqs: ActionContextRequirements,
    capability: Option<CapabilityRef>,
}

fn decode_action_ir(bytes: &[u8]) -> Result<Vec<DecodedRoute>, ProjectionBundleError> {
    let mut cursor = Cursor::new(bytes);
    let count = cursor.read_len()?;
    let mut routes = Vec::with_capacity(count.min(1024));
    for _ in 0..count {
        let node = cursor.read_u64()?;
        let slot = cursor.read_u32()?;
        let order = cursor.read_u32()?;
        let id = cursor.read_u64()?;
        let semantic_action = cursor.read_u64()?;
        let node_id = StaticNodeId::new(node)
            .map_err(|_| cursor.err(ProjectionBundleErrorKind::Structural))?;
        let source_ref = cursor.read_optional_source()?;
        let requires_actor = cursor.read_u8()? != 0;
        let requires_session = cursor.read_u8()? != 0;
        let requires_client = cursor.read_u8()? != 0;
        let required_revision = cursor
            .read_optional_reference_token()?
            .map(RevisionRef::new);
        let required_epoch = cursor.read_optional_reference_token()?.map(EpochRef::new);
        let capability = cursor
            .read_optional_reference_token()?
            .map(CapabilityRef::new);
        routes.push(DecodedRoute {
            target: ActionTarget {
                node: node_id,
                slot: ActionTargetSlot(slot),
            },
            order: ActionRouteOrder(order),
            id: ActionRouteId(id),
            semantic_action: SemanticActionRef::new(semantic_action),
            source_ref,
            context_reqs: ActionContextRequirements {
                requires_actor,
                requires_session,
                requires_client,
                required_revision,
                required_epoch,
            },
            capability,
        });
    }
    if cursor.offset != bytes.len() {
        return Err(cursor.err(ProjectionBundleErrorKind::TrailingBytes));
    }
    Ok(routes)
}

struct DecodedCollectionAnchors {
    document_id: u64,
    revision: u64,
    epoch: u64,
    anchors: Vec<StaticNodeId>,
}

fn decode_collection_anchors(
    bytes: &[u8],
) -> Result<DecodedCollectionAnchors, ProjectionBundleError> {
    let mut cursor = Cursor::new(bytes);
    let document_id = cursor.read_u64()?;
    let revision = cursor.read_u64()?;
    let epoch = cursor.read_u64()?;
    let count = cursor.read_len()?;
    let mut anchors = Vec::with_capacity(count.min(1024));
    for _ in 0..count {
        let raw = cursor.read_u64()?;
        let anchor = StaticNodeId::new(raw)
            .map_err(|_| cursor.err(ProjectionBundleErrorKind::Structural))?;
        // Canonical order is strictly ascending -- this simultaneously
        // proves canonicality and rejects duplicate targets, since a
        // bundle-encoded CollectionAnchors section bypasses the original
        // source-document qualification pipeline's own Rule 2 check.
        if let Some(previous) = anchors.last() {
            if anchor <= *previous {
                return Err(cursor.err(ProjectionBundleErrorKind::Structural));
            }
        }
        anchors.push(anchor);
    }
    if cursor.offset != bytes.len() {
        return Err(cursor.err(ProjectionBundleErrorKind::TrailingBytes));
    }
    Ok(DecodedCollectionAnchors {
        document_id,
        revision,
        epoch,
        anchors,
    })
}

fn decode_accessibility(bytes: &[u8]) -> Result<Vec<AccessibilityEntry>, ProjectionBundleError> {
    let mut cursor = Cursor::new(bytes);
    let count = cursor.read_len()?;
    let mut entries = Vec::with_capacity(count.min(1024));
    for _ in 0..count {
        let node = cursor.read_u64()?;
        let node_id = StaticNodeId::new(node)
            .map_err(|_| cursor.err(ProjectionBundleErrorKind::Structural))?;
        let role_tag = cursor.read_u8()?;
        let role = AccessibilityRole::from_tag(role_tag)
            .ok_or_else(|| cursor.err(ProjectionBundleErrorKind::Structural))?;
        let label = cursor.read_str()?;
        // Canonical order is strictly ascending by node id, mirroring
        // decode_collection_anchors -- proves canonicality and rejects
        // duplicate/out-of-order node entries.
        if let Some(previous) = entries.last().map(|e: &AccessibilityEntry| e.node) {
            if node_id <= previous {
                return Err(cursor.err(ProjectionBundleErrorKind::Structural));
            }
        }
        entries.push(AccessibilityEntry {
            node: node_id,
            role,
            label,
        });
    }
    if cursor.offset != bytes.len() {
        return Err(cursor.err(ProjectionBundleErrorKind::TrailingBytes));
    }
    Ok(entries)
}

fn decode_provenance(bytes: &[u8]) -> Result<BundleProvenance, ProjectionBundleError> {
    let mut cursor = Cursor::new(bytes);
    let compiler_identity = cursor.read_str()?;
    let source_id = cursor.read_u64()?;
    let source_revision = cursor.read_u64()?;
    let source_epoch = cursor.read_u64()?;
    if cursor.offset != bytes.len() {
        return Err(cursor.err(ProjectionBundleErrorKind::TrailingBytes));
    }
    let source_id =
        SourceId::new(source_id).map_err(|_| cursor.err(ProjectionBundleErrorKind::Structural))?;
    Ok(BundleProvenance {
        compiler_identity,
        source_id,
        source_revision: Revision::new(source_revision),
        source_epoch: Epoch::new(source_epoch),
    })
}

// ---- Full verification pipeline -------------------------------------------

/// Verifies one candidate `ProjectionBundle v0` byte sequence end to end:
/// resource preflight, exact bounded decoding, header/magic/version
/// checks, complete-consumption, structural validation (required section
/// presence/order/uniqueness and per-section decode), cross-artifact
/// validation (target references and identity agreement across sections),
/// compatibility validation (bundle and Role Dictionary version
/// acceptance), and deterministic self-consistency ("trust") verification
/// via canonical re-encoding and exact byte equality.
///
/// Returns the opaque [`VerifiedProjectionBundleV0`] only if every stage
/// succeeds. No stage output grants authority to skip a later stage; no
/// partial result escapes a failed stage.
pub(crate) fn verify_projection_bundle_v0(
    bytes: &[u8],
    compat: ProjectionBundleCompatContext,
    limits: ProjectionBundleLimits,
) -> Result<VerifiedProjectionBundleV0, ProjectionBundleError> {
    // Stage: Resource (checked before any variable-length traversal).
    if bytes.len() > limits.max_bundle_bytes {
        return Err(ProjectionBundleError::new(
            ProjectionBundleErrorKind::InputTooLarge,
            None,
        ));
    }

    // Stage: Decode + Header.
    let mut cursor = Cursor::new(bytes);
    let magic = cursor.read_byte_string()?;
    if magic != PROJECTION_BUNDLE_V0_MAGIC {
        return Err(cursor.err(ProjectionBundleErrorKind::InvalidMagic));
    }
    let schema_version_raw = cursor.read_u32()?;
    let contract_version_raw = cursor.read_u32()?;
    let bundle_id_raw = cursor.read_u64()?;
    let bundle_revision_raw = cursor.read_u64()?;
    let bundle_epoch_raw = cursor.read_u64()?;
    let role_dictionary_schema_version_raw = cursor.read_u32()?;
    let role_dictionary_contract_version_raw = cursor.read_u32()?;

    if schema_version_raw != compat.accepted_schema_version.raw() {
        return Err(cursor.err(ProjectionBundleErrorKind::UnsupportedSchemaVersion));
    }
    if contract_version_raw != compat.accepted_contract_version.raw() {
        return Err(cursor.err(ProjectionBundleErrorKind::UnsupportedContractVersion));
    }

    // Stage: Structural (section table).
    let section_count = cursor.read_len()?;
    if section_count != REQUIRED_SECTION_COUNT {
        return Err(cursor.err(ProjectionBundleErrorKind::MissingSection(
            ProjectionBundleSectionKind::StaticUiIr,
        )));
    }

    let mut seen: [bool; REQUIRED_SECTION_COUNT] = [false; REQUIRED_SECTION_COUNT];
    let mut static_ir_bytes: Option<&[u8]> = None;
    let mut binding_graph_bytes: Option<&[u8]> = None;
    let mut action_ir_bytes: Option<&[u8]> = None;
    let mut collection_anchor_bytes: Option<&[u8]> = None;
    let mut accessibility_bytes: Option<&[u8]> = None;
    let mut provenance_bytes: Option<&[u8]> = None;

    for expected_tag in 0..REQUIRED_SECTION_COUNT as u8 {
        let tag = cursor.read_u8()?;
        let kind = ProjectionBundleSectionKind::from_tag(tag)
            .ok_or_else(|| cursor.err(ProjectionBundleErrorKind::UnknownSectionKind))?;
        if tag != expected_tag {
            return Err(cursor.err(ProjectionBundleErrorKind::NoncanonicalSectionOrder));
        }
        if seen[tag as usize] {
            return Err(cursor.err(ProjectionBundleErrorKind::DuplicateSection(kind)));
        }
        seen[tag as usize] = true;
        let payload = cursor.read_byte_string()?;
        match kind {
            ProjectionBundleSectionKind::StaticUiIr => static_ir_bytes = Some(payload),
            ProjectionBundleSectionKind::BindingGraph => binding_graph_bytes = Some(payload),
            ProjectionBundleSectionKind::ActionIr => action_ir_bytes = Some(payload),
            ProjectionBundleSectionKind::CollectionAnchors => {
                collection_anchor_bytes = Some(payload)
            }
            ProjectionBundleSectionKind::Accessibility => accessibility_bytes = Some(payload),
            ProjectionBundleSectionKind::DiagnosticProvenance => provenance_bytes = Some(payload),
        }
    }

    // Stage: CompleteConsumption.
    if cursor.offset != bytes.len() {
        return Err(cursor.err(ProjectionBundleErrorKind::TrailingBytes));
    }

    let static_ir_bytes = static_ir_bytes.expect("all 6 tags observed above");
    let binding_graph_bytes = binding_graph_bytes.expect("all 6 tags observed above");
    let action_ir_bytes = action_ir_bytes.expect("all 6 tags observed above");
    let collection_anchor_bytes = collection_anchor_bytes.expect("all 6 tags observed above");
    let accessibility_bytes = accessibility_bytes.expect("all 6 tags observed above");
    let provenance_bytes = provenance_bytes.expect("all 6 tags observed above");

    // Stage: Structural, per-section decode. The embedded Static UI IR
    // section is independently verified through its own frozen Artifact V1
    // pipeline (structural + role + its own canonical self-consistency).
    let verified_static_ir: VerifiedStaticUiArtifactV1 = verify_static_ui_ir_artifact_v1(
        static_ir_bytes,
        compat.role_dictionary,
        limits.static_ir_limits,
    )
    .map_err(|_err: StaticUiArtifactV1Error| {
        ProjectionBundleError::new(ProjectionBundleErrorKind::StaticUiIr, None)
    })?;
    let static_document = verified_static_ir.document().clone();

    let decoded_bindings = decode_binding_graph(binding_graph_bytes)?;
    if decoded_bindings.len() > limits.max_binding_count {
        return Err(ProjectionBundleError::new(
            ProjectionBundleErrorKind::ResourceLimitExceeded,
            None,
        ));
    }
    let decoded_routes = decode_action_ir(action_ir_bytes)?;
    if decoded_routes.len() > limits.max_action_route_count {
        return Err(ProjectionBundleError::new(
            ProjectionBundleErrorKind::ResourceLimitExceeded,
            None,
        ));
    }
    let decoded_anchors = decode_collection_anchors(collection_anchor_bytes)?;
    if decoded_anchors.anchors.len() > limits.max_collection_anchor_count {
        return Err(ProjectionBundleError::new(
            ProjectionBundleErrorKind::ResourceLimitExceeded,
            None,
        ));
    }
    let decoded_accessibility = decode_accessibility(accessibility_bytes)?;
    if decoded_accessibility.len() > limits.max_accessibility_entry_count {
        return Err(ProjectionBundleError::new(
            ProjectionBundleErrorKind::ResourceLimitExceeded,
            None,
        ));
    }
    let provenance = decode_provenance(provenance_bytes)?;

    let binding_graph = BindingGraphDocument::build(
        decoded_bindings
            .into_iter()
            .map(|b| BindingDeclaration {
                id: b.id,
                target: b.target,
                domain: b.domain,
                dependencies: b.dependencies,
                collection_key: b.collection_key,
                semantic_value: b
                    .semantic_value
                    .map(crate::semantic_refs::SemanticValueRef::new),
                source_ref: b.source_ref,
            })
            .collect(),
        &static_document,
    )
    .map_err(|_| ProjectionBundleError::new(ProjectionBundleErrorKind::Structural, None))?;

    let action_ir = ActionIrDocument::build(
        decoded_routes
            .into_iter()
            .map(|r| ActionRoute {
                target: r.target,
                order: r.order,
                id: r.id,
                semantic_action: r.semantic_action,
                source_ref: r.source_ref,
                context_reqs: r.context_reqs,
                capability: r.capability,
            })
            .collect(),
        &static_document,
    )
    .map_err(|_| ProjectionBundleError::new(ProjectionBundleErrorKind::Structural, None))?;

    // Stage: CrossArtifact -- the CollectionAnchors section's stated
    // document identity must exactly agree with the embedded Static UI IR's
    // actual identity (they are two independently-decoded sections in the
    // same bundle and must not silently diverge).
    if decoded_anchors.document_id != static_document.id().raw()
        || decoded_anchors.revision != static_document.revision().raw()
        || decoded_anchors.epoch != static_document.epoch().raw()
    {
        return Err(ProjectionBundleError::new(
            ProjectionBundleErrorKind::CrossArtifactMismatch,
            None,
        ));
    }
    let static_node_ids: Vec<StaticNodeId> =
        static_document.nodes().iter().map(|n| n.id()).collect();
    if decoded_anchors
        .anchors
        .iter()
        .any(|anchor| !static_node_ids.contains(anchor))
    {
        return Err(ProjectionBundleError::new(
            ProjectionBundleErrorKind::CrossArtifactMismatch,
            None,
        ));
    }
    if decoded_accessibility
        .iter()
        .any(|entry| !static_node_ids.contains(&entry.node))
    {
        return Err(ProjectionBundleError::new(
            ProjectionBundleErrorKind::CrossArtifactMismatch,
            None,
        ));
    }

    // Stage: Compatibility.
    if role_dictionary_schema_version_raw != compat.role_dictionary.schema_version().raw()
        || role_dictionary_contract_version_raw != compat.role_dictionary.contract_version().raw()
    {
        return Err(ProjectionBundleError::new(
            ProjectionBundleErrorKind::IncompatibleRoleDictionary,
            None,
        ));
    }

    let bundle_id = StaticDocumentId::new(bundle_id_raw)
        .map_err(|_| ProjectionBundleError::new(ProjectionBundleErrorKind::Structural, None))?;

    let candidate = ProjectionBundleV0 {
        schema_version: compat.accepted_schema_version,
        contract_version: compat.accepted_contract_version,
        bundle_id,
        bundle_revision: Revision::new(bundle_revision_raw),
        bundle_epoch: Epoch::new(bundle_epoch_raw),
        role_dictionary_schema_version: compat.role_dictionary.schema_version(),
        role_dictionary_contract_version: compat.role_dictionary.contract_version(),
        static_document,
        binding_graph,
        action_ir,
        collection_anchors: rebuild_qualified_collection_anchors(
            bundle_id,
            Revision::new(bundle_revision_raw),
            Epoch::new(bundle_epoch_raw),
            decoded_anchors.anchors,
        ),
        accessibility: decoded_accessibility,
        provenance,
    };

    // Stage: Trust (deterministic self-consistency, NOT cryptographic).
    // Placeholder/malformed trust material never reaches this point without
    // failing an earlier stage; canonical re-encoding additionally proves
    // the candidate has no non-canonical alternate encoding.
    let canonical = candidate
        .canonical_bytes(compat.role_dictionary)
        .map_err(|_| {
            ProjectionBundleError::new(ProjectionBundleErrorKind::CanonicalMismatch, None)
        })?;
    if canonical != bytes {
        return Err(ProjectionBundleError::new(
            ProjectionBundleErrorKind::CanonicalMismatch,
            None,
        ));
    }

    Ok(VerifiedProjectionBundleV0 { bundle: candidate })
}

/// Reconstructs a [`QualifiedCollectionAnchorDeclarations`] from
/// bundle-verified evidence. This is a same-crate rebuild, not a public
/// constructor: the values have already been proven, in this same
/// verification pass, to (a) come from an ascending-sorted, already-`CAD_*`-
/// qualified section and (b) reference nodes that exist in the same
/// bundle's verified Static UI IR.
fn rebuild_qualified_collection_anchors(
    bundle_id: StaticDocumentId,
    revision: Revision,
    epoch: Epoch,
    anchors: Vec<StaticNodeId>,
) -> QualifiedCollectionAnchorDeclarations {
    QualifiedCollectionAnchorDeclarations::from_verified_bundle_evidence(
        bundle_id, revision, epoch, anchors,
    )
}
