use semantic_language::frontend::{
    compile_program_to_semcode, lex, lower_logos_laws_to_ir, parse_logos_program, TokenKind,
};

fn read_text(path: &str) -> String {
    let raw = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read '{}': {}", path, e));
    normalize_newlines(&raw)
}

fn write_text(path: &str, text: &str) {
    std::fs::write(path, text).unwrap_or_else(|e| panic!("write '{}': {}", path, e))
}

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

fn update_mode() -> bool {
    std::env::var("SM_UPDATE_SNAPSHOTS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn assert_snapshot(path: &str, got: &str) {
    if update_mode() {
        write_text(path, got);
        return;
    }
    let expected = normalize_newlines(&read_text(path));
    let got = normalize_newlines(got);
    assert_eq!(expected, got, "snapshot mismatch at {}", path);
}

fn tokens_snapshot(input: &str) -> String {
    let toks = lex(input).expect("lex");
    let mut out = String::new();
    for t in toks {
        let kind = match t.kind {
            TokenKind::Indent => "INDENT",
            TokenKind::Dedent => "DEDENT",
            TokenKind::Newline => "NEWLINE",
            _ => t.text.as_str(),
        };
        out.push_str(&format!(
            "{:>4}:{:<3} {:<10} {}\n",
            t.mark.line,
            t.mark.col,
            format!("{:?}", t.kind),
            kind
        ));
    }
    out
}

fn ast_snapshot(input: &str) -> String {
    let p = parse_logos_program(input).expect("logos parse");
    format!("{:#?}\n", p)
}

fn ir_snapshot(input: &str) -> String {
    let p = parse_logos_program(input).expect("logos parse");
    let ir = lower_logos_laws_to_ir(&p);
    format!("{:#?}\n", ir)
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

fn semcode_hash_snapshot(input: &str) -> String {
    let bytes = compile_program_to_semcode(input).expect("compile");
    format!("{:016x}\n", fnv1a64(&bytes))
}

#[test]
fn golden_tokens_indent() {
    let src = read_text("tests/golden_snapshots/indent/basic.sm");
    let got = tokens_snapshot(&src);
    assert_snapshot("tests/golden_snapshots/indent/basic.tokens", &got);
}

#[test]
fn golden_ast_logos() {
    let src = read_text("tests/golden_snapshots/parser/entity_law.sm");
    let got = ast_snapshot(&src);
    assert_snapshot("tests/golden_snapshots/parser/entity_law.ast", &got);
}

/// FA-02-042 / #1987: accepted Pulse/Profile lines are visible in the Logos
/// AST inspection projection next to the shared `Import` directive.
#[test]
fn golden_ast_logos_legacy_directives() {
    let src = read_text("tests/golden_snapshots/parser/legacy_directives.sm");
    let got = ast_snapshot(&src);
    assert_snapshot("tests/golden_snapshots/parser/legacy_directives.ast", &got);
}

#[test]
fn golden_ir_logos() {
    let src = read_text("tests/golden_snapshots/lowering/priorities.sm");
    let got = ir_snapshot(&src);
    assert_snapshot("tests/golden_snapshots/lowering/priorities.ir", &got);
}

#[test]
fn golden_emit_hash_stable() {
    let src = read_text("tests/golden_snapshots/emit/stable.sm");
    let got = semcode_hash_snapshot(&src);
    assert_snapshot("tests/golden_snapshots/emit/stable.hash", &got);
}
