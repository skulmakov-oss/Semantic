use crate::types::{FrontendError, Token, TokenKind};
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;
use alloc::vec::Vec;
use ton618_core::SourceMark;

fn push_tok(out: &mut Vec<Token>, kind: TokenKind, text: &str, pos: usize, line: u32, col: u32) {
    out.push(Token {
        kind,
        text: text.to_string(),
        pos,
        mark: SourceMark {
            line,
            col,
            file_id: 0,
        },
    });
}

/// SSF-09 C1A: the lexer's own producer-owned failure record. The lexer is
/// the sole authority for its diagnostic code (`E0001`/`E0002`/`E0004`/
/// `E0101`), its bare diagnostic detail, and the genuine UTF-8 byte range of
/// the offending source text. `error` is the legacy rendered
/// [`FrontendError`] every pre-existing `lex` caller still receives.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexFailure {
    pub code: &'static str,
    pub detail: String,
    pub range: Option<core::ops::Range<usize>>,
    pub error: FrontendError,
}

fn fmt_mark_error(
    code: &'static str,
    line: u32,
    col: u32,
    line_text: &str,
    detail: &str,
    pos: usize,
    range: Option<core::ops::Range<usize>>,
) -> LexFailure {
    let mut caret = String::new();
    let spaces = col.saturating_sub(1) as usize;
    for _ in 0..spaces {
        caret.push(' ');
    }
    caret.push('^');
    LexFailure {
        code,
        detail: detail.to_string(),
        range,
        error: FrontendError {
            detail: None,
            pos,
            message: format!(
                "error[{code}]: {detail}\n --> <input>:{line}:{col}\n  |\n{line:>2} | {line_text}\n  | {caret}"
            ),
        },
    }
}

