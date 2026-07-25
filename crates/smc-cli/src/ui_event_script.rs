//! Deterministic Event Script v0: the `--events` input format for
//! `smc look ui frame` source mode (Issue #1365).
//!
//! Frozen frame-step model: each `steps[i]` is delivered as exactly one
//! batch to one `DesktopSession::tick_in_memory_frame` call and produces
//! exactly one captured frame (multiple events may share a step, matching
//! `tick_in_memory_frame`'s own real "whatever was queued since the last
//! drain" batching -- this is not an invented rule). Frame 0 is always the
//! initial state captured before any step runs, so captured frame count is
//! always `steps.len() + 1`.
//!
//! Supported `kind`s are exactly the vocabulary
//! `prom_ui_runtime::reference_contour::dispatch_input_event` has ever
//! processed: `pointer_move`, `pointer_down`, `pointer_up`, `key_down`,
//! `key_up`, `close`. `pointer_up`/`key_up`/`close` are accepted and
//! delivered but are no-ops against the current reference contour, exactly
//! as they are in the native reference application's own event loop --
//! this format does not invent new event semantics to fill them in.
//!
//! `x`/`y`/`button`/`key_code` are required to be JSON integers (no
//! fractional part): pointer coordinates are restricted to integers so
//! there is no floating-point parsing/formatting ambiguity to solve at all.

use prom_ui_runtime::InputEventKind;

use crate::canonical_json::{JsonCursor, JsonErrorKind, JsonSyntaxError};

pub const SCHEMA_NAME: &str = "semantic.ui.event_script.v0";
pub const SCHEMA_VERSION: u32 = 0;

pub const MAX_EVENT_SCRIPT_BYTES: u64 = 256 * 1024;
pub const MAX_STEPS: usize = 4096;
pub const MAX_EVENTS_PER_STEP: usize = 64;
pub const MAX_TOTAL_EVENTS: usize = 16384;
/// Pointer/key numeric fields are bounded to this magnitude so they fit
/// exactly and losslessly into the `f64`/`u32` fields `InputEventKind`
/// actually carries.
pub const MAX_COORDINATE_MAGNITUDE: i64 = 1_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventKindV0 {
    PointerMove { x: i32, y: i32 },
    PointerDown { button: u32 },
    PointerUp { button: u32 },
    KeyDown { key_code: u32 },
    KeyUp { key_code: u32 },
    Close,
}

