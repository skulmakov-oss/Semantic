# Semantic UI Effect Request and Capability Boundary

Status: Draft
Track: POST-UI / H-series
Purpose: define UI effect request and capability boundaries before implementation

## 1. Goal

This document defines the boundary between Semantic UI actions, UI effect requests, UI capabilities, runtime capabilities, and committed effects.

A Semantic UI action may request an effect.

That request is not the effect itself.

A UI capability may explain whether an effect request is allowed at the UI layer.

That UI capability is not automatically a runtime capability grant.

The project must preserve this chain:

```text
semantic UI action
  -> effect request
  -> UI capability admission
  -> runtime capability mapping if admitted
  -> prepared effect
  -> commit boundary
  -> committed effect
```

No layer may skip capability/effect admission and directly produce effects.

## 2. Relationship to semantic action boundary

Effect requests depend on the semantic action boundary:

```text
docs/architecture/ui_semantic_action_boundary.md
```

Semantic UI actions are admitted UI-level operations.

Some actions are purely local UI actions.

Some actions may request an effect.

Example:

```text
ui.action.open_inspector
  -> local semantic UI action
  -> no external effect request

ui.action.prepare_effect
  -> effect request
  -> capability/effect admission
```

H8 does not implement either path.
It defines the boundary.

## 3. Layer separation

| Layer | Meaning | Owner |
| --- | --- | --- |
| semantic UI action | admitted UI operation | future UI action layer |
| effect request | request to perform controlled effect | future UI/effect bridge |
| UI capability | UI-level permission/admission concept | future UI policy layer |
| runtime capability | actual runtime capability gate | Prometheus/runtime capability layer |
| prepared effect | effect accepted for controlled execution | effect boundary |
| committed effect | effect executed/committed | runtime/effect system |
| trace/audit | observable causal record | audit/runtime boundary |

This preserves:

```text
Action is not effect.
Effect request is not committed effect.
UI capability is not runtime capability by default.
Prepared effect is not committed effect.
```

## 4. Effect request definition

An effect request is:

```text
a named request from an admitted Semantic UI action to prepare a controlled effect
```

It must define:

- source semantic action;
- requested effect kind;
- target identity;
- required UI capability;
- required runtime capability if mapped;
- lifecycle preconditions;
- trace behavior;
- denial behavior;
- prepare/commit relationship.

Effect request is not execution.

## Effect request descriptor dependency

The I-series effect request descriptor boundary is defined separately in:

```text
docs/architecture/ui_effect_request_descriptor_boundary.md
```

The descriptor step narrows this general H-series boundary.

```text
dispatch trace / summary
  -> future EffectRequestDescriptor
  -> future UI capability admission
  -> future prepared effect
```

Effect request descriptor is not execution.
Effect request descriptor is not capability grant.
Prepared effect is not committed effect.

## Workbench effect and capability dependency

Workbench may display and request effect/capability operations.

Workbench UI consumption is defined separately in:

```text
docs/architecture/ui_workbench_consumption_boundary.md
```

Workbench must not grant capabilities or perform effects without admission.

## 5. UI capability definition

A UI capability is a UI-layer permission concept.

It may describe:

```text
can_view_trace
can_open_inspector
can_select_target
can_prepare_effect
can_commit_effect
can_rollback_effect
can_close_window
can_view_renderer_transcript
```

UI capability answers:

```text
is this UI operation admitted at the UI policy layer?
```

It does not automatically answer:

```text
does runtime have the capability to perform external effect?
```

That requires explicit mapping.

## 6. Runtime capability mapping

A UI capability may map to a runtime capability.

This mapping must be explicit.

Example candidate mapping:

```text
ui.capability.prepare_effect
  -> runtime capability: effect.prepare

ui.capability.commit_effect
  -> runtime capability: effect.commit

ui.capability.view_trace
  -> maybe no external runtime capability
```

H8 does not define actual runtime capability names.

H8 only requires that future mappings are explicit, testable, and traceable.

## 7. Capability visibility vs capability grant

Showing a capability in UI does not grant it.

Example:

```text
CapabilityStatusBadge
  -> visible capability state
  -> not a grant
```

A visual capability indicator may show:

```text
available
missing
denied
admitted
quarantined
unknown
```

But the grant must come from the admitted capability/policy layer.

Visual state is not authority.

## 8. Prepared effect vs committed effect

Prepared effect is not committed effect.

Example:

```text
effect request
  -> admission
  -> prepared effect
  -> commit boundary
  -> committed effect
```

Prepared effect may still be:

- cancelled;
- denied at commit;
- rolled back if supported;
- quarantined;
- expired;
- superseded by state change.

Committed effect must be traceable.

## 9. Local UI action vs effectful UI action

Semantic UI actions may be local or effectful.

Local examples:

```text
ui.action.open_inspector
ui.action.select_trace_event
ui.action.focus_module
ui.action.open_denial_reason
```

Effectful examples:

```text
ui.action.prepare_effect
ui.action.commit_effect
ui.action.rollback_effect
ui.action.close_window
ui.action.quarantine_target
```

The classification must be explicit in future action implementation.

No effectful action may silently execute as if it were local.

## 10. Admission boundary

Effect request admission may check:

- source action admission;
- UI capability;
- runtime capability mapping;
- lifecycle state;
- target ownership;
- target quarantine/conflict state;
- traceability requirement;
- effect prepare/commit rules;
- renderer/native readiness if effect touches UI/window/presentation;
- policy gates.

Denied effect request must be visible and traceable if user-visible.

## 11. Trace and audit requirement

Effect request path must be explainable.

For an effectful UI action, the system should answer:

