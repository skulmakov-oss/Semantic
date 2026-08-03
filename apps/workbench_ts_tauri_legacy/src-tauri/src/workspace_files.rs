use crate::adapter::repo_root;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const SKIP_DIRS: &[&str] = &[".git", "node_modules", "target", "dist"];
const TEXT_EXTENSIONS: &[&str] = &[
  "sm", "md", "toml", "json", "ts", "tsx", "css", "rs", "yml", "yaml", "ps1", "txt",
];

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceTreeNode {
  pub name: String,
  pub relative_path: String,
  pub node_type: String,
  pub children: Vec<WorkspaceTreeNode>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFileDocument {
  pub relative_path: String,
  pub absolute_path: String,
  pub content: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFileRequest {
  pub workspace_root: String,
  pub relative_path: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveWorkspaceFileRequest {
  pub workspace_root: String,
  pub relative_path: String,
  pub content: String,
}

pub fn list_workspace_tree(workspace_root: String) -> Result<Vec<WorkspaceTreeNode>, String> {
  let root = resolve_workspace_root_path(&workspace_root)?;
  read_children(&root, &root)
}

pub fn read_workspace_file(request: WorkspaceFileRequest) -> Result<WorkspaceFileDocument, String> {
  let root = resolve_workspace_root_path(&request.workspace_root)?;
  let path = resolve_file_path(&root, &request.relative_path)?;
  let content = fs::read_to_string(&path)
    .map_err(|error| format!("failed to read '{}': {error}", path.display()))?;

  Ok(WorkspaceFileDocument {
    relative_path: request.relative_path,
    absolute_path: path.to_string_lossy().into_owned(),
    content,
  })
}

pub fn save_workspace_file(request: SaveWorkspaceFileRequest) -> Result<WorkspaceFileDocument, String> {
  let root = resolve_workspace_root_path(&request.workspace_root)?;
  let path = resolve_file_path(&root, &request.relative_path)?;

  fs::write(&path, request.content.as_bytes())
    .map_err(|error| format!("failed to write '{}': {error}", path.display()))?;

  Ok(WorkspaceFileDocument {
    relative_path: request.relative_path,
    absolute_path: path.to_string_lossy().into_owned(),
    content: request.content,
  })
}

fn read_children(root: &Path, current: &Path) -> Result<Vec<WorkspaceTreeNode>, String> {
  let mut entries = fs::read_dir(current)
    .map_err(|error| format!("failed to read '{}': {error}", current.display()))?
    .collect::<Result<Vec<_>, _>>()
    .map_err(|error| format!("failed to read directory entry in '{}': {error}", current.display()))?;

  entries.sort_by_key(|entry| entry.file_name());

  let mut nodes = Vec::new();

  for entry in entries {
    let path = entry.path();
    let file_name = entry.file_name().to_string_lossy().into_owned();

    if entry.file_type().map_err(|error| format!("failed to inspect '{}': {error}", path.display()))?.is_dir()
    {
      if SKIP_DIRS.iter().any(|skip| skip == &file_name.as_str()) {
        continue;
      }

      let children = read_children(root, &path)?;
      if children.is_empty() {
        continue;
      }

      nodes.push(WorkspaceTreeNode {
        name: file_name,
        relative_path: relative_to_root(root, &path),
        node_type: "dir".into(),
        children,
      });
      continue;
    }

    if !is_text_file(&path) {
      continue;
    }

    nodes.push(WorkspaceTreeNode {
      name: file_name,
      relative_path: relative_to_root(root, &path),
      node_type: "file".into(),
      children: Vec::new(),
    });
  }

  Ok(nodes)
}

fn resolve_workspace_root_path(workspace_root: &str) -> Result<PathBuf, String> {
  let repo_root = repo_root()?;
  let requested = PathBuf::from(workspace_root);
  let absolute = if requested.is_absolute() {
    requested
  } else {
    repo_root.join(requested)
  };
  let canonical = absolute
    .canonicalize()
    .map_err(|error| format!("failed to resolve workspace root '{}': {error}", absolute.display()))?;

  if !canonical.starts_with(&repo_root) {
    return Err("workspace root must stay inside the repository root".into());
  }

  Ok(canonical)
}

fn resolve_file_path(workspace_root: &Path, relative_path: &str) -> Result<PathBuf, String> {
  let requested = workspace_root.join(relative_path);
  let canonical = requested
    .canonicalize()
    .map_err(|error| format!("failed to resolve file '{}': {error}", requested.display()))?;

  if !canonical.starts_with(workspace_root) {
    return Err("file path must stay inside the selected workspace root".into());
  }

  if !canonical.is_file() {
    return Err(format!("'{}' is not a file", canonical.display()));
  }

  if !is_text_file(&canonical) {
    return Err(format!("'{}' is not an editable text file", canonical.display()));
  }

  Ok(canonical)
}

fn relative_to_root(root: &Path, path: &Path) -> String {
  path
    .strip_prefix(root)
    .ok()
    .map(|path| path.to_string_lossy().replace('\\', "/"))
    .unwrap_or_else(|| path.to_string_lossy().replace('\\', "/"))
}

fn is_text_file(path: &Path) -> bool {
  path
    .extension()
    .and_then(|ext| ext.to_str())
    .map(|ext| TEXT_EXTENSIONS.iter().any(|candidate| candidate.eq_ignore_ascii_case(ext)))
    .unwrap_or(false)
}
