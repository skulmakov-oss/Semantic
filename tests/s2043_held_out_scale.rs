//! S2043 CP09: held-out inputs, resource characterization and hostile
//! inputs, generated deterministically in test code (no fixtures, no seeds
//! from the environment).
//!
//! These are NOT performance claims. They pin that, for the listed sizes and
//! shapes, the toolchain never panics, answers deterministically, attributes
//! diagnostics inside the source, runs valid programs at O0 and O1 within the
//! default `verified-local` quotas, fails quota exhaustion closed with stable
//! counters, and never executes unverified SemCode on the trusted route.
//! Run with `--nocapture` to print the measured counters.

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

/// A valid program padded with `//` comment lines to exactly `size` bytes.
fn padded_program(body: &str, size: usize) -> String {
    let mut src = format!("fn main() {{\n{body}    return;\n}}\n");
    assert!(src.len() <= size, "body too large for {size}");
    while src.len() < size {
        let room = size - src.len();
        if room < 4 {
            src.push_str(&" ".repeat(room - 1));
            src.push('\n');
            break;
        }
        let fill = room.min(80) - 1;
        let mut line = String::from("//");
        line.push_str(&"x".repeat(fill.saturating_sub(2)));
        line.push('\n');
        if line.len() > room {
            line = format!("{}\n", " ".repeat(room - 1));
        }
        src.push_str(&line);
    }
    assert_eq!(src.len(), size);
    src
}

/// A Unicode-heavy body: multibyte comments and text literals drawn from a
/// bounded set, so the function stays within the SemCode per-function
/// string-table limit (see `per_function_string_limit_fails_closed`).
fn unicode_program(size: usize) -> String {
    let mut body = String::from("    let mut t: text = \"\";\n");
    let mut i = 0;
    while body.len() + 250 < size {
        let k = i % 40;
        body.push_str(&format!(
            "    // \u{e9}\u{20ac}\u{1f642} {i}\n    t = \"\u{e9}\u{1f642}{k}\";\n    assert(t == \"\u{e9}\u{1f642}{k}\");\n"
        ));
        i += 1;
    }
    padded_program(&body, size)
}

/// An ASCII body whose statements reuse a bounded set of local names.
fn ascii_program(size: usize) -> String {
    let mut body = String::from("    let mut acc: i32 = 0;\n    let mut expected: i32 = 0;\n");
    let mut i = 0;
    while body.len() + 200 < size {
        body.push_str(&format!(
            "    acc = acc + {i};\n    expected = expected + {i};\n    assert(acc == expected);\n"
        ));
        i += 1;
    }
    padded_program(&body, size)
}

fn metrics(dir: &Path, path: &str, extra: &[&str]) -> serde_json::Value {
    let mut args = vec![
        "run",
        path,
        "--profile",
        "pure",
        "--root",
        dir.to_str().unwrap(),
        "--metrics",
        "json",
    ];
    args.extend_from_slice(extra);
    let (_, stdout, stderr) = smc(&args);
    let line = stdout
        .lines()
        .chain(stderr.lines())
        .find(|l| l.contains("semantic.run.metrics"))
        .unwrap_or_else(|| panic!("no metrics: {stdout}{stderr}"));
    let mut json: serde_json::Value = serde_json::from_str(line).expect("metrics json");
    json["nondeterministic"] = serde_json::Value::Null;
    json
}

