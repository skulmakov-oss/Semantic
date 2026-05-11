# Local Runtime Command / Result Envelope

Status: Draft
Track: POST-UI
Layer: UI / Application Boundary
Owner: prom-ui-runtime
Depends on: local_runtime_skeleton_api_map.md
Scope: message envelope contract only
Implementation: out of scope

Related:

- `README.md`
- `local_runtime_skeleton_api_map.md`
- `ui_runtime_implementation_checkpoint.md`

## 1. Purpose

This document defines the command/result envelope between Workbench UI and the
local UI runtime skeleton.

The envelope is a UI-runtime transport contract. It is not Semantic IR,
SemCode, VM instruction format, verifier internals, PROMETHEUS Pulse, or audit
trail format.

## 2. Architecture position

```text
Workbench UI
  -> UiCommandRequest
  -> prom-ui-runtime
  -> toolchain / verifier / VM bridge
  -> UiCommandResult / UiRuntimeEvent / UiRuntimeError
  -> Workbench UI
```

The envelope is orchestration metadata, not new language semantics.

## 3. Relationship to the local runtime API map

`local_runtime_skeleton_api_map.md` defines the API groups and ownership
boundaries.

This document defines the request/result shapes that move across those API
groups.

## 4. Non-goals

This document does not define:

- Rust structs;
- serialization format;
- IPC protocol;
- Tauri command names;
- async executor design;
- persistent command database;
- cloud sync;
- telemetry;
- VM internals;
- SemCode binary format;
- verifier internals;
- PROMETHEUS Pulse event format;
- audit trail storage format.

## 5. Command lifecycle

```text
Created
  -> Queued
  -> Running
  -> Succeeded | Failed | Rejected | Trapped | Cancelled
```

Lifecycle events may emit diagnostics, artifacts, progress updates, and runtime
errors, but the envelope itself must remain structured.

## 6. UiCommandRequest

Purpose: a single request from UI to local runtime.

Representative shape:

```text
UiCommandRequest {
  request_id
  session_id
  project_id
  kind
  target
  options
  correlation
}
```

Rules:

- `request_id` must stay stable for the whole command lifecycle;
- `session_id` binds the request to the current UI runtime session;
- `project_id` binds project-scoped requests when relevant;
- `correlation` links UI events or editor actions to the command;
- the request must not become a hidden ABI or VM instruction format.

## 7. UiCommandKind

Minimal set:

- `Check`
- `Lint`
- `Compile`
- `Verify`
- `RunSource`
- `RunSemCode`
- `Disasm`
- `DumpAst`
- `DumpIr`
- `DumpBytecode`
- `FormatCheck`
- `FormatWrite`
- `PackagePlan`

Rules:

- `Check` and `Lint` are source/project inspection operations;
- `Compile` produces SemCode artifacts;
- `Verify` performs SemCode admission;
- `RunSource` must internally pass through compile and verify before run;
- `RunSemCode` must go through verified execution unless the artifact is
  already admitted in the same trusted session;
- `Disasm`, `DumpAst`, `DumpIr`, and `DumpBytecode` are views, not ownership
  transfers.

## 8. UiCommandTarget

Target kinds:

- `ProjectRoot`
- `SourceFile`
- `SemCodeArtifact`
- `PackageManifest`
- `VirtualBuffer`

Rules:

- `ProjectRoot` covers project-wide commands;
- `SourceFile` targets a specific source file;
- `SemCodeArtifact` targets a SemCode artifact;
- `PackageManifest` targets package planning;
- `VirtualBuffer` is UI/runtime input only and must not become canonical
  project state unless explicitly saved.

## 9. UiCommandResult

Purpose: the structured response from local runtime.

Representative shape:

```text
UiCommandResult {
  request_id
  status
  exit_class
  diagnostics
  artifacts
  metrics
  output
  error
}
```

Rules:

- `request_id` must match the request that produced the result;
- `diagnostics` carries structured diagnostic data;
- `artifacts` carries references, not ownership of compiler internals;
- `metrics` is local diagnostic data, not telemetry;
- `error` must be structured and not force the UI to parse human-readable
  strings.

## 10. UiCommandStatus

- `Queued`
- `Running`
- `Succeeded`
- `Failed`
- `Cancelled`
- `Rejected`
- `Trapped`

