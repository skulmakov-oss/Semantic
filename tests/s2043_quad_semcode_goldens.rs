//! S2043 CP08: quad law and SemCode header regressions (tests only; no
//! behavior change).
//!
//! - A mathematical two-plane oracle (`t`, `f` bits: N=00, F=01, T=10, S=11)
//!   is pinned in `tests/golden_snapshots/s2043/quad_4x4_static.csv`, and every
//!   row is independently executed through the verified VM at O0 and O1.
//!   Lattice `&&`/`||` (meet/join) and the truth map (`qtruth_*`) are distinct
//!   families and never alias each other.
//! - `&&` / `||` stay eager for `bool` and `quad`: the right operand's effect
//!   and trap happen even when the left operand decides the result.
//! - The SemCode header is selected from the opcodes actually emitted.

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

/// Compiles at O0 and O1, verifies and runs both; returns
/// `[(magic, run_code, run_stdout, run_stderr); 2]`.
fn pipeline(dir: &Path, name: &str, src: &str) -> Vec<(String, i32, String, String)> {
    let path = dir.join(format!("{name}.sm"));
    std::fs::write(&path, src).unwrap();
    let path = path.to_str().unwrap();
    let (code, stdout, stderr) = smc(&["check", path, "--format", "json"]);
    assert_eq!(code, 0, "{name}: check: {stdout}{stderr}");
    ["O0", "O1"]
        .iter()
        .map(|level| {
            let out = dir.join(format!("{name}.{level}.smc"));
            let out_s = out.to_str().unwrap();
            let (code, stdout, stderr) = smc(&["compile", path, "-o", out_s, "--opt-level", level]);
            assert_eq!(code, 0, "{name} {level}: compile: {stdout}{stderr}");
            let (code, stdout, stderr) = smc(&["verify", out_s]);
            assert_eq!(code, 0, "{name} {level}: verify: {stdout}{stderr}");
            let magic = String::from_utf8(std::fs::read(&out).unwrap()[..8].to_vec()).unwrap();
            let (code, stdout, stderr) = smc(&["run-smc", out_s]);
            (magic, code, stdout, stderr)
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Q {
    t: u8,
    f: u8,
}

const STATES: [(&str, Q); 4] = [
    ("N", Q { t: 0, f: 0 }),
    ("F", Q { t: 0, f: 1 }),
    ("T", Q { t: 1, f: 0 }),
    ("S", Q { t: 1, f: 1 }),
];

fn name(q: Q) -> &'static str {
    STATES.iter().find(|(_, s)| *s == q).unwrap().0
}

/// (family, source spelling of `a op b`, plane formula)
type Binary = (&'static str, fn(&str, &str) -> String, fn(Q, Q) -> Q);

fn binary_families() -> [Binary; 6] {
    [
        // Source quad implication: the lattice inverse of `a` joined with `b`.
        (
            "implies_op",
            |a, b| format!("({a} -> {b})"),
            |a, b| Q {
                t: a.f | b.t,
                f: a.t | b.f,
            },
        ),
        (
            "lattice_and",
            |a, b| format!("({a} && {b})"),
            |a, b| Q {
                t: a.t & b.t,
                f: a.f & b.f,
            },
        ),
        (
            "lattice_or",
            |a, b| format!("({a} || {b})"),
            |a, b| Q {
                t: a.t | b.t,
                f: a.f | b.f,
            },
        ),
        (
            "truth_and",
            |a, b| format!("qtruth_and({a}, {b})"),
            |a, b| Q {
                t: a.t & b.t,
                f: a.f | b.f,
            },
        ),
        (
            "truth_or",
            |a, b| format!("qtruth_or({a}, {b})"),
            |a, b| Q {
                t: a.t | b.t,
                f: a.f & b.f,
            },
        ),
        // not(a) joined with b: (a.f | b.t, a.t | b.f).
        (
            "truth_impl",
            |a, b| format!("qtruth_impl({a}, {b})"),
            |a, b| Q {
                t: a.f | b.t,
                f: a.t | b.f,
            },
        ),
    ]
}

fn ledger() -> String {
    let mut out = String::from("family,a,b,result\n");
    for (family, _, law) in binary_families() {
        for (a, qa) in STATES {
            for (b, qb) in STATES {
                out.push_str(&format!("{family},{a},{b},{}\n", name(law(qa, qb))));
            }
        }
    }
    for (a, qa) in STATES {
        out.push_str(&format!(
            "truth_not,{a},,{}\n",
            name(Q { t: qa.f, f: qa.t })
        ));
    }
    for (a, qa) in STATES {
        out.push_str(&format!("not_op,{a},,{}\n", name(Q { t: qa.f, f: qa.t })));
    }
    out
}

#[test]
fn quad_oracle_matches_the_pinned_ledger() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/golden_snapshots/s2043/quad_4x4_static.csv");
    let actual = ledger();
    if std::env::var("SM_UPDATE_S2043_GOLDENS").is_ok_and(|v| v == "1") {
        std::fs::create_dir_all(path.parent().unwrap()).unwrap();
        std::fs::write(&path, &actual).unwrap();
        return;
    }
    let expected = std::fs::read_to_string(&path)
        .expect("ledger present")
        .replace("\r\n", "\n");
    assert_eq!(
        actual, expected,
        "quad oracle drifted from the pinned ledger"
    );
    // The documented anchors.
    for row in [
        "lattice_and,T,F,N",
        "lattice_or,T,F,S",
        "lattice_or,T,N,T",
        "lattice_and,F,N,N",
        "truth_and,T,F,F",
        "truth_impl,T,T,S",
    ] {
        assert!(expected.contains(&format!("{row}\n")), "{row}");
    }
}

#[test]
fn every_quad_pair_executes_as_the_oracle_at_o0_and_o1() {
    let dir = mk_temp_dir("s2043_cp08_quad");
    for (family, spell, law) in binary_families() {
        let mut body = String::new();
        for (a, qa) in STATES {
            for (b, qb) in STATES {
                body.push_str(&format!(
                    "    assert({} == {});\n",
                    spell(a, b),
                    name(law(qa, qb))
                ));
            }
        }
        let src =
            format!("fn main() {{\n    let n: quad = N;\n    let _ = n;\n{body}    return;\n}}\n");
        for (magic, code, stdout, stderr) in pipeline(&dir, family, &src) {
            assert_eq!(code, 0, "{family}: {stdout}{stderr}");
            assert_eq!(magic, "SEMCOD22", "{family}: quad laws need no promotion");
        }
    }
    let mut body = String::new();
    for (a, qa) in STATES {
        let swapped = name(Q { t: qa.f, f: qa.t });
        body.push_str(&format!("    assert(qtruth_not({a}) == {swapped});\n"));
        body.push_str(&format!("    assert(!{a} == {swapped});\n"));
    }
    // Lattice and truth families are distinct for the same operands.
    body.push_str("    assert((T && F) != qtruth_and(T, F));\n");
    body.push_str("    assert((T || F) != qtruth_or(T, F));\n");
    for (magic, code, stdout, stderr) in pipeline(
        &dir,
        "truth_not",
        &format!("fn main() {{\n{body}    return;\n}}\n"),
    ) {
        assert_eq!(code, 0, "{stdout}{stderr}");
        assert_eq!(magic, "SEMCOD22");
    }
}

#[test]
fn quad_never_collapses_to_bool_and_quad_match_needs_default() {
    let dir = mk_temp_dir("s2043_cp08_reject");
    for (name, src, fragment) in [
        (
            "implicit_condition",
            "fn main() {\n    let q: quad = T;\n    if q {\n        return;\n    } else {\n        return;\n    }\n}\n",
            "explicit compare is required for quad",
        ),
        (
            "quad_into_bool",
            "fn main() {\n    let q: quad = T;\n    let b: bool = q;\n    let _ = b;\n    return;\n}\n",
            "type mismatch",
        ),
        (
            "match_all_four_without_default",
            "fn main() {\n    let q: quad = S;\n    let v: i32 = match q {\n        N => { 0 }\n        F => { 1 }\n        T => { 2 }\n        S => { 3 }\n    };\n    let _ = v;\n    return;\n}\n",
            "_",
        ),
    ] {
        let path = dir.join(format!("{name}.sm"));
        std::fs::write(&path, src).unwrap();
        let (code, stdout, stderr) = smc(&["check", path.to_str().unwrap(), "--format", "json"]);
        assert_eq!(code, 1, "{name}: {stdout}{stderr}");
        assert!(stdout.contains("\"E0201\""), "{name}: {stdout}");
        assert!(stdout.contains(fragment), "{name}: {stdout}");
    }
}

#[test]
fn bool_and_quad_operators_stay_eager() {
    let dir = mk_temp_dir("s2043_cp08_eager");
    let effect = "fn side() -> bool {\n    print(\"rhs-bool\");\n    return true;\n}\nfn sideq() -> quad {\n    print(\"rhs-quad\");\n    return T;\n}\nfn main() {\n    let a: bool = false && side();\n    let b: bool = true || side();\n    let c: quad = F && sideq();\n    assert(!a);\n    assert(b);\n    assert(c == N);\n    return;\n}\n";
    for (magic, code, stdout, stderr) in pipeline(&dir, "eager_effects", effect) {
        assert_eq!(code, 0, "{stdout}{stderr}");
        assert_eq!(magic, "SEMCOD22");
        assert_eq!(stdout.matches("rhs-bool").count(), 2, "{stdout}");
        assert_eq!(stdout.matches("rhs-quad").count(), 1, "{stdout}");
    }
    // A trapping right operand traps even though `false` decides `&&`.
    let trap = "fn boom(d: i32) -> bool {\n    let z: i32 = 10 / d;\n    return z == 0;\n}\nfn main() {\n    let a: bool = false && boom(0);\n    let _ = a;\n    return;\n}\n";
    for (_, code, stdout, stderr) in pipeline(&dir, "eager_trap", trap) {
        assert_ne!(code, 0, "eager RHS must trap: {stdout}{stderr}");
        assert!(
            format!("{stdout}{stderr}").contains("DivisionByZero"),
            "{stdout}{stderr}"
        );
    }
}

#[test]
fn semcode_header_follows_the_emitted_opcodes() {
    let dir = mk_temp_dir("s2043_cp08_header");
    for (name, src, expected) in [
        (
            "u32_load_only",
            "fn main() {\n    let x: u32 = 61u32;\n    assert(x == 61u32);\n    return;\n}\n",
            ["SEMCOD22", "SEMCOD22"],
        ),
        (
            "u32_literal_arith_folds_at_o1",
            "fn main() {\n    let x: u32 = 61u32 + 1u32;\n    assert(x == 62u32);\n    return;\n}\n",
            ["SEMCOD23", "SEMCOD22"],
        ),
        (
            "u32_runtime_arith",
            "fn add(a: u32, b: u32) -> u32 {\n    return a + b;\n}\nfn main() {\n    assert(add(61u32, 1u32) == 62u32);\n    return;\n}\n",
            ["SEMCOD23", "SEMCOD23"],
        ),
        (
            "bytes_core",
            "fn main() {\n    let b: Bytes = bytes_empty();\n    assert(bytes_len(b) == 0u32);\n    return;\n}\n",
            ["SEMCOD24", "SEMCOD24"],
        ),
        (
            "bytes_endian",
            "fn main() {\n    let w: Bytes = write_u32_le(7u32);\n    assert(bytes_len(w) == 4u32);\n    return;\n}\n",
            ["SEMCOD25", "SEMCOD25"],
        ),
    ] {
        let results = pipeline(&dir, name, src);
        for (level, ((magic, code, stdout, stderr), want)) in
            ["O0", "O1"].iter().zip(results.iter().zip(expected))
        {
            assert_eq!(*code, 0, "{name} {level}: {stdout}{stderr}");
            assert_eq!(magic, want, "{name} {level}");
        }
    }
}
