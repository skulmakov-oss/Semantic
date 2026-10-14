//! SHF-P1 (#2030, #2037): persistent `Map` storage in `sm-vm`.
//!
//! The representation change must be invisible to Semantic programs: map values stay
//! logically independent (old versions, call arguments, record fields), lookups and
//! replacement keep their meaning, and Steps / Calls / Frames are exactly the values
//! the previous `Vec<(MapKey, Value)>` representation produced on `main` @ `94c5c50`.

use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

static DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

const METRICS_PREFIX: &str = "{\"schema\":\"semantic.run.metrics/0.1\"";

fn mk_temp_dir() -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "shf_p1_map_{}_{}_{}",
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

/// Runs `source` under the trusted-compiler envelope and returns the metrics line.
fn run_metrics(source: &str) -> Value {
    let dir = mk_temp_dir();
    let program = dir.join("main.sm");
    std::fs::write(&program, source).expect("write fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_smc"))
        .arg("run")
        .arg(&program)
        .args(["--profile", "pure", "--root"])
        .arg(&dir)
        .args([
            "--envelope",
            "trusted-compiler",
            "--max-steps",
            "200000000",
            "--metrics",
            "json",
        ])
        .current_dir(&dir)
        .output()
        .expect("run smc");
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let lines: Vec<&str> = stderr
        .lines()
        .filter(|line| line.starts_with(METRICS_PREFIX))
        .collect();
    assert_eq!(
        lines.len(),
        1,
        "expected one metrics line, stderr:\n{stderr}"
    );
    let _ = std::fs::remove_dir_all(&dir);
    serde_json::from_str(lines[0]).expect("metrics line is valid JSON")
}

fn assert_ok_with_counters(source: &str, steps: u64, calls: u64, peak_frames: u64) {
    let metrics = run_metrics(source);
    assert_eq!(metrics["outcome"], "ok", "metrics: {metrics}");
    let counters = &metrics["counters"];
    assert_eq!(counters["steps"], steps, "Steps drifted: {metrics}");
    assert_eq!(counters["calls"], calls, "Calls drifted: {metrics}");
    assert_eq!(
        counters["peak_frames"], peak_frames,
        "Frames drifted: {metrics}"
    );
    assert_eq!(counters["effect_calls"], 0, "effects drifted: {metrics}");
}

/// Old versions, call arguments, record fields and `with` copies stay independent;
/// replacement, missing-key defaults and every admitted scalar key family keep their
/// meaning. Counters are the values measured on `main` @ `94c5c50` (pre-P1).
const VALUE_SEMANTICS: &str = r#"record Ctx { syms: Map(text, i32), next_id: i32 }

fn add(m: Map(i32, i32), k: i32) -> Map(i32, i32) {
    return map_set(m, k, k * 10);
}

fn main() {
    let mut m: Map(i32, i32) = map_empty();
    let mut i: i32 = 0;
    while i < 300 {
        m = map_set(m, (i * 7919) % 1000, i);
        i = i + 1;
    }
    let snapshot: Map(i32, i32) = m;
    let grown: Map(i32, i32) = add(m, 5000);
    m = map_set(m, 0, -1);
    m = map_set(m, 919, 42);
    assert(map_get(snapshot, 0, -9) == 0);
    assert(map_get(snapshot, 919, -9) == 1);
    assert(map_get(m, 0, -9) == -1);
    assert(map_get(m, 919, -9) == 42);
    assert(map_contains(grown, 5000));
    assert(map_get(grown, 5000, -9) == 50000);
    assert(!map_contains(snapshot, 5000));
    assert(!map_contains(m, 5000));
    assert(map_get(m, 1, -9) == -9);

    let c: Ctx = Ctx { syms: map_empty(), next_id: 0 };
    let c2: Ctx = c with { syms: map_set(c.syms, "a", 1), next_id: 1 };
    assert(!map_contains(c.syms, "a"));
    assert(map_get(c2.syms, "a", 0) == 1);

    let mut u: Map(u32, bool) = map_empty();
    u = map_set(u, 7u32, true);
    u = map_set(u, 7u32, false);
    assert(map_contains(u, 7u32));
    assert(!map_get(u, 7u32, true));

    let mut b: Map(bool, text) = map_empty();
    b = map_set(b, true, "t");
    b = map_set(b, false, "f");
    assert(map_get(b, true, "") == "t");
    assert(map_get(b, false, "") == "f");
    return;
}
"#;

/// Compiler-shaped symbol table: a map inside a context record, threaded through a
/// function call per declaration, with duplicate declarations. Counters are the
/// values measured on `main` @ `94c5c50` (pre-P1).
const SYMBOL_TABLE: &str = r#"record Ctx { syms: Map(text, i32), next_id: i32 }

fn declare(c: Ctx, name: text) -> Ctx {
    if map_contains(c.syms, name) {
        return c;
    }
    return c with { syms: map_set(c.syms, name, c.next_id), next_id: c.next_id + 1 };
}

fn main() {
    let mut c: Ctx = Ctx { syms: map_empty(), next_id: 0 };
    let mut i: i32 = 0;
    while i < 4000 {
        c = declare(c, "sym" + to_text(i % 2000));
        i = i + 1;
    }
    assert(c.next_id == 2000);
    assert(map_get(c.syms, "sym0", -1) == 0);
    assert(map_get(c.syms, "sym1999", -1) == 1999);
    assert(map_get(c.syms, "sym2000", -1) == -1);
    return;
}
"#;

#[test]
fn map_value_semantics_and_counters_are_unchanged() {
    assert_ok_with_counters(VALUE_SEMANTICS, 5584, 1, 2);
}

#[test]
fn compiler_shaped_symbol_table_keeps_counters() {
    assert_ok_with_counters(SYMBOL_TABLE, 134049, 4000, 2);
}

#[test]
fn deterministic_fields_repeat_exactly() {
    let strip = |mut m: Value| {
        m.as_object_mut()
            .expect("object")
            .remove("nondeterministic");
        m
    };
    let first = strip(run_metrics(SYMBOL_TABLE));
    for _ in 0..2 {
        assert_eq!(strip(run_metrics(SYMBOL_TABLE)), first);
    }
}

#[test]
fn large_map_construction_completes_under_the_envelope() {
    // 50k distinct text keys. Under the previous representation every MAP_SET copied
    // the whole map, so this was quadratic; no wall-clock bound is asserted here.
    let source = "fn main() {\n    let mut m: Map(text, i32) = map_empty();\n    let mut i: i32 = 0;\n    while i < 50000 {\n        m = map_set(m, \"k\" + to_text(i), i);\n        i = i + 1;\n    }\n    assert(map_get(m, \"k0\", -1) == 0);\n    assert(map_get(m, \"k49999\", -1) == 49999);\n    assert(!map_contains(m, \"k50000\"));\n    return;\n}\n";
    let metrics = run_metrics(source);
    assert_eq!(metrics["outcome"], "ok", "metrics: {metrics}");
}
