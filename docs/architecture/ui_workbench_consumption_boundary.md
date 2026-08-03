# Semantic UI Workbench Consumption Boundary

Status: Draft
Track: POST-UI / H-series
Purpose: define how Workbench may consume Semantic UI contracts before implementation

## 1. Goal

This document defines the boundary for Workbench as a consumer of Semantic UI contracts.

Workbench is not the source of core UI semantics.

Workbench may consume, project, inspect, and operate through admitted UI contracts.

The project must distinguish:

```text
Workbench consumes admitted UI contracts
Workbench does not define core UI semantics
Workbench convenience != architecture rule
Workbench view != source of truth
Workbench command != semantic action by default
```

Workbench must not become the hidden owner of Semantic UI meaning.

## 2. Relationship to existing UI boundaries

Workbench consumption depends on the admitted UI boundary chain:

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
  -> recovery/rollback visual boundary
  -> renderer transcript / presentation status boundary
```

Workbench may consume these contracts.

Workbench must not replace them.

## 3. Layer separation

| Layer              | Meaning                                                        | Owner                      |
| ------------------ | -------------------------------------------------------------- | -------------------------- |
| core UI contracts  | doctrine, tokens, layout, components, actions, effects, traces | UI architecture layer      |
| Workbench view     | projection of admitted UI state                                | Workbench                  |
| Workbench command  | local tool/action request                                      | Workbench                  |
| semantic UI action | admitted UI-level operation                                    | UI action layer            |
| effect request     | controlled effect request                                      | effect/capability boundary |
| trace/audit record | authoritative causality/audit                                  | trace/audit boundary       |
| renderer output    | visual presentation                                            | renderer boundary          |

This preserves:

```text
Workbench view is not source of truth.
Workbench command is not semantic action by default.
Workbench convenience is not architecture.
Workbench display is not authority.
```

## 4. Workbench definition

Workbench is a developer/operator surface for inspecting and interacting with Semantic UI/runtime concepts.

Workbench may provide:

* state inspection;
* trace projection;
* module views;
* capability views;
* action request surfaces;
* effect request surfaces;
* renderer transcript views;
* diagnostics;
* development tools.

Workbench must not define core meaning for those concepts.

## 5. Workbench view boundary

A Workbench view is a projection.

It may show:

```text
trace lanes
capability state
module state
component state
renderer transcript
effect request status
error/denial/quarantine status
recovery options
```

A Workbench view must not become:

```text
trace authority
audit authority
capability authority
effect authority
renderer authority
semantic state authority
```

Workbench views must show source links or state provenance when meaning is non-local.

## 6. Workbench command boundary

A Workbench command is a tool-level request.

It is not automatically a semantic UI action.

Example:

```text
workbench.command.inspect_trace
  -> may become local Workbench view operation

workbench.command.rollback_effect
  -> must request semantic action/effect admission
```

Workbench command must pass through the same admission boundaries as any other action if it affects semantic state, capability, effect, trace, renderer, or native lifecycle.

## 7. Workbench convenience boundary

Workbench convenience must not become architecture rule.

Examples of convenience:

```text
quick filter
temporary panel
debug-only toggle
local layout preset
developer shortcut
local diagnostic view
```

These may exist inside Workbench.

They must not redefine:

* component semantics;
* action semantics;
* effect semantics;
* trace/audit semantics;
* renderer transcript semantics;
* recovery semantics;
* capability semantics.

## 8. Core vs Workbench namespace

Workbench-specific concepts must be namespaced.

Allowed direction:

```text
workbench.view.*
workbench.command.*
workbench.shortcut.*
workbench.panel.*
workbench.diagnostic.*
```

Core UI concepts must remain in core namespaces:

```text
ui.action.*
ui.capability.*
ui.effect.*
ui.trace.*
ui.renderer.*
```

Workbench-local names must not leak into core UI contracts without explicit admission.

## 9. Workbench and visual doctrine

Workbench must follow Semantic UI visual doctrine:

```text
docs/architecture/ui_visual_design_doctrine.md
```

Workbench may have local layout density or developer tooling views.

But it must preserve:

* semantic-first visual meaning;
* traceability;
* capability/admission visibility;
* lifecycle clarity;
* denial/failure visibility;
* renderer transcript distinction.

Workbench must not become a generic IDE/dashboard skin.

## 10. Workbench and components

Workbench may consume admitted components.

Workbench may also define local Workbench-only components.

But Workbench-local components must not become core components unless admitted through:

```text
docs/architecture/ui_component_admission_boundary.md
```

Example distinction:

```text
TraceEventRow
  -> core semantic component candidate

workbench.trace.FilterBar
  -> Workbench-local convenience component
```

## 11. Workbench and interactions

Workbench may provide interaction surfaces.

Workbench interactions must preserve:

```text
native/input event
  -> interaction intent
  -> admission
  -> semantic action if admitted
```

Workbench shortcuts or command palette entries must not bypass admission.

A Workbench keyboard shortcut is not semantic permission.

## 12. Workbench and semantic actions

Workbench may request semantic UI actions.

Workbench must not define core action semantics.

Example:

```text
workbench.command.close_panel
  -> local Workbench action

