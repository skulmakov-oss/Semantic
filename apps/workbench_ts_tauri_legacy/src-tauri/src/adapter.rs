use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
  Smc,
  Svm,
  Cargo,
  ReleaseBundleVerify,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdapterJobSpec {
  pub kind: JobKind,
  pub label: &'static str,
  pub resolution: &'static str,
  pub example_args: Vec<String>,
  pub notes: &'static str,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdapterContract {
  pub repo_root: String,
  pub jobs: Vec<AdapterJobSpec>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceSummary {
  pub repo_root: String,
  pub resolved_path: String,
  pub repo_relative_path: Option<String>,
  pub is_repo_root: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobRequest {
  pub kind: JobKind,
  pub args: Vec<String>,
  pub cwd: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct JobResult {
  pub kind: JobKind,
  pub resolved_command: Vec<String>,
  pub cwd: String,
  pub exit_code: i32,
  pub duration_ms: u64,
  pub success: bool,
  pub stdout: String,
  pub stderr: String,
}

pub fn adapter_contract() -> Result<AdapterContract, String> {
  let repo_root = repo_root()?;

  Ok(AdapterContract {
    repo_root: repo_root.to_string_lossy().into_owned(),
    jobs: vec![
      AdapterJobSpec {
        kind: JobKind::Cargo,
        label: "Cargo",
        resolution: "cargo from PATH",
        example_args: vec!["--version".into()],
        notes: "Used for workspace validation and repository-centric commands.",
      },
      AdapterJobSpec {
        kind: JobKind::Smc,
        label: "smc",
        resolution:
          "target/release/smc(.exe) -> target/debug/smc(.exe) -> smc on PATH -> cargo run --bin smc --",
        example_args: vec!["--help".into()],
        notes: "Canonical Semantic compile/check/run CLI surface.",
      },
      AdapterJobSpec {
        kind: JobKind::Svm,
        label: "svm",
        resolution:
          "target/release/svm(.exe) -> target/debug/svm(.exe) -> svm on PATH -> cargo run --bin svm --",
        example_args: vec!["--help".into()],
        notes: "Canonical Semantic execution and disassembly CLI surface.",
      },
      AdapterJobSpec {
        kind: JobKind::ReleaseBundleVerify,
        label: "Release bundle verify",
        resolution: "pwsh -File scripts/verify_release_bundle.ps1",
        example_args: vec![
          "-ManifestPath".into(),
          "artifacts/baselines/semantic_v1_language_maturity_release_bundle_manifest.json".into(),
        ],
        notes: "Release validation remains driven by the existing repository script.",
      },
    ],
  })
}

pub fn execute_job(request: JobRequest) -> Result<JobResult, String> {
  let repo_root = repo_root()?;
  let cwd = resolve_cwd(&repo_root, request.cwd.as_deref())?;
  let (program, args) = resolve_invocation(&repo_root, &request.kind, &request.args);

  let started = Instant::now();
  let output = Command::new(&program)
    .args(&args)
    .current_dir(&cwd)
    .output()
    .map_err(|error| format!("failed to spawn command '{}': {error}", program))?;

  let duration_ms = started
    .elapsed()
    .as_millis()
    .try_into()
    .unwrap_or(u64::MAX);

  Ok(JobResult {
    kind: request.kind,
    resolved_command: std::iter::once(program.clone())
      .chain(args.iter().cloned())
      .collect(),
    cwd: cwd.to_string_lossy().into_owned(),
    exit_code: output.status.code().unwrap_or(-1),
    duration_ms,
    success: output.status.success(),
    stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
    stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
  })
}

pub fn resolve_workspace(candidate: Option<String>) -> Result<WorkspaceSummary, String> {
  let repo_root = repo_root()?;
  let resolved = resolve_cwd(&repo_root, candidate.as_deref())?;
  let repo_relative_path = resolved
    .strip_prefix(&repo_root)
    .ok()
    .map(|path| path.to_string_lossy().replace('\\', "/"))
    .and_then(|path| if path.is_empty() { None } else { Some(path) });

  Ok(WorkspaceSummary {
    repo_root: repo_root.to_string_lossy().into_owned(),
    resolved_path: resolved.to_string_lossy().into_owned(),
    repo_relative_path,
    is_repo_root: resolved == repo_root,
  })
}

fn resolve_invocation(
  repo_root: &Path,
  kind: &JobKind,
  args: &[String],
) -> (String, Vec<String>) {
  match kind {
    JobKind::Cargo => ("cargo".into(), args.to_vec()),
    JobKind::Smc => resolve_semantic_cli(repo_root, "smc", args),
    JobKind::Svm => resolve_semantic_cli(repo_root, "svm", args),
    JobKind::ReleaseBundleVerify => {
      let script = repo_root.join("scripts").join("verify_release_bundle.ps1");
      let resolved_args = if args.is_empty() {
        vec![
          "-ManifestPath".into(),
          repo_root
            .join("artifacts")
            .join("baselines")
            .join("semantic_v1_language_maturity_release_bundle_manifest.json")
            .to_string_lossy()
            .into_owned(),
        ]
      } else {
        args.to_vec()
      };

      let mut command_args = vec!["-File".into(), script.to_string_lossy().into_owned()];
      command_args.extend(resolved_args);
      ("pwsh".into(), command_args)
    }
  }
}

fn resolve_semantic_cli(
  repo_root: &Path,
  tool: &str,
  args: &[String],
) -> (String, Vec<String>) {
  let exe_name = executable_name(tool);

  for candidate in [
    repo_root.join("target").join("release").join(&exe_name),
    repo_root.join("target").join("debug").join(&exe_name),
  ] {
    if candidate.exists() {
      return (
        candidate.to_string_lossy().into_owned(),
        args.to_vec(),
      );
    }
  }

  if command_available(&exe_name) {
    return (exe_name, args.to_vec());
  }

  let mut fallback_args = vec![
    "run".into(),
    "--quiet".into(),
    "--bin".into(),
    tool.into(),
    "--".into(),
  ];
  fallback_args.extend(args.iter().cloned());
  ("cargo".into(), fallback_args)
}

pub fn repo_root() -> Result<PathBuf, String> {
  let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
  base
    .join("..")
    .join("..")
    .join("..")
    .canonicalize()
    .map_err(|error| format!("failed to resolve repository root: {error}"))
}

fn resolve_cwd(repo_root: &Path, cwd: Option<&str>) -> Result<PathBuf, String> {
  let requested = cwd
    .map(PathBuf::from)
    .unwrap_or_else(|| repo_root.to_path_buf());
  let absolute = if requested.is_absolute() {
    requested
  } else {
    repo_root.join(requested)
  };

  let canonical = absolute
    .canonicalize()
    .map_err(|error| format!("failed to resolve job cwd: {error}"))?;
  let root = repo_root
    .canonicalize()
    .map_err(|error| format!("failed to canonicalize repository root: {error}"))?;

  if !canonical.starts_with(&root) {
    return Err("job cwd must stay inside the repository root".into());
  }

  Ok(canonical)
}

fn executable_name(tool: &str) -> String {
  if cfg!(target_os = "windows") {
    format!("{tool}.exe")
  } else {
    tool.into()
  }
}

fn command_available(command: &str) -> bool {
  if cfg!(target_os = "windows") {
    Command::new("where")
      .arg(command)
      .output()
      .map(|output| output.status.success())
      .unwrap_or(false)
  } else {
    Command::new("which")
      .arg(command)
      .output()
      .map(|output| output.status.success())
      .unwrap_or(false)
  }
}
