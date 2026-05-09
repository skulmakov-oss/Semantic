# Semantic UI Trace and Audit Visual Boundary

Status: Draft
Track: POST-UI / H-series
Purpose: define trace/audit visual boundaries before implementation

## 1. Goal

This document defines the boundary between trace/audit records and their visual representation in Semantic UI.

Trace is not a decorative log.

Audit is not a visual timeline.

A UI trace view is a projection of trace/audit facts, not the source of truth.

The project must preserve this chain:

```text
operation / action / effect
  -> trace/audit record
  -> visual trace projection
  -> inspection UI
```

No visual layer may invent, overwrite, or silently omit trace/audit meaning.

## 2. Relationship to action/effect boundaries

Trace and audit visualization depends on:

```text
docs/architecture/ui_semantic_action_boundary.md
docs/architecture/ui_effect_request_capability_boundary.md
```

Semantic actions may produce traces.

Effect requests and committed effects must be traceable.

The UI may display trace facts, but the UI display is not the audit record itself.

## 3. Layer separation

| Layer | Meaning | Owner |
| --- | --- | --- |
| semantic action | admitted UI operation | UI action layer |
| effect request | requested controlled effect | effect/capability boundary |
| prepared/committed effect | controlled effect state | runtime/effect system |
| trace record | causal operation record | trace/audit layer |
| audit record | durable/security-relevant record | audit/runtime boundary |
| visual trace projection | UI representation of trace/audit facts | UI architecture layer |
| renderer output | pixels/native presentation | renderer |

This preserves:

```text
Trace is not log decoration.
Audit is not UI state.
Visual trace is not source of truth.
Renderer output is not audit authority.
```

## 4. Trace definition

A trace is a structured causal record.

It may describe:

- source input;
- interaction intent;
- admission result;
- semantic action;
- focus/selection target;
- capability checks;
- effect request;
- prepare/commit boundary;
- denial/failure reason;
- renderer/presentation facts if admitted later.

Trace is not arbitrary text logging.

Trace must preserve causality.

## 5. Audit definition

Audit is a record with accountability requirements.

Audit may be required for:

- effectful actions;
- capability admission;
- denied capability requests;
- prepared/committed effects;
- rollback;
- quarantine/conflict transitions;
- policy refusal;
- security-relevant UI operations.

Audit must not depend on whether the UI chooses to display it.

## 6. Visual trace projection definition

A visual trace projection is a UI representation of trace/audit facts.

Examples:

```text
TraceLane
TraceEventRow
TraceSummaryPanel
RollbackTraceView
EffectCommitView
RendererTranscriptView
DenialReasonOverlay
```

A visual projection may filter, group, collapse, or focus trace data.

But it must not change the meaning of trace/audit records.

## 7. Trace vs log

A log may be chronological text.

A trace must preserve semantic causality.

Example:

```text
log:
  "button clicked"
  "effect done"

trace:
  input -> intent -> admission -> action -> effect request -> capability check -> prepare -> commit
```

Semantic UI must prefer trace over raw log display.

Logs may be used as implementation details, but visual doctrine must be trace-first.

## 8. Trace visibility vs effect success

Displaying a trace does not mean the effect succeeded.

Examples:

```text
trace visible -> effect denied
trace visible -> effect prepared
trace visible -> effect committed
trace visible -> effect failed
trace visible -> rollback requested
```

Visual trace state must clearly distinguish:

```text
requested
admitted
denied
prepared
committed
failed
rolled_back
quarantined
```

No trace view may collapse these into a generic “done” state.

## 9. Audit visibility vs audit existence

Audit may exist even when not currently visible.

Audit visibility is a UI concern.

Audit existence is a runtime/security concern.

The UI must not imply:

```text
not visible == not audited
visible == authoritative audit source
```

The source of truth remains the audit/runtime boundary.

## 10. Trace ownership

Trace/audit meaning is not owned by renderer, native backend, or Workbench.

| Component | Trace/audit role |
| --- | --- |
| semantic action layer | may produce action trace facts |
| effect boundary | may produce effect trace facts |
| capability/admission layer | may produce admission/denial trace facts |
| audit/runtime boundary | owns authoritative audit records |
| UI trace projection layer | displays trace/audit facts |
| component system | exposes trace surfaces |
| layout primitive system | provides trace lanes/panels |
| renderer | renders projection only |
| native backend | may expose native transcript facts, not audit meaning |
| Workbench | may consume/display trace, not define core trace meaning |

