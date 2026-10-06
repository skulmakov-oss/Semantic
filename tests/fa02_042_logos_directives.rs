//! FA-02-042 / #1987: accepted Logos `Pulse`/`Profile` directives are
//! preserved as opaque inspection nodes. Preservation must not widen the
//! Model-B boundary: Logos still has no SemCode, verifier or VM path, and
//! surface authority is unchanged.

use std::path::PathBuf;
use std::process::Command;

const SEMCODE_BOUNDARY: &str =
    "Logos input lowers to LogosIrLaw stream; SemCode function IR requires RustLike frontend";

const DIRECTIVE_SOURCE: &str = "Pulse emit = sensor.edge\n\
Entity Sensor:\n    state val: quad\n\n\
Profile fast mode\n\
Law \"CheckSignal\" [priority 10]:\n    When Sensor.val == T -> System.recovery()\n";

fn write_fixture(label: &str, text: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "fa02_042_{label}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("mkdir");
    let path = dir.join("directives.sm");
    std::fs::write(&path, text).expect("write fixture");
    path
}

fn run_err(args: Vec<String>, context: &str) -> String {
    match smc_cli::run(args) {
        Ok(()) => panic!("{context} unexpectedly passed"),
        Err(error) => error,
    }
}

/// The admitted Logos inspection workflow shows the preserved lines.
#[test]
fn dump_ast_shows_preserved_pulse_and_profile() {
    let path = write_fixture("dump_ast", DIRECTIVE_SOURCE);
    let output = Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(["dump-ast", path.to_str().expect("utf-8 path")])
        .output()
        .expect("run smc");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        output.status.success(),
        "dump-ast failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    for needle in [
        "legacy_directives",
        "kind: Pulse",
        "\"Pulse emit = sensor.edge\"",
        "kind: Profile",
        "\"Profile fast mode\"",
    ] {
        assert!(
            stdout.contains(needle),
            "dump-ast lacks {needle:?}:\n{stdout}"
        );
    }
}

/// Preserving the directives must not open any SemCode-producing path.
#[test]
fn preserved_directives_do_not_widen_the_model_b_boundary() {
    let path = write_fixture("model_b", DIRECTIVE_SOURCE);
    let source = path.to_string_lossy().into_owned();
    let artifact = path.with_extension("smc");

    let compile = run_err(
        vec![
            "compile".into(),
            source.clone(),
            "-o".into(),
            artifact.to_string_lossy().into_owned(),
            "--profile".into(),
            "logos".into(),
        ],
        "Logos compile with Pulse/Profile",
    );
    assert!(compile.contains(SEMCODE_BOUNDARY), "{compile}");
    assert!(!artifact.exists(), "rejected compile emitted an artifact");

    for command in ["dump-bytecode", "hash-smc"] {
        let err = run_err(
            vec![
                command.into(),
                source.clone(),
                "--profile".into(),
                "logos".into(),
            ],
            command,
        );
        assert!(err.contains(SEMCODE_BOUNDARY), "{command}: {err}");
    }

    run_err(vec!["run".into(), source], "Logos run with Pulse/Profile");
    assert!(
        semantic_language::frontend::compile_program_to_semcode(DIRECTIVE_SOURCE).is_err(),
        "library SemCode path must reject Logos with Pulse/Profile"
    );
}

/// A Pulse line is Logos-exclusive evidence; mixing it with RustLike
/// evidence stays a deterministic rejection, never a fallback.
#[test]
fn pulse_with_rustlike_evidence_is_rejected_deterministically() {
    let path = write_fixture("mixed", "Pulse p\nfn main() {\n    return;\n}\n");
    let args = vec!["dump-ast".to_string(), path.to_string_lossy().into_owned()];
    let first = run_err(args.clone(), "mixed Pulse + RustLike");
    let second = run_err(args, "repeated mixed Pulse + RustLike");
    assert_eq!(first, second, "mixed-surface rejection drifted");
}
