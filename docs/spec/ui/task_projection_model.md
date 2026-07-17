# Long-Running Task Projection Model

Status: HISTORICAL / DIRECTIONAL ONLY
Note: This document provides directional context. The normative specification is now [task_projection_v0.md](file:///C:/Users/said3/Desktop/EXOcode/Semantic_ui_dna2_3b_artifact_v1/docs/spec/ui/task_projection_v0.md).
Track: POST-UI / Intent-Driven Projection
Scope type: documentation only
Depends on:
- docs/dna/SEMANTIC_UI_DNA.md
- docs/dna/SEMANTIC_UI_DNA_v2.md
- docs/roadmap/post_ui/intent_driven_projection_roadmap.md
- docs/spec/ui/projection_source_model.md
- docs/spec/ui/ui_ir_schema.md
- docs/spec/ui/action_ir_routing.md
- docs/spec/ui/projection_patch_model.md
- docs/spec/ui/denial_recovery_projection.md
Related:
- #1310
- #1328
- #1329
- #1330
- #1331
- #1332
- #1333
- #1334
- #1335

Long-running task projection defines how task state is shown to users and operators.

It does not define Semantic truth, task engine behavior, VM behavior, runtime authority, admission policy, recovery implementation, renderer behavior, host effects, or production UI wiring.

This document does not implement TaskRecord, TaskStatePatch, task execution, runtime queues, shell behavior, Rust types, compiler behavior, or renderer backend behavior.

## 1. Purpose

This spec exists to prevent the bad workflow:

```text
long action starts -> UI blocks button and guesses progress
```

The intended flow is:

```text
ActionIntent is admitted.
Semantic creates or updates TaskRecord.
Projection receives TaskStatePatch / evidence.
UI shows task phase, progress, controls, locks, and recovery.
Shell renders projection.
UI does not own the task.
```

```text
Task lives in Semantic.
Progress lives in Projection.
Pixels live in Shell.
```

Long-running task projection keeps operator-visible progress honest without turning the UI into the task engine.

## 2. Non-Authority Rule

Task projection must not own or redefine:

- Semantic truth;
- verifier admission;
- task engine behavior;
- runtime scheduling;
- capability policy;
- recovery policy;
- VM behavior;
- host effects;
- renderer lifecycle.

```text
Projection reports task state.
Semantic owns task state.
Shell displays task state.
```

Task projection is a projection contract, not a task authority layer.

## 3. TaskRecord

`TaskRecord` is the Semantic-owned record for a long-running task.

It should cover:

- task id;
- originating `ActionIntent` ref;
- originating `ActionOffer` ref;
- actor / session / client refs;
- task kind;
- task state;
- current phase;
- progress;
- scope locks;
- allowed controls;
- recovery options;
- evidence timeline ref;
- created / updated / completed timestamps;
- source revisions;
- privacy / redaction note.

```text
TaskRecord is Semantic-owned.
Projection observes it.
```

TaskRecord is the source-side fact that projection reflects, not a UI-local task object.

## 4. Task States

Required task states include:

- `Pending`
- `Started`
- `Running`
- `AwaitingInput`
- `Paused`
- `Completing`
- `Completed`
- `Failed`
- `Denied`
- `Quarantined`
- `Cancelled`
- `PendingUnknown`

Required distinctions:

- `Failed` is not `Denied`;
- `Denied` is not `Cancelled`;
- `Quarantined` is not generic failure;
- `PendingUnknown` is not `Running`;
- `Completed` requires authority evidence.

```text
Failed != Denied != Quarantined.
```

Task state must preserve whether the task is waiting, executing, blocked, refused, completed, or uncertain.

## 5. TaskStatePatch

`TaskStatePatch` updates the projection view of a task.

It should cover:

- patch id;
- task id;
- task state;
- previous task revision;
- new task revision;
- phase;
- progress;
- allowed controls;
- scope locks;
- evidence ref;
- causal ref;
- actor / session / client attribution if relevant;
- projection target refs;
- diagnostics.

```text
TaskStatePatch updates task projection.
It does not execute the task.
```

TaskStatePatch is a deterministic projection update, not a runtime command.

## 6. Phases

Task phases describe operator-readable task progress structure.

Each phase should cover:

- phase id;
- phase label;
- phase order;
- phase status;
- phase evidence;
- phase diagnostics;
- optional progress contribution.

```text
Phase is operator-readable progress structure, not task implementation.
```

Phases make multi-step work understandable without exposing the engine internals as UI policy.

## 7. Progress

Progress projection may be determinate or indeterminate.

It should cover:

- determinate progress;
- indeterminate progress;
- progress units;
- progress confidence;
- progress source;
- coalescing;
- evidence checkpoints;
- stale progress handling.

```text
Progress must not claim more certainty than the task authority provides.
```

Progress is a projection of task advancement, not a promise from the UI.

## 8. AwaitingInput

`AwaitingInput` means the task requires structured input before it can continue.

It should cover:

- required input description;
- input target / outlet;
- allowed actors / sessions;
- timeout or expiration if any;
- evidence ref;
- allowed controls from `ActionOffers`;
- denial route for invalid input;
- recovery route if available.

```text
AwaitingInput requests structured input.
It does not grant arbitrary UI authority.
```

`AwaitingInput` is a task phase, not a UI loophole for freeform action.

## 9. Allowed Controls / ActionOffers

Task controls are projected from authority via `ActionOffers`.

Examples include:

- pause;
- resume;
- cancel;
- retry;
- acknowledge;
- provide input.

Rules:

- controls may be actor / session / client relative;
- controls require capability / freshness;
- `GuardedAction` and `DangerAction` restrictions still apply;
- controls must not be invented by UI.

```text
Task controls are offered by authority.
UI does not invent them.
```

Task controls are projections of admissible affordances, not local widget behavior.

## 10. Scope Locks

Task scope locks describe what the task has reserved or constrained.

They should cover:

- locked semantic scope;
- locked projection surface;
- locked action slots;
- read-only projection regions;
- viewer-relative lock display;
- lock owner / task ref;
- lock evidence;
- lock release condition.

```text
Tasks lock declared scopes, not the entire UI by default.
```

Scope locks keep concurrent work visible without freezing unrelated surfaces.

## 11. Recovery Options

Task recovery is projected from authority, not improvised by UI.

Recovery options may include:

- retry;
- resume;
- cancel;
- acknowledge;
- quarantine recovery if authority provides it;
- `ResumeToken` if resume is allowed;
- recovery evidence.

```text
Recovery is projected, but never improvised.
```

If recovery is available, the projection should show why and under what contract.

## 12. Task Evidence Timeline

The task evidence timeline records observable task lifecycle events.

It should cover:

- task created;
- phase changed;
- progress changed;
- awaiting input;
- control offered;
- control invoked;
- denial;
- failure;
- quarantine;
- completion;
- cancellation;
- recovery attempt.

```text
Task projection must be inspectable after the fact.
```

The evidence timeline makes task progression auditable as a projection surface.

## 13. Failure, Denial, and Quarantine

These outcomes must remain distinct.

Required meaning:

- failure means task execution failed after it was admitted;
- denial means a proposal or task control was refused;
- quarantine means a guarded authority state requiring explicit evidence;
- UI must not flatten these into generic failure.

```text
Failed != Denied != Quarantined.
```

Different outcomes require different operator explanations and recovery routes.

## 14. PendingUnknown

`PendingUnknown` is used when the task result is not yet known.

It should cover:

- task result unknown after connection loss;
- broken causal chain;
- unknown admission / control result;
- resync required;
- no success claim;
- no failure claim unless later evidence arrives;
- controls restricted until freshness restored.

```text
PendingUnknown must not be rendered as success or failure.
```

PendingUnknown is uncertainty, not success by omission.

## 15. Freshness and Task Controls

Freshness affects which controls are visible and actionable.

Freshness states include:

- `Fresh`;
- `Degraded`;
- `Stale`;
- `Offline`;
- `Resyncing`.

Rules:

- no offline queue for critical task controls;
- stale / offline disables `GuardedAction` / `DangerAction` / `TaskControl`;
- stale critical task control attempts route through `LocalDenied`.

```text
No freshness, no control.
```

Freshness is a control prerequisite, not a decorative status chip.

## 16. Multi-Client Task Projection

The same task can be projected differently per viewer.

Required meaning:

- same task visible to many clients;
- controls may differ by actor / session / client;
- task state is global / authority-owned;
- projection is viewer-relative;
- attribution may be redacted;
- capability determines available controls.

```text
One task.
Many views.
No shared illusion.
```

Multi-client projection keeps authority global while keeping projections viewer-relative.

## 17. Accessibility and Operator Readability

Task projection accessibility is part of the contract.

It should include:

- task label;
- state label;
- phase label;
- progress label;
- control labels;
- lock explanation;
- evidence route;
- recovery route;
- non-visual interpretation;
- criticality.

```text
Task state must be operator-readable, not only visually styled.
```

Accessibility makes task projection legible across visual and non-visual surfaces.

## 18. Diagnostics

Task projection diagnostics should include:

- missing `TaskPanel`;
- missing task id;
- missing evidence ref;
- unknown task state;
- invalid phase transition;
- progress regression without evidence;
- missing allowed controls;
- UI-invented task control;
- `Resume` without `ResumeToken`;
- stale critical task control shown as available;
- missing scope lock explanation;
- `PendingUnknown` rendered as success / failure.

```text
Diagnostics are evidence, not silent UI guesses.
```

Diagnostics are part of projection accountability, not a fallback for missing design.

## 19. Non-Normative Sketch

Non-normative sketch — not final serialization

```text
task_record {
  task_id: "task-042"
  kind: "projection.compile"
  originating_intent: "intent-101"
  state: Running
  phase: "emit-ui-ir"
  progress: { kind: determinate, value: 65, unit: percent }
  scope_locks: ["projection.CalculatorView"]
  allowed_controls: [
    ActionOffers.task.cancel
  ]
  evidence_timeline: "evidence.task-042"
}

patch TaskStatePatch {
  patch_id: "patch-task-007"
  task_id: "task-042"
  previous_task_rev: 8
  task_rev: 9
  state: AwaitingInput
  phase: "resolve-role-dictionary"
  awaiting_input: "select compatible role dictionary"
  allowed_controls: [
    ActionOffers.task.provide_input,
    ActionOffers.task.cancel
  ]
  evidence_ref: "evidence-950"
}
```

This sketch only illustrates task projection shape.
It is not final grammar, not implementation, and not a runtime contract.

## 20. Acceptance Criteria

The spec is acceptable when:

- it defines `TaskRecord`;
- it defines task state taxonomy;
- it defines `TaskStatePatch`;
- it defines phases;
- it defines progress projection;
- it defines `AwaitingInput`;
- it defines allowed controls via `ActionOffers`;
- it defines scope locks;
- it defines recovery options;
- it defines task evidence timeline;
- it distinguishes failure, denial, quarantine, and pending-unknown;
- it defines freshness impact on task controls;
- it defines multi-client task projection;
- it defines accessibility / operator readability;
- it defines diagnostics;
- it preserves Semantic authority;
- it includes a non-normative sketch only;
- it does not implement task execution;
- it does not implement task projection runtime;
- it does not claim production readiness.

