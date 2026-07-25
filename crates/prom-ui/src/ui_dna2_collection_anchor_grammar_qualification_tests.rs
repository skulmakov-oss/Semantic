//! Qualification tests for the textual `CollectionAnchor` declaration
//! grammar extension (`collection_anchor <nonzero_u64>;`) and its wiring
//! through the parser and the parser-to-compiler frontend.
//!
//! Scope: the parser accepts/rejects the new clause shape and populates
//! `ProjectionSourceDocument::collection_anchor_declarations`; qualification
//! against the compiled Static UI IR (`CAD_*`) is exercised end to end via
//! `compile_projection_source_text_with_collection_anchors`. Duplicate
//! declarations and unresolved targets are qualification-time, not
//! parser-time, concerns -- proven explicitly below.

use crate::collection_anchor_declarations::CollectionAnchorDeclarationDiagnosticKind;
use crate::contract_primitives::{SourceId, StaticDocumentId};
use crate::projection_source_frontend::{
    compile_projection_source_text_with_collection_anchors,
    ProjectionSourceFrontendWithAnchorsError,
};
use crate::projection_source_parser::{
    parse_projection_source, ProjectionSourceParseError,
    ProjectionSourceParserDiagnosticKind as Kind,
};
use crate::role_dictionary::RoleDictionary;

const SOURCE_RAW: u64 = 61;
const DOCUMENT_RAW: u64 = 61;

fn source_id() -> SourceId {
    SourceId::new(SOURCE_RAW).expect("non-zero test source id")
}

fn document_id() -> StaticDocumentId {
    StaticDocumentId::new(DOCUMENT_RAW).expect("non-zero test document id")
}

fn base_source(extra: &str) -> alloc::string::String {
    alloc::format!(
        concat!(
            "projection v0 {{\n",
            "revision 1; epoch 1;\n",
            "surface 1 root 10 key 1;\n",
            "node 10 role root key 10 {{ child 11 order 0; child 12 order 1; }}\n",
            "node 11 role text key 11 {{}}\n",
            "node 12 role text key 12 {{}}\n",
            "{extra}",
            "}}"
        ),
        extra = extra
    )
}

#[test]
fn parser_accepts_one_textual_collection_anchor_declaration() {
    let source = base_source("collection_anchor 11;\n");
    let document = parse_projection_source(source_id(), &source).expect("valid source");
    assert_eq!(document.collection_anchor_declarations().len(), 1);
    assert_eq!(
        document.collection_anchor_declarations()[0].target().raw(),
        11
    );
}

#[test]
fn parser_accepts_multiple_declarations_and_preserves_authored_order() {
    let source = base_source("collection_anchor 12;\ncollection_anchor 11;\n");
    let document = parse_projection_source(source_id(), &source).expect("valid source");
    let declarations = document.collection_anchor_declarations();
    assert_eq!(declarations.len(), 2);
    assert_eq!(declarations[0].target().raw(), 12);
    assert_eq!(declarations[1].target().raw(), 11);
}

#[test]
fn parser_is_deterministic_across_repeated_parses() {
    let source = base_source("collection_anchor 11;\ncollection_anchor 12;\n");
    let first = parse_projection_source(source_id(), &source).expect("valid source");
    let second = parse_projection_source(source_id(), &source).expect("valid source");
    assert_eq!(first, second);
}

#[test]
fn parser_accepts_duplicate_textual_targets_duplicate_rejection_is_qualification_time_only() {
    // The parser itself does not reject a repeated target: CAD_DUPLICATE_DECLARATION
    // is owned by qualify_collection_anchor_declarations, not the parser, per
    // docs/spec/ui/projection_source_model.md section 12 Rule 2.
    let source = base_source("collection_anchor 11;\ncollection_anchor 11;\n");
    let document = parse_projection_source(source_id(), &source).expect("valid source");
    assert_eq!(document.collection_anchor_declarations().len(), 2);
    assert_eq!(
        document.collection_anchor_declarations()[0].target().raw(),
        11
    );
    assert_eq!(
        document.collection_anchor_declarations()[1].target().raw(),
        11
    );
}

#[test]
fn parser_accepts_declaration_targeting_a_node_not_declared_anywhere_else() {
    // Target-node existence (CAD_MISSING_TARGET_NODE) is a qualification-time
    // concern; the parser does not cross-check declared node identities.
    let source = base_source("collection_anchor 999;\n");
    let document = parse_projection_source(source_id(), &source).expect("valid source");
    assert_eq!(
        document.collection_anchor_declarations()[0].target().raw(),
        999
    );
}

#[test]
fn parser_rejects_zero_id_operand() {
    let source = base_source("collection_anchor 0;\n");
    match parse_projection_source(source_id(), &source) {
        Err(ProjectionSourceParseError::Diagnostic(diagnostic)) => {
            assert_eq!(diagnostic.kind(), Kind::ZeroId);
        }
        other => panic!("expected PSP_ZERO_ID diagnostic, got {other:?}"),
    }
}

#[test]
fn parser_rejects_missing_semicolon() {
    let source = base_source("collection_anchor 11\n");
    match parse_projection_source(source_id(), &source) {
        Err(ProjectionSourceParseError::Diagnostic(diagnostic)) => {
            assert_eq!(diagnostic.kind(), Kind::MissingSemicolon);
        }
        other => panic!("expected PSP_MISSING_SEMICOLON diagnostic, got {other:?}"),
    }
}

