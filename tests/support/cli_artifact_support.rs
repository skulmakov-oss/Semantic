use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub(crate) struct SourceFixturePath(PathBuf);

#[derive(Debug)]
pub(crate) struct SemCodeArtifactPath {
    root: PathBuf,
    path: PathBuf,
}

fn cli_ok(args: Vec<String>, context: &str) {
    smc_cli::run(args).unwrap_or_else(|err| panic!("{context} failed: {err}"));
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

impl SourceFixturePath {
    pub(crate) fn cli_arg(&self) -> String {
        normalize_path(&self.0)
    }
}

impl SemCodeArtifactPath {
    pub(crate) fn cli_arg(&self) -> String {
        normalize_path(&self.path)
    }
}

impl Drop for SemCodeArtifactPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

pub(crate) fn source_fixture(path: impl Into<PathBuf>) -> SourceFixturePath {
    SourceFixturePath(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path.into()))
}

pub(crate) fn temp_semcode_artifact(scope: &str, fixture_name: &str) -> SemCodeArtifactPath {
    let root = std::env::temp_dir()
        .join("semantic-tests")
        .join(scope)
        .join(format!(
            "{}_{}_{}_{}",
            fixture_name,
            std::process::id(),
            TEMP_DIR_COUNTER.fetch_add(1, Ordering::Relaxed),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
    std::fs::create_dir_all(&root).expect("mkdir");
    SemCodeArtifactPath {
        path: root.join("out.smc"),
        root,
    }
}

pub(crate) fn check_source(source: &SourceFixturePath) {
    cli_ok(
        vec!["check".to_string(), source.cli_arg()],
        "smc check for source fixture",
    );
}

pub(crate) fn run_source(source: &SourceFixturePath) {
    cli_ok(
        vec!["run".to_string(), source.cli_arg()],
        "smc run for source fixture",
    );
}

pub(crate) fn compile_source_to_artifact(
    source: &SourceFixturePath,
    artifact: &SemCodeArtifactPath,
) {
    std::fs::create_dir_all(
        artifact
            .path
            .parent()
            .expect("SemCode artifact should always have a parent"),
    )
    .expect("mkdir artifact parent");
    cli_ok(
        vec![
            "compile".to_string(),
            source.cli_arg(),
            "-o".to_string(),
            artifact.cli_arg(),
        ],
        "smc compile for source fixture",
    );
}

pub(crate) fn verify_artifact(artifact: &SemCodeArtifactPath) {
    cli_ok(
        vec!["verify".to_string(), artifact.cli_arg()],
        "smc verify for SemCode artifact",
    );
}

pub(crate) fn run_smc_artifact(artifact: &SemCodeArtifactPath) {
    cli_ok(
        vec!["run-smc".to_string(), artifact.cli_arg()],
        "smc run-smc for SemCode artifact",
    );
}