fn tokenize_line(
    line_text: &str,
    line_no: u32,
    line_start: usize,
    out: &mut Vec<Token>,
) -> Result<(), LexFailure> {
    let bytes = line_text.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let c = bytes[i];
        if c == b' ' || c == b'\t' || c == b'\r' {
            i += 1;
            continue;
        }
        if c == b'#' {
            break;
        }
        if c == b'/' && i + 1 < bytes.len() && bytes[i + 1] == b'/' {
            break;
        }

        let start = i;
        let abs_pos = line_start + start;
        let col = (start + 1) as u32;

        match c {
            b'{' => {
                push_tok(out, TokenKind::LBrace, "{", abs_pos, line_no, col);
                i += 1;
            }
            b'}' => {
                push_tok(out, TokenKind::RBrace, "}", abs_pos, line_no, col);
                i += 1;
            }
            b'(' => {
                push_tok(out, TokenKind::LParen, "(", abs_pos, line_no, col);
                i += 1;
            }
            b')' => {
                push_tok(out, TokenKind::RParen, ")", abs_pos, line_no, col);
                i += 1;
            }
            b'[' => {
                push_tok(out, TokenKind::LBracket, "[", abs_pos, line_no, col);
                i += 1;
            }
            b']' => {
                push_tok(out, TokenKind::RBracket, "]", abs_pos, line_no, col);
                i += 1;
            }
            b';' => {
                push_tok(out, TokenKind::Semi, ";", abs_pos, line_no, col);
                i += 1;
            }
            b',' => {
                push_tok(out, TokenKind::Comma, ",", abs_pos, line_no, col);
                i += 1;
            }
            b':' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b':' {
                    push_tok(out, TokenKind::PathSep, "::", abs_pos, line_no, col);
                    i += 2;
                } else if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    push_tok(out, TokenKind::Assign, ":=", abs_pos, line_no, col);
                    i += 2;
                } else {
                    push_tok(out, TokenKind::Colon, ":", abs_pos, line_no, col);
                    i += 1;
                }
            }
            b'.' => {
                if i + 2 < bytes.len() && bytes[i + 1] == b'.' && bytes[i + 2] == b'=' {
                    push_tok(out, TokenKind::DotDotEq, "..=", abs_pos, line_no, col);
                    i += 3;
                } else if i + 1 < bytes.len() && bytes[i + 1] == b'.' {
                    push_tok(out, TokenKind::DotDot, "..", abs_pos, line_no, col);
                    i += 2;
                } else {
                    push_tok(out, TokenKind::Dot, ".", abs_pos, line_no, col);
                    i += 1;
                }
            }
            b'_' => {
                push_tok(out, TokenKind::Underscore, "_", abs_pos, line_no, col);
                i += 1;
            }
            b'!' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    push_tok(out, TokenKind::Ne, "!=", abs_pos, line_no, col);
                    i += 2;
                } else {
                    push_tok(out, TokenKind::Bang, "!", abs_pos, line_no, col);
                    i += 1;
                }
            }
            b'=' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    push_tok(out, TokenKind::EqEq, "==", abs_pos, line_no, col);
                    i += 2;
                } else if i + 1 < bytes.len() && bytes[i + 1] == b'>' {
                    push_tok(out, TokenKind::FatArrow, "=>", abs_pos, line_no, col);
                    i += 2;
                } else {
                    push_tok(out, TokenKind::Assign, "=", abs_pos, line_no, col);
                    i += 1;
                }
            }
            b'<' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    push_tok(out, TokenKind::Le, "<=", abs_pos, line_no, col);
                    i += 2;
                } else {
                    push_tok(out, TokenKind::LAngle, "<", abs_pos, line_no, col);
                    i += 1;
                }
            }
            b'>' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    push_tok(out, TokenKind::Ge, ">=", abs_pos, line_no, col);
                    i += 2;
                } else {
                    push_tok(out, TokenKind::RAngle, ">", abs_pos, line_no, col);
                    i += 1;
                }
            }
            b'&' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'&' {
                    if i + 2 < bytes.len() && bytes[i + 2] == b'=' {
                        push_tok(out, TokenKind::AndAndAssign, "&&=", abs_pos, line_no, col);
                        i += 3;
                    } else {
                        push_tok(out, TokenKind::AndAnd, "&&", abs_pos, line_no, col);
                        i += 2;
                    }
                } else {
                    return Err(fmt_mark_error(
                        "E0002",
                        line_no,
                        col,
                        line_text,
                        "expected '&&'",
                        abs_pos,
                        Some(abs_pos..abs_pos + 1),
                    ));
                }
            }
            b'|' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'|' {
                    if i + 2 < bytes.len() && bytes[i + 2] == b'=' {
                        push_tok(out, TokenKind::OrOrAssign, "||=", abs_pos, line_no, col);
                        i += 3;
                    } else {
                        push_tok(out, TokenKind::OrOr, "||", abs_pos, line_no, col);
                        i += 2;
                    }
                } else if i + 1 < bytes.len() && bytes[i + 1] == b'>' {
                    push_tok(out, TokenKind::PipeForward, "|>", abs_pos, line_no, col);
                    i += 2;
                } else {
                    // M9.4 Wave 2: bare `|` is the or-pattern separator.
                    push_tok(out, TokenKind::Pipe, "|", abs_pos, line_no, col);
                    i += 1;
                }
            }
            b'+' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    push_tok(out, TokenKind::PlusAssign, "+=", abs_pos, line_no, col);
                    i += 2;
                } else {
                    push_tok(out, TokenKind::Plus, "+", abs_pos, line_no, col);
                    i += 1;
                }
            }
            b'*' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    push_tok(out, TokenKind::StarAssign, "*=", abs_pos, line_no, col);
                    i += 2;
                } else {
                    push_tok(out, TokenKind::Star, "*", abs_pos, line_no, col);
                    i += 1;
                }
            }
            b'%' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    push_tok(out, TokenKind::PercentAssign, "%=", abs_pos, line_no, col);
                    i += 2;
                } else {
                    push_tok(out, TokenKind::Percent, "%", abs_pos, line_no, col);
                    i += 1;
                }
            }
            b'/' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    push_tok(out, TokenKind::SlashAssign, "/=", abs_pos, line_no, col);
                    i += 2;
                } else {
                    push_tok(out, TokenKind::Slash, "/", abs_pos, line_no, col);
                    i += 1;
                }
            }
            b'-' => {
                if i + 1 < bytes.len() && bytes[i + 1] == b'>' {
                    push_tok(out, TokenKind::Implies, "->", abs_pos, line_no, col);
                    i += 2;
                } else if i + 1 < bytes.len() && bytes[i + 1] == b'=' {
                    push_tok(out, TokenKind::MinusAssign, "-=", abs_pos, line_no, col);
                    i += 2;
                } else {
                    push_tok(out, TokenKind::Minus, "-", abs_pos, line_no, col);
                    i += 1;
                }
            }
            b'"' => {
                i += 1;
                while i < bytes.len() && bytes[i] != b'"' {
                    i += 1;
                }
                if i >= bytes.len() {
                    return Err(fmt_mark_error(
                        "E0004",
                        line_no,
                        col,
                        line_text,
                        "unterminated string literal",
                        abs_pos,
                        Some(abs_pos..line_start + line_text.trim_end_matches('\r').len()),
                    ));
                }
                i += 1;
                let text = &line_text[start..i];
                push_tok(out, TokenKind::String, text, abs_pos, line_no, col);
            }
            d if d.is_ascii_digit() => {
                i += 1;
                if d == b'0' && i < bytes.len() && (bytes[i] == b'x' || bytes[i] == b'X') {
                    i += 1;
                    while i < bytes.len() && (bytes[i].is_ascii_hexdigit() || bytes[i] == b'_') {
                        i += 1;
                    }
                } else {
                    while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'_') {
                        i += 1;
                    }
                    if i + 1 < bytes.len() && bytes[i] == b'.' && bytes[i + 1].is_ascii_digit() {
                        i += 1;
                        while i < bytes.len() && (bytes[i].is_ascii_digit() || bytes[i] == b'_') {
                            i += 1;
                        }
                    }
                }
                // Byte comparison: `i` is a byte offset and the suffix bytes
                // are ASCII, so no `&str` slice may land inside a scalar.
                if bytes.get(i..i + 2) == Some(b"fx".as_slice()) {
                    i += 2;
                } else if matches!(
                    bytes.get(i..i + 3),
                    Some(b"i32") | Some(b"u32") | Some(b"f64")
                ) {
                    i += 3;
                }
                push_tok(
                    out,
                    TokenKind::Num,
                    &line_text[start..i],
                    abs_pos,
                    line_no,
                    col,
                );
            }
            b'b' if bytes.get(i + 1) == Some(&b'\'') => {
                let end = scan_ascii_byte_literal(line_text, start, line_no, line_start)?;
                push_tok(
                    out,
                    TokenKind::ByteLit,
                    &line_text[start..end],
                    abs_pos,
                    line_no,
                    col,
                );
                i = end;
            }
            a if a.is_ascii_alphabetic() => {
                i += 1;
                while i < bytes.len() && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_') {
                    i += 1;
                }
                let text = &line_text[start..i];
                let kind = match text {
                    "fn" => TokenKind::KwFn,
                    "requires" => TokenKind::KwRequires,
                    "ensures" => TokenKind::KwEnsures,
                    "invariant" => TokenKind::KwInvariant,
                    "record" => TokenKind::KwRecord,
                    "schema" => TokenKind::KwSchema,
                    "enum" => TokenKind::KwEnum,
                    "const" => TokenKind::KwConst,
                    "trait" => TokenKind::KwTrait,
                    "impl" => TokenKind::KwImpl,
                    "let" => TokenKind::KwLet,
                    "for" => TokenKind::KwFor,
                    "in" => TokenKind::KwIn,
                    "guard" => TokenKind::KwGuard,
                    "if" => TokenKind::KwIf,
                    "else" => TokenKind::KwElse,
                    "is" => TokenKind::KwIs,
                    "when" => TokenKind::KwWhen,
                    "while" => TokenKind::KwWhile,
                    "loop" => TokenKind::KwLoop,
                    "break" => TokenKind::KwBreak,
                    "continue" => TokenKind::KwContinue,
                    "where" => TokenKind::KwWhere,
                    "with" => TokenKind::KwWith,
                    "return" => TokenKind::KwReturn,
                    "match" => TokenKind::KwMatch,
                    "true" => TokenKind::KwTrue,
                    "false" => TokenKind::KwFalse,
                    "System" => TokenKind::KwSystem,
                    "Entity" => TokenKind::KwEntity,
                    "Law" => TokenKind::KwLaw,
                    "When" => TokenKind::KwWhen,
                    "Pulse" => TokenKind::KwPulse,
                    "Profile" => TokenKind::KwProfile,
                    "Import" => TokenKind::KwImport,
                    "ref" => TokenKind::KwRef,
                    "quad" => TokenKind::TyQuad,
                    "bool" => TokenKind::TyBool,
                    "i32" => TokenKind::TyI32,
                    "u32" => TokenKind::TyU32,
                    "fx" => TokenKind::TyFx,
                    "f64" => TokenKind::TyF64,
                    "N" => TokenKind::QuadN,
                    "F" => TokenKind::QuadF,
                    "T" => TokenKind::QuadT,
                    "S" => TokenKind::QuadS,
                    _ => TokenKind::Ident,
                };
                push_tok(out, kind, text, abs_pos, line_no, col);
            }
            _ => {
                // Decode the whole UTF-8 scalar: `c` is only its first byte.
                let unexpected = line_text.get(start..).and_then(|rest| rest.chars().next());
                return Err(fmt_mark_error(
                    "E0001",
                    line_no,
                    col,
                    line_text,
                    &format!("unexpected character '{}'", unexpected.unwrap_or(c as char)),
                    abs_pos,
                    unexpected.map(|ch| abs_pos..abs_pos + ch.len_utf8()),
                ));
            }
        }
    }
    Ok(())
}

