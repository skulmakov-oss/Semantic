# Semantic UI Boundary Index

Status: Draft
Track: POST-UI / H-series
Purpose: provide the canonical index of admitted Semantic UI architecture boundaries

## 1. Goal

This document is the canonical index for Semantic UI architecture boundaries.

It exists so implementation PRs do not search through scattered documents or accidentally bypass earlier decisions.

Every future Semantic UI implementation PR must identify which boundary documents it depends on.

## 2. Boundary stack

Current admitted UI boundary stack:

```text
visual doctrine
  -> visual token boundary
  -> layout primitive boundary
  -> component admission boundary
  -> interaction/input semantic boundary
  -> focus/selection semantic boundary
  -> semantic action boundary
  -> action admission descriptor
  -> action admission result / denial trace
  -> admitted semantic action object
  -> semantic action dispatcher
  -> effect request / UI capability boundary
  -> trace/audit visual boundary
  -> error/denial/quarantine visual boundary
  -> recovery/rollback visual boundary
  -> renderer transcript / presentation status boundary
  -> Workbench UI consumption boundary
  -> simulation/snapshot UI boundary
  -> implementation gate
```

## 3. Canonical documents

| Boundary                          | Document                                                            |
| --------------------------------- | ------------------------------------------------------------------- |
| visual doctrine                   | `docs/architecture/ui_visual_design_doctrine.md`                    |
| visual token system               | `docs/architecture/ui_visual_token_system_boundary.md`              |
| layout primitives                 | `docs/architecture/ui_layout_primitive_boundary.md`                 |
| component admission               | `docs/architecture/ui_component_admission_boundary.md`              |
| interaction/input semantics       | `docs/architecture/ui_interaction_input_semantic_boundary.md`       |
| focus/selection semantics         | `docs/architecture/ui_focus_selection_semantic_boundary.md`         |
| semantic actions                  | `docs/architecture/ui_semantic_action_boundary.md`                  |
| interaction-action trace ladder   | `docs/architecture/ui_interaction_action_trace_ladder.md`           |
| action admission descriptor       | `docs/architecture/ui_action_admission_descriptor_boundary.md`      |
| action admission result / denial trace | `docs/architecture/ui_action_admission_result_denial_boundary.md` |
| admitted semantic action object   | `docs/architecture/ui_admitted_semantic_action_boundary.md`         |
| semantic action dispatcher        | `docs/architecture/ui_semantic_action_dispatcher_boundary.md`      |
| effect requests / UI capabilities | `docs/architecture/ui_effect_request_capability_boundary.md`        |
| trace/audit visual projection     | `docs/architecture/ui_trace_audit_visual_boundary.md`               |
| error/denial/quarantine visuals   | `docs/architecture/ui_error_denial_quarantine_visual_boundary.md`   |
| recovery/rollback visuals         | `docs/architecture/ui_recovery_rollback_visual_boundary.md`         |
| renderer transcript/presentation  | `docs/architecture/ui_renderer_transcript_presentation_boundary.md` |
| Workbench consumption             | `docs/architecture/ui_workbench_consumption_boundary.md`            |
| simulation/snapshot UI            | `docs/architecture/ui_simulation_snapshot_boundary.md`              |
| implementation gate               | `docs/architecture/ui_implementation_gate.md`                       |
| ownership map                     | `docs/architecture/ui_ownership_map.md`                             |
| renderer admission                | `docs/architecture/ui_renderer_admission_boundary.md`               |
| native backend boundary           | `docs/architecture/ui_native_backend_boundary.md`                   |

## 4. Non-negotiable invariants

Future UI code must preserve:

```text
meaning first
tokens second
layout third
components fourth
interaction fifth
actions/effects only after admission
trace/audit source of truth remains outside visual projection
renderer output is not semantic authority
Workbench consumes contracts, does not define them
non-live views are not authority
```

## 5. First implementation families

Allowed implementation families after this index:

| Family              | Allowed first step                                              |
| ------------------- | --------------------------------------------------------------- |
| visual tokens       | type/enum scaffold or docs-backed token map, no renderer        |
| layout primitives   | type scaffold, no rendering                                     |
| components          | admitted component metadata scaffold, no widgets                |
| interaction         | intent enum scaffold, no effect execution                       |
| focus/selection     | state scaffold, no pointer/hit-test                             |
| actions             | action descriptor scaffold, no effect bridge                    |
| renderer transcript | transcript type scaffold, no renderer                           |
| Workbench           | consumption map docs or local projection scaffold, no authority |
| simulation/snapshot | mode enum scaffold, no replay engine                            |

Renderer implementation is not the first recommended step.

## 6. Recommended first code PR

The recommended first implementation PR is:

```text
PR-UI-I1 — feat(ui): add UI boundary registry scaffold
```

It should add a small registry/metadata layer that lists admitted boundaries and exposes them to tests or docs tooling.

It should not implement renderer, components, actions, effects, Workbench, or simulation.

## 7. Forbidden implementation shortcuts

Future UI implementation must not:

* start with renderer pixels;
* start with Workbench dashboard;
* start with component library;
* add buttons before action/admission model;
* treat visual tokens as arbitrary theme values;
* treat Workbench command as semantic action;
* treat submitted frames as presented frames;
* treat preview/simulation as live authority;
* treat trace display as audit authority;
* hide denial/failure/quarantine as no-op.

## 8. Change policy

If implementation reveals that a boundary is incomplete, do not patch code around it.

Correct order:

```text
boundary correction PR
  -> implementation PR
```

Emergency exceptions require explicit note in PR body.

## 9. Current decision

The H-series boundary phase is complete after `ui_implementation_gate.md` is admitted.

After H15:

```text
docs freeze
  -> implementation PRs
  -> small scoped code changes
  -> tests before expansion
```
