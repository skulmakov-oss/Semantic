//! Crate-private in-memory Projection Source front-end composition.
#![allow(dead_code)]

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