1. Which semantic action requested the effect?
2. What effect was requested?
3. Which UI capability was required?
4. Was UI capability admitted?
5. Was runtime capability mapping required?
6. Was runtime capability present?
7. Was effect prepared?
8. Was effect committed?
9. What trace/audit record was produced?
10. What failed, if anything?

No effect path should be opaque.

## 12. Component relationship

Components may expose effect request affordances.

Examples:

```text
EffectCommitView
  -> request commit effect

RollbackTraceView
  -> request rollback

CapabilityDecisionView
  -> show capability admission result
```

Components must not perform effects directly.

Components may request semantic actions.
Semantic actions may request effects.
Effect requests require capability/effect admission.

## 13. Renderer relationship

Renderer must not perform effects.

Renderer may display:

- capability state;
- effect request state;
- prepared effect state;
- committed effect trace;
- denial/failure state.

Renderer must not decide:

- UI capability grant;
- runtime capability grant;
- effect preparation;
- effect commitment;
- rollback authority;
- trace/audit result.

Renderer output is not capability authority.

## 14. Native backend relationship

Native backend must not perform Semantic UI effects by itself.

Native backend may own platform operations only when an admitted native/backend effect boundary exists.

Example:

```text
window close
  -> semantic UI action
  -> effect/lifecycle request
  -> admission
  -> native backend operation
```

Native backend must not treat host events as effect permission.

## 15. Workbench relationship

Workbench may expose effectful UI actions.

Workbench must not define core UI effect semantics.

Workbench-specific effects require:

- Workbench-local effect namespace; or
- explicit admission into core UI effect contract; or
- separate boundary document.

No Workbench convenience effect should leak into core UI semantics.

## 16. UI capability vs runtime capability examples

| UI capability | Runtime capability relationship |
| --- | --- |
| `ui.capability.view_trace` | may be local UI-only |
| `ui.capability.open_inspector` | may be local UI-only |
| `ui.capability.prepare_effect` | may require runtime effect capability |
| `ui.capability.commit_effect` | likely requires runtime effect capability |
| `ui.capability.close_window` | may require lifecycle/native backend admission |
| `ui.capability.view_renderer_transcript` | may be local unless renderer state is protected |

These are examples, not implementation commitments.

## 17. Denial behavior

Denied effect requests must not disappear silently.

They may produce:

```text
effect denial trace
capability missing state
runtime capability missing state
policy refusal
lifecycle invalid state
quarantine/conflict reason
commit window closed reason
```

Denied effect requests must not increment committed-effect state.

## 18. Forbidden shortcuts

The system must not:

- treat semantic UI action as effect;
- treat effect request as committed effect;
- treat UI capability display as capability grant;
- treat UI capability as runtime capability without mapping;
- perform external effects from component callbacks;
- let renderer perform effects;
- let native backend perform effects from host events directly;
- let Workbench define core effect semantics;
- bypass trace/audit for effectful actions;
- collapse prepare and commit into hidden one-step mutation unless explicitly admitted.

## 19. Required effect request admission rule

A future effect request implementation PR must define:

1. effect request name;
2. source semantic action;
3. target identity model;
4. required UI capability;
5. runtime capability mapping if any;
6. prepare behavior;
7. commit behavior;
8. cancellation/rollback behavior if supported;
9. denial behavior;
10. trace/audit behavior;
11. tests/snapshots where applicable.

No effect request should be added only because it is convenient for UI wiring.

## 20. Future implementation shape

H8 does not mandate implementation.

Possible future shapes:

```text
docs/spec/ui_effect_requests.md
docs/spec/ui_capabilities.md
crates/prom-ui-actions/
crates/prom-ui-cap/
crates/prom-ui-effect/
prom-ui-runtime effect admission module
Workbench effect map
renderer capability display map
```

Any implementation must preserve:

```text
semantic UI action
  -> effect request
  -> UI capability admission
  -> runtime capability mapping if admitted
  -> prepared effect
  -> commit boundary
  -> committed effect
```

## 21. Current decision

Effect requests and UI capabilities are not implemented in H8.

H8 only defines the boundary.

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
```

Not yet admitted:

```text
UI capability structs
effect request structs
effect dispatcher
UI-to-runtime capability map
prepare/commit implementation
rollback implementation
Workbench effect implementation
renderer effect/capability authority
native backend direct effect execution
```

## Trace and audit visual dependency

Trace/audit visual boundaries are defined separately in:

```text
docs/architecture/ui_trace_audit_visual_boundary.md
```

Effect requests and committed effects may produce trace/audit records.

UI trace projection may display those records, but must not become the source of truth.

## Error, denial, and quarantine visual dependency

Effect request and capability denial states must be visually distinct.

Error, denial, and quarantine visual boundaries are defined separately in:

```text
docs/architecture/ui_error_denial_quarantine_visual_boundary.md
```

Effect denial is not effect failure.
Prepared effect is not committed effect.
Quarantine is not deletion.

## Recovery and rollback visual dependency

Effect recovery and rollback states must be visually distinct.

Recovery and rollback visual boundaries are defined separately in:

```text
docs/architecture/ui_recovery_rollback_visual_boundary.md
```

Retry is not blind re-execute.
Rollback is not generic undo.
Prepared effect recovery is not committed effect rollback.

## Renderer transcript and presentation status dependency

Frame presentation is not semantic effect success.

Renderer transcript and presentation status boundaries are defined separately in:

```text
docs/architecture/ui_renderer_transcript_presentation_boundary.md
```

Effect success remains owned by the effect/capability boundary.

## Simulation and snapshot dependency

Previewed or simulated capability/effect state is not capability/effect authority.

Simulation and snapshot UI boundaries are defined separately in:

```text
docs/architecture/ui_simulation_snapshot_boundary.md
```

A simulated capability is not a grant.
A previewed effect is not prepared or committed.
