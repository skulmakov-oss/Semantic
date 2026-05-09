# Semantic UI Error, Denial, and Quarantine Visual Boundary

Status: Draft
Track: POST-UI / H-series
Purpose: define error, denial, conflict, and quarantine visual boundaries before implementation

## 1. Goal

This document defines the visual and semantic boundary for errors, denials, failures, conflicts, and quarantine states in Semantic UI.

The project must distinguish:

```text
error != denial
denial != failure
failure != crash
quarantine != deletion
conflict != crash
visual refusal != hidden no-op
```

Denied, failed, conflicted, or quarantined states must be explicit, inspectable, and traceable when they affect user-visible behavior.

The UI must not hide refusal behind silence.

## 2. Relationship to trace/audit boundary

Error, denial, and quarantine visualization depends on the trace/audit visual boundary:

```text
docs/architecture/ui_trace_audit_visual_boundary.md
```

Trace/audit records may explain:

- what was requested;
- what was denied;
- what failed;
- what entered quarantine;
- what conflict was detected;
- what state was preserved;
- what action remains possible.

The visual boundary must preserve trace meaning.

## 3. Layer separation

| State | Meaning | Required visual behavior |
| --- | --- | --- |
| denial | requested operation was refused before execution | show reason and gate |
| error | invalid or exceptional condition | show source and recovery path |
| failure | attempted operation did not complete successfully | show failed stage |
| crash | uncontrolled process/runtime failure | show hard boundary |
| conflict | contradictory or incompatible semantic state | show conflict relation |
| quarantine | isolated state requiring inspection or safe handling | show isolation, not deletion |
| no-op | operation intentionally produced no state change | show if user expectation is affected |

This preserves:

```text
Denied is not failed.
Failed is not crashed.
Quarantined is not deleted.
No-op is not hidden refusal.
```

## 4. Denial definition

A denial means an operation was refused by an admission, capability, lifecycle, policy, or ownership gate.

Examples:

```text
capability denied
lifecycle invalid
target not selectable
effect commit not admitted
renderer not admitted
native backend not ready
policy refused
```

A denial must show:

1. requested operation;
2. denying gate;
3. denial reason;
4. required condition if known;
5. trace/audit relation if available;
6. possible recovery if available.

A denial must not be displayed as a generic error.

## 5. Error definition

An error means the system encountered an invalid condition or exceptional state.

Examples:

```text
invalid state
missing target
malformed input
runtime unavailable
renderer resource unavailable
internal invariant violation
```

An error must show:

- source layer;
- failed condition;
- severity;
- whether state was preserved;
- whether retry is possible;
- trace/audit relation if available.

An error must not be silently collapsed into denial.

## 6. Failure definition

A failure means an attempted operation did not complete successfully after it began.

Examples:

```text
effect prepare failed
effect commit failed
renderer presentation failed
native window creation failed
trace projection failed
```

A failure must show:

- operation attempted;
- stage where failure occurred;
- state before failure;
- state after failure if known;
- rollback/cancellation status if relevant;
- trace/audit relation.

Failure must not be shown as successful trace visibility.

## 7. Crash boundary

Crash is an uncontrolled process/runtime failure or fatal subsystem loss.

Crash is not ordinary denial or recoverable failure.

Crash visualization must be hard, explicit, and non-ambiguous.

Crash state must not be used for:

- denied admission;
- missing capability;
- validation error;
- expected refusal;
- unsupported feature;
- quarantined conflict.

Crash UI is out of scope for H10 implementation, but the semantic distinction is reserved.

## 8. Conflict definition

A conflict means two or more semantic states, claims, capabilities, or transitions are incompatible.

Examples:

```text
state conflict
capability conflict
trace conflict
effect conflict
selection conflict
renderer transcript conflict
```

Conflict must show:

- conflicting parties;
- conflict type;
- affected target;
- whether operation is blocked;
- whether quarantine occurred;
- trace/audit relation.

Conflict is not crash.

Conflict is a semantic condition requiring inspection or resolution.

## 9. Quarantine definition

Quarantine means a state, target, effect, trace, or object is isolated for safe handling.

Quarantine must not imply deletion.

Quarantine may mean:

```text
do not mutate automatically
do not commit effect
require inspection
preserve original state
restrict action set
route to recovery flow
```

Quarantine visualization must show:

- quarantined target;
- quarantine reason;
- allowed actions;
- forbidden actions;
- trace/audit relation;
- whether original state is preserved.

## 10. No-op and visual refusal

A no-op is valid only if the system intentionally performs no state change.

But a no-op must not hide denial.

Example:

```text
request unsupported
  -> denial visible

request admitted but no state change needed
  -> explicit no-op if user expectation is affected
```

Hidden no-op is forbidden when the user expects an action.

## 11. Visual grammar requirements

Error/denial/quarantine visual grammar must distinguish at least:

```text
denied
invalid
failed
conflicted
quarantined
crashed
no_effect
```

These states must not collapse into one generic red state.

Visual representation must not rely on color alone.

It should use:

- label;
- boundary;
- icon/symbol if admitted later;
- trace link;
- denial/failure reason;
- recovery/action affordance if admitted.

## 12. Trace and audit relationship

Denied, failed, conflicted, and quarantined states may require trace/audit records.

Trace/audit must explain causality:

```text
request
  -> gate/check
  -> denial/failure/conflict/quarantine
  -> preserved state
  -> next allowed action
```

Visual state is a projection.

Trace/audit remains the source of truth.

## 13. Capability relationship

Capability denial is not generic error.

Example:

```text
ui.capability.commit_effect missing
  -> denial
  -> capability reason visible
  -> no effect request commit
```

Capability visual state must distinguish:

```text
missing
denied
unknown
admitted
quarantined
conflicted
```

Showing a missing capability must not imply system failure.

## 14. Effect relationship

Effect refusal/failure must distinguish:

```text
effect request denied
effect prepare failed
effect prepared
effect commit denied
effect commit failed
effect committed
effect rolled back
effect quarantined
```

Prepared effect must not be shown as committed.

Denied commit must not increment committed-effect state.

## 15. Renderer relationship

Renderer must not define error/denial/quarantine meaning.

Renderer may display admitted visual projections.

Renderer must not decide:

- denial reason;
- error classification;
- quarantine authority;
- conflict meaning;
- effect success;
- audit authority.

Renderer failure itself may become a failure fact if admitted by future renderer transcript boundary.

## 16. Native backend relationship

Native backend may expose native failure/transcript facts.

Examples:

```text
window creation failed
event loop failed
native window closed
native input unsupported
```

Native backend must not decide core semantic denial or quarantine meaning.

Native backend transcript facts may be projected into UI only through admitted trace/error boundaries.

## 17. Component relationship

Components may expose error/denial/quarantine surfaces.

Candidate components:

```text
DenialReasonOverlay
QuarantineNotice
ConflictBoundaryView
ErrorDetailPanel
FailureStageView
CapabilityMissingBadge
RollbackTraceView
```

Components must not invent denial or error meaning.

They display admitted state and trace facts.

## 18. Layout relationship

Layout primitives may provide error/denial/quarantine regions.

Examples:

```text
ConflictBoundary
QuarantineRegion
OverlaySurface
TraceLane
InspectorPane
```

Layout primitives must not own error meaning.

They provide spatial structure for inspection.

## 19. Required visual distinction table

Future visual implementation must distinguish:

| Condition | Not equivalent to | Required visibility |
| --- | --- | --- |
| denial | error/failure | reason and gate |
| error | denial/crash | source and condition |
| failure | denial/crash | failed stage |
| conflict | crash | conflicting parties |
| quarantine | deletion | isolated target and allowed actions |
| no-op | hidden refusal | reason if user-visible |
| crash | normal failure | hard boundary |

## 20. Forbidden shortcuts

The system must not:

- hide denial as no-op;
- show denial as crash;
- show missing capability as generic failure;
- show prepared effect as committed;
- show quarantine as deletion;
- show conflict as crash;
- let renderer invent denial reason;
- let native backend own quarantine semantics;
- let Workbench define core error categories;
- remove trace/audit path from user-visible denial/failure;
- use color alone to distinguish failure states.

## 21. Required admission rule

A future error/denial/quarantine visual implementation PR must define:

1. state category;
2. source layer;
3. visible label;
4. required trace/audit link if applicable;
5. denial/failure/quarantine reason shape;
6. recovery/action affordance if any;
7. component/layout projection;
8. renderer projection boundary;
9. tests/snapshots where applicable.

No error/denial/quarantine visual state should be added only as cosmetic feedback.

## 22. Future implementation shape

H10 does not mandate implementation.

Possible future shapes:

```text
docs/spec/ui_error_denial_quarantine.md
crates/prom-ui-status/
crates/prom-ui-errors/
crates/prom-ui-components/
apps/workbench error/quarantine views
renderer status projection map
```

Any implementation must preserve:

```text
semantic status
  -> trace/audit relation
  -> visual projection
  -> inspectable reason
```

## 23. Current decision

Error, denial, and quarantine visual handling is not implemented in H10.

H10 only defines the boundary.

Current admitted visual/interaction/action architecture:

```text
visual doctrine
  -> visual token boundary
  -> layout primitive boundary
  -> component admission boundary
  -> interaction/input semantic boundary
  -> focus/selection semantic boundary
  -> semantic action boundary
  -> effect request / UI capability boundary
  -> trace/audit visual boundary
  -> error/denial/quarantine visual boundary
```

Not yet admitted:

```text
error visual structs
denial visual structs
quarantine visual structs
status components
error overlay implementation
quarantine region implementation
conflict visual resolver
Workbench error views
renderer-owned error semantics
native backend-owned quarantine semantics
```

## Renderer transcript and presentation status dependency

Renderer denial, renderer failure, and presentation failure must remain visually distinct.

Renderer transcript and presentation status boundaries are defined separately in:

```text
docs/architecture/ui_renderer_transcript_presentation_boundary.md
```

Render success must not hide presentation failure.
