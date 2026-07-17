# UI DNA v2 Task Projection v0

Status: candidate in PR #1517
Track: UI DNA v2
Authority: crate-private presentation projection only

This specification defines the bounded Task Projection v0 contract for projecting caller-supplied task evidence into a canonical validated representation and exactly one inert `ProjectionPatch`.

It does not read Semantic state, own task truth, admit or execute controls, apply patches, mutate shell state, dispatch runtime work, open Gate D, or authorize production promotion.

## State taxonomy

The state taxonomy is exactly:

```text
Pending
Started
Running
AwaitingInput
Paused
Completing
Completed
Failed
Denied
Quarantined
Cancelled
PendingUnknown
```

Stable tokens are:

```text
pending
started
running
awaiting_input
paused
completing
completed
failed
denied
quarantined
cancelled
pending_unknown
```

Task evidence is required only for `Completed`, `Failed`, `Denied`, `Quarantined`, `Cancelled`, and `PendingUnknown`. A zero-valued evidence reference does not satisfy the requirement.

## Phases

A phase contains a nonzero id, collection key, order, label, and one of:

```text
Pending
Active
Completed
Blocked
Failed
```

Phase order and collection key are unique. `Started`, `Running`, `AwaitingInput`, `Paused`, and `Completing` require exactly one active phase. All other states forbid active phases.

## Progress

Progress is exactly:

```text
Indeterminate
Determinate { completed, total }
```

For determinate progress, `total > 0` and `completed <= total`. `Completed` requires determinate full progress. A regression requires a present nonzero evidence reference.

## Controls

Control kinds are exactly:

```text
Pause
Resume
Cancel
Retry
Acknowledge
ProvideInput
```

Each control carries order, collection key, a nonzero `SemanticActionRef`, and an optional existing `ReferenceToken`. `Resume` requires a token. Controls are allowed under `Fresh` and `Degraded`; `Stale`, `Offline`, and `Resyncing` require an empty control collection.

A projected control retains its action reference and token data. A control offer is presentation evidence, not execution authority.

## Scope locks

A scope-lock item carries order, collection key, opaque `ReferenceToken`, and a nonempty explanation. Order and collection key are unique. Projection of a lock does not create a runtime lock.

## AwaitingInput route

`AwaitingInput` requires nonempty input text and an AwaitingInput route. Other states forbid AwaitingInput text. When a non-AwaitingInput projection has no AwaitingInput route, projection succeeds and emits no AwaitingInput operation.

## Validation stages

Validation stage precedence is exactly:

```text
1. ResourcePreflight
2. RouteValidation
3. IdentityRevisionValidation
4. StateValidation
5. PhaseValidation
6. ProgressValidation
7. ControlValidation
8. ScopeLockValidation
9. OperationConstruction
10. PatchValidation
```

Freshness validation remains owned by the existing CFP diagnostic domain; it is not a Task Projection-owned stage.

## TPP diagnostics

The Task Projection taxonomy contains exactly:

```text
TPP_RESOURCE_LIMIT_EXCEEDED
TPP_MISSING_TASK_REF
TPP_NON_INCREASING_TASK_REVISION
TPP_MISSING_EVIDENCE_REF
TPP_INVALID_STATE_DETAIL
TPP_MISSING_AWAITING_INPUT
TPP_UNEXPECTED_AWAITING_INPUT
TPP_DUPLICATE_PHASE_ORDER
TPP_DUPLICATE_PHASE_KEY
TPP_INVALID_PHASE_SET
TPP_INVALID_PROGRESS
TPP_PROGRESS_REGRESSION_WITHOUT_EVIDENCE
TPP_DUPLICATE_CONTROL_ORDER
TPP_DUPLICATE_CONTROL_KEY
TPP_CONTROL_ACTION_REF_MISSING
TPP_RESUME_TOKEN_MISSING
TPP_STALE_CONTROL_OFFER
TPP_DUPLICATE_LOCK_ORDER
TPP_DUPLICATE_LOCK_KEY
TPP_EMPTY_LOCK_EXPLANATION
TPP_MISSING_PHASE_ROUTE
TPP_MISSING_CONTROL_ROUTE
TPP_MISSING_LOCK_ROUTE
TPP_MISSING_AWAITING_INPUT_ROUTE
TPP_OPERATION_LIMIT_EXCEEDED
TPP_PATCH_REJECTED
```

`TPP_PATCH_REJECTED` is reserved. A downstream ProjectionPatch validator failure is returned as exact `ProjectionPatchDiagnostics`, not translated into TPP.

## Error ownership

```text
Task Projection-owned failure
    -> TaskProjectionError::Task(exact earliest-stage TPP diagnostic)

Freshness projector failure
    -> TaskProjectionError::Freshness(exact CFP diagnostic vector)

ProjectionPatch validator failure
    -> TaskProjectionError::Patch(exact ProjectionPatchDiagnostics)
```

## Limits

All limits are caller-supplied. There is no hidden default. Limits cover phase count, control count, scope-lock count, phase-label bytes, AwaitingInput bytes, lock-explanation bytes, total projected text bytes, total operations, and nested freshness limits.

Aggregate accounting uses checked arithmetic. Operation count is determined and rejected before the proportional Task Projection operation vector is constructed.

## Canonical output

Success returns:

1. a canonical validated Task Projection representation retaining task identity, revisions, state, canonical phases, progress, evidence references, freshness, canonical controls, and canonical scope locks;
2. exactly one populated inert `ProjectionPatch`.

Canonical collection order is:

```text
phases: order, key, id
controls: order, key
scope locks: order, key
```

Input permutation must not change the canonical representation or patch operation order.

## Non-authority boundary

```text
TaskRecordRef != task truth
task evidence != admission
task control offer != execution
ReferenceToken != authority
scope-lock projection != runtime locking
freshness carrier != connection truth
ProjectionPatch construction != patch application
projector success != UI mutation
implementation != public API
implementation != runtime integration
Gate D = CLOSED
production promotion = NOT AUTHORIZED
```

This slice does not authorize a follow-on implementation slice. Any follow-on work requires a separate task authorization.
