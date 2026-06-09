use crate::adapter::repo_root;
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::UNIX_EPOCH;

struct DocEntryDef {
  key: &'static str,
  section_key: &'static str,
  section_title: &'static str,
  title: &'static str,
  relative_path: &'static str,
}

const DOC_ENTRIES: &[DocEntryDef] = &[
  DocEntryDef {
    key: "language_overview",
    section_key: "language",
    section_title: "Language",
    title: "Language overview",
    relative_path: "docs/LANGUAGE.md",
  },
  DocEntryDef {
    key: "syntax",
    section_key: "language",
    section_title: "Language",
    title: "Syntax",
    relative_path: "docs/spec/syntax.md",
  },
  DocEntryDef {
    key: "types",
    section_key: "language",
    section_title: "Language",
    title: "Types",
    relative_path: "docs/spec/types.md",
  },
  DocEntryDef {
    key: "source_semantics",
    section_key: "language",
    section_title: "Language",
    title: "Source semantics",
    relative_path: "docs/spec/source_semantics.md",
  },
  DocEntryDef {
    key: "modules",
    section_key: "language",
    section_title: "Language",
    title: "Modules",
    relative_path: "docs/spec/modules.md",
  },
  DocEntryDef {
    key: "imports",
    section_key: "language",
    section_title: "Language",
    title: "Imports",
    relative_path: "docs/imports.md",
  },
  DocEntryDef {
    key: "exports",
    section_key: "language",
    section_title: "Language",
    title: "Exports",
    relative_path: "docs/exports.md",
  },
  DocEntryDef {
    key: "logos",
    section_key: "language",
    section_title: "Language",
    title: "Logos",
    relative_path: "docs/spec/logos.md",
  },
  DocEntryDef {
    key: "diagnostics",
    section_key: "language",
    section_title: "Language",
    title: "Diagnostics",
    relative_path: "docs/spec/diagnostics.md",
  },
  DocEntryDef {
    key: "error_index",
    section_key: "language",
    section_title: "Language",
    title: "Error index",
    relative_path: "docs/errors/index.md",
  },
  DocEntryDef {
    key: "error_e0242",
    section_key: "language",
    section_title: "Language",
    title: "Error E0242",
    relative_path: "docs/errors/E0242.md",
  },
  DocEntryDef {
    key: "error_e0243",
    section_key: "language",
    section_title: "Language",
    title: "Error E0243",
    relative_path: "docs/errors/E0243.md",
  },
  DocEntryDef {
    key: "error_e0244",
    section_key: "language",
    section_title: "Language",
    title: "Error E0244",
    relative_path: "docs/errors/E0244.md",
  },
  DocEntryDef {
    key: "error_e0245",
    section_key: "language",
    section_title: "Language",
    title: "Error E0245",
    relative_path: "docs/errors/E0245.md",
  },
  DocEntryDef {
    key: "semcode",
    section_key: "execution",
    section_title: "Execution",
    title: "SemCode",
    relative_path: "docs/spec/semcode.md",
  },
  DocEntryDef {
    key: "verifier",
    section_key: "execution",
    section_title: "Execution",
    title: "Verifier",
    relative_path: "docs/spec/verifier.md",
  },
  DocEntryDef {
    key: "vm",
    section_key: "execution",
    section_title: "Execution",
    title: "VM",
    relative_path: "docs/spec/vm.md",
  },
  DocEntryDef {
    key: "quotas",
    section_key: "execution",
    section_title: "Execution",
    title: "Quotas",
    relative_path: "docs/spec/quotas.md",
  },
  DocEntryDef {
    key: "profile",
    section_key: "execution",
    section_title: "Execution",
    title: "Profile",
    relative_path: "docs/spec/profile.md",
  },
  DocEntryDef {
    key: "cli",
    section_key: "execution",
    section_title: "Execution",
    title: "CLI",
    relative_path: "docs/spec/cli.md",
  },
  DocEntryDef {
    key: "ir",
    section_key: "execution",
    section_title: "Execution",
    title: "IR",
    relative_path: "docs/spec/ir.md",
  },
  DocEntryDef {
    key: "abi",
    section_key: "prometheus",
    section_title: "PROMETHEUS",
    title: "ABI",
    relative_path: "docs/spec/abi.md",
  },
  DocEntryDef {
    key: "capabilities",
    section_key: "prometheus",
    section_title: "PROMETHEUS",
    title: "Capabilities",
    relative_path: "docs/spec/capabilities.md",
  },
  DocEntryDef {
    key: "gates",
    section_key: "prometheus",
    section_title: "PROMETHEUS",
    title: "Gates",
    relative_path: "docs/spec/gates.md",
  },
  DocEntryDef {
    key: "runtime",
    section_key: "prometheus",
    section_title: "PROMETHEUS",
    title: "Runtime",
    relative_path: "docs/spec/runtime.md",
  },
  DocEntryDef {
    key: "state",
    section_key: "prometheus",
    section_title: "PROMETHEUS",
    title: "State",
    relative_path: "docs/spec/state.md",
  },
  DocEntryDef {
    key: "rules",
    section_key: "prometheus",
    section_title: "PROMETHEUS",
    title: "Rules",
    relative_path: "docs/spec/rules.md",
  },
  DocEntryDef {
    key: "audit",
    section_key: "prometheus",
    section_title: "PROMETHEUS",
    title: "Audit",
    relative_path: "docs/spec/audit.md",
  },
  DocEntryDef {
    key: "milestones",
    section_key: "release",
    section_title: "Release",
    title: "Milestones",
    relative_path: "docs/roadmap/milestones.md",
  },
  DocEntryDef {
    key: "readiness",
    section_key: "release",
    section_title: "Release",
    title: "Readiness",
    relative_path: "docs/roadmap/v1_readiness.md",
  },
  DocEntryDef {
    key: "compatibility",
    section_key: "release",
    section_title: "Release",
    title: "Compatibility statement",
    relative_path: "docs/roadmap/compatibility_statement.md",
  },
  DocEntryDef {
    key: "bundle_checklist",
    section_key: "release",
    section_title: "Release",
    title: "Release bundle checklist",
    relative_path: "docs/roadmap/release_bundle_checklist.md",
  },
  DocEntryDef {
    key: "runtime_validation",
    section_key: "release",
    section_title: "Release",
    title: "Runtime validation policy",
    relative_path: "docs/roadmap/runtime_validation_policy.md",
  },
  DocEntryDef {
    key: "asset_smoke",
    section_key: "release",
    section_title: "Release",
    title: "Release asset smoke matrix",
    relative_path: "docs/roadmap/release_asset_smoke_matrix.md",
  },
  DocEntryDef {
    key: "stable_policy",
    section_key: "release",
    section_title: "Release",
    title: "Stable release policy",
    relative_path: "docs/roadmap/stable_release_policy.md",
  },
  DocEntryDef {
    key: "backlog",
    section_key: "release",
    section_title: "Release",
    title: "Backlog",
    relative_path: "docs/roadmap/backlog.md",
  },
  DocEntryDef {
    key: "wbs",
    section_key: "release",
    section_title: "Release",
    title: "WBS",
    relative_path: "docs/roadmap/wbs.md",
  },
  DocEntryDef {
    key: "workbench_spec_index_reader",
    section_key: "workbench",
    section_title: "Workbench",
    title: "Workbench Spec Index Reader v0",
    relative_path: "docs/roadmap/workbench_spec_index_reader_v0.md",
  },
];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecCatalogDocument {
  pub key: String,
  pub title: String,
  pub relative_path: String,
  pub absolute_path: String,
  pub status: Option<String>,
  pub modified_epoch_ms: Option<u128>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecCatalogSection {
  pub key: String,
  pub title: String,
  pub documents: Vec<SpecCatalogDocument>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecDocumentHeading {
  pub level: u8,
  pub title: String,
  pub anchor: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SpecDocumentView {
  pub key: String,
  pub section_key: String,
  pub section_title: String,
  pub title: String,
  pub relative_path: String,
  pub absolute_path: String,
  pub status: Option<String>,
  pub modified_epoch_ms: Option<u128>,
  pub markdown: String,
  pub headings: Vec<SpecDocumentHeading>,
}

pub fn read_spec_catalog() -> Result<Vec<SpecCatalogSection>, String> {
  let repo_root = repo_root()?;
  let mut sections = BTreeMap::<String, (String, Vec<SpecCatalogDocument>)>::new();

  for entry in DOC_ENTRIES {
    let path = repo_relative_path(&repo_root, entry.relative_path);
    let markdown = fs::read_to_string(&path)
      .map_err(|error| format!("failed to read '{}': {error}", path.display()))?;

    sections
      .entry(entry.section_key.to_owned())
      .or_insert_with(|| (entry.section_title.to_owned(), Vec::new()))
      .1
      .push(SpecCatalogDocument {
        key: entry.key.to_owned(),
        title: entry.title.to_owned(),
        relative_path: entry.relative_path.to_owned(),
        absolute_path: path.to_string_lossy().into_owned(),
        status: status_line(&markdown),
        modified_epoch_ms: modified_epoch_ms(&path),
      });
  }

  Ok(
    sections
      .into_iter()
      .map(|(key, (title, documents))| SpecCatalogSection {
        key,
        title,
        documents,
      })
      .collect(),
  )
}

pub fn read_spec_document(relative_path: String) -> Result<SpecDocumentView, String> {
  let entry = DOC_ENTRIES
    .iter()
    .find(|entry| entry.relative_path == relative_path)
    .ok_or_else(|| format!("document '{}' is not part of the canonical navigator", relative_path))?;
  let repo_root = repo_root()?;
  let path = repo_relative_path(&repo_root, entry.relative_path);
  let markdown = fs::read_to_string(&path)
    .map_err(|error| format!("failed to read '{}': {error}", path.display()))?;

  Ok(SpecDocumentView {
    key: entry.key.to_owned(),
    section_key: entry.section_key.to_owned(),
    section_title: entry.section_title.to_owned(),
    title: first_heading(&markdown).unwrap_or_else(|| entry.title.to_owned()),
    relative_path: entry.relative_path.to_owned(),
    absolute_path: path.to_string_lossy().into_owned(),
    status: status_line(&markdown),
    modified_epoch_ms: modified_epoch_ms(&path),
    markdown: markdown.clone(),
    headings: extract_headings(&markdown),
  })
}

fn repo_relative_path(repo_root: &Path, relative_path: &str) -> PathBuf {
  relative_path
    .split('/')
    .fold(repo_root.to_path_buf(), |path, segment| path.join(segment))
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

fn extract_headings(markdown: &str) -> Vec<SpecDocumentHeading> {
  let mut slug_counts = HashMap::<String, usize>::new();
  let mut headings = Vec::new();

  for line in markdown.lines() {
    let trimmed = line.trim();
    let (level, title) = if let Some(title) = trimmed.strip_prefix("# ") {
      (1, title)
    } else if let Some(title) = trimmed.strip_prefix("## ") {
      (2, title)
    } else if let Some(title) = trimmed.strip_prefix("### ") {
      (3, title)
    } else {
      continue;
    };

    let title = title.trim();
    if title.is_empty() {
      continue;
    }

    let base_slug = slugify(title);
    let counter = slug_counts.entry(base_slug.clone()).or_insert(0);
    let anchor = if *counter == 0 {
      base_slug
    } else {
      format!("{base_slug}-{}", *counter + 1)
    };
    *counter += 1;

    headings.push(SpecDocumentHeading {
      level,
      title: title.to_owned(),
      anchor,
    });
  }

  headings
}

fn slugify(input: &str) -> String {
  let mut slug = String::new();
  let mut last_dash = false;

  for ch in input.chars() {
    let normalized = ch.to_ascii_lowercase();
    if normalized.is_ascii_alphanumeric() {
      slug.push(normalized);
      last_dash = false;
    } else if !last_dash {
      slug.push('-');
      last_dash = true;
    }
  }

  slug.trim_matches('-').to_owned()
}

fn modified_epoch_ms(path: &Path) -> Option<u128> {
  fs::metadata(path)
    .ok()
    .and_then(|metadata| metadata.modified().ok())
    .and_then(|modified| modified.duration_since(UNIX_EPOCH).ok())
    .map(|duration| duration.as_millis())
}