impl EventKindV0 {
    pub fn to_input_event_kind(self) -> InputEventKind {
        match self {
            EventKindV0::PointerMove { x, y } => InputEventKind::PointerMoved {
                x: x as f64,
                y: y as f64,
            },
            EventKindV0::PointerDown { button } => InputEventKind::PointerDown { button },
            EventKindV0::PointerUp { button } => InputEventKind::PointerUp { button },
            EventKindV0::KeyDown { key_code } => InputEventKind::KeyDown { key_code },
            EventKindV0::KeyUp { key_code } => InputEventKind::KeyUp { key_code },
            EventKindV0::Close => InputEventKind::CloseRequested,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StepV0 {
    pub events: Vec<EventKindV0>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct EventScriptV0 {
    pub steps: Vec<StepV0>,
}

impl EventScriptV0 {
    pub fn decode_canonical(bytes: &[u8]) -> Result<Self, EventScriptDecodeError> {
        if bytes.len() as u64 > MAX_EVENT_SCRIPT_BYTES {
            return Err(EventScriptDecodeError::OversizedInput {
                max_bytes: MAX_EVENT_SCRIPT_BYTES,
                actual_bytes: bytes.len() as u64,
            });
        }
        let text = std::str::from_utf8(bytes)
            .map_err(|_| EventScriptDecodeError::malformed("input is not valid UTF-8", 0))?;
        let mut p = Parser {
            cursor: JsonCursor::new(text),
            total_events: 0,
        };
        p.parse_document()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EventScriptDecodeError {
    OversizedInput { max_bytes: u64, actual_bytes: u64 },
    Malformed { message: String, offset: usize },
    UnsupportedVersion { found: String },
    UnknownEventKind { kind: String, offset: usize },
    ResourceLimitExceeded(String),
}

impl EventScriptDecodeError {
    fn malformed(message: impl Into<String>, offset: usize) -> Self {
        EventScriptDecodeError::Malformed {
            message: message.into(),
            offset,
        }
    }
}

impl std::fmt::Display for EventScriptDecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventScriptDecodeError::OversizedInput {
                max_bytes,
                actual_bytes,
            } => write!(
                f,
                "event script is {actual_bytes} bytes, exceeds MAX_EVENT_SCRIPT_BYTES={max_bytes}"
            ),
            EventScriptDecodeError::Malformed { message, offset } => {
                write!(
                    f,
                    "malformed event script at byte offset {offset}: {message}"
                )
            }
            EventScriptDecodeError::UnsupportedVersion { found } => write!(
                f,
                "unsupported schema_version {found}; only {SCHEMA_VERSION} is accepted"
            ),
            EventScriptDecodeError::UnknownEventKind { kind, offset } => {
                write!(f, "unknown event kind {kind:?} at byte offset {offset}")
            }
            EventScriptDecodeError::ResourceLimitExceeded(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<JsonSyntaxError> for EventScriptDecodeError {
    fn from(e: JsonSyntaxError) -> Self {
        match e.kind {
            JsonErrorKind::LimitExceeded => {
                EventScriptDecodeError::ResourceLimitExceeded(e.message)
            }
            JsonErrorKind::Syntax => EventScriptDecodeError::Malformed {
                message: e.message,
                offset: e.offset,
            },
        }
    }
}

struct Parser<'a> {
    cursor: JsonCursor<'a>,
    total_events: usize,
}

impl<'a> Parser<'a> {
    fn err(&self, message: impl Into<String>) -> EventScriptDecodeError {
        EventScriptDecodeError::malformed(message, self.cursor.pos())
    }

    fn parse_bounded_i64(&mut self) -> Result<i64, EventScriptDecodeError> {
        let start = self.cursor.pos();
        let v = self.cursor.parse_canonical_int()?;
        if v.unsigned_abs() > MAX_COORDINATE_MAGNITUDE as u64 {
            return Err(EventScriptDecodeError::malformed(
                format!("value exceeds MAX_COORDINATE_MAGNITUDE={MAX_COORDINATE_MAGNITUDE}"),
                start,
            ));
        }
        Ok(v)
    }

    fn parse_i32_field(&mut self) -> Result<i32, EventScriptDecodeError> {
        let v = self.parse_bounded_i64()?;
        Ok(v as i32)
    }

    fn parse_u32_field(&mut self) -> Result<u32, EventScriptDecodeError> {
        let start = self.cursor.pos();
        let v = self.parse_bounded_i64()?;
        u32::try_from(v)
            .map_err(|_| EventScriptDecodeError::malformed("expected a non-negative value", start))
    }

    fn parse_event(&mut self) -> Result<EventKindV0, EventScriptDecodeError> {
        self.cursor.expect_byte(b'{')?;
        self.cursor.expect_literal("\"kind\":")?;
        let kind_offset = self.cursor.pos();
        let kind = self.cursor.parse_json_string(64)?;
        let event = match kind.as_str() {
            "pointer_move" => {
                self.cursor.expect_byte(b',')?;
                self.cursor.expect_literal("\"x\":")?;
                let x = self.parse_i32_field()?;
                self.cursor.expect_byte(b',')?;
                self.cursor.expect_literal("\"y\":")?;
                let y = self.parse_i32_field()?;
                EventKindV0::PointerMove { x, y }
            }
            "pointer_down" => {
                self.cursor.expect_byte(b',')?;
                self.cursor.expect_literal("\"button\":")?;
                let button = self.parse_u32_field()?;
                EventKindV0::PointerDown { button }
            }
            "pointer_up" => {
                self.cursor.expect_byte(b',')?;
                self.cursor.expect_literal("\"button\":")?;
                let button = self.parse_u32_field()?;
                EventKindV0::PointerUp { button }
            }
            "key_down" => {
                self.cursor.expect_byte(b',')?;
                self.cursor.expect_literal("\"key_code\":")?;
                let key_code = self.parse_u32_field()?;
                EventKindV0::KeyDown { key_code }
            }
            "key_up" => {
                self.cursor.expect_byte(b',')?;
                self.cursor.expect_literal("\"key_code\":")?;
                let key_code = self.parse_u32_field()?;
                EventKindV0::KeyUp { key_code }
            }
            "close" => EventKindV0::Close,
            other => {
                return Err(EventScriptDecodeError::UnknownEventKind {
                    kind: other.to_string(),
                    offset: kind_offset,
                });
            }
        };
        self.cursor.expect_byte(b'}')?;
        Ok(event)
    }

    fn parse_step(&mut self) -> Result<StepV0, EventScriptDecodeError> {
        self.cursor.expect_byte(b'{')?;
        self.cursor.expect_literal("\"events\":[")?;
        let mut events = Vec::new();
        if self.cursor.peek() != Some(b']') {
            loop {
                if events.len() >= MAX_EVENTS_PER_STEP {
                    return Err(EventScriptDecodeError::ResourceLimitExceeded(format!(
                        "step exceeds MAX_EVENTS_PER_STEP={MAX_EVENTS_PER_STEP}"
                    )));
                }
                self.total_events += 1;
                if self.total_events > MAX_TOTAL_EVENTS {
                    return Err(EventScriptDecodeError::ResourceLimitExceeded(format!(
                        "event script exceeds MAX_TOTAL_EVENTS={MAX_TOTAL_EVENTS}"
                    )));
                }
                events.push(self.parse_event()?);
                if self.cursor.peek() == Some(b',') {
                    self.cursor.expect_byte(b',')?;
                    continue;
                }
                break;
            }
        }
        self.cursor.expect_byte(b']')?;
        self.cursor.expect_byte(b'}')?;
        Ok(StepV0 { events })
    }

    fn parse_document(&mut self) -> Result<EventScriptV0, EventScriptDecodeError> {
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
            return Err(EventScriptDecodeError::UnsupportedVersion {
                found: version.to_string(),
            });
        }
        self.cursor.expect_byte(b',')?;
        self.cursor.expect_literal("\"steps\":[")?;
        let mut steps = Vec::new();
        if self.cursor.peek() != Some(b']') {
            loop {
                if steps.len() >= MAX_STEPS {
                    return Err(EventScriptDecodeError::ResourceLimitExceeded(format!(
                        "event script exceeds MAX_STEPS={MAX_STEPS}"
                    )));
                }
                steps.push(self.parse_step()?);
                if self.cursor.peek() == Some(b',') {
                    self.cursor.expect_byte(b',')?;
                    continue;
                }
                break;
            }
        }
        self.cursor.expect_byte(b']')?;
        self.cursor.expect_byte(b'}')?;
        self.cursor.expect_byte(b'\n')?;
        if !self.cursor.at_end() {
            return Err(EventScriptDecodeError::malformed(
                "trailing bytes after document",
                self.cursor.pos(),
            ));
        }
        Ok(EventScriptV0 { steps })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn doc(steps_json: &str) -> String {
        format!("{{\"schema\":\"{SCHEMA_NAME}\",\"schema_version\":0,\"steps\":[{steps_json}]}}\n")
    }

    #[test]
    fn valid_event_sequence_decodes_in_order() {
        let text = doc(
            "{\"events\":[{\"kind\":\"pointer_move\",\"x\":300,\"y\":75}]},\
             {\"events\":[{\"kind\":\"pointer_down\",\"button\":0}]},\
             {\"events\":[{\"kind\":\"key_down\",\"key_code\":9},{\"kind\":\"key_down\",\"key_code\":13}]}",
        );
        let script = EventScriptV0::decode_canonical(text.as_bytes()).expect("decodes");
        assert_eq!(script.steps.len(), 3);
        assert_eq!(
            script.steps[0].events,
            vec![EventKindV0::PointerMove { x: 300, y: 75 }]
        );
        assert_eq!(
            script.steps[1].events,
            vec![EventKindV0::PointerDown { button: 0 }]
        );
        assert_eq!(
            script.steps[2].events,
            vec![
                EventKindV0::KeyDown { key_code: 9 },
                EventKindV0::KeyDown { key_code: 13 }
            ]
        );
        let total: usize = script.steps.iter().map(|s| s.events.len()).sum();
        assert_eq!(total, 4);
    }

    #[test]
    fn empty_script_decodes() {
        let text = doc("");
        let script = EventScriptV0::decode_canonical(text.as_bytes()).expect("decodes");
        assert_eq!(script.steps.len(), 0);
    }

    #[test]
    fn all_event_kinds_map_to_expected_input_event_kind() {
        assert_eq!(
            EventKindV0::PointerMove { x: 1, y: 2 }.to_input_event_kind(),
            InputEventKind::PointerMoved { x: 1.0, y: 2.0 }
        );
        assert_eq!(
            EventKindV0::PointerDown { button: 3 }.to_input_event_kind(),
            InputEventKind::PointerDown { button: 3 }
        );
        assert_eq!(
            EventKindV0::PointerUp { button: 3 }.to_input_event_kind(),
            InputEventKind::PointerUp { button: 3 }
        );
        assert_eq!(
            EventKindV0::KeyDown { key_code: 9 }.to_input_event_kind(),
            InputEventKind::KeyDown { key_code: 9 }
        );
        assert_eq!(
            EventKindV0::KeyUp { key_code: 9 }.to_input_event_kind(),
            InputEventKind::KeyUp { key_code: 9 }
        );
        assert_eq!(
            EventKindV0::Close.to_input_event_kind(),
            InputEventKind::CloseRequested
        );
    }

    #[test]
    fn rejects_invalid_event_kind() {
        let text = doc("{\"events\":[{\"kind\":\"teleport\",\"x\":0,\"y\":0}]}");
        assert!(matches!(
            EventScriptV0::decode_canonical(text.as_bytes()),
            Err(EventScriptDecodeError::UnknownEventKind { kind, .. }) if kind == "teleport"
        ));
    }

    #[test]
    fn rejects_malformed_numeric_field() {
        let text = doc("{\"events\":[{\"kind\":\"pointer_move\",\"x\":1.5,\"y\":0}]}");
        assert!(matches!(
            EventScriptV0::decode_canonical(text.as_bytes()),
            Err(EventScriptDecodeError::Malformed { .. })
        ));
    }

    #[test]
    fn rejects_out_of_magnitude_numeric_field() {
        let text = doc("{\"events\":[{\"kind\":\"pointer_move\",\"x\":999999999,\"y\":0}]}");
        assert!(matches!(
            EventScriptV0::decode_canonical(text.as_bytes()),
            Err(EventScriptDecodeError::Malformed { .. })
        ));
    }

    #[test]
    fn rejects_negative_button() {
        let text = doc("{\"events\":[{\"kind\":\"pointer_down\",\"button\":-1}]}");
        assert!(matches!(
            EventScriptV0::decode_canonical(text.as_bytes()),
            Err(EventScriptDecodeError::Malformed { .. })
        ));
    }

    #[test]
    fn rejects_unsupported_version() {
        let text = format!("{{\"schema\":\"{SCHEMA_NAME}\",\"schema_version\":7,\"steps\":[]}}\n");
        assert!(matches!(
            EventScriptV0::decode_canonical(text.as_bytes()),
            Err(EventScriptDecodeError::UnsupportedVersion { found }) if found == "7"
        ));
    }

    #[test]
    fn rejects_excessive_step_count() {
        let mut steps = String::new();
        for i in 0..=MAX_STEPS {
            if i > 0 {
                steps.push(',');
            }
            steps.push_str("{\"events\":[]}");
        }
        let text = doc(&steps);
        assert!(matches!(
            EventScriptV0::decode_canonical(text.as_bytes()),
            Err(EventScriptDecodeError::ResourceLimitExceeded(_))
        ));
    }

    #[test]
    fn rejects_excessive_events_per_step() {
        let mut events = String::new();
        for i in 0..=MAX_EVENTS_PER_STEP {
            if i > 0 {
                events.push(',');
            }
            events.push_str("{\"kind\":\"close\"}");
        }
        let text = doc(&format!("{{\"events\":[{events}]}}"));
        assert!(matches!(
            EventScriptV0::decode_canonical(text.as_bytes()),
            Err(EventScriptDecodeError::ResourceLimitExceeded(_))
        ));
    }

    #[test]
    fn rejects_oversized_input() {
        let mut huge = String::from("{\"schema\":\"x\",\"padding\":\"");
        while (huge.len() as u64) <= MAX_EVENT_SCRIPT_BYTES {
            huge.push('a');
        }
        assert!(matches!(
            EventScriptV0::decode_canonical(huge.as_bytes()),
            Err(EventScriptDecodeError::OversizedInput { .. })
        ));
    }

    #[test]
    fn deterministic_ordering_is_preserved_across_decodes() {
        let text = doc(
            "{\"events\":[{\"kind\":\"pointer_move\",\"x\":1,\"y\":2}]},{\"events\":[{\"kind\":\"key_down\",\"key_code\":9}]}",
        );
        let a = EventScriptV0::decode_canonical(text.as_bytes()).expect("decodes");
        let b = EventScriptV0::decode_canonical(text.as_bytes()).expect("decodes");
        assert_eq!(a, b);
    }

    #[test]
    fn rejects_trailing_garbage() {
        let mut text = doc("{\"events\":[]}");
        text.push_str("garbage");
        assert!(matches!(
            EventScriptV0::decode_canonical(text.as_bytes()),
            Err(EventScriptDecodeError::Malformed { .. })
        ));
    }
}
