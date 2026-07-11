//! Deterministic UI DNA v2 Projection Source to Static UI IR lowering.
#![allow(dead_code)]

use alloc::vec::Vec;

use crate::contract_primitives::{
    ChildOrder, DiagnosticCode, DiagnosticCoordinate, StaticDocumentId, StaticNodeId,
    StaticSurfaceId,
};
use crate::projection_source::{
    ProjectionSourceDiagnosticKind, ProjectionSourceDiagnostics, ProjectionSourceDocument,
};
use crate::role_dictionary::RoleDictionary;
use crate::static_ir::{
    StaticUiChild, StaticUiDocument, StaticUiIrDiagnosticKind, StaticUiIrDiagnostics, StaticUiNode,
    StaticUiSurface,
};

pub(crate) fn compile_projection_source_to_static_ir(
    document_id: StaticDocumentId,
    source: &ProjectionSourceDocument,
    dictionary: RoleDictionary,
) -> Result<StaticUiDocument, ProjectionCompileDiagnostics> {
    let normalized = source
        .normalized(dictionary)
        .map_err(ProjectionCompileDiagnostics::from_source)?;
    let mut document = StaticUiDocument::new(document_id, source.revision(), source.epoch());

    for source_node in normalized.nodes() {
        let mut static_node = StaticUiNode::new(
            static_node_id(source_node.id().raw())?,
            source_node.role(),
            source_node.key(),
            Some(source_node.source()),
        );
        for child in source_node.children() {
            static_node.push_child(StaticUiChild::new(
                static_node_id(child.child().raw())?,
                ChildOrder::new(child.order().raw()),
            ));
        }
        document.push_node(static_node);
    }

    for surface in normalized.surfaces() {
        document.push_surface(StaticUiSurface::new(
            static_surface_id(surface.id().raw())?,
            static_node_id(surface.root().raw())?,
            surface.key(),
            Some(surface.source()),
        ));
    }

    document
        .canonicalized(dictionary)
        .map_err(ProjectionCompileDiagnostics::from_static_ir)
}

fn static_node_id(raw: u64) -> Result<StaticNodeId, ProjectionCompileDiagnostics> {
    StaticNodeId::new(raw).map_err(|_| {
        ProjectionCompileDiagnostics::from_compile_kind(
            ProjectionCompileDiagnosticKind::InvalidLoweredIdentity,
            raw,
            0,
        )
    })
}

