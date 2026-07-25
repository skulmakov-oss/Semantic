//! Canonical Frame Snapshot v0: the on-disk/`--out` artifact format for
//! `smc look ui frame` (Issue #1365).
//!
//! Pure inspection evidence for a set of captured `DrawFrame` values -- not
//! a session checkpoint, not Semantic truth, not live runtime state. One
//! frozen schema, `schema_version = 0`. Used identically as the `--from`
//! input format and as the `--format draw-json` output, so a captured
//! source-mode run written with `--out` and re-read with `--from` round-
//! trips byte-for-byte.
//!
//! The encoder always emits the same compact canonical form: no
//! insignificant whitespace anywhere, keys in a fixed order, exactly one
//! trailing `\n`. The decoder is correspondingly strict: it accepts only
//! that exact canonical form (not permissive JSON that happens to carry the
//! same values) -- this is a purpose-built parser for one fixed schema, not
//! a general JSON library, matching this crate's existing zero-external-
//! JSON-dependency convention.

use std::fmt;

use prom_ui_runtime::{Color, DrawCommand, Rect};

use crate::canonical_json::{JsonCursor, JsonErrorKind, JsonSyntaxError};

/// Schema identity string embedded in every canonical document.
pub const SCHEMA_NAME: &str = "semantic.ui.frame_snapshot.v0";
/// The only schema version this codec accepts.
pub const SCHEMA_VERSION: u32 = 0;

