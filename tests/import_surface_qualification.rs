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

    let temp_dir = mk_temp_dir("import_surface");
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
fn import_surface_qualification_pack_is_depth_classified() {
    let depth_ladder = [
        Depth::CheckOnly,
        Depth::CompileOk,
        Depth::VerifyOk,
        Depth::RunOk,
    ];
    assert_eq!(depth_ladder[3].label(), "RunOk");

    let positive_cases = [
        (
            "examples/qualification/import_surface/positive_single_import/src/main.sm",
            Depth::RunOk,
        ),
        (
            "examples/qualification/import_surface/positive_nested_relative_import/src/main.sm",
            Depth::RunOk,
        ),
        (
            "examples/qualification/import_surface/positive_transitive_import/src/main.sm",
            Depth::RunOk,
        ),
    ];

    for (rel, depth) in positive_cases {
        assert_eq!(depth.label(), "RunOk", "unexpected depth for {rel}");
        run_depth(rel, depth);
    }

    let negative_cases = [
        (
            "examples/qualification/import_surface/negative_missing_import_file/src/main.sm",
            "failed to resolve entry module",
        ),
        (
            "examples/qualification/import_surface/negative_duplicate_import_symbol/src/main.sm",
            "duplicate function 'score'",
        ),
        (
            "examples/qualification/import_surface/negative_import_cycle/src/main.sm",
            "cyclic executable helper import detected",
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
