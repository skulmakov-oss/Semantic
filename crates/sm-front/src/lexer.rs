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

fn fmt_mark_error(
    code: &str,
    line: u32,
    col: u32,
    line_text: &str,
    detail: &str,
    pos: usize,
) -> FrontendError {
    let mut caret = String::new();
    let spaces = col.saturating_sub(1) as usize;
    for _ in 0..spaces {
        caret.push(' ');
    }
    caret.push('^');
    FrontendError {
        pos,
        message: format!(
            "error[{code}]: {detail}\n --> <input>:{line}:{col}\n  |\n{line:>2} | {line_text}\n  | {caret}"
        ),
    }
}

fn tokenize_line(
    line_text: &str,
    line_no: u32,
    line_start: usize,
    out: &mut Vec<Token>,
) -> Result<(), FrontendError> {
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
                if i + 2 <= bytes.len() && &line_text[i..i + 2] == "fx" {
                    i += 2;
                } else if i + 3 <= bytes.len() {
                    let suffix = &line_text[i..i + 3];
                    if matches!(suffix, "i32" | "u32" | "f64") {
                        i += 3;
                    }
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
                return Err(fmt_mark_error(
                    "E0001",
                    line_no,
                    col,
                    line_text,
                    &format!("unexpected character '{}'", c as char),
                    abs_pos,
                ));
            }
        }
    }
    Ok(())
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

fn line_is_blank_or_comment(line: &str) -> bool {
    let t = line.trim_start_matches([' ', '\t', '\r']);
    t.is_empty() || t.starts_with("//") || t.starts_with('#')
}

pub fn lex_tokens(input: &str) -> Result<Vec<Token>, FrontendError> {
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
    fn lexer_recognizes_relational_operator_pairs() {
        let src = "fn main() { let ok: bool = 3 <= 4 && 5 >= 2; return; }";
        let toks = lex_tokens(src).expect("frontend lexer");
        let kinds: Vec<_> = toks.iter().map(|t| t.kind).collect();
        assert!(kinds.contains(&TokenKind::Le));
        assert!(kinds.contains(&TokenKind::Ge));
    }
}
