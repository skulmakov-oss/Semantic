//! UI-DNA2 explicit `CollectionAnchor` declaration qualification tests.
//!
//! These tests qualify the crate-private, pure in-memory
//! `ProjectionSourceNodeId` declaration -> deterministic source-to-static
//! lowering -> `QualifiedCollectionAnchorDeclarations` path only. They do not
//! exercise prepared-handoff, catalog, stage-4/5/6, patch-application, or
//! public-API surfaces, which remain separately unauthorized.

use crate::collection_anchor_declarations::{
    qualify_collection_anchor_declarations, CollectionAnchorDeclarationDiagnosticKind,
};
use crate::contract_primitives::{
    ChildOrder, CollectionKey, DiagnosticCode, Epoch, ProjectionSourceNodeId,
    ProjectionSourceSurfaceId, Revision, SourceId, SourceRef, SourceSpan, StaticDocumentId,
    StaticNodeId, StaticSurfaceId,
};
use crate::projection_compile::{
    compile_projection_source_to_static_ir, lower_projection_source_node_id,
};
use crate::projection_source::{
    ProjectionSourceChild, ProjectionSourceCollectionAnchorDeclaration, ProjectionSourceDocument,
    ProjectionSourceNode, ProjectionSourceRoleName, ProjectionSourceSurface,
};
use crate::role_dictionary::{RoleDictionary, RoleId};
use crate::static_ir::{StaticUiDocument, StaticUiNode, StaticUiSurface};

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

/// A source document with one root and `count` reachable children at raw ids
/// `20, 21, 22, ...`.
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

/// A static document matching `source`'s identity/version that contains only
/// the root node (raw id 10), intentionally omitting every other node so
/// that any declaration targeting them fails Rule 3 (static membership)
/// while still passing Rule 1 (source membership).
fn missing_node_static_document(source: &ProjectionSourceDocument) -> StaticUiDocument {
    let mut static_document =
        StaticUiDocument::new(static_doc_id(1), source.revision(), source.epoch());
    static_document.push_node(StaticUiNode::new(
        static_node_id(10),
        RoleId::new("root"),
        key(10),
        Some(source_ref(0, 1)),
    ));
    static_document.push_surface(StaticUiSurface::new(
        StaticSurfaceId::new(1).expect("surface id"),
        static_node_id(10),
        key(1),
        Some(source_ref(0, 1)),
    ));
    static_document
}

// 1. Empty declaration set succeeds.
#[test]
fn collection_anchor_qualification_empty_declaration_set_succeeds() {
    let source = source_with_children(2);
    let static_document = compiled(&source);

    let qualified =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect("empty set qualifies");

    assert!(qualified.is_empty());
    assert_eq!(qualified.len(), 0);
    assert_eq!(qualified.document_id(), static_doc_id(1));
    assert_eq!(qualified.revision(), Revision::new(1));
    assert_eq!(qualified.epoch(), Epoch::new(1));
}

// 2. One explicit declaration succeeds.
#[test]
fn collection_anchor_qualification_single_declaration_succeeds() {
    let mut source = source_with_children(1);
    source.push_collection_anchor_declaration(declaration(20, (2, 3)));
    let static_document = compiled(&source);

    let qualified =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect("single declaration qualifies");

    assert_eq!(qualified.anchors(), &[static_node_id(20)]);
}

// 3. Multiple declarations are returned in ascending StaticNodeId.
#[test]
fn collection_anchor_qualification_orders_ascending_static_node_id() {
    let mut source = source_with_children(3);
    source.push_collection_anchor_declaration(declaration(22, (0, 1)));
    source.push_collection_anchor_declaration(declaration(20, (2, 3)));
    source.push_collection_anchor_declaration(declaration(21, (4, 5)));
    let static_document = compiled(&source);

    let qualified =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect("qualifies");

    assert_eq!(
        qualified.anchors(),
        &[static_node_id(20), static_node_id(21), static_node_id(22)]
    );
}

// 4. Reversed authored order produces the identical qualified result.
#[test]
fn collection_anchor_qualification_is_authored_order_independent() {
    let mut forward = source_with_children(3);
    forward.push_collection_anchor_declaration(declaration(20, (0, 1)));
    forward.push_collection_anchor_declaration(declaration(21, (2, 3)));
    forward.push_collection_anchor_declaration(declaration(22, (4, 5)));

    let mut reversed = source_with_children(3);
    reversed.push_collection_anchor_declaration(declaration(22, (4, 5)));
    reversed.push_collection_anchor_declaration(declaration(21, (2, 3)));
    reversed.push_collection_anchor_declaration(declaration(20, (0, 1)));

    let static_document = compiled(&forward);

    let forward_qualified =
        qualify_collection_anchor_declarations(static_doc_id(1), &forward, &static_document)
            .expect("forward qualifies");
    let reversed_qualified =
        qualify_collection_anchor_declarations(static_doc_id(1), &reversed, &static_document)
            .expect("reversed qualifies");

    assert_eq!(forward_qualified, reversed_qualified);
}