/// check + compile/verify/run-smc at O0 and O1, plus counters under the
/// default envelope; returns the counters.
fn assert_valid(dir: &Path, name: &str, src: &str) -> serde_json::Value {
    let path = dir.join(format!("{name}.sm"));
    std::fs::write(&path, src).unwrap();
    let p = path.to_str().unwrap();
    let (code, stdout, stderr) = smc(&["check", p, "--format", "json"]);
    assert_eq!(code, 0, "{name}: {stdout}{stderr}");
    for level in ["O0", "O1"] {
        let out = dir.join(format!("{name}.{level}.smc"));
        let out = out.to_str().unwrap();
        let (code, stdout, stderr) = smc(&["compile", p, "-o", out, "--opt-level", level]);
        assert_eq!(code, 0, "{name} {level}: {stdout}{stderr}");
        assert_eq!(smc(&["verify", out]).0, 0, "{name} {level}: verify");
        let (code, stdout, stderr) = smc(&["run-smc", out]);
        assert_eq!(code, 0, "{name} {level}: {stdout}{stderr}");
    }
    let first = metrics(dir, p, &[]);
    let second = metrics(dir, p, &[]);
    assert_eq!(first, second, "{name}: counters must be deterministic");
    assert_eq!(first["outcome"], "ok", "{name}: {first}");
    println!(
        "{name:<28} bytes={:>6} steps={:>6} calls={:>4} peak_frames={:>3}",
        src.len(),
        first["counters"]["steps"],
        first["counters"]["calls"],
        first["counters"]["peak_frames"]
    );
    first
}

/// Rejected deterministically, without panic, with any range inside `src`.
fn assert_rejected(dir: &Path, name: &str, src: &str) -> String {
    let path = dir.join(format!("{name}.sm"));
    std::fs::write(&path, src).unwrap();
    let p = path.to_str().unwrap();
    let first = smc(&["check", p, "--format", "json"]);
    let second = smc(&["check", p, "--format", "json"]);
    assert_eq!(first, second, "{name}: deterministic");
    let (code, stdout, stderr) = first;
    assert_eq!(code, 1, "{name}: {stdout}{stderr}");
    assert!(!stderr.contains("panicked"), "{name}: {stderr}");
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let diag = &report["diagnostics"][0];
    if let Some(end) = diag["range"]["end"]["offset"].as_u64() {
        assert!(end as usize <= src.len(), "{name}: range outside source");
    }
    let out = dir.join(format!("{name}.smc"));
    let (code, _, stderr) = smc(&["compile", p, "-o", out.to_str().unwrap()]);
    assert_eq!(code, 1, "{name}: compile must reject");
    assert!(!stderr.contains("panicked"), "{name}: {stderr}");
    diag["code"].as_str().unwrap_or("").to_string()
}

#[test]
fn valid_held_out_programs_scale_across_sizes() {
    let dir = mk_temp_dir("s2043_cp09_valid");
    for size in [256usize, 1024, 4096, 16384] {
        assert_valid(&dir, &format!("ascii_{size}"), &ascii_program(size));
        assert_valid(&dir, &format!("unicode_{size}"), &unicode_program(size));
    }
    // Nested blocks and nested if-let over Option(Option(Option(i32))).
    let mut nested = String::from("    let mut depth: i32 = 0;\n");
    for _ in 0..16 {
        nested.push_str("    if true {\n");
    }
    nested.push_str("    depth = 16;\n");
    for _ in 0..16 {
        nested.push_str("    } else {\n    depth = -1;\n    }\n");
    }
    nested.push_str("    assert(depth == 16);\n");
    nested.push_str("    let o: Option(Option(Option(i32))) = Option::Some(Option::Some(Option::Some(7)));\n    let v: i32 = if let Option::Some(a) = o {\n        if let Option::Some(b) = a {\n            if let Option::Some(c) = b { c } else { -3 }\n        } else {\n            -2\n        }\n    } else {\n        -1\n    };\n    assert(v == 7);\n");
    assert_valid(&dir, "nested", &padded_program(&nested, 4096));
}

