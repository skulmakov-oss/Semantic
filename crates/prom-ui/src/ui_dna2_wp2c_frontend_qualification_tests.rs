use crate::contract_primitives::{SourceId, SourceRef, StaticDocumentId};
use crate::projection_compile::{ProjectionCompileDiagnosticKind, ProjectionCompileStage};
use crate::projection_source::ProjectionSourceDiagnosticKind;
use crate::projection_source_frontend::{
    compile_projection_source_text_to_static_ir, ProjectionSourceFrontendError,
};
use crate::projection_source_parser::{
    ProjectionSourceParseError, ProjectionSourceParserDiagnosticKind,
};
use crate::role_dictionary::RoleDictionary;

const SOURCE_RAW: u64 = 41;
const DOCUMENT_RAW: u64 = 73;

fn source_id() -> SourceId {
    SourceId::new(SOURCE_RAW).expect("non-zero test source id")
}

fn document_id() -> StaticDocumentId {
    StaticDocumentId::new(DOCUMENT_RAW).expect("non-zero test document id")
}

fn compile(
    source: &str,
) -> Result<crate::static_ir::StaticUiDocument, ProjectionSourceFrontendError> {
    compile_projection_source_text_to_static_ir(
        source_id(),
        source,
        document_id(),
        RoleDictionary::current(),
    )
}

fn assert_source_ref(source_ref: SourceRef, source: &str, declaration: &str) {
    assert_eq!(source_ref.source(), source_id());
    let start = source.find(declaration).expect("declaration start");
    let end = start
        .checked_add(declaration.len())
        .expect("declaration end arithmetic");
    assert_eq!(
        source_ref.span().start(),
        u32::try_from(start).expect("representable declaration start")
    );
    assert_eq!(
        source_ref.span().end(),
        u32::try_from(end).expect("representable declaration end")
    );
}

fn valid_source() -> &'static str {
    concat!(
        "projection v0 {\n",
        "revision 7; epoch 9;\n",
        "surface 2 root 20 key 200;\n",
        "node 20 role root key 201 { child 22 order 4; child 21 order 1; }\n",
        "node 21 role text key 101 {}\n",
        "node 22 role numeric_readout key 102 {}\n",
        "}"
    )
}

#[test]
fn ui_dna2_wp2c_frontend_compiles_text_and_preserves_identity_and_provenance() {
    let source = valid_source();
    let document = compile(source).expect("valid source compiles");

    assert_eq!(document.id(), document_id());
    assert_eq!(document.revision().raw(), 7);
    assert_eq!(document.epoch().raw(), 9);
    assert_eq!(document.surfaces().len(), 1);
    assert_eq!(document.nodes().len(), 3);

    let surface = &document.surfaces()[0];
    assert_eq!(surface.id().raw(), 2);
    assert_eq!(surface.root().raw(), 20);
    assert_eq!(surface.key().raw(), 200);
    assert_source_ref(
        surface.source().expect("surface source provenance"),
        source,
        "surface 2 root 20 key 200;",
    );

    let root = document
        .nodes()
        .iter()
        .find(|node| node.id().raw() == 20)
        .expect("root node");
    assert_eq!(root.key().raw(), 201);
    assert_eq!(
        root.role(),
        RoleDictionary::current()
            .resolve_name("root")
            .expect("built-in root role")
    );
    assert_eq!(root.children().len(), 2);
    assert_eq!(root.children()[0].child().raw(), 21);
    assert_eq!(root.children()[0].order().raw(), 1);
    assert_eq!(root.children()[1].child().raw(), 22);
    assert_eq!(root.children()[1].order().raw(), 4);
    assert_source_ref(
        root.source().expect("node source provenance"),
        source,
        "node 20 role root key 201 { child 22 order 4; child 21 order 1; }",
    );

    let text = document
        .nodes()
        .iter()
        .find(|node| node.id().raw() == 21)
        .expect("text node");
    assert_eq!(
        text.role(),
        RoleDictionary::current()
            .resolve_name("text")
            .expect("built-in text role")
    );
    let numeric = document
        .nodes()
        .iter()
        .find(|node| node.id().raw() == 22)
        .expect("numeric readout node");
    assert_eq!(
        numeric.role(),
        RoleDictionary::current()
            .resolve_name("numeric_readout")
            .expect("built-in numeric_readout role")
    );
}

#[test]
fn ui_dna2_wp2c_frontend_repeated_compilation_is_canonical() {
    let first = compile(valid_source()).expect("first compilation");
    let second = compile(valid_source()).expect("second compilation");

    assert_eq!(first, second);
    assert_eq!(
        first
            .canonical_bytes(RoleDictionary::current())
            .expect("first canonical bytes"),
        second
            .canonical_bytes(RoleDictionary::current())
            .expect("second canonical bytes")
    );
}

#[test]
fn ui_dna2_wp2c_frontend_preserves_parser_error_identity() {
    let source = "projection v0 { revision 0; epoch 0; node 1 role Root key 1 {} }";

    match compile(source).expect_err("malformed identifier must fail parsing") {
        ProjectionSourceFrontendError::Parse(ProjectionSourceParseError::Diagnostic(
            diagnostic,
        )) => {
            assert_eq!(
                diagnostic.kind(),
                ProjectionSourceParserDiagnosticKind::InvalidIdentifier
            );
            assert_eq!(diagnostic.code().as_str(), "PSP_INVALID_IDENTIFIER");
        }
        other => panic!("expected parser diagnostic, got {other:?}"),
    }
}

#[test]
fn ui_dna2_wp2c_frontend_routes_unknown_role_through_source_compile_stage() {
    let source = concat!(
        "projection v0 { revision 0; epoch 0; ",
        "surface 1 root 1 key 1; node 1 role button key 2 {} }"
    );

    let ProjectionSourceFrontendError::Compile(diagnostics) =
        compile(source).expect_err("unknown role must fail semantic compilation")
    else {
        panic!("semantic failure must not masquerade as parser failure");
    };
    assert_eq!(diagnostics.diagnostics().len(), 1);
    let diagnostic = diagnostics.diagnostics()[0];
    assert_eq!(
        diagnostic.kind(),
        ProjectionCompileDiagnosticKind::Source(ProjectionSourceDiagnosticKind::UnknownRole)
    );
    assert_eq!(diagnostic.stage(), ProjectionCompileStage::Source);
    assert_eq!(diagnostic.coordinate().code().as_str(), "PS_UNKNOWN_ROLE");
}

#[test]
fn ui_dna2_wp2c_frontend_preserves_structural_source_diagnostic() {
    let source = concat!(
        "projection v0 { revision 0; epoch 0; ",
        "surface 1 root 1 key 1; ",
        "node 1 role root key 2 { child 9 order 0; } }"
    );

    let ProjectionSourceFrontendError::Compile(diagnostics) =
        compile(source).expect_err("missing child must fail semantic compilation")
    else {
        panic!("semantic failure must not masquerade as parser failure");
    };
    assert_eq!(diagnostics.diagnostics().len(), 1);
    let diagnostic = diagnostics.diagnostics()[0];
    assert_eq!(
        diagnostic.kind(),
        ProjectionCompileDiagnosticKind::Source(ProjectionSourceDiagnosticKind::MissingChild)
    );
    assert_eq!(diagnostic.stage(), ProjectionCompileStage::Source);
    assert_eq!(diagnostic.coordinate().code().as_str(), "PS_MISSING_CHILD");
}
