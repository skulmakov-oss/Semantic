use crate::adapter::repo_root;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const BASELINE_TAG: &str = "semantic-v1-language-maturity-baseline";

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewDocument {
  pub key: &'static str,
  pub title: String,
  pub path: String,
  pub status: Option<String>,
  pub highlight: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OverviewSnapshot {
  pub repo_root: String,
  pub branch: String,
  pub head_commit: String,
  pub short_commit: String,
  pub head_tags: Vec<String>,
  pub baseline_tag_name: String,
  pub baseline_tag_points_at_head: bool,
  pub baseline_manifest_path: String,
  pub baseline_manifest_exists: bool,
  pub release_manifest: Option<ReleaseBundleManifest>,
  pub asset_smoke: Option<AssetSmokeSnapshot>,
  pub release_docs: Vec<OverviewDocument>,
  pub known_limits: Vec<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseBundleManifest {
  pub generated_at: String,
  pub documentation_bundle: Vec<String>,
  pub validation_tests: Vec<String>,
  pub snapshot_directories: Vec<String>,
  pub current_scope: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetSmokeSnapshot {
  pub validated_tag: Option<String>,
  pub validated_assets: Vec<String>,
  pub scenarios: Vec<AssetSmokeScenario>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AssetSmokeScenario {
  pub scenario: String,
  pub source: String,
  pub validation: String,
  pub expected_signal: String,
  pub current_result: String,
}

#[derive(Debug, Deserialize)]
struct ReleaseBundleManifestFile {
  generated_at: String,
  documentation_bundle: Vec<String>,
  validation_tests: Vec<String>,
  snapshot_directories: Vec<String>,
  current_scope: String,
}

pub fn read_overview_snapshot() -> Result<OverviewSnapshot, String> {
  let repo_root = repo_root()?;
  let branch = git_output(&repo_root, &["branch", "--show-current"])?;
  let head_commit = git_output(&repo_root, &["rev-parse", "HEAD"])?;
  let short_commit = git_output(&repo_root, &["rev-parse", "--short", "HEAD"])?;
  let head_tags = git_output(&repo_root, &["tag", "--points-at", "HEAD"])?
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .map(ToOwned::to_owned)
    .collect::<Vec<_>>();
  let baseline_tag_points_at_head = head_tags.iter().any(|tag| tag == BASELINE_TAG);
  let baseline_manifest = repo_root
    .join("artifacts")
    .join("baselines")
    .join("semantic_v1_language_maturity_release_bundle_manifest.json");

  let readiness_path = repo_root.join("docs").join("roadmap").join("v1_readiness.md");
  let readiness_body = read_markdown(&readiness_path)?;
  let asset_smoke_path = repo_root
    .join("docs")
    .join("roadmap")
    .join("release_asset_smoke_matrix.md");
  let asset_smoke_body = read_markdown(&asset_smoke_path)?;

  Ok(OverviewSnapshot {
    repo_root: repo_root.to_string_lossy().into_owned(),
    branch,
    head_commit,
    short_commit,
    head_tags,
    baseline_tag_name: BASELINE_TAG.into(),
    baseline_tag_points_at_head,
    baseline_manifest_path: baseline_manifest.to_string_lossy().into_owned(),
    baseline_manifest_exists: baseline_manifest.exists(),
    release_manifest: read_release_manifest(&baseline_manifest).ok(),
    asset_smoke: parse_asset_smoke(&asset_smoke_body),
    release_docs: vec![
      read_overview_document(&repo_root, "readiness", "docs/roadmap/v1_readiness.md", None)?,
      read_overview_document(
        &repo_root,
        "compatibility",
        "docs/roadmap/compatibility_statement.md",
        None,
      )?,
      read_overview_document(
        &repo_root,
        "bundle_checklist",
        "docs/roadmap/release_bundle_checklist.md",
        None,
      )?,
      read_overview_document(
        &repo_root,
        "asset_smoke",
        "docs/roadmap/release_asset_smoke_matrix.md",
        Some("Current Validated Tag"),
      )?,
      read_overview_document(
        &repo_root,
        "stable_policy",
        "docs/roadmap/stable_release_policy.md",
        None,
      )?,
    ],
    known_limits: extract_section_bullets(&readiness_body, "Current Known Limits"),
  })
}

fn read_release_manifest(path: &Path) -> Result<ReleaseBundleManifest, String> {
  let text = fs::read_to_string(path)
    .map_err(|error| format!("failed to read '{}': {error}", path.display()))?;
  let manifest: ReleaseBundleManifestFile = serde_json::from_str(&text)
    .map_err(|error| format!("failed to parse '{}': {error}", path.display()))?;

  Ok(ReleaseBundleManifest {
    generated_at: manifest.generated_at,
    documentation_bundle: manifest.documentation_bundle,
    validation_tests: manifest.validation_tests,
    snapshot_directories: manifest.snapshot_directories,
    current_scope: manifest.current_scope,
  })
}

fn read_overview_document(
  repo_root: &Path,
  key: &'static str,
  relative_path: &str,
  highlight_heading: Option<&str>,
) -> Result<OverviewDocument, String> {
  let path = relative_repo_path(repo_root, relative_path);
  let markdown = read_markdown(&path)?;
  Ok(OverviewDocument {
    key,
    title: first_heading(&markdown).unwrap_or_else(|| relative_path.into()),
    path: path.to_string_lossy().into_owned(),
    status: status_line(&markdown),
    highlight: highlight_heading.and_then(|heading| first_bullet_in_section(&markdown, heading)),
  })
}

fn git_output(repo_root: &Path, args: &[&str]) -> Result<String, String> {
  let output = Command::new("git")
    .args(args)
    .current_dir(repo_root)
    .output()
    .map_err(|error| format!("failed to run git {:?}: {error}", args))?;

  if !output.status.success() {
    return Err(format!(
      "git {:?} failed: {}",
      args,
      String::from_utf8_lossy(&output.stderr)
    ));
  }

  Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

fn read_markdown(path: &Path) -> Result<String, String> {
  fs::read_to_string(path).map_err(|error| format!("failed to read '{}': {error}", path.display()))
}

fn first_heading(markdown: &str) -> Option<String> {
  markdown.lines().find_map(|line| {
    line.strip_prefix("# ")
      .map(str::trim)
      .filter(|line| !line.is_empty())
      .map(ToOwned::to_owned)
  })
}

fn status_line(markdown: &str) -> Option<String> {
  markdown.lines().find_map(|line| {
    line.strip_prefix("Status:")
      .map(str::trim)
      .filter(|line| !line.is_empty())
      .map(ToOwned::to_owned)
  })
}

fn extract_section_bullets(markdown: &str, heading: &str) -> Vec<String> {
  let mut in_section = false;
  let mut bullets = Vec::new();

  for line in markdown.lines() {
    if let Some(title) = line.strip_prefix("## ") {
      if in_section {
        break;
      }
      in_section = title.trim() == heading;
      continue;
    }

    if in_section {
      let trimmed = line.trim_start();
      if let Some(bullet) = trimmed.strip_prefix("- ") {
        bullets.push(bullet.trim().to_owned());
      }
    }
  }

  bullets
}

fn first_bullet_in_section(markdown: &str, heading: &str) -> Option<String> {
  extract_section_bullets(markdown, heading).into_iter().next()
}

fn parse_asset_smoke(markdown: &str) -> Option<AssetSmokeSnapshot> {
  let validated_assets = extract_section_bullets(markdown, "Validated Assets");
  let scenarios = extract_markdown_table(markdown, "Current Smoke Matrix")
    .into_iter()
    .filter_map(|row| {
      if row.len() < 5 {
        return None;
      }

      Some(AssetSmokeScenario {
        scenario: row[0].clone(),
        source: row[1].clone(),
        validation: row[2].clone(),
        expected_signal: row[3].clone(),
        current_result: row[4].clone(),
      })
    })
    .collect::<Vec<_>>();

  if validated_assets.is_empty() && scenarios.is_empty() {
    return None;
  }

  Some(AssetSmokeSnapshot {
    validated_tag: first_bullet_in_section(markdown, "Current Validated Tag"),
    validated_assets,
    scenarios,
  })
}

fn extract_markdown_table(markdown: &str, heading: &str) -> Vec<Vec<String>> {
  let mut in_section = false;
  let mut rows = Vec::new();

  for line in markdown.lines() {
    if let Some(title) = line.strip_prefix("## ") {
      if in_section {
        break;
      }
      in_section = title.trim() == heading;
      continue;
    }

    if !in_section {
      continue;
    }

    let trimmed = line.trim();
    if !trimmed.starts_with('|') {
      continue;
    }

    let cells = trimmed
      .trim_matches('|')
      .split('|')
      .map(|cell| cell.trim().to_owned())
      .collect::<Vec<_>>();

    if cells.iter().all(|cell| cell.chars().all(|ch| ch == '-' || ch == ':' || ch == ' ')) {
      continue;
    }

    rows.push(cells);
  }

  if rows.len() <= 1 {
    return Vec::new();
  }

  rows.into_iter().skip(1).collect()
}

fn relative_repo_path(repo_root: &Path, relative_path: &str) -> PathBuf {
  relative_path
    .split('/')
    .fold(repo_root.to_path_buf(), |path, segment| path.join(segment))
}
