import { startTransition, useEffect, useRef, useState, type ReactNode } from 'react'
import { NavLink, Route, Routes, useNavigate } from 'react-router-dom'
import {
  exportReleaseReportFile,
  fetchAdapterContract,
  fetchOverviewSnapshot,
  fetchSpecCatalog,
  fetchSpecDocument,
  fetchWorkspaceFile,
  fetchWorkspaceTree,
  resolveWorkspaceRoot,
  runCliJob,
  runSmlspProtocolBridge,
  saveWorkspaceFile,
  scaffoldSemanticProject,
  type AdapterContract,
  type AssetSmokeSnapshot,
  type AdapterJobSpec,
  type JobKind,
  type JobResult,
  type OverviewDocument,
  type OverviewSnapshot,
  type ReleaseBundleManifest,
  type SmlspBridgeResult,
  type SpecCatalogDocument,
  type ScaffoldProjectResult,
  type SpecCatalogSection,
  type SpecDocumentHeading,
  type SpecDocumentView,
  type WorkspaceFileDocument,
  type WorkspaceTreeNode,
  type WorkspaceSummary,
} from './workbench-api'
import {
  loadWorkbenchState,
  mergeRecentWorkspace,
  saveWorkbenchState,
  type RecentWorkspace,
  type WorkbenchSettings,
} from './workbench-state'
import {
  deriveDiagnosticsFromJobs,
  diagnosticDocMapping,
  diagnosticDocLinks,
  diagnosticFamilyLabel,
  diagnosticFamilyOrder,
  type DiagnosticFamily,
  type WorkbenchDiagnostic,
} from './diagnostics'
import './App.css'

type ScreenSpec = {
  path: string
  label: string
  eyebrow: string
  title: string
  summary: string
  current: string[]
  next: string[]
}

type JobRecord = {
  id: string
  kind: JobKind
  label: string
  status: 'running' | 'success' | 'failed'
  commandLine: string
  cwd: string
  resolvedCommand: string[]
  durationMs?: number
  exitCode?: number
  stdout: string
  stderr: string
}

type JobActionSpec = {
  kind: JobKind
  label: string
  args: string[]
  notes: string
  cwdMode: 'repo' | 'workspace'
}

type InspectFamily = 'trace' | 'verify' | 'disasm' | 'verified-run'

type InspectableJob = {
  job: JobRecord
  family: InspectFamily
  artifactPath: string | null
  artifactSource: 'explicit-command-arg' | 'not-explicit'
  summary: string
  summarySource: 'stdout-first-line' | 'stderr-first-line' | 'job-status'
  stdoutText: string | null
  stderrText: string | null
}

type EditorTab = {
  relativePath: string
  absolutePath: string
  title: string
  content: string
  savedContent: string
  status: 'clean' | 'dirty' | 'saving'
}

type PackageManifestPreview = {
  name: string | null
  version: string | null
  edition: string | null
  entry: string | null
}

type SmlspSessionResult = {
  relativePath: string
  command: string
  result: SmlspBridgeResult
}

type EditorCursorPosition = {
  line: number
  character: number
}

type WorkspaceOpenSource = 'default' | 'manual' | 'recent' | 'preset' | 'fallback'

type WorkspaceOpenOptions = {
  persist?: boolean
  source?: WorkspaceOpenSource
  successMessage?: string | null
}

type EditorOpenOptions = {
  line?: number | null
  column?: number | null
  source?: 'diagnostic' | 'definition' | 'scaffold' | 'workspace'
}

type SaveEditorOptions = {
  applyFormatOnSave?: boolean
}

type EditorFocusTarget = {
  relativePath: string
  line: number | null
  column: number | null
  source: NonNullable<EditorOpenOptions['source']>
}

const initialWorkbenchState = loadWorkbenchState()

const workflowActions: JobActionSpec[] = [
  {
    kind: 'cargo',
    label: 'Run workspace tests',
    args: ['test', '--workspace'],
    notes: 'Run the full repository validation suite from the repository root.',
    cwdMode: 'repo',
  },
  {
    kind: 'smc',
    label: 'Compile semantic stress example',
    args: [
      'compile',
      'examples/semantic_policy_overdrive_trace.sm',
      '-o',
      'target/semantic_policy_overdrive_trace.smc',
    ],
    notes: 'Compile the canonical Workbench stress example into SemCode.',
    cwdMode: 'repo',
  },
  {
    kind: 'smc',
    label: 'Run semantic stress source',
    args: ['run', 'examples/semantic_policy_overdrive_trace.sm'],
    notes: 'Run the source example through the public smc surface.',
    cwdMode: 'repo',
  },
  {
    kind: 'smc',
    label: 'Trace semantic stress cache path',
    args: ['check', 'examples/semantic_policy_overdrive_trace.sm', '--trace-cache'],
    notes: 'Inspect the canonical cache-trace surface through smc check --trace-cache.',
    cwdMode: 'repo',
  },
  {
    kind: 'svm',
    label: 'Run compiled semantic stress bytecode',
    args: ['run', 'target/semantic_policy_overdrive_trace.smc'],
    notes: 'Execute the compiled SemCode artifact through svm.',
    cwdMode: 'repo',
  },
  {
    kind: 'svm',
    label: 'Disassemble semantic stress bytecode',
    args: ['disasm', 'target/semantic_policy_overdrive_trace.smc'],
    notes: 'Inspect the compiled SemCode artifact with the public disasm surface.',
    cwdMode: 'repo',
  },
  {
    kind: 'smc',
    label: 'Verify compiled semantic stress bytecode',
    args: ['verify', 'target/semantic_policy_overdrive_trace.smc'],
    notes: 'Verify the compiled SemCode artifact through the canonical smc verify surface.',
    cwdMode: 'repo',
  },
  {
    kind: 'release_bundle_verify',
    label: 'Verify release bundle',
    args: [],
    notes: 'Run the canonical release bundle verification script on the baseline manifest.',
    cwdMode: 'repo',
  },
]

const releaseValidationPlan: JobActionSpec[] = [
  {
    kind: 'cargo',
    label: 'Clean validation: workspace tests',
    args: ['test', '--workspace'],
    notes: 'Run the full repository validation suite from the repository root.',
    cwdMode: 'repo',
  },
  {
    kind: 'smc',
    label: 'Clean validation: compile canonical stress example',
    args: [
      'compile',
      'examples/semantic_policy_overdrive_trace.sm',
      '-o',
      'target/semantic_policy_overdrive_trace.smc',
    ],
    notes: 'Compile the canonical Semantic stress example to a deterministic SemCode artifact.',
    cwdMode: 'repo',
  },
  {
    kind: 'smc',
    label: 'Clean validation: verify canonical SemCode artifact',
    args: ['verify', 'target/semantic_policy_overdrive_trace.smc'],
    notes: 'Run the canonical verifier over the compiled SemCode artifact.',
    cwdMode: 'repo',
  },
  {
    kind: 'svm',
    label: 'Clean validation: disasm canonical SemCode artifact',
    args: ['disasm', 'target/semantic_policy_overdrive_trace.smc'],
    notes: 'Inspect the compiled SemCode artifact through the canonical disassembly surface.',
    cwdMode: 'repo',
  },
  {
    kind: 'svm',
    label: 'Clean validation: run canonical SemCode artifact',
    args: ['run', 'target/semantic_policy_overdrive_trace.smc'],
    notes: 'Execute the compiled SemCode artifact through the verified svm surface.',
    cwdMode: 'repo',
  },
  {
    kind: 'release_bundle_verify',
    label: 'Clean validation: verify release bundle',
    args: [],
    notes: 'Run the canonical release bundle verification script on the baseline manifest.',
    cwdMode: 'repo',
  },
]

const routeSpecs: ScreenSpec[] = [
  {
    path: '/',
    label: 'Overview',
    eyebrow: 'proposed v1 / cockpit',
    title: 'Repository evidence without UI-owned truth.',
    summary:
      'Overview is the cockpit route for branch, baseline, command evidence, readiness visibility, and known limits from repository sources.',
    current: [
      'Branch, commit, and baseline tag cards',
      'Source-of-truth callouts for specs, roadmap, and release artifacts',
      'Presentation-only status summaries over captured repository evidence',
      'Command and job links routed through public smc, svm, cargo, and release-script surfaces',
    ],
    next: [
      'Split jobs and diagnostics into richer structured views',
      'Add spec navigation without mutating canonical docs',
    ],
  },
  {
    path: '/project',
    label: 'Project',
    eyebrow: 'proposed v1 / workspace',
    title: 'Workspace view without package-model authority.',
    summary:
      'Project owns workspace selection, repository-relative file navigation, and local project context. It does not create alternate package or repository truth.',
    current: [
      'Workspace resolver over canonical repository paths',
      'Recent projects list and default workspace persistence',
      'Canonical project bootstrap for Semantic.toml, src/main.sm, and examples/smoke.sm',
      'Read-only package metadata preview derived from Semantic.toml',
      'Workspace file tree and local UI state kept separate from repository authority',
    ],
    next: [
      'Keep editor actions routed through public smc commands',
      'Preserve local cache staleness instead of hiding changed repository state',
    ],
  },
  {
    path: '/editor',
    label: 'Editor',
    eyebrow: 'proposed v1 / editor shell',
    title: 'Authoring shell without compiler ownership.',
    summary:
      'Editor provides file tabs, dirty markers, save/reload, and current-file actions through public commands. Syntax highlighting remains presentation only.',
    current: [
      'Text editor tabs and dirty markers as local UI state',
      'Current-file check and compile actions through public smc commands',
      'Formatter actions routed through the documented public formatter surface when available',
      'Command output shown through execution/feedback rules instead of editor-owned diagnostics truth',
    ],
    next: [
      'Keep parser, typechecker, compiler diagnostics, and formatting semantics outside Workbench',
      'Make unsupported syntax visible through emitted command output, not optimistic UI labels',
    ],
  },
  {
    path: '/spec',
    label: 'Spec',
    eyebrow: 'read-only / docs navigator',
    title: 'Read-only entry into the contract bundle.',
    summary:
      'Spec navigation is a presentation layer over docs/spec and docs/roadmap. Workbench points at the documents; it does not fork them.',
    current: [
      'Docs entry cards for language, execution, and release anchors',
      'Canonical tree over docs/spec, docs/roadmap, and synced language overview documents',
      'Title/path search and section navigator driven by repository markdown',
      'Source-path discipline called out directly in the UI',
    ],
    next: [
      'Add freshness hints and stronger release-document callouts',
      'Keep navigator read-only even when authoring shell arrives',
    ],
  },
  {
    path: '/diagnostics',
    label: 'Diagnostics',
    eyebrow: 'proposed v1 / emitted diagnostics',
    title: 'Emitted diagnostics, not UI-owned semantics.',
    summary:
      'Diagnostics groups parse, type, module, verify, and runtime outputs emitted by public commands. It does not duplicate parser or verifier semantics.',
    current: [
      'Dedicated diagnostics route and panel shell',
      'Family buckets for parse, type, module, verify, and runtime',
      'Space reserved for spec-linked error drilldowns',
    ],
    next: [
      'Feed diagnostics from command results',
      'Preserve code, severity, file, and range fields exactly',
    ],
  },
  {
    path: '/inspect',
    label: 'Inspect',
    eyebrow: 'read-only / output views',
    title: 'Disasm, verify, and trace output views.',
    summary:
      'Inspect is where Workbench will render SemCode, verifier output, and runtime summaries. It stays downstream from existing execution contracts.',
    current: [
      'Dedicated trace, verify, disasm, and verified-run inspectors over real CLI jobs',
      'Dedicated inspector over smc verify, svm disasm, and verified-run jobs',
      'Raw command output is preserved as the only bytecode, verifier, and runtime source of truth',
      'Clear note that source-level debugging is not promised yet',
    ],
    next: [
      'Extend the same inspect route with richer quota and capability context when public outputs grow',
      'Add trace and runtime summaries without inventing VM semantics',
    ],
  },
  {
    path: '/release',
    label: 'Release',
    eyebrow: 'visibility-only / release console',
    title: 'Release visibility without release authority.',
    summary:
      'Release shows gate, bundle, asset-smoke, docs-alignment, and known-limit visibility. Final release decisions remain outside Workbench.',
    current: [
      'Release route anchored around canonical release docs and baseline artifacts',
      'Known-limits panel separated from pass/fail gates',
      'Freshness hints and source paths shown for release-facing documents',
    ],
    next: [
      'Export a validation report from real job history',
      'Keep release map in sync with future operate slices without adding UI-owned scoring',
    ],
  },
  {
    path: '/settings',
    label: 'Settings',
    eyebrow: 'local UI state',
    title: 'Local preferences, not hidden feature flags.',
    summary:
      'Settings is for shell-level behavior only: display, formatter preferences, shell defaults, and workspace affordances. It must not widen Semantic scope.',
    current: [
      'Settings route with persisted local preferences',
      'Scope guard against hidden runtime or language toggles',
      'Formatter and shell preference toggles wired only to canonical public surfaces',
    ],
    next: [
      'Route settings into later formatter and command surfaces',
      'Keep experimentation visibly labeled and opt-in',
    ],
  },
]