/// S2043 CP07: scans the ASCII byte literal starting with `b'` at `start` and
/// returns the byte index just past its closing quote. Grammar:
/// `b'` (printable ASCII 0x20..=0x7E except `'` and `\` | `\0` `\t` `\n` `\r`
/// `\'` `\\` `\"` | `\x` HEX HEX with value <= 0x7F) `'`. Every malformed
/// spelling is an `E0001` anchored at the offending character, or at the
/// whole partial literal when the line ends first.
fn scan_ascii_byte_literal(
    line_text: &str,
    start: usize,
    line_no: u32,
    line_start: usize,
) -> Result<usize, LexFailure> {
    let line_end = line_text.trim_end_matches('\r').len();
    let fail_at = |at: usize, detail: &str| {
        if at >= line_end {
            return fmt_mark_error(
                "E0001",
                line_no,
                (start + 1) as u32,
                line_text,
                "invalid byte literal: unterminated",
                line_start + start,
                Some(line_start + start..line_start + line_end),
            );
        }
        let ch = line_text[at..].chars().next().unwrap_or('\u{fffd}');
        fmt_mark_error(
            "E0001",
            line_no,
            (at + 1) as u32,
            line_text,
            &format!("invalid byte literal: {detail}"),
            line_start + at,
            Some(line_start + at..line_start + at + ch.len_utf8()),
        )
    };
    let bytes = line_text.as_bytes();
    let content = start + 2;
    let byte_at = |at: usize| (at < line_end).then(|| bytes[at]);
    let after_content = match byte_at(content) {
        None => return Err(fail_at(content, "")),
        Some(b'\'') => return Err(fail_at(content, "empty")),
        Some(b'\\') => match byte_at(content + 1) {
            None => return Err(fail_at(content + 1, "")),
            Some(b'0' | b't' | b'n' | b'r' | b'\'' | b'\\' | b'"') => content + 2,
            Some(b'x') => {
                let hi = content + 2;
                match byte_at(hi) {
                    None => return Err(fail_at(hi, "")),
                    Some(b'0'..=b'7') => {}
                    Some(d) if d.is_ascii_hexdigit() => {
                        return Err(fail_at(hi, "\\x escape above \\x7F is not ASCII"))
                    }
                    Some(_) => {
                        let found = line_text[hi..].chars().next().unwrap_or('\u{fffd}');
                        return Err(fail_at(hi, &format!("expected hex digit, found '{found}'")));
                    }
                }
                match byte_at(hi + 1) {
                    None => return Err(fail_at(hi + 1, "")),
                    Some(d) if d.is_ascii_hexdigit() => hi + 2,
                    Some(_) => {
                        let found = line_text[hi + 1..].chars().next().unwrap_or('\u{fffd}');
                        return Err(fail_at(
                            hi + 1,
                            &format!("expected hex digit, found '{found}'"),
                        ));
                    }
                }
            }
            Some(_) => {
                let found = line_text[content + 1..]
                    .chars()
                    .next()
                    .unwrap_or('\u{fffd}');
                return Err(fail_at(content + 1, &format!("unknown escape '\\{found}'")));
            }
        },
        Some(c) if (0x20..=0x7E).contains(&c) => content + 1,
        Some(c) if c.is_ascii() => {
            return Err(fail_at(
                content,
                &format!("control character U+{c:04X} must be escaped"),
            ))
        }
        Some(_) => {
            let found = line_text[content..].chars().next().unwrap_or('\u{fffd}');
            return Err(fail_at(content, &format!("non-ASCII character '{found}'")));
        }
    };
    match byte_at(after_content) {
        Some(b'\'') => Ok(after_content + 1),
        None => Err(fail_at(after_content, "")),
        Some(_) => {
            let found = line_text[after_content..]
                .chars()
                .next()
                .unwrap_or('\u{fffd}');
            Err(fail_at(
                after_content,
                &format!("expected closing quote, found '{found}'"),
            ))
        }
    }
}

