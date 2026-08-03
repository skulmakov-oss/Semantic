# Semantic Workbench (React/TypeScript/Tauri — archived, non-canonical)

> **Status: archived reference implementation, moved out of the canonical
> path.** This directory used to live at `apps/workbench`; it now lives at
> `apps/workbench_ts_tauri_legacy` to make its non-canonical status
> unambiguous from the path alone. The canonical Semantic Workbench is
> `examples/workbench_semantic/` — a native Semantic + Prom UI application
> with no Node/npm/Vite/React/Tauri/WebView dependency anywhere on its build
> path (see [`docs/workbench/native_architecture.md`](../../docs/workbench/native_architecture.md)).
> This React/TypeScript/Tauri app predates that architecture and is kept only
> for its information architecture (screen inventory, host-capability list)
> and as a historical reference. It is not built by any CI workflow and must
> not be treated as a source of Workbench domain logic, command-bus
> architecture, or UI contracts going forward.

Semantic Workbench is the desktop orchestration shell for the Semantic
repository.

This app is intentionally scoped as a UI and command-orchestration layer over
public Semantic surfaces. It does not own compiler, verifier, VM, PROMETHEUS,
or release semantics.

## Current Slice

This bootstrap slice provides:

- React + TypeScript frontend shell
- Tauri desktop wrapper
- route layout for overview, project, spec, diagnostics, inspect, release, and
  settings
- configuration for local dev and debug builds

## Commands

```powershell
npm install
npm run dev
npm run lint
npm run build
npm run tauri:build -- --debug --no-bundle
```

## Beta Packaging

```powershell
pwsh -File ..\..\scripts\package_workbench_beta.ps1
```

This builds the release executable, creates a portable beta zip, launches the
packaged app for a short smoke window, and records evidence under
`artifacts/workbench/beta-smoke/`.

## Beta Notes

See:

- `docs/workbench/beta_release_notes.md`
- `docs/workbench/beta_packaging.md`

These pages distinguish stable-now versus experimental workflows and document
the current beta known limits without promising behavior beyond `main`.

## Non-Canonical / Experimental Surfaces (evidence)

Two Tauri backend modules in `src-tauri/src/` are live, wired command
handlers (registered in `lib.rs`'s `invoke_handler`, not dead code) that
warrant explicit non-canonical status rather than silent archival:

- **`lsp_bridge.rs`** (`run_smlsp_bridge`, Tauri command `smlsp_bridge_request`
  at `lib.rs:102`): spawns a subprocess and speaks an LSP-shaped JSON
  request/response (`SmlspBridgeRequest`/`SmlspBridgeResult`: hover,
  definition, formatting, diagnostics) over stdio. This is a bespoke bridge
  invented for this app, not a real Language Server Protocol client and not
  backed by any canonical `smc`/`svm` subcommand — it does not exist in the
  canonical Workbench.
- **`scaffold.rs`** (`scaffold_project`, Tauri command
  `scaffold_semantic_project` at `lib.rs:92`): generates starter `.sm`
  project files directly from Rust string templates embedded in this app,
  bypassing any canonical project-generation path.

Both are experimental, non-canonical, and were never adopted by any other
part of the repository. The canonical Workbench
(`examples/workbench_semantic`) gets equivalent capability without either
bridge: diagnostics come from real `smc 7hell --json` output
(`examples/workbench_semantic/src/diagnostics.rs`), and all process
execution — including any future scaffold/format jobs — is routed through
the capability-gated `HostCapabilities::check_spawn` boundary
(`examples/workbench_semantic/src/host_capabilities.rs`), not a bespoke
bridge. Do not port `lsp_bridge.rs` or `scaffold.rs` forward; if equivalent
capability is needed, implement it as a real job kind through the existing
capability-gated job queue.

## Scope Guard

The first implementation waves must continue to respect the repository rule that
Workbench talks to Semantic through:

- `smc`
- `svm`
- `cargo`
- public release scripts

Direct private crate coupling is out of scope.