function App() {
  const [adapterContract, setAdapterContract] = useState<AdapterContract | null>(
    null,
  )
  const [overviewSnapshot, setOverviewSnapshot] = useState<OverviewSnapshot | null>(
    null,
  )
  const [adapterError, setAdapterError] = useState<string | null>(null)
  const [snapshotError, setSnapshotError] = useState<string | null>(null)
  const [specError, setSpecError] = useState<string | null>(null)
  const [jobs, setJobs] = useState<JobRecord[]>([])
  const [selectedJobId, setSelectedJobId] = useState<string | null>(null)
  const [runningActionKeys, setRunningActionKeys] = useState<string[]>([])
  const [specCatalog, setSpecCatalog] = useState<SpecCatalogSection[]>([])
  const [specSearch, setSpecSearch] = useState('')
  const [selectedSpecPath, setSelectedSpecPath] = useState<string | null>(null)
  const [selectedSpecDocument, setSelectedSpecDocument] =
    useState<SpecDocumentView | null>(null)
  const [workspaceTree, setWorkspaceTree] = useState<WorkspaceTreeNode[]>([])
  const [workspaceTreeError, setWorkspaceTreeError] = useState<string | null>(null)
  const [workspaceTreeBusy, setWorkspaceTreeBusy] = useState(false)
  const [editorTabs, setEditorTabs] = useState<EditorTab[]>([])
  const [activeEditorPath, setActiveEditorPath] = useState<string | null>(null)
  const [editorFocusTarget, setEditorFocusTarget] = useState<EditorFocusTarget | null>(null)
  const [workspaceInput, setWorkspaceInput] = useState('')
  const [workspaceError, setWorkspaceError] = useState<string | null>(null)
  const [workspaceNotice, setWorkspaceNotice] = useState<string | null>(null)
  const [workspaceBusy, setWorkspaceBusy] = useState(false)
  const [workspaceSource, setWorkspaceSource] = useState<WorkspaceOpenSource | null>(null)
  const [selectedWorkspace, setSelectedWorkspace] = useState<WorkspaceSummary | null>(
    null,
  )
  const [recentWorkspaces, setRecentWorkspaces] = useState<RecentWorkspace[]>(
    initialWorkbenchState.recentWorkspaces,
  )
  const [settings, setSettings] = useState<WorkbenchSettings>(
    initialWorkbenchState.settings,
  )
  const workspaceTreeRequestIdRef = useRef(0)
  const runningActionJobIdsRef = useRef(new Map<string, string>())

  useEffect(() => {
    let cancelled = false

    fetchAdapterContract()
      .then((contract) => {
        if (!cancelled) {
          startTransition(() => setAdapterContract(contract))
          setAdapterError(null)
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setAdapterError(String(error))
        }
      })

    return () => {
      cancelled = true
    }
  }, [])

  useEffect(() => {
    let cancelled = false

    fetchOverviewSnapshot()
      .then((snapshot) => {
        if (!cancelled) {
          startTransition(() => setOverviewSnapshot(snapshot))
          setSnapshotError(null)
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setSnapshotError(String(error))
        }
      })

    return () => {
      cancelled = true
    }
  }, [])

  useEffect(() => {
    let cancelled = false

    fetchSpecCatalog()
      .then((catalog) => {
        if (!cancelled) {
          setSpecCatalog(catalog)
          setSpecError(null)
          const firstPath = catalog[0]?.documents[0]?.relativePath
          if (firstPath) {
            setSelectedSpecPath((current) => current ?? firstPath)
          }
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setSpecError(String(error))
        }
      })

    return () => {
      cancelled = true
    }
  }, [])

  useEffect(() => {
    if (!selectedSpecPath) {
      return
    }

    let cancelled = false

    fetchSpecDocument(selectedSpecPath)
      .then((document) => {
        if (!cancelled) {
          setSelectedSpecDocument(document)
          setSpecError(null)
        }
      })
      .catch((error) => {
        if (!cancelled) {
          setSpecError(String(error))
        }
      })

    return () => {
      cancelled = true
    }
  }, [selectedSpecPath])

  useEffect(() => {
    if (!selectedWorkspace?.resolvedPath) {
      setWorkspaceTree([])
      setWorkspaceTreeError(null)
      setWorkspaceTreeBusy(false)
      return
    }

    startTransition(() => {
      setEditorTabs([])
      setActiveEditorPath(null)
      setWorkspaceTree([])
      setWorkspaceTreeError(null)
    })

    void loadWorkspaceTree(selectedWorkspace.resolvedPath)
  }, [selectedWorkspace?.resolvedPath])

  useEffect(() => {
    saveWorkbenchState({
      recentWorkspaces,
      settings,
    })
  }, [recentWorkspaces, settings])

  useEffect(() => {
    if (!adapterContract || selectedWorkspace) {
      return
    }

    const initialWorkspacePath =
      settings.defaultWorkspacePath ?? adapterContract.repoRoot

    void (async () => {
      setWorkspaceBusy(true)
      try {
        const workspace = await resolveWorkspaceRoot(initialWorkspacePath)
        setWorkspaceError(null)
        setWorkspaceNotice(
          settings.defaultWorkspacePath
            ? 'Restored the saved default workspace for this session.'
            : 'Using the repository root as the current workspace.',
        )
        setSelectedWorkspace(workspace)
        setWorkspaceSource(settings.defaultWorkspacePath ? 'default' : 'preset')
        setWorkspaceInput(workspace.resolvedPath)
      } catch (error) {
        if (!settings.defaultWorkspacePath) {
          setWorkspaceError(describeWorkspaceOpenError(initialWorkspacePath, error))
          setWorkspaceNotice(null)
          setWorkspaceSource(null)
          return
        }

        try {
          const fallbackWorkspace = await resolveWorkspaceRoot(adapterContract.repoRoot)
          setWorkspaceError(null)
          setWorkspaceNotice(
            `Saved default workspace could not be restored. Workbench fell back to the repository root. ${describeWorkspaceOpenError(initialWorkspacePath, error)}`,
          )
          setSelectedWorkspace(fallbackWorkspace)
          setWorkspaceSource('fallback')
          setWorkspaceInput(fallbackWorkspace.resolvedPath)
          setRecentWorkspaces((current) => mergeRecentWorkspace(current, fallbackWorkspace))
          setSettings((current) => ({
            ...current,
            defaultWorkspacePath: fallbackWorkspace.resolvedPath,
          }))
        } catch (fallbackError) {
          setWorkspaceError(
            `Saved default workspace could not be restored, and repository root fallback failed. ${describeWorkspaceOpenError(adapterContract.repoRoot, fallbackError)}`,
          )
          setWorkspaceNotice(null)
          setWorkspaceSource(null)
        }
      } finally {
        setWorkspaceBusy(false)
      }
    })()
  }, [adapterContract, selectedWorkspace, settings.defaultWorkspacePath])

  async function runJobAction(action: JobActionSpec): Promise<JobResult | null> {
    const cwd = resolveActionCwd(action, adapterContract, selectedWorkspace)
    const actionKey = createActionExecutionKey(action.kind, cwd, action.args)
    const existingJobId = runningActionJobIdsRef.current.get(actionKey)

    if (existingJobId) {
      setSelectedJobId(existingJobId)
      return null
    }

    const id = crypto.randomUUID()
    runningActionJobIdsRef.current.set(actionKey, id)
    setRunningActionKeys((current) =>
      current.includes(actionKey) ? current : [...current, actionKey],
    )

    startTransition(() =>
      setJobs((current) => [
        {
          id,
          kind: action.kind,
          label: action.label,
          status: 'running',
          commandLine: [action.label, ...action.args].join(' '),
          cwd,
          resolvedCommand: [],
          stdout: '',
          stderr: '',
        },
        ...current,
      ]),
    )
    setSelectedJobId(id)

    try {
      const result = await runCliJob({
        kind: action.kind,
        args: action.args,
        cwd,
      })
      setAdapterError(null)
      commitJob(id, action.label, result)
      return result
    } catch (error) {
      const message = String(error)
      startTransition(() =>
        setJobs((current) =>
          current.map((job) =>
            job.id === id
              ? {
                  ...job,
                  status: 'failed',
                  stderr: message,
                }
              : job,
          ),
        ),
      )
      setAdapterError(message)
      return null
    } finally {
      if (runningActionJobIdsRef.current.get(actionKey) === id) {
        runningActionJobIdsRef.current.delete(actionKey)
      }
      setRunningActionKeys((current) => current.filter((entry) => entry !== actionKey))
    }
  }

  async function runProbe(spec: AdapterJobSpec) {
    await runJobAction({
      kind: spec.kind,
      label: `${spec.label} probe`,
      args: spec.exampleArgs,
      notes: spec.notes,
      cwdMode: 'workspace',
    })
  }

  function commitJob(id: string, label: string, result: JobResult) {
    startTransition(() =>
      setJobs((current) =>
        current.map((job) =>
          job.id === id
            ? {
                ...job,
                label,
                status: result.success ? 'success' : 'failed',
                commandLine: result.resolvedCommand.join(' '),
                cwd: result.cwd,
                resolvedCommand: result.resolvedCommand,
                durationMs: result.durationMs,
                exitCode: result.exitCode,
                stdout: result.stdout,
                stderr: result.stderr,
              }
            : job,
        ),
      ),
    )
  }

  function isActionRunning(action: JobActionSpec) {
    const cwd = resolveActionCwd(action, adapterContract, selectedWorkspace)
    const actionKey = createActionExecutionKey(action.kind, cwd, action.args)
    return runningActionKeys.includes(actionKey)
  }

  function updateWorkspaceInput(value: string) {
    setWorkspaceInput(value)
    setWorkspaceError(null)
    if (workspaceSource !== 'fallback') {
      setWorkspaceNotice(null)
    }
  }

  async function openWorkspace(candidate: string, optionsInput?: boolean | WorkspaceOpenOptions) {
    const options = normalizeWorkspaceOpenOptions(optionsInput)
    const persist = options.persist ?? true
    const source = options.source ?? 'manual'
    const normalizedCandidate = candidate.trim()

    if (!normalizedCandidate) {
      setWorkspaceError('Enter an absolute path or a repository-relative path before opening a workspace.')
      setWorkspaceNotice(null)
      return
    }

    setWorkspaceBusy(true)
    setWorkspaceError(null)
    if (source !== 'fallback') {
      setWorkspaceNotice(null)
    }

    try {
      const workspace = await resolveWorkspaceRoot(normalizedCandidate)
      setWorkspaceError(null)
      setSelectedWorkspace(workspace)
      setWorkspaceSource(source)
      setWorkspaceInput(workspace.resolvedPath)
      if (persist) {
        setRecentWorkspaces((current) => mergeRecentWorkspace(current, workspace))
      }
      setSettings((current) => ({
        ...current,
        defaultWorkspacePath: workspace.resolvedPath,
      }))
      setWorkspaceNotice(
        options.successMessage ?? defaultWorkspaceSuccessMessage(workspace, source),
      )
    } catch (error) {
      setWorkspaceError(describeWorkspaceOpenError(normalizedCandidate, error))
    } finally {
      setWorkspaceBusy(false)
    }
  }

  async function loadWorkspaceTree(workspaceRoot: string) {
    const requestId = workspaceTreeRequestIdRef.current + 1
    workspaceTreeRequestIdRef.current = requestId
    setWorkspaceTreeBusy(true)

    try {
      const tree = await fetchWorkspaceTree(workspaceRoot)
      if (workspaceTreeRequestIdRef.current !== requestId) {
        return
      }

      setWorkspaceTree(tree)
      setWorkspaceTreeError(null)
    } catch (error) {
      if (workspaceTreeRequestIdRef.current !== requestId) {
        return
      }

      setWorkspaceTreeError(String(error))
    } finally {
      if (workspaceTreeRequestIdRef.current === requestId) {
        setWorkspaceTreeBusy(false)
      }
    }
  }

  async function refreshWorkspaceTree() {
    if (!selectedWorkspace?.resolvedPath) {
      return
    }

    await loadWorkspaceTree(selectedWorkspace.resolvedPath)
  }

  async function openEditorFile(relativePath: string, options?: EditorOpenOptions) {
    if (!selectedWorkspace) {
      return
    }

    const nextFocusTarget =
      options?.source && (options.line !== undefined || options.column !== undefined)
        ? {
            relativePath,
            line: options.line ?? null,
            column: options.column ?? null,
            source: options.source,
          }
        : null

    const existingTab = editorTabs.find((tab) => tab.relativePath === relativePath)
    if (existingTab) {
      setActiveEditorPath(relativePath)
      setEditorFocusTarget(nextFocusTarget)
      return
    }

    try {
      const document = await fetchWorkspaceFile({
        workspaceRoot: selectedWorkspace.resolvedPath,
        relativePath,
      })

        startTransition(() => {
          setEditorTabs((current) => [
            ...current,
            createEditorTab(document),
          ])
          setActiveEditorPath(relativePath)
          setEditorFocusTarget(nextFocusTarget)
          setWorkspaceTreeError(null)
        })
      } catch (error) {
        setWorkspaceTreeError(String(error))
      }
  }

  function updateEditorContent(relativePath: string, content: string) {
    setEditorTabs((current) =>
      current.map((tab) =>
        tab.relativePath === relativePath
          ? {
              ...tab,
              content,
              status: content === tab.savedContent ? 'clean' : 'dirty',
            }
          : tab,
      ),
    )
  }

  async function saveEditorFile(relativePath: string, options?: SaveEditorOptions) {
    if (!selectedWorkspace) {
      return false
    }

    const tab = editorTabs.find((entry) => entry.relativePath === relativePath)
    if (!tab) {
      return false
    }

    setEditorTabs((current) =>
      current.map((entry) =>
        entry.relativePath === relativePath
          ? { ...entry, status: 'saving' }
          : entry,
      ),
    )

    try {
      const document = await saveWorkspaceFile({
        workspaceRoot: selectedWorkspace.resolvedPath,
        relativePath,
        content: tab.content,
      })
      let nextDocument = document

      const applyFormatOnSave = options?.applyFormatOnSave ?? settings.formatOnSave

      if (applyFormatOnSave && isSemanticSource(relativePath)) {
        const repoRelativePath = toRepoRelativePath(relativePath, selectedWorkspace)
        const formatResult = await runJobAction({
          kind: 'smc',
          label: `Format ${document.relativePath}`,
          args: ['fmt', repoRelativePath],
          notes: 'Format the current Semantic source file through the canonical smc fmt surface.',
          cwdMode: 'repo',
        })

        if (formatResult?.success) {
          nextDocument = await fetchWorkspaceFile({
            workspaceRoot: selectedWorkspace.resolvedPath,
            relativePath,
          })
        }
      }

      setEditorTabs((current) =>
        current.map((entry) =>
          entry.relativePath === relativePath ? createEditorTab(nextDocument) : entry,
        ),
      )
      setWorkspaceTreeError(null)
      return true
    } catch (error) {
      setWorkspaceTreeError(String(error))
      setEditorTabs((current) =>
        current.map((entry) =>
          entry.relativePath === relativePath
            ? { ...entry, status: entry.content === entry.savedContent ? 'clean' : 'dirty' }
            : entry,
        ),
      )
      return false
    }
  }

  async function reloadEditorFile(relativePath: string) {
    if (!selectedWorkspace) {
      return
    }

    try {
      const document = await fetchWorkspaceFile({
        workspaceRoot: selectedWorkspace.resolvedPath,
        relativePath,
      })

      setEditorTabs((current) =>
        current.map((entry) =>
          entry.relativePath === relativePath
            ? createEditorTab(document)
            : entry,
        ),
      )
      setWorkspaceTreeError(null)
    } catch (error) {
      setWorkspaceTreeError(String(error))
    }
  }

  function closeEditorTab(relativePath: string) {
    setEditorTabs((current) => {
      const remaining = current.filter((tab) => tab.relativePath !== relativePath)
      if (activeEditorPath === relativePath) {
        setActiveEditorPath(remaining[remaining.length - 1]?.relativePath ?? null)
      }
      return remaining
    })
  }

  function updateSettings(next: Partial<WorkbenchSettings>) {
    setSettings((current) => ({
      ...current,
      ...next,
    }))
  }

  return (
    <div className="workbench-shell">
      <aside className="sidebar">
        <div className="brand-block">
          <p className="eyebrow">Semantic Workbench</p>
          <h1>Workbench shell for orchestration, not reinvention.</h1>
          <p className="brand-copy">
            This bootstrap slice locks the desktop shell, route map, and public-surface discipline before any command or editor logic arrives.
          </p>
        </div>

        <nav className="primary-nav" aria-label="Workbench routes">
          {routeSpecs.map((route) => (
            <NavLink
              key={route.path}
              to={route.path}
              end={route.path === '/'}
              className={({ isActive }) =>
                isActive ? 'nav-link nav-link-active' : 'nav-link'
              }
            >
              <span className="nav-label">{route.label}</span>
              <span className="nav-meta">{route.eyebrow}</span>
            </NavLink>
          ))}
        </nav>

        <section className="sidebar-card">
          <p className="card-kicker">Source of truth</p>
          <ul className="bullet-list compact">
            <li>`docs/spec/*` and `docs/roadmap/*` remain canonical.</li>
            <li>`smc`, `svm`, `cargo`, and release scripts stay the first integration path.</li>
            <li>Workbench owns only UI state, orchestration, and presentation caches.</li>
            <li>Local Admission Guard remains authoritative; GitHub CI is not a Workbench gate.</li>
          </ul>
        </section>

        <section className="sidebar-card">
          <p className="card-kicker">Workspace context</p>
          <p className="sidebar-strong">
            {selectedWorkspace?.repoRelativePath ?? 'repository root'}
          </p>
          <p className="sidebar-copy">
            {selectedWorkspace?.resolvedPath ??
              adapterContract?.repoRoot ??
              'Loading workspace root...'}
          </p>
        </section>
      </aside>

      <main className="main-panel">
        <header className="topbar">
          <div>
            <p className="eyebrow">Workbench foundation</p>
            <h2>Route map aligned to repository truth</h2>
          </div>
          <div className="status-cluster">
            <span className="status-pill stable">Presentation/orchestration only</span>
            <span className="status-pill draft">Current beta evidence, not production-ready</span>
          </div>
        </header>

        <section className="hero-grid">
          <article className="hero-card">
            <p className="card-kicker">Current slice</p>
            <h3>Foundation route families now match the readiness docs</h3>
            <p>
              The shell exposes Overview, Project, Editor, Diagnostics, Spec, Inspect, Release, and Settings as bounded Workbench v1 route families.
            </p>
          </article>
          <article className="hero-card">
            <p className="card-kicker">Do not cross</p>
            <h3>No alternate readiness model inside the UI</h3>
            <p>
              Workbench presents repository evidence. Commands, docs, scripts, captured outputs, and local Admission Guard remain authoritative.
            </p>
          </article>
          <article className="hero-card">
            <p className="card-kicker">Immediate next</p>
            <h3>Implement one route family at a time</h3>
            <p>
              Future slices can fill routes without moving parser, typechecker, verifier, VM, runtime, or release authority into Workbench.
            </p>
          </article>
        </section>

        <Routes>
          {routeSpecs.map((route) => (
            <Route
              key={route.path}
              path={route.path}
              element={
                <WorkbenchScreen
                  route={route}
                  adapterContract={adapterContract}
                  overviewSnapshot={overviewSnapshot}
                  adapterError={adapterError}
                  snapshotError={snapshotError}
                  specCatalog={specCatalog}
                  specError={specError}
                  specSearch={specSearch}
                  selectedSpecDocument={selectedSpecDocument}
                  selectedSpecPath={selectedSpecPath}
                  workspaceTree={workspaceTree}
                  workspaceTreeBusy={workspaceTreeBusy}
                  workspaceTreeError={workspaceTreeError}
                  editorTabs={editorTabs}
                  activeEditorPath={activeEditorPath}
                  editorFocusTarget={editorFocusTarget}
                  jobs={jobs}
                  selectedJobId={selectedJobId}
                  onRunAction={runJobAction}
                  isActionRunning={isActionRunning}
                  onRunProbe={runProbe}
                  onSpecSearchChange={setSpecSearch}
                  onSelectSpecPath={setSelectedSpecPath}
                  onOpenEditorFile={openEditorFile}
                  onSelectEditorPath={setActiveEditorPath}
                  onUpdateEditorContent={updateEditorContent}
                  onRefreshWorkspace={refreshWorkspaceTree}
                  onSaveEditorFile={saveEditorFile}
                  onReloadEditorFile={reloadEditorFile}
                  onCloseEditorTab={closeEditorTab}
                  onClearEditorFocusTarget={() => setEditorFocusTarget(null)}
                  onSelectJob={setSelectedJobId}
                  selectedWorkspace={selectedWorkspace}
                  workspaceInput={workspaceInput}
                  workspaceError={workspaceError}
                  workspaceNotice={workspaceNotice}
                  workspaceBusy={workspaceBusy}
                  workspaceSource={workspaceSource}
                  recentWorkspaces={recentWorkspaces}
                  settings={settings}
                  onWorkspaceInputChange={updateWorkspaceInput}
                  onOpenWorkspace={openWorkspace}
                  onUpdateSettings={updateSettings}
                />
              }
            />
          ))}
        </Routes>
      </main>
    </div>
  )
}