/// S2043 CP07: the value of a byte literal token the lexer admitted.
pub fn ascii_byte_literal_value(text: &str) -> Option<u32> {
    let inner = text.strip_prefix("b'")?.strip_suffix('\'')?;
    let value = match inner.as_bytes() {
        [c] => *c as u32,
        [b'\\', b'0'] => 0,
        [b'\\', b't'] => 9,
        [b'\\', b'n'] => 10,
        [b'\\', b'r'] => 13,
        [b'\\', c @ (b'\'' | b'\\' | b'"')] => *c as u32,
        [b'\\', b'x', _, _] => u32::from_str_radix(&inner[2..], 16).ok()?,
        _ => return None,
    };
    (value <= 0x7F).then_some(value)
}

fn compute_indent(line: &str) -> usize {
    let mut n = 0usize;
    for b in line.as_bytes() {
        if *b == b' ' {
            n += 1;
        } else if *b == b'\t' {
            n += 4;
        } else {
            break;
        }
    }
    n
}

fn leading_indent_bytes(line: &str) -> usize {
    line.bytes()
        .take_while(|b| *b == b' ' || *b == b'\t')
        .count()
}

fn line_is_blank_or_comment(line: &str) -> bool {
    let t = line.trim_start_matches([' ', '\t', '\r']);
    t.is_empty() || t.starts_with("//") || t.starts_with('#')
}

