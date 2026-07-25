//! Crate-private in-memory Projection Source front-end composition.
#![allow(dead_code)]

use crate::collection_anchor_declarations::{
    qualify_collection_anchor_declarations, CollectionAnchorDeclarationDiagnostics,
    QualifiedCollectionAnchorDeclarations,
};
use crate::contract_primitives::{SourceId, StaticDocumentId};
use crate::projection_compile::{
    compile_projection_source_to_static_ir, ProjectionCompileDiagnostics,
};
use crate::projection_source_parser::{parse_projection_source, ProjectionSourceParseError};
use crate::role_dictionary::RoleDictionary;
use crate::static_ir::StaticUiDocument;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProjectionSourceFrontendError {
    Parse(ProjectionSourceParseError),
    Compile(ProjectionCompileDiagnostics),
}

pub(crate) fn compile_projection_source_text_to_static_ir(
    source_id: SourceId,
    source_text: &str,
    document_id: StaticDocumentId,
    dictionary: RoleDictionary,
) -> Result<StaticUiDocument, ProjectionSourceFrontendError> {
    let source = parse_projection_source(source_id, source_text)
        .map_err(ProjectionSourceFrontendError::Parse)?;

    compile_projection_source_to_static_ir(document_id, &source, dictionary)
        .map_err(ProjectionSourceFrontendError::Compile)
}

/// Whole-pipeline result of compiling Projection Source text to both the
/// Static UI IR and its qualified explicit `CollectionAnchor` declarations
/// in one pass, sharing the single parsed `ProjectionSourceDocument`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CompiledProjectionSource {
    static_document: StaticUiDocument,
    collection_anchors: QualifiedCollectionAnchorDeclarations,
}

impl CompiledProjectionSource {
    pub(crate) const fn static_document(&self) -> &StaticUiDocument {
        &self.static_document
    }

    pub(crate) const fn collection_anchors(&self) -> &QualifiedCollectionAnchorDeclarations {
        &self.collection_anchors
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProjectionSourceFrontendWithAnchorsError {
    Parse(ProjectionSourceParseError),
    Compile(ProjectionCompileDiagnostics),
    CollectionAnchor(CollectionAnchorDeclarationDiagnostics),
}

/// Compiles Projection Source text through the same parser and Static UI IR
/// lowering as [`compile_projection_source_text_to_static_ir`], then
/// additionally qualifies the parsed document's explicit textual
/// `CollectionAnchor` declarations against the freshly compiled Static UI
/// IR. Both artifacts are derived from the same one parse; the caller
/// cannot supply a mismatched pair.
pub(crate) fn compile_projection_source_text_with_collection_anchors(
    source_id: SourceId,
    source_text: &str,
    document_id: StaticDocumentId,
    dictionary: RoleDictionary,
) -> Result<CompiledProjectionSource, ProjectionSourceFrontendWithAnchorsError> {
    let source = parse_projection_source(source_id, source_text)
        .map_err(ProjectionSourceFrontendWithAnchorsError::Parse)?;

    let static_document = compile_projection_source_to_static_ir(document_id, &source, dictionary)
        .map_err(ProjectionSourceFrontendWithAnchorsError::Compile)?;

    let collection_anchors =
        qualify_collection_anchor_declarations(document_id, &source, &static_document)
            .map_err(ProjectionSourceFrontendWithAnchorsError::CollectionAnchor)?;

    Ok(CompiledProjectionSource {
        static_document,
        collection_anchors,
    })
}