## 11. Trace visual grammar

Trace visual grammar must show causality.

Required concepts:

```text
source
intent
admission
action
effect request
capability check
prepare
commit
denial/failure
trace link
audit link
```

Trace views must support inspection of why something happened, not only that it happened.

## 12. Trace lane boundary

A trace lane is a layout/visual structure.

It is not the trace record itself.

A trace lane may display:

```text
ordered events
causal groups
effect stages
denials
rollbacks
capability checks
renderer transcript facts
```

But the trace lane must not be the authoritative trace store.

## 13. Denial and failure trace rules

Denied or failed operations must be visually distinguishable.

Denied trace must show:

- requested operation;
- admission gate;
- denial reason;
- missing capability if any;
- lifecycle conflict if any;
- target context;
- trace/audit reference if available.

Failure trace must show:

- attempted operation;
- failure stage;
- error or status;
- effect state if any;
- rollback/cancellation state if any.

No denial/failure should vanish into a generic disabled state.

## 14. Renderer transcript relationship

Renderer transcript facts are not audit records by default.

Renderer transcript may describe:

```text
draw staging
render attempted
render succeeded
frame presented
render failed
```

These may be displayed in trace UI.

But renderer transcript does not become audit authority unless a future boundary explicitly admits it.

## 15. Native backend transcript relationship

Native backend may expose:

```text
window lifecycle facts
event translation facts
draw staging facts
native facade summary
```

These are transcript facts.

They are not automatically audit records.

Mapping native transcript facts into trace/audit must be explicit.

## 16. Workbench relationship

Workbench may display trace/audit projections.

Workbench must not define core trace/audit semantics.

Workbench-specific trace views require:

- Workbench-local projection namespace; or
- explicit admission into core UI trace contracts; or
- separate boundary document.

No Workbench convenience timeline should become the core trace model.

## 17. Trace filtering and collapse rules

Trace UI may filter or collapse records only if it preserves meaning.

Allowed:

```text
collapse purely repetitive non-semantic details
group by action/effect/request
focus on selected target
hide low-level renderer details by default
```

Forbidden:

```text
hide denial reason
hide failure stage
hide capability refusal
show prepared effect as committed
show trace presence as success
drop audit-relevant data from inspection path
```

## 18. Required trace visual admission rule

A future trace/audit visual implementation PR must define:

1. trace source;
2. audit source if applicable;
3. visual projection type;
4. filtering/collapse rules;
5. denial/failure display rules;
6. capability/effect relation;
7. renderer/native transcript relation;
8. authority boundary;
9. tests/snapshots where applicable.

No trace view should be added only because it looks like a useful timeline.

## 19. Forbidden shortcuts

The system must not:

- treat trace as decorative log;
- treat audit as UI state;
- treat visual trace as source of truth;
- treat trace visibility as effect success;
- treat renderer transcript as audit record by default;
- let renderer define trace meaning;
- let native backend define audit meaning;
- let Workbench define core trace semantics;
- hide denial/failure trace;
- collapse prepare and commit in visual state;
- omit capability refusal from trace inspection.

## 20. Future implementation shape

H9 does not mandate implementation.

Possible future shapes:

```text
docs/spec/ui_trace_visuals.md
docs/spec/ui_audit_projection.md
crates/prom-ui-trace/
crates/prom-ui-audit-view/
apps/workbench trace projection map
renderer trace lane resolver
```

Any implementation must preserve:

```text
operation / action / effect
  -> trace/audit record
  -> visual trace projection
  -> inspection UI
```

## 21. Current decision

Trace/audit visual projection is not implemented in H9.

H9 only defines the boundary.

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
```

Not yet admitted:

```text
trace visual structs
audit projection structs
trace lane implementation
trace filtering implementation
audit view implementation
Workbench trace timeline
renderer trace lane resolver
trace-to-audit mapping
renderer transcript audit mapping
native transcript audit mapping
```

## Error, denial, and quarantine visual dependency

Error, denial, and quarantine visual boundaries are defined separately in:

```text
docs/architecture/ui_error_denial_quarantine_visual_boundary.md
```

Trace/audit projection may explain denial, failure, conflict, and quarantine.

Visual trace must not hide denial/failure/quarantine state.

## Recovery and rollback visual dependency

Recovery and rollback visualization may require trace/audit records.

Recovery and rollback visual boundaries are defined separately in:

```text
docs/architecture/ui_recovery_rollback_visual_boundary.md
```

Visual recovery must not hide trace/audit causality.
