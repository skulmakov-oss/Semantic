# Semantic UI Runtime Capability Mapping Boundary

Status: Draft  
Track: POST-UI / I-series  
Purpose: define the boundary for future runtime capability mapping before prepared effects, committed effects, VM/Host ABI calls, or runtime mutation exist

## 1. Goal

This document defines the boundary for future runtime capability mapping.

Runtime capability mapping is the future layer that may translate an admitted UI
capability admission result into an explicit runtime capability mapping
descriptor.

It is not runtime capability grant.
It is not a Host ABI call.
It is not a VM operation.
It is not a prepared effect.
It is not a committed effect.
It must not mutate runtime state.

## 2. Position in the ladder

The current implemented scaffold reaches:

```text
InteractionUiCapabilityAdmissionDescriptor
  -> InteractionUiCapabilityAdmissionResult
  -> InteractionUiCapabilityDenialTrace
```

The next future boundary is:

```text
InteractionUiCapabilityAdmissionResult::Admitted
  -> future RuntimeCapabilityMappingDescriptor
  -> future RuntimeCapabilityMappingResult
  -> future PreparedEffect
  -> future CommitBoundary
  -> future CommittedEffect
```

Only the runtime capability mapping boundary is defined here.

## 3. Core separation

```text
UI capability admission result is not runtime capability grant
UI capability denial trace is not runtime mapping
runtime mapping descriptor is not runtime capability grant
runtime mapping result is not Host ABI authority
runtime mapping result is not prepared effect
prepared effect is not committed effect
committed effect requires separate boundary
```

## 4. Mapping input rule

A future runtime capability mapping layer may only consume:

```text
InteractionUiCapabilityAdmissionResult::Admitted
```

or a future narrowed equivalent derived from it.

It must not consume directly:

* raw input;
* interaction intent;
* effect request descriptor;
* effect request trace;
* effect request summary;
* UI capability admission descriptor alone;
* denied UI capability admission result;
* UI capability denial trace;
* renderer affordance;
* Workbench command;
* component callback.

## 5. Required future mapping descriptor fields

A future runtime capability mapping descriptor must preserve:

1. UI capability admission result id;
2. UI capability admission descriptor id;
3. effect request descriptor id;
4. source admitted action id;
5. dispatch record id;
6. dispatch route id;
7. requested effect kind;
8. declared UI capability;
9. declared runtime capability requirement;
10. runtime mapping requirement;
11. lifecycle precondition;
12. target policy;
13. trace requirement;
14. policy gate namespace;
15. scope;
16. runtime capability namespace;
17. future mapping result shape.

## 6. Required future mapping result statuses

A future runtime capability mapping result must distinguish at minimum:

```text
mapped
denied_missing_runtime_capability
denied_lifecycle
denied_target
denied_policy
denied_host_boundary
denied_unknown
```

A mapped result still does not execute Host ABI.
A mapped result still does not construct prepared effect by itself.

## 7. Runtime capability declaration vs mapping

Declared runtime capability requirement is not runtime capability mapping.

```text
declared_runtime_capability_requirement
  -> RuntimeCapabilityMappingDescriptor
  -> RuntimeCapabilityMappingResult
```

A descriptor field must never be treated as runtime authority.

## 8. Runtime mapping vs Host ABI

Runtime capability mapping is not a Host ABI call.

Required future order:

```text
RuntimeCapabilityMappingResult::Mapped
  -> PreparedEffectDescriptor
  -> PreparedEffect
  -> CommitBoundary
  -> Host ABI / runtime effect path if admitted
```

No runtime mapping PR may call Host ABI.

## 9. Runtime mapping vs VM

Runtime capability mapping is not a VM operation.

The UI layer must not call VM directly during runtime capability mapping.

Any future VM relationship must be explicit and separately bounded.

## 10. Prepared effect separation

Runtime capability mapping is not prepared effect construction.

Required future order:

```text
RuntimeCapabilityMappingResult::Mapped
  -> PreparedEffectBoundary
  -> PreparedEffect
```

Prepared effect remains inert until commit boundary.

## 11. Commit separation

Prepared effect is not committed effect.

Required future order:

```text
PreparedEffect
  -> CommitBoundary
  -> CommittedEffect
```

Runtime mapping must not collapse mapping, prepare, and commit.

## 12. Denial and trace requirements

Denied runtime mapping must be visible when it affects user expectation or semantic UI state.

Future denial reasons may include:

* missing runtime capability;
* lifecycle blocked;
* target unavailable;
* target invalid;
* policy denied;
* host boundary denied;
* unknown denial.

Denied runtime mapping must not become hidden no-op.

## 13. Workbench and renderer relationship

Workbench may display future runtime mapping data only through explicit consumption boundaries.

Renderer may display mapping state only as presentation.

Neither Workbench nor renderer may define:

* runtime capability grant;
* Host ABI authority;
* effect permission;
* prepare authority;
* commit authority;
* audit finality.

## 14. Forbidden shortcuts

Future PRs must not:

* treat UI capability admission result as runtime capability grant;
* treat runtime mapping descriptor as mapped runtime capability;
* consume denied UI capability admission result for mapping;
* consume UI capability denial trace for mapping;
* map runtime capability from renderer affordance;
* map runtime capability from Workbench command;
* call VM/Host ABI from runtime mapping;
* create prepared effect in runtime mapping PR;
* create committed effect in runtime mapping PR;
* mutate runtime state from runtime mapping;
* collapse runtime mapping, prepared effect, commit, and runtime mutation into one PR.

## 15. Required implementation order

Future implementation must proceed in separate PRs:

```text
docs runtime capability mapping boundary
  -> runtime capability mapping descriptor scaffold
  -> runtime capability mapping result scaffold
  -> runtime capability mapping denial trace scaffold if needed
  -> prepared effect boundary docs
  -> prepared effect scaffold
  -> commit boundary docs
  -> committed effect scaffold
```

No PR should combine runtime mapping, prepared effect, commit, Host ABI, and runtime mutation.

## 16. Relationship to UI capability admission boundary

The UI capability admission boundary is defined in:

```text
docs/architecture/ui_capability_admission_boundary.md
```

This document starts the next stage after UI capability admission result and denial trace scaffolding.

## 17. Relationship to H8 effect/capability boundary

The general effect request and capability boundary is defined in:

```text
docs/architecture/ui_effect_request_capability_boundary.md
```

This document narrows the I-series runtime capability mapping step.

## 18. Validation expectation

Docs-only PR validation:

```text
git diff --check
```

No Rust tests are required for this PR.
