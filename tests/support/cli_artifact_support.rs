#![allow(dead_code)]
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_DIR_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Debug)]
pub(crate) struct SourceFixturePath(PathBuf);

#[derive(Debug)]
pub(crate) struct TempSourcePath {
    root: PathBuf,
    path: PathBuf,
}

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

pub(crate) trait SourceInputPath {
    fn cli_arg(&self) -> String;
}

impl SourceFixturePath {
    pub(crate) fn cli_arg(&self) -> String {
        normalize_path(&self.0)
    }
}

impl TempSourcePath {
    pub(crate) fn cli_arg(&self) -> String {
        normalize_path(&self.path)
    }
}

impl SourceInputPath for SourceFixturePath {
    fn cli_arg(&self) -> String {
        self.cli_arg()
    }
}

impl SourceInputPath for TempSourcePath {
    fn cli_arg(&self) -> String {
        self.cli_arg()
    }
}

impl SemCodeArtifactPath {
    pub(crate) fn cli_arg(&self) -> String {
        normalize_path(&self.path)
    }

    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    pub(crate) fn root_dir(&self) -> &Path {
        &self.root
    }
}

impl Drop for SemCodeArtifactPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

impl Drop for TempSourcePath {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

pub(crate) fn source_fixture(path: impl Into<PathBuf>) -> SourceFixturePath {
    SourceFixturePath(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path.into()))
}

pub(crate) fn temp_source_file(scope: &str, fixture_name: &str, source: &str) -> TempSourcePath {
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
    let path = root.join(fixture_name);
    std::fs::write(&path, source).expect("write temp source");
    TempSourcePath { root, path }
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

pub(crate) fn target_cli_smoke_artifact(scope: &str, fixture_name: &str) -> SemCodeArtifactPath {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("cli-smoke")
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

pub(crate) fn check_source(source: &impl SourceInputPath) {
    cli_ok(
        vec!["check".to_string(), source.cli_arg()],
        "smc check for source input",
    );
}

pub(crate) fn run_source(source: &impl SourceInputPath) {
    cli_ok(
        vec!["run".to_string(), source.cli_arg()],
        "smc run for source input",
    );
}

pub(crate) fn compile_source_to_artifact(
    source: &impl SourceInputPath,
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
        "smc compile for source input",
    );
}

pub(crate) fn compile_project_root_to_artifact(
    root: &Path,
    artifact: &SemCodeArtifactPath,
    context: &str,
) {
    std::fs::create_dir_all(
        artifact
            .path()
            .parent()
            .expect("SemCode artifact should always have a parent"),
    )
    .expect("mkdir artifact parent");
    cli_ok(
        vec![
            "compile".to_string(),
            normalize_path(root),
            "-o".to_string(),
            artifact.cli_arg(),
        ],
        context,
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