#[test]
fn parser_rejects_non_numeric_operand() {
    let source = base_source("collection_anchor node;\n");
    match parse_projection_source(source_id(), &source) {
        Err(ProjectionSourceParseError::Diagnostic(diagnostic)) => {
            assert_eq!(diagnostic.kind(), Kind::UnexpectedToken);
        }
        other => panic!("expected diagnostic, got {other:?}"),
    }
}

#[test]
fn parser_reports_exact_diagnostic_coordinates_for_zero_id() {
    let source = base_source("collection_anchor 0;\n");
    let needle = "collection_anchor 0;";
    let needle_start = source.find(needle).expect("needle present");
    let start = needle_start + "collection_anchor ".len();
    let end = start + 1; // just the digit "0", not the semicolon
    match parse_projection_source(source_id(), &source) {
        Err(ProjectionSourceParseError::Diagnostic(diagnostic)) => {
            assert_eq!(
                diagnostic.span().start(),
                u32::try_from(start).expect("representable")
            );
            assert_eq!(
                diagnostic.span().end(),
                u32::try_from(end).expect("representable")
            );
            assert_eq!(diagnostic.source_id(), source_id());
        }
        other => panic!("expected diagnostic, got {other:?}"),
    }
}

#[test]
fn frontend_preserves_textual_declaration_through_qualification() {
    let source = base_source("collection_anchor 11;\n");
    let compiled = compile_projection_source_text_with_collection_anchors(
        source_id(),
        &source,
        document_id(),
        RoleDictionary::current(),
    )
    .expect("valid source qualifies");

    assert_eq!(compiled.collection_anchors().len(), 1);
    assert_eq!(compiled.collection_anchors().anchors()[0].raw(), 11);
    assert_eq!(compiled.collection_anchors().document_id(), document_id());
}

#[test]
fn frontend_qualifies_multiple_declarations_in_ascending_static_node_id_order() {
    let source = base_source("collection_anchor 12;\ncollection_anchor 11;\n");
    let compiled = compile_projection_source_text_with_collection_anchors(
        source_id(),
        &source,
        document_id(),
        RoleDictionary::current(),
    )
    .expect("valid source qualifies");

    // Rule 7: deterministic ascending StaticNodeId ordering, independent of
    // authored declaration order (12 was authored first, 11 second).
    let anchors = compiled.collection_anchors().anchors();
    assert_eq!(anchors.len(), 2);
    assert_eq!(anchors[0].raw(), 11);
    assert_eq!(anchors[1].raw(), 12);
}

#[test]
fn frontend_rejects_duplicate_textual_declarations_at_qualification_time() {
    let source = base_source("collection_anchor 11;\ncollection_anchor 11;\n");
    match compile_projection_source_text_with_collection_anchors(
        source_id(),
        &source,
        document_id(),
        RoleDictionary::current(),
    ) {
        Err(ProjectionSourceFrontendWithAnchorsError::CollectionAnchor(diagnostics)) => {
            assert_eq!(diagnostics.diagnostics().len(), 1);
            assert_eq!(
                diagnostics.diagnostics()[0].kind(),
                CollectionAnchorDeclarationDiagnosticKind::DuplicateDeclaration
            );
        }
        other => panic!("expected CAD_DUPLICATE_DECLARATION rejection, got {other:?}"),
    }
}

#[test]
fn frontend_rejects_declaration_targeting_undeclared_node_at_qualification_time() {
    let source = base_source("collection_anchor 999;\n");
    match compile_projection_source_text_with_collection_anchors(
        source_id(),
        &source,
        document_id(),
        RoleDictionary::current(),
    ) {
        Err(ProjectionSourceFrontendWithAnchorsError::CollectionAnchor(diagnostics)) => {
            assert_eq!(diagnostics.diagnostics().len(), 1);
            assert_eq!(
                diagnostics.diagnostics()[0].kind(),
                CollectionAnchorDeclarationDiagnosticKind::MissingTargetNode
            );
        }
        other => panic!("expected CAD_MISSING_TARGET_NODE rejection, got {other:?}"),
    }
}

#[test]
fn frontend_accepts_zero_declarations_as_empty_qualified_set() {
    let source = base_source("");
    let compiled = compile_projection_source_text_with_collection_anchors(
        source_id(),
        &source,
        document_id(),
        RoleDictionary::current(),
    )
    .expect("valid source with no declarations qualifies");
    assert!(compiled.collection_anchors().is_empty());
}

#[test]
fn frontend_is_deterministic_across_repeated_compiles() {
    let source = base_source("collection_anchor 12;\ncollection_anchor 11;\n");
    let first = compile_projection_source_text_with_collection_anchors(
        source_id(),
        &source,
        document_id(),
        RoleDictionary::current(),
    )
    .expect("valid source qualifies");
    let second = compile_projection_source_text_with_collection_anchors(
        source_id(),
        &source,
        document_id(),
        RoleDictionary::current(),
    )
    .expect("valid source qualifies");
    assert_eq!(first, second);
}
