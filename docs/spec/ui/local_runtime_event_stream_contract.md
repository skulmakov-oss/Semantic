# Local Runtime Event Stream Contract

Status: Draft
Track: POST-UI
Layer: UI / Local Runtime Boundary
Owner: prom-ui-runtime
Depends on:
- local_runtime_skeleton_api_map.md
- local_runtime_command_result_envelope.md
Scope: event stream contract only
Implementation: out of scope

Related:

- `README.md`
- `local_runtime_skeleton_api_map.md`
- `local_runtime_command_result_envelope.md`

## 1. Purpose

This document defines the local runtime event stream for Workbench UI and the
local UI runtime skeleton.

The local runtime event stream is a UI-facing progress and state stream. It is
not PROMETHEUS Pulse, VM trace format, audit trail format, SemCode event
format, UI frame event model, or platform window/input event model.

## 2. Architecture position

```text
UiCommandRequest
  -> UiRuntimeEvent*
  -> UiCommandResult
```

The event stream reports what happened while a command or runtime session was
in flight. It does not define new execution semantics.

## 3. Relationship to I67/I68

- `I67` defines the local runtime API map.
- `I68` defines the local runtime command/result envelope.
- `I69` defines the ordered live event stream emitted around command and
  session processing.

`UiCommandResult` is the final summarized outcome.
`UiRuntimeEvent` is the ordered live stream emitted before, during, and after
command execution.

## 4. Non-goals

This document does not define:

- Rust structs;
- serialization format;
- Tauri event names;
- IPC protocol;
- async executor;
- event bus implementation;
- UI rendering behavior;
- platform input events;
- PROMETHEUS Pulse format;
- VM trace storage;
- audit trail format;
- telemetry pipeline.

## 5. Event stream model

Core formula:

```text
UiCommandRequest(request_id)
  -> UiRuntimeEvent* keyed by request_id
  -> UiCommandResult(request_id)
```

Correlation rules:

- every command-scoped event must carry `request_id`;
- if an event is session-scoped, it must carry `session_id`;
- if an event is project-scoped, it must carry `project_id`.

## 6. UiRuntimeEvent envelope

Representative shape:

```text
UiRuntimeEvent {
  event_id
  event_kind
  scope
  timestamp
  sequence
  request_id?
  session_id?
  project_id?
  payload
  severity
}
```

Field notes:

- `event_id` is a unique event identifier;
- `event_kind` is the event type;
- `scope` indicates session, project, command, or execution scope;
- `timestamp` is a local time marker;
- `sequence` is a monotonic number within the stream;
- `request_id` links command-scoped events to the command;
- `session_id` links session-scoped events to the session;
- `project_id` links project-scoped events to the project;
- `payload` is typed event content;
- `severity` is info, warning, or error.

## 7. Event scopes

Supported scopes:

- `Session`
- `Project`
- `Command`
- `Execution`

Rules:

- scope controls correlation requirements;
- scope does not change ownership of Semantic semantics;
- scope does not turn the stream into PROMETHEUS Pulse.

## 8. Event classes

### 8.1 Session events

- `SessionOpened`
- `SessionReady`
- `SessionClosing`
- `SessionClosed`
- `SessionError`

### 8.2 Project events

- `ProjectLoading`
- `ProjectLoaded`
- `ProjectReloaded`
- `ProjectSnapshotUpdated`
- `ProjectError`

### 8.3 Command lifecycle events

- `CommandQueued`
- `CommandStarted`
- `CommandProgress`
- `CommandFinished`
- `CommandCancelled`
- `CommandFailed`

### 8.4 Diagnostics events

- `DiagnosticsCleared`
- `DiagnosticsUpdated`
- `DiagnosticsPublished`

Diagnostics events transport diagnostics. They do not own diagnostic semantics.

### 8.5 Artifact events

- `ArtifactProduced`
- `ArtifactUpdated`
- `ArtifactInvalidated`

Typical artifacts:

- AST dump
- IR dump
- SemCode artifact
- disassembly
- trace report
- package plan

### 8.6 Execution events

- `ExecutionAdmitted`
- `ExecutionStarted`
- `ExecutionProgress`
- `ExecutionTrapped`
- `ExecutionFinished`

Rule:

`ExecutionStarted` must only occur after verifier admission.

### 8.7 Runtime health events

- `RuntimeReady`
- `RuntimeBusy`
- `RuntimeDegraded`
- `RuntimeRecoverableError`
- `RuntimeFatalError`

## 9. Ordering rules

Rules:

- events within one stream must be emitted in monotonic sequence order;
- for a single `request_id`, `CommandQueued` must precede `CommandStarted`;
- for a single `request_id`, `CommandStarted` must precede `CommandFinished`;
- `CommandFinished` must not be followed by `CommandProgress` for the same
  `request_id`;
- `CommandCancelled` must be terminal;
- `CommandFailed` must be terminal;
- `ExecutionStarted` must not occur before verification/admission success.

## 10. Terminal event rules

Terminal events:

- `CommandFinished`
- `CommandCancelled`
- `CommandFailed`
- `ExecutionTrapped`
- `RuntimeFatalError`

After a terminal command event, no more command-scoped progress events may be
emitted for the same `request_id`.

## 11. Payload categories

### 11.1 Progress payload

```text
ProgressPayload {
  phase
  message
  current?
  total?
}
```

Phases:

- `loading`
- `checking`
- `linting`
- `compiling`
- `verifying`
- `running`
- `disassembling`
- `formatting`
- `packaging`

### 11.2 Diagnostics payload

```text
DiagnosticsPayload {
  diagnostics
  source
  replaced_previous
}
```

### 11.3 Artifact payload

```text
ArtifactPayload {
  artifact_id
  artifact_kind
  producer_request_id
  path?
  content_type
  is_temporary
}
```

### 11.4 Execution payload

```text
ExecutionPayload {
  admission_status
  steps?
  quota_used?
  trap?
  exit_class?
}
```

## 12. Security rules

- the event stream must not expose raw private VM state;
- the event stream must not expose host secrets;
- the event stream must not bypass verifier admission;
- the event stream must not trigger effects by itself;
- the event stream is observational, not authoritative execution control;
- no telemetry;
- no hidden upload;
- local-only by default.

Events report what happened. Events do not cause Semantic execution effects.

## 13. Dependency and ownership rules

Rules:

- the event stream belongs to the local UI runtime boundary;
- it is separate from general UI frame/input event models;
- it does not own Semantic compiler or VM semantics;
- it does not own PROMETHEUS Pulse.

Forbidden dependencies:

- `prom-ui-runtime -> sm-front` private modules;
- `prom-ui-runtime -> sm-ir` private lowering internals;
- `prom-ui-runtime -> sm-vm` raw execution internals;
- `prom-ui-runtime -> prom-state` mutation internals;
- `prom-ui-runtime -> prom-rules` engine internals.

Allowed directions:

- `prom-ui-runtime -> local runtime command/result envelope`;
- `prom-ui-runtime -> serialized diagnostics/result models`;
- `prom-ui-runtime -> local project/package metadata`.

## 14. Acceptance checklist

- event stream purpose is clear;
- event stream is separated from command result;
- event envelope fields are defined;
- event classes are defined;
- ordering rules are defined;
- terminal event rules are defined;
- payload categories are defined;
- security rules are explicit;
- no verifier bypass is implied;
- local-only by default and no telemetry are explicit;
- not PROMETHEUS Pulse;
- not VM trace format;
- not audit trail format;
- docs-only;
- no code changes.
