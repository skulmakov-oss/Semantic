import { invoke } from '@tauri-apps/api/core'

export type JobKind = 'smc' | 'svm' | 'cargo' | 'release_bundle_verify'

export type AdapterJobSpec = {
  kind: JobKind
  label: string
  resolution: string
  exampleArgs: string[]
  notes: string
}

export type AdapterContract = {
  repoRoot: string
  jobs: AdapterJobSpec[]
}

export type JobRequest = {
  kind: JobKind
  args: string[]
  cwd?: string
}

export type JobResult = {
  kind: JobKind
  resolvedCommand: string[]
  cwd: string
  exitCode: number
  durationMs: number
  success: boolean
  stdout: string
  stderr: string
}

export type WorkspaceSummary = {
  repoRoot: string
  resolvedPath: string
  repoRelativePath: string | null
  isRepoRoot: boolean
}

export type WorkspaceTreeNode = {
  name: string
  relativePath: string
  nodeType: 'dir' | 'file'
  children: WorkspaceTreeNode[]
}

export type WorkspaceFileRequest = {
  workspaceRoot: string
  relativePath: string
}

export type WorkspaceFileDocument = {
  relativePath: string
  absolutePath: string
  content: string
}

export type SaveWorkspaceFileRequest = {
  workspaceRoot: string
  relativePath: string
  content: string
}

export type OverviewDocument = {
  key: string
  title: string
  path: string
  status: string | null
  highlight: string | null
}

export type ReleaseBundleManifest = {
  generatedAt: string
  documentationBundle: string[]
  validationTests: string[]
  snapshotDirectories: string[]
  currentScope: string
}

export type AssetSmokeScenario = {
  scenario: string
  source: string
  validation: string
  expectedSignal: string
  currentResult: string
}

export type AssetSmokeSnapshot = {
  validatedTag: string | null
  validatedAssets: string[]
  scenarios: AssetSmokeScenario[]
}

export type ReleaseReportExportRequest = {
  markdown: string
  fileName?: string
}

export type ReleaseReportExportResult = {
  absolutePath: string
  repoRelativePath: string
}

export type ScaffoldProjectRequest = {
  workspaceRoot: string
  mode: 'new' | 'init'
  packageName: string
}

export type ScaffoldProjectResult = {
  workspaceRoot: string
  repoRelativePath: string
  packageName: string
  createdPaths: string[]
  entryRelativePath: string
}

export type SmlspBridgeRequest = {
  workspaceRoot: string
  relativePath: string
  content: string
  line: number
  character: number
  command: string
  args: string[]
}

export type SmlspBridgeDiagnostic = {
  severity: string
  code: string | null
  message: string
  line: number
  character: number
  endLine: number
  endCharacter: number
}

export type SmlspBridgeResult = {
  transport: string
  commandLine: string[]
  capabilities: string[]
  hoverMarkdown: string | null
  definitionPath: string | null
  definitionLine: number | null
  definitionCharacter: number | null
  formattingText: string | null
  diagnostics: SmlspBridgeDiagnostic[]
  stderr: string
}

export type OverviewSnapshot = {
  repoRoot: string
  branch: string
  headCommit: string
  shortCommit: string
  headTags: string[]
  baselineTagName: string
  baselineTagPointsAtHead: boolean
  baselineManifestPath: string
  baselineManifestExists: boolean
  releaseManifest: ReleaseBundleManifest | null
  assetSmoke: AssetSmokeSnapshot | null
  releaseDocs: OverviewDocument[]
  knownLimits: string[]
}

export type SpecCatalogDocument = {
  key: string
  title: string
  relativePath: string
  absolutePath: string
  status: string | null
  modifiedEpochMs: number | null
}

export type SpecCatalogSection = {
  key: string
  title: string
  documents: SpecCatalogDocument[]
}

export type SpecDocumentHeading = {
  level: number
  title: string
  anchor: string
}

export type SpecDocumentView = {
  key: string
  sectionKey: string
  sectionTitle: string
  title: string
  relativePath: string
  absolutePath: string
  status: string | null
  modifiedEpochMs: number | null
  markdown: string
  headings: SpecDocumentHeading[]
}

export async function fetchAdapterContract() {
  return invoke<AdapterContract>('get_adapter_contract')
}

export async function runCliJob(request: JobRequest) {
  return invoke<JobResult>('run_cli_job', { request })
}

export async function resolveWorkspaceRoot(candidate?: string) {
  return invoke<WorkspaceSummary>('resolve_workspace_root', { candidate })
}

export async function fetchOverviewSnapshot() {
  return invoke<OverviewSnapshot>('get_overview_snapshot')
}

export async function fetchSpecCatalog() {
  return invoke<SpecCatalogSection[]>('get_spec_catalog')
}

export async function fetchSpecDocument(relativePath: string) {
  return invoke<SpecDocumentView>('get_spec_document', { relativePath })
}

export async function fetchWorkspaceTree(workspaceRoot: string) {
  return invoke<WorkspaceTreeNode[]>('get_workspace_tree', { workspaceRoot })
}

export async function fetchWorkspaceFile(request: WorkspaceFileRequest) {
  return invoke<WorkspaceFileDocument>('get_workspace_file', { request })
}

export async function saveWorkspaceFile(request: SaveWorkspaceFileRequest) {
  return invoke<WorkspaceFileDocument>('save_workspace_file_contents', { request })
}

export async function exportReleaseReportFile(request: ReleaseReportExportRequest) {
  return invoke<ReleaseReportExportResult>('export_release_report_file', { request })
}

export async function scaffoldSemanticProject(request: ScaffoldProjectRequest) {
  return invoke<ScaffoldProjectResult>('scaffold_semantic_project', { request })
}

export async function runSmlspProtocolBridge(request: SmlspBridgeRequest) {
  return invoke<SmlspBridgeResult>('run_smlsp_protocol_bridge', { request })
}