fn static_surface_id(raw: u64) -> Result<StaticSurfaceId, ProjectionCompileDiagnostics> {
    StaticSurfaceId::new(raw).map_err(|_| {
        ProjectionCompileDiagnostics::from_compile_kind(
            ProjectionCompileDiagnosticKind::InvalidLoweredIdentity,
            raw,
            1,
        )
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ProjectionCompileDiagnosticKind {
    Source(ProjectionSourceDiagnosticKind),
    StaticIr(StaticUiIrDiagnosticKind),
    InvalidLoweredIdentity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct ProjectionCompileDiagnostic {
    kind: ProjectionCompileDiagnosticKind,
    stage: ProjectionCompileStage,
    coordinate: DiagnosticCoordinate,
}

impl ProjectionCompileDiagnostic {
    const fn new(
        kind: ProjectionCompileDiagnosticKind,
        stage: ProjectionCompileStage,
        coordinate: DiagnosticCoordinate,
    ) -> Self {
        Self {
            kind,
            stage,
            coordinate,
        }
    }

    pub(crate) const fn kind(self) -> ProjectionCompileDiagnosticKind {
        self.kind
    }

    pub(crate) const fn stage(self) -> ProjectionCompileStage {
        self.stage
    }

    pub(crate) const fn coordinate(self) -> DiagnosticCoordinate {
        self.coordinate
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) enum ProjectionCompileStage {
    Source,
    Lowering,
    StaticIr,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ProjectionCompileDiagnostics {
    diagnostics: Vec<ProjectionCompileDiagnostic>,
}

impl ProjectionCompileDiagnostics {
    fn from_compile_kind(
        kind: ProjectionCompileDiagnosticKind,
        domain: u64,
        secondary: u64,
    ) -> Self {
        Self::new(Vec::from([ProjectionCompileDiagnostic::new(
            kind,
            ProjectionCompileStage::Lowering,
            DiagnosticCoordinate::new(
                None,
                DiagnosticCode::new("PC_INVALID_LOWERED_IDENTITY"),
                domain,
                secondary,
            ),
        )]))
    }

    fn from_source(source: ProjectionSourceDiagnostics) -> Self {
        Self::new(
            source
                .diagnostics()
                .iter()
                .map(|diagnostic| {
                    ProjectionCompileDiagnostic::new(
                        ProjectionCompileDiagnosticKind::Source(diagnostic.kind()),
                        ProjectionCompileStage::Source,
                        diagnostic.coordinate(),
                    )
                })
                .collect(),
        )
    }

    fn from_static_ir(static_ir: StaticUiIrDiagnostics) -> Self {
        Self::new(
            static_ir
                .diagnostics()
                .iter()
                .map(|diagnostic| {
                    ProjectionCompileDiagnostic::new(
                        ProjectionCompileDiagnosticKind::StaticIr(diagnostic.kind()),
                        ProjectionCompileStage::StaticIr,
                        diagnostic.coordinate(),
                    )
                })
                .collect(),
        )
    }

    fn new(mut diagnostics: Vec<ProjectionCompileDiagnostic>) -> Self {
        diagnostics.sort();
        diagnostics.dedup();
        Self { diagnostics }
    }

    pub(crate) fn diagnostics(&self) -> &[ProjectionCompileDiagnostic] {
        &self.diagnostics
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract_primitives::{
        ChildOrder, CollectionKey, Epoch, ProjectionSourceNodeId, ProjectionSourceSurfaceId,
        Revision, SourceId, SourceRef, SourceSpan,
    };
    use crate::projection_source::{
        ProjectionSourceChild, ProjectionSourceNode, ProjectionSourceSurface,
    };
    use crate::role_dictionary::RoleId;

    fn source_ref(start: u32, end: u32) -> SourceRef {
        SourceRef::new(
            SourceId::new(1).expect("source id"),
            SourceSpan::new(start, end).expect("source span"),
        )
    }

    fn source_node_id(raw: u64) -> ProjectionSourceNodeId {
        ProjectionSourceNodeId::new(raw).expect("source node id")
    }

    fn source_surface_id(raw: u64) -> ProjectionSourceSurfaceId {
        ProjectionSourceSurfaceId::new(raw).expect("source surface id")
    }

    fn static_doc_id(raw: u64) -> StaticDocumentId {
        StaticDocumentId::new(raw).expect("static document id")
    }

    fn key(raw: u64) -> CollectionKey {
        CollectionKey::new(raw).expect("key")
    }

    fn minimal_source() -> ProjectionSourceDocument {
        let mut source = ProjectionSourceDocument::new(
            SourceId::new(1).expect("source id"),
            Revision::new(0),
            Epoch::new(0),
        );
        source.push_node(ProjectionSourceNode::new(
            source_node_id(10),
            RoleId::new("root"),
            key(10),
            source_ref(0, 1),
        ));
        source.push_surface(ProjectionSourceSurface::new(
            source_surface_id(1),
            source_node_id(10),
            key(1),
            source_ref(0, 1),
        ));
        source
    }

    #[test]
    fn projection_compile_lowers_minimal_source() {
        let document = compile_projection_source_to_static_ir(
            static_doc_id(1),
            &minimal_source(),
            RoleDictionary::current(),
        )
        .expect("compiled");

        assert_eq!(document.surfaces().len(), 1);
        assert_eq!(document.nodes().len(), 1);
    }

    #[test]
    fn projection_compile_repeated_lowering_is_equal() {
        let source = minimal_source();

        let first = compile_projection_source_to_static_ir(
            static_doc_id(1),
            &source,
            RoleDictionary::current(),
        )
        .expect("first");
        let second = compile_projection_source_to_static_ir(
            static_doc_id(1),
            &source,
            RoleDictionary::current(),
        )
        .expect("second");

        assert_eq!(first, second);
        assert_eq!(
            first
                .canonical_bytes(RoleDictionary::current())
                .expect("first bytes"),
            second
                .canonical_bytes(RoleDictionary::current())
                .expect("second bytes")
        );
    }

    #[test]
    fn projection_compile_reports_source_diagnostics() {
        let mut source = ProjectionSourceDocument::new(
            SourceId::new(1).expect("source id"),
            Revision::new(0),
            Epoch::new(0),
        );
        let mut root = ProjectionSourceNode::new(
            source_node_id(10),
            RoleId::new("root"),
            key(10),
            source_ref(0, 1),
        );
        root.push_child(ProjectionSourceChild::new(
            source_node_id(11),
            ChildOrder::new(0),
        ));
        source.push_node(root);
        source.push_node(ProjectionSourceNode::new(
            source_node_id(11),
            RoleId::new("renderer_button"),
            key(11),
            source_ref(2, 3),
        ));
        source.push_surface(ProjectionSourceSurface::new(
            source_surface_id(1),
            source_node_id(10),
            key(1),
            source_ref(0, 1),
        ));

        let diagnostics = compile_projection_source_to_static_ir(
            static_doc_id(1),
            &source,
            RoleDictionary::current(),
        )
        .expect_err("source diagnostic");

        assert_eq!(
            diagnostics.diagnostics()[0].kind(),
            ProjectionCompileDiagnosticKind::Source(ProjectionSourceDiagnosticKind::UnknownRole)
        );
        assert_eq!(
            diagnostics.diagnostics()[0].stage(),
            ProjectionCompileStage::Source
        );
        assert_eq!(
            diagnostics.diagnostics()[0]
                .coordinate()
                .source()
                .expect("source coordinate")
                .span()
                .start(),
            2
        );
        assert_eq!(
            diagnostics.diagnostics()[0].coordinate().code(),
            DiagnosticCode::new("PS_UNKNOWN_ROLE")
        );
        assert_eq!(diagnostics.diagnostics()[0].coordinate().domain(), 11);
        assert_eq!(diagnostics.diagnostics()[0].coordinate().secondary(), 0);
    }

    #[test]
    fn projection_compile_stage_wrappers_preserve_diagnostic_identity() {
        let lowering = ProjectionCompileDiagnostics::from_compile_kind(
            ProjectionCompileDiagnosticKind::InvalidLoweredIdentity,
            41,
            7,
        );
        let lowering_diagnostic = lowering.diagnostics()[0];
        assert_eq!(
            lowering_diagnostic.stage(),
            ProjectionCompileStage::Lowering
        );
        assert_eq!(
            lowering_diagnostic.kind(),
            ProjectionCompileDiagnosticKind::InvalidLoweredIdentity
        );
        assert_eq!(lowering_diagnostic.coordinate().source(), None);
        assert_eq!(
            lowering_diagnostic.coordinate().code(),
            DiagnosticCode::new("PC_INVALID_LOWERED_IDENTITY")
        );
        assert_eq!(lowering_diagnostic.coordinate().domain(), 41);
        assert_eq!(lowering_diagnostic.coordinate().secondary(), 7);

        let static_ir =
            ProjectionCompileDiagnostics::new(Vec::from([ProjectionCompileDiagnostic::new(
                ProjectionCompileDiagnosticKind::StaticIr(StaticUiIrDiagnosticKind::MissingRoot),
                ProjectionCompileStage::StaticIr,
                DiagnosticCoordinate::new(
                    Some(source_ref(3, 4)),
                    DiagnosticCode::new("SIR_MISSING_ROOT"),
                    51,
                    9,
                ),
            )]));
        let static_ir_diagnostic = static_ir.diagnostics()[0];
        assert_eq!(
            static_ir_diagnostic.stage(),
            ProjectionCompileStage::StaticIr
        );
        assert_eq!(
            static_ir_diagnostic.kind(),
            ProjectionCompileDiagnosticKind::StaticIr(StaticUiIrDiagnosticKind::MissingRoot)
        );
        assert_eq!(
            static_ir_diagnostic
                .coordinate()
                .source()
                .expect("static IR coordinate")
                .span()
                .start(),
            3
        );
        assert_eq!(static_ir_diagnostic.coordinate().domain(), 51);
        assert_eq!(static_ir_diagnostic.coordinate().secondary(), 9);
    }

    #[test]
    fn projection_compile_cross_stage_order_is_deterministic() {
        let first = ProjectionCompileDiagnostics::new(Vec::from([
            ProjectionCompileDiagnostic::new(
                ProjectionCompileDiagnosticKind::InvalidLoweredIdentity,
                ProjectionCompileStage::Lowering,
                DiagnosticCoordinate::new(
                    None,
                    DiagnosticCode::new("PC_INVALID_LOWERED_IDENTITY"),
                    2,
                    0,
                ),
            ),
            ProjectionCompileDiagnostic::new(
                ProjectionCompileDiagnosticKind::StaticIr(StaticUiIrDiagnosticKind::MissingRoot),
                ProjectionCompileStage::StaticIr,
                DiagnosticCoordinate::new(None, DiagnosticCode::new("SIR_MISSING_ROOT"), 1, 0),
            ),
        ]));
        let second = ProjectionCompileDiagnostics::new(Vec::from([
            first.diagnostics()[1],
            first.diagnostics()[0],
        ]));

        assert_eq!(first, second);
    }

    #[test]
    fn projection_compile_diagnostic_order_uses_coordinate_not_message_text() {
        let mut source = ProjectionSourceDocument::new(
            SourceId::new(1).expect("source id"),
            Revision::new(0),
            Epoch::new(0),
        );
        source.push_node(ProjectionSourceNode::new(
            source_node_id(10),
            RoleId::new("renderer_button"),
            key(10),
            source_ref(10, 11),
        ));
        source.push_surface(ProjectionSourceSurface::new(
            source_surface_id(1),
            source_node_id(10),
            key(1),
            source_ref(0, 1),
        ));

        let first = compile_projection_source_to_static_ir(
            static_doc_id(1),
            &source,
            RoleDictionary::current(),
        )
        .expect_err("diagnostics");
        let second = compile_projection_source_to_static_ir(
            static_doc_id(1),
            &source,
            RoleDictionary::current(),
        )
        .expect_err("diagnostics");

        assert_eq!(first, second);
        assert_eq!(
            first.diagnostics()[0].kind(),
            ProjectionCompileDiagnosticKind::Source(ProjectionSourceDiagnosticKind::UnknownRole)
        );
    }
}