workbench.command.commit_effect
  -> request ui.action.commit_effect
  -> admission
  -> effect request boundary
```

Workbench action maps must clearly distinguish local Workbench actions from core Semantic UI actions.

## 13. Workbench and effect/capability boundary

Workbench may display and request effect/capability operations.

Workbench must not grant capabilities.

Workbench must not treat UI capability display as grant.

Workbench must not map UI capability to runtime capability without explicit admitted mapping.

Workbench may show:

```text
capability available
capability denied
runtime capability missing
effect request prepared
effect committed
effect rollback available
```

But the source of truth remains the admitted capability/effect boundary.

## 14. Workbench and trace/audit boundary

Workbench may display trace/audit projections.

Workbench must not be the audit record.

Workbench trace views may filter, group, or search.

But they must not:

* hide denial/failure meaning;
* show prepared effect as committed;
* show trace visibility as success;
* omit audit-relevant causality from inspection path.

Workbench view is projection, not authority.

## 15. Workbench and renderer transcript boundary

Workbench may display renderer transcript and presentation status.

Workbench must preserve:

```text
draw staging != render attempted
render attempted != render succeeded
render succeeded != frame presented
frame presented != semantic success
renderer transcript != audit authority
```

Workbench must not show `submitted_frames` as presented frames.

## 16. Workbench and error/recovery boundaries

Workbench may display denial, failure, quarantine, recovery, rollback, retry, cancel, and inspect options.

Workbench must preserve:

```text
error != denial
denial != failure
quarantine != deletion
recovery != rollback
rollback != undo
retry != blind re-execute
```

Workbench recovery commands must not bypass admission.

## 17. Workbench source-of-truth rule

Workbench is never source of truth for:

```text
semantic state
capability grant
runtime capability
effect commitment
trace/audit authority
renderer presentation authority
quarantine authority
recovery admission
```

Workbench may cache or project state only if source/provenance is clear.

Stale Workbench views must not look authoritative.

## 18. Workbench stale-state rule

Workbench may display stale state only if marked as stale.

Future implementation must distinguish:

```text
live
snapshot
stale
disconnected
replayed
simulated
```

Workbench must not allow effectful action from stale state without admission and freshness checks.

H13 does not implement this.

## 19. Workbench simulation boundary

Workbench may later support simulated views.

Simulation must be explicit.

A simulated Workbench state is not runtime state.

A replayed trace is not live trace.

A previewed action is not admitted action.

Simulation requires separate boundary if implemented.

## 20. Workbench renderer/native boundary

Workbench may use renderer/native backend to display views.

Renderer/native backend must not define Workbench semantics.

Workbench must not use native renderer facts as semantic success.

Workbench must not use native window state as core lifecycle truth unless admitted.

## 21. Forbidden shortcuts

The system must not:

* let Workbench define core UI semantics;
* treat Workbench command as semantic action by default;
* treat Workbench view as source of truth;
* treat Workbench convenience as architecture rule;
* let Workbench grant capabilities;
* let Workbench perform effects without admission;
* let Workbench hide denial/failure/quarantine;
* let Workbench collapse renderer transcript states;
* let Workbench bypass trace/audit;
* let Workbench-specific components leak into core without admission.

## 22. Required Workbench consumption admission rule

A future Workbench UI implementation PR must define:

1. consumed core contract;
2. Workbench view or command name;
3. source of truth;
4. projection rules;
5. stale/snapshot behavior if applicable;
6. admission path for actions/effects;
7. capability requirements;
8. trace/audit relation;
9. local vs core namespace;
10. tests/snapshots where applicable.

No Workbench feature should be added only because it is convenient for development.

## 23. Future implementation shape

H13 does not mandate implementation.

Possible future shapes:

```text
docs/spec/ui_workbench_consumption.md
apps/workbench_ts_tauri_legacy/docs/
apps/workbench_ts_tauri_legacy/src/views/
apps/workbench_ts_tauri_legacy/src/commands/
crates/prom-ui-workbench-bridge/
```

Any implementation must preserve:

```text
core UI contract
  -> Workbench projection / command request
  -> admission if state/effect/action is affected
  -> trace/audit relation if relevant
```

## 24. Current decision

Workbench UI consumption is not implemented in H13.

H13 only defines the boundary.

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
  -> recovery/rollback visual boundary
  -> renderer transcript / presentation status boundary
  -> Workbench UI consumption boundary
```

Not yet admitted:

```text
Workbench UI components
Workbench action bridge
Workbench effect bridge
Workbench capability grant
Workbench trace authority
Workbench renderer authority
Workbench stale-state model
Workbench simulation model
Workbench command palette semantics
```

## Simulation and snapshot dependency

Workbench may later display live, snapshot, replay, simulation, preview, stale, and disconnected views.

Simulation and snapshot UI boundaries are defined separately in:

```text
docs/architecture/ui_simulation_snapshot_boundary.md
```

Workbench must not let non-live views look authoritative.

## Implementation gate dependency

Workbench implementation remains gated by:

```text
docs/architecture/ui_implementation_gate.md
docs/architecture/ui_boundary_index.md
```

Workbench must consume admitted contracts and must not define core semantics.
