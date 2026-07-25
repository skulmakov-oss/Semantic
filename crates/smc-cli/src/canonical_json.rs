//! Shared strict, canonical-only JSON parsing primitives used by the two
//! `smc look ui frame` codecs ([`crate::ui_frame_snapshot`] and
//! [`crate::ui_event_script`]).
//!
//! This is not a general JSON library: it accepts only exactly the bytes a
//! matching canonical encoder would have produced (no insignificant
//! whitespace, no leading zeros, no leading `+`) and rejects everything
//! else as a syntax error, including otherwise-valid-JSON reformatting.
//! Each codec owns its own object/array grammar and resource limits on top
//! of these byte-level primitives.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JsonErrorKind {
    Syntax,
    LimitExceeded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonSyntaxError {
    pub kind: JsonErrorKind,
    pub message: String,
    pub offset: usize,
}

pub struct JsonCursor<'a> {
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> JsonCursor<'a> {
    pub fn new(text: &'a str) -> Self {
        Self {
            bytes: text.as_bytes(),
            pos: 0,
        }
    }

    pub fn pos(&self) -> usize {
        self.pos
    }

    pub fn at_end(&self) -> bool {
        self.pos == self.bytes.len()
    }

    pub fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    pub fn err(&self, message: impl Into<String>) -> JsonSyntaxError {
        JsonSyntaxError {
            kind: JsonErrorKind::Syntax,
            message: message.into(),
            offset: self.pos,
        }
    }

    fn limit_err(&self, message: impl Into<String>) -> JsonSyntaxError {
        JsonSyntaxError {
            kind: JsonErrorKind::LimitExceeded,
            message: message.into(),
            offset: self.pos,
        }
    }

    pub fn expect_byte(&mut self, b: u8) -> Result<(), JsonSyntaxError> {
        if self.peek() == Some(b) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.err(format!("expected byte '{}'", b as char)))
        }
    }

    pub fn expect_literal(&mut self, lit: &str) -> Result<(), JsonSyntaxError> {
        if self.bytes[self.pos..].starts_with(lit.as_bytes()) {
            self.pos += lit.len();
            Ok(())
        } else {
            Err(self.err(format!("expected literal {lit:?}")))
        }
    }

    /// Parses a JSON string. `max_bytes` bounds the decoded (not encoded)
    /// byte length, checked incrementally so an attacker-controlled length
    /// prefix can never force unbounded allocation first.
    pub fn parse_json_string(&mut self, max_bytes: usize) -> Result<String, JsonSyntaxError> {
        self.expect_byte(b'"')?;
        let mut out = String::new();
        loop {
            match self.peek() {
                None => return Err(self.err("unterminated string")),
                Some(b'"') => {
                    self.pos += 1;
                    return Ok(out);
                }
                Some(b'\\') => {
                    self.pos += 1;
                    match self.peek() {
                        Some(b'"') => {
                            out.push('"');
                            self.pos += 1;
                        }
                        Some(b'\\') => {
                            out.push('\\');
                            self.pos += 1;
                        }
                        Some(b'n') => {
                            out.push('\n');
                            self.pos += 1;
                        }
                        Some(b'r') => {
                            out.push('\r');
                            self.pos += 1;
                        }
                        Some(b't') => {
                            out.push('\t');
                            self.pos += 1;
                        }
                        Some(b'u') => {
                            self.pos += 1;
                            if self.pos + 4 > self.bytes.len() {
                                return Err(self.err("truncated \\u escape"));
                            }
                            let hex = std::str::from_utf8(&self.bytes[self.pos..self.pos + 4])
                                .map_err(|_| self.err("invalid \\u escape"))?;
                            let cp = u32::from_str_radix(hex, 16)
                                .map_err(|_| self.err("invalid \\u escape hex digits"))?;
                            let ch = char::from_u32(cp)
                                .ok_or_else(|| self.err("invalid \\u escape codepoint"))?;
                            out.push(ch);
                            self.pos += 4;
                        }
                        _ => return Err(self.err("unsupported escape sequence")),
                    }
                }
                Some(b) if b < 0x20 => {
                    return Err(self.err("raw control character in string"));
                }
                Some(_) => {
                    let rest = std::str::from_utf8(&self.bytes[self.pos..])
                        .map_err(|_| self.err("invalid UTF-8 in string"))?;
                    let ch = rest.chars().next().expect("non-empty checked above");
                    out.push(ch);
                    self.pos += ch.len_utf8();
                }
            }
            if out.len() > max_bytes {
                return Err(self.limit_err(format!("string exceeds max {max_bytes} bytes")));
            }
        }
    }

    /// Parses a canonical (no leading zeros, no leading `+`) decimal
    /// integer, optionally signed, into an `i64` for later range-checking
    /// by the caller.
    pub fn parse_canonical_int(&mut self) -> Result<i64, JsonSyntaxError> {
        let start = self.pos;
        let negative = self.peek() == Some(b'-');
        if negative {
            self.pos += 1;
        }
        let digits_start = self.pos;
        while matches!(self.peek(), Some(b) if b.is_ascii_digit()) {
            self.pos += 1;
        }
        if self.pos == digits_start {
            return Err(self.err("expected a decimal integer"));
        }
        let digits = &self.bytes[digits_start..self.pos];
        if digits.len() > 1 && digits[0] == b'0' {
            return Err(JsonSyntaxError {
                kind: JsonErrorKind::Syntax,
                message: "noncanonical integer: leading zero".to_string(),
                offset: start,
            });
        }
        let text = std::str::from_utf8(&self.bytes[start..self.pos]).expect("ascii digits");
        text.parse::<i64>().map_err(|_| JsonSyntaxError {
            kind: JsonErrorKind::Syntax,
            message: "integer out of range".to_string(),
            offset: start,
        })
    }
}
