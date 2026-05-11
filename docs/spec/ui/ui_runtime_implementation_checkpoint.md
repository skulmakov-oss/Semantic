# UI Runtime Implementation Checkpoint

Status: Draft
Track: POST-UI
Scope: checkpoint only
Implementation: out of scope

Related:

- `README.md`
- `ui_runtime_adapter_boundary.md`
- `host_runtime_effect_path_boundary.md`

## 1. Purpose

This document records the implementation boundary reached after PR-UI-I50
through PR-UI-I63.

It is a checkpoint, not a feature specification and not an implementation
claim.

## 2. Completed specification stack

### I50 - Host runtime effect path boundary

- defines the host/runtime UI effect path;
- freezes the route from VM effect request to UI runtime and adapter execution.

### I51 - UI effect envelope v0

- defines the UiEffectEnvelope v0 shape;
- defines the envelope result model and forbidden payload surface.

### I52 - UI capability taxonomy

- defines the UI capability set for admitted UI-visible host effects;
- defines effect-to-capability mapping, direction, scope, budget, and audit
  slots.

### I53 - Deterministic UI event envelope model

- defines normalized UI event envelopes and batches;
- freezes the explicit boundary for host input normalization.

### I54 - UI frame lifecycle contract

- defines the legal frame protocol;
- freezes BeginFrame, SubmitDrawCommands, EndFrame ordering and invariants.

### I55 - Minimal draw command batch contract

- defines the minimal bounded draw batch shape;
- freezes the declarative draw command surface.

### I56 - UI runtime adapter boundary

- defines the boundary between `prom-ui-runtime` and platform adapters;
- separates normalized runtime semantics from OS-specific execution.

## 3. Completed local runtime skeleton stack

### I57 - Adapter boundary skeleton

- introduces the local adapter boundary seam;
- defines logical IDs, request/result types, and recording adapter shape.

### I58 - Adapter boundary negative tests

- adds boundary hardening tests for logical IDs, result separation, and
  request recording behavior.

### I59 - Admission facade skeleton

- introduces a local admission facade;
- validates only target shape before mapping to the adapter boundary.

### I60 - Admission facade shape matrix tests

- freezes the effect-to-target validation matrix;
- ensures invalid shapes do not reach the adapter.

### I61 - Recording adapter facade smoke path

- proves the local facade and recording adapter path works end-to-end;
- keeps the path local and deterministic.

### I62 - Facade result helper methods

- adds helper methods for `UiAdmissionResult` and `UiAdmissionReject`;
- improves readability without changing admission semantics.

### I63 - Facade request builder helpers

- adds named builders for valid local request shapes;
- reduces manual target construction noise in tests.

## 4. Current implemented boundary

The current implementation is local-only and deterministic.

It includes:

- logical runtime IDs;
- local `UiRuntimeEffect` enum;
- `UiAdapterRequest` and `UiAdapterResult` seam;
- `RecordingAdapter`;
- `UiAdmissionFacade`;
- target-shape validation;
- request builders;
- result helpers;
- negative, matrix, smoke, and helper tests.

## 5. Explicit non-goals

Not implemented:

- real capability enforcement;
- budget accounting;
- audit persistence;
- ABI host-call surface;
- VM integration;
- native platform adapter;
- OS event loop;
- native window creation;
- renderer;
- draw command binary encoding;
- GPU/shader pipeline;
- widget/layout framework.

## 6. Boundary invariants

- `WindowId`, `FrameId`, and `DrawBatchId` are logical IDs, not capabilities.
- `UiRuntimeEffect` is local, not an ABI opcode.
- `RecordingAdapter` is not a backend.
- `UiAdmissionFacade` is not security admission.
- facade rejection is distinct from adapter rejection or failure.
- invalid shapes do not reach the adapter.
- valid requests preserve `request_id`, `effect_id`, and `target`.

## 7. Next allowed steps

Allowed next:

- local helper methods;
- local boundary tests;
- local docs and invariants;
- recording adapter hardening;
- local error mapping helpers.

Not yet allowed:

- native backend;
- renderer;
- event loop;
- prom-cap enforcement;
- ABI widening;
- VM integration.

## 8. Suggested next PR

PR-UI-I65 - prom-ui-runtime: add admission facade module-level invariants

or

PR-UI-I65 - prom-ui-runtime: add local error mapping helpers

## 9. Acceptance checklist

- checkpoint document exists;
- README links it;
- spec index links it;
- completed I50-I63 stack is listed;
- current implemented local boundary is listed;
- explicit non-goals are listed;
- next allowed and disallowed steps are listed;
- docs-only;
- no code changes.
