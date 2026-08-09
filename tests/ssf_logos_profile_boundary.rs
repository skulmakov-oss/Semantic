use std::path::{Path, PathBuf};

const LOGOS_EXAMPLE: &str = "examples/canonical/quad_cycle_logos/src/main.sm";
const MIXED_FIXTURE: &str = "tests/fixtures/ssf02/mixed_profiles.sm";
const SEMCODE_BOUNDARY: &str =
    "Logos input lowers to LogosIrLaw stream; SemCode function IR requires RustLike frontend";

fn repo_path(relative: &str) -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join(relative)
        .to_string_lossy()
        .into_owned()
}

fn run_ok(args: Vec<String>, context: &str) {
    smc_cli::run(args).unwrap_or_else(|error| panic!("{context} failed: {error}"));
}

fn run_err(args: Vec<String>, context: &str) -> String {
    match smc_cli::run(args) {
        Ok(()) => panic!("{context} unexpectedly passed"),
        Err(error) => error,
    }
}

fn absent_artifact(name: &str) -> PathBuf {
    std::env::temp_dir().join(format!("semantic-ssf02-{}-{name}.smc", std::process::id()))
}

#[test]
fn logos_declarative_inspection_path_remains_available() {
    let source = repo_path(LOGOS_EXAMPLE);
    run_ok(
        vec!["dump-ast".to_string(), source.clone()],
        "Logos AST inspection",
    );
    run_ok(
        vec![
            "dump-ir".to_string(),
            source.clone(),
            "--profile".to_string(),
            "logos".to_string(),
        ],
        "Logos IR inspection",
    );
    run_ok(
        vec![
            "hash-ir".to_string(),
            source,
            "--profile".to_string(),
            "logos".to_string(),
        ],
        "Logos IR hashing",
    );
}

#[test]
fn logos_cannot_emit_semcode_or_bytecode() {
    let source = repo_path(LOGOS_EXAMPLE);
    let artifact = absent_artifact("compile");
    let compile_error = run_err(
        vec![
            "compile".to_string(),
            source.clone(),
            "-o".to_string(),
            artifact.to_string_lossy().into_owned(),
            "--profile".to_string(),
            "logos".to_string(),
        ],
        "Logos SemCode compilation",
    );
    assert!(compile_error.contains(SEMCODE_BOUNDARY), "{compile_error}");
    assert!(!artifact.exists(), "rejected compile emitted an artifact");

    let bytecode_error = run_err(
        vec![
            "dump-bytecode".to_string(),
            source,
            "--profile".to_string(),
            "logos".to_string(),
        ],
        "Logos bytecode dump",
    );
    assert!(
        bytecode_error.contains(SEMCODE_BOUNDARY),
        "{bytecode_error}"
    );

    let run_error = run_err(
        vec!["run".to_string(), repo_path(LOGOS_EXAMPLE)],
        "Logos source execution",
    );
    assert!(
        run_error.contains("expected top-level 'Import'"),
        "{run_error}"
    );
}

#[test]
fn mixed_profile_source_is_rejected_deterministically() {
    let source = repo_path(MIXED_FIXTURE);
    let args = vec![
        "dump-ir".to_string(),
        source,
        "--profile".to_string(),
        "logos".to_string(),
    ];
    let first = run_err(args.clone(), "mixed-profile Logos projection");
    let second = run_err(args, "repeated mixed-profile Logos projection");
    assert_eq!(first, second, "mixed-profile rejection drifted");
    assert!(
        first.contains("error[E0200]: expected Logos declaration"),
        "unexpected mixed-profile diagnostic: {first}"
    );
}

#[test]
fn cli_and_docs_advertise_the_same_profile_boundary() {
    let usage = run_err(Vec::new(), "empty CLI invocation");
    assert!(usage.contains("compile <input.sm|project-root> -o <out.smc> [--profile auto|rust]"));
    assert!(usage.contains("dump-ir <input.sm> [--profile auto|rust|logos]"));
    assert!(usage.contains("dump-bytecode <input.sm> [--profile auto|rust]"));
    assert!(!usage.contains("dump-bytecode <input.sm> [--profile auto|rust|logos]"));

    let logos_spec = include_str!("../docs/spec/logos.md");
    let decision =
        include_str!("../docs/roadmap/stable_foundation/rustlike_logos_coherence_decision.md");
    for text in [logos_spec, decision] {
        assert!(text.contains("semantic.logos.declarative/0.1"));
        assert!(text.contains("Model B"));
    }
}