// 5. Missing source target produces only CAD_MISSING_TARGET_NODE.
#[test]
fn collection_anchor_qualification_missing_source_target_is_isolated() {
    let mut source = source_with_children(1);
    source.push_collection_anchor_declaration(declaration(999, (0, 1)));
    let static_document = compiled(&source);

    let diagnostics =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect_err("missing target");

    assert_eq!(diagnostics.diagnostics().len(), 1);
    assert_eq!(
        diagnostics.diagnostics()[0].kind(),
        CollectionAnchorDeclarationDiagnosticKind::MissingTargetNode
    );
    assert_eq!(
        diagnostics.diagnostics()[0].coordinate().code(),
        DiagnosticCode::new("CAD_MISSING_TARGET_NODE")
    );
}

// 6. Duplicate declaration produces CAD_DUPLICATE_DECLARATION with
// deterministic SourceRef provenance (the least SourceRef among the
// contributing declarations).
#[test]
fn collection_anchor_qualification_rejects_duplicate_declaration() {
    let mut source = source_with_children(1);
    source.push_collection_anchor_declaration(declaration(20, (0, 1)));
    source.push_collection_anchor_declaration(declaration(20, (2, 3)));
    let static_document = compiled(&source);

    let diagnostics =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect_err("duplicate declaration");

    assert_eq!(diagnostics.diagnostics().len(), 1);
    assert_eq!(
        diagnostics.diagnostics()[0].kind(),
        CollectionAnchorDeclarationDiagnosticKind::DuplicateDeclaration
    );
    assert_eq!(
        diagnostics.diagnostics()[0].coordinate().code(),
        DiagnosticCode::new("CAD_DUPLICATE_DECLARATION")
    );
    assert_eq!(
        diagnostics.diagnostics()[0].coordinate().source(),
        Some(source_ref(0, 1))
    );
}

// 6a. Duplicate diagnostic provenance is independent of authored order: the
// same least SourceRef is selected regardless of which declaration was
// pushed first.
#[test]
fn collection_anchor_qualification_duplicate_diagnostic_is_authored_order_independent() {
    let mut forward = source_with_children(1);
    forward.push_collection_anchor_declaration(declaration(20, (0, 1)));
    forward.push_collection_anchor_declaration(declaration(20, (2, 3)));

    let mut reversed = source_with_children(1);
    reversed.push_collection_anchor_declaration(declaration(20, (2, 3)));
    reversed.push_collection_anchor_declaration(declaration(20, (0, 1)));

    let static_document = compiled(&forward);

    let forward_diagnostics =
        qualify_collection_anchor_declarations(static_doc_id(1), &forward, &static_document)
            .expect_err("forward duplicate");
    let reversed_diagnostics =
        qualify_collection_anchor_declarations(static_doc_id(1), &reversed, &static_document)
            .expect_err("reversed duplicate");

    assert_eq!(forward_diagnostics, reversed_diagnostics);
    assert_eq!(
        forward_diagnostics.diagnostics()[0].coordinate().source(),
        Some(source_ref(0, 1))
    );
}

// 6b. A duplicate declaration whose lowered target is also absent from the
// static document must still be reported as CAD_DUPLICATE_DECLARATION;
// duplicate detection must not depend on Rule 3 (static membership)
// succeeding.
#[test]
fn collection_anchor_qualification_duplicate_with_missing_static_node() {
    let mut source = source_with_children(1);
    source.push_collection_anchor_declaration(declaration(20, (0, 1)));
    source.push_collection_anchor_declaration(declaration(20, (2, 3)));

    // A static document that matches source identity/version but omits the
    // lowered node entirely.
    let static_document = missing_node_static_document(&source);

    let diagnostics =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect_err("duplicate plus missing static node");

    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        diagnostic.kind() == CollectionAnchorDeclarationDiagnosticKind::DuplicateDeclaration
    }));
    assert!(diagnostics.diagnostics().iter().any(|diagnostic| {
        diagnostic.kind() == CollectionAnchorDeclarationDiagnosticKind::MissingStaticNode
    }));
}

