//! UI-DNA2 `PreparedActiveProjectionTargets` qualification tests.
//!
//! These tests qualify the crate-private, pure in-memory
//! `(StaticUiDocument, BindingGraphDocument, QualifiedCollectionAnchorDeclarations)`
//! -> `PreparedActiveProjectionTargets` derivation path only. They do not
//! exercise the runtime catalog, stage-4/5/6, patch application, or
//! public-API surfaces, which remain separately bounded.

use alloc::vec;

use crate::binding_graph::{
    BindingDeclaration, BindingGraphDocument, BindingId, BindingSlot, BindingTarget,
    BindingValueDomain,
};
use crate::collection_anchor_declarations::qualify_collection_anchor_declarations;
use crate::contract_primitives::{
    ChildOrder, CollectionKey, Epoch, ProjectionSourceNodeId, ProjectionSourceSurfaceId, Revision,
    SourceId, SourceRef, SourceSpan, StaticDocumentId, StaticNodeId,
};
use crate::prepared_active_projection_targets::{
    derive_prepared_active_projection_targets, PreparedActiveProjectionTargetsError,
};
use crate::projection_compile::compile_projection_source_to_static_ir;
use crate::projection_source::{
    ProjectionSourceChild, ProjectionSourceCollectionAnchorDeclaration, ProjectionSourceDocument,
    ProjectionSourceNode, ProjectionSourceRoleName, ProjectionSourceSurface,
};
use crate::role_dictionary::RoleDictionary;
use crate::static_ir::StaticUiDocument;
use crate::target_anchor::{BindingAnchor, CollectionAnchor, NodeAnchor};

fn source_ref(start: u32, end: u32) -> SourceRef {
    SourceRef::new(
        SourceId::new(1).expect("source id"),
        SourceSpan::new(start, end).expect("source span"),
    )
}

fn node_id(raw: u64) -> ProjectionSourceNodeId {
    ProjectionSourceNodeId::new(raw).expect("source node id")
}

fn static_node_id(raw: u64) -> StaticNodeId {
    StaticNodeId::new(raw).expect("static node id")
}

fn key(raw: u64) -> CollectionKey {
    CollectionKey::new(raw).expect("collection key")
}

fn static_doc_id(raw: u64) -> StaticDocumentId {
    StaticDocumentId::new(raw).expect("static document id")
}

fn declaration(target: u64, span: (u32, u32)) -> ProjectionSourceCollectionAnchorDeclaration {
    ProjectionSourceCollectionAnchorDeclaration::new(node_id(target), source_ref(span.0, span.1))
}

/// A source document with one root (raw id 10) and `count` reachable
/// children at raw ids `20, 21, 22, ...`.
fn source_with_children(count: u64) -> ProjectionSourceDocument {
    let mut source = ProjectionSourceDocument::new(
        SourceId::new(1).expect("source id"),
        Revision::new(1),
        Epoch::new(1),
    );
    let mut root = ProjectionSourceNode::new(
        node_id(10),
        ProjectionSourceRoleName::new("root"),
        key(10),
        source_ref(0, 1),
    );
    for index in 0..count {
        root.push_child(ProjectionSourceChild::new(
            node_id(20 + index),
            ChildOrder::new(index as u32),
        ));
    }
    source.push_node(root);
    for index in 0..count {
        source.push_node(ProjectionSourceNode::new(
            node_id(20 + index),
            ProjectionSourceRoleName::new("text"),
            key(20 + index),
            source_ref(0, 1),
        ));
    }
    source.push_surface(ProjectionSourceSurface::new(
        ProjectionSourceSurfaceId::new(1).expect("surface id"),
        node_id(10),
        key(1),
        source_ref(0, 1),
    ));
    source
}

fn compiled(source: &ProjectionSourceDocument) -> StaticUiDocument {
    compile_projection_source_to_static_ir(static_doc_id(1), source, RoleDictionary::current())
        .expect("compiled")
}

fn binding(raw_id: u64, node_raw: u64, slot_raw: u32) -> BindingDeclaration {
    BindingDeclaration {
        id: BindingId(raw_id),
        target: BindingTarget {
            node: static_node_id(node_raw),
            slot: BindingSlot(slot_raw),
        },
        domain: BindingValueDomain::Text,
        dependencies: Vec::new(),
        collection_key: None,
        semantic_value: None,
        source_ref: None,
    }
}

#[test]
fn test_every_static_node_becomes_a_node_anchor() {
    let source = source_with_children(3);
    let static_document = compiled(&source);
    let binding_graph = BindingGraphDocument::build(Vec::new(), &static_document).expect("empty");
    let collection_anchors =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect("empty declaration set qualifies");

    let prepared = derive_prepared_active_projection_targets(
        &static_document,
        &binding_graph,
        &collection_anchors,
    )
    .expect("derivation succeeds");

    // root (10) + 3 children (20, 21, 22) = 4 nodes.
    assert_eq!(
        prepared.node_anchors(),
        &[
            NodeAnchor::new(static_node_id(10)),
            NodeAnchor::new(static_node_id(20)),
            NodeAnchor::new(static_node_id(21)),
            NodeAnchor::new(static_node_id(22)),
        ]
    );
}