pub fn lex_tokens(input: &str) -> Result<Vec<Token>, FrontendError> {
    lex_tokens_with_authority(input).map_err(|failure| failure.error)
}

/// SSF-09 C1A: the one lexer, exposing its producer-owned failure record
/// instead of only the legacy rendered [`FrontendError`].
pub fn lex_tokens_with_authority(input: &str) -> Result<Vec<Token>, LexFailure> {
    let mut out = Vec::new();
    let mut indent_stack: Vec<usize> = vec![0];
    let mut line_no: u32 = 1;
    let mut line_start = 0usize;
    let mut continuation_depth = 0usize;
    let mut continuation_after_arrow = false;

    for raw in input.split_inclusive('\n') {
        let has_nl = raw.ends_with('\n');
        let line_text = if has_nl {
            &raw[..raw.len().saturating_sub(1)]
        } else {
            raw
        };

        let significant = !line_is_blank_or_comment(line_text);
        if significant && continuation_depth == 0 && !continuation_after_arrow {
            let indent = compute_indent(line_text);
            let current = *indent_stack.last().unwrap_or(&0);
            if indent > current {
                push_tok(
                    &mut out,
                    TokenKind::Indent,
                    "<INDENT>",
                    line_start,
                    line_no,
                    1,
                );
                indent_stack.push(indent);
            } else if indent < current {
                while indent < *indent_stack.last().unwrap_or(&0) {
                    let _ = indent_stack.pop();
                    push_tok(
                        &mut out,
                        TokenKind::Dedent,
                        "<DEDENT>",
                        line_start,
                        line_no,
                        1,
                    );
                }
                if indent != *indent_stack.last().unwrap_or(&0) {
                    return Err(fmt_mark_error(
                        "E0101",
                        line_no,
                        1,
                        line_text,
                        "Bad Indent",
                        line_start,
                        Some(line_start..line_start + leading_indent_bytes(line_text)),
                    ));
                }
            }
        }

        let before = out.len();
        tokenize_line(line_text, line_no, line_start, &mut out)?;
        let line_tokens = &out[before..];
        for tok in line_tokens {
            match tok.kind {
                TokenKind::LParen => continuation_depth += 1,
                TokenKind::RParen => continuation_depth = continuation_depth.saturating_sub(1),
                _ => {}
            }
        }
        continuation_after_arrow = line_tokens
            .last()
            .map(|t| matches!(t.kind, TokenKind::Implies))
            .unwrap_or(false);

        push_tok(
            &mut out,
            TokenKind::Newline,
            "\\n",
            line_start + line_text.len(),
            line_no,
            (line_text.len() + 1) as u32,
        );

        line_start += raw.len();
        line_no += 1;
    }

    while indent_stack.len() > 1 {
        let _ = indent_stack.pop();
        push_tok(
            &mut out,
            TokenKind::Dedent,
            "<DEDENT>",
            input.len(),
            line_no,
            1,
        );
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lexer_smoke() {
        let src = "Entity E:\n    state x: quad\nLaw \"L\" [priority 1]:\n    When true -> System.recovery()\n";
        let toks = lex_tokens(src).expect("frontend lexer");
        assert!(toks.iter().any(|t| t.kind == TokenKind::KwEntity));
        assert!(toks.iter().any(|t| t.kind == TokenKind::Indent));
    }

    #[test]
    fn unexpected_non_ascii_character_is_reported_whole() {
        let src = "fn main() {\n    let \u{e9} = 1;\n}\n";
        let failure = lex_tokens_with_authority(src).unwrap_err();
        assert_eq!(failure.code, "E0001");
        assert_eq!(failure.detail, "unexpected character '\u{e9}'");
        assert_eq!(&src[failure.range.unwrap()], "\u{e9}");
    }

    /// S2043 CP01: the numeric suffix scan works on byte offsets, so a
    /// multibyte scalar after the digits must produce the ordinary E0001
    /// for that scalar, never a panic from a non-char-boundary slice.
    #[test]
    fn numeric_suffix_scan_is_utf8_safe() {
        let heads = ["1", "61", "1.0", "0x3D", "1_000"];
        let tails = [
            "", "f", "fx", "fu", "u", "u3", "u32", "i", "i3", "i32", "f6", "f64",
        ];
        // 2-, 3- and 4-byte scalars.
        let scalars = ["\u{e9}", "\u{20ac}", "\u{1f642}"];
        let afters = ["", ";", " ", "\u{e9}", "\u{1f642}"];
        for head in heads {
            for tail in tails {
                for scalar in scalars {
                    for after in afters {
                        let src = format!("let x = {head}{tail}{scalar}{after}");
                        let failure = lex_tokens_with_authority(&src)
                            .expect_err(&format!("{src:?} must be rejected"));
                        assert_eq!(failure.code, "E0001", "{src:?}");
                        assert_eq!(
                            failure.detail,
                            format!("unexpected character '{scalar}'"),
                            "{src:?}"
                        );
                        let range = failure.range.expect("E0001 carries a range");
                        assert_eq!(&src[range], scalar, "{src:?}");
                    }
                }
            }
        }
    }

    /// S2043 CP01: valid numeric literals keep their token text and byte
    /// position after the UTF-8-safe suffix scan.
    #[test]
    fn valid_numeric_literals_keep_text_and_position() {
        for lit in [
            "7", "61u32", "2i32", "1.5f64", "1.0fx", "3fx", "0x3Du32", "0x3D", "1_000", "1_000u32",
        ] {
            for after in ["", ";", " ", ")", "\n"] {
                let src = format!("let x = {lit}{after}");
                let toks = lex_tokens(&src).expect("valid literal must lex");
                let num = toks
                    .iter()
                    .find(|t| t.kind == TokenKind::Num)
                    .expect("numeric token");
                assert_eq!(num.text, lit, "{src:?}");
                assert_eq!(num.pos, 8, "{src:?}");
            }
        }
    }

    #[test]
    fn lexer_recognizes_relational_operator_pairs() {
        let src = "fn main() { let ok: bool = 3 <= 4 && 5 >= 2; return; }";
        let toks = lex_tokens(src).expect("frontend lexer");
        let kinds: Vec<_> = toks.iter().map(|t| t.kind).collect();
        assert!(kinds.contains(&TokenKind::Le));
        assert!(kinds.contains(&TokenKind::Ge));
    }
}
