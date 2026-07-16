# UI DNA v2 Task Projection v0 Normative Specification

Status: accepted
Track: UI DNA v2
Implements: Task Projection Model (from `docs/spec/ui/task_projection_model.md`)

This normative specification defines the explicit, deterministic contract for UI DNA v2 Task Projection v0. It implements the Task Projection Model by providing the concrete taxonomy, validation stages, and diagnostics for projecting a Semantic-owned `TaskRecord` to inert `ProjectionPatch` mutations.

This specification remains subject to the Non-Authority Rule: it does not grant runtime authority, shell mutation capability, or execute the underlying task.

## 1. Task Projection Taxonomy

The task projection state taxonomy must exactly match the following 12 mutually exclusive states:

- `Pending`: Task is admitted but execution has not started.
- `Started`: Task execution has begun but meaningful progress is not yet reportable.
- `Running`: Task is actively executing and advancing.
- `AwaitingInput`: Task execution is paused waiting for mandatory structured input.
- `Paused`: Task execution is paused without awaiting input (e.g., via operator control).
- `Completing`: Task is finalizing and assembling completion evidence.
- `Completed`: Task finished successfully with authority evidence.
- `Failed`: Task execution resulted in an error (requires diagnosis, not quarantine).
- `Denied`: A task proposal or control invocation was refused by admission or policy.
- `Quarantined`: Task is blocked by a guarded authority state requiring explicit resolution.
- `Cancelled`: Task was explicitly aborted before completion.
- `PendingUnknown`: Task result is uncertain due to connection loss or broken causal chain.

## 2. Validation Precedence and Stages

Task projection operates as a pure, deterministic validation pipeline. The validation stages evaluate evidence and enforce explicit invariants. The stages must be applied in the following precedence order:

1. `ResourcePreflight`: Asserts strict bounded limits on projected collection counts and payload bytes to prevent unbounded allocations.
2. `RouteValidation`: Validates presence of required binding targets and collections (`phase_collection`, `control_collection`, `scope_lock_collection`, and conditionally `awaiting_input_route`).
3. `IdentityRevisionValidation`: Asserts valid Semantic references and strictly increasing revision counters (`new_revision > previous_revision`).
4. `StateValidation`: Asserts invariants specific to the current task state (e.g., `AwaitingInput` demands input description; `Completed` cannot project active phases).
5. `PhaseValidation`: Validates structural integrity and monotonic ordering of active and completed phases.
6. `ProgressValidation`: Prevents unevidenced regression in determinate progress (e.g., progress value cannot decrease between revisions unless explicitly regressed).
7. `FreshnessValidation`: Projects `FreshnessState` to conditionally disable or limit projection if evidence is stale or offline.
8. `ControlValidation`: Validates operator task controls (`ActionOffers`) and enforces freshness restrictions (e.g., `StaleControlOffer`).
9. `ScopeLockValidation`: Validates locking constraints, ensuring explanations and semantic references are well-formed.
10. `PatchValidation`: The final phase wrapping generated `ProjectionPatchOperation` vectors into an inert `ProjectionPatchEnvelope` for shell application.

## 3. Normative Diagnostics

Failure to satisfy any validation stage produces an exact coordinate via `TaskProjectionDiagnosticKind`. Task projection must not silently recover from these errors.

The 26 authorized diagnostic codes are:

- `TPP_RESOURCE_LIMIT_EXCEEDED`
- `TPP_MISSING_PHASE_ROUTE`
- `TPP_MISSING_CONTROL_ROUTE`
- `TPP_MISSING_LOCK_ROUTE`
- `TPP_MISSING_AWAITING_INPUT_ROUTE`
- `TPP_MISSING_TASK_IDENTITY`
- `TPP_NON_INCREASING_REVISION`
- `TPP_MISSING_AWAITING_INPUT_EVIDENCE`
- `TPP_UNEXPECTED_AWAITING_INPUT_EVIDENCE`
- `TPP_INVALID_STATE_DETAIL`
- `TPP_DUPLICATE_PHASE_ORDER`
- `TPP_DUPLICATE_PHASE_ID`
- `TPP_PROGRESS_REGRESSION_WITHOUT_EVIDENCE`
- `TPP_INVALID_PROGRESS_BOUNDS`
- `TPP_MISSING_FRESHNESS_EVIDENCE`
- `TPP_STALE_CONTROL_OFFER`
- `TPP_DUPLICATE_CONTROL_ORDER`
- `TPP_MISSING_CONTROL_ACTION`
- `TPP_INVALID_CONTROL_KIND`
- `TPP_EMPTY_LOCK_EXPLANATION`
- `TPP_DUPLICATE_LOCK_ORDER`
- `TPP_MISSING_LOCK_REFERENCE`
- `TPP_INVALID_LOCK_TARGET`
- `TPP_PROJECTION_PATCH_ERROR`
- `TPP_INVALID_ROUTE_TARGET`
- `TPP_UNEXPECTED_RESUME_TOKEN`

## 4. Contract Output

Upon successful validation, task projection outputs a collection of `ProjectionPatchOperation` instances. These describe the explicit structural and value modifications required to reflect task evidence in the Canonical Static UI IR. They do not trigger shell layout, rendering, or dispatch execution directly.
