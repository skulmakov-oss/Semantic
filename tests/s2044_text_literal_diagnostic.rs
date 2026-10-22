//! S2044: executable text literal admission precedes SemCode emission.
use std::path::Path;
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

struct TempDir(std::path::PathBuf);

impl TempDir {
    fn new() -> Self {
        let dir = std::env::temp_dir().join(format!(
            "s2044_{}_{}_{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT_DIR.fetch_add(1, Ordering::SeqCst)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        Self(dir)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn smc(dir: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_smc"))
        .current_dir(dir)
        .args(args)
        .output()
        .expect("run smc")
}

fn output_text(output: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn source(content: &str) -> String {
    format!(
        "// UTF-8 prefix: é\nfn main() {{\n    let s: text = \"{content}\";\n    assert(text_len(s) == {}u32);\n    return;\n}}\n",
        content.len()
    )
}

#[test]
fn oversized_literals_have_source_diagnostics_and_no_artifact() {
    for content in [
        "a".repeat(1023),
        "a".repeat(1024),
        format!("{}a", "é".repeat(511)),
        "é".repeat(512),
    ] {
        let dir = TempDir::new();
        let src = source(&content);
        std::fs::write(dir.path().join("main.sm"), &src).unwrap();
        let literal = format!("\"{content}\"");
        let start = src.find(&literal).unwrap();
        let message = format!(
            "text literal exceeds the maximum supported size: {} UTF-8 content bytes; maximum 1022 bytes",
            content.len()
        );
        let mut previous = None;
        for _ in 0..3 {
            let out = smc(dir.path(), &["check", "main.sm", "--format", "json"]);
            assert_eq!(out.status.code(), Some(1), "{}", output_text(&out));
            assert!(out.stderr.is_empty(), "{}", output_text(&out));
            if let Some(bytes) = &previous {
                assert_eq!(bytes, &out.stdout, "diagnostics are deterministic");
            }
            previous = Some(out.stdout.clone());
            let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
            assert_eq!(report["schema"], "semantic.diagnostics");
            assert_eq!(report["schema_version"], 1);
            assert_eq!(report["status"], "failed");
            let diag = &report["diagnostics"][0];
            assert_eq!(diag["code"], "E0005");
            assert_eq!(diag["family"], "frontend");
            assert_eq!(diag["severity"], "error");
            assert_eq!(diag["message"], message);
            assert_eq!(diag["source"], 0);
            assert_eq!(diag["range"]["start"]["offset"], start);
            assert_eq!(diag["range"]["end"]["offset"], start + literal.len());
            assert_eq!(diag["range"]["start"]["line"], 3);
            assert_eq!(diag["range"]["start"]["column"], 19);
            assert_eq!(diag["range"]["end"]["column"], 19 + literal.chars().count());
            assert_eq!(report["sources"][0]["path"], "main.sm");
            assert_eq!(report["sources"][0]["identity"], serde_json::Value::Null);
            assert!(!output_text(&out).contains(dir.path().to_str().unwrap()));
        }
        let human = smc(dir.path(), &["check", "main.sm", "--format", "human"]);
        assert_eq!(human.status.code(), Some(1));
        let human = output_text(&human);
        assert!(human.contains("E0005"), "{human}");
        assert!(human.contains(&message), "{human}");
        assert!(human.contains(&literal), "{human}");
        assert!(!human.contains("internal compiler error"), "{human}");
        for level in ["O0", "O1"] {
            let out = smc(
                dir.path(),
                &["compile", "main.sm", "-o", "out.smc", "--opt-level", level],
            );
            assert_eq!(out.status.code(), Some(1));
            let text = output_text(&out);
            assert!(text.contains(&message), "{level}: {text}");
            assert!(!text.contains("internal compiler error"), "{text}");
            assert!(!dir.path().join("out.smc").exists());
        }
        let out = smc(
            dir.path(),
            &["run", "main.sm", "--profile", "pure", "--root", "."],
        );
        assert_eq!(out.status.code(), Some(1));
        assert!(
            output_text(&out).contains(&message),
            "{}",
            output_text(&out)
        );
        assert!(!output_text(&out).contains("internal compiler error"));
    }
}

#[test]
fn admitted_literals_preserve_bytes_and_execute_at_both_optimization_levels() {
    // Pin the source spelling bound to the authoritative format limit without
    // adding a format dependency to the frontend crate.
    assert_eq!(sm_format::semcode_decode::MAX_STRING_LEN, 1022 + 2);
    for content in [
        String::new(),
        "a".to_owned(),
        "a".repeat(1021),
        "a".repeat(1022),
        format!("{}a", "é".repeat(510)),
        "é".repeat(511),
    ] {
        let dir = TempDir::new();
        std::fs::write(dir.path().join("main.sm"), source(&content)).unwrap();
        let out = smc(dir.path(), &["check", "main.sm"]);
        assert!(out.status.success(), "{}", output_text(&out));
        for level in ["O0", "O1"] {
            let mut previous = None;
            for _ in 0..3 {
                let out = smc(
                    dir.path(),
                    &["compile", "main.sm", "-o", "out.smc", "--opt-level", level],
                );
                assert!(out.status.success(), "{}", output_text(&out));
                let bytes = std::fs::read(dir.path().join("out.smc")).unwrap();
                let decoded = sm_format::semcode_decode::decode_semcode(&bytes).unwrap();
                assert!(decoded
                    .functions
                    .iter()
                    .any(|f| f.strings.contains(&format!("\"{content}\""))));
                if let Some(expected) = &previous {
                    assert_eq!(expected, &bytes, "{level}: deterministic artifact");
                }
                previous = Some(bytes);
                for command in ["verify", "run-smc"] {
                    let out = smc(dir.path(), &[command, "out.smc"]);
                    assert!(out.status.success(), "{command}: {}", output_text(&out));
                }
            }
        }
    }
}

#[test]
fn import_path_strings_do_not_inherit_executable_literal_limit() {
    let path = "a".repeat(1025);
    let src = format!("Import \"{path}\"\nfn main() {{ return; }}\n");
    sm_front::parse_program(&src).expect("a path token is not an executable TextLiteral");
}

#[test]
fn imported_literal_limit_preserves_bundle_diagnostic_boundary() {
    for size in [1023, 1022] {
        let dir = TempDir::new();
        std::fs::write(
            dir.path().join("main.sm"),
            "Import \"helper.sm\"\nfn main() { helper(); return; }\n",
        )
        .unwrap();
        std::fs::write(
            dir.path().join("helper.sm"),
            source(&"a".repeat(size)).replacen("fn main()", "fn helper()", 1),
        )
        .unwrap();
        let mut previous = None;
        for _ in 0..3 {
            let out = smc(dir.path(), &["check", "main.sm", "--format", "json"]);
            assert_eq!(out.status.code(), Some(i32::from(size > 1022)));
            assert!(out.stderr.is_empty(), "{}", output_text(&out));
            assert!(!output_text(&out).contains("internal compiler error"));
            if let Some(bytes) = &previous {
                assert_eq!(bytes, &out.stdout, "deterministic bundle diagnostics");
            }
            previous = Some(out.stdout.clone());
            let report: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
            assert_eq!(report["schema"], "semantic.diagnostics");
            assert_eq!(report["schema_version"], 1);
            if size > 1022 {
                assert_eq!(report["status"], "failed");
                let diagnostics = report["diagnostics"].as_array().unwrap();
                assert_eq!(diagnostics.len(), 1);
                let diag = &diagnostics[0];
                assert_eq!(diag["code"], "E0009");
                assert_eq!(diag.get("source"), Some(&serde_json::Value::Null));
                assert_eq!(diag.get("range"), Some(&serde_json::Value::Null));
                let message = diag["message"].as_str().unwrap();
                assert!(message.contains("helper.sm"), "{message}");
                assert!(message.contains("text literal exceeds the maximum supported size: 1023 UTF-8 content bytes; maximum 1022 bytes"), "{message}");
            } else {
                assert_eq!(report["diagnostics"].as_array().unwrap().len(), 0);
            }
        }
        for level in ["O0", "O1"] {
            let out = smc(
                dir.path(),
                &["compile", "main.sm", "-o", "out.smc", "--opt-level", level],
            );
            assert_eq!(
                out.status.code(),
                Some(i32::from(size > 1022)),
                "{level}: {}",
                output_text(&out)
            );
            assert!(!output_text(&out).contains("internal compiler error"));
            if size > 1022 {
                assert!(!dir.path().join("out.smc").exists());
            } else {
                for command in ["verify", "run-smc"] {
                    let out = smc(dir.path(), &[command, "out.smc"]);
                    assert!(out.status.success(), "{command}: {}", output_text(&out));
                }
            }
        }
    }
}
