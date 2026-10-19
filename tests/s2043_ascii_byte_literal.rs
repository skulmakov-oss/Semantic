//! S2043 CP07: the OWNER-approved ASCII byte literal `b'...'`.
//!
//! Grammar (docs/spec/syntax.md):
//!   AsciiByteLiteral = "b'" ( RawPrintableAscii | Escape ) "'"
//!   RawPrintableAscii = 0x20..=0x7E except `'` and `\`
//!   Escape = \0 \t \n \r \' \\ \" \xHH   (HH in 00..=7F)
//! The value is exactly a `u32` in 0..=127, decoded at compile time and lowered
//! through the existing `u32` literal path: no new type, opcode, SemCode
//! revision or VM behavior. Every malformed spelling is a deterministic
//! lexical `E0001`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn mk_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{}_{}_{}_{}",
        prefix,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos(),
        DIR_COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

fn smc(args: &[&str]) -> (i32, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(args)
        .output()
        .expect("run smc");
    (
        out.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn write(dir: &Path, name: &str, src: &str) -> String {
    let path = dir.join(format!("{name}.sm"));
    std::fs::write(&path, src).expect("write case");
    path.to_str().unwrap().to_string()
}

fn run_everywhere(dir: &Path, name: &str, src: &str) {
    let path = write(dir, name, src);
    let (code, stdout, stderr) = smc(&["check", &path, "--format", "json"]);
    assert_eq!(code, 0, "{name}: check: {stdout}{stderr}");
    for level in ["O0", "O1"] {
        let out = dir.join(format!("{name}.{level}.smc"));
        let out = out.to_str().unwrap();
        let (code, stdout, stderr) = smc(&["compile", &path, "-o", out, "--opt-level", level]);
        assert_eq!(code, 0, "{name} {level}: compile: {stdout}{stderr}");
        let (code, stdout, stderr) = smc(&["verify", out]);
        assert_eq!(code, 0, "{name} {level}: verify: {stdout}{stderr}");
        let (code, stdout, stderr) = smc(&["run-smc", out]);
        assert_eq!(code, 0, "{name} {level}: run: {stdout}{stderr}");
    }
}

/// Every source spelling with its value: 95 printable characters (raw where
/// the grammar allows it), every named escape, and all 128 `\xHH` forms.
fn spellings() -> Vec<(String, u32)> {
    let mut out = Vec::new();
    for byte in 0x20u8..=0x7E {
        let raw = match byte {
            b'\'' => "\\'".to_string(),
            b'\\' => "\\\\".to_string(),
            other => (other as char).to_string(),
        };
        out.push((raw, byte as u32));
    }
    for (escape, value) in [
        ("\\0", 0u32),
        ("\\t", 9),
        ("\\n", 10),
        ("\\r", 13),
        ("\\'", 39),
        ("\\\\", 92),
        ("\\\"", 34),
    ] {
        out.push((escape.to_string(), value));
    }
    for value in 0u32..=0x7F {
        out.push((format!("\\x{value:02X}"), value));
        out.push((format!("\\x{value:02x}"), value));
    }
    out
}

#[test]
fn every_admitted_spelling_has_its_ascii_value_at_o0_and_o1() {
    let dir = mk_temp_dir("s2043_cp07_values");
    let mut body = String::new();
    for (spelling, value) in spellings() {
        body.push_str(&format!("    assert(b'{spelling}' == {value}u32);\n"));
    }
    run_everywhere(
        &dir,
        "values",
        &format!("fn main() {{\n{body}    return;\n}}\n"),
    );
}

#[test]
fn byte_literal_is_exactly_u32() {
    let dir = mk_temp_dir("s2043_cp07_types");
    run_everywhere(
        &dir,
        "u32_uses",
        "fn is_eq(c: u32) -> bool {\n    return c == b'=';\n}\nfn main() {\n    const EQ: u32 = b'=';\n    let lt: u32 = b'<';\n    assert(is_eq(EQ));\n    assert(lt < EQ);\n    assert(b'a' + 1u32 == b'b');\n    assert(b'z' - b'a' == 25u32);\n    let c: u32 = 61u32;\n    let v: i32 = match c {\n        61u32 => { 1 }\n        _ => { 0 }\n    };\n    assert(v == 1);\n    return;\n}\n",
    );
    for (name, decl) in [
        ("as_i32", "let x: i32 = b'a';"),
        ("as_text", "let x: text = b'a';"),
        ("as_bool", "let x: bool = b'a';"),
        ("as_quad", "let x: quad = b'a';"),
        ("mixed_i32", "let x: u32 = b'a' + 1;"),
    ] {
        let path = write(
            &dir,
            name,
            &format!("fn main() {{\n    {decl}\n    let _ = x;\n    return;\n}}\n"),
        );
        let (code, stdout, stderr) = smc(&["check", &path, "--format", "json"]);
        assert_eq!(code, 1, "{name}: {stdout}{stderr}");
        assert!(stdout.contains("\"E0201\""), "{name}: {stdout}");
    }
}

#[test]
fn malformed_byte_literals_are_lexical_errors_with_exact_ranges() {
    let dir = mk_temp_dir("s2043_cp07_reject");
    // (literal text as it appears after `let x: u32 = `, offending text)
    let cases: &[(&str, &str)] = &[
        ("b''", "'"),
        ("b'ab'", "b"),
        ("b'\u{e9}'", "\u{e9}"),
        ("b'\u{1f642}'", "\u{1f642}"),
        ("b'\\q'", "q"),
        ("b'\\x80'", "8"),
        ("b'\\xFF'", "F"),
        ("b'\\xG1'", "G"),
        ("b'\\x7'", "'"),
        ("b'\t'", "\t"),
        // A character other than the closing quote after the content.
        ("b'a", ";"),
        ("b'\\'", ";"),
    ];
    for (i, (literal, offending)) in cases.iter().enumerate() {
        let src =
            format!("fn main() {{\n    let x: u32 = {literal};\n    let _ = x;\n    return;\n}}\n");
        let path = write(&dir, &format!("bad{i}"), &src);
        let (code, stdout, stderr) = smc(&["check", &path, "--format", "json"]);
        assert_eq!(code, 1, "{literal:?}: {stdout}{stderr}");
        let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let diag = &report["diagnostics"][0];
        assert_eq!(diag["code"], "E0001", "{literal:?}: {report}");
        assert!(
            diag["message"]
                .as_str()
                .unwrap_or("")
                .starts_with("invalid byte literal"),
            "{literal:?}: {report}"
        );
        let start = diag["range"]["start"]["offset"].as_u64().expect("range") as usize;
        let end = diag["range"]["end"]["offset"].as_u64().expect("range") as usize;
        assert_eq!(&src[start..end], *offending, "{literal:?}");
        // Same rejection, no panic, through compile.
        let out = dir.join(format!("bad{i}.smc"));
        let (code, _, stderr) = smc(&["compile", &path, "-o", out.to_str().unwrap()]);
        assert_eq!(code, 1, "{literal:?}");
        assert!(!stderr.contains("panicked"), "{literal:?}: {stderr}");
    }
}

#[test]
fn byte_literal_ending_the_line_is_unterminated() {
    let dir = mk_temp_dir("s2043_cp07_eol");
    for (i, literal) in ["b'a", "b'", "b'\\x4"].iter().enumerate() {
        let src = format!("fn main() {{\n    let x: u32 = {literal}\n}}\n");
        let path = write(&dir, &format!("eol{i}"), &src);
        let (code, stdout, _) = smc(&["check", &path, "--format", "json"]);
        assert_eq!(code, 1, "{literal:?}");
        let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
        let diag = &report["diagnostics"][0];
        assert_eq!(diag["code"], "E0001", "{report}");
        assert_eq!(
            diag["message"], "invalid byte literal: unterminated",
            "{report}"
        );
        let start = diag["range"]["start"]["offset"].as_u64().unwrap() as usize;
        let end = diag["range"]["end"]["offset"].as_u64().unwrap() as usize;
        assert_eq!(&src[start..end], *literal, "{literal:?}");
    }
}

#[test]
fn identifier_b_and_neighbouring_tokens_are_unchanged() {
    let dir = mk_temp_dir("s2043_cp07_ident");
    run_everywhere(
        &dir,
        "ident_b",
        "fn main() {\n    let b: i32 = 1;\n    let bb: i32 = b + 1;\n    let b2: i32 = bb;\n    assert(b2 == 2);\n    return;\n}\n",
    );
    // A `b` followed by whitespace and a quote is not a byte literal.
    let path = write(
        &dir,
        "spaced",
        "fn main() {\n    let b: u32 = 1u32;\n    let x: u32 = b '=';\n    let _ = x;\n    return;\n}\n",
    );
    let (code, stdout, _) = smc(&["check", &path, "--format", "json"]);
    assert_eq!(code, 1);
    assert!(stdout.contains("unexpected character '''"), "{stdout}");
}

#[test]
fn semcode_is_byte_identical_to_the_numeric_spelling() {
    let dir = mk_temp_dir("s2043_cp07_semcode");
    let pairs = [
        (
            "fn main() {\n    let x: u32 = b'=';\n    assert(x == b'=');\n    return;\n}\n",
            "fn main() {\n    let x: u32 = 61u32;\n    assert(x == 61u32);\n    return;\n}\n",
            "SEMCOD22",
            "SEMCOD22",
        ),
        (
            "fn main() {\n    let x: u32 = b'=' + 1u32;\n    assert(x == b'>');\n    return;\n}\n",
            "fn main() {\n    let x: u32 = 61u32 + 1u32;\n    assert(x == 62u32);\n    return;\n}\n",
            "SEMCOD23",
            "SEMCOD22",
        ),
    ];
    for (i, (byte_src, num_src, o0_magic, o1_magic)) in pairs.iter().enumerate() {
        let byte_path = write(&dir, &format!("byte{i}"), byte_src);
        let num_path = write(&dir, &format!("num{i}"), num_src);
        for (level, magic) in [("O0", o0_magic), ("O1", o1_magic)] {
            let a = dir.join(format!("byte{i}.{level}.smc"));
            let b = dir.join(format!("num{i}.{level}.smc"));
            for (src, out) in [(&byte_path, &a), (&num_path, &b)] {
                let (code, stdout, stderr) = smc(&[
                    "compile",
                    src,
                    "-o",
                    out.to_str().unwrap(),
                    "--opt-level",
                    level,
                ]);
                assert_eq!(code, 0, "{stdout}{stderr}");
            }
            let byte_artifact = std::fs::read(&a).unwrap();
            let num_artifact = std::fs::read(&b).unwrap();
            assert_eq!(
                byte_artifact, num_artifact,
                "pair {i} {level}: bytes differ"
            );
            assert_eq!(&byte_artifact[..8], magic.as_bytes(), "pair {i} {level}");
        }
    }
}

#[test]
fn formatter_preserves_byte_literal_tokens() {
    let dir = mk_temp_dir("s2043_cp07_fmt");
    let src = "fn main() {\n    let q: u32 = b'\\'';\n    let s: u32 = b' ';\n    let d: u32 = b'\"';\n    let h: u32 = b'\\x7F';\n    assert(q == 39u32);\n    assert(s == 32u32);\n    assert(d == 34u32);\n    assert(h == 127u32);\n    return;\n}\n";
    let path = write(&dir, "fmt", src);
    let (code, stdout, stderr) = smc(&["fmt", "--check", &path]);
    assert_eq!(code, 0, "{stdout}{stderr}");
    // The formatter only normalizes whitespace it can prove token-neutral
    // (trailing blanks, trailing empty lines); it never re-indents.
    let messy = dir.join("messy.sm");
    std::fs::write(&messy, format!("{}\n\n", src.replace(";\n", "; \t\n"))).unwrap();
    let (code, stdout, stderr) = smc(&["fmt", messy.to_str().unwrap()]);
    assert_eq!(code, 0, "{stdout}{stderr}");
    assert_eq!(std::fs::read_to_string(&messy).unwrap(), src);
    run_everywhere(&dir, "fmt_run", src);
}
