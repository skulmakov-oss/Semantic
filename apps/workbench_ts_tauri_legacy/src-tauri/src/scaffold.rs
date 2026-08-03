use crate::adapter::repo_root;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldProjectRequest {
  pub workspace_root: String,
  pub mode: String,
  pub package_name: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScaffoldProjectResult {
  pub workspace_root: String,
  pub repo_relative_path: String,
  pub package_name: String,
  pub created_paths: Vec<String>,
  pub entry_relative_path: String,
}

pub fn scaffold_project(
  request: ScaffoldProjectRequest,
) -> Result<ScaffoldProjectResult, String> {
  let repo_root = repo_root()?;
  let package_name = normalize_package_name(&request.package_name)?;
  let workspace_root = resolve_workspace_root(&repo_root, &request.workspace_root)?;
  let mode = request.mode.trim().to_ascii_lowercase();

  let target_root = match mode.as_str() {
    "new" => workspace_root.join(&package_name),
    "init" => workspace_root,
    _ => return Err("project scaffold mode must be 'new' or 'init'".into()),
  };

  ensure_target_allowed(&repo_root, &target_root, mode.as_str())?;

  let src_dir = target_root.join("src");
  let examples_dir = target_root.join("examples");
  fs::create_dir_all(&src_dir)
    .map_err(|error| format!("failed to create '{}': {error}", src_dir.display()))?;
  fs::create_dir_all(&examples_dir)
    .map_err(|error| format!("failed to create '{}': {error}", examples_dir.display()))?;

  let manifest_path = target_root.join("Semantic.toml");
  let entry_path = src_dir.join("main.sm");
  let smoke_path = examples_dir.join("smoke.sm");

  let manifest_text = render_manifest(&package_name);
  let main_text = render_main_source();
  let smoke_text = render_smoke_source();

  write_new_file(&manifest_path, &manifest_text)?;
  write_new_file(&entry_path, &main_text)?;
  write_new_file(&smoke_path, &smoke_text)?;

  let created_paths = vec![manifest_path, entry_path, smoke_path]
    .into_iter()
    .map(|path| relative_to_repo(&repo_root, &path))
    .collect::<Vec<_>>();
  let repo_relative_path = relative_to_repo(&repo_root, &target_root);

  Ok(ScaffoldProjectResult {
    workspace_root: target_root.to_string_lossy().into_owned(),
    repo_relative_path,
    package_name,
    created_paths,
    entry_relative_path: "src/main.sm".into(),
  })
}

fn resolve_workspace_root(repo_root: &Path, workspace_root: &str) -> Result<PathBuf, String> {
  let requested = PathBuf::from(workspace_root);
  let absolute = if requested.is_absolute() {
    requested
  } else {
    repo_root.join(requested)
  };

  let canonical = absolute
    .canonicalize()
    .map_err(|error| format!("failed to resolve workspace root '{}': {error}", absolute.display()))?;

  if !canonical.starts_with(repo_root) {
    return Err("workspace root must stay inside the repository root".into());
  }

  Ok(canonical)
}

fn ensure_target_allowed(repo_root: &Path, target_root: &Path, mode: &str) -> Result<(), String> {
  if !target_root.starts_with(repo_root) {
    return Err("project scaffold target must stay inside the repository root".into());
  }

  match mode {
    "new" => {
      if target_root.exists() {
        let is_empty = fs::read_dir(target_root)
          .map_err(|error| format!("failed to read '{}': {error}", target_root.display()))?
          .next()
          .is_none();
        if !is_empty {
          return Err(format!(
            "new project target '{}' already exists and is not empty",
            target_root.display()
          ));
        }
      }
      Ok(())
    }
    "init" => {
      if !target_root.is_dir() {
        return Err(format!(
          "init target '{}' is not a directory",
          target_root.display()
        ));
      }
      for path in [
        target_root.join("Semantic.toml"),
        target_root.join("src").join("main.sm"),
        target_root.join("examples").join("smoke.sm"),
      ] {
        if path.exists() {
          return Err(format!(
            "init target '{}' already contains '{}'",
            target_root.display(),
            path.file_name().and_then(|name| name.to_str()).unwrap_or("existing scaffold file")
          ));
        }
      }
      Ok(())
    }
    _ => Err("unsupported scaffold mode".into()),
  }
}

fn write_new_file(path: &Path, contents: &str) -> Result<(), String> {
  if path.exists() {
    return Err(format!("refusing to overwrite existing '{}'", path.display()));
  }

  fs::write(path, contents)
    .map_err(|error| format!("failed to write '{}': {error}", path.display()))
}

fn render_manifest(package_name: &str) -> String {
  format!(
    "[package]\nname = \"{package_name}\"\nversion = \"0.1.0\"\nedition = \"v1\"\nentry = \"src/main.sm\"\n"
  )
}

fn render_main_source() -> &'static str {
  "fn main() {\n    return;\n}\n"
}

fn render_smoke_source() -> &'static str {
  "fn allow() -> quad {\n    return T;\n}\n\nfn main() -> quad {\n    return allow();\n}\n"
}

fn normalize_package_name(input: &str) -> Result<String, String> {
  let mut normalized = String::with_capacity(input.len());
  let mut last_was_dash = false;

  for ch in input.trim().chars() {
    if ch.is_ascii_alphanumeric() {
      normalized.push(ch.to_ascii_lowercase());
      last_was_dash = false;
      continue;
    }

    if matches!(ch, ' ' | '_' | '-') && !last_was_dash {
      normalized.push('-');
      last_was_dash = true;
    }
  }

  let normalized = normalized.trim_matches('-').to_string();
  if normalized.is_empty() {
    return Err("package name must contain at least one ASCII letter or digit".into());
  }

  Ok(normalized)
}

fn relative_to_repo(repo_root: &Path, path: &Path) -> String {
  path
    .strip_prefix(repo_root)
    .ok()
    .map(|value| value.to_string_lossy().replace('\\', "/"))
    .unwrap_or_else(|| path.to_string_lossy().replace('\\', "/"))
}

#[cfg(test)]
mod tests {
  use super::{normalize_package_name, render_manifest};

  #[test]
  fn normalize_package_name_collapses_spaces_and_case() {
    assert_eq!(normalize_package_name("Access Policy").unwrap(), "access-policy");
    assert_eq!(normalize_package_name("policy_core").unwrap(), "policy-core");
  }

  #[test]
  fn normalize_package_name_rejects_empty_result() {
    assert!(normalize_package_name("___").is_err());
  }

  #[test]
  fn manifest_renders_expected_entry() {
    let manifest = render_manifest("access-policy");
    assert!(manifest.contains("name = \"access-policy\""));
    assert!(manifest.contains("entry = \"src/main.sm\""));
  }
}
