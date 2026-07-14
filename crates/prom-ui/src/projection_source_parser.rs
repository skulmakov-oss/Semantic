//! Crate-private Grammar v0 Projection Source parser.
#![allow(dead_code)]

use core::convert::TryFrom;

use alloc::vec::Vec;

use crate::contract_primitives::{
    ChildOrder, CollectionKey, DiagnosticCode, DiagnosticCoordinate, Epoch, ProjectionSourceNodeId,
    ProjectionSourceSurfaceId, Revision, SourceId, SourceRef, SourceSpan,
};
use crate::projection_source::{
    ProjectionSourceChild, ProjectionSourceDocument, ProjectionSourceNode,
    ProjectionSourceRoleName, ProjectionSourceSurface,
};

pub(crate) const MAX_PROJECTION_SOURCE_BYTES: u32 = u32::MAX;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProjectionSourceInputError {
    SourceTooLarge {
        source_id: SourceId,
        actual_len: usize,
        max_len: u32,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ProjectionSourceParseError {
    Input(ProjectionSourceInputError),
    Diagnostic(ProjectionSourceParserDiagnostic),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum ProjectionSourceParserDiagnosticKind {
    UnexpectedChar,
    UnexpectedToken,
    UnexpectedEof,
    MissingSemicolon,
    InvalidIdentifier,
    InvalidNumber,
    NumberOverflow,
    ZeroId,
    DuplicateRevision,
    DuplicateEpoch,
    UnsupportedVersion,
    UnknownClause,
}

impl ProjectionSourceParserDiagnosticKind {
    pub(crate) const fn code(self) -> DiagnosticCode {
        let code = match self {
            Self::UnexpectedChar => "PSP_UNEXPECTED_CHAR",
            Self::UnexpectedToken => "PSP_UNEXPECTED_TOKEN",
            Self::UnexpectedEof => "PSP_UNEXPECTED_EOF",
            Self::MissingSemicolon => "PSP_MISSING_SEMICOLON",
            Self::InvalidIdentifier => "PSP_INVALID_IDENTIFIER",
            Self::InvalidNumber => "PSP_INVALID_NUMBER",
            Self::NumberOverflow => "PSP_NUMBER_OVERFLOW",
            Self::ZeroId => "PSP_ZERO_ID",
            Self::DuplicateRevision => "PSP_DUP_REVISION",
            Self::DuplicateEpoch => "PSP_DUP_EPOCH",
            Self::UnsupportedVersion => "PSP_UNSUPPORTED_VERSION",
            Self::UnknownClause => "PSP_UNKNOWN_CLAUSE",
        };
        DiagnosticCode::new(code)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) struct ProjectionSourceParserDiagnostic {
    kind: ProjectionSourceParserDiagnosticKind,
    coordinate: DiagnosticCoordinate,
    source_id: SourceId,
    span: SourceSpan,
}

impl ProjectionSourceParserDiagnostic {
    pub(crate) const fn kind(self) -> ProjectionSourceParserDiagnosticKind {
        self.kind
    }

    pub(crate) const fn code(self) -> DiagnosticCode {
        self.kind.code()
    }

    pub(crate) const fn coordinate(self) -> DiagnosticCoordinate {
        self.coordinate
    }

    pub(crate) const fn source_id(self) -> SourceId {
        self.source_id
    }

    pub(crate) const fn span(self) -> SourceSpan {
        self.span
    }
}

pub(crate) fn parse_projection_source(
    source_id: SourceId,
    source_text: &str,
) -> Result<ProjectionSourceDocument, ProjectionSourceParseError> {
    preflight_source_len(source_id, source_text.len())
        .map_err(ProjectionSourceParseError::Input)?;
    Parser::new(source_id, source_text).parse()
}

pub(crate) fn preflight_source_len(
    source_id: SourceId,
    actual_len: usize,
) -> Result<(), ProjectionSourceInputError> {
    if actual_len > MAX_PROJECTION_SOURCE_BYTES as usize {
        Err(ProjectionSourceInputError::SourceTooLarge {
            source_id,
            actual_len,
            max_len: MAX_PROJECTION_SOURCE_BYTES,
        })
    } else {
        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
struct Candidate<'a> {
    text: &'a str,
    start: usize,
    end: usize,
}

struct Parser<'a> {
    source_id: SourceId,
    text: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn new(source_id: SourceId, text: &'a str) -> Self {
        Self {
            source_id,
            text,
            bytes: text.as_bytes(),
            pos: 0,
        }
    }

    fn parse(mut self) -> Result<ProjectionSourceDocument, ProjectionSourceParseError> {
        self.expect_keyword("projection")?;
        self.parse_version()?;
        self.expect_byte(b'{')?;

        self.expect_keyword("revision")?;
        let revision = self.parse_number(u64::MAX, false)?;
        self.expect_semicolon()?;

        self.skip_trivia()?;
        if self.peek_candidate_text() == Some("revision") {
            let token = self
                .scan_identifier_candidate()
                .expect("candidate was peeked");
            return Err(self.error(
                ProjectionSourceParserDiagnosticKind::DuplicateRevision,
                token.start,
                token.end,
            ));
        }
        self.expect_keyword("epoch")?;
        let epoch = self.parse_number(u64::MAX, false)?;
        self.expect_semicolon()?;

        let mut document = ProjectionSourceDocument::new(
            self.source_id,
            Revision::new(revision),
            Epoch::new(epoch),
        );

        loop {
            self.skip_trivia()?;
            if self.at_end() {
                return Err(self.eof_error());
            }
            if self.peek_byte() == Some(b'}') {
                self.pos += 1;
                break;
            }

            let token = match self.scan_identifier_candidate() {
                Some(token) => token,
                None => return Err(self.current_unexpected()),
            };
            match token.text {
                "surface" => document.push_surface(self.parse_surface(token.start)?),
                "node" => document.push_node(self.parse_node(token.start)?),
                "revision" => {
                    return Err(self.error(
                        ProjectionSourceParserDiagnosticKind::DuplicateRevision,
                        token.start,
                        token.end,
                    ));
                }
                "epoch" => {
                    return Err(self.error(
                        ProjectionSourceParserDiagnosticKind::DuplicateEpoch,
                        token.start,
                        token.end,
                    ));
                }
                _ => return Err(self.clause_error(token)),
            }
        }

        self.skip_trivia()?;
        if self.at_end() {
            Ok(document)
        } else {
            Err(self.current_unexpected())
        }
    }

    fn parse_version(&mut self) -> Result<(), ProjectionSourceParseError> {
        self.skip_trivia()?;
        if self.at_end() {
            return Err(self.eof_error());
        }
        let token = match self.scan_identifier_candidate() {
            Some(token) => token,
            None => return Err(self.current_unexpected()),
        };
        if token.text == "v0" {
            Ok(())
        } else {
            Err(self.error(
                ProjectionSourceParserDiagnosticKind::UnsupportedVersion,
                token.start,
                token.end,
            ))
        }
    }

    fn parse_surface(
        &mut self,
        start: usize,
    ) -> Result<ProjectionSourceSurface, ProjectionSourceParseError> {
        let id = self.parse_number(u64::MAX, true)?;
        self.expect_keyword("root")?;
        let root = self.parse_number(u64::MAX, true)?;
        self.expect_keyword("key")?;
        let key = self.parse_number(u64::MAX, true)?;
        let end = self.expect_semicolon()?;
        Ok(ProjectionSourceSurface::new(
            ProjectionSourceSurfaceId::new(id).expect("non-zero surface id"),
            ProjectionSourceNodeId::new(root).expect("non-zero root id"),
            CollectionKey::new(key).expect("non-zero surface key"),
            self.source_ref(start, end),
        ))
    }

    fn parse_node(
        &mut self,
        start: usize,
    ) -> Result<ProjectionSourceNode, ProjectionSourceParseError> {
        let id = self.parse_number(u64::MAX, true)?;
        self.expect_keyword("role")?;
        let role = self.parse_role_identifier()?;
        self.expect_keyword("key")?;
        let key = self.parse_number(u64::MAX, true)?;
        self.expect_byte(b'{')?;

        let mut children = Vec::new();

        loop {
            self.skip_trivia()?;
            if self.at_end() {
                return Err(self.eof_error());
            }
            if self.peek_byte() == Some(b'}') {
                self.pos += 1;
                break;
            }

            let token = match self.scan_identifier_candidate() {
                Some(token) => token,
                None => return Err(self.current_unexpected()),
            };
            if token.text != "child" {
                return Err(self.clause_error(token));
            }
            children.push(self.parse_child()?);
        }

        let mut node = ProjectionSourceNode::new(
            ProjectionSourceNodeId::new(id).expect("non-zero node id"),
            role,
            CollectionKey::new(key).expect("non-zero node key"),
            self.source_ref(start, self.pos),
        );
        for child in children {
            node.push_child(child);
        }
        Ok(node)
    }

    fn parse_child(&mut self) -> Result<ProjectionSourceChild, ProjectionSourceParseError> {
        let child = self.parse_number(u64::MAX, true)?;
        self.expect_keyword("order")?;
        let order = self.parse_number(u32::MAX as u64, false)?;
        self.expect_semicolon()?;
        Ok(ProjectionSourceChild::new(
            ProjectionSourceNodeId::new(child).expect("non-zero child id"),
            ChildOrder::new(u32::try_from(order).expect("bounded child order")),
        ))
    }

    fn parse_role_identifier(
        &mut self,
    ) -> Result<ProjectionSourceRoleName, ProjectionSourceParseError> {
        self.skip_trivia()?;
        if self.at_end() {
            return Err(self.eof_error());
        }
        if self.peek_byte() == Some(b'"') {
            return Err(self.scan_quoted_role_error());
        }
        let token = match self.scan_identifier_candidate() {
            Some(token) => token,
            None => return Err(self.current_unexpected()),
        };
        if is_valid_identifier(token.text) {
            Ok(ProjectionSourceRoleName::new(token.text))
        } else {
            Err(self.error(
                ProjectionSourceParserDiagnosticKind::InvalidIdentifier,
                token.start,
                token.end,
            ))
        }
    }

    fn parse_number(
        &mut self,
        max: u64,
        non_zero: bool,
    ) -> Result<u64, ProjectionSourceParseError> {
        self.skip_trivia()?;
        if self.at_end() {
            return Err(self.eof_error());
        }

        let start = self.pos;
        if matches!(self.peek_byte(), Some(b'+' | b'-')) {
            let sign = self.peek_byte().expect("sign was matched");
            if self
                .bytes
                .get(self.pos + 1)
                .is_some_and(|byte| byte.is_ascii_digit())
            {
                self.pos += 1;
                self.scan_number_like();
                return Err(self.error(
                    ProjectionSourceParserDiagnosticKind::InvalidNumber,
                    start,
                    self.pos,
                ));
            }
            return Err(if sign == b'+' {
                self.error(
                    ProjectionSourceParserDiagnosticKind::UnexpectedToken,
                    start,
                    start + 1,
                )
            } else {
                self.error(
                    ProjectionSourceParserDiagnosticKind::UnexpectedChar,
                    start,
                    start + 1,
                )
            });
        }

        if !self.peek_byte().is_some_and(|byte| byte.is_ascii_digit()) {
            return Err(self.current_unexpected());
        }
        self.scan_number_like();
        let end = self.pos;
        let text = &self.text[start..end];
        if !text.bytes().all(|byte| byte.is_ascii_digit())
            || (text.len() > 1 && text.starts_with('0'))
        {
            return Err(self.error(
                ProjectionSourceParserDiagnosticKind::InvalidNumber,
                start,
                end,
            ));
        }
        let value = text.parse::<u64>().map_err(|_| {
            self.error(
                ProjectionSourceParserDiagnosticKind::NumberOverflow,
                start,
                end,
            )
        })?;
        if value > max {
            return Err(self.error(
                ProjectionSourceParserDiagnosticKind::NumberOverflow,
                start,
                end,
            ));
        }
        if non_zero && value == 0 {
            return Err(self.error(ProjectionSourceParserDiagnosticKind::ZeroId, start, end));
        }
        Ok(value)
    }

    fn expect_keyword(
        &mut self,
        expected: &str,
    ) -> Result<Candidate<'a>, ProjectionSourceParseError> {
        self.skip_trivia()?;
        if self.at_end() {
            return Err(self.eof_error());
        }
        let token = match self.scan_identifier_candidate() {
            Some(token) => token,
            None => return Err(self.current_unexpected()),
        };
        if token.text == expected {
            Ok(token)
        } else {
            Err(self.error(
                ProjectionSourceParserDiagnosticKind::UnexpectedToken,
                token.start,
                token.end,
            ))
        }
    }

    fn expect_byte(&mut self, expected: u8) -> Result<usize, ProjectionSourceParseError> {
        self.skip_trivia()?;
        if self.at_end() {
            return Err(self.eof_error());
        }
        if self.peek_byte() == Some(expected) {
            self.pos += 1;
            Ok(self.pos)
        } else {
            Err(self.current_unexpected())
        }
    }

    fn expect_semicolon(&mut self) -> Result<usize, ProjectionSourceParseError> {
        self.skip_trivia()?;
        if self.at_end() {
            return Err(self.eof_error());
        }
        if self.peek_byte() == Some(b';') {
            self.pos += 1;
            Ok(self.pos)
        } else {
            Err(self.error(
                ProjectionSourceParserDiagnosticKind::MissingSemicolon,
                self.pos,
                self.pos,
            ))
        }
    }

    fn skip_trivia(&mut self) -> Result<(), ProjectionSourceParseError> {
        loop {
            match self.peek_byte() {
                Some(b' ' | b'\t' | b'\n') => self.pos += 1,
                Some(b'\r') if self.bytes.get(self.pos + 1) == Some(&b'\n') => self.pos += 2,
                Some(b'/') if self.bytes.get(self.pos + 1) == Some(&b'/') => {
                    self.pos += 2;
                    while !self.at_end() {
                        match self.peek_byte() {
                            Some(b'\n') => {
                                self.pos += 1;
                                break;
                            }
                            Some(b'\r') if self.bytes.get(self.pos + 1) == Some(&b'\n') => {
                                self.pos += 2;
                                break;
                            }
                            Some(b'\r') => {
                                return Err(self.error(
                                    ProjectionSourceParserDiagnosticKind::UnexpectedChar,
                                    self.pos,
                                    self.pos + 1,
                                ));
                            }
                            Some(_) => self.pos += self.scalar_width(),
                            None => break,
                        }
                    }
                }
                _ => return Ok(()),
            }
        }
    }

    fn scan_identifier_candidate(&mut self) -> Option<Candidate<'a>> {
        let first = self.peek_byte()?;
        if !(first.is_ascii_alphanumeric() || first == b'_') {
            return None;
        }
        let start = self.pos;
        self.pos += 1;
        while let Some(byte) = self.peek_byte() {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b':' | b'-') {
                self.pos += 1;
            } else {
                break;
            }
        }
        Some(Candidate {
            text: &self.text[start..self.pos],
            start,
            end: self.pos,
        })
    }

    fn scan_number_like(&mut self) {
        while let Some(byte) = self.peek_byte() {
            if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'.' | b':') {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn scan_quoted_role_error(&mut self) -> ProjectionSourceParseError {
        let start = self.pos;
        self.pos += 1;
        while !self.at_end() {
            match self.peek_byte() {
                Some(b'"') => {
                    self.pos += 1;
                    return self.error(
                        ProjectionSourceParserDiagnosticKind::InvalidIdentifier,
                        start,
                        self.pos,
                    );
                }
                Some(b'\n' | b'\r') => {
                    return self.error(
                        ProjectionSourceParserDiagnosticKind::InvalidIdentifier,
                        start,
                        self.pos,
                    );
                }
                Some(byte) if byte.is_ascii() => self.pos += 1,
                Some(_) => {
                    let scalar_start = self.pos;
                    self.pos += self.scalar_width();
                    return self.error(
                        ProjectionSourceParserDiagnosticKind::UnexpectedChar,
                        scalar_start,
                        self.pos,
                    );
                }
                None => break,
            }
        }
        self.error(
            ProjectionSourceParserDiagnosticKind::InvalidIdentifier,
            start,
            self.pos,
        )
    }

    fn clause_error(&self, token: Candidate<'a>) -> ProjectionSourceParseError {
        let kind = if is_keyword(token.text) || token.text.bytes().all(|byte| byte.is_ascii_digit())
        {
            ProjectionSourceParserDiagnosticKind::UnexpectedToken
        } else if is_valid_identifier(token.text) {
            ProjectionSourceParserDiagnosticKind::UnknownClause
        } else {
            ProjectionSourceParserDiagnosticKind::InvalidIdentifier
        };
        self.error(kind, token.start, token.end)
    }

    fn current_unexpected(&mut self) -> ProjectionSourceParseError {
        if self.at_end() {
            return self.eof_error();
        }
        let start = self.pos;
        if let Some(token) = self.scan_identifier_candidate() {
            return self.error(
                ProjectionSourceParserDiagnosticKind::UnexpectedToken,
                token.start,
                token.end,
            );
        }
        let byte = self.peek_byte().expect("not at EOF");
        if byte == b'+' {
            self.pos += 1;
            return self.error(
                ProjectionSourceParserDiagnosticKind::UnexpectedToken,
                start,
                self.pos,
            );
        }
        if matches!(byte, b'{' | b'}' | b';') {
            self.pos += 1;
            return self.error(
                ProjectionSourceParserDiagnosticKind::UnexpectedToken,
                start,
                self.pos,
            );
        }
        self.pos += self.scalar_width();
        self.error(
            ProjectionSourceParserDiagnosticKind::UnexpectedChar,
            start,
            self.pos,
        )
    }

    fn peek_candidate_text(&mut self) -> Option<&'a str> {
        let saved = self.pos;
        let candidate = self.scan_identifier_candidate().map(|token| token.text);
        self.pos = saved;
        candidate
    }

    fn error(
        &self,
        kind: ProjectionSourceParserDiagnosticKind,
        start: usize,
        end: usize,
    ) -> ProjectionSourceParseError {
        let span = self.span(start, end);
        let source = SourceRef::new(self.source_id, span);
        ProjectionSourceParseError::Diagnostic(ProjectionSourceParserDiagnostic {
            kind,
            coordinate: DiagnosticCoordinate::new(
                Some(source),
                kind.code(),
                u64::from(span.start()),
                u64::from(span.end()),
            ),
            source_id: self.source_id,
            span,
        })
    }

    fn eof_error(&self) -> ProjectionSourceParseError {
        self.error(
            ProjectionSourceParserDiagnosticKind::UnexpectedEof,
            self.bytes.len(),
            self.bytes.len(),
        )
    }

    fn source_ref(&self, start: usize, end: usize) -> SourceRef {
        SourceRef::new(self.source_id, self.span(start, end))
    }

    fn span(&self, start: usize, end: usize) -> SourceSpan {
        SourceSpan::new(
            u32::try_from(start).expect("source-size preflight bounds start"),
            u32::try_from(end).expect("source-size preflight bounds end"),
        )
        .expect("parser preserves ordered spans")
    }

    fn scalar_width(&self) -> usize {
        self.text[self.pos..]
            .chars()
            .next()
            .expect("not at EOF")
            .len_utf8()
    }

    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn at_end(&self) -> bool {
        self.pos == self.bytes.len()
    }
}

fn is_valid_identifier(text: &str) -> bool {
    let mut bytes = text.bytes();
    bytes.next().is_some_and(|byte| byte.is_ascii_lowercase())
        && bytes.all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_')
}

fn is_keyword(text: &str) -> bool {
    matches!(
        text,
        "projection"
            | "v0"
            | "revision"
            | "epoch"
            | "surface"
            | "root"
            | "key"
            | "node"
            | "role"
            | "child"
            | "order"
    )
}