#[test]
fn test_every_qualified_binding_becomes_a_binding_anchor() {
    let source = source_with_children(2);
    let static_document = compiled(&source);
    let bindings = vec![binding(1, 20, 0), binding(2, 21, 1)];
    let binding_graph = BindingGraphDocument::build(bindings, &static_document).expect("valid");
    let collection_anchors =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect("empty declaration set qualifies");

    let prepared = derive_prepared_active_projection_targets(
        &static_document,
        &binding_graph,
        &collection_anchors,
    )
    .expect("derivation succeeds");

    assert_eq!(
        prepared.binding_anchors(),
        &[
            BindingAnchor::new(static_node_id(20), BindingSlot(0)),
            BindingAnchor::new(static_node_id(21), BindingSlot(1)),
        ]
    );
}

#[test]
fn test_only_explicitly_declared_nodes_become_collection_anchors() {
    let mut source = source_with_children(3);
    // Only node 21 is explicitly declared; 20 and 22 must not appear even
    // though they exist as static nodes.
    source.push_collection_anchor_declaration(declaration(21, (2, 3)));
    let static_document = compiled(&source);
    let binding_graph = BindingGraphDocument::build(Vec::new(), &static_document).expect("empty");
    let collection_anchors =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect("declaration qualifies");

    let prepared = derive_prepared_active_projection_targets(
        &static_document,
        &binding_graph,
        &collection_anchors,
    )
    .expect("derivation succeeds");

    assert_eq!(
        prepared.collection_anchors(),
        &[CollectionAnchor::new(static_node_id(21))]
    );
    // Node existence alone does not imply collection-anchor membership.
    assert!(prepared
        .node_anchors()
        .contains(&NodeAnchor::new(static_node_id(20))));
    assert!(!prepared
        .collection_anchors()
        .contains(&CollectionAnchor::new(static_node_id(20))));
}

#[test]
fn test_no_collection_anchors_when_none_declared() {
    let source = source_with_children(2);
    let static_document = compiled(&source);
    let binding_graph = BindingGraphDocument::build(Vec::new(), &static_document).expect("empty");
    let collection_anchors =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect("empty declaration set qualifies");

    let prepared = derive_prepared_active_projection_targets(
        &static_document,
        &binding_graph,
        &collection_anchors,
    )
    .expect("derivation succeeds");

    assert!(prepared.collection_anchors().is_empty());
    assert!(!prepared.node_anchors().is_empty());
}

#[test]
fn test_incoherent_collection_anchor_source_is_rejected() {
    let source = source_with_children(2);
    let static_document = compiled(&source);
    let binding_graph = BindingGraphDocument::build(Vec::new(), &static_document).expect("empty");
    let collection_anchors =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect("empty declaration set qualifies");

    // A static document with a different identity must be rejected as
    // incoherent with the already-qualified collection-anchor evidence, even
    // though the qualification call itself succeeded.
    let mismatched_document =
        StaticUiDocument::new(static_doc_id(2), Revision::new(1), Epoch::new(1));

    let result = derive_prepared_active_projection_targets(
        &mismatched_document,
        &binding_graph,
        &collection_anchors,
    );

    assert_eq!(
        result,
        Err(PreparedActiveProjectionTargetsError::IncoherentCollectionAnchorSource)
    );
}

#[test]
fn test_ordering_is_deterministic_regardless_of_declaration_order() {
    let mut source = source_with_children(3);
    source.push_collection_anchor_declaration(declaration(22, (0, 1)));
    source.push_collection_anchor_declaration(declaration(20, (2, 3)));
    let static_document = compiled(&source);
    let bindings = vec![binding(2, 21, 5), binding(1, 20, 1)];
    let binding_graph = BindingGraphDocument::build(bindings, &static_document).expect("valid");
    let collection_anchors =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect("qualifies");

    let prepared = derive_prepared_active_projection_targets(
        &static_document,
        &binding_graph,
        &collection_anchors,
    )
    .expect("derivation succeeds");

    let mut sorted_bindings = prepared.binding_anchors().to_vec();
    sorted_bindings.sort();
    assert_eq!(prepared.binding_anchors(), sorted_bindings.as_slice());

    let mut sorted_collections = prepared.collection_anchors().to_vec();
    sorted_collections.sort();
    assert_eq!(prepared.collection_anchors(), sorted_collections.as_slice());
}

#[test]
fn test_determinism_identical_input_produces_identical_output() {
    let mut source = source_with_children(2);
    source.push_collection_anchor_declaration(declaration(20, (0, 1)));
    let static_document = compiled(&source);
    let binding_graph =
        BindingGraphDocument::build(vec![binding(1, 21, 0)], &static_document).expect("valid");
    let collection_anchors =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect("qualifies");

    let prepared_1 = derive_prepared_active_projection_targets(
        &static_document,
        &binding_graph,
        &collection_anchors,
    )
    .expect("derivation succeeds");
    let prepared_2 = derive_prepared_active_projection_targets(
        &static_document,
        &binding_graph,
        &collection_anchors,
    )
    .expect("derivation succeeds");

    assert_eq!(prepared_1, prepared_2);
}