/// Maximum accepted byte length of a snapshot document, checked against
/// file size *before* the file is read into memory.
pub const MAX_SNAPSHOT_BYTES: u64 = 4 * 1024 * 1024;
/// Maximum number of frames a single document may contain.
pub const MAX_FRAMES: usize = 4096;
/// Maximum number of draw commands a single frame may contain.
pub const MAX_COMMANDS_PER_FRAME: usize = 4096;
/// Maximum UTF-8 byte length of a single `DrawText` command's text.
pub const MAX_TEXT_BYTES: usize = 8192;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorV0 {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<Color> for ColorV0 {
    fn from(c: Color) -> Self {
        Self {
            r: c.r,
            g: c.g,
            b: c.b,
            a: c.a,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RectV0 {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

impl From<Rect> for RectV0 {
    fn from(r: Rect) -> Self {
        Self {
            x: r.x,
            y: r.y,
            width: r.width,
            height: r.height,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DrawCommandV0 {
    Clear {
        color: ColorV0,
    },
    FillRect {
        rect: RectV0,
        color: ColorV0,
    },
    DrawText {
        text: String,
        x: i32,
        y: i32,
        color: ColorV0,
    },
}

impl From<&DrawCommand> for DrawCommandV0 {
    fn from(cmd: &DrawCommand) -> Self {
        match cmd {
            DrawCommand::Clear { color } => DrawCommandV0::Clear {
                color: (*color).into(),
            },
            DrawCommand::FillRect { rect, color } => DrawCommandV0::FillRect {
                rect: (*rect).into(),
                color: (*color).into(),
            },
            DrawCommand::DrawText { text, x, y, color } => DrawCommandV0::DrawText {
                text: text.clone(),
                x: *x,
                y: *y,
                color: (*color).into(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameV0 {
    pub commands: Vec<DrawCommandV0>,
}

/// A complete canonical Frame Snapshot v0 document: zero or more captured
/// frames, in capture order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FrameSnapshotV0 {
    pub frames: Vec<FrameV0>,
}

impl FrameSnapshotV0 {
    pub fn from_captured_frames<'a, I>(frames: I) -> Result<Self, SnapshotEncodeError>
    where
        I: IntoIterator<Item = &'a [DrawCommand]>,
    {
        let mut out = Vec::new();
        for (index, commands) in frames.into_iter().enumerate() {
            if out.len() >= MAX_FRAMES {
                return Err(SnapshotEncodeError::ResourceLimitExceeded(format!(
                    "captured frame count exceeds MAX_FRAMES={MAX_FRAMES} at frame {index}"
                )));
            }
            if commands.len() > MAX_COMMANDS_PER_FRAME {
                return Err(SnapshotEncodeError::ResourceLimitExceeded(format!(
                    "frame {index} has {} draw commands, exceeds MAX_COMMANDS_PER_FRAME={MAX_COMMANDS_PER_FRAME}",
                    commands.len()
                )));
            }
            out.push(FrameV0 {
                commands: commands.iter().map(DrawCommandV0::from).collect(),
            });
        }
        Ok(Self { frames: out })
    }

    /// Encodes to the canonical compact form: no insignificant whitespace,
    /// fixed key order, exactly one trailing `\n`.
    pub fn encode_canonical(&self) -> String {
        let mut out = String::new();
        out.push('{');
        out.push_str("\"schema\":");
        push_json_string(&mut out, SCHEMA_NAME);
        out.push(',');
        out.push_str("\"schema_version\":");
        out.push_str(&SCHEMA_VERSION.to_string());
        out.push(',');
        out.push_str("\"frame_count\":");
        out.push_str(&self.frames.len().to_string());
        out.push(',');
        out.push_str("\"frames\":[");
        for (index, frame) in self.frames.iter().enumerate() {
            if index > 0 {
                out.push(',');
            }
            out.push('{');
            out.push_str("\"index\":");
            out.push_str(&index.to_string());
            out.push(',');
            out.push_str("\"commands\":[");
            for (cmd_index, cmd) in frame.commands.iter().enumerate() {
                if cmd_index > 0 {
                    out.push(',');
                }
                push_command(&mut out, cmd);
            }
            out.push(']');
            out.push('}');
        }
        out.push(']');
        out.push('}');
        out.push('\n');
        out
    }

    /// Reads and decodes a document, enforcing every bound before doing any
    /// proportional allocation: size is checked, then structural/resource
    /// limits are enforced as parsing proceeds token by token.
    pub fn decode_canonical(bytes: &[u8]) -> Result<Self, SnapshotDecodeError> {
        if bytes.len() as u64 > MAX_SNAPSHOT_BYTES {
            return Err(SnapshotDecodeError::OversizedInput {
                max_bytes: MAX_SNAPSHOT_BYTES,
                actual_bytes: bytes.len() as u64,
            });
        }
        let text = std::str::from_utf8(bytes)
            .map_err(|_| SnapshotDecodeError::malformed("input is not valid UTF-8", 0))?;
        let mut p = Parser::new(text);
        let snapshot = p.parse_document()?;
        Ok(snapshot)
    }
}

fn push_command(out: &mut String, cmd: &DrawCommandV0) {
    match cmd {
        DrawCommandV0::Clear { color } => {
            out.push_str("{\"kind\":\"clear\",\"color\":");
            push_color(out, color);
            out.push('}');
        }
        DrawCommandV0::FillRect { rect, color } => {
            out.push_str("{\"kind\":\"fill_rect\",\"rect\":");
            push_rect(out, rect);
            out.push_str(",\"color\":");
            push_color(out, color);
            out.push('}');
        }
        DrawCommandV0::DrawText { text, x, y, color } => {
            out.push_str("{\"kind\":\"draw_text\",\"text\":");
            push_json_string(out, text);
            out.push_str(",\"x\":");
            out.push_str(&x.to_string());
            out.push_str(",\"y\":");
            out.push_str(&y.to_string());
            out.push_str(",\"color\":");
            push_color(out, color);
            out.push('}');
        }
    }
}

fn push_color(out: &mut String, c: &ColorV0) {
    out.push_str("{\"r\":");
    out.push_str(&c.r.to_string());
    out.push_str(",\"g\":");
    out.push_str(&c.g.to_string());
    out.push_str(",\"b\":");
    out.push_str(&c.b.to_string());
    out.push_str(",\"a\":");
    out.push_str(&c.a.to_string());
    out.push('}');
}

fn push_rect(out: &mut String, r: &RectV0) {
    out.push_str("{\"x\":");
    out.push_str(&r.x.to_string());
    out.push_str(",\"y\":");
    out.push_str(&r.y.to_string());
    out.push_str(",\"width\":");
    out.push_str(&r.width.to_string());
    out.push_str(",\"height\":");
    out.push_str(&r.height.to_string());
    out.push('}');
}

fn push_json_string(out: &mut String, s: &str) {
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", c as u32));
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotEncodeError {
    ResourceLimitExceeded(String),
}

impl fmt::Display for SnapshotEncodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnapshotEncodeError::ResourceLimitExceeded(msg) => write!(f, "{msg}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SnapshotDecodeError {
    OversizedInput { max_bytes: u64, actual_bytes: u64 },
    Malformed { message: String, offset: usize },
    UnsupportedVersion { found: String },
    ResourceLimitExceeded(String),
}

impl SnapshotDecodeError {
    fn malformed(message: impl Into<String>, offset: usize) -> Self {
        SnapshotDecodeError::Malformed {
            message: message.into(),
            offset,
        }
    }
}

impl fmt::Display for SnapshotDecodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SnapshotDecodeError::OversizedInput {
                max_bytes,
                actual_bytes,
            } => write!(
                f,
                "snapshot is {actual_bytes} bytes, exceeds MAX_SNAPSHOT_BYTES={max_bytes}"
            ),
            SnapshotDecodeError::Malformed { message, offset } => {
                write!(f, "malformed snapshot at byte offset {offset}: {message}")
            }
            SnapshotDecodeError::UnsupportedVersion { found } => {
                write!(
                    f,
                    "unsupported schema_version {found}; only {SCHEMA_VERSION} is accepted"
                )
            }
            SnapshotDecodeError::ResourceLimitExceeded(msg) => write!(f, "{msg}"),
        }
    }
}

// ---- strict canonical-only decoder, built on the shared JSON cursor ----

impl From<JsonSyntaxError> for SnapshotDecodeError {
    fn from(e: JsonSyntaxError) -> Self {
        match e.kind {
            JsonErrorKind::LimitExceeded => SnapshotDecodeError::ResourceLimitExceeded(e.message),
            JsonErrorKind::Syntax => SnapshotDecodeError::Malformed {
                message: e.message,
                offset: e.offset,
            },
        }
    }
}

struct Parser<'a> {
    cursor: JsonCursor<'a>,
}

impl<'a> Parser<'a> {
    fn new(text: &'a str) -> Self {
        Self {
            cursor: JsonCursor::new(text),
        }
    }

    fn err(&self, message: impl Into<String>) -> SnapshotDecodeError {
        SnapshotDecodeError::malformed(message, self.cursor.pos())
    }

    fn parse_u8(&mut self) -> Result<u8, SnapshotDecodeError> {
        let start = self.cursor.pos();
        let v = self.cursor.parse_canonical_int()?;
        u8::try_from(v)
            .map_err(|_| SnapshotDecodeError::malformed("value out of range for u8", start))
    }

    fn parse_i32(&mut self) -> Result<i32, SnapshotDecodeError> {
        let start = self.cursor.pos();
        let v = self.cursor.parse_canonical_int()?;
        i32::try_from(v)
            .map_err(|_| SnapshotDecodeError::malformed("value out of range for i32", start))
    }

    fn parse_u32(&mut self) -> Result<u32, SnapshotDecodeError> {
        let start = self.cursor.pos();
        let v = self.cursor.parse_canonical_int()?;
        u32::try_from(v)
            .map_err(|_| SnapshotDecodeError::malformed("value out of range for u32", start))
    }

    fn parse_usize(&mut self) -> Result<usize, SnapshotDecodeError> {
        let start = self.cursor.pos();
        let v = self.cursor.parse_canonical_int()?;
        usize::try_from(v).map_err(|_| SnapshotDecodeError::malformed("value out of range", start))
    }

    fn parse_color(&mut self) -> Result<ColorV0, SnapshotDecodeError> {
        self.cursor.expect_byte(b'{')?;
        self.cursor.expect_literal("\"r\":")?;
        let r = self.parse_u8()?;
        self.cursor.expect_byte(b',')?;
        self.cursor.expect_literal("\"g\":")?;
        let g = self.parse_u8()?;
        self.cursor.expect_byte(b',')?;
        self.cursor.expect_literal("\"b\":")?;
        let b = self.parse_u8()?;
        self.cursor.expect_byte(b',')?;
        self.cursor.expect_literal("\"a\":")?;
        let a = self.parse_u8()?;
        self.cursor.expect_byte(b'}')?;
        Ok(ColorV0 { r, g, b, a })
    }

    fn parse_rect(&mut self) -> Result<RectV0, SnapshotDecodeError> {
        self.cursor.expect_byte(b'{')?;
        self.cursor.expect_literal("\"x\":")?;
        let x = self.parse_i32()?;
        self.cursor.expect_byte(b',')?;
        self.cursor.expect_literal("\"y\":")?;
        let y = self.parse_i32()?;
        self.cursor.expect_byte(b',')?;
        self.cursor.expect_literal("\"width\":")?;
        let width = self.parse_u32()?;
        self.cursor.expect_byte(b',')?;
        self.cursor.expect_literal("\"height\":")?;
        let height = self.parse_u32()?;
        self.cursor.expect_byte(b'}')?;
        Ok(RectV0 {
            x,
            y,
            width,
            height,
        })
    }

    fn parse_command(&mut self) -> Result<DrawCommandV0, SnapshotDecodeError> {
        self.cursor.expect_byte(b'{')?;
        self.cursor.expect_literal("\"kind\":")?;
        let kind = self.cursor.parse_json_string(MAX_TEXT_BYTES)?;
        let cmd = match kind.as_str() {
            "clear" => {
                self.cursor.expect_byte(b',')?;
                self.cursor.expect_literal("\"color\":")?;
                let color = self.parse_color()?;
                DrawCommandV0::Clear { color }
            }
            "fill_rect" => {
                self.cursor.expect_byte(b',')?;
                self.cursor.expect_literal("\"rect\":")?;
                let rect = self.parse_rect()?;
                self.cursor.expect_byte(b',')?;
                self.cursor.expect_literal("\"color\":")?;
                let color = self.parse_color()?;
                DrawCommandV0::FillRect { rect, color }
            }
            "draw_text" => {
                self.cursor.expect_byte(b',')?;
                self.cursor.expect_literal("\"text\":")?;
                let text = self.cursor.parse_json_string(MAX_TEXT_BYTES)?;
                self.cursor.expect_byte(b',')?;
                self.cursor.expect_literal("\"x\":")?;
                let x = self.parse_i32()?;
                self.cursor.expect_byte(b',')?;
                self.cursor.expect_literal("\"y\":")?;
                let y = self.parse_i32()?;
                self.cursor.expect_byte(b',')?;
                self.cursor.expect_literal("\"color\":")?;
                let color = self.parse_color()?;
                DrawCommandV0::DrawText { text, x, y, color }
            }
            other => {
                return Err(self.err(format!("unknown draw command kind {other:?}")));
            }
        };
        self.cursor.expect_byte(b'}')?;
        Ok(cmd)
    }

    fn parse_frame(&mut self, expected_index: usize) -> Result<FrameV0, SnapshotDecodeError> {
        self.cursor.expect_byte(b'{')?;
        self.cursor.expect_literal("\"index\":")?;
        let index = self.parse_usize()?;
        if index != expected_index {
            return Err(self.err(format!(
                "frame index {index} out of sequence; expected {expected_index}"
            )));
        }
        self.cursor.expect_byte(b',')?;
        self.cursor.expect_literal("\"commands\":[")?;
        let mut commands = Vec::new();
        if self.cursor.peek() != Some(b']') {
            loop {
                if commands.len() >= MAX_COMMANDS_PER_FRAME {
                    return Err(SnapshotDecodeError::ResourceLimitExceeded(format!(
                        "frame {index} exceeds MAX_COMMANDS_PER_FRAME={MAX_COMMANDS_PER_FRAME}"
                    )));
                }
                commands.push(self.parse_command()?);
                if self.cursor.peek() == Some(b',') {
                    self.cursor.expect_byte(b',')?;
                    continue;
                }
                break;
            }
        }
        self.cursor.expect_byte(b']')?;
        self.cursor.expect_byte(b'}')?;
        Ok(FrameV0 { commands })
    }

    fn parse_document(&mut self) -> Result<FrameSnapshotV0, SnapshotDecodeError> {
        self.cursor.expect_byte(b'{')?;
        self.cursor.expect_literal("\"schema\":")?;
        let schema = self.cursor.parse_json_string(SCHEMA_NAME.len() + 64)?;
        if schema != SCHEMA_NAME {
            return Err(self.err(format!("unknown schema {schema:?}")));
        }
        self.cursor.expect_byte(b',')?;
        self.cursor.expect_literal("\"schema_version\":")?;
        let version = self.cursor.parse_canonical_int()?;
        if version != SCHEMA_VERSION as i64 {
            return Err(SnapshotDecodeError::UnsupportedVersion {
                found: version.to_string(),
            });
        }
        self.cursor.expect_byte(b',')?;
        self.cursor.expect_literal("\"frame_count\":")?;
        let frame_count = self.parse_usize()?;
        if frame_count > MAX_FRAMES {
            return Err(SnapshotDecodeError::ResourceLimitExceeded(format!(
                "frame_count {frame_count} exceeds MAX_FRAMES={MAX_FRAMES}"
            )));
        }
        self.cursor.expect_byte(b',')?;
        self.cursor.expect_literal("\"frames\":[")?;
        let mut frames = Vec::with_capacity(frame_count.min(MAX_FRAMES));
        if self.cursor.peek() != Some(b']') {
            let mut index = 0usize;
            loop {
                if frames.len() >= MAX_FRAMES {
                    return Err(SnapshotDecodeError::ResourceLimitExceeded(format!(
                        "frame count exceeds MAX_FRAMES={MAX_FRAMES}"
                    )));
                }
                frames.push(self.parse_frame(index)?);
                index += 1;
                if self.cursor.peek() == Some(b',') {
                    self.cursor.expect_byte(b',')?;
                    continue;
                }
                break;
            }
        }
        self.cursor.expect_byte(b']')?;
        if frames.len() != frame_count {
            return Err(self.err(format!(
                "frame_count {frame_count} does not match actual frame array length {}",
                frames.len()
            )));
        }
        self.cursor.expect_byte(b'}')?;
        self.cursor.expect_byte(b'\n')?;
        if !self.cursor.at_end() {
            return Err(SnapshotDecodeError::malformed(
                "trailing bytes after document",
                self.cursor.pos(),
            ));
        }
        Ok(FrameSnapshotV0 { frames })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> FrameSnapshotV0 {
        FrameSnapshotV0 {
            frames: vec![
                FrameV0 {
                    commands: vec![
                        DrawCommandV0::Clear {
                            color: ColorV0 {
                                r: 18,
                                g: 18,
                                b: 24,
                                a: 255,
                            },
                        },
                        DrawCommandV0::FillRect {
                            rect: RectV0 {
                                x: -5,
                                y: 20,
                                width: 560,
                                height: 110,
                            },
                            color: ColorV0 {
                                r: 50,
                                g: 56,
                                b: 74,
                                a: 255,
                            },
                        },
                        DrawCommandV0::DrawText {
                            text: "Label \"quoted\"\nline2".to_string(),
                            x: 32,
                            y: 44,
                            color: ColorV0 {
                                r: 255,
                                g: 255,
                                b: 255,
                                a: 255,
                            },
                        },
                    ],
                },
                FrameV0 { commands: vec![] },
            ],
        }
    }

    #[test]
    fn exact_roundtrip() {
        let snap = sample();
        let encoded = snap.encode_canonical();
        let decoded = FrameSnapshotV0::decode_canonical(encoded.as_bytes()).expect("decodes");
        assert_eq!(decoded, snap);
        let re_encoded = decoded.encode_canonical();
        assert_eq!(re_encoded, encoded, "re-encoding must be byte-identical");
    }

    #[test]
    fn canonical_form_has_no_insignificant_whitespace_and_one_trailing_newline() {
        // Uses an empty document deliberately: string *content* (e.g. a
        // DrawText payload) may legitimately contain spaces -- what must
        // never appear is insignificant JSON structural whitespace
        // (after `:`/`,`, pretty-printing, etc).
        let encoded = FrameSnapshotV0::default().encode_canonical();
        assert!(encoded.ends_with('\n'));
        let body = &encoded[..encoded.len() - 1];
        assert!(!body.contains(' '));
        assert!(!body.contains('\t'));
        assert!(!body.contains('\n'));
    }

    #[test]
    fn empty_snapshot_roundtrips() {
        let snap = FrameSnapshotV0::default();
        let encoded = snap.encode_canonical();
        assert_eq!(
            encoded,
            "{\"schema\":\"semantic.ui.frame_snapshot.v0\",\"schema_version\":0,\"frame_count\":0,\"frames\":[]}\n"
        );
        let decoded = FrameSnapshotV0::decode_canonical(encoded.as_bytes()).expect("decodes");
        assert_eq!(decoded, snap);
    }

    #[test]
    fn rejects_truncated_input() {
        let encoded = sample().encode_canonical();
        let truncated = &encoded.as_bytes()[..encoded.len() / 2];
        assert!(matches!(
            FrameSnapshotV0::decode_canonical(truncated),
            Err(SnapshotDecodeError::Malformed { .. })
        ));
    }

    #[test]
    fn rejects_malformed_json() {
        let bad = b"not json at all\n";
        assert!(matches!(
            FrameSnapshotV0::decode_canonical(bad),
            Err(SnapshotDecodeError::Malformed { .. })
        ));
    }

    #[test]
    fn rejects_unsupported_version() {
        let doc = "{\"schema\":\"semantic.ui.frame_snapshot.v0\",\"schema_version\":1,\"frame_count\":0,\"frames\":[]}\n";
        assert!(matches!(
            FrameSnapshotV0::decode_canonical(doc.as_bytes()),
            Err(SnapshotDecodeError::UnsupportedVersion { found }) if found == "1"
        ));
    }

    #[test]
    fn rejects_unknown_command_kind() {
        let doc = "{\"schema\":\"semantic.ui.frame_snapshot.v0\",\"schema_version\":0,\"frame_count\":1,\"frames\":[{\"index\":0,\"commands\":[{\"kind\":\"explode\",\"color\":{\"r\":0,\"g\":0,\"b\":0,\"a\":0}}]}]}\n";
        assert!(matches!(
            FrameSnapshotV0::decode_canonical(doc.as_bytes()),
            Err(SnapshotDecodeError::Malformed { .. })
        ));
    }

    #[test]
    fn rejects_trailing_garbage() {
        let mut encoded = sample().encode_canonical();
        encoded.push_str("garbage");
        assert!(matches!(
            FrameSnapshotV0::decode_canonical(encoded.as_bytes()),
            Err(SnapshotDecodeError::Malformed { .. })
        ));
    }

    #[test]
    fn rejects_oversized_input() {
        let mut huge = String::from("{\"schema\":\"x\",\"padding\":\"");
        while (huge.len() as u64) <= MAX_SNAPSHOT_BYTES {
            huge.push('a');
        }
        assert!(matches!(
            FrameSnapshotV0::decode_canonical(huge.as_bytes()),
            Err(SnapshotDecodeError::OversizedInput { .. })
        ));
    }

    #[test]
    fn rejects_oversized_command_list() {
        let mut doc = String::from(
            "{\"schema\":\"semantic.ui.frame_snapshot.v0\",\"schema_version\":0,\"frame_count\":1,\"frames\":[{\"index\":0,\"commands\":[",
        );
        for i in 0..=MAX_COMMANDS_PER_FRAME {
            if i > 0 {
                doc.push(',');
            }
            doc.push_str("{\"kind\":\"clear\",\"color\":{\"r\":0,\"g\":0,\"b\":0,\"a\":0}}");
        }
        doc.push_str("]}]}\n");
        assert!(matches!(
            FrameSnapshotV0::decode_canonical(doc.as_bytes()),
            Err(SnapshotDecodeError::ResourceLimitExceeded(_))
        ));
    }

    #[test]
    fn rejects_oversized_text() {
        let big_text = "a".repeat(MAX_TEXT_BYTES + 1);
        let doc = format!(
            "{{\"schema\":\"semantic.ui.frame_snapshot.v0\",\"schema_version\":0,\"frame_count\":1,\"frames\":[{{\"index\":0,\"commands\":[{{\"kind\":\"draw_text\",\"text\":\"{big_text}\",\"x\":0,\"y\":0,\"color\":{{\"r\":0,\"g\":0,\"b\":0,\"a\":0}}}}]}}]}}\n"
        );
        assert!(matches!(
            FrameSnapshotV0::decode_canonical(doc.as_bytes()),
            Err(SnapshotDecodeError::ResourceLimitExceeded(_))
        ));
    }

    #[test]
    fn rejects_noncanonical_whitespace() {
        let doc = "{\"schema\": \"semantic.ui.frame_snapshot.v0\",\"schema_version\":0,\"frame_count\":0,\"frames\":[]}\n";
        assert!(matches!(
            FrameSnapshotV0::decode_canonical(doc.as_bytes()),
            Err(SnapshotDecodeError::Malformed { .. })
        ));
    }

    #[test]
    fn rejects_noncanonical_leading_zero_integer() {
        let doc = "{\"schema\":\"semantic.ui.frame_snapshot.v0\",\"schema_version\":00,\"frame_count\":0,\"frames\":[]}\n";
        assert!(matches!(
            FrameSnapshotV0::decode_canonical(doc.as_bytes()),
            Err(SnapshotDecodeError::Malformed { .. })
        ));
    }

    #[test]
    fn rejects_frame_count_mismatch() {
        let doc = "{\"schema\":\"semantic.ui.frame_snapshot.v0\",\"schema_version\":0,\"frame_count\":2,\"frames\":[{\"index\":0,\"commands\":[]}]}\n";
        assert!(matches!(
            FrameSnapshotV0::decode_canonical(doc.as_bytes()),
            Err(SnapshotDecodeError::Malformed { .. })
        ));
    }

    #[test]
    fn rejects_out_of_sequence_frame_index() {
        let doc = "{\"schema\":\"semantic.ui.frame_snapshot.v0\",\"schema_version\":0,\"frame_count\":2,\"frames\":[{\"index\":0,\"commands\":[]},{\"index\":5,\"commands\":[]}]}\n";
        assert!(matches!(
            FrameSnapshotV0::decode_canonical(doc.as_bytes()),
            Err(SnapshotDecodeError::Malformed { .. })
        ));
    }

    #[test]
    fn rejects_out_of_range_u8_color_component() {
        let doc = "{\"schema\":\"semantic.ui.frame_snapshot.v0\",\"schema_version\":0,\"frame_count\":1,\"frames\":[{\"index\":0,\"commands\":[{\"kind\":\"clear\",\"color\":{\"r\":999,\"g\":0,\"b\":0,\"a\":0}}]}]}\n";
        assert!(matches!(
            FrameSnapshotV0::decode_canonical(doc.as_bytes()),
            Err(SnapshotDecodeError::Malformed { .. })
        ));
    }

    #[test]
    fn all_draw_command_variants_roundtrip() {
        let snap = FrameSnapshotV0 {
            frames: vec![FrameV0 {
                commands: vec![
                    DrawCommandV0::Clear {
                        color: ColorV0 {
                            r: 1,
                            g: 2,
                            b: 3,
                            a: 4,
                        },
                    },
                    DrawCommandV0::FillRect {
                        rect: RectV0 {
                            x: 0,
                            y: 0,
                            width: 1,
                            height: 1,
                        },
                        color: ColorV0 {
                            r: 5,
                            g: 6,
                            b: 7,
                            a: 8,
                        },
                    },
                    DrawCommandV0::DrawText {
                        text: "hi".to_string(),
                        x: -1,
                        y: -2,
                        color: ColorV0 {
                            r: 9,
                            g: 10,
                            b: 11,
                            a: 12,
                        },
                    },
                ],
            }],
        };
        let encoded = snap.encode_canonical();
        let decoded = FrameSnapshotV0::decode_canonical(encoded.as_bytes()).expect("decodes");
        assert_eq!(decoded, snap);
    }
}
