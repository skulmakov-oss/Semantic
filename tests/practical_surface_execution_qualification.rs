use std::path::Path;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn cli_ok(args: Vec<String>, context: &str) {
    smc_cli::run(args).unwrap_or_else(|err| panic!("{context} failed: {err}"));
}

fn cli_err(args: Vec<String>, context: &str) -> String {
    smc_cli::run(args).expect_err(&format!("{context} unexpectedly passed"))
}

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Depth {
    CheckOnly,
    CompileOk,
    VerifyOk,
    RunOk,
}

impl Depth {
    fn label(self) -> &'static str {
        match self {
            Self::CheckOnly => "CheckOnly",
            Self::CompileOk => "CompileOk",
            Self::VerifyOk => "VerifyOk",
            Self::RunOk => "RunOk",
        }
    }
}

fn run_depth(rel: &str, depth: Depth) {
    let path = repo_path(rel);
    cli_ok(
        vec!["check".to_string(), path.clone()],
        &format!("smc check for {path}"),
    );
    if matches!(depth, Depth::CheckOnly) {
        return;
    }

    let temp_dir = mk_temp_dir("practical_surface_execution");
    let out = temp_dir.join(
        Path::new(rel)
            .file_name()
            .unwrap_or_else(|| std::ffi::OsStr::new("output.smc")),
    );
    let out_arg = out.to_string_lossy().replace('\\', "/");

    cli_ok(
        vec![
            "compile".to_string(),
            path.clone(),
            "-o".to_string(),
            out_arg.clone(),
        ],
        &format!("smc compile for {path}"),
    );
    if matches!(depth, Depth::CompileOk) {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return;
    }

    cli_ok(
        vec!["verify".to_string(), out_arg.clone()],
        &format!("smc verify for {out_arg}"),
    );
    if matches!(depth, Depth::VerifyOk) {
        let _ = std::fs::remove_dir_all(&temp_dir);
        return;
    }

    cli_ok(
        vec!["run-smc".to_string(), out_arg.clone()],
        &format!("smc run-smc for {out_arg}"),
    );

    let _ = std::fs::remove_dir_all(&temp_dir);
}

#[test]
fn practical_surface_positive_fixtures_are_depth_classified_by_current_pipeline_support() {
    // All current positive practical fixtures are executable on main.
    // Keep the check-only bucket explicit so future partially-supported fixtures can be added
    // without changing the structure of this test.
    let _check_only_fixtures: &[(&str, Depth)] = &[];
    let depth_ladder = [
        Depth::CheckOnly,
        Depth::CompileOk,
        Depth::VerifyOk,
        Depth::RunOk,
    ];
    assert_eq!(depth_ladder[3].label(), "RunOk");

    let executable_fixtures = [
        (
            "examples/qualification/practical_surface/positive_comments_and_blocks/src/main.sm",
            Depth::RunOk,
        ),
        (
            "examples/qualification/practical_surface/positive_records_and_match/src/main.sm",
            Depth::RunOk,
        ),
        (
            "examples/qualification/practical_surface/positive_option_result_flow/src/main.sm",
            Depth::RunOk,
        ),
        (
            "examples/qualification/practical_surface/positive_loops_and_sequences/src/main.sm",
            Depth::RunOk,
        ),
        (
            "examples/qualification/practical_surface/positive_text_and_numbers/src/main.sm",
            Depth::RunOk,
        ),
        (
            "examples/qualification/practical_surface/positive_module_import_surface/src/main.sm",
            Depth::RunOk,
        ),
    ];

    for (rel, depth) in executable_fixtures {
        assert_eq!(depth.label(), "RunOk", "unexpected depth for {rel}");
        run_depth(rel, depth);
    }

    let negative_cases = [
        (
            "examples/qualification/practical_surface/negative_bad_record_field/src/main.sm",
            "record literal 'Pair' is missing field 'right'",
        ),
        (
            "examples/qualification/practical_surface/negative_bad_match_shape/src/main.sm",
            "match arm pattern type 'Signal' does not match scrutinee enum 'Direction'",
        ),
        (
            "examples/qualification/practical_surface/negative_bad_collection_index/src/main.sm",
            "sequence indexing currently requires i32 index",
        ),
        (
            "examples/qualification/practical_surface/negative_bad_option_result_use/src/main.sm",
            "Result::Ok currently requires contextual Result(T, E) type in v0",
        ),
        (
            "examples/qualification/practical_surface/negative_bad_numeric_mismatch/src/main.sm",
            "type mismatch in let",
        ),
    ];

    for (rel, needle) in negative_cases {
        let err = cli_err(
            vec!["check".to_string(), repo_path(rel)],
            &format!("smc check for {rel}"),
        );
        assert!(
            err.contains(needle),
            "expected diagnostic '{needle}' for {rel}, got: {err}"
        );
    }
}