/// Held-out finding (S2043 CP09): one function may reference at most 256
/// distinct strings (local names and text constants) in its SemCode string
/// table. Beyond that, `check` accepts the program but `compile` stops with
/// the producer's deterministic "SemCode producer limit" error (currently
/// worded as an internal compiler error) at both levels, with no artifact
/// and no panic. Pinned as an observed limit, not fixed here: lifting it is
/// a SemCode format change.
#[test]
fn per_function_string_limit_fails_closed() {
    let dir = mk_temp_dir("s2043_cp09_strings");
    let mut body = String::new();
    for i in 0..300 {
        body.push_str(&format!("    let v{i}: i32 = {i};\n"));
    }
    let src = format!("fn main() {{\n{body}    return;\n}}\n");
    let path = dir.join("many_locals.sm");
    std::fs::write(&path, &src).unwrap();
    let p = path.to_str().unwrap();
    assert_eq!(smc(&["check", p, "--format", "json"]).0, 0);
    for level in ["O0", "O1"] {
        let out = dir.join(format!("many_locals.{level}.smc"));
        let first = smc(&[
            "compile",
            p,
            "-o",
            out.to_str().unwrap(),
            "--opt-level",
            level,
        ]);
        let second = smc(&[
            "compile",
            p,
            "-o",
            out.to_str().unwrap(),
            "--opt-level",
            level,
        ]);
        assert_eq!(first, second, "{level}: deterministic");
        assert_eq!(first.0, 1, "{level}");
        assert!(
            first.2.contains("SemCode producer limit"),
            "{level}: {}",
            first.2
        );
        assert!(
            first.2.contains("exceed the format maximum of 256"),
            "{}",
            first.2
        );
        assert!(!first.2.contains("panicked"), "{}", first.2);
        assert!(!out.exists(), "{level}: no artifact");
    }
}

#[test]
fn hostile_and_truncated_inputs_fail_closed() {
    let dir = mk_temp_dir("s2043_cp09_hostile");
    let valid = ascii_program(256);
    for size in [0usize, 1, 22, 23] {
        let code = assert_rejected(&dir, &format!("trunc_{size}"), &valid[..size]);
        assert!(code.starts_with('E'), "trunc_{size}: {code}");
    }
    // A 4 KiB identifier followed by a multibyte scalar.
    let long_ident = format!(
        "fn main() {{\n    let {}\u{1f642} = 1;\n}}\n",
        "x".repeat(4096)
    );
    assert_eq!(assert_rejected(&dir, "long_ident", &long_ident), "E0001");
    // Suffix-adjacent Unicode as the last statement of a ~16 KiB function.
    let mut body = String::from("    let mut acc: i32 = 0;\n");
    let mut i = 0;
    while body.len() < 16_000 {
        body.push_str(&format!("    acc = acc + {i};\n"));
        i += 1;
    }
    body.push_str("    let z: i32 = 1\u{1f642};\n");
    let tail = format!("fn main() {{\n{body}    return;\n}}\n");
    assert_eq!(assert_rejected(&dir, "unicode_at_end", &tail), "E0001");
    let path = dir.join("unicode_at_end.sm");
    let (_, stdout, _) = smc(&["check", path.to_str().unwrap(), "--format", "json"]);
    let report: serde_json::Value = serde_json::from_str(&stdout).unwrap();
    let start = report["diagnostics"][0]["range"]["start"]["offset"]
        .as_u64()
        .unwrap() as usize;
    assert_eq!(&tail[start..start + 4], "\u{1f642}");
    // A 4 KiB numeric literal.
    let huge = format!(
        "fn main() {{\n    let n: i32 = {};\n    let _ = n;\n    return;\n}}\n",
        "9".repeat(4096)
    );
    assert_rejected(&dir, "huge_literal", &huge);
}

#[test]
fn import_failures_are_deterministic() {
    let dir = mk_temp_dir("s2043_cp09_imports");
    std::fs::write(
        dir.join("missing.sm"),
        "Import \"nowhere.sm\"\nfn main() {\n    return;\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("a.sm"),
        "Import \"b.sm\"\nfn from_a() -> i32 {\n    return 1;\n}\nfn main() {\n    return;\n}\n",
    )
    .unwrap();
    std::fs::write(
        dir.join("b.sm"),
        "Import \"a.sm\"\nfn from_b() -> i32 {\n    return 2;\n}\n",
    )
    .unwrap();
    for file in ["missing.sm", "a.sm"] {
        let path = dir.join(file);
        let p = path.to_str().unwrap();
        let first = smc(&["check", p, "--format", "json"]);
        let second = smc(&["check", p, "--format", "json"]);
        assert_eq!(first, second, "{file}");
        assert_eq!(first.0, 1, "{file}: {}{}", first.1, first.2);
        assert!(!first.2.contains("panicked"), "{file}");
        let out = dir.join(format!("{file}.smc"));
        assert_eq!(
            smc(&["compile", p, "-o", out.to_str().unwrap()]).0,
            1,
            "{file}"
        );
        assert!(!out.exists(), "{file}: no artifact");
    }
}