// 6c. The combined duplicate-plus-missing-static-node diagnostic set is
// identical regardless of authored declaration order.
#[test]
fn collection_anchor_qualification_duplicate_with_missing_static_node_is_order_independent() {
    let mut forward = source_with_children(1);
    forward.push_collection_anchor_declaration(declaration(20, (0, 1)));
    forward.push_collection_anchor_declaration(declaration(20, (2, 3)));

    let mut reversed = source_with_children(1);
    reversed.push_collection_anchor_declaration(declaration(20, (2, 3)));
    reversed.push_collection_anchor_declaration(declaration(20, (0, 1)));

    let static_document = missing_node_static_document(&forward);

    let forward_diagnostics =
        qualify_collection_anchor_declarations(static_doc_id(1), &forward, &static_document)
            .expect_err("forward");
    let reversed_diagnostics =
        qualify_collection_anchor_declarations(static_doc_id(1), &reversed, &static_document)
            .expect_err("reversed");

    assert_eq!(forward_diagnostics, reversed_diagnostics);
}

// 7. Lowered target absent from the static document produces CAD_MISSING_STATIC_NODE.
#[test]
fn collection_anchor_qualification_missing_static_node_is_isolated() {
    let mut source = source_with_children(2);
    source.push_collection_anchor_declaration(declaration(21, (0, 1)));

    // A static document that matches source identity/version but omits the
    // lowered node, isolating Rule 3 from Rule 1 and Rule 4.
    let mut static_document =
        StaticUiDocument::new(static_doc_id(1), source.revision(), source.epoch());
    static_document.push_node(StaticUiNode::new(
        static_node_id(10),
        RoleId::new("root"),
        key(10),
        Some(source_ref(0, 1)),
    ));
    static_document.push_node(StaticUiNode::new(
        static_node_id(20),
        RoleId::new("text"),
        key(20),
        Some(source_ref(0, 1)),
    ));
    static_document.push_surface(StaticUiSurface::new(
        StaticSurfaceId::new(1).expect("surface id"),
        static_node_id(10),
        key(1),
        Some(source_ref(0, 1)),
    ));

    let diagnostics =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect_err("missing static node");

    assert_eq!(diagnostics.diagnostics().len(), 1);
    assert_eq!(
        diagnostics.diagnostics()[0].kind(),
        CollectionAnchorDeclarationDiagnosticKind::MissingStaticNode
    );
    assert_eq!(
        diagnostics.diagnostics()[0].coordinate().code(),
        DiagnosticCode::new("CAD_MISSING_STATIC_NODE")
    );
}

// 8. Static document ID mismatch produces CAD_DOCUMENT_VERSION_MISMATCH.
#[test]
fn collection_anchor_qualification_document_id_mismatch_is_isolated() {
    let source = source_with_children(1);
    let static_document = compile_projection_source_to_static_ir(
        static_doc_id(2),
        &source,
        RoleDictionary::current(),
    )
    .expect("compiled");

    let diagnostics =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect_err("document id mismatch");

    assert_eq!(diagnostics.diagnostics().len(), 1);
    assert_eq!(
        diagnostics.diagnostics()[0].kind(),
        CollectionAnchorDeclarationDiagnosticKind::DocumentVersionMismatch
    );
    assert_eq!(
        diagnostics.diagnostics()[0].coordinate().code(),
        DiagnosticCode::new("CAD_DOCUMENT_VERSION_MISMATCH")
    );
}

// 9. Revision mismatch produces CAD_DOCUMENT_VERSION_MISMATCH.
#[test]
fn collection_anchor_qualification_revision_mismatch_is_isolated() {
    let source = source_with_children(1);
    let compiled_document = compiled(&source);

    let mut static_document =
        StaticUiDocument::new(static_doc_id(1), Revision::new(999), source.epoch());
    for node in compiled_document.nodes() {
        static_document.push_node(node.clone());
    }
    for surface in compiled_document.surfaces() {
        static_document.push_surface(surface.clone());
    }

    let diagnostics =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect_err("revision mismatch");

    assert_eq!(diagnostics.diagnostics().len(), 1);
    assert_eq!(
        diagnostics.diagnostics()[0].kind(),
        CollectionAnchorDeclarationDiagnosticKind::DocumentVersionMismatch
    );
}

// 10. Epoch mismatch produces CAD_DOCUMENT_VERSION_MISMATCH.
#[test]
fn collection_anchor_qualification_epoch_mismatch_is_isolated() {
    let source = source_with_children(1);
    let compiled_document = compiled(&source);

    let mut static_document =
        StaticUiDocument::new(static_doc_id(1), source.revision(), Epoch::new(999));
    for node in compiled_document.nodes() {
        static_document.push_node(node.clone());
    }
    for surface in compiled_document.surfaces() {
        static_document.push_surface(surface.clone());
    }

    let diagnostics =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect_err("epoch mismatch");

    assert_eq!(diagnostics.diagnostics().len(), 1);
    assert_eq!(
        diagnostics.diagnostics()[0].kind(),
        CollectionAnchorDeclarationDiagnosticKind::DocumentVersionMismatch
    );
}

