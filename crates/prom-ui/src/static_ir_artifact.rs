//! Crate-private, in-memory Static UI IR Artifact V1 verification.
#![allow(dead_code)]

use alloc::string::{String, ToString};
use alloc::vec::Vec;

use crate::contract_primitives::{
    ChildOrder, CollectionKey, ContractVersion, Epoch, Revision, SchemaVersion, SourceId,
    SourceRef, SourceSpan, StaticDocumentId, StaticNodeId, StaticSurfaceId,
};
use crate::role_dictionary::RoleDictionary;
use crate::static_ir::{
    validate_static_ir_structure, CanonicalEncodingError, StaticUiChild, StaticUiDocument,
    StaticUiIrDiagnostics, StaticUiNode, StaticUiStructureNode, StaticUiSurface,
};

pub(crate) const STATIC_UI_IR_ARTIFACT_V1_MAGIC: &[u8] = b"UI_DNA2_STATIC_IR_V1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct StaticUiArtifactV1Limits {
    pub(crate) max_input_bytes: usize,
    pub(crate) max_surfaces: u32,
    pub(crate) max_nodes: u32,
    pub(crate) max_children_per_node: u32,
    pub(crate) max_role_bytes: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VerifiedStaticUiArtifactV1 {
    document: StaticUiDocument,
}

