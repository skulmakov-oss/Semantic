export type WorkspaceSummary = {
  repoRoot: string
  resolvedPath: string
  repoRelativePath: string | null
  isRepoRoot: boolean
}

export type RecentWorkspace = {
  path: string
  repoRelativePath: string | null
  openedAtIso: string
}

export type WorkbenchSettings = {
  defaultWorkspacePath: string | null
  preferredShell: 'pwsh'
  formatOnSave: boolean
  showExperimental: boolean
  smlspCommand: string
}

export type StoredWorkbenchState = {
  recentWorkspaces: RecentWorkspace[]
  settings: WorkbenchSettings
}

const STORAGE_KEY = 'semantic-workbench.state.v1'

const defaultState: StoredWorkbenchState = {
  recentWorkspaces: [],
  settings: {
    defaultWorkspacePath: null,
    preferredShell: 'pwsh',
    formatOnSave: false,
    showExperimental: false,
    smlspCommand: 'smlsp',
  },
}

export function loadWorkbenchState(): StoredWorkbenchState {
  try {
    const raw = window.localStorage.getItem(STORAGE_KEY)
    if (!raw) {
      return defaultState
    }

    const parsed = JSON.parse(raw) as Partial<StoredWorkbenchState>
    return {
      recentWorkspaces: Array.isArray(parsed.recentWorkspaces)
        ? parsed.recentWorkspaces.filter(
            (entry): entry is RecentWorkspace =>
              typeof entry?.path === 'string' &&
              typeof entry?.openedAtIso === 'string',
          )
        : [],
      settings: {
        defaultWorkspacePath:
          typeof parsed.settings?.defaultWorkspacePath === 'string'
            ? parsed.settings.defaultWorkspacePath
            : null,
        preferredShell: 'pwsh',
        formatOnSave: Boolean(parsed.settings?.formatOnSave),
        showExperimental: Boolean(parsed.settings?.showExperimental),
        smlspCommand:
          typeof parsed.settings?.smlspCommand === 'string' &&
          parsed.settings.smlspCommand.trim().length > 0
            ? parsed.settings.smlspCommand
            : 'smlsp',
      },
    }
  } catch {
    return defaultState
  }
}

export function saveWorkbenchState(state: StoredWorkbenchState) {
  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(state))
}

export function mergeRecentWorkspace(
  current: RecentWorkspace[],
  workspace: WorkspaceSummary,
): RecentWorkspace[] {
  const nextEntry: RecentWorkspace = {
    path: workspace.resolvedPath,
    repoRelativePath: workspace.repoRelativePath,
    openedAtIso: new Date().toISOString(),
  }

  return [nextEntry, ...current.filter((entry) => entry.path !== workspace.resolvedPath)].slice(
    0,
    6,
  )
}