// 11. One valid plus one invalid declaration returns no partial set.
#[test]
fn collection_anchor_qualification_rejects_partial_set_on_mixed_validity() {
    let mut source = source_with_children(1);
    source.push_collection_anchor_declaration(declaration(20, (0, 1)));
    source.push_collection_anchor_declaration(declaration(999, (2, 3)));
    let static_document = compiled(&source);

    let diagnostics =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect_err("no partial set");

    assert_eq!(diagnostics.diagnostics().len(), 1);
    assert_eq!(
        diagnostics.diagnostics()[0].kind(),
        CollectionAnchorDeclarationDiagnosticKind::MissingTargetNode
    );
}

// 12. Equivalent invalid inputs with different authored order produce
// identical ordered diagnostics.
#[test]
fn collection_anchor_qualification_invalid_diagnostics_are_order_independent() {
    let mut forward = source_with_children(2);
    forward.push_collection_anchor_declaration(declaration(998, (0, 1)));
    forward.push_collection_anchor_declaration(declaration(999, (2, 3)));

    let mut reversed = source_with_children(2);
    reversed.push_collection_anchor_declaration(declaration(999, (2, 3)));
    reversed.push_collection_anchor_declaration(declaration(998, (0, 1)));

    let static_document = compiled(&forward);

    let forward_diagnostics =
        qualify_collection_anchor_declarations(static_doc_id(1), &forward, &static_document)
            .expect_err("forward invalid");
    let reversed_diagnostics =
        qualify_collection_anchor_declarations(static_doc_id(1), &reversed, &static_document)
            .expect_err("reversed invalid");

    assert_eq!(forward_diagnostics, reversed_diagnostics);
}

// 13. Existing nodes without explicit declarations produce an empty set.
#[test]
fn collection_anchor_qualification_nodes_without_declarations_are_not_inferred() {
    let source = source_with_children(3);
    let static_document = compiled(&source);

    let qualified =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect("empty declarations qualify");

    assert!(qualified.is_empty());
}

// 14. A valid explicit declaration does not require a list-like role.
#[test]
fn collection_anchor_qualification_does_not_require_list_like_role() {
    let mut source = source_with_children(1);
    source.push_collection_anchor_declaration(declaration(20, (0, 1)));
    let static_document = compiled(&source);

    let qualified =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect("qualifies despite non-list role");

    assert_eq!(qualified.anchors(), &[static_node_id(20)]);
}

// 15. CollectionKey does not participate in qualified identity.
#[test]
fn collection_anchor_qualification_ignores_collection_key_identity() {
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
    root.push_child(ProjectionSourceChild::new(node_id(20), ChildOrder::new(0)));
    source.push_node(root);
    // CollectionKey deliberately unrelated to node identity.
    source.push_node(ProjectionSourceNode::new(
        node_id(20),
        ProjectionSourceRoleName::new("text"),
        key(555),
        source_ref(0, 1),
    ));
    source.push_surface(ProjectionSourceSurface::new(
        ProjectionSourceSurfaceId::new(1).expect("surface id"),
        node_id(10),
        key(1),
        source_ref(0, 1),
    ));
    source.push_collection_anchor_declaration(declaration(20, (2, 3)));

    let static_document = compiled(&source);

    let qualified =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect("qualifies regardless of CollectionKey value");

    assert_eq!(qualified.anchors(), &[static_node_id(20)]);
}

// 16. Repeated qualification of identical input is exactly equal.
#[test]
fn collection_anchor_qualification_repeated_qualification_is_equal() {
    let mut source = source_with_children(2);
    source.push_collection_anchor_declaration(declaration(20, (0, 1)));
    source.push_collection_anchor_declaration(declaration(21, (2, 3)));
    let static_document = compiled(&source);

    let first = qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
        .expect("first");
    let second =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect("second");

    assert_eq!(first, second);
}

// 17. The source-to-static lowering used by qualification matches the
// compiler's lowering path.
#[test]
fn collection_anchor_qualification_lowering_matches_compiler_lowering() {
    let mut source = source_with_children(1);
    source.push_collection_anchor_declaration(declaration(20, (0, 1)));
    let static_document = compiled(&source);

    let qualified =
        qualify_collection_anchor_declarations(static_doc_id(1), &source, &static_document)
            .expect("qualifies");

    let expected =
        lower_projection_source_node_id(node_id(20)).expect("compiler lowering succeeds");

    assert_eq!(qualified.anchors()[0], expected);
}
