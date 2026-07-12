use alloc::string::String;

use crate::contract_primitives::{
    ChildOrder, CollectionKey, Epoch, ProjectionSourceNodeId, ProjectionSourceSurfaceId, Revision,
    SourceId, SourceRef, SourceSpan, StaticDocumentId, StaticSurfaceId,
};
use crate::model::{UiAstNodeId, UiIr, UiIrNode, UiIrNodeId, UiIrNodeKind};
use crate::projection_compile::compile_projection_source_to_static_ir;
use crate::projection_source::{
    ProjectionSourceChild, ProjectionSourceDocument, ProjectionSourceNode,
    ProjectionSourceRoleName, ProjectionSourceSurface,
};
use crate::role_dictionary::RoleDictionary;
use crate::static_ir::{LegacyUiIrAdapterContext, StaticUiDocument};

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

fn static_surface_id(raw: u64) -> StaticSurfaceId {
    StaticSurfaceId::new(raw).expect("static surface id")
}

fn key(raw: u64) -> CollectionKey {
    CollectionKey::new(raw).expect("key")
}

fn legacy_context<'a>(source_refs: &'a [(UiAstNodeId, SourceRef)]) -> LegacyUiIrAdapterContext<'a> {
    LegacyUiIrAdapterContext::new(
        static_doc_id(1),
        static_surface_id(1),
        key(1),
        Some(source_ref(0, 1)),
        Revision::new(0),
        Epoch::new(0),
        source_refs,
    )
}

fn minimal_source() -> ProjectionSourceDocument {
    let mut source = ProjectionSourceDocument::new(
        SourceId::new(1).expect("source id"),
        Revision::new(0),
        Epoch::new(0),
    );
    source.push_node(ProjectionSourceNode::new(
        source_node_id(10),
        ProjectionSourceRoleName::new("root"),
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

fn multi_surface_source(order: &[u64]) -> ProjectionSourceDocument {
    let mut source = ProjectionSourceDocument::new(
        SourceId::new(1).expect("source id"),
        Revision::new(0),
        Epoch::new(0),
    );
    for raw in order {
        let role = if *raw == 10 {
            ProjectionSourceRoleName::new("root")
        } else {
            ProjectionSourceRoleName::new("text")
        };
        source.push_node(ProjectionSourceNode::new(
            source_node_id(*raw),
            role,
            key(*raw),
            source_ref(*raw as u32, *raw as u32 + 1),
        ));
    }
    for raw in order {
        source.push_surface(ProjectionSourceSurface::new(
            source_surface_id(*raw),
            source_node_id(*raw),
            key(*raw),
            source_ref(*raw as u32, *raw as u32 + 1),
        ));
    }
    source
}

#[test]
fn ui_dna2_wp2_minimal_valid_projection_has_stable_bytes() {
    let source = minimal_source();
    let first = compile_projection_source_to_static_ir(
        static_doc_id(1),
        &source,
        RoleDictionary::current(),
    )
    .expect("compile");
    let second = compile_projection_source_to_static_ir(
        static_doc_id(1),
        &source,
        RoleDictionary::current(),
    )
    .expect("compile again");

    assert_eq!(first, second);
    assert_eq!(
        first
            .canonical_bytes(RoleDictionary::current())
            .expect("bytes"),
        second
            .canonical_bytes(RoleDictionary::current())
            .expect("bytes again")
    );
}

#[test]
fn ui_dna2_wp2_multi_surface_projection_is_deterministic() {
    let first = compile_projection_source_to_static_ir(
        static_doc_id(1),
        &multi_surface_source(&[10, 20]),
        RoleDictionary::current(),
    )
    .expect("first");
    let second = compile_projection_source_to_static_ir(
        static_doc_id(1),
        &multi_surface_source(&[20, 10]),
        RoleDictionary::current(),
    )
    .expect("second");

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
fn ui_dna2_wp2_invalid_projection_diagnostics_are_stable() {
    let mut source = minimal_source();
    source.push_node(ProjectionSourceNode::new(
        source_node_id(20),
        ProjectionSourceRoleName::new("renderer_button"),
        key(20),
        source_ref(20, 21),
    ));

    let first = compile_projection_source_to_static_ir(
        static_doc_id(1),
        &source,
        RoleDictionary::current(),
    )
    .expect_err("first diagnostics");
    let second = compile_projection_source_to_static_ir(
        static_doc_id(1),
        &source,
        RoleDictionary::current(),
    )
    .expect_err("second diagnostics");

    assert_eq!(first, second);
}

#[test]
fn ui_dna2_wp2_owned_unknown_role_survives_source_drop() {
    let role = String::from("renderer_button");
    let mut source = ProjectionSourceDocument::new(
        SourceId::new(1).expect("source id"),
        Revision::new(0),
        Epoch::new(0),
    );
    let mut root = ProjectionSourceNode::new(
        source_node_id(10),
        ProjectionSourceRoleName::new("root"),
        key(10),
        source_ref(0, 1),
    );
    root.push_child(ProjectionSourceChild::new(
        source_node_id(20),
        ChildOrder::new(0),
    ));
    source.push_node(root);
    source.push_node(ProjectionSourceNode::new(
        source_node_id(20),
        ProjectionSourceRoleName::new(&role),
        key(20),
        source_ref(20, 21),
    ));
    source.push_surface(ProjectionSourceSurface::new(
        source_surface_id(1),
        source_node_id(10),
        key(1),
        source_ref(0, 1),
    ));
    drop(role);

    let diagnostics = compile_projection_source_to_static_ir(
        static_doc_id(1),
        &source,
        RoleDictionary::current(),
    )
    .expect_err("unknown role must fail semantic validation");

    assert_eq!(
        diagnostics.diagnostics()[0].kind(),
        crate::projection_compile::ProjectionCompileDiagnosticKind::Source(
            crate::projection_source::ProjectionSourceDiagnosticKind::UnknownRole
        )
    );
}

#[test]
fn ui_dna2_wp2_legacy_ui_ir_adaptation_is_stable_and_bounded() {
    let mut ir = UiIr::new();
    let mut root = UiIrNode::new(UiIrNodeId::new(1), UiIrNodeKind::Root);
    root.set_source_ast_node_id(Some(UiAstNodeId::new(101)));
    root.push_child(UiIrNodeId::new(2));
    ir.push_node(root);
    let mut child =
        UiIrNode::with_parent(UiIrNodeId::new(2), UiIrNodeKind::Text, UiIrNodeId::new(1));
    child.set_source_ast_node_id(Some(UiAstNodeId::new(102)));
    ir.push_node(child);
    let source_refs = [
        (UiAstNodeId::new(101), source_ref(0, 1)),
        (UiAstNodeId::new(102), source_ref(2, 3)),
    ];

    let first = StaticUiDocument::from_legacy_ui_ir(legacy_context(&source_refs), &ir)
        .expect("first adapter");
    let second = StaticUiDocument::from_legacy_ui_ir(legacy_context(&source_refs), &ir)
        .expect("second adapter");

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
fn ui_dna2_wp2_stable_keyed_collection_vector_is_order_independent() {
    let first = multi_surface_source(&[10, 20, 30]);
    let second = multi_surface_source(&[30, 10, 20]);

    assert_eq!(
        first
            .normalized(RoleDictionary::current())
            .expect("first")
            .nodes()
            .iter()
            .map(|node| node.key().raw())
            .collect::<alloc::vec::Vec<_>>(),
        second
            .normalized(RoleDictionary::current())
            .expect("second")
            .nodes()
            .iter()
            .map(|node| node.key().raw())
            .collect::<alloc::vec::Vec<_>>()
    );
}
