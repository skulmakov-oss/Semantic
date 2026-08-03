use crate::adapter::repo_root;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseReportExportRequest {
  pub markdown: String,
  pub file_name: Option<String>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReleaseReportExportResult {
  pub absolute_path: String,
  pub repo_relative_path: String,
}

pub fn export_release_report(
  request: ReleaseReportExportRequest,
) -> Result<ReleaseReportExportResult, String> {
  let repo_root = repo_root()?;
  let reports_dir = repo_root.join("artifacts").join("workbench").join("reports");
  fs::create_dir_all(&reports_dir).map_err(|error| {
    format!(
      "failed to create report directory '{}': {error}",
      reports_dir.display()
    )
  })?;

  let file_name = request
    .file_name
    .as_deref()
    .map(sanitize_file_name)
    .filter(|value| !value.is_empty())
    .unwrap_or_else(default_report_file_name);
  let target = reports_dir.join(file_name);

  fs::write(&target, request.markdown).map_err(|error| {
    format!(
      "failed to write release report '{}': {error}",
      target.display()
    )
  })?;

  let repo_relative_path = target
    .strip_prefix(&repo_root)
    .unwrap_or(&target)
    .to_string_lossy()
    .replace('\\', "/");

  Ok(ReleaseReportExportResult {
    absolute_path: target.to_string_lossy().into_owned(),
    repo_relative_path,
  })
}

fn sanitize_file_name(input: &str) -> String {
  let mut out = String::with_capacity(input.len());
  for ch in input.chars() {
    if ch.is_ascii_alphanumeric() || matches!(ch, '.' | '-' | '_') {
      out.push(ch);
    } else if ch.is_whitespace() {
      out.push('-');
    }
  }

  if out.ends_with(".md") {
    out
  } else if out.is_empty() {
    String::new()
  } else {
    format!("{out}.md")
  }
}

fn default_report_file_name() -> String {
  let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)
    .map(|duration| duration.as_secs())
    .unwrap_or(0);
  format!("release-console-report-{timestamp}.md")
}
