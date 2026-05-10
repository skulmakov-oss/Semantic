# Semantic UI Prepared Effect Boundary

Status: Draft  
Track: POST-UI / I-series  
Purpose: define the boundary for future prepared effects before commit, Host ABI calls, VM calls, effect execution, or runtime mutation exist

## 1. Goal

This document defines the boundary for future Semantic UI prepared effects.

A prepared effect is a future inert object representing an effect that has passed
the required UI capability and runtime capability mapping stages, but has not
been committed.

It is not a committed effect.
It is not Host ABI execution.
It is not a VM operation.
It is not effect execution.
It must not mutate runtime state.

## 2. Position in the ladder

The current implemented scaffold reaches:

```text
InteractionRuntimeCapabilityMappingDescriptor
  -> InteractionRuntimeCapabilityMappingResult
```

The next future boundary is:

```text
InteractionRuntimeCapabilityMappingResult::Mapped
  -> future PreparedEffectDescriptor
  -> future PreparedEffect
  -> future CommitBoundary
  -> future CommittedEffect
```

Only the prepared effect boundary is defined here.

## 3. Core separation

```text
runtime capability mapping result is not prepared effect
runtime capability mapping denial is not prepared effect
prepared effect descriptor is not prepared effect execution
prepared effect is not committed effect
prepared effect is not Host ABI authority
prepared effect is not VM authority
prepared effect is not runtime mutation
committed effect requires separate boundary
```

## 4. Prepared effect input rule

A future prepared effect layer may only consume:

```text
InteractionRuntimeCapabilityMappingResult::Mapped
```

or a future narrowed equivalent derived from it.

It must not consume directly:

* raw input;
* interaction intent;
* effect request descriptor;
* effect request trace;
* effect request summary;
* UI capability admission descriptor;
* UI capability admission result alone if denied;
* UI capability denial trace;
* runtime capability mapping descriptor alone;
* denied runtime capability mapping result;
* renderer affordance;
* Workbench command;
* component callback.

## 5. Required future prepared effect descriptor fields

A future prepared effect descriptor must preserve:

1. runtime capability mapping result id;
2. runtime capability mapping descriptor id;
3. UI capability admission result id;
4. UI capability admission descriptor id;
5. effect request descriptor id;
6. source admitted action id;
7. dispatch record id;
8. dispatch route id;
9. requested effect kind;
10. declared UI capability;
11. declared runtime capability requirement;
12. runtime capability namespace;
13. lifecycle precondition;
14. target policy;
15. trace requirement;
16. policy gate namespace;
17. scope;
18. prepare status shape;
19. future commit requirement.

## 6. Required future prepared effect statuses

A future prepared effect result/state must distinguish at minimum:

```text
prepared
denied_missing_mapping
denied_lifecycle
denied_target
denied_policy
denied_unknown
```

A prepared effect still does not execute Host ABI.
A prepared effect still does not mutate runtime state.
A prepared effect still requires commit boundary.

## 7. Prepared effect vs Host ABI

Prepared effect is not a Host ABI call.

Required future order:

```text
PreparedEffect
  -> CommitBoundary
  -> CommittedEffect
  -> Host ABI / runtime effect path if admitted
```

No prepared effect PR may call Host ABI.

## 8. Prepared effect vs VM

Prepared effect is not a VM operation.

The UI layer must not call VM directly during prepared effect construction.

Any future VM relationship must be explicit and separately bounded.

## 9. Prepared effect vs runtime mutation

Prepared effect does not mutate runtime state.

It is an inert pre-commit object.

It may be visible to UI / debug / Workbench only through explicit presentation or consumption boundaries.

## 10. Commit separation

Prepared effect is not committed effect.

Required future order:

```text
PreparedEffect
  -> CommitBoundary
  -> CommittedEffect
```

No prepared effect PR may collapse prepare and commit.

## 11. Denial and trace requirements

Denied prepared effect construction must be visible when it affects user expectation or semantic UI state.

Future denial reasons may include:

* missing runtime mapping;
* lifecycle blocked;
* target unavailable;
* target invalid;
* policy denied;
* unknown denial.

Denied prepared effect construction must not become hidden no-op.

## 12. Workbench and renderer relationship

Workbench may display future prepared effect data only through explicit consumption boundaries.

Renderer may display prepared effect state only as presentation.

Neither Workbench nor renderer may define:

* prepare authority;
* commit authority;
* Host ABI authority;
* VM authority;
* effect execution;
* runtime mutation;
* audit finality.

## 13. Forbidden shortcuts

Future PRs must not:

* treat runtime capability mapping result as prepared effect;
* consume denied runtime mapping result for prepared effect;
* create prepared effect from renderer affordance;
* create prepared effect from Workbench command;
* call VM/Host ABI from prepared effect construction;
* mutate runtime state from prepared effect construction;
* create committed effect in prepared effect PR;
* collapse prepared effect, commit, Host ABI, and runtime mutation into one PR.

## 14. Required implementation order

Future implementation must proceed in separate PRs:

```text
docs prepared effect boundary
  -> prepared effect descriptor scaffold
  -> prepared effect scaffold/result scaffold
  -> prepared effect denial trace scaffold if needed
  -> commit boundary docs
  -> committed effect scaffold
```

No PR should combine prepared effect, commit, Host ABI, VM, and runtime mutation.

## 15. Relationship to runtime capability mapping boundary

The runtime capability mapping boundary is defined in:

```text
docs/architecture/ui_runtime_capability_mapping_boundary.md
```

This document starts the next stage after runtime capability mapping result scaffolding.

## 16. Relationship to H8 effect/capability boundary

The general effect request and capability boundary is defined in:

```text
docs/architecture/ui_effect_request_capability_boundary.md
```

This document narrows the I-series prepared effect step.

## Committed effect dependency

Committed effect boundary is defined separately in:

```text
docs/architecture/ui_committed_effect_boundary.md
```

The prepared effect layer stops before commit and committed effect construction.

```text
InteractionPreparedEffectResult::Prepared
  -> future CommitBoundary
  -> future CommittedEffect
```

Prepared effect result is not commit boundary.
Commit boundary is not Host ABI authority.
Committed effect still requires explicit audit/runtime boundaries.

## 17. Validation expectation

Docs-only PR validation:

```text
git diff --check
```

No Rust tests are required for this PR.
