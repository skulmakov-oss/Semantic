# Local Runtime Skeleton API Map

Status: Draft
Track: POST-UI
Scope: checkpoint only
Owner: prom-ui-runtime
Implementation: out of scope

Related:

- `README.md`
- `ui_runtime_implementation_checkpoint.md`
- `ui_runtime_adapter_boundary.md`
- `host_runtime_effect_path_boundary.md`

## 1. Purpose

This document defines the intended local UI runtime skeleton API map for
Semantic Workbench and `prom-ui-runtime`.

The local UI runtime is an orchestration shell for application/session UX. It
coordinates projects, commands, diagnostics, traces, and packaging views, but
it does not own compiler semantics, VM semantics, or PROMETHEUS runtime
semantics.

## 2. Position in architecture

```text
Workbench UI
  -> Local UI Runtime API
  -> Semantic toolchain / verifier / VM / package-runtime bridge
  -> Optional later PROMETHEUS runtime integration
```

The local UI runtime may orchestrate.
It must not own semantics.

## 3. Non-goals

This document does not define an implementation.

Not included:

- Rust code;
- `prom-ui-runtime` runtime logic;
- Tauri integration;
- actual command execution;
- packaging implementation;
- UI widgets;
- async runtime design;
- VM changes;
- verifier changes;
- PROMETHEUS runtime widening.

## 4. Ownership rules

| Entity | Owner | UI runtime may | UI runtime must not |
| --- | --- | --- | --- |
| Source files | project / filesystem layer | read and pass through | own semantics |
| AST | `sm-front` | display via facade | build or mutate it |
| Type diagnostics | `sm-sema` | show results | generate semantic truth |
| IR | `sm-ir` | request dumps | change it |
| SemCode | `sm-emit` | hold artifact refs | patch bytes |
| Verification | `sm-verify` | invoke and display results | bypass it |
| VM execution | `sm-vm` | run through verified path | execute raw/unverified |
| PROMETHEUS runtime | `prom-runtime` | future integration only | replace or widen it |
| UI session | `prom-ui-runtime` | own local session state | leak compiler internals |

## 5. API groups

The local UI runtime API is grouped into these surfaces:

- session API;
- project API;
- toolchain command API;
- diagnostics API;
- execution API;
- trace/audit viewing API;
- package API;
- local storage API.

## 6. Session lifecycle

Minimal flow:

```text
start Workbench
  -> open UI session
  -> load project
  -> build snapshot
  -> run commands
  -> publish UI events
  -> render diagnostics/results/traces
  -> close session
```

Session rules:

- `open_session(project_root)` opens a local UI session;
- `close_session(session_id)` closes the session;
- `session_status(session_id)` reports current state;
- `runtime_capabilities()` reports available local functions.

Session API is not an execution context for the Semantic VM.

## 7. Project API

Purpose: local project inspection and snapshotting.

Representative calls:

- `load_project(path)`
- `reload_project()`
- `list_sources()`
- `read_source(file_id)`
- `project_snapshot()`

Rules:

- the project API may read files;
- it must not parse Semantic as the source of truth;
- it must pass source through toolchain facades for real analysis.

## 8. Toolchain command API

Purpose: one local layer for command-like operations.

Representative commands:

- `check`
- `lint`
- `compile`
- `verify`
- `run`
- `run-smc`
- `disasm`
- `dump-ast`
- `dump-ir`
- `dump-bytecode`
- `format`

Rules:

- `check` and `lint` may use `smc-cli` facade or published toolchain entrypoints;
- `compile` may use published `sm-emit` surface;
- `verify` must go through `sm-verify`;
- `run` must go through verified execution path;
- `disasm` must use published disassembly surface;
- the local runtime must not depend on private `sm-front`, `sm-ir`, or `sm-vm` modules.

## 9. Diagnostics API

Purpose: unified UI-facing error and diagnostic display.

Representative calls:

- `latest_diagnostics(file_id)`
- `diagnostics_for_command(command_id)`
- `explain_diagnostic(code)`
- `diagnostic_locations()`

Rules:

- diagnostics may be formatted locally;
- diagnostics must come from toolchain or verifier surfaces;
- the local runtime must not invent compiler diagnostics as semantic truth.

## 10. Execution API

Purpose: launch only through verified or admitted paths.

Representative calls:

- `run_source(file)`
- `run_semcode(artifact)`
- `stop_execution(id)`
- `execution_metrics(id)`

Rules:

- raw VM execution from UI runtime is forbidden;
- execution must pass verifier admission;
- raw or unverified code must not be executed from the local runtime.

## 11. Trace / audit viewing API

Purpose: view trace and audit-compatible data, not own it.

Representative calls:

- `trace_for_execution(id)`
- `audit_view(id)`
- `timeline(id)`
- `export_trace(id)`

Rules:

- the local runtime may render trace and audit views;
- it does not own `prom-audit`;
- it only reads or displays audit-compatible data.

## 12. Package API

Purpose: future package planning and target selection.

Representative calls:

- `package_plan()`
- `validate_package_manifest()`
- `build_package()`
- `list_targets()`

Rules:

- package flow is a skeleton map only;
- packaging implementation is out of scope for this document.

## 13. Local storage API

Purpose: local UI state such as settings and recent projects.

Representative calls:

- `load_preferences()`
- `save_preferences()`
- `recent_projects()`
- `clear_local_cache()`

Rules:

- local store remains local;
- no telemetry;
- no hidden upload;
- no silent state exfiltration.

## 14. Event model

Minimal UI events:

- `SessionOpened`
- `ProjectLoaded`
- `FileChanged`
- `CommandStarted`
- `CommandFinished`
- `DiagnosticsUpdated`
- `ExecutionStarted`
- `ExecutionFinished`
- `TraceUpdated`
- `PackagePlanUpdated`
- `RuntimeError`

Rules:

- UI runtime events are UI-facing state notifications;
- they are not Semantic language events;
- they are not PROMETHEUS Pulse events.

## 15. Error model

Minimal error taxonomy:

- `ProjectLoadFailed`
- `SourceReadFailed`
- `CommandFailed`
- `ToolchainUnavailable`
- `VerificationRejected`
- `ExecutionTrapped`
- `PackageUnsupported`
- `InvalidRuntimeState`
- `InternalUiRuntimeError`

Rules:

- errors are mapped to UI-facing messages and logs;
- verification rejection must remain visible as rejection;
- execution traps must remain visible with trace context when available.

## 16. Security and capability boundary

Rules:

- the local runtime must not bypass verifier admission;
- the local runtime must not own capability policy;
- the local runtime must not widen PROMETHEUS runtime semantics;
- the local runtime must not treat UI orchestration as authority over code execution.

## 17. Dependency rules

Forbidden dependencies:

- `prom-ui-runtime -> sm-front` private modules;
- `prom-ui-runtime -> sm-ir` private lowering internals;
- `prom-ui-runtime -> sm-vm` raw execution internals;
- `prom-ui-runtime -> prom-state` mutation internals;
- `prom-ui-runtime -> prom-rules` engine internals.

Allowed directions:

- `prom-ui-runtime -> smc-cli` public facade;
- `prom-ui-runtime -> published toolchain APIs`;
- `prom-ui-runtime -> serialized diagnostics/result models`;
- `prom-ui-runtime -> local project/package metadata`.

## 18. Future extension points

Possible future extensions:

- richer session management;
- package build orchestration;
- trace export formats;
- additional diagnostics rendering;
- later PROMETHEUS runtime integration.

Any future extension that starts to own compiler semantics, VM semantics,
capability enforcement, or audit/budget logic must be split into a separate PR
with explicit boundary review.

## 19. Acceptance checklist

- local runtime purpose is clear;
- UI runtime is separated from Semantic VM;
- UI runtime is separated from PROMETHEUS runtime;
- API groups are defined;
- ownership rules are defined;
- dependency rules are defined;
- non-goals are explicit;
- UI runtime must not bypass verifier admission;
- local store is local-only by default;
- no telemetry is implied;
- docs-only;
- no code changes.