Status distinctions:

- `Failed` means the command could not complete technically;
- `Rejected` means verifier or admission rejected the requested path;
- `Trapped` means execution started and then stopped by trap;
- `Cancelled` means the user or runtime cancelled the command.

## 11. UiCommandExitClass

- `Clean`
- `DiagnosticsOnly`
- `VerificationReject`
- `RuntimeTrap`
- `ToolchainError`
- `RuntimeError`
- `UserCancelled`
- `InternalError`

This class exists so the UI does not infer semantics from stdout or stderr
strings alone.

## 12. UiDiagnosticEnvelope

Representative shape:

```text
UiDiagnosticEnvelope {
  source
  code
  severity
  message
  span
  help
  related
}
```

Rules:

- parser diagnostics belong to the frontend;
- type diagnostics belong to sema;
- verifier diagnostics belong to the verifier;
- runtime traps belong to VM/runtime;
- UI only transports and displays the diagnostics.

## 13. UiCommandArtifact

Artifact kinds:

- `AstDump`
- `IrDump`
- `SemCodeFile`
- `Disassembly`
- `TraceFile`
- `PackagePlan`
- `Report`

Representative fields:

- `artifact_id`
- `kind`
- `path`
- `content_type`
- `producer_command_id`
- `is_temporary`

Rules:

- artifacts are references, not ownership transfer of internal compiler objects;
- temporary artifacts may exist for view/export flows only.

## 14. UiCommandMetrics

Minimal metrics:

- `started_at`
- `finished_at`
- `duration_ms`
- `input_bytes`
- `output_bytes`
- `diagnostic_count`
- `artifact_count`

Execution metrics may also include:

- `vm_steps`
- `quota_used`
- `trap_count`
- `effect_call_count`

Rules:

- metrics are local diagnostic data;
- metrics are not telemetry.

## 15. UiRuntimeEvent

Event kinds:

- `CommandQueued`
- `CommandStarted`
- `CommandProgress`
- `DiagnosticsUpdated`
- `ArtifactProduced`
- `CommandFinished`
- `CommandCancelled`
- `RuntimeError`

Rules:

- runtime events are UI-facing state notifications;
- `UiRuntimeEvent.request_id` must correlate with `UiCommandRequest.request_id`;
- runtime events are not Semantic language events and not PROMETHEUS Pulse
  events.

## 16. UiRuntimeError

Minimal taxonomy:

- `InvalidRequest`
- `InvalidTarget`
- `ProjectUnavailable`
- `ToolchainUnavailable`
- `CommandUnsupported`
- `VerificationRejected`
- `ExecutionTrapped`
- `ArtifactUnavailable`
- `PermissionDenied`
- `InternalRuntimeError`

Rules:

- runtime errors must be structured;
- UI must not be forced to parse human-readable strings as the contract.

## 17. Security rules

- command envelopes must not bypass verifier admission;
- `RunSource` must internally pass through compile and verify;
- `RunSemCode` must pass through verify unless the artifact is already admitted
  in the same trusted session;
- no hidden network calls;
- no telemetry;
- no automatic upload of diagnostics, traces, source code, artifacts, or crash
  data;
- local paths must not be exposed outside the local runtime boundary.

## 18. Dependency and ownership rules

Rules:

- the local runtime does not own compiler semantics;
- the local runtime does not own VM semantics;
- the local runtime does not own PROMETHEUS runtime semantics;
- the local runtime may orchestrate, not redefine, the underlying layers.

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

## 19. Future extension points

Possible future extensions:

- richer command progress reporting;
- additional artifact kinds;
- more detailed trace export formats;
- package build orchestration;
- later PROMETHEUS integration.

Any extension that starts to own compiler semantics, VM semantics,
capability enforcement, or audit/budget logic must be split into a separate PR
with explicit boundary review.

## 20. Acceptance checklist

- envelope contract exists;
- local runtime API map is referenced;
- request and result shapes are defined;
- lifecycle statuses are defined;
- diagnostics envelope is defined;
- artifact references are defined;
- runtime events are defined;
- runtime errors are defined;
- security and dependency boundaries are explicit;
- local-only by default and no telemetry are explicit;
- docs-only;
- no code changes.