function WorkbenchScreen({
  route,
  adapterContract,
  overviewSnapshot,
  adapterError,
  snapshotError,
  specCatalog,
  specError,
  specSearch,
  selectedSpecDocument,
  selectedSpecPath,
  workspaceTree,
  workspaceTreeBusy,
  workspaceTreeError,
  editorTabs,
  activeEditorPath,
  editorFocusTarget,
  jobs,
  selectedJobId,
  onRunAction,
  isActionRunning,
  onRunProbe,
  onSpecSearchChange,
  onSelectSpecPath,
  onOpenEditorFile,
  onSelectEditorPath,
  onUpdateEditorContent,
  onRefreshWorkspace,
  onSaveEditorFile,
  onReloadEditorFile,
  onCloseEditorTab,
  onClearEditorFocusTarget,
  onSelectJob,
  selectedWorkspace,
  workspaceInput,
  workspaceError,
  workspaceNotice,
  workspaceBusy,
  workspaceSource,
  recentWorkspaces,
  settings,
  onWorkspaceInputChange,
  onOpenWorkspace,
  onUpdateSettings,
}: {
  route: ScreenSpec
  adapterContract: AdapterContract | null
  overviewSnapshot: OverviewSnapshot | null
  adapterError: string | null
  snapshotError: string | null
  specCatalog: SpecCatalogSection[]
  specError: string | null
  specSearch: string
  selectedSpecDocument: SpecDocumentView | null
  selectedSpecPath: string | null
  workspaceTree: WorkspaceTreeNode[]
  workspaceTreeBusy: boolean
  workspaceTreeError: string | null
  editorTabs: EditorTab[]
  activeEditorPath: string | null
  editorFocusTarget: EditorFocusTarget | null
  jobs: JobRecord[]
  selectedJobId: string | null
  onRunAction: (action: JobActionSpec) => Promise<JobResult | null>
  isActionRunning: (action: JobActionSpec) => boolean
  onRunProbe: (spec: AdapterJobSpec) => Promise<void>
  onSpecSearchChange: (value: string) => void
  onSelectSpecPath: (value: string) => void
  onOpenEditorFile: (relativePath: string, options?: EditorOpenOptions) => Promise<void>
  onSelectEditorPath: (relativePath: string | null) => void
  onUpdateEditorContent: (relativePath: string, content: string) => void
  onRefreshWorkspace: () => Promise<void>
  onSaveEditorFile: (relativePath: string, options?: SaveEditorOptions) => Promise<boolean>
  onReloadEditorFile: (relativePath: string) => Promise<void>
  onCloseEditorTab: (relativePath: string) => void
  onClearEditorFocusTarget: () => void
  onSelectJob: (jobId: string) => void
  selectedWorkspace: WorkspaceSummary | null
  workspaceInput: string
  workspaceError: string | null
  workspaceNotice: string | null
  workspaceBusy: boolean
  workspaceSource: WorkspaceOpenSource | null
  recentWorkspaces: RecentWorkspace[]
  settings: WorkbenchSettings
  onWorkspaceInputChange: (value: string) => void
  onOpenWorkspace: (
    candidate: string,
    options?: boolean | WorkspaceOpenOptions,
  ) => Promise<void>
  onUpdateSettings: (next: Partial<WorkbenchSettings>) => void
}) {
  return (
    <div className="screen-stack">
      <section className="screen-grid">
        <article className="screen-card screen-card-primary">
          <p className="card-kicker">{route.eyebrow}</p>
          <h3>{route.title}</h3>
          <p className="screen-summary">{route.summary}</p>
        </article>

        <article className="screen-card">
          <p className="card-kicker">In this slice</p>
          <ul className="bullet-list">
            {route.current.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Next implementation steps</p>
          <ul className="bullet-list">
            {route.next.map((item) => (
              <li key={item}>{item}</li>
            ))}
          </ul>
        </article>
      </section>

      {route.path === '/' ? (
        <CommandBusPanel
          adapterContract={adapterContract}
          overviewSnapshot={overviewSnapshot}
          adapterError={adapterError}
          snapshotError={snapshotError}
          jobs={jobs}
          selectedJobId={selectedJobId}
          onRunAction={onRunAction}
          isActionRunning={isActionRunning}
          onRunProbe={onRunProbe}
          onSelectJob={onSelectJob}
          selectedWorkspace={selectedWorkspace}
        />
      ) : null}

      {route.path === '/spec' ? (
        <SpecNavigatorPanel
          specCatalog={specCatalog}
          specError={specError}
          specSearch={specSearch}
          selectedSpecDocument={selectedSpecDocument}
          selectedSpecPath={selectedSpecPath}
          onSpecSearchChange={onSpecSearchChange}
          onSelectSpecPath={onSelectSpecPath}
        />
      ) : null}

      {route.path === '/release' ? (
        <ReleasePanel
          overviewSnapshot={overviewSnapshot}
          specCatalog={specCatalog}
          jobs={jobs}
          selectedWorkspace={selectedWorkspace}
          onRunAction={onRunAction}
        />
      ) : null}

      {route.path === '/project' || route.path === '/editor' ? (
        <ProjectPanel
          adapterContract={adapterContract}
          selectedWorkspace={selectedWorkspace}
          workspaceTree={workspaceTree}
          workspaceTreeBusy={workspaceTreeBusy}
          workspaceTreeError={workspaceTreeError}
          jobs={jobs}
          editorTabs={editorTabs}
          activeEditorPath={activeEditorPath}
          editorFocusTarget={editorFocusTarget}
          workspaceInput={workspaceInput}
          workspaceError={workspaceError}
          workspaceNotice={workspaceNotice}
          workspaceBusy={workspaceBusy}
          workspaceSource={workspaceSource}
          recentWorkspaces={recentWorkspaces}
          settings={settings}
          onWorkspaceInputChange={onWorkspaceInputChange}
          onOpenWorkspace={onOpenWorkspace}
  onOpenEditorFile={onOpenEditorFile}
  onSelectEditorPath={onSelectEditorPath}
  onUpdateEditorContent={onUpdateEditorContent}
          onRunAction={onRunAction}
  isActionRunning={isActionRunning}
  onRefreshWorkspace={onRefreshWorkspace}
          onSaveEditorFile={onSaveEditorFile}
          onReloadEditorFile={onReloadEditorFile}
          onCloseEditorTab={onCloseEditorTab}
          onClearEditorFocusTarget={onClearEditorFocusTarget}
          onSelectJob={onSelectJob}
        />
      ) : null}

      {route.path === '/diagnostics' ? (
        <DiagnosticsPanel
          jobs={jobs}
          selectedJobId={selectedJobId}
          selectedWorkspace={selectedWorkspace}
          onOpenEditorFile={onOpenEditorFile}
          onSelectJob={onSelectJob}
          onSelectSpecPath={onSelectSpecPath}
        />
      ) : null}

      {route.path === '/inspect' ? (
        <InspectPanel
          jobs={jobs}
          selectedJobId={selectedJobId}
          onSelectJob={onSelectJob}
        />
      ) : null}

      {route.path === '/settings' ? (
        <SettingsPanel
          settings={settings}
          selectedWorkspace={selectedWorkspace}
          onUpdateSettings={onUpdateSettings}
        />
      ) : null}
    </div>
  )
}

function CommandBusPanel({
  adapterContract,
  overviewSnapshot,
  adapterError,
  snapshotError,
  jobs,
  selectedJobId,
  onRunAction,
  isActionRunning,
  onRunProbe,
  onSelectJob,
  selectedWorkspace,
}: {
  adapterContract: AdapterContract | null
  overviewSnapshot: OverviewSnapshot | null
  adapterError: string | null
  snapshotError: string | null
  jobs: JobRecord[]
  selectedJobId: string | null
  onRunAction: (action: JobActionSpec) => Promise<JobResult | null>
  isActionRunning: (action: JobActionSpec) => boolean
  onRunProbe: (spec: AdapterJobSpec) => Promise<void>
  onSelectJob: (jobId: string) => void
  selectedWorkspace: WorkspaceSummary | null
}) {
  const latestCargo = latestJobOfKind(jobs, 'cargo')
  const latestSmc = latestJobOfKind(jobs, 'smc')
  const latestSvm = latestJobOfKind(jobs, 'svm')
  const latestBundle = latestJobOfKind(jobs, 'release_bundle_verify')
  const selectedJob = jobs.find((job) => job.id === selectedJobId) ?? jobs[0]

  return (
    <div className="screen-stack">
      <section className="overview-grid">
        <article className="screen-card">
          <p className="card-kicker">Repository snapshot</p>
          <h3>Current git and baseline identity</h3>
          {snapshotError ? <p className="adapter-error">{snapshotError}</p> : null}
          <dl className="facts-grid">
            <div>
              <dt>Branch</dt>
              <dd>{overviewSnapshot?.branch ?? 'Loading...'}</dd>
            </div>
            <div>
              <dt>Commit</dt>
              <dd>
                <code>{overviewSnapshot?.shortCommit ?? 'Loading...'}</code>
              </dd>
            </div>
            <div>
              <dt>Head tags</dt>
              <dd>
                {overviewSnapshot?.headTags.length ? (
                  overviewSnapshot.headTags.join(', ')
                ) : (
                  'No tags on HEAD'
                )}
              </dd>
            </div>
            <div>
              <dt>Baseline tag</dt>
              <dd>
                {overviewSnapshot
                  ? overviewSnapshot.baselineTagPointsAtHead
                    ? `${overviewSnapshot.baselineTagName} on HEAD`
                    : `${overviewSnapshot.baselineTagName} exists off HEAD`
                  : 'Loading...'}
              </dd>
            </div>
            <div className="facts-grid-wide">
              <dt>Baseline manifest</dt>
              <dd>
                {overviewSnapshot?.baselineManifestExists ? 'Present' : 'Missing'}
                {overviewSnapshot ? (
                  <>
                    {' '}
                    <code>{overviewSnapshot.baselineManifestPath}</code>
                  </>
                ) : null}
              </dd>
            </div>
            <div className="facts-grid-wide">
              <dt>Workspace root</dt>
              <dd>
                <code>{selectedWorkspace?.resolvedPath ?? adapterContract?.repoRoot ?? 'Loading...'}</code>
              </dd>
            </div>
          </dl>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Baseline guardrails</p>
          <h3>What the cockpit is allowed to claim</h3>
          <ul className="bullet-list">
            <li>Only git state, release docs, manifests, and real job executions drive the overview.</li>
            <li>Known limits stay visible even when the last local commands were green.</li>
            <li>Published readiness still belongs to repository artifacts, not to cached UI percentages.</li>
          </ul>
        </article>
      </section>

      <section className="overview-grid">
        <article className="screen-card">
          <p className="card-kicker">Release docs</p>
          <h3>Readiness and compatibility pointers</h3>
          <div className="document-list">
            {(overviewSnapshot?.releaseDocs ?? []).map((document) => (
              <section key={document.key} className="document-card">
                <div className="document-topline">
                  <strong>{document.title}</strong>
                  {document.status ? (
                    <span className="status-pill stable">{document.status}</span>
                  ) : null}
                </div>
                <p className="job-meta">
                  <code>{document.path}</code>
                </p>
                {document.highlight ? (
                  <p className="document-highlight">{document.highlight}</p>
                ) : null}
              </section>
            ))}
          </div>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Current known limits</p>
          <h3>Honesty rules carried from readiness</h3>
          <ul className="bullet-list">
            {(overviewSnapshot?.knownLimits ?? []).map((limit) => (
              <li key={limit}>{limit}</li>
            ))}
          </ul>
        </article>
      </section>

      <section className="command-grid">
        <article className="screen-card">
          <p className="card-kicker">Command runner</p>
          <h3>Core workflows through approved public surfaces</h3>
          <p className="screen-summary">
            These actions run real repository workflows through the backend adapter. Repository-wide validation always executes from the repository root.
          </p>
          <div className="workflow-grid">
            {workflowActions.map((action) => {
              const actionRunning = isActionRunning(action)

              return (
              <section key={action.label} className="workflow-card">
                <div className="adapter-header">
                  <h4>{action.label}</h4>
                  <span className={`status-pill ${actionRunning ? 'running' : 'stable'}`}>
                    {actionRunning ? 'running' : action.kind}
                  </span>
                </div>
                <p>{action.notes}</p>
                <p className="job-meta">
                  scope:{' '}
                  <strong>
                    {action.cwdMode === 'repo' ? 'repository root' : 'current workspace'}
                  </strong>
                </p>
                <code className="code-block">
                  {action.args.join(' ') || '(default script args)'}
                </code>
                <button
                  type="button"
                  className="action-button"
                  onClick={() => void onRunAction(action)}
                  disabled={actionRunning}
                >
                  {actionRunning ? 'Running command...' : `Run ${action.label}`}
                </button>
              </section>
            )})}
          </div>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Latest local validation activity</p>
          <h3>Signals from real commands</h3>
          <div className="activity-list">
            <ValidationRow label="Cargo" job={latestCargo} />
            <ValidationRow label="smc" job={latestSmc} />
            <ValidationRow label="svm" job={latestSvm} />
            <ValidationRow label="Release bundle verify" job={latestBundle} />
          </div>
        </article>
      </section>

      <section className="command-grid">
        <article className="screen-card">
          <p className="card-kicker">Adapter contract</p>
          <h3>Supported public command surfaces</h3>
          <p className="screen-summary">
            The backend adapter resolves only approved tools and keeps all job cwd values inside the repository root.
          </p>
          <div className="repo-root">
            <span className="repo-root-label">Active workspace root</span>
            <code>{adapterContract?.repoRoot ?? 'Loading adapter contract...'}</code>
          </div>
          <p className="job-meta">
            current workspace:{' '}
            <code>{selectedWorkspace?.resolvedPath ?? adapterContract?.repoRoot ?? 'Loading...'}</code>
          </p>
          {adapterError ? <p className="adapter-error">{adapterError}</p> : null}
          <div className="spec-grid">
            {(adapterContract?.jobs ?? []).map((spec) => {
              const probeAction: JobActionSpec = {
                kind: spec.kind,
                label: `${spec.label} probe`,
                args: spec.exampleArgs,
                notes: spec.notes,
                cwdMode: 'workspace',
              }
              const probeRunning = isActionRunning(probeAction)

              return (
              <section key={spec.kind} className="adapter-spec">
                <div className="adapter-header">
                  <h4>{spec.label}</h4>
                  <span className={`status-pill ${probeRunning ? 'running' : 'draft'}`}>
                    {probeRunning ? 'running' : 'probe'}
                  </span>
                </div>
                <p>{spec.notes}</p>
                <code className="code-block">{spec.resolution}</code>
                <button
                  type="button"
                  className="ghost-button"
                  onClick={() => void onRunProbe(spec)}
                  disabled={probeRunning}
                >
                  {probeRunning ? 'Running probe...' : `Run ${spec.label} probe`}
                </button>
              </section>
            )})}
          </div>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Job history</p>
          <h3>Deterministic execution ledger</h3>
          <div className="job-list">
            {jobs.length === 0 ? (
              <p className="empty-state">
                No jobs yet. Run a workflow or probe to populate the execution ledger.
              </p>
            ) : (
              jobs.map((job) => (
                <button
                  key={job.id}
                  type="button"
                  className={`job-card job-card-button ${selectedJob?.id === job.id ? 'job-card-selected' : ''}`}
                  onClick={() => onSelectJob(job.id)}
                >
                  <div className="job-topline">
                    <div>
                      <strong>{job.label}</strong>
                      <p className="job-meta">{job.commandLine}</p>
                    </div>
                    <span className={`status-pill ${job.status}`}>{job.status}</span>
                  </div>
                  <p className="job-meta">
                    cwd: <code>{job.cwd}</code>
                  </p>
                  <p className="job-meta">
                    exit: {job.exitCode ?? 'pending'} | duration:{' '}
                    {job.durationMs !== undefined ? `${job.durationMs} ms` : 'running'}
                  </p>
                </button>
              ))
            )}
          </div>
        </article>
      </section>

      <section className="command-grid">
        <article className="screen-card">
          <p className="card-kicker">Command output</p>
          <h3>Stdout and stderr without terminal archaeology</h3>
          {selectedJob ? (
            <div className="job-detail-stack">
              <p className="job-meta">
                command: <code>{selectedJob.commandLine}</code>
              </p>
              <p className="job-meta">
                cwd: <code>{selectedJob.cwd}</code>
              </p>
              <p className="job-meta">
                exit: {selectedJob.exitCode ?? 'pending'} | duration:{' '}
                {selectedJob.durationMs !== undefined
                  ? `${selectedJob.durationMs} ms`
                  : 'running'}
              </p>
              <div className="job-output-stack">
                <OutputPane
                  key={`${selectedJob.id}-stdout`}
                  label="stdout"
                  content={selectedJob.stdout}
                  emptyMessage="No stdout captured for this job."
                />
                <OutputPane
                  key={`${selectedJob.id}-stderr`}
                  label="stderr"
                  content={selectedJob.stderr}
                  emptyMessage="No stderr captured for this job."
                  tone="error"
                />
              </div>
            </div>
          ) : (
            <p className="empty-state">
              Select a job from the history to inspect its resolved command and captured output.
            </p>
          )}
        </article>

        <article className="screen-card">
          <p className="card-kicker">Execution rule</p>
          <h3>Jobs remain explainable and reproducible</h3>
          <ul className="bullet-list">
            <li>Every command records its resolved invocation, cwd, exit code, and duration.</li>
            <li>Repository-wide workflows run from the repository root even if the active workspace points deeper.</li>
            <li>Workbench still does not interpret Semantic semantics; it only runs and presents public surfaces.</li>
          </ul>
        </article>
      </section>
    </div>
  )
}

function ValidationRow({
  label,
  job,
}: {
  label: string
  job?: JobRecord
}) {
  return (
    <section className="validation-row">
      <div>
        <strong>{label}</strong>
        <p className="job-meta">{job ? job.commandLine : 'No local run yet'}</p>
      </div>
      <span className={`status-pill ${job?.status ?? 'draft'}`}>
        {job?.status ?? 'not run'}
      </span>
    </section>
  )
}

function countOutputLines(content: string) {
  if (content.length === 0) {
    return 0
  }

  return content.split(/\r?\n/).length
}

function OutputPane({
  label,
  content,
  emptyMessage,
  tone = 'default',
}: {
  label: string
  content: string
  emptyMessage: string
  tone?: 'default' | 'error'
}) {
  const [wrap, setWrap] = useState(true)
  const [copyState, setCopyState] = useState<'idle' | 'copied' | 'failed'>('idle')
  const outputRef = useRef<HTMLPreElement | null>(null)
  const hasContent = content.trim().length > 0
  const lineCount = countOutputLines(content)
  const charCount = content.length

  async function copyOutput() {
    if (!hasContent) {
      return
    }

    try {
      await navigator.clipboard.writeText(content)
      setCopyState('copied')
    } catch {
      setCopyState('failed')
    }
  }

  function scrollToTop() {
    outputRef.current?.scrollTo({ top: 0, behavior: 'smooth' })
  }

  function scrollToBottom() {
    if (!outputRef.current) {
      return
    }

    outputRef.current.scrollTo({
      top: outputRef.current.scrollHeight,
      behavior: 'smooth',
    })
  }

  return (
    <section className="output-pane">
      <div className="output-pane-header">
        <div>
          <p className="card-kicker">{label}</p>
          <p className="job-meta">
            {hasContent
              ? `Full capture preserved · ${lineCount.toLocaleString()} line(s) · ${charCount.toLocaleString()} char(s)`
              : 'Empty capture'}
          </p>
        </div>
        <div className="output-pane-actions">
          <button type="button" className="ghost-button" onClick={() => setWrap((current) => !current)}>
            {wrap ? 'No wrap' : 'Wrap'}
          </button>
          <button type="button" className="ghost-button" onClick={() => void copyOutput()} disabled={!hasContent}>
            {copyState === 'copied' ? 'Copied' : copyState === 'failed' ? 'Copy failed' : 'Copy'}
          </button>
          <button type="button" className="ghost-button" onClick={scrollToTop} disabled={!hasContent}>
            Top
          </button>
          <button type="button" className="ghost-button" onClick={scrollToBottom} disabled={!hasContent}>
            Bottom
          </button>
        </div>
      </div>
      {hasContent ? (
        <pre
          ref={outputRef}
          className={`terminal-output ${tone === 'error' ? 'terminal-output-error' : ''} ${wrap ? '' : 'terminal-output-nowrap'}`}
        >
          {content}
        </pre>
      ) : (
        <p className="empty-state">{emptyMessage}</p>
      )}
    </section>
  )
}

function SpecNavigatorPanel({
  specCatalog,
  specError,
  specSearch,
  selectedSpecDocument,
  selectedSpecPath,
  onSpecSearchChange,
  onSelectSpecPath,
}: {
  specCatalog: SpecCatalogSection[]
  specError: string | null
  specSearch: string
  selectedSpecDocument: SpecDocumentView | null
  selectedSpecPath: string | null
  onSpecSearchChange: (value: string) => void
  onSelectSpecPath: (value: string) => void
}) {
  const query = specSearch.trim().toLowerCase()
  const quickSearchFilters = [
    { label: 'Language', value: 'language' },
    { label: 'Execution', value: 'vm' },
    { label: 'PROMETHEUS', value: 'abi' },
    { label: 'Release', value: 'readiness' },
  ]
  const docsEntryDocuments = [
    'docs/spec/index.md',
    'docs/LANGUAGE.md',
    'docs/spec/vm.md',
    'docs/roadmap/v1_readiness.md',
    'docs/roadmap/stable_release_policy.md',
    'docs/roadmap/release_asset_smoke_matrix.md',
  ]
    .map((relativePath) => findCatalogDocument(specCatalog, relativePath))
    .filter((document): document is NonNullable<typeof document> => Boolean(document))
  const filteredSections = specCatalog
    .map((section) => ({
      ...section,
      documents: section.documents.filter((document) => {
        if (!query) {
          return true
        }

        const haystack = `${document.title} ${document.relativePath}`.toLowerCase()
        return haystack.includes(query)
      }),
    }))
    .filter((section) => section.documents.length > 0)
  const totalDocumentCount = specCatalog.reduce((count, section) => count + section.documents.length, 0)
  const filteredDocumentCount = filteredSections.reduce(
    (count, section) => count + section.documents.length,
    0,
  )

  return (
    <div className="screen-stack">
      <section className="command-grid">
        <article className="screen-card">
          <p className="card-kicker">Docs entry</p>
          <h3>Start from the canonical anchors</h3>
          <p className="screen-summary">
            These entry points are derived from the indexed repository docs, not maintained as a
            separate Workbench knowledge base.
          </p>
          <div className="spec-doc-list">
            {docsEntryDocuments.map((document) => (
              <button
                key={document.relativePath}
                type="button"
                className={`spec-doc-button ${selectedSpecPath === document.relativePath ? 'spec-doc-button-active' : ''}`}
                onClick={() => {
                  onSpecSearchChange('')
                  onSelectSpecPath(document.relativePath)
                }}
              >
                <span className="spec-doc-title">{document.title}</span>
                <span className="spec-doc-path">{document.relativePath}</span>
                {document.status ? (
                  <span className={`status-pill ${statusTone(document.status)}`}>
                    {document.status}
                  </span>
                ) : null}
              </button>
            ))}
          </div>
        </article>
      </section>

      <section className="spec-shell">
        <article className="screen-card spec-sidebar-panel">
          <p className="card-kicker">Canonical document tree</p>
          <h3>Spec and roadmap navigator</h3>
          <label className="field-label" htmlFor="spec-search">
            Search titles and paths
          </label>
          <input
            id="spec-search"
            className="text-field"
            type="text"
            value={specSearch}
            onChange={(event) => onSpecSearchChange(event.target.value)}
            placeholder="Search syntax, vm, readiness, release..."
          />
          <div className="field-row">
            <p className="job-meta">
              search scope: canonical titles and relative paths only. freshness and other derived metadata never affect
              matches.
            </p>
            {specSearch ? (
              <button type="button" className="ghost-button" onClick={() => onSpecSearchChange('')}>
                Clear search
              </button>
            ) : null}
          </div>
          <div className="status-cluster">
            {quickSearchFilters.map((filter) => (
              <button
                key={filter.label}
                type="button"
                className="ghost-button"
                onClick={() => onSpecSearchChange(filter.value)}
              >
                {filter.label}
              </button>
            ))}
          </div>
          <p className="job-meta">
            showing {filteredDocumentCount} of {totalDocumentCount} canonical documents
            {query ? ` for "${specSearch}"` : ''}.
          </p>
          {specError ? <p className="adapter-error">{specError}</p> : null}
          <div className="spec-section-list">
            {filteredSections.length === 0 ? (
              <p className="empty-state">No canonical documents match the current search.</p>
            ) : (
              filteredSections.map((section) => (
                <section key={section.key} className="spec-section-card">
                  <p className="card-kicker">{section.title}</p>
                  <div className="spec-doc-list">
                    {section.documents.map((document) => (
                      <button
                        key={document.relativePath}
                        type="button"
                        className={`spec-doc-button ${selectedSpecPath === document.relativePath ? 'spec-doc-button-active' : ''}`}
                        onClick={() => onSelectSpecPath(document.relativePath)}
                      >
                        <span className="spec-doc-title">{document.title}</span>
                        <span className="spec-doc-path">{document.relativePath}</span>
                        {document.status ? (
                          <span className={`status-pill ${statusTone(document.status)}`}>
                            {document.status}
                          </span>
                        ) : null}
                      </button>
                    ))}
                  </div>
                </section>
              ))
            )}
          </div>
        </article>

        <article className="screen-card spec-document-panel">
          {selectedSpecDocument ? (
            <div className="screen-stack">
              <div className="document-topline">
                <div>
                  <p className="card-kicker">{selectedSpecDocument.sectionTitle}</p>
                  <h3>{selectedSpecDocument.title}</h3>
                </div>
                {selectedSpecDocument.status ? (
                  <span className={`status-pill ${statusTone(selectedSpecDocument.status)}`}>
                    {selectedSpecDocument.status}
                  </span>
                ) : null}
              </div>
              <section className="diagnostic-callout document-truth-callout">
                <div className="document-topline">
                  <span className="diagnostic-meta-label">Repository truth</span>
                  <span
                    className={`status-pill ${selectedSpecDocument.status ? statusTone(selectedSpecDocument.status) : 'draft'}`}
                  >
                    {selectedSpecDocument.status ?? 'status unlabelled'}
                  </span>
                </div>
                <p className="job-meta">
                  source path: <code>{selectedSpecDocument.absolutePath}</code>
                </p>
                <p className="job-meta">
                  Workbench reads this document directly from the repository and does not keep an alternative spec
                  mirror.
                </p>
                <p className="job-meta">
                  derived freshness: <code>{formatFreshness(selectedSpecDocument.modifiedEpochMs)}</code> from filesystem
                  modification time only.
                </p>
              </section>
              <div className="spec-document-grid">
                <aside className="spec-outline-panel">
                  <p className="card-kicker">Section navigator</p>
                  <div className="spec-outline-list">
                    {selectedSpecDocument.headings.map((heading) => (
                      <button
                        key={heading.anchor}
                        type="button"
                        className={`spec-outline-button spec-outline-level-${heading.level}`}
                        onClick={() => jumpToHeading(heading.anchor)}
                      >
                        {heading.title}
                      </button>
                    ))}
                  </div>
                </aside>

                <div className="markdown-sheet">
                  {renderMarkdown(selectedSpecDocument.markdown, selectedSpecDocument.headings)}
                </div>
              </div>
            </div>
          ) : (
            <p className="empty-state">
              Select a canonical document to inspect its headings and body.
            </p>
          )}
        </article>
      </section>
    </div>
  )
}

function ReleasePanel({
  overviewSnapshot,
  specCatalog,
  jobs,
  selectedWorkspace,
  onRunAction,
}: {
  overviewSnapshot: OverviewSnapshot | null
  specCatalog: SpecCatalogSection[]
  jobs: JobRecord[]
  selectedWorkspace: WorkspaceSummary | null
  onRunAction: (action: JobActionSpec) => Promise<JobResult | null>
}) {
  const [validationMessage, setValidationMessage] = useState<string | null>(null)
  const [validationRunning, setValidationRunning] = useState(false)
  const [exportMessage, setExportMessage] = useState<string | null>(null)
  const releaseView = deriveReleaseConsoleView(overviewSnapshot, specCatalog, jobs)
  const releaseManifest = releaseView.releaseManifest
  const assetSmoke = releaseView.assetSmoke
  const releaseDocuments = releaseView.releaseDocuments
  const docsAlignment = releaseView.docsAlignment
  const gateRows = releaseView.gateRows

  async function runCleanValidationPass() {
    setValidationRunning(true)
    setValidationMessage(null)

    try {
      for (const action of releaseValidationPlan) {
        const result = await onRunAction(action)
        if (!result?.success) {
          setValidationMessage(`Validation stopped at '${action.label}'.`)
          return
        }
      }

      setValidationMessage('Clean validation pass completed successfully.')
    } finally {
      setValidationRunning(false)
    }
  }

  async function exportReleaseReport() {
    if (!overviewSnapshot) {
      setExportMessage('Release snapshot is not loaded yet.')
      return
    }

    const markdown = buildReleaseReportMarkdown({
      overviewSnapshot,
      releaseView,
    })

    try {
      const result = await exportReleaseReportFile({
        markdown,
        fileName: 'workbench-release-console-report.md',
      })
      setExportMessage(`Report exported to ${result.repoRelativePath}.`)
    } catch (error) {
      setExportMessage(String(error))
    }
  }

  return (
    <div className="screen-stack">
      <section className="release-console-grid">
        <article className="screen-card">
          <p className="card-kicker">Release identity</p>
          <h3>Current release line anchor</h3>
          <div className="field-actions">
            <button
              type="button"
              className="action-button"
              onClick={() => void runCleanValidationPass()}
              disabled={validationRunning}
            >
              {validationRunning ? 'Running validation...' : 'Run clean validation'}
            </button>
            <button
              type="button"
              className="ghost-button"
              onClick={() => void exportReleaseReport()}
            >
              Export release report
            </button>
          </div>
          {validationMessage ? <p className="job-meta">{validationMessage}</p> : null}
          {exportMessage ? <p className="job-meta">{exportMessage}</p> : null}
          <section className="diagnostic-callout document-truth-callout">
            <span className="diagnostic-meta-label">Release truth contract</span>
            <p className="job-meta">
              Gate badges below come only from local job history. Docs alignment, known limits, and
              asset smoke come only from canonical repository documents and the baseline manifest.
            </p>
          </section>
          <dl className="facts-grid">
            <div>
              <dt>Branch</dt>
              <dd>{overviewSnapshot?.branch ?? 'Loading...'}</dd>
            </div>
            <div>
              <dt>HEAD</dt>
              <dd>
                <code>{overviewSnapshot?.shortCommit ?? 'Loading...'}</code>
              </dd>
            </div>
            <div className="facts-grid-wide">
              <dt>Baseline tag</dt>
              <dd>
                {overviewSnapshot
                  ? overviewSnapshot.baselineTagPointsAtHead
                    ? `${overviewSnapshot.baselineTagName} on HEAD`
                    : `${overviewSnapshot.baselineTagName} exists off HEAD`
                  : 'Loading...'}
              </dd>
            </div>
            <div className="facts-grid-wide">
              <dt>Baseline manifest</dt>
              <dd>
                {overviewSnapshot?.baselineManifestExists ? 'Present' : 'Missing'}
                {overviewSnapshot ? (
                  <>
                    {' '}
                    <code>{overviewSnapshot.baselineManifestPath}</code>
                  </>
                ) : null}
              </dd>
            </div>
            <div className="facts-grid-wide">
              <dt>Active workspace</dt>
              <dd>
                <code>{selectedWorkspace?.resolvedPath ?? overviewSnapshot?.repoRoot ?? 'Loading...'}</code>
              </dd>
            </div>
            {releaseManifest ? (
              <>
                <div>
                  <dt>Manifest generated</dt>
                  <dd>{releaseManifest.generatedAt}</dd>
                </div>
                <div className="facts-grid-wide">
                  <dt>Current scope</dt>
                  <dd>{releaseManifest.currentScope}</dd>
                </div>
              </>
            ) : null}
          </dl>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Release gates</p>
          <h3>Status from actual jobs only</h3>
          <div className="release-gate-list">
            {gateRows.map((gate) => (
              <section key={gate.label} className="release-gate-card">
                <div className="document-topline">
                  <strong>{gate.label}</strong>
                  <span className={`status-pill ${gate.job?.status ?? 'draft'}`}>
                    {gate.job?.status ?? 'not run'}
                  </span>
                </div>
                <p className="job-meta">{gate.detail}</p>
                <p className="job-meta">
                  {gate.job ? gate.job.commandLine : 'No local job recorded for this gate yet.'}
                </p>
                <section className="diagnostic-callout document-truth-callout">
                  <span className="diagnostic-meta-label">Truth source</span>
                  <p className="job-meta">{gate.truthSource}</p>
                  <p className="job-meta">{gate.meaning}</p>
                </section>
              </section>
            ))}
          </div>
        </article>
      </section>

      <section className="release-console-grid">
        <article className="screen-card">
          <p className="card-kicker">Docs alignment</p>
          <h3>Canonical release docs remain the truth</h3>
          <div className="release-checklist">
            {docsAlignment.map((item) => (
              <section key={item.label} className="release-check-card">
                <div className="document-topline">
                  <strong>{item.label}</strong>
                  <span className={`status-pill ${item.ok ? 'stable' : 'draft'}`}>
                    {item.ok ? 'aligned' : 'attention'}
                  </span>
                </div>
                <p className="job-meta">{item.detail}</p>
                <p className="job-meta">
                  truth source: <code>{item.truthSource}</code>
                </p>
              </section>
            ))}
          </div>
          <div className="document-list">
            {releaseDocuments.map((document) => (
              <section key={document.relativePath} className="document-card">
                <div className="document-topline">
                  <strong>{document.title}</strong>
                  <span className={`status-pill ${statusTone(document.status ?? 'draft')}`}>
                    {document.status ?? 'draft'}
                  </span>
                </div>
                <section className="diagnostic-callout document-truth-callout">
                  <span className="diagnostic-meta-label">Repository truth</span>
                  <p className="job-meta">
                    source path: <code>{document.absolutePath}</code>
                  </p>
                  <p className="job-meta">
                    status label: <code>{document.status ?? 'draft'}</code>
                  </p>
                  <p className="job-meta">
                    derived freshness: <code>{formatFreshness(document.modifiedEpochMs)}</code> from filesystem
                    modification time only.
                  </p>
                </section>
              </section>
            ))}
          </div>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Artifacts and asset smoke</p>
          <h3>Release inventory from manifest and smoke matrix</h3>
          <div className="screen-stack">
            {releaseManifest ? (
              <section className="document-card">
                <div className="document-topline">
                  <strong>Baseline artifact inventory</strong>
                  <span className="status-pill stable">manifest</span>
                </div>
                <section className="diagnostic-callout document-truth-callout">
                  <span className="diagnostic-meta-label">Repository truth</span>
                  <p className="job-meta">
                    source path: <code>{overviewSnapshot?.baselineManifestPath ?? 'not resolved'}</code>
                  </p>
                  <p className="job-meta">
                    generated at: <code>{releaseManifest.generatedAt}</code>
                  </p>
                </section>
                <p className="job-meta">documentation bundle</p>
                <ul className="bullet-list">
                  {releaseManifest.documentationBundle.map((path) => (
                    <li key={path}>
                      <code>{path}</code>
                    </li>
                  ))}
                </ul>
                <p className="job-meta">validation tests</p>
                <ul className="bullet-list">
                  {releaseManifest.validationTests.map((command) => (
                    <li key={command}>
                      <code>{command}</code>
                    </li>
                  ))}
                </ul>
                <p className="job-meta">snapshot directories</p>
                <ul className="bullet-list">
                  {releaseManifest.snapshotDirectories.map((path) => (
                    <li key={path}>
                      <code>{path}</code>
                    </li>
                  ))}
                </ul>
              </section>
            ) : null}

            <section className="document-card">
              <div className="document-topline">
                <strong>Asset smoke status</strong>
                <span className={`status-pill ${assetSmoke?.validatedTag ? 'stable' : 'draft'}`}>
                  {assetSmoke?.validatedTag ? 'recorded' : 'missing'}
                </span>
              </div>
              <section className="diagnostic-callout document-truth-callout">
                <span className="diagnostic-meta-label">Repository truth</span>
                <p className="job-meta">
                  source path: <code>{releaseView.assetSmokeDocPath}</code>
                </p>
                <p className="job-meta">
                  badge meaning: recorded means the validated tag is present in the canonical smoke
                  matrix, not that a fresh publish smoke just happened in this session.
                </p>
              </section>
              <p className="job-meta">
                validated tag: <code>{assetSmoke?.validatedTag ?? 'not recorded'}</code>
              </p>
              <ul className="bullet-list">
                {(assetSmoke?.validatedAssets ?? []).map((asset) => (
                  <li key={asset}>
                    <code>{asset}</code>
                  </li>
                ))}
              </ul>
              <div className="smoke-scenario-list">
                {(assetSmoke?.scenarios ?? []).map((scenario) => (
                  <section key={scenario.scenario} className="smoke-scenario-card">
                    <div className="document-topline">
                      <strong>{scenario.scenario}</strong>
                      <span
                        className={`status-pill ${scenario.currentResult.toLowerCase() === 'pass' ? 'stable' : 'draft'}`}
                      >
                        {scenario.currentResult}
                      </span>
                    </div>
                    <p className="job-meta">source: {scenario.source}</p>
                    <p className="job-meta">validation: {scenario.validation}</p>
                    <p className="job-meta">expected: {scenario.expectedSignal}</p>
                  </section>
                ))}
              </div>
            </section>

            <section className="document-card">
              <div className="document-topline">
                <strong>Known limits</strong>
                <span className="status-pill draft">honesty</span>
              </div>
              <section className="diagnostic-callout document-truth-callout">
                <span className="diagnostic-meta-label">Repository truth</span>
                <p className="job-meta">
                  Known limits stay visible even when every local release gate above is green.
                </p>
                <p className="job-meta">
                  source docs:{' '}
                  <code>
                    {releaseView.knownLimitSources.length > 0
                      ? releaseView.knownLimitSources.map((document) => document.path).join(', ')
                      : 'No explicit highlight source recorded in release docs.'}
                  </code>
                </p>
              </section>
              <ul className="bullet-list">
                {(overviewSnapshot?.knownLimits ?? []).map((limit) => (
                  <li key={limit}>{limit}</li>
                ))}
              </ul>
              {(overviewSnapshot?.releaseDocs ?? [])
                .filter((document) => document.highlight)
                .map((document) => (
                  <div key={document.key}>
                    <p className="job-meta">
                      <code>{document.path}</code>
                    </p>
                    <p className="document-highlight">{document.highlight}</p>
                  </div>
                ))}
            </section>
          </div>
        </article>
      </section>

      <section className="release-console-grid">
        <article className="screen-card">
          <p className="card-kicker">Release honesty guard</p>
          <h3>What the console must not do</h3>
          <ul className="bullet-list">
            <li>Do not invent a release score that is not backed by repository docs or job output.</li>
            <li>Do not hide known limits behind green local commands.</li>
            <li>Do not claim stable readiness unless the canonical release documents and bundle checks say so.</li>
          </ul>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Next operate slices</p>
          <h3>What still belongs to later PRs</h3>
          <ul className="bullet-list">
            <li>`WB-17` adds one-click validation runs and report export.</li>
            <li>`WB-18` and `WB-19` stay downstream from the release contract and should not reopen scope.</li>
            <li>This slice remains read-only over current release artifacts, docs, and job history.</li>
          </ul>
        </article>
      </section>
    </div>
  )
}

function jumpToHeading(anchor: string) {
  const element = globalThis.document?.getElementById(anchor)
  element?.scrollIntoView({ behavior: 'smooth', block: 'start' })
}

function statusTone(status: string) {
  const normalized = status.toLowerCase()
  if (
    normalized.includes('stable') ||
    normalized.includes('validated') ||
    normalized.includes('ready')
  ) {
    return 'stable'
  }

  if (normalized.includes('failed')) {
    return 'failed'
  }

  return 'draft'
}

function diagnosticDocMappingLabel(status: 'exact' | 'family' | 'generic') {
  switch (status) {
    case 'exact':
      return 'exact mapping'
    case 'family':
      return 'family mapping'
    default:
      return 'generic only'
  }
}

function formatFreshness(modifiedEpochMs: number | null) {
  if (!modifiedEpochMs) {
    return 'unknown'
  }

  const deltaMs = Date.now() - modifiedEpochMs
  const minute = 60_000
  const hour = 60 * minute
  const day = 24 * hour

  if (deltaMs < hour) {
    return `${Math.max(1, Math.round(deltaMs / minute))} min ago`
  }

  if (deltaMs < day) {
    return `${Math.max(1, Math.round(deltaMs / hour))} hr ago`
  }

  return `${Math.max(1, Math.round(deltaMs / day))} day ago`
}

function createEditorTab(document: WorkspaceFileDocument): EditorTab {
  return {
    relativePath: document.relativePath,
    absolutePath: document.absolutePath,
    title: document.relativePath.split('/').pop() ?? document.relativePath,
    content: document.content,
    savedContent: document.content,
    status: 'clean',
  }
}

function isSemanticSource(relativePath: string) {
  return relativePath.toLowerCase().endsWith('.sm')
}

function toRepoRelativePath(relativePath: string, workspace: WorkspaceSummary) {
  return workspace.repoRelativePath
    ? `${workspace.repoRelativePath}/${relativePath}`.replace(/\\/g, '/')
    : relativePath.replace(/\\/g, '/')
}

function compileOutputPath(repoRelativePath: string) {
  const stem = repoRelativePath.replace(/\\/g, '/').replace(/\//g, '__').replace(/\.sm$/i, '')
  return `target/workbench-${stem}.smc`
}

function severityPillClass(severity: WorkbenchDiagnostic['severity']) {
  switch (severity) {
    case 'warning':
      return 'draft'
    case 'info':
      return 'running'
    default:
      return 'failed'
  }
}

function formatDiagnosticLocation(diagnostic: WorkbenchDiagnostic) {
  const filePart = diagnostic.filePath ?? 'no file path'
  if (diagnostic.line !== null && diagnostic.column !== null) {
    return `${filePart}:${diagnostic.line}:${diagnostic.column}`
  }

  if (diagnostic.offsetHex) {
    return `${filePart} @ ${diagnostic.offsetHex}`
  }

  if (diagnostic.instruction !== null) {
    return `${filePart} @ instruction ${diagnostic.instruction}`
  }

  return filePart
}

const inspectFamilyOrder: InspectFamily[] = ['trace', 'verify', 'disasm', 'verified-run']

function inspectFamilyLabel(family: InspectFamily) {
  switch (family) {
    case 'trace':
      return 'Trace'
    case 'verify':
      return 'Verify'
    case 'disasm':
      return 'Disasm'
    case 'verified-run':
      return 'Verified run'
  }
}

function inspectFamilyDescription(family: InspectFamily) {
  switch (family) {
    case 'trace':
      return 'Cache-trace and inspection metadata from public smc workflows.'
    case 'verify':
      return 'Verification reports over compiled SemCode artifacts.'
    case 'disasm':
      return 'Read-only SemCode inspection from the public disassembly surface.'
    case 'verified-run':
      return 'Execution results for bytecode that passed the verified path.'
  }
}

function deriveInspectableJobs(jobs: JobRecord[]): InspectableJob[] {
  return jobs.flatMap((job) => {
    const family = classifyInspectFamily(job)
    if (!family) {
      return []
    }

    const stdoutText = normalizeInspectOutput(job.stdout)
    const stderrText = normalizeInspectOutput(job.stderr)
    const artifact = extractInspectArtifact(job, family)
    const summary = summarizeInspectJob(job, stdoutText, stderrText)

    return [
      {
        job,
        family,
        artifactPath: artifact.path,
        artifactSource: artifact.source,
        summary: summary.text,
        summarySource: summary.source,
        stdoutText,
        stderrText,
      },
    ]
  })
}

function classifyInspectFamily(job: JobRecord): InspectFamily | null {
  const command = effectiveResolvedCommand(job)

  if (
    job.kind === 'smc' &&
    (command.includes('--trace-cache') || command.some((token) => token.includes("cache_")))
  ) {
    return 'trace'
  }

  if (job.kind === 'smc' && command.includes('verify')) {
    return 'verify'
  }

  if ((job.kind === 'smc' || job.kind === 'svm') && command.includes('disasm')) {
    return 'disasm'
  }

  if (
    (job.kind === 'svm' && command.includes('run')) ||
    (job.kind === 'smc' && command.includes('run-smc'))
  ) {
    return 'verified-run'
  }

  return null
}

function effectiveResolvedCommand(job: JobRecord) {
  if (job.resolvedCommand.length > 0) {
    return job.resolvedCommand
  }

  return job.commandLine.split(/\s+/).filter(Boolean)
}

function extractInspectArtifact(job: JobRecord, family: InspectFamily) {
  const command = effectiveResolvedCommand(job)
  const subcommand =
    family === 'trace'
      ? 'check'
      : family === 'verify'
      ? 'verify'
      : family === 'disasm'
        ? 'disasm'
        : job.kind === 'smc'
          ? 'run-smc'
          : 'run'
  const subcommandIndex = command.findIndex((token) => token === subcommand)

  if (subcommandIndex === -1) {
    return {
      path: null,
      source: 'not-explicit' as const,
    }
  }

  const path = command[subcommandIndex + 1] ?? null

  return {
    path,
    source: path ? ('explicit-command-arg' as const) : ('not-explicit' as const),
  }
}

function normalizeInspectOutput(output: string) {
  const trimmed = output.trim()
  return trimmed.length > 0 ? trimmed : null
}

function summarizeInspectJob(
  job: JobRecord,
  stdoutText: string | null,
  stderrText: string | null,
) {
  const stdoutFirstLine = stdoutText?.split(/\r?\n/, 1)[0] ?? null
  if (stdoutFirstLine) {
    return {
      text: stdoutFirstLine,
      source: 'stdout-first-line' as const,
    }
  }

  const stderrFirstLine = stderrText?.split(/\r?\n/, 1)[0] ?? null
  if (stderrFirstLine) {
    return {
      text: stderrFirstLine,
      source: 'stderr-first-line' as const,
    }
  }

  if (job.status === 'success') {
    return {
      text: 'Completed without captured stdout or stderr.',
      source: 'job-status' as const,
    }
  }

  if (job.status === 'running') {
    return {
      text: 'Command still running.',
      source: 'job-status' as const,
    }
  }

  return {
    text: 'Command failed without captured output.',
    source: 'job-status' as const,
  }
}

function inspectArtifactSourceLabel(source: InspectableJob['artifactSource']) {
  switch (source) {
    case 'explicit-command-arg':
      return 'explicit command argument'
    case 'not-explicit':
      return 'not explicit in command'
  }
}

function inspectSummarySourceLabel(source: InspectableJob['summarySource']) {
  switch (source) {
    case 'stdout-first-line':
      return 'derived from first stdout line'
    case 'stderr-first-line':
      return 'derived from first stderr line'
    case 'job-status':
      return 'derived from job status only'
  }
}

type InspectSignal = {
  label: string
  tone: 'stable' | 'draft' | 'failed'
  detail: string
}

function deriveInspectSignals(entry: InspectableJob): InspectSignal[] {
  if (entry.family === 'trace') {
    const signals = parseTraceSignals(entry.stdoutText)
    if (signals.length > 0) {
      return signals
    }
    return [
      {
        label: 'Trace output preserved',
        tone: entry.job.status === 'success' ? 'stable' : 'draft',
        detail: 'Workbench captured raw trace-cache output without reinterpreting compiler ownership.',
      },
    ]
  }

  if (entry.family === 'verify') {
    const verifiedMatch = entry.stdoutText?.match(
      /verified '(.+)' \((\d+) function\(s\), header=([^,]+), epoch=([^)]+)\)/,
    )
    if (verifiedMatch) {
      return [
        {
          label: 'Verification passed',
          tone: 'stable',
          detail: `${verifiedMatch[2]} function(s), header ${verifiedMatch[3]}, epoch ${verifiedMatch[4]}.`,
        },
      ]
    }

    if (entry.stderrText) {
      return [
        {
          label: 'Verification failed',
          tone: 'failed',
          detail: entry.stderrText.split(/\r?\n/, 1)[0] ?? 'Verifier output captured in stderr.',
        },
      ]
    }
  }

  const runtimeSignals = parseRuntimeSignals(entry)
  if (runtimeSignals.length > 0) {
    return runtimeSignals
  }

  return [
    {
      label: entry.job.status === 'success' ? 'No runtime faults captured' : 'No derived runtime summary',
      tone: entry.job.status === 'success' ? 'stable' : 'draft',
      detail:
        entry.job.status === 'success'
          ? 'The current public output does not report quota, capability, or runtime failures for this job.'
          : 'Captured output is still preserved below even when no derived summary rule matched.',
    },
  ]
}

function parseTraceSignals(stdoutText: string | null): InspectSignal[] {
  if (!stdoutText) {
    return []
  }

  const signals: InspectSignal[] = []
  for (const line of stdoutText.split(/\r?\n/)) {
    const trimmed = line.trim()
    if (!trimmed.startsWith('{')) {
      continue
    }

    try {
      const event = JSON.parse(trimmed) as Record<string, string>
      const label = event.event === 'cache_miss'
        ? 'Cache miss'
        : event.event === 'invalidate'
          ? 'Invalidation'
          : 'Trace event'
      const detailParts = [event.reason, event.module, event.pack_kind].filter(Boolean)
      signals.push({
        label,
        tone: event.event === 'cache_miss' ? 'draft' : 'stable',
        detail: detailParts.join(' | '),
      })
    } catch {
      // Preserve raw output below; ignore non-JSON trace lines here.
    }
  }

  return signals
}

function parseRuntimeSignals(entry: InspectableJob): InspectSignal[] {
  const output = [entry.stdoutText, entry.stderrText].filter(Boolean).join('\n')
  if (!output) {
    return []
  }

  const signals: InspectSignal[] = []
  const quotaMatch = output.match(/quota exceeded: ([^ ]+) limit=(\d+) used=(\d+)/i)
  if (quotaMatch) {
    signals.push({
      label: 'Quota exceeded',
      tone: 'failed',
      detail: `${quotaMatch[1]} limit=${quotaMatch[2]} used=${quotaMatch[3]}`,
    })
  }

  const capabilityLine = output
    .split(/\r?\n/)
    .find((line) => /capability/i.test(line) && /(denied|missing|blocked)/i.test(line))
  if (capabilityLine) {
    signals.push({
      label: 'Capability denied',
      tone: 'failed',
      detail: capabilityLine.trim(),
    })
  }

  const stackLine = output.split(/\r?\n/).find((line) => /stack overflow/i.test(line))
  if (stackLine) {
    signals.push({
      label: 'Stack overflow',
      tone: 'failed',
      detail: stackLine.trim(),
    })
  }

  const verifierLine = output.split(/\r?\n/).find((line) => /verify error|verifier rejected/i.test(line))
  if (verifierLine) {
    signals.push({
      label: 'Verifier rejected input',
      tone: 'failed',
      detail: verifierLine.trim(),
    })
  }

  if (signals.length === 0 && entry.family === 'verified-run' && entry.job.status === 'success') {
    signals.push({
      label: 'Verified execution completed',
      tone: 'stable',
      detail: 'Verified bytecode execution completed without emitted runtime faults.',
    })
  }

  return signals
}

function resolveDiagnosticWorkspacePath(
  filePath: string | null,
  workspace: WorkspaceSummary,
) {
  if (!filePath) {
    return null
  }

  const normalizedFilePath = filePath.replace(/\\/g, '/')
  const normalizedWorkspacePath = workspace.resolvedPath.replace(/\\/g, '/')
  const normalizedRepoRoot = workspace.repoRoot.replace(/\\/g, '/')

  if (/^[A-Za-z]:\//.test(normalizedFilePath) || normalizedFilePath.startsWith('/')) {
    if (!normalizedFilePath.toLowerCase().startsWith(normalizedWorkspacePath.toLowerCase())) {
      return null
    }

    return normalizedFilePath
      .slice(normalizedWorkspacePath.length)
      .replace(/^\/+/, '')
  }

  const repoRelativePath = normalizedFilePath.startsWith(normalizedRepoRoot)
    ? normalizedFilePath.slice(normalizedRepoRoot.length).replace(/^\/+/, '')
    : normalizedFilePath

  if (!workspace.repoRelativePath) {
    return repoRelativePath
  }

  const workspacePrefix = workspace.repoRelativePath.replace(/\\/g, '/')
  if (!repoRelativePath.startsWith(`${workspacePrefix}/`)) {
    return null
  }

  return repoRelativePath.slice(workspacePrefix.length + 1)
}

function renderMarkdown(markdown: string, headings: SpecDocumentHeading[]) {
  const headingAnchors = new Map(headings.map((heading) => [heading.title, heading.anchor]))
  const lines = markdown.split(/\r?\n/)
  const nodes: ReactNode[] = []
  let index = 0

  while (index < lines.length) {
    const line = lines[index]
    const trimmed = line.trim()

    if (!trimmed) {
      index += 1
      continue
    }

    if (trimmed.startsWith('```')) {
      const codeLines: string[] = []
      index += 1
      while (index < lines.length && !lines[index].trim().startsWith('```')) {
        codeLines.push(lines[index])
        index += 1
      }
      index += 1
      nodes.push(
        <pre key={`code-${index}`} className="terminal-output">
          {codeLines.join('\n')}
        </pre>,
      )
      continue
    }

    const headingMatch = /^(#{1,3})\s+(.*)$/.exec(trimmed)
    if (headingMatch) {
      const level = headingMatch[1].length
      const title = headingMatch[2].trim()
      const anchor = headingAnchors.get(title) ?? `heading-${index}`
      if (level === 1) {
        nodes.push(
          <h1 key={anchor} id={anchor} className="markdown-heading markdown-heading-1">
            {title}
          </h1>,
        )
      } else if (level === 2) {
        nodes.push(
          <h2 key={anchor} id={anchor} className="markdown-heading markdown-heading-2">
            {title}
          </h2>,
        )
      } else {
        nodes.push(
          <h3 key={anchor} id={anchor} className="markdown-heading markdown-heading-3">
            {title}
          </h3>,
        )
      }
      index += 1
      continue
    }

    if (trimmed.startsWith('- ')) {
      const bullets: string[] = []
      while (index < lines.length && lines[index].trim().startsWith('- ')) {
        bullets.push(lines[index].trim().slice(2))
        index += 1
      }
      nodes.push(
        <ul key={`list-${index}`} className="bullet-list">
          {bullets.map((bullet) => (
            <li key={`${bullet}-${index}`}>{bullet}</li>
          ))}
        </ul>,
      )
      continue
    }

    const paragraphLines: string[] = []
    while (index < lines.length) {
      const next = lines[index].trim()
      if (!next || next.startsWith('#') || next.startsWith('- ') || next.startsWith('```')) {
        break
      }
      paragraphLines.push(lines[index].trim())
      index += 1
    }

    nodes.push(
      <p key={`p-${index}`} className="markdown-paragraph">
        {paragraphLines.join(' ')}
      </p>,
    )
  }

  return nodes
}

function WorkspaceTreeBranch({
  node,
  activePath,
  onOpenFile,
}: {
  node: WorkspaceTreeNode
  activePath: string | null
  onOpenFile: (relativePath: string) => Promise<void>
}) {
  if (node.nodeType === 'file') {
    return (
      <button
        type="button"
        className={`tree-file-button ${activePath === node.relativePath ? 'tree-file-button-active' : ''}`}
        onClick={() => void onOpenFile(node.relativePath)}
      >
        <span className="tree-node-label">{node.name}</span>
        <span className="tree-node-path">{node.relativePath}</span>
      </button>
    )
  }

  return (
    <section className="tree-branch">
      <p className="tree-branch-label">{node.name}</p>
      <div className="tree-children">
        {node.children.map((child) => (
          <WorkspaceTreeBranch
            key={child.relativePath || child.name}
            node={child}
            activePath={activePath}
            onOpenFile={onOpenFile}
          />
        ))}
      </div>
    </section>
  )
}

function ProjectPanel({
  adapterContract,
  selectedWorkspace,
  workspaceTree,
  workspaceTreeBusy,
  workspaceTreeError,
  editorTabs,
  activeEditorPath,
  editorFocusTarget,
  workspaceInput,
  workspaceError,
  workspaceNotice,
  workspaceBusy,
  workspaceSource,
  recentWorkspaces,
  settings,
  jobs,
  onWorkspaceInputChange,
  onOpenWorkspace,
  onOpenEditorFile,
  onSelectEditorPath,
  onUpdateEditorContent,
  onRunAction,
  isActionRunning,
  onRefreshWorkspace,
  onSaveEditorFile,
  onReloadEditorFile,
  onCloseEditorTab,
  onClearEditorFocusTarget,
  onSelectJob,
}: {
  adapterContract: AdapterContract | null
  selectedWorkspace: WorkspaceSummary | null
  workspaceTree: WorkspaceTreeNode[]
  workspaceTreeBusy: boolean
  workspaceTreeError: string | null
  jobs: JobRecord[]
  editorTabs: EditorTab[]
  activeEditorPath: string | null
  editorFocusTarget: EditorFocusTarget | null
  workspaceInput: string
  workspaceError: string | null
  workspaceNotice: string | null
  workspaceBusy: boolean
  workspaceSource: WorkspaceOpenSource | null
  recentWorkspaces: RecentWorkspace[]
  settings: WorkbenchSettings
  onWorkspaceInputChange: (value: string) => void
  onOpenWorkspace: (
    candidate: string,
    options?: boolean | WorkspaceOpenOptions,
  ) => Promise<void>
  onOpenEditorFile: (relativePath: string, options?: EditorOpenOptions) => Promise<void>
  onSelectEditorPath: (relativePath: string | null) => void
  onUpdateEditorContent: (relativePath: string, content: string) => void
  onRunAction: (action: JobActionSpec) => Promise<JobResult | null>
  isActionRunning: (action: JobActionSpec) => boolean
  onRefreshWorkspace: () => Promise<void>
  onSaveEditorFile: (relativePath: string, options?: SaveEditorOptions) => Promise<boolean>
  onReloadEditorFile: (relativePath: string) => Promise<void>
  onCloseEditorTab: (relativePath: string) => void
  onClearEditorFocusTarget: () => void
  onSelectJob: (jobId: string) => void
}) {
  const navigate = useNavigate()
  const activeEditorTab =
    editorTabs.find((tab) => tab.relativePath === activeEditorPath) ?? editorTabs[0] ?? null
  const activeEditorRepoPath =
    activeEditorTab && selectedWorkspace
      ? toRepoRelativePath(activeEditorTab.relativePath, selectedWorkspace)
      : null
  const currentFileArtifactPath = activeEditorRepoPath
    ? compileOutputPath(activeEditorRepoPath)
    : null
  const canRunSemanticFileAction =
    !!activeEditorTab && !!activeEditorRepoPath && isSemanticSource(activeEditorTab.relativePath)
  const currentFileCheckAction =
    canRunSemanticFileAction && activeEditorTab && activeEditorRepoPath
      ? {
          kind: 'smc' as const,
          label: `Check ${activeEditorTab.title}`,
          args: ['check', activeEditorRepoPath],
          notes: 'Check the active .sm file through the canonical smc check surface.',
          cwdMode: 'repo' as const,
        }
      : null
  const currentFileCompileAction =
    canRunSemanticFileAction && activeEditorTab && activeEditorRepoPath
      ? {
          kind: 'smc' as const,
          label: `Compile ${activeEditorTab.title}`,
          args: ['compile', activeEditorRepoPath, '-o', currentFileArtifactPath ?? compileOutputPath(activeEditorRepoPath)],
          notes: 'Compile the active .sm file through the canonical smc compile surface.',
          cwdMode: 'repo' as const,
        }
      : null
  const currentFileRunAction =
    canRunSemanticFileAction && activeEditorTab && currentFileArtifactPath
      ? {
          kind: 'svm' as const,
          label: `Run ${activeEditorTab.title}`,
          args: ['run', currentFileArtifactPath],
          notes: 'Run the compiled artifact for the active .sm file through the canonical svm run surface.',
          cwdMode: 'repo' as const,
        }
      : null
  const currentFileDisasmAction =
    canRunSemanticFileAction && activeEditorTab && currentFileArtifactPath
      ? {
          kind: 'svm' as const,
          label: `Disasm ${activeEditorTab.title}`,
          args: ['disasm', currentFileArtifactPath],
          notes: 'Disassemble the compiled artifact for the active .sm file through the canonical svm disasm surface.',
          cwdMode: 'repo' as const,
        }
      : null
  const workspaceFormatTarget = selectedWorkspace?.repoRelativePath ?? '.'
  const hasDirtySemanticTabs = editorTabs.some(
    (tab) => isSemanticSource(tab.relativePath) && tab.status !== 'clean',
  )
  const currentFileFormatAction =
    canRunSemanticFileAction && activeEditorTab && activeEditorRepoPath
      ? {
          kind: 'smc' as const,
          label: `Format ${activeEditorTab.title}`,
          args: ['fmt', activeEditorRepoPath],
          notes: 'Format the active Semantic source file through the canonical smc fmt surface.',
          cwdMode: 'repo' as const,
        }
      : null
  const workspaceFormatAction = selectedWorkspace
    ? {
        kind: 'smc' as const,
        label: `Format ${selectedWorkspace.repoRelativePath ?? 'repository root'}`,
        args: ['fmt', workspaceFormatTarget],
        notes: 'Format all Semantic source files under the selected workspace through smc fmt.',
        cwdMode: 'repo' as const,
      }
    : null
  const workspaceFormatCheckAction = selectedWorkspace
    ? {
        kind: 'smc' as const,
        label: `Format check ${selectedWorkspace.repoRelativePath ?? 'repository root'}`,
        args: ['fmt', '--check', workspaceFormatTarget],
        notes: 'Run canonical formatter check for the selected workspace.',
        cwdMode: 'repo' as const,
      }
    : null
  const dirtySemanticTabs = editorTabs.filter(
    (tab) => isSemanticSource(tab.relativePath) && tab.status !== 'clean',
  )
  const dirtySemanticTabTitles = dirtySemanticTabs.map((tab) => tab.title)
  const currentFileFormatNeedsSave = Boolean(
    activeEditorTab &&
      canRunSemanticFileAction &&
      activeEditorTab.status !== 'clean',
  )
  const latestWorkspaceFormatCheckJob = workspaceFormatCheckAction
    ? latestJobMatching(jobs, (job) => jobMatchesAction(job, workspaceFormatCheckAction))
    : null
  const [scaffoldPackageName, setScaffoldPackageName] = useState(
    deriveScaffoldPackageName(selectedWorkspace),
  )
  const [scaffoldBusy, setScaffoldBusy] = useState(false)
  const [scaffoldMessage, setScaffoldMessage] = useState<string | null>(null)
  const [packageManifestPreview, setPackageManifestPreview] = useState<PackageManifestPreview | null>(
    null,
  )
  const [packageManifestState, setPackageManifestState] = useState<
    'idle' | 'loading' | 'ready' | 'missing' | 'error'
  >('idle')
  const [packageManifestError, setPackageManifestError] = useState<string | null>(null)
  const [editorCursor, setEditorCursor] = useState<EditorCursorPosition>({ line: 0, character: 0 })
  const [smlspBusy, setSmlspBusy] = useState(false)
  const [smlspSession, setSmlspSession] = useState<SmlspSessionResult | null>(null)
  const [smlspError, setSmlspError] = useState<string | null>(null)
  const packageManifestRequestIdRef = useRef(0)
  const editorTextareaRef = useRef<HTMLTextAreaElement | null>(null)
  const activeSmlspResult =
    activeEditorTab && smlspSession?.relativePath === activeEditorTab.relativePath
      ? smlspSession.result
      : null
  const staleSmlspSession =
    smlspSession && activeEditorTab && smlspSession.relativePath !== activeEditorTab.relativePath
      ? smlspSession
      : null
  const smlspDefinitionRelativePath =
    activeSmlspResult?.definitionPath && selectedWorkspace
      ? resolveAbsoluteWorkspacePath(
          activeSmlspResult.definitionPath,
          selectedWorkspace,
        )
      : null

  useEffect(() => {
    setScaffoldPackageName(deriveScaffoldPackageName(selectedWorkspace))
    setScaffoldMessage(null)
  }, [selectedWorkspace])

  useEffect(() => {
    setEditorCursor({ line: 0, character: 0 })
    setSmlspError(null)
  }, [activeEditorPath])

  useEffect(() => {
    if (!settings.showExperimental) {
      setSmlspSession(null)
      setSmlspError(null)
    }
  }, [settings.showExperimental])

  useEffect(() => {
    if (
      !editorFocusTarget ||
      !activeEditorTab ||
      editorFocusTarget.relativePath !== activeEditorTab.relativePath ||
      !editorTextareaRef.current
    ) {
      return
    }

    const offset = resolveEditorOffsetFromOneBasedLocation(
      activeEditorTab.content,
      editorFocusTarget.line,
      editorFocusTarget.column,
    )

    editorTextareaRef.current.focus()
    editorTextareaRef.current.setSelectionRange(offset, offset)
    setEditorCursor(deriveCursorPosition(activeEditorTab.content, offset))
    onClearEditorFocusTarget()
  }, [activeEditorTab, editorFocusTarget, onClearEditorFocusTarget])

  async function loadPackageManifestPreview(workspace: WorkspaceSummary | null) {
    const requestId = packageManifestRequestIdRef.current + 1
    packageManifestRequestIdRef.current = requestId

    if (!workspace) {
      setPackageManifestState('idle')
      setPackageManifestPreview(null)
      setPackageManifestError(null)
      return
    }

    setPackageManifestState('loading')
    setPackageManifestError(null)

    try {
      const document = await fetchWorkspaceFile({
        workspaceRoot: workspace.resolvedPath,
        relativePath: 'Semantic.toml',
      })

      if (packageManifestRequestIdRef.current !== requestId) {
        return
      }

      setPackageManifestPreview(parsePackageManifest(document.content))
      setPackageManifestState('ready')
    } catch (error) {
      if (packageManifestRequestIdRef.current !== requestId) {
        return
      }

      const message = String(error)
      setPackageManifestPreview(null)
      if (message.toLowerCase().includes('semantic.toml')) {
        setPackageManifestState('missing')
        return
      }

      setPackageManifestState('error')
      setPackageManifestError(message)
    }
  }

  useEffect(() => {
    void loadPackageManifestPreview(selectedWorkspace)
  }, [selectedWorkspace])

  async function runCurrentFileAction(mode: 'check' | 'compile' | 'run' | 'disasm') {
    if (!activeEditorTab || !selectedWorkspace || !adapterContract || !activeEditorRepoPath) {
      return
    }

    if (activeEditorTab.status !== 'clean') {
      const saved = await onSaveEditorFile(activeEditorTab.relativePath)
      if (!saved) {
        return
      }
    }

    if (mode === 'check') {
      await onRunAction(
        currentFileCheckAction ?? {
          kind: 'smc',
          label: `Check ${activeEditorTab.title}`,
          args: ['check', activeEditorRepoPath],
          notes: 'Check the active .sm file through the canonical smc check surface.',
          cwdMode: 'repo',
        },
      )
      navigate('/diagnostics')
      return
    }

    const compileResult = await onRunAction(
      currentFileCompileAction ?? {
        kind: 'smc',
        label: `Compile ${activeEditorTab.title}`,
        args: ['compile', activeEditorRepoPath, '-o', currentFileArtifactPath ?? compileOutputPath(activeEditorRepoPath)],
        notes: 'Compile the active .sm file through the canonical smc compile surface.',
        cwdMode: 'repo',
      },
    )

    if (compileResult === null) {
      return
    }

    if (!compileResult.success) {
      navigate('/diagnostics')
      return
    }

    if (mode === 'compile') {
      return
    }

    const downstreamAction =
      mode === 'run'
        ? currentFileRunAction ?? {
            kind: 'svm',
            label: `Run ${activeEditorTab.title}`,
            args: ['run', currentFileArtifactPath ?? compileOutputPath(activeEditorRepoPath)],
            notes: 'Run the compiled artifact for the active .sm file through the canonical svm run surface.',
            cwdMode: 'repo',
          }
        : currentFileDisasmAction ?? {
            kind: 'svm',
            label: `Disasm ${activeEditorTab.title}`,
            args: ['disasm', currentFileArtifactPath ?? compileOutputPath(activeEditorRepoPath)],
            notes: 'Disassemble the compiled artifact for the active .sm file through the canonical svm disasm surface.',
            cwdMode: 'repo',
          }

    await onRunAction(downstreamAction)
    navigate('/inspect')
  }

  async function runFormatterAction(mode: 'file' | 'workspace' | 'check') {
    if (!selectedWorkspace || !adapterContract) {
      return
    }

      if (mode === 'file') {
        if (!activeEditorTab || !activeEditorRepoPath || !canRunSemanticFileAction) {
          return
        }

        if (activeEditorTab.status !== 'clean') {
          const saved = await onSaveEditorFile(activeEditorTab.relativePath, {
            applyFormatOnSave: false,
          })
          if (!saved) {
            return
          }
        }

      const result = await onRunAction(
        currentFileFormatAction ?? {
          kind: 'smc',
          label: `Format ${activeEditorTab.title}`,
          args: ['fmt', activeEditorRepoPath],
          notes: 'Format the active Semantic source file through the canonical smc fmt surface.',
          cwdMode: 'repo',
        },
      )

      if (result?.success) {
        await onReloadEditorFile(activeEditorTab.relativePath)
      }
      return
    }

    if (hasDirtySemanticTabs) {
      return
    }

    const result = await onRunAction(
      mode === 'check'
        ? workspaceFormatCheckAction ?? {
            kind: 'smc',
            label: `Format check ${selectedWorkspace.repoRelativePath ?? 'repository root'}`,
            args: ['fmt', '--check', workspaceFormatTarget],
            notes: 'Run canonical formatter check for the selected workspace.',
            cwdMode: 'repo',
          }
        : workspaceFormatAction ?? {
            kind: 'smc',
            label: `Format ${selectedWorkspace.repoRelativePath ?? 'repository root'}`,
            args: ['fmt', workspaceFormatTarget],
            notes: 'Format all Semantic source files under the selected workspace through smc fmt.',
            cwdMode: 'repo',
          },
    )

    if (mode === 'check' && result) {
      navigate('/overview')
      return
    }

    if (mode === 'workspace' && result?.success) {
      for (const tab of editorTabs) {
        if (tab.status === 'clean' && isSemanticSource(tab.relativePath)) {
          await onReloadEditorFile(tab.relativePath)
        }
      }
    }
  }

  async function runScaffold(mode: 'new' | 'init') {
    if (!selectedWorkspace || scaffoldBusy) {
      return
    }

    setScaffoldBusy(true)
    setScaffoldMessage(null)

    try {
      const result = await scaffoldSemanticProject({
        workspaceRoot: selectedWorkspace.resolvedPath,
        mode,
        packageName: scaffoldPackageName,
      })

      if (mode === 'new') {
        await onOpenWorkspace(result.workspaceRoot, {
          persist: true,
          source: 'preset',
          successMessage: `Created child project ${result.repoRelativePath} and switched Workbench to it.`,
        })
      } else {
        await Promise.all([
          onRefreshWorkspace(),
          loadPackageManifestPreview(selectedWorkspace),
        ])
        await onOpenEditorFile(result.entryRelativePath)
      }

      setScaffoldPackageName(deriveScaffoldPackageNameFromResult(result))
      setScaffoldMessage(
        mode === 'init'
          ? `Created ${result.createdPaths.length} canonical files under ${result.repoRelativePath}, refreshed the tree and manifest preview, and opened ${result.entryRelativePath}.`
          : `Created ${result.createdPaths.length} canonical files under ${result.repoRelativePath}: ${result.createdPaths.join(', ')}`,
      )
    } catch (error) {
      setScaffoldMessage(String(error))
    } finally {
      setScaffoldBusy(false)
    }
  }

  async function runSmlspBridge() {
    if (
      !settings.showExperimental ||
      !selectedWorkspace ||
      !activeEditorTab ||
      !isSemanticSource(activeEditorTab.relativePath) ||
      smlspBusy
    ) {
      return
    }

    const sessionRelativePath = activeEditorTab.relativePath
    const sessionContent = activeEditorTab.content
    const sessionCommand = settings.smlspCommand

    setSmlspBusy(true)
    setSmlspError(null)

    try {
      const result = await runSmlspProtocolBridge({
        workspaceRoot: selectedWorkspace.resolvedPath,
        relativePath: sessionRelativePath,
        content: sessionContent,
        line: editorCursor.line,
        character: editorCursor.character,
        command: sessionCommand,
        args: [],
      })
      setSmlspSession({
        relativePath: sessionRelativePath,
        command: sessionCommand,
        result,
      })
    } catch (error) {
      setSmlspSession(null)
      setSmlspError(normalizeSmlspError(String(error), sessionCommand))
    } finally {
      setSmlspBusy(false)
    }
  }

  function applySmlspFormatting() {
    if (
      !activeEditorTab ||
      !activeSmlspResult?.formattingText ||
      smlspSession?.relativePath !== activeEditorTab.relativePath
    ) {
      return
    }

    onUpdateEditorContent(activeEditorTab.relativePath, activeSmlspResult.formattingText)
  }

  return (
    <div className="screen-stack">
      <section className="command-grid">
        <article className="screen-card">
          <p className="card-kicker">Open workspace</p>
          <h3>Canonical root selection for every job</h3>
          <p className="screen-summary">
            Enter an absolute path or repository-relative path. The backend resolver
            canonicalizes it and refuses anything outside the repository boundary.
          </p>
          <label className="field-label" htmlFor="workspace-path">
            Workspace path
          </label>
          <div className="field-row">
            <input
              id="workspace-path"
              className="text-field"
              type="text"
              value={workspaceInput}
              onChange={(event) => onWorkspaceInputChange(event.target.value)}
              placeholder={adapterContract?.repoRoot ?? 'Loading repository root...'}
              disabled={workspaceBusy}
            />
            <button
              type="button"
              className="action-button"
              onClick={() =>
                void onOpenWorkspace(
                  workspaceInput.trim() || adapterContract?.repoRoot || '',
                  { source: 'manual' },
                )
              }
              disabled={!adapterContract || workspaceBusy}
            >
              {workspaceBusy ? 'Opening...' : 'Open'}
            </button>
          </div>
          <div className="field-actions">
            <button
              type="button"
              className="ghost-button"
              onClick={() =>
                void onOpenWorkspace(adapterContract?.repoRoot ?? '', {
                  source: 'preset',
                  successMessage: 'Switched back to the repository root workspace.',
                })
              }
              disabled={!adapterContract || workspaceBusy}
            >
              Use repository root
            </button>
            <button
              type="button"
              className="ghost-button"
              onClick={() =>
                void onOpenWorkspace('examples', {
                  source: 'preset',
                  successMessage: 'Opened the canonical examples workspace.',
                })
              }
              disabled={!adapterContract || workspaceBusy}
            >
              Use `examples`
            </button>
            <button
              type="button"
              className="ghost-button"
              onClick={() =>
                void onOpenWorkspace('docs', {
                  source: 'preset',
                  successMessage: 'Opened the canonical docs workspace.',
                })
              }
              disabled={!adapterContract || workspaceBusy}
            >
              Use `docs`
            </button>
          </div>
          {workspaceNotice ? (
            <p className={`workspace-banner ${workspaceSource === 'fallback' ? 'draft' : 'stable'}`}>
              {workspaceNotice}
            </p>
          ) : null}
          {workspaceError ? <p className="adapter-error">{workspaceError}</p> : null}
          <div className="repo-root">
            <span className="repo-root-label">Selected workspace</span>
            <code>{selectedWorkspace?.resolvedPath ?? 'No workspace selected yet.'}</code>
          </div>
          <p className="job-meta">
            repo-relative:{' '}
            <code>{selectedWorkspace?.repoRelativePath ?? '(repository root)'}</code>
          </p>
          <p className="job-meta">
            source:{' '}
            <span className={`status-pill ${workspaceSourceTone(workspaceSource)}`}>
              {workspaceSourceLabel(workspaceSource)}
            </span>
          </p>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Recent projects</p>
          <h3>Persisted local workspace history</h3>
          <div className="job-list">
            {recentWorkspaces.length === 0 ? (
              <p className="empty-state">
                No recent workspaces yet. Opening a canonical root stores it locally for future sessions.
              </p>
            ) : (
              recentWorkspaces.map((workspace) => (
                <section key={workspace.path} className="job-card">
                  <div className="job-topline">
                    <div>
                      <strong>{workspace.repoRelativePath ?? 'repository root'}</strong>
                      <p className="job-meta">{workspace.path}</p>
                    </div>
                    <button
                      type="button"
                      className="ghost-button"
                      onClick={() =>
                        void onOpenWorkspace(workspace.path, {
                          source: 'recent',
                          successMessage: `Reopened ${workspace.repoRelativePath ?? 'the repository root'} from recent workspaces.`,
                        })
                      }
                      disabled={workspaceBusy}
                    >
                      Reopen
                    </button>
                  </div>
                  <p className="job-meta">last opened: {workspace.openedAtIso}</p>
                </section>
              ))
            )}
          </div>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Project bootstrap</p>
          <h3>Canonical Semantic project scaffold</h3>
          <p className="screen-summary">
            Workbench only creates the canonical layout: <code>Semantic.toml</code>,
            <code> src/main.sm</code>, and <code>examples/smoke.sm</code>. No extra
            package magic or hidden layout rules.
          </p>
          <label className="field-label" htmlFor="scaffold-package-name">
            Package name
          </label>
          <div className="field-row">
            <input
              id="scaffold-package-name"
              className="text-field"
              type="text"
              value={scaffoldPackageName}
              onChange={(event) => setScaffoldPackageName(event.target.value)}
              placeholder="semantic-project"
              disabled={!selectedWorkspace || scaffoldBusy}
            />
          </div>
          <div className="field-actions">
            <button
              type="button"
              className="action-button"
              onClick={() => void runScaffold('init')}
              disabled={!selectedWorkspace || scaffoldBusy}
            >
              {scaffoldBusy ? 'Scaffolding...' : 'Init current workspace'}
            </button>
            <button
              type="button"
              className="ghost-button"
              onClick={() => void runScaffold('new')}
              disabled={!selectedWorkspace || scaffoldBusy}
            >
              Create child project
            </button>
          </div>
          <p className="job-meta">
            target root:{' '}
            <code>{selectedWorkspace?.resolvedPath ?? 'Select a workspace first.'}</code>
          </p>
          <p className="job-meta">
            package mode:{' '}
            <span className="status-pill stable">init</span>{' '}
            <span className="status-pill draft">new child</span>
          </p>
          {scaffoldMessage ? (
            <p className={scaffoldMessage.startsWith('Created ') ? 'job-meta' : 'adapter-error'}>
              {scaffoldMessage}
            </p>
          ) : null}
        </article>

        <article className="screen-card">
          <p className="card-kicker">Package metadata</p>
          <h3>Derived preview from Semantic.toml</h3>
          <p className="screen-summary">
            This preview reads the selected workspace manifest as-is and surfaces first-wave package
            metadata without creating a second package model in the UI.
          </p>
          <p className="job-meta">
            manifest path:{' '}
            <code>
              {selectedWorkspace
                ? `${selectedWorkspace.resolvedPath.replace(/\\/g, '/')}/Semantic.toml`
                : 'Select a workspace first.'}
            </code>
          </p>
          {packageManifestState === 'loading' ? (
            <p className="empty-state">Loading manifest preview...</p>
          ) : null}
          {packageManifestState === 'missing' ? (
            <p className="empty-state">
              No <code>Semantic.toml</code> found in this workspace yet. Use project bootstrap to
              initialize one.
            </p>
          ) : null}
          {packageManifestState === 'error' ? (
            <p className="adapter-error">{packageManifestError}</p>
          ) : null}
          {packageManifestPreview ? (
            <dl className="facts-grid">
              <div>
                <dt>Name</dt>
                <dd>{packageManifestPreview.name ?? 'not declared'}</dd>
              </div>
              <div>
                <dt>Version</dt>
                <dd>{packageManifestPreview.version ?? 'not declared'}</dd>
              </div>
              <div>
                <dt>Edition</dt>
                <dd>{packageManifestPreview.edition ?? 'not declared'}</dd>
              </div>
              <div className="facts-grid-wide">
                <dt>Entry</dt>
                <dd>
                  <code>{packageManifestPreview.entry ?? 'not declared'}</code>
                </dd>
              </div>
            </dl>
          ) : null}
        </article>

        {settings.showExperimental ? (
          <article className="screen-card">
            <p className="card-kicker">Experimental protocol bridge</p>
            <h3>`smlsp` over stdio for the active editor buffer</h3>
            <p className="screen-summary">
              This bridge talks only to an external <code>{settings.smlspCommand}</code> process.
              Workbench does not synthesize hover, definition, diagnostics, or formatting on its
              own.
            </p>
            <section className="diagnostic-callout document-truth-callout">
              <span className="diagnostic-meta-label">Experimental guardrails</span>
              <p className="job-meta">
                This panel stays hidden until experimental workflows are enabled in Settings.
                Failures stay local to this panel and do not alter editor, compiler, verifier, or
                runtime ownership.
              </p>
              <p className="job-meta">
                Formatted text can only be applied back to the same buffer that produced the
                current `smlsp` result.
              </p>
            </section>
            <div className="field-actions">
              <button
                type="button"
                className="action-button"
                onClick={() => void runSmlspBridge()}
                disabled={
                  !selectedWorkspace ||
                  !activeEditorTab ||
                  !isSemanticSource(activeEditorTab.relativePath) ||
                  smlspBusy
                }
              >
                {smlspBusy ? 'Running smlsp...' : 'Run smlsp bridge'}
              </button>
              <button
                type="button"
                className="ghost-button"
                onClick={() => applySmlspFormatting()}
                disabled={
                  !activeEditorTab ||
                  !activeSmlspResult?.formattingText ||
                  smlspSession?.relativePath !== activeEditorTab.relativePath
                }
              >
                Apply formatted text
              </button>
            </div>
            <p className="job-meta">
              cursor: <code>{editorCursor.line}:{editorCursor.character}</code>
            </p>
            <p className="job-meta">
              active file:{' '}
              <code>{activeEditorTab?.relativePath ?? 'open a .sm file first'}</code>
            </p>
            <p className="job-meta">
              command: <code>{settings.smlspCommand}</code>
            </p>
            {smlspError ? <p className="adapter-error">{smlspError}</p> : null}
            {staleSmlspSession ? (
              <div className="diagnostic-callout document-truth-callout">
                <span className="diagnostic-meta-label">Buffered experimental result</span>
                <p className="job-meta">
                  The last successful `smlsp` session belongs to{' '}
                  <code>{staleSmlspSession.relativePath}</code>. Open that buffer again to inspect
                  its hover, definition, diagnostics, or formatting result.
                </p>
              </div>
            ) : null}
            {activeSmlspResult ? (
              <div className="screen-stack">
                <div className="diagnostics-filter-row">
                  <span className="status-pill draft">experimental</span>
                  <span className="status-pill stable">{activeSmlspResult.transport}</span>
                  {activeSmlspResult.capabilities.map((capability) => (
                    <span key={capability} className="status-pill stable">
                      {capability}
                    </span>
                  ))}
                </div>
                <p className="job-meta">
                  session source: <code>{smlspSession?.relativePath}</code> via{' '}
                  <code>{smlspSession?.command}</code>
                </p>
                {activeSmlspResult.hoverMarkdown ? (
                  <section className="inspect-output-block">
                    <span className="diagnostic-meta-label">Hover</span>
                    <pre className="inspect-output-code">{activeSmlspResult.hoverMarkdown}</pre>
                  </section>
                ) : null}
                {activeSmlspResult.definitionPath ? (
                  <section className="inspect-signal-card">
                    <div className="diagnostic-card-topline">
                      <strong>Definition</strong>
                      <span className="status-pill stable">linked</span>
                    </div>
                    <p className="job-meta">
                      <code>{activeSmlspResult.definitionPath}</code>
                    </p>
                    <p className="job-meta">
                      line {activeSmlspResult.definitionLine ?? 0}, character{' '}
                      {activeSmlspResult.definitionCharacter ?? 0}
                    </p>
                    {smlspDefinitionRelativePath ? (
                      <button
                        type="button"
                        className="ghost-button"
                        onClick={() => void onOpenEditorFile(smlspDefinitionRelativePath)}
                      >
                        Open definition file
                      </button>
                    ) : null}
                  </section>
                ) : null}
                <section className="inspect-output-block">
                  <span className="diagnostic-meta-label">
                    Inline diagnostics ({activeSmlspResult.diagnostics.length})
                  </span>
                  {activeSmlspResult.diagnostics.length > 0 ? (
                    <div className="diagnostics-group-list">
                      {activeSmlspResult.diagnostics.map((diagnostic, index) => (
                        <section key={`${diagnostic.message}-${index}`} className="diagnostic-card">
                          <div className="diagnostic-card-topline">
                            <span className={`status-pill ${severityPillClass(diagnostic.severity as WorkbenchDiagnostic['severity'])}`}>
                              {diagnostic.severity}
                            </span>
                            {diagnostic.code ? (
                              <code>{diagnostic.code}</code>
                            ) : null}
                          </div>
                          <strong>{diagnostic.message}</strong>
                          <p className="job-meta">
                            {diagnostic.line}:{diagnostic.character} → {diagnostic.endLine}:
                            {diagnostic.endCharacter}
                          </p>
                        </section>
                      ))}
                    </div>
                  ) : (
                    <p className="empty-state">
                      No diagnostics were published by the current `smlsp` session.
                    </p>
                  )}
                </section>
                {activeSmlspResult.stderr.trim().length > 0 ? (
                  <section className="inspect-output-block">
                    <span className="diagnostic-meta-label">smlsp stderr</span>
                    <pre className="inspect-output-code">{activeSmlspResult.stderr}</pre>
                  </section>
                ) : null}
              </div>
            ) : (
              <p className="empty-state">
                Run the bridge to inspect hover, definition, formatting, and published diagnostics
                for the current buffer.
              </p>
            )}
          </article>
        ) : null}
      </section>

      <section className="project-shell">
        <article className="screen-card project-tree-panel">
          <p className="card-kicker">Project explorer</p>
          <h3>Workspace file tree</h3>
          <p className="screen-summary">
            The explorer stays inside the selected workspace root and only exposes text files for the editor shell.
          </p>
          <div className="field-actions">
            <button
              type="button"
              className="ghost-button"
              onClick={() => void onRefreshWorkspace()}
              disabled={!selectedWorkspace || workspaceTreeBusy}
            >
              {workspaceTreeBusy ? 'Refreshing tree...' : 'Refresh tree'}
            </button>
          </div>
          {workspaceTreeError ? <p className="adapter-error">{workspaceTreeError}</p> : null}
          <div className="project-tree">
            {workspaceTreeBusy && workspaceTree.length === 0 ? (
              <p className="empty-state">
                Refreshing the workspace tree against the current canonical root...
              </p>
            ) : workspaceTree.length === 0 ? (
              <p className="empty-state">
                No editable text files are visible under the selected workspace yet.
              </p>
            ) : (
              workspaceTree.map((node) => (
                <WorkspaceTreeBranch
                  key={node.relativePath || node.name}
                  node={node}
                  activePath={activeEditorPath}
                  onOpenFile={onOpenEditorFile}
                />
              ))
            )}
          </div>
        </article>

        <article className="screen-card editor-shell-panel">
          <p className="card-kicker">Editor shell</p>
          <h3>Tabs, dirty state, and safe file actions</h3>
          <div className="editor-tab-strip">
            {editorTabs.length === 0 ? (
              <p className="empty-state">
                Open a file from the project explorer to start the authoring loop.
              </p>
            ) : (
              editorTabs.map((tab) => (
                <div
                  key={tab.relativePath}
                  className={`editor-tab ${activeEditorTab?.relativePath === tab.relativePath ? 'editor-tab-active' : ''}`}
                >
                  <button
                    type="button"
                    className="editor-tab-select"
                    onClick={() => onSelectEditorPath(tab.relativePath)}
                  >
                    <span>{tab.title}</span>
                    {tab.status !== 'clean' ? (
                      <span className={`status-pill ${tab.status === 'saving' ? 'running' : 'draft'}`}>
                        {tab.status}
                      </span>
                    ) : null}
                  </button>
                  <button
                    type="button"
                    className="editor-tab-close"
                    onClick={() => onCloseEditorTab(tab.relativePath)}
                    aria-label={`Close ${tab.title}`}
                  >
                    x
                  </button>
                </div>
              ))
            )}
          </div>

          {activeEditorTab ? (
            <div className="editor-shell-stack">
              <div className="field-actions">
                <button
                  type="button"
                  className="action-button"
                  onClick={() => void onSaveEditorFile(activeEditorTab.relativePath)}
                  disabled={activeEditorTab.status === 'saving'}
                >
                  {activeEditorTab.status === 'saving' ? 'Saving...' : 'Save file'}
                </button>
                <button
                  type="button"
                  className="ghost-button"
                  onClick={() => void onReloadEditorFile(activeEditorTab.relativePath)}
                >
                  Reload from disk
                </button>
                <button
                  type="button"
                  className="ghost-button"
                  onClick={() => void runFormatterAction('file')}
                  disabled={
                    !canRunSemanticFileAction ||
                    activeEditorTab.status === 'saving' ||
                    !currentFileFormatAction ||
                    isActionRunning(currentFileFormatAction)
                  }
                >
                  {currentFileFormatAction && isActionRunning(currentFileFormatAction)
                    ? 'Formatting file...'
                    : currentFileFormatNeedsSave
                      ? 'Save + format file'
                      : 'Format file'}
                </button>
                <button
                  type="button"
                  className="ghost-button"
                  onClick={() => void runFormatterAction('workspace')}
                  disabled={
                    !selectedWorkspace ||
                    hasDirtySemanticTabs ||
                    !workspaceFormatAction ||
                    isActionRunning(workspaceFormatAction)
                  }
                >
                  {workspaceFormatAction && isActionRunning(workspaceFormatAction)
                    ? 'Formatting workspace...'
                    : 'Format workspace'}
                </button>
                <button
                  type="button"
                  className="ghost-button"
                  onClick={() => void runFormatterAction('check')}
                  disabled={
                    !selectedWorkspace ||
                    hasDirtySemanticTabs ||
                    !workspaceFormatCheckAction ||
                    isActionRunning(workspaceFormatCheckAction)
                  }
                >
                  {workspaceFormatCheckAction && isActionRunning(workspaceFormatCheckAction)
                    ? 'Running format check...'
                    : 'Format check'}
                </button>
                <button
                  type="button"
                  className="ghost-button"
                  onClick={() => void runCurrentFileAction('check')}
                  disabled={
                    !canRunSemanticFileAction ||
                    activeEditorTab.status === 'saving' ||
                    !currentFileCheckAction ||
                    isActionRunning(currentFileCheckAction)
                  }
                >
                  {currentFileCheckAction && isActionRunning(currentFileCheckAction)
                    ? 'Checking current file...'
                    : 'Check current file'}
                </button>
                <button
                  type="button"
                  className="ghost-button"
                  onClick={() => void runCurrentFileAction('compile')}
                  disabled={
                    !canRunSemanticFileAction ||
                    activeEditorTab.status === 'saving' ||
                    !currentFileCompileAction ||
                    isActionRunning(currentFileCompileAction)
                  }
                >
                  {currentFileCompileAction && isActionRunning(currentFileCompileAction)
                    ? 'Compiling current file...'
                    : 'Compile current file'}
                </button>
                <button
                  type="button"
                  className="ghost-button"
                  onClick={() => void runCurrentFileAction('run')}
                  disabled={
                    !canRunSemanticFileAction ||
                    activeEditorTab.status === 'saving' ||
                    !currentFileRunAction ||
                    !currentFileCompileAction ||
                    isActionRunning(currentFileCompileAction) ||
                    isActionRunning(currentFileRunAction)
                  }
                >
                  {currentFileRunAction &&
                  (isActionRunning(currentFileCompileAction ?? currentFileRunAction) ||
                    isActionRunning(currentFileRunAction))
                    ? 'Running current build...'
                    : 'Run current build'}
                </button>
                <button
                  type="button"
                  className="ghost-button"
                  onClick={() => void runCurrentFileAction('disasm')}
                  disabled={
                    !canRunSemanticFileAction ||
                    activeEditorTab.status === 'saving' ||
                    !currentFileDisasmAction ||
                    !currentFileCompileAction ||
                    isActionRunning(currentFileCompileAction) ||
                    isActionRunning(currentFileDisasmAction)
                  }
                >
                  {currentFileDisasmAction &&
                  (isActionRunning(currentFileCompileAction ?? currentFileDisasmAction) ||
                    isActionRunning(currentFileDisasmAction))
                    ? 'Disassembling current build...'
                    : 'Disasm current build'}
                </button>
              </div>
              <p className="job-meta">
                path: <code>{activeEditorTab.absolutePath}</code>
              </p>
              <p className="job-meta">
                repo path:{' '}
                <code>{activeEditorRepoPath ?? 'not a repository-scoped semantic source'}</code>
              </p>
              <p className="job-meta">
                formatter surface:{' '}
                <span className="status-pill draft">smc fmt</span>{' '}
                {settings.formatOnSave ? 'with format-on-save enabled' : 'format-on-save disabled'}
              </p>
              {settings.formatOnSave && canRunSemanticFileAction ? (
                <div className="diagnostic-callout">
                  <span className="diagnostic-meta-label">Format-on-save behavior</span>
                  <p>
                    Save file runs the canonical <code>smc fmt</code> surface only after a successful disk write.
                    Format file uses one explicit formatter pass and does not trigger a hidden second pass.
                  </p>
                </div>
              ) : null}
              <p className="job-meta">
                compiled artifact:{' '}
                <code>{currentFileArtifactPath ?? 'open a repository-scoped .sm file to derive a canonical .smc path'}</code>
              </p>
              {!isSemanticSource(activeEditorTab.relativePath) ? (
                <p className="empty-state">
                  Current-file compile, check, run, disasm, and format actions are only enabled for `.sm` source files.
                </p>
              ) : null}
              {hasDirtySemanticTabs ? (
                <div className="diagnostic-callout">
                  <span className="diagnostic-meta-label">Workspace formatter guard</span>
                  <p>
                    Format workspace and format check stay disabled until every dirty <code>.sm</code> tab is saved.
                    Workbench does not auto-save those buffers silently before a workspace-wide formatter run.
                  </p>
                  <ul className="bullet-list compact">
                    {dirtySemanticTabTitles.map((title) => (
                      <li key={title}>{title}</li>
                    ))}
                  </ul>
                </div>
              ) : null}
              {latestWorkspaceFormatCheckJob ? (
                <div className="diagnostic-callout">
                  <div className="document-topline">
                    <span className="diagnostic-meta-label">Latest format check</span>
                    <span className={`status-pill ${latestWorkspaceFormatCheckJob.status}`}>
                      {latestWorkspaceFormatCheckJob.status}
                    </span>
                  </div>
                  <p className="job-meta">{latestWorkspaceFormatCheckJob.commandLine}</p>
                  <p className="job-meta">
                    {latestWorkspaceFormatCheckJob.status === 'success'
                      ? 'Canonical formatter check passed for the selected workspace.'
                      : 'Canonical formatter check reported drift. The full captured output is preserved in the jobs panel.'}
                  </p>
                  <button
                    type="button"
                    className="ghost-button"
                    onClick={() => {
                      onSelectJob(latestWorkspaceFormatCheckJob.id)
                      navigate('/overview')
                    }}
                  >
                    Focus format check result
                  </button>
                </div>
              ) : null}
              {canRunSemanticFileAction ? (
                <p className="screen-summary">
                  Current-file check routes to Diagnostics. Current-file run and disasm save and compile the active buffer first, then route to Inspect over the canonical compiled artifact.
                </p>
              ) : null}
              <textarea
                ref={editorTextareaRef}
                className="editor-textarea"
                value={activeEditorTab.content}
                onChange={(event) =>
                  onUpdateEditorContent(activeEditorTab.relativePath, event.target.value)
                }
                onSelect={(event) =>
                  setEditorCursor(
                    deriveCursorPosition(
                      event.currentTarget.value,
                      event.currentTarget.selectionStart ?? 0,
                    ),
                  )
                }
                spellCheck={false}
              />
            </div>
          ) : null}
        </article>
      </section>
    </div>
  )
}

function deriveScaffoldPackageName(selectedWorkspace: WorkspaceSummary | null) {
  if (!selectedWorkspace) {
    return 'semantic-project'
  }

  const rawSource =
    selectedWorkspace.repoRelativePath?.split('/').filter(Boolean).pop() ??
    selectedWorkspace.resolvedPath.split(/[\\/]/).filter(Boolean).pop() ??
    'semantic-project'

  const normalized = rawSource
    .toLowerCase()
    .replace(/[^a-z0-9]+/g, '-')
    .replace(/^-+|-+$/g, '')

  return normalized || 'semantic-project'
}

function normalizeWorkspaceOpenOptions(
  value?: boolean | WorkspaceOpenOptions,
): WorkspaceOpenOptions {
  if (typeof value === 'boolean') {
    return { persist: value }
  }

  return value ?? {}
}

function defaultWorkspaceSuccessMessage(
  workspace: WorkspaceSummary,
  source: WorkspaceOpenSource,
) {
  switch (source) {
    case 'default':
      return 'Restored the saved default workspace for this session.'
    case 'recent':
      return `Reopened ${workspace.repoRelativePath ?? 'the repository root'} from recent workspaces.`
    case 'preset':
      return `Opened ${workspace.repoRelativePath ?? 'the repository root'} through the Workbench shortcut.`
    case 'fallback':
      return 'Workbench fell back to the repository root to keep the session recoverable.'
    case 'manual':
    default:
      return `Opened ${workspace.repoRelativePath ?? 'the repository root'} from the requested workspace path.`
  }
}

function workspaceSourceLabel(source: WorkspaceOpenSource | null) {
  switch (source) {
    case 'default':
      return 'Saved default'
    case 'recent':
      return 'Recent'
    case 'preset':
      return 'Shortcut'
    case 'fallback':
      return 'Fallback'
    case 'manual':
      return 'Manual'
    default:
      return 'Unset'
  }
}

function workspaceSourceTone(source: WorkspaceOpenSource | null) {
  return source === 'fallback' ? 'draft' : 'stable'
}

function describeWorkspaceOpenError(candidate: string, error: unknown) {
  const detail = String(error)
  return `Could not open workspace "${candidate}". Use an absolute path or a repository-relative path that stays inside the repository boundary. ${detail}`
}

function resolveActionCwd(
  action: JobActionSpec,
  adapterContract: AdapterContract | null,
  selectedWorkspace: WorkspaceSummary | null,
) {
  return action.cwdMode === 'repo'
    ? adapterContract?.repoRoot ?? selectedWorkspace?.repoRoot ?? ''
    : selectedWorkspace?.resolvedPath ?? adapterContract?.repoRoot ?? ''
}

function createActionExecutionKey(kind: JobKind, cwd: string, args: string[]) {
  return [kind, cwd, ...args].join('\u001f')
}

function deriveScaffoldPackageNameFromResult(result: ScaffoldProjectResult) {
  return result.packageName
}

function deriveCursorPosition(content: string, selectionStart: number): EditorCursorPosition {
  const safeOffset = Math.max(0, Math.min(selectionStart, content.length))
  const prefix = content.slice(0, safeOffset)
  const lines = prefix.split(/\r?\n/)
  return {
    line: Math.max(0, lines.length - 1),
    character: lines[lines.length - 1]?.length ?? 0,
  }
}

function resolveEditorOffsetFromOneBasedLocation(
  content: string,
  line: number | null,
  column: number | null,
) {
  if (!line || line < 1) {
    return 0
  }

  const normalizedLines = content.split(/\r?\n/)
  const lineIndex = Math.min(Math.max(line - 1, 0), Math.max(normalizedLines.length - 1, 0))
  const characterIndex = Math.max((column ?? 1) - 1, 0)

  let offset = 0
  for (let index = 0; index < lineIndex; index += 1) {
    offset += normalizedLines[index]?.length ?? 0
    offset += 1
  }

  return Math.min(
    offset + Math.min(characterIndex, normalizedLines[lineIndex]?.length ?? 0),
    content.length,
  )
}

function resolveAbsoluteWorkspacePath(
  absolutePath: string,
  workspace: WorkspaceSummary,
) {
  const normalizedAbsolute = absolutePath.replace(/\\/g, '/').toLowerCase()
  const normalizedWorkspace = workspace.resolvedPath.replace(/\\/g, '/').toLowerCase()

  if (!normalizedAbsolute.startsWith(normalizedWorkspace)) {
    return null
  }

  const relative = absolutePath
    .replace(/\\/g, '/')
    .slice(workspace.resolvedPath.replace(/\\/g, '/').length)
    .replace(/^\/+/, '')

  return relative || null
}

function parsePackageManifest(content: string): PackageManifestPreview {
  const preview: PackageManifestPreview = {
    name: null,
    version: null,
    edition: null,
    entry: null,
  }

  let inPackageSection = false

  for (const rawLine of content.split(/\r?\n/)) {
    const line = rawLine.replace(/#.*/, '').trim()
    if (!line) {
      continue
    }

    const sectionMatch = line.match(/^\[(.+)\]$/)
    if (sectionMatch) {
      inPackageSection = sectionMatch[1].trim() === 'package'
      continue
    }

    if (!inPackageSection) {
      continue
    }

    const fieldMatch = line.match(/^([A-Za-z0-9_-]+)\s*=\s*"(.*)"$/)
    if (!fieldMatch) {
      continue
    }

    const [, key, value] = fieldMatch
    switch (key) {
      case 'name':
        preview.name = value
        break
      case 'version':
        preview.version = value
        break
      case 'edition':
        preview.edition = value
        break
      case 'entry':
        preview.entry = value
        break
      default:
        break
    }
  }

  return preview
}

function findCatalogDocument(specCatalog: SpecCatalogSection[], relativePath: string) {
  for (const section of specCatalog) {
    const match = section.documents.find((document) => document.relativePath === relativePath)
    if (match) {
      return match
    }
  }

  return null
}

function DiagnosticsPanel({
  jobs,
  selectedJobId,
  selectedWorkspace,
  onOpenEditorFile,
  onSelectJob,
  onSelectSpecPath,
}: {
  jobs: JobRecord[]
  selectedJobId: string | null
  selectedWorkspace: WorkspaceSummary | null
  onOpenEditorFile: (relativePath: string, options?: EditorOpenOptions) => Promise<void>
  onSelectJob: (jobId: string) => void
  onSelectSpecPath: (value: string) => void
}) {
  const navigate = useNavigate()
  const diagnostics = deriveDiagnosticsFromJobs(jobs)
  const [familyFilter, setFamilyFilter] = useState<DiagnosticFamily | 'all'>('all')
  const [selectedDiagnosticId, setSelectedDiagnosticId] = useState<string | null>(null)

  const availableFamilies = diagnosticFamilyOrder.filter((family) =>
    diagnostics.some((diagnostic) => diagnostic.family === family),
  )
  const effectiveFamilyFilter =
    familyFilter !== 'all' && !availableFamilies.includes(familyFilter)
      ? 'all'
      : familyFilter
  const visibleDiagnostics =
    effectiveFamilyFilter === 'all'
      ? diagnostics
      : diagnostics.filter((diagnostic) => diagnostic.family === effectiveFamilyFilter)
  const effectiveSelectedDiagnosticId =
    selectedDiagnosticId && visibleDiagnostics.some((entry) => entry.id === selectedDiagnosticId)
      ? selectedDiagnosticId
      : visibleDiagnostics[0]?.id ?? null
  const selectedDiagnostic =
    visibleDiagnostics.find((diagnostic) => diagnostic.id === effectiveSelectedDiagnosticId) ??
    visibleDiagnostics[0] ??
    null

  const diagnosticCounts = diagnosticFamilyOrder
    .map((family) => ({
      family,
      count: diagnostics.filter((diagnostic) => diagnostic.family === family).length,
    }))
    .filter((entry) => entry.count > 0)

  const errorCount = diagnostics.filter((diagnostic) => diagnostic.severity === 'error').length
  const warningCount = diagnostics.filter((diagnostic) => diagnostic.severity === 'warning').length
  const jobsWithDiagnostics = new Set(diagnostics.map((diagnostic) => diagnostic.jobId)).size
  const openableRelativePath =
    selectedDiagnostic && selectedWorkspace
      ? resolveDiagnosticWorkspacePath(selectedDiagnostic.filePath, selectedWorkspace)
      : null
  const docMapping = selectedDiagnostic ? diagnosticDocMapping(selectedDiagnostic) : null
  const relatedDocs = selectedDiagnostic ? diagnosticDocLinks(selectedDiagnostic) : []

  return (
    <div className="screen-stack">
      <section className="diagnostics-summary-grid">
        <article className="screen-card">
          <p className="card-kicker">Diagnostics summary</p>
          <h3>Derived from real `smc` / `svm` output</h3>
          <div className="diagnostics-metrics">
            <div className="diagnostics-metric">
              <strong>{diagnostics.length}</strong>
              <span>Total diagnostics</span>
            </div>
            <div className="diagnostics-metric">
              <strong>{errorCount}</strong>
              <span>Errors</span>
            </div>
            <div className="diagnostics-metric">
              <strong>{warningCount}</strong>
              <span>Warnings</span>
            </div>
            <div className="diagnostics-metric">
              <strong>{jobsWithDiagnostics}</strong>
              <span>Jobs with diagnostics</span>
            </div>
          </div>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Family filters</p>
          <h3>Parse, type, module, verify, runtime</h3>
          <div className="diagnostics-filter-row">
            <button
              type="button"
              className={`diagnostics-filter-button ${effectiveFamilyFilter === 'all' ? 'diagnostics-filter-button-active' : ''}`}
              onClick={() => setFamilyFilter('all')}
            >
              All
            </button>
            {diagnosticCounts.map((entry) => (
              <button
                key={entry.family}
                type="button"
                className={`diagnostics-filter-button ${effectiveFamilyFilter === entry.family ? 'diagnostics-filter-button-active' : ''}`}
                onClick={() => setFamilyFilter(entry.family)}
              >
                {diagnosticFamilyLabel(entry.family)} <span>{entry.count}</span>
              </button>
            ))}
          </div>
          <p className="screen-summary">
            Workbench groups diagnostics from command output, but preserves the original code, line, column, job, and raw message block when those fields exist.
          </p>
        </article>
      </section>

      {visibleDiagnostics.length === 0 ? (
        <article className="screen-card">
          <p className="card-kicker">No diagnostics yet</p>
          <p className="empty-state">
            Run `smc check`, `smc compile`, `svm run`, or `svm disasm` from the cockpit or current-file actions to populate this panel.
          </p>
        </article>
      ) : (
        <section className="diagnostics-shell">
          <article className="screen-card diagnostics-list-panel">
            <p className="card-kicker">Diagnostics ledger</p>
            <h3>Grouped by public failure family</h3>
            <div className="diagnostics-groups">
              {diagnosticFamilyOrder
                .filter((family) =>
                  visibleDiagnostics.some((diagnostic) => diagnostic.family === family),
                )
                .map((family) => {
                  const familyDiagnostics = visibleDiagnostics.filter(
                    (diagnostic) => diagnostic.family === family,
                  )

                  return (
                    <section key={family} className="diagnostics-group">
                      <div className="diagnostics-group-header">
                        <h4>{diagnosticFamilyLabel(family)}</h4>
                        <span className="status-pill stable">{familyDiagnostics.length}</span>
                      </div>
                      <div className="diagnostics-group-list">
                        {familyDiagnostics.map((diagnostic) => (
                          <button
                            key={diagnostic.id}
                            type="button"
                            className={`diagnostic-card ${selectedDiagnostic?.id === diagnostic.id ? 'diagnostic-card-active' : ''}`}
                            onClick={() => {
                              setSelectedDiagnosticId(diagnostic.id)
                              onSelectJob(diagnostic.jobId)
                            }}
                          >
                            <div className="diagnostic-card-topline">
                              <span className={`status-pill ${severityPillClass(diagnostic.severity)}`}>
                                {diagnostic.severity}
                              </span>
                              {diagnostic.code ? (
                                <code className="diagnostic-code">{diagnostic.code}</code>
                              ) : null}
                            </div>
                            <strong>{diagnostic.message}</strong>
                            <p className="job-meta">{diagnostic.jobLabel}</p>
                            <p className="job-meta">
                              {formatDiagnosticLocation(diagnostic)}
                            </p>
                          </button>
                        ))}
                      </div>
                    </section>
                  )
                })}
            </div>
          </article>

          <article className="screen-card diagnostics-detail-panel">
            <p className="card-kicker">Selected diagnostic</p>
            {selectedDiagnostic ? (
              <div className="diagnostics-detail-stack">
                <div className="diagnostic-detail-header">
                  <div>
                    <h3>{selectedDiagnostic.message}</h3>
                    <p className="job-meta">
                      {diagnosticFamilyLabel(selectedDiagnostic.family)} from {selectedDiagnostic.jobLabel}
                    </p>
                  </div>
                  <div className="status-cluster">
                    <span className={`status-pill ${severityPillClass(selectedDiagnostic.severity)}`}>
                      {selectedDiagnostic.severity}
                    </span>
                    {selectedDiagnostic.code ? (
                      <code className="diagnostic-code">{selectedDiagnostic.code}</code>
                    ) : null}
                  </div>
                </div>

                <div className="diagnostic-meta-grid">
                  <div>
                    <span className="diagnostic-meta-label">Location</span>
                    <code>{formatDiagnosticLocation(selectedDiagnostic)}</code>
                  </div>
                  <div>
                    <span className="diagnostic-meta-label">Job</span>
                    <code>{selectedDiagnostic.commandLine}</code>
                  </div>
                  <div>
                    <span className="diagnostic-meta-label">Working directory</span>
                    <code>{selectedDiagnostic.cwd}</code>
                  </div>
                  <div>
                    <span className="diagnostic-meta-label">Output channel</span>
                    <code>{selectedDiagnostic.sourceChannel}</code>
                  </div>
                  {selectedDiagnostic.functionName ? (
                    <div>
                      <span className="diagnostic-meta-label">Function</span>
                      <code>{selectedDiagnostic.functionName}</code>
                    </div>
                  ) : null}
                  {selectedDiagnostic.offsetHex ? (
                    <div>
                      <span className="diagnostic-meta-label">Byte offset</span>
                      <code>{selectedDiagnostic.offsetHex}</code>
                    </div>
                  ) : null}
                  {selectedDiagnostic.instruction !== null ? (
                    <div>
                      <span className="diagnostic-meta-label">Instruction</span>
                      <code>{selectedDiagnostic.instruction}</code>
                    </div>
                  ) : null}
                </div>

                <div className="field-actions">
                  <button
                    type="button"
                    className="ghost-button"
                    onClick={() => onSelectJob(selectedDiagnostic.jobId)}
                  >
                    Focus job in history
                  </button>
                    {openableRelativePath ? (
                      <button
                        type="button"
                        className="action-button"
                        onClick={() => {
                          void onOpenEditorFile(openableRelativePath, {
                            line: selectedDiagnostic.line,
                            column: selectedDiagnostic.column,
                            source: 'diagnostic',
                          })
                          navigate('/project')
                        }}
                      >
                        {selectedDiagnostic.line !== null
                          ? `Open source at ${selectedDiagnostic.line}:${selectedDiagnostic.column ?? 1}`
                          : 'Open source file'}
                      </button>
                    ) : null}
                </div>

                {docMapping ? (
                  <div className="diagnostic-callout">
                    <div className="document-topline">
                      <span className="diagnostic-meta-label">Canonical mapping status</span>
                      <span
                        className={`status-pill ${docMapping.status === 'generic' ? 'draft' : 'stable'}`}
                      >
                        {diagnosticDocMappingLabel(docMapping.status)}
                      </span>
                    </div>
                    <p>{docMapping.detail}</p>
                    {docMapping.status === 'generic' ? (
                      <p className="job-meta">
                        No exact code or family mapping is recorded yet, so Workbench does not guess additional
                        spec links.
                      </p>
                    ) : null}
                  </div>
                ) : null}

                {relatedDocs.length > 0 ? (
                  <div className="diagnostic-related-docs">
                    <span className="diagnostic-meta-label">Canonical spec and error docs</span>
                    <div className="diagnostic-doc-links">
                      {relatedDocs.map((document) => (
                        <button
                          key={document.relativePath}
                          type="button"
                          className="diagnostic-doc-button"
                          onClick={() => {
                            onSelectSpecPath(document.relativePath)
                            navigate('/spec')
                          }}
                        >
                          <strong>{document.label}</strong>
                          <span>{document.relativePath}</span>
                        </button>
                      ))}
                    </div>
                  </div>
                ) : null}

                {selectedDiagnostic.helpText ? (
                  <div className="diagnostic-callout">
                    <span className="diagnostic-meta-label">Why this error?</span>
                    <p>{selectedDiagnostic.helpText}</p>
                  </div>
                ) : null}

                <div>
                  <span className="diagnostic-meta-label">Raw diagnostic block</span>
                  <pre className="terminal-output terminal-output-error">
                    {selectedDiagnostic.rawBlock}
                  </pre>
                </div>

                {selectedJobId === selectedDiagnostic.jobId ? (
                  <p className="job-meta">
                    This diagnostic already points at the currently selected job in the cockpit ledger.
                  </p>
                ) : null}
              </div>
            ) : (
              <p className="empty-state">Select a diagnostic to inspect its preserved fields.</p>
            )}
          </article>
        </section>
      )}
    </div>
  )
}

function InspectPanel({
  jobs,
  selectedJobId,
  onSelectJob,
}: {
  jobs: JobRecord[]
  selectedJobId: string | null
  onSelectJob: (jobId: string) => void
}) {
  const [familyFilter, setFamilyFilter] = useState<InspectFamily | 'all'>('all')
  const inspectableJobs = deriveInspectableJobs(jobs)
  const familyCounts = inspectFamilyOrder
    .map((family) => ({
      family,
      count: inspectableJobs.filter((entry) => entry.family === family).length,
    }))
    .filter((entry) => entry.count > 0)
  const effectiveFilter =
    familyFilter !== 'all' && !familyCounts.some((entry) => entry.family === familyFilter)
      ? 'all'
      : familyFilter
  const visibleJobs =
    effectiveFilter === 'all'
      ? inspectableJobs
      : inspectableJobs.filter((entry) => entry.family === effectiveFilter)
  const selectedInspectableJob =
    visibleJobs.find((entry) => entry.job.id === selectedJobId) ?? visibleJobs[0] ?? null
  const inspectSignals = selectedInspectableJob
    ? deriveInspectSignals(selectedInspectableJob)
    : []
  const disasmLineCount =
    selectedInspectableJob?.family === 'disasm' && selectedInspectableJob.stdoutText
      ? selectedInspectableJob.stdoutText
          .split(/\r?\n/)
          .filter((line) => line.trim().length > 0).length
      : 0

  return (
    <div className="screen-stack">
      <section className="inspect-summary-grid">
        <article className="screen-card">
          <p className="card-kicker">Inspectable jobs</p>
          <h3>Disasm, verify, and verified-path execution</h3>
          <div className="inspect-metrics">
            <div className="inspect-metric">
              <strong>{inspectableJobs.length}</strong>
              <span>Total inspectable jobs</span>
            </div>
            {familyCounts.map((entry) => (
              <div key={entry.family} className="inspect-metric">
                <strong>{entry.count}</strong>
                <span>{inspectFamilyLabel(entry.family)}</span>
              </div>
            ))}
          </div>
        </article>

        <article className="screen-card">
          <p className="card-kicker">Scope guard</p>
          <h3>What this panel is allowed to claim</h3>
          <ul className="bullet-list">
            <li>SemCode text comes only from real `svm disasm` or `smc disasm` stdout.</li>
            <li>Verification status comes only from `smc verify` stdout and stderr.</li>
            <li>Verified-run status comes only from real CLI exit codes and captured output.</li>
          </ul>
          <div className="diagnostics-filter-row">
            <button
              type="button"
              className={`diagnostics-filter-button ${effectiveFilter === 'all' ? 'diagnostics-filter-button-active' : ''}`}
              onClick={() => setFamilyFilter('all')}
            >
              All
            </button>
            {familyCounts.map((entry) => (
              <button
                key={entry.family}
                type="button"
                className={`diagnostics-filter-button ${effectiveFilter === entry.family ? 'diagnostics-filter-button-active' : ''}`}
                onClick={() => setFamilyFilter(entry.family)}
              >
                {inspectFamilyLabel(entry.family)} <span>{entry.count}</span>
              </button>
            ))}
          </div>
        </article>
      </section>

      {visibleJobs.length === 0 ? (
        <article className="screen-card">
          <p className="card-kicker">No inspectable jobs yet</p>
          <p className="empty-state">
            Run `smc check --trace-cache`, `smc verify`, `svm disasm`, or a verified bytecode execution from the cockpit to populate the inspector.
          </p>
        </article>
      ) : (
        <section className="inspect-shell">
          <article className="screen-card inspect-list-panel">
            <p className="card-kicker">Inspect ledger</p>
            <h3>Real jobs only</h3>
            <div className="inspect-job-list">
              {visibleJobs.map((entry) => (
                <button
                  key={entry.job.id}
                  type="button"
                  className={`inspect-job-card ${selectedInspectableJob?.job.id === entry.job.id ? 'inspect-job-card-active' : ''}`}
                  onClick={() => onSelectJob(entry.job.id)}
                >
                  <div className="diagnostic-card-topline">
                    <span className="status-pill stable">{inspectFamilyLabel(entry.family)}</span>
                    <span
                      className={`status-pill ${entry.job.status === 'success' ? 'stable' : entry.job.status === 'running' ? 'running' : 'draft'}`}
                    >
                      {entry.job.status}
                    </span>
                  </div>
                  <strong>{entry.job.label}</strong>
                  <div className="inspect-card-meta">
                    <span className="diagnostic-meta-label">Derived summary</span>
                    <p className="job-meta">{entry.summary}</p>
                    <p className="job-meta">{inspectSummarySourceLabel(entry.summarySource)}</p>
                  </div>
                  <div className="inspect-card-meta">
                    <span className="diagnostic-meta-label">Artifact path</span>
                    <p className="job-meta">
                      {entry.artifactPath ?? 'Not explicit in captured command arguments.'}
                    </p>
                    <p className="job-meta">{inspectArtifactSourceLabel(entry.artifactSource)}</p>
                  </div>
                </button>
              ))}
            </div>
          </article>

          <article className="screen-card inspect-detail-panel">
            <p className="card-kicker">Selected inspection</p>
            {selectedInspectableJob ? (
              <div className="inspect-detail-stack">
                <div className="diagnostic-detail-header">
                  <div>
                    <h3>{selectedInspectableJob.job.label}</h3>
                    <p className="job-meta">{inspectFamilyDescription(selectedInspectableJob.family)}</p>
                  </div>
                  <div className="status-cluster">
                    <span className="status-pill stable">{inspectFamilyLabel(selectedInspectableJob.family)}</span>
                    <span
                      className={`status-pill ${selectedInspectableJob.job.status === 'success' ? 'stable' : selectedInspectableJob.job.status === 'running' ? 'running' : 'draft'}`}
                    >
                      {selectedInspectableJob.job.status}
                    </span>
                  </div>
                </div>

                <div className="diagnostic-meta-grid">
                  <div>
                    <span className="diagnostic-meta-label">Artifact</span>
                    <code>
                      {selectedInspectableJob.artifactPath ??
                        'Not explicit in captured command arguments.'}
                    </code>
                  </div>
                  <div>
                    <span className="diagnostic-meta-label">Artifact source</span>
                    <code>{inspectArtifactSourceLabel(selectedInspectableJob.artifactSource)}</code>
                  </div>
                  <div>
                    <span className="diagnostic-meta-label">Command</span>
                    <code>{selectedInspectableJob.job.commandLine}</code>
                  </div>
                  <div>
                    <span className="diagnostic-meta-label">Working directory</span>
                    <code>{selectedInspectableJob.job.cwd}</code>
                  </div>
                  <div>
                    <span className="diagnostic-meta-label">Exit code</span>
                    <code>{selectedInspectableJob.job.exitCode ?? 'running'}</code>
                  </div>
                  <div>
                    <span className="diagnostic-meta-label">Duration</span>
                    <code>
                      {selectedInspectableJob.job.durationMs
                        ? `${selectedInspectableJob.job.durationMs} ms`
                        : 'n/a'}
                    </code>
                  </div>
                  {selectedInspectableJob.family === 'disasm' ? (
                    <div>
                      <span className="diagnostic-meta-label">Disasm lines</span>
                      <code>{disasmLineCount}</code>
                    </div>
                  ) : null}
                </div>

                <div className="inspect-callout">
                  <span className="diagnostic-meta-label">Derived summary</span>
                  <p>{selectedInspectableJob.summary}</p>
                  <p className="job-meta">
                    {inspectSummarySourceLabel(selectedInspectableJob.summarySource)}
                  </p>
                </div>

                <div className="inspect-callout">
                  <span className="diagnostic-meta-label">Inspector contract</span>
                  <p>
                    {selectedInspectableJob.family === 'trace'
                      ? 'Workbench displays trace-cache output exactly as emitted by the public smc check surface. It does not infer compiler ownership or cache semantics beyond that output.'
                      : selectedInspectableJob.family === 'verify'
                      ? 'Workbench displays the verifier result exactly as emitted by smc verify. It does not recompute verification.'
                      : selectedInspectableJob.family === 'disasm'
                        ? 'Workbench displays raw SemCode disassembly from the public CLI surface. It does not own a second bytecode model.'
                        : 'Workbench displays verified execution status from the public run surface. It does not infer runtime semantics beyond exit code and captured output.'}
                  </p>
                </div>

                <div className="inspect-callout">
                  <span className="diagnostic-meta-label">Derived signals</span>
                  <p>
                    Signal cards below are derived from preserved stdout, stderr, exit code, and
                    artifact metadata. Raw output remains authoritative.
                  </p>
                </div>

                <div className="inspect-signals-grid">
                  {inspectSignals.map((signal) => (
                    <section key={`${signal.label}-${signal.detail}`} className="inspect-signal-card">
                      <div className="diagnostic-card-topline">
                        <strong>{signal.label}</strong>
                        <span className={`status-pill ${signal.tone}`}>{signal.tone}</span>
                      </div>
                      <p className="job-meta">{signal.detail}</p>
                    </section>
                  ))}
                </div>

                <div className="inspect-output-stack">
                  <OutputPane
                    key={`${selectedInspectableJob.job.id}-stdout`}
                    label="stdout"
                    content={selectedInspectableJob.stdoutText ?? ''}
                    emptyMessage="No stdout captured for this job."
                  />
                  <OutputPane
                    key={`${selectedInspectableJob.job.id}-stderr`}
                    label="stderr"
                    content={selectedInspectableJob.stderrText ?? ''}
                    emptyMessage="No stderr captured for this job."
                    tone="error"
                  />
                </div>
              </div>
            ) : (
              <p className="empty-state">
                Select an inspectable job to view raw disasm, verify, or verified-run output.
              </p>
            )}
          </article>
        </section>
      )}
    </div>
  )
}

function SettingsPanel({
  settings,
  selectedWorkspace,
  onUpdateSettings,
}: {
  settings: WorkbenchSettings
  selectedWorkspace: WorkspaceSummary | null
  onUpdateSettings: (next: Partial<WorkbenchSettings>) => void
}) {
  return (
    <section className="command-grid">
      <article className="screen-card">
        <p className="card-kicker">Local settings</p>
        <h3>Preferences that stay in the UI layer</h3>
        <div className="settings-grid">
          <label className="toggle-row">
            <span>
              <strong>Default workspace</strong>
              <p className="job-meta">
                Persist the currently selected canonical workspace for future sessions.
              </p>
            </span>
            <code>{settings.defaultWorkspacePath ?? selectedWorkspace?.resolvedPath ?? 'unset'}</code>
          </label>

          <label className="toggle-row">
            <span>
              <strong>Preferred shell</strong>
              <p className="job-meta">Current bootstrap only supports PowerShell-based release flows.</p>
            </span>
            <span className="status-pill stable">{settings.preferredShell}</span>
          </label>

          <label className="toggle-row toggle-row-interactive">
            <span>
              <strong>Format on save</strong>
              <p className="job-meta">Uses the canonical `smc fmt` surface after saving `.sm` files.</p>
            </span>
            <input
              type="checkbox"
              checked={settings.formatOnSave}
              onChange={(event) =>
                onUpdateSettings({ formatOnSave: event.target.checked })
              }
            />
          </label>

          <label className="toggle-row toggle-row-interactive">
            <span>
              <strong>Show experimental workflows</strong>
              <p className="job-meta">Controls visibility only; it must not widen Semantic scope.</p>
            </span>
            <input
              type="checkbox"
              checked={settings.showExperimental}
              onChange={(event) =>
                onUpdateSettings({ showExperimental: event.target.checked })
              }
            />
          </label>

          <label className="toggle-row toggle-row-input">
            <span>
              <strong>`smlsp` command</strong>
              <p className="job-meta">
                External editor-protocol bridge command. Workbench only shells out to this process;
                it does not implement hover, definition, diagnostics, or formatting itself.
              </p>
            </span>
            <input
              type="text"
              className="text-field settings-text-field"
              value={settings.smlspCommand}
              onChange={(event) =>
                onUpdateSettings({ smlspCommand: event.target.value || 'smlsp' })
              }
            />
          </label>
        </div>
      </article>

      <article className="screen-card">
        <p className="card-kicker">Scope guard</p>
        <h3>What settings cannot do</h3>
        <ul className="bullet-list">
          <li>They cannot enable hidden runtime semantics.</li>
          <li>They cannot widen PROMETHEUS scope or alter capability rules.</li>
          <li>They cannot override repository truth for readiness or compatibility.</li>
          <li>They exist only for shell behavior and local workflow preferences.</li>
        </ul>
      </article>
    </section>
  )
}

function latestJobOfKind(jobs: JobRecord[], kind: JobKind) {
  return jobs.find((job) => job.kind === kind)
}

function latestJobMatching(
  jobs: JobRecord[],
  predicate: (job: JobRecord) => boolean,
) {
  return jobs.find(predicate)
}

type ReleaseGateRow = {
  label: string
  detail: string
  job: JobRecord | null
  truthSource: string
  meaning: string
}

type ReleaseDocsAlignmentItem = {
  label: string
  ok: boolean
  detail: string
  truthSource: string
}

type ReleaseConsoleView = {
  releaseDocuments: SpecCatalogDocument[]
  releaseManifest: ReleaseBundleManifest | null
  assetSmoke: AssetSmokeSnapshot | null
  gateRows: ReleaseGateRow[]
  docsAlignment: ReleaseDocsAlignmentItem[]
  assetSmokeDocPath: string
  knownLimitSources: OverviewDocument[]
}

function findReleaseDocumentBySuffix(
  releaseDocuments: SpecCatalogDocument[],
  suffix: string,
) {
  return (
    releaseDocuments.find((document) => document.relativePath.endsWith(suffix)) ?? null
  )
}

function deriveReleaseConsoleView(
  overviewSnapshot: OverviewSnapshot | null,
  specCatalog: SpecCatalogSection[],
  jobs: JobRecord[],
): ReleaseConsoleView {
  const releaseSection = specCatalog.find((section) => section.key === 'release')
  const releaseDocuments = releaseSection?.documents ?? []
  const releaseManifest = overviewSnapshot?.releaseManifest ?? null
  const assetSmoke = overviewSnapshot?.assetSmoke ?? null
  const latestTrace = latestJobMatching(
    jobs,
    (job) => job.kind === 'smc' && effectiveResolvedCommand(job).includes('--trace-cache'),
  )
  const stablePolicyDoc = findReleaseDocumentBySuffix(
    releaseDocuments,
    'stable_release_policy.md',
  )
  const assetSmokeDoc = findReleaseDocumentBySuffix(
    releaseDocuments,
    'release_asset_smoke_matrix.md',
  )
  const knownLimitSources =
    overviewSnapshot?.releaseDocs.filter((document) => Boolean(document.highlight)) ??
    []
  const gateRows: ReleaseGateRow[] = [
    ...releaseValidationPlan.map((action) => {
      const job =
        latestJobMatching(jobs, (candidate) => jobMatchesAction(candidate, action)) ?? null

      return {
        label: action.label.replace(/^Clean validation:\s*/, ''),
        detail: action.args.join(' '),
        job,
        truthSource: job
          ? `latest matching local job: ${job.commandLine}`
          : `no local job recorded yet for public command ${action.kind} ${action.args.join(' ')}`,
        meaning: 'Badge reflects only the latest matching local job status.',
      }
    }),
    {
      label: 'Trace workflow',
      detail: 'smc check --trace-cache',
      job: latestTrace ?? null,
      truthSource: latestTrace
        ? `latest matching local job: ${latestTrace.commandLine}`
        : 'no local trace workflow recorded yet',
      meaning: 'Badge reflects only the latest local trace workflow status.',
    },
  ]
  const docsAlignment: ReleaseDocsAlignmentItem[] = [
    {
      label: 'Baseline manifest present',
      ok: overviewSnapshot?.baselineManifestExists ?? false,
      detail: overviewSnapshot?.baselineManifestPath ?? 'No baseline manifest path resolved.',
      truthSource: overviewSnapshot?.baselineManifestPath ?? 'baseline manifest path unresolved',
    },
    {
      label: 'Release docs indexed',
      ok: releaseDocuments.length >= 5,
      detail: `${releaseDocuments.length} release document(s) indexed in the canonical navigator.`,
      truthSource: `canonical release section in docs/spec and docs/roadmap (${releaseDocuments.length} indexed document(s))`,
    },
    {
      label: 'Stable policy doc present',
      ok: Boolean(stablePolicyDoc),
      detail:
        stablePolicyDoc?.absolutePath ??
        'stable_release_policy.md was not found in the indexed release bundle.',
      truthSource:
        stablePolicyDoc?.absolutePath ??
        'expected canonical doc: docs/roadmap/stable_release_policy.md',
    },
    {
      label: 'Asset smoke tag recorded',
      ok: Boolean(assetSmoke?.validatedTag),
      detail: assetSmoke?.validatedTag ?? 'No validated asset tag found in release_asset_smoke_matrix.md.',
      truthSource:
        assetSmokeDoc?.absolutePath ??
        'expected canonical doc: docs/roadmap/release_asset_smoke_matrix.md',
    },
  ]

  return {
    releaseDocuments,
    releaseManifest,
    assetSmoke,
    gateRows,
    docsAlignment,
    assetSmokeDocPath:
      assetSmokeDoc?.absolutePath ??
      'docs/roadmap/release_asset_smoke_matrix.md',
    knownLimitSources,
  }
}

function jobMatchesAction(job: JobRecord, action: JobActionSpec) {
  if (job.kind !== action.kind) {
    return false
  }

  const command = effectiveResolvedCommand(job)
  return action.args.every((arg) => command.includes(arg))
}

function buildReleaseReportMarkdown({
  overviewSnapshot,
  releaseView,
}: {
  overviewSnapshot: OverviewSnapshot
  releaseView: ReleaseConsoleView
}) {
  const { gateRows, docsAlignment, releaseManifest, assetSmoke, knownLimitSources } = releaseView
  const lines = [
    '# Workbench Release Console Report',
    '',
    `- branch: \`${overviewSnapshot.branch}\``,
    `- commit: \`${overviewSnapshot.shortCommit}\``,
    `- baseline tag: \`${overviewSnapshot.baselineTagName}\``,
    `- baseline tag on head: ${overviewSnapshot.baselineTagPointsAtHead ? 'yes' : 'no'}`,
    `- baseline manifest: \`${overviewSnapshot.baselineManifestPath}\``,
    '',
    '## Gates',
    '',
    ...gateRows.flatMap((gate) => [
      `### ${gate.label}`,
      `- status: ${gate.job?.status ?? 'not run'}`,
      `- detail: \`${gate.detail}\``,
      `- command: \`${gate.job?.commandLine ?? 'not recorded'}\``,
      `- cwd: \`${gate.job?.cwd ?? overviewSnapshot.repoRoot}\``,
      `- exit: ${gate.job?.exitCode ?? 'n/a'}`,
      `- duration_ms: ${gate.job?.durationMs ?? 'n/a'}`,
      `- truth_source: ${gate.truthSource}`,
      `- meaning: ${gate.meaning}`,
      '',
    ]),
    '## Docs Alignment',
    '',
    ...docsAlignment.flatMap((item) => [
      `- ${item.ok ? '[x]' : '[ ]'} ${item.label}: ${item.detail}`,
      `  - truth_source: ${item.truthSource}`,
    ]),
    '',
    '## Baseline Artifact Inventory',
    '',
    `- manifest_path: \`${overviewSnapshot.baselineManifestPath}\``,
    ...(releaseManifest
      ? [
          `- generated_at: ${releaseManifest.generatedAt}`,
          `- current_scope: ${releaseManifest.currentScope}`,
          ...releaseManifest.documentationBundle.map((path) => `- documentation: \`${path}\``),
          ...releaseManifest.validationTests.map((command) => `- validation_test: \`${command}\``),
          ...releaseManifest.snapshotDirectories.map((path) => `- snapshot_dir: \`${path}\``),
        ]
      : ['- No parsed release manifest snapshot available.']),
    '',
    '## Known Limits',
    '',
    ...(overviewSnapshot.knownLimits.length > 0
      ? overviewSnapshot.knownLimits.map((limit) => `- ${limit}`)
      : ['- No known limits extracted from readiness docs.']),
    `- truth_sources: ${
      knownLimitSources.length > 0
        ? knownLimitSources.map((document) => document.path).join(', ')
        : 'No explicit highlight source recorded in release docs.'
    }`,
    '',
    '## Asset Smoke',
    '',
    `- source_path: \`${releaseView.assetSmokeDocPath}\``,
    '- badge_meaning: recorded means the validated tag is present in the canonical smoke matrix, not that a fresh publish smoke just happened in this session.',
    `- validated_tag: \`${assetSmoke?.validatedTag ?? 'not recorded'}\``,
    ...(assetSmoke?.validatedAssets.length
      ? assetSmoke.validatedAssets.map((asset) => `- asset: \`${asset}\``)
      : ['- No validated assets recorded.']),
    '',
  ]

  if (assetSmoke?.scenarios.length) {
    lines.push('## Smoke Scenarios', '')
    for (const scenario of assetSmoke.scenarios) {
      lines.push(`### ${scenario.scenario}`)
      lines.push(`- source: ${scenario.source}`)
      lines.push(`- validation: ${scenario.validation}`)
      lines.push(`- expected: ${scenario.expectedSignal}`)
      lines.push(`- result: ${scenario.currentResult}`)
      lines.push('')
    }
  }

  return `${lines.join('\n').trimEnd()}\n`
}

function normalizeSmlspError(message: string, command: string) {
  const trimmed = message.trim()
  const lower = trimmed.toLowerCase()

  if (
    lower.includes('failed to start smlsp command') ||
    lower.includes('command cannot be empty')
  ) {
    return `Failed to start experimental smlsp bridge with \`${command}\`. Check the \`smlsp\` command in Settings, install the executable or point it to a full path, or disable experimental workflows if smlsp is unavailable.\n\nRaw error: ${trimmed}`
  }

  if (lower.includes('timed out waiting for smlsp response')) {
    return `The experimental smlsp process started but did not answer the protocol handshake in time. Verify that \`${command}\` is a working language-server command and not a different executable.\n\nRaw error: ${trimmed}`
  }

  if (lower.includes('workspace root') || lower.includes('document path')) {
    return `The experimental smlsp bridge refused the current workspace or document path. Reopen a canonical workspace inside the repository and retry on a real \`.sm\` buffer.\n\nRaw error: ${trimmed}`
  }

  return trimmed
}

export default App