#[test]
fn call_decomposition_and_quota_exhaustion_are_characterized() {
    let dir = mk_temp_dir("s2043_cp09_quota");
    let inline = "fn main() {\n    let mut total: i32 = 0;\n    for i in 0..200 {\n        total += i * 2;\n    }\n    assert(total == 39800);\n    return;\n}\n";
    let helper = "fn twice(x: i32) -> i32 {\n    return x * 2;\n}\nfn main() {\n    let mut total: i32 = 0;\n    for i in 0..200 {\n        total += twice(i);\n    }\n    assert(total == 39800);\n    return;\n}\n";
    let a = assert_valid(&dir, "inline_loop", inline);
    let b = assert_valid(&dir, "helper_loop", helper);
    assert_eq!(a["counters"]["calls"], 0);
    assert_eq!(
        b["counters"]["calls"], 200,
        "one charged call per iteration"
    );

    // The same loop scaled past the default 100000-step quota fails closed
    // with stable counters; a larger quota is an explicit, reported
    // parameter of the trusted-compiler envelope, never a default change.
    let big = "fn main() {\n    let mut total: i32 = 0;\n    for i in 0..60000 {\n        total += i;\n    }\n    let _ = total;\n    return;\n}\n";
    let path = dir.join("big.sm");
    std::fs::write(&path, big).unwrap();
    let p = path.to_str().unwrap();
    let first = metrics(&dir, p, &[]);
    let second = metrics(&dir, p, &[]);
    assert_eq!(first, second, "quota failure must be deterministic");
    assert_ne!(first["outcome"], "ok", "{first}");
    assert_eq!(first["effective_quotas"]["max_steps"], 100000, "{first}");
    assert!(
        first["quota_exceeded"].to_string().contains("Steps"),
        "{first}"
    );
    println!("quota default: {}", first["counters"]);

    let raised = metrics(
        &dir,
        p,
        &["--envelope", "trusted-compiler", "--max-steps", "2000000"],
    );
    assert_eq!(raised["outcome"], "ok", "{raised}");
    assert_eq!(raised["effective_quotas"]["max_steps"], 2000000, "{raised}");
    println!("quota raised (explicit): {}", raised["counters"]);
}

#[test]
fn relabelled_semcode_is_never_executed_on_the_trusted_route() {
    let dir = mk_temp_dir("s2043_cp09_verify");
    let path = dir.join("u32.sm");
    std::fs::write(
        &path,
        "fn add(a: u32, b: u32) -> u32 {\n    return a + b;\n}\nfn main() {\n    print(\"ran\");\n    assert(add(1u32, 2u32) == 3u32);\n    return;\n}\n",
    )
    .unwrap();
    let out = dir.join("u32.smc");
    let (code, _, stderr) = smc(&[
        "compile",
        path.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert_eq!(code, 0, "{stderr}");
    let mut bytes = std::fs::read(&out).unwrap();
    assert_eq!(&bytes[..8], b"SEMCOD23");
    bytes[..8].copy_from_slice(b"SEMCOD22");
    let forged = dir.join("forged.smc");
    std::fs::write(&forged, &bytes).unwrap();
    let forged = forged.to_str().unwrap();
    assert_ne!(
        smc(&["verify", forged]).0,
        0,
        "verifier must reject the relabel"
    );
    let (code, stdout, _) = smc(&["run-smc", forged]);
    assert_ne!(code, 0, "run-smc must not execute unverified SemCode");
    assert!(!stdout.contains("ran"), "program body executed: {stdout}");
}
