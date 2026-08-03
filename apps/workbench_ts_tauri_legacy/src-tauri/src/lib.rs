mod adapter;
mod docs;
mod lsp_bridge;
mod reports;
mod scaffold;
mod snapshot;
mod workspace_files;

use adapter::{
  adapter_contract,
  execute_job,
  resolve_workspace,
  AdapterContract,
  JobRequest,
  JobResult,
  WorkspaceSummary,
};
use docs::{read_spec_catalog, read_spec_document, SpecCatalogSection, SpecDocumentView};
use lsp_bridge::{run_smlsp_bridge, SmlspBridgeRequest, SmlspBridgeResult};
use reports::{
  export_release_report, ReleaseReportExportRequest, ReleaseReportExportResult,
};
use scaffold::{scaffold_project, ScaffoldProjectRequest, ScaffoldProjectResult};
use snapshot::{read_overview_snapshot, OverviewSnapshot};
use workspace_files::{
  list_workspace_tree,
  read_workspace_file,
  save_workspace_file,
  SaveWorkspaceFileRequest,
  WorkspaceFileDocument,
  WorkspaceFileRequest,
  WorkspaceTreeNode,
};

#[tauri::command]
fn get_adapter_contract() -> Result<AdapterContract, String> {
  adapter_contract()
}

#[tauri::command]
async fn run_cli_job(request: JobRequest) -> Result<JobResult, String> {
  tauri::async_runtime::spawn_blocking(move || execute_job(request))
    .await
    .map_err(|error| format!("command task failed: {error}"))?
}

#[tauri::command]
fn resolve_workspace_root(candidate: Option<String>) -> Result<WorkspaceSummary, String> {
  resolve_workspace(candidate)
}

#[tauri::command]
fn get_overview_snapshot() -> Result<OverviewSnapshot, String> {
  read_overview_snapshot()
}

#[tauri::command]
fn get_spec_catalog() -> Result<Vec<SpecCatalogSection>, String> {
  read_spec_catalog()
}

#[tauri::command]
fn get_spec_document(relative_path: String) -> Result<SpecDocumentView, String> {
  read_spec_document(relative_path)
}

#[tauri::command]
fn get_workspace_tree(workspace_root: String) -> Result<Vec<WorkspaceTreeNode>, String> {
  list_workspace_tree(workspace_root)
}

#[tauri::command]
fn get_workspace_file(request: WorkspaceFileRequest) -> Result<WorkspaceFileDocument, String> {
  read_workspace_file(request)
}

#[tauri::command]
fn save_workspace_file_contents(
  request: SaveWorkspaceFileRequest,
) -> Result<WorkspaceFileDocument, String> {
  save_workspace_file(request)
}

#[tauri::command]
fn export_release_report_file(
  request: ReleaseReportExportRequest,
) -> Result<ReleaseReportExportResult, String> {
  export_release_report(request)
}

#[tauri::command]
fn scaffold_semantic_project(
  request: ScaffoldProjectRequest,
) -> Result<ScaffoldProjectResult, String> {
  scaffold_project(request)
}

#[tauri::command]
fn run_smlsp_protocol_bridge(
  request: SmlspBridgeRequest,
) -> Result<SmlspBridgeResult, String> {
  run_smlsp_bridge(request)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }
      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      get_adapter_contract,
      run_cli_job,
      resolve_workspace_root,
      get_overview_snapshot,
      get_spec_catalog,
      get_spec_document,
      get_workspace_tree,
      get_workspace_file,
      save_workspace_file_contents,
      export_release_report_file,
      scaffold_semantic_project,
      run_smlsp_protocol_bridge
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
