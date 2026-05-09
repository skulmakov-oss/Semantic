# Semantic UI Capability Admission Boundary

Status: Draft
Track: POST-UI / I-series
Purpose: define the boundary for future UI capability admission before runtime capability mapping, prepared effects, or committed effects exist

## 1. Goal

This document defines the boundary for future Semantic UI capability admission.

UI capability admission is the future decision layer that determines whether an
effect request descriptor may proceed toward runtime capability mapping and
prepared effect construction.

It is not runtime capability grant.
It is not prepared effect construction.
It is not committed effect execution.
It is not a VM operation.
It is not a Host ABI operation.
It must not mutate runtime state.

## 2. Position in the ladder

The current implemented scaffold reaches:

```text
InteractionEffectRequestDescriptor
  -> InteractionEffectRequestTraceReport
  -> InteractionEffectRequestSummary
```

The next future boundary is:

```text
InteractionEffectRequestDescriptor / InteractionEffectRequestTraceReport
  -> future UiCapabilityAdmissionDescriptor
  -> future UiCapabilityAdmissionResult
  -> future RuntimeCapabilityMapping
  -> future PreparedEffect
  -> future CommitBoundary
  -> future CommittedEffect
```

Only the UI capability admission boundary is defined here.

## 3. Core separation

```text
effect request descriptor is not capability admission
effect request trace is not capability admission
effect request summary is not capability admission
declared UI capability is not capability grant
UI capability admission is not runtime capability grant
runtime capability mapping requires separate boundary
capability admission is not prepared effect
prepared effect is not committed effect
committed effect requires separate boundary
```

## 4. Admission input rule

A future UI capability admission layer may only consume explicit effect request metadata:

```text
InteractionEffectRequestDescriptor
```

or a future narrowed equivalent derived from it.

It must not consume directly:

* raw input;
* interaction intent;
* action binding trace;
* action candidate summary;
* admitted action object;
* dispatch route descriptor;
* dispatch summary alone;
* renderer affordance;
* Workbench command;
* component callback.

## 5. Required future admission descriptor fields

A future UI capability admission descriptor must preserve:

1. effect request descriptor id;
2. source admitted action id;
3. dispatch record id;
4. dispatch route id;
5. requested effect kind;
6. declared UI capability;
7. declared runtime capability requirement;
8. lifecycle precondition;
9. target policy;
10. denial behavior;
11. trace requirement;
12. policy gate namespace;
13. scope;
14. runtime mapping requirement;
15. future admission result shape.

## 6. Required future admission result statuses

A future UI capability admission result must distinguish at minimum:

```text
admitted
denied_missing_ui_capability
denied_lifecycle
denied_target
denied_policy
denied_runtime_mapping_required
denied_unknown
```

An admitted result still does not grant runtime capability by itself.

## 7. UI capability declaration vs admission

Effect request descriptors may declare required UI capability.

Declaration is not admission.

```text
required_ui_capability
  -> UI capability admission
  -> UI capability admission result
```

A descriptor field must never be treated as permission.

## 8. UI capability vs runtime capability

UI capability admission is still not runtime capability grant.

Required future order:

```text
UI capability admission result
  -> runtime capability mapping descriptor
  -> runtime capability mapping admission
  -> prepared effect
```

Runtime capability mapping must be explicit.

No UI capability admission PR may silently map to Host ABI or runtime authority.

## 9. Prepared effect separation

UI capability admission is not prepared effect construction.

Required future order:

```text
EffectRequestDescriptor
  -> UI capability admission
  -> runtime capability mapping
  -> PreparedEffect
```

Prepared effect remains inert until commit boundary.

## 10. Commit separation

Prepared effect is not committed effect.

Required future order:

```text
PreparedEffect
  -> CommitBoundary
  -> CommittedEffect
```

No capability admission PR may collapse admission, preparation, and commit.

## 11. Denial and trace requirements

Denied UI capability admission must be visible when it affects user expectation or semantic UI state.

Future denial reasons may include:

* missing UI capability;
* lifecycle blocked;
* target unavailable;
* target invalid;
* policy denied;
* runtime mapping required;
* unknown denial.

Denied capability admission must not become hidden no-op.

## 12. Workbench and renderer relationship

Workbench may display future capability admission data only through explicit consumption boundaries.

Renderer may display capability state only as presentation.

Neither Workbench nor renderer may define:

* capability grant;
* runtime authority;
* effect permission;
* prepare authority;
* commit authority;
* audit finality.

## 13. Forbidden shortcuts

Future PRs must not:

* treat declared UI capability as admitted capability;
* treat effect request summary as admission input by itself;
* admit capability from renderer affordance;
* admit capability from Workbench command;
* map UI capability to runtime capability implicitly;
* create prepared effect in capability admission PR;
* create committed effect in capability admission PR;
* call VM/Host ABI from capability admission;
* mutate runtime state from capability admission;
* hide denied capability admission as no-op;
* collapse capability admission, runtime mapping, prepare, and commit into one PR.

## 14. Required implementation order

Future implementation must proceed in separate PRs:

```text
docs UI capability admission boundary
  -> UI capability admission descriptor scaffold
  -> UI capability admission result scaffold
  -> UI capability denial trace scaffold
  -> runtime capability mapping boundary docs
  -> runtime capability mapping descriptor scaffold
  -> prepared effect boundary docs
  -> prepared effect scaffold
  -> commit boundary docs
  -> committed effect scaffold
```

No PR should combine capability admission, runtime mapping, prepared effect, commit, and runtime mutation.

## 15. Relationship to effect request descriptor boundary

The effect request descriptor boundary is defined in:

```text
docs/architecture/ui_effect_request_descriptor_boundary.md
```

This document starts the next stage after descriptor/trace/summary scaffolding.

## 16. Relationship to H8 effect/capability boundary

The general effect request and capability boundary is defined in:

```text
docs/architecture/ui_effect_request_capability_boundary.md
```

This document narrows the I-series UI capability admission step.

## 17. Validation expectation

Docs-only PR validation:

```text
git diff --check
```

No Rust tests are required for this PR.