impl VerifiedStaticUiArtifactV1 {
    pub(crate) fn document(&self) -> &StaticUiDocument {
        &self.document
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum StaticUiArtifactV1Stage {
    Resource,
    Decode,
    Header,
    CompleteConsumption,
    Representation,
    StaticIr,
    Role,
    Canonical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum StaticUiArtifactV1Field {
    DocumentId,
    SurfaceId,
    RootNodeId,
    SourceId,
    NodeId,
    ChildNodeId,
    AccessibilityNodeId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum StaticUiArtifactV1ErrorKind {
    InputTooLarge,
    SurfaceLimitExceeded,
    NodeLimitExceeded,
    ChildLimitExceeded,
    RoleLengthLimitExceeded,
    TruncatedPrimitive,
    TruncatedByteString,
    ArithmeticOverflow,
    InvalidUtf8,
    InvalidOptionTag,
    InvalidMagic,
    UnsupportedSchemaVersion,
    UnsupportedContractVersion,
    TrailingBytes,
    ZeroIdentifier(StaticUiArtifactV1Field),
    ZeroCollectionKey,
    InvalidSourceSpan,
    StaticIr(StaticUiIrDiagnostics),
    UnknownRole {
        node_id: u64,
        authored_role: String,
        role_byte_offset: usize,
    },
    CanonicalMismatch,
    CanonicalEncodingFailure(CanonicalEncodingError),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct StaticUiArtifactV1Error {
    kind: StaticUiArtifactV1ErrorKind,
    byte_offset: Option<usize>,
}

impl StaticUiArtifactV1Error {
    pub(crate) fn stage(&self) -> StaticUiArtifactV1Stage {
        match self.kind {
            StaticUiArtifactV1ErrorKind::InputTooLarge
            | StaticUiArtifactV1ErrorKind::SurfaceLimitExceeded
            | StaticUiArtifactV1ErrorKind::NodeLimitExceeded
            | StaticUiArtifactV1ErrorKind::ChildLimitExceeded
            | StaticUiArtifactV1ErrorKind::RoleLengthLimitExceeded => {
                StaticUiArtifactV1Stage::Resource
            }
            StaticUiArtifactV1ErrorKind::TruncatedPrimitive
            | StaticUiArtifactV1ErrorKind::TruncatedByteString
            | StaticUiArtifactV1ErrorKind::ArithmeticOverflow
            | StaticUiArtifactV1ErrorKind::InvalidUtf8
            | StaticUiArtifactV1ErrorKind::InvalidOptionTag => StaticUiArtifactV1Stage::Decode,
            StaticUiArtifactV1ErrorKind::InvalidMagic
            | StaticUiArtifactV1ErrorKind::UnsupportedSchemaVersion
            | StaticUiArtifactV1ErrorKind::UnsupportedContractVersion => {
                StaticUiArtifactV1Stage::Header
            }
            StaticUiArtifactV1ErrorKind::TrailingBytes => {
                StaticUiArtifactV1Stage::CompleteConsumption
            }
            StaticUiArtifactV1ErrorKind::ZeroIdentifier(_)
            | StaticUiArtifactV1ErrorKind::ZeroCollectionKey
            | StaticUiArtifactV1ErrorKind::InvalidSourceSpan => {
                StaticUiArtifactV1Stage::Representation
            }
            StaticUiArtifactV1ErrorKind::StaticIr(_) => StaticUiArtifactV1Stage::StaticIr,
            StaticUiArtifactV1ErrorKind::UnknownRole { .. } => StaticUiArtifactV1Stage::Role,
            StaticUiArtifactV1ErrorKind::CanonicalMismatch
            | StaticUiArtifactV1ErrorKind::CanonicalEncodingFailure(_) => {
                StaticUiArtifactV1Stage::Canonical
            }
        }
    }

    pub(crate) const fn byte_offset(&self) -> Option<usize> {
        self.byte_offset
    }

    pub(crate) fn kind(&self) -> &StaticUiArtifactV1ErrorKind {
        &self.kind
    }

    const fn at(kind: StaticUiArtifactV1ErrorKind, byte_offset: usize) -> Self {
        Self {
            kind,
            byte_offset: Some(byte_offset),
        }
    }

    const fn without_offset(kind: StaticUiArtifactV1ErrorKind) -> Self {
        Self {
            kind,
            byte_offset: None,
        }
    }
}

pub(crate) fn verify_static_ui_ir_artifact_v1(
    bytes: &[u8],
    dictionary: RoleDictionary,
    limits: StaticUiArtifactV1Limits,
) -> Result<VerifiedStaticUiArtifactV1, StaticUiArtifactV1Error> {
    if bytes.len() > limits.max_input_bytes {
        return Err(StaticUiArtifactV1Error::without_offset(
            StaticUiArtifactV1ErrorKind::InputTooLarge,
        ));
    }

    let raw = decode(bytes, limits)?;
    let candidate = raw.into_candidate()?;
    let structural = validate_static_ir_structure(
        candidate.document_id,
        &candidate.surfaces,
        &candidate
            .nodes
            .iter()
            .map(|node| node.structure.clone())
            .collect::<Vec<_>>(),
    );
    if !structural.is_empty() {
        return Err(StaticUiArtifactV1Error::without_offset(
            StaticUiArtifactV1ErrorKind::StaticIr(structural),
        ));
    }

    let document = candidate.resolve_roles(dictionary)?;
    let canonical = document.canonical_bytes(dictionary).map_err(|error| {
        StaticUiArtifactV1Error::without_offset(
            StaticUiArtifactV1ErrorKind::CanonicalEncodingFailure(error),
        )
    })?;
    if canonical.as_slice() != bytes {
        let differing = canonical
            .iter()
            .zip(bytes)
            .position(|(left, right)| left != right);
        let offset = match differing {
            Some(offset) => offset,
            None => core::cmp::min(canonical.len(), bytes.len()),
        };
        return Err(StaticUiArtifactV1Error::at(
            StaticUiArtifactV1ErrorKind::CanonicalMismatch,
            offset,
        ));
    }

    Ok(VerifiedStaticUiArtifactV1 { document })
}

#[derive(Debug)]
struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, offset: 0 }
    }

    fn read_u8(&mut self) -> Result<(u8, usize), StaticUiArtifactV1Error> {
        let offset = self.offset;
        let slice = self.read_exact(1, StaticUiArtifactV1ErrorKind::TruncatedPrimitive)?;
        let array: [u8; 1] = slice.try_into().map_err(|_| {
            StaticUiArtifactV1Error::at(StaticUiArtifactV1ErrorKind::TruncatedPrimitive, offset)
        })?;
        let [value] = array;
        Ok((value, offset))
    }

    fn read_u32(&mut self) -> Result<(u32, usize), StaticUiArtifactV1Error> {
        let offset = self.offset;
        let slice = self.read_exact(4, StaticUiArtifactV1ErrorKind::TruncatedPrimitive)?;
        let array: [u8; 4] = slice.try_into().map_err(|_| {
            StaticUiArtifactV1Error::at(StaticUiArtifactV1ErrorKind::TruncatedPrimitive, offset)
        })?;
        Ok((u32::from_le_bytes(array), offset))
    }

    fn read_u64(&mut self) -> Result<(u64, usize), StaticUiArtifactV1Error> {
        let offset = self.offset;
        let slice = self.read_exact(8, StaticUiArtifactV1ErrorKind::TruncatedPrimitive)?;
        let array: [u8; 8] = slice.try_into().map_err(|_| {
            StaticUiArtifactV1Error::at(StaticUiArtifactV1ErrorKind::TruncatedPrimitive, offset)
        })?;
        Ok((u64::from_le_bytes(array), offset))
    }

    fn read_byte_string(&mut self) -> Result<(&'a [u8], usize), StaticUiArtifactV1Error> {
        let (length, _) = self.read_u32()?;
        let start = self.offset;
        let length = usize::try_from(length).map_err(|_| {
            StaticUiArtifactV1Error::at(StaticUiArtifactV1ErrorKind::ArithmeticOverflow, start)
        })?;
        let bytes = self.read_exact(length, StaticUiArtifactV1ErrorKind::TruncatedByteString)?;
        Ok((bytes, start))
    }

    fn read_exact(
        &mut self,
        length: usize,
        truncated: StaticUiArtifactV1ErrorKind,
    ) -> Result<&'a [u8], StaticUiArtifactV1Error> {
        let start = self.offset;
        let end = start.checked_add(length).ok_or_else(|| {
            StaticUiArtifactV1Error::at(StaticUiArtifactV1ErrorKind::ArithmeticOverflow, start)
        })?;
        let slice = self
            .bytes
            .get(start..end)
            .ok_or_else(|| StaticUiArtifactV1Error::at(truncated, start))?;
        self.offset = end;
        Ok(slice)
    }
}

#[cfg(test)]
pub(crate) fn qualify_checked_cursor_overflow() -> StaticUiArtifactV1Error {
    let mut cursor = Cursor {
        bytes: &[0],
        offset: 1,
    };
    match cursor.read_exact(usize::MAX, StaticUiArtifactV1ErrorKind::TruncatedByteString) {
        Ok(_) => StaticUiArtifactV1Error::without_offset(
            StaticUiArtifactV1ErrorKind::TruncatedByteString,
        ),
        Err(error) => error,
    }
}

#[derive(Debug)]
struct RawDocument {
    document_id: (u64, usize),
    revision: u64,
    epoch: u64,
    surfaces: Vec<RawSurface>,
    nodes: Vec<RawNode>,
}

#[derive(Debug)]
struct RawSurface {
    id: (u64, usize),
    root: (u64, usize),
    key: (u64, usize),
    source: Option<RawSourceRef>,
}

#[derive(Debug)]
struct RawNode {
    id: (u64, usize),
    role: String,
    role_offset: usize,
    key: (u64, usize),
    source: Option<RawSourceRef>,
    children: Vec<RawChild>,
    accessibility_ref: Option<(u64, usize)>,
}

#[derive(Debug)]
struct RawChild {
    order: u32,
    child: (u64, usize),
}

#[derive(Debug)]
struct RawSourceRef {
    source: (u64, usize),
    start: (u32, usize),
    end: (u32, usize),
}

fn decode(
    bytes: &[u8],
    limits: StaticUiArtifactV1Limits,
) -> Result<RawDocument, StaticUiArtifactV1Error> {
    let mut cursor = Cursor::new(bytes);
    let (magic, magic_offset) = cursor.read_byte_string()?;
    let (schema, schema_offset) = cursor.read_u32()?;
    let (contract, contract_offset) = cursor.read_u32()?;
    let document_id = cursor.read_u64()?;
    let (revision, _) = cursor.read_u64()?;
    let (epoch, _) = cursor.read_u64()?;
    let (surface_count, surface_count_offset) = cursor.read_u32()?;
    if surface_count > limits.max_surfaces {
        return Err(StaticUiArtifactV1Error::at(
            StaticUiArtifactV1ErrorKind::SurfaceLimitExceeded,
            surface_count_offset,
        ));
    }
    let mut surfaces = Vec::new();
    for _ in 0..surface_count {
        surfaces.push(RawSurface {
            id: cursor.read_u64()?,
            root: cursor.read_u64()?,
            key: cursor.read_u64()?,
            source: decode_optional_source(&mut cursor)?,
        });
    }

    let (node_count, node_count_offset) = cursor.read_u32()?;
    if node_count > limits.max_nodes {
        return Err(StaticUiArtifactV1Error::at(
            StaticUiArtifactV1ErrorKind::NodeLimitExceeded,
            node_count_offset,
        ));
    }
    let mut nodes = Vec::new();
    for _ in 0..node_count {
        let id = cursor.read_u64()?;
        let (role_length, role_length_offset) = cursor.read_u32()?;
        if role_length > limits.max_role_bytes {
            return Err(StaticUiArtifactV1Error::at(
                StaticUiArtifactV1ErrorKind::RoleLengthLimitExceeded,
                role_length_offset,
            ));
        }
        let role_offset = cursor.offset;
        let role_bytes = cursor.read_exact(
            usize::try_from(role_length).map_err(|_| {
                StaticUiArtifactV1Error::at(
                    StaticUiArtifactV1ErrorKind::ArithmeticOverflow,
                    role_length_offset,
                )
            })?,
            StaticUiArtifactV1ErrorKind::TruncatedByteString,
        )?;
        let role = match core::str::from_utf8(role_bytes) {
            Ok(role) => role,
            Err(error) => {
                let offset = role_offset
                    .checked_add(error.valid_up_to())
                    .ok_or_else(|| {
                        StaticUiArtifactV1Error::at(
                            StaticUiArtifactV1ErrorKind::ArithmeticOverflow,
                            role_offset,
                        )
                    })?;
                return Err(StaticUiArtifactV1Error::at(
                    StaticUiArtifactV1ErrorKind::InvalidUtf8,
                    offset,
                ));
            }
        };
        let key = cursor.read_u64()?;
        let source = decode_optional_source(&mut cursor)?;
        let (child_count, child_count_offset) = cursor.read_u32()?;
        if child_count > limits.max_children_per_node {
            return Err(StaticUiArtifactV1Error::at(
                StaticUiArtifactV1ErrorKind::ChildLimitExceeded,
                child_count_offset,
            ));
        }
        let mut children = Vec::new();
        for _ in 0..child_count {
            let (order, _) = cursor.read_u32()?;
            children.push(RawChild {
                order,
                child: cursor.read_u64()?,
            });
        }
        let accessibility_ref = decode_optional_u64(&mut cursor)?;
        nodes.push(RawNode {
            id,
            role: role.to_string(),
            role_offset,
            key,
            source,
            children,
            accessibility_ref,
        });
    }

    if magic != STATIC_UI_IR_ARTIFACT_V1_MAGIC {
        return Err(StaticUiArtifactV1Error::at(
            StaticUiArtifactV1ErrorKind::InvalidMagic,
            magic_offset,
        ));
    }
    if schema != SchemaVersion::CURRENT.raw() {
        return Err(StaticUiArtifactV1Error::at(
            StaticUiArtifactV1ErrorKind::UnsupportedSchemaVersion,
            schema_offset,
        ));
    }
    if contract != ContractVersion::CURRENT.raw() {
        return Err(StaticUiArtifactV1Error::at(
            StaticUiArtifactV1ErrorKind::UnsupportedContractVersion,
            contract_offset,
        ));
    }
    if cursor.offset != bytes.len() {
        return Err(StaticUiArtifactV1Error::at(
            StaticUiArtifactV1ErrorKind::TrailingBytes,
            cursor.offset,
        ));
    }

    Ok(RawDocument {
        document_id,
        revision,
        epoch,
        surfaces,
        nodes,
    })
}

fn decode_optional_source(
    cursor: &mut Cursor<'_>,
) -> Result<Option<RawSourceRef>, StaticUiArtifactV1Error> {
    let (tag, tag_offset) = cursor.read_u8()?;
    match tag {
        0 => Ok(None),
        1 => Ok(Some(RawSourceRef {
            source: cursor.read_u64()?,
            start: cursor.read_u32()?,
            end: cursor.read_u32()?,
        })),
        _ => Err(StaticUiArtifactV1Error::at(
            StaticUiArtifactV1ErrorKind::InvalidOptionTag,
            tag_offset,
        )),
    }
}

fn decode_optional_u64(
    cursor: &mut Cursor<'_>,
) -> Result<Option<(u64, usize)>, StaticUiArtifactV1Error> {
    let (tag, tag_offset) = cursor.read_u8()?;
    match tag {
        0 => Ok(None),
        1 => cursor.read_u64().map(Some),
        _ => Err(StaticUiArtifactV1Error::at(
            StaticUiArtifactV1ErrorKind::InvalidOptionTag,
            tag_offset,
        )),
    }
}

struct CandidateDocument {
    document_id: StaticDocumentId,
    revision: Revision,
    epoch: Epoch,
    surfaces: Vec<StaticUiSurface>,
    nodes: Vec<CandidateNode>,
}

struct CandidateNode {
    structure: StaticUiStructureNode,
    role: String,
    role_offset: usize,
}

impl RawDocument {
    fn into_candidate(self) -> Result<CandidateDocument, StaticUiArtifactV1Error> {
        let document_id = nonzero_document_id(self.document_id)?;
        let mut surfaces = Vec::new();
        for surface in self.surfaces {
            surfaces.push(StaticUiSurface::new(
                nonzero_surface_id(surface.id)?,
                nonzero_node_id(surface.root, StaticUiArtifactV1Field::RootNodeId)?,
                nonzero_key(surface.key)?,
                source_ref(surface.source)?,
            ));
        }
        let mut nodes = Vec::new();
        for node in self.nodes {
            let id = nonzero_node_id(node.id, StaticUiArtifactV1Field::NodeId)?;
            let key = nonzero_key(node.key)?;
            let source = source_ref(node.source)?;
            let mut children = Vec::new();
            for child in node.children {
                children.push(StaticUiChild::new(
                    nonzero_node_id(child.child, StaticUiArtifactV1Field::ChildNodeId)?,
                    ChildOrder::new(child.order),
                ));
            }
            let accessibility_ref = node
                .accessibility_ref
                .map(|raw| nonzero_node_id(raw, StaticUiArtifactV1Field::AccessibilityNodeId))
                .transpose()?;
            nodes.push(CandidateNode {
                structure: StaticUiStructureNode::new(id, key, source, children, accessibility_ref),
                role: node.role,
                role_offset: node.role_offset,
            });
        }
        Ok(CandidateDocument {
            document_id,
            revision: Revision::new(self.revision),
            epoch: Epoch::new(self.epoch),
            surfaces,
            nodes,
        })
    }
}

impl CandidateDocument {
    fn resolve_roles(
        self,
        dictionary: RoleDictionary,
    ) -> Result<StaticUiDocument, StaticUiArtifactV1Error> {
        let mut document = StaticUiDocument::new(self.document_id, self.revision, self.epoch);
        for surface in self.surfaces {
            document.push_surface(surface);
        }
        for candidate in self.nodes {
            let id = candidate.structure.id();
            let role = dictionary.resolve_name(&candidate.role).ok_or_else(|| {
                StaticUiArtifactV1Error::at(
                    StaticUiArtifactV1ErrorKind::UnknownRole {
                        node_id: id.raw(),
                        authored_role: candidate.role.clone(),
                        role_byte_offset: candidate.role_offset,
                    },
                    candidate.role_offset,
                )
            })?;
            let (id, key, source, children, accessibility_ref) = candidate.structure.into_parts();
            let mut node = StaticUiNode::new(id, role, key, source);
            for child in children {
                node.push_child(child);
            }
            if let Some(target) = accessibility_ref {
                node.set_accessibility_ref(target);
            }
            document.push_node(node);
        }
        Ok(document)
    }
}

fn nonzero_document_id(raw: (u64, usize)) -> Result<StaticDocumentId, StaticUiArtifactV1Error> {
    StaticDocumentId::new(raw.0).map_err(|_| {
        StaticUiArtifactV1Error::at(
            StaticUiArtifactV1ErrorKind::ZeroIdentifier(StaticUiArtifactV1Field::DocumentId),
            raw.1,
        )
    })
}

fn nonzero_surface_id(raw: (u64, usize)) -> Result<StaticSurfaceId, StaticUiArtifactV1Error> {
    StaticSurfaceId::new(raw.0).map_err(|_| {
        StaticUiArtifactV1Error::at(
            StaticUiArtifactV1ErrorKind::ZeroIdentifier(StaticUiArtifactV1Field::SurfaceId),
            raw.1,
        )
    })
}

fn nonzero_node_id(
    raw: (u64, usize),
    field: StaticUiArtifactV1Field,
) -> Result<StaticNodeId, StaticUiArtifactV1Error> {
    StaticNodeId::new(raw.0).map_err(|_| {
        StaticUiArtifactV1Error::at(StaticUiArtifactV1ErrorKind::ZeroIdentifier(field), raw.1)
    })
}

fn nonzero_key(raw: (u64, usize)) -> Result<CollectionKey, StaticUiArtifactV1Error> {
    CollectionKey::new(raw.0).map_err(|_| {
        StaticUiArtifactV1Error::at(StaticUiArtifactV1ErrorKind::ZeroCollectionKey, raw.1)
    })
}

fn source_ref(raw: Option<RawSourceRef>) -> Result<Option<SourceRef>, StaticUiArtifactV1Error> {
    let Some(raw) = raw else {
        return Ok(None);
    };
    let source = SourceId::new(raw.source.0).map_err(|_| {
        StaticUiArtifactV1Error::at(
            StaticUiArtifactV1ErrorKind::ZeroIdentifier(StaticUiArtifactV1Field::SourceId),
            raw.source.1,
        )
    })?;
    let span = SourceSpan::new(raw.start.0, raw.end.0).map_err(|_| {
        StaticUiArtifactV1Error::at(StaticUiArtifactV1ErrorKind::InvalidSourceSpan, raw.start.1)
    })?;
    Ok(Some(SourceRef::new(source, span)))
}
