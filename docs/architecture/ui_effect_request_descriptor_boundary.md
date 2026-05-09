# Semantic UI Effect Request Descriptor Boundary

Status: Draft
Track: POST-UI / I-series
Purpose: define the boundary for future effect request descriptors before capability admission, prepared effects, or committed effects exist

## 1. Goal

This document defines the boundary for future Semantic UI effect request descriptors.

An effect request descriptor is an inert description of a requested effect path derived from admitted semantic UI action and dispatch metadata.

It is not effect execution.
It is not capability admission.
It is not a prepared effect.
It is not a committed effect.
It is not a VM operation.
It is not a Host ABI operation.
It must not mutate runtime state.

## 2. Position in the ladder

The current implemented scaffold reaches:

```text
InteractionAdmittedSemanticAction
  -> InteractionSemanticActionDispatchRouteDescriptor
  -> InteractionSemanticActionDispatchRecord
  -> InteractionSemanticActionDispatchTraceReport
  -> InteractionSemanticActionDispatchSummary
```

The next future boundary is:

```text
InteractionSemanticActionDispatchTraceReport / DispatchSummary
  -> future EffectRequestDescriptor
  -> future EffectCapabilityAdmission
  -> future PreparedEffect
  -> future CommitBoundary
  -> future CommittedEffect
```

Only the effect request descriptor boundary is defined here.

## 3. Core separation

```text
dispatch trace is not effect request
dispatch summary is not effect request
effect request descriptor is not capability admission
capability admission is not prepared effect
prepared effect is not committed effect
committed effect requires separate boundary
```

## 4. Descriptor input rule

A future effect request descriptor may only be derived from dispatch metadata that explicitly indicates an effect path.

Allowed input candidates:

```text
InteractionSemanticActionDispatchTraceReport
where effect_eligibility == RequiresFutureEffectBoundary
```

or a future narrowed equivalent.

It must not be derived directly from:

* raw input;
* interaction intent;
* binding descriptor;
* binding trace;
* candidate summary;
* admission descriptor;
* admission result alone;
* denial trace;
* admitted action object alone;
* renderer affordance;
* Workbench command;
* component callback.

## 5. Required descriptor fields

A future effect request descriptor must preserve:

1. source admitted action identity;
2. dispatch record identity;
3. dispatch route identity;
4. action name;
5. source intent kind;
6. requested effect kind;
7. target identity / target policy;
8. required UI capability;
9. required runtime capability mapping if any;
10. lifecycle preconditions;
11. trace requirement;
12. denial behavior;
13. prepare/commit relationship;
14. policy gate namespace;
15. effect request scope.

It must not include executable callback handles.

## 6. Effect request kind classes

Future effect request kinds may include:

| Kind               | Meaning                                                  |
| ------------------ | -------------------------------------------------------- |
| `PrepareEffect`    | prepare a controlled effect without committing it        |
| `CommitEffect`     | request commit of a previously prepared effect           |
| `RollbackEffect`   | request rollback through explicit rollback boundary      |
| `CloseWindow`      | request platform/window lifecycle effect                 |
| `QuarantineTarget` | request target quarantine through policy/effect boundary |
| `Unknown`          | unresolved effect request kind                           |

These are descriptor-level classes only.

## 7. Capability separation

A descriptor may declare required UI and runtime capabilities.

It must not grant them.

Required future order:

```text
EffectRequestDescriptor
  -> UI capability admission
  -> runtime capability mapping if admitted
  -> prepared effect
```

UI capability is not runtime capability by default.

Runtime capability mapping must be explicit.

## 8. Prepared effect separation

An effect request descriptor is not a prepared effect.

Required future order:

```text
EffectRequestDescriptor
  -> capability/effect admission
  -> PreparedEffect
```

A prepared effect is still not committed.

Prepared effect may later be:

* committed;
* denied at commit;
* cancelled;
* expired;
* rolled back if supported;
* quarantined;
* superseded.

## 9. Commit separation

A committed effect may only exist after a separate commit boundary.

Required future order:

```text
PreparedEffect
  -> CommitBoundary
  -> CommittedEffect
```

No descriptor PR may collapse prepare and commit into a hidden one-step mutation.

## 10. Denial and trace requirements

Denied effect request descriptors must not disappear silently.

Future denial may include:

* missing UI capability;
* missing runtime capability;
* lifecycle blocked;
* target invalid;
* target quarantined;
* policy denied;
* prepare boundary denied;
* commit boundary denied;
* unknown denial.

Trace display is not audit authority by itself.

## 11. Workbench and renderer relationship

Workbench may display or request future effect descriptors only through explicit consumption boundaries.

Renderer may display effect request state only as presentation.

Neither Workbench nor renderer may define:

* effect meaning;
* effect permission;
* capability grant;
* runtime mutation;
* commit authority;
* audit finality.

## 12. Forbidden shortcuts

Future PRs must not:

* create effect request descriptor directly from raw input;
* create effect request descriptor directly from component callback;
* create effect request descriptor directly from renderer affordance;
* create effect request descriptor directly from Workbench command;
* treat dispatch summary as effect request;
* treat effect request descriptor as capability admission;
* treat UI capability display as capability grant;
* map UI capability to runtime capability implicitly;
* create prepared effect in descriptor PR;
* create committed effect in descriptor PR;
* call VM/Host ABI from descriptor;
* mutate runtime state from descriptor construction;
* collapse prepare/commit into hidden one-step mutation.

## 13. Required implementation order

Future implementation must proceed in separate PRs:

```text
docs effect request descriptor boundary
  -> effect request descriptor scaffold
  -> effect request trace/summary scaffold if needed
  -> UI capability admission boundary docs
  -> UI capability admission scaffold
  -> runtime capability mapping boundary docs
  -> prepared effect boundary docs
  -> prepared effect scaffold
  -> commit boundary docs
  -> committed effect scaffold
```

No PR should combine descriptor, capability admission, prepared effect, commit, and runtime mutation.

## 14. Relationship to H8 effect/capability boundary

The general effect request and capability boundary is defined in:

```text
docs/architecture/ui_effect_request_capability_boundary.md
```

This document narrows that general boundary for the I-series effect request descriptor step.

## 15. Validation expectation

Docs-only PR validation:

```text
git diff --check
```

No Rust tests are required for this PR.
