# Semantic UI Host Runtime Effect Boundary

Status: Draft  
Track: POST-UI / I-series  
Purpose: define the boundary between inert UI-side committed effect records and any future host/runtime effect path

## 1. Goal

This document defines the future boundary between Semantic UI committed effect
records and host/runtime effect paths.

The current UI ladder stops at:

```text
InteractionCommittedEffectRecord
```

A committed effect record is still inert UI-side metadata.

It is not Host ABI authority.
It is not VM authority.
It is not effect execution.
It is not runtime mutation.
It is not an audit backend write.
It is not a host runtime effect path.

## 2. Future position in the ladder

The future host runtime effect stage may only begin after:

```text
InteractionCommittedEffectRecord
  -> future HostRuntimeEffectBoundary
  -> future HostRuntimeEffectPath
```

Only the boundary is defined here.

No implementation exists in this PR.

## 3. Core separation

```text
committed effect record is not Host ABI authority
committed effect record is not runtime mutation
host runtime effect boundary is not Host ABI execution by itself
host runtime effect boundary is not VM authority
host runtime effect path is not unbounded runtime mutation
runtime mutation requires explicit capability-gated boundary
audit backend writes require separate boundary
VM calls require separate boundary
```

## 4. Host runtime effect input rule

A future host runtime effect boundary may only consume:

```text
InteractionCommittedEffectRecord
```

or a future narrowed equivalent derived from it.

It must not consume directly:

* raw input;
* interaction intent;
* action binding;
* action admission descriptor;
* admitted action;
* dispatch route;
* dispatch record;
* effect request descriptor;
* UI capability admission descriptor;
* UI capability admission result;
* runtime capability mapping descriptor;
* runtime capability mapping result;
* prepared effect descriptor;
* prepared effect result;
* commit boundary descriptor;
* denied commit boundary result;
* committed effect descriptor alone;
* renderer affordance;
* Workbench command;
* component callback.

## 5. Required future host boundary descriptor fields

A future host runtime effect boundary descriptor must preserve:

1. committed effect record id;
2. committed effect descriptor id;
3. commit boundary result id;
4. commit boundary descriptor id;
5. prepared effect result id;
6. prepared effect descriptor id;
7. runtime capability mapping result id;
8. runtime capability mapping descriptor id;
9. UI capability admission result id;
10. UI capability admission descriptor id;
11. effect request descriptor id;
12. source admitted action id;
13. dispatch record id;
14. dispatch route id;
15. requested effect kind;
16. declared UI capability;
17. declared runtime capability requirement;
18. runtime capability namespace;
19. lifecycle precondition;
20. target policy;
21. trace requirement;
22. policy gate namespace;
23. scope;
24. audit requirement;
25. audit visibility;
26. runtime mutation requirement;
27. host path requirement;
28. host boundary decision shape.

## 6. Required future host boundary statuses

A future host runtime effect boundary result must distinguish at minimum:

```text
admitted_to_host_path
denied_missing_committed_record
denied_missing_runtime_capability
denied_lifecycle
denied_target
denied_policy
denied_audit_required
denied_host_boundary
denied_unknown
```

An admitted host boundary result still does not imply unbounded mutation.

## 7. Host ABI separation

Host runtime effect boundary is not a Host ABI call.

Required future order:

```text
HostRuntimeEffectBoundaryResult::Admitted
  -> future HostRuntimeEffectPathDescriptor
  -> future HostRuntimeEffectPath
  -> future HostAbiCall if separately bounded
```

No host runtime boundary PR may call Host ABI directly.

## 8. VM separation

Host runtime effect boundary is not VM authority.

The UI layer must not call VM directly from the host runtime effect boundary.

Any future VM path must be explicit and separately bounded.

## 9. Runtime mutation separation

Host runtime effect path is not unbounded runtime mutation.

Runtime mutation must be:

* capability-gated;
* policy-gated;
* auditable;
* traceable to committed effect record;
* bounded by explicit runtime mutation rules.

## 10. Audit separation

Host runtime effect boundary is not audit backend implementation.

A future host runtime effect boundary may require audit visibility metadata, but
writing to an audit backend requires a separate boundary.

No host runtime effect work may become hidden mutation.

## 11. Workbench and renderer relationship

Workbench may display host runtime effect boundary metadata only through explicit
read-only consumption boundaries.

Renderer may display host runtime state only as presentation.

Neither Workbench nor renderer may define:

* Host ABI authority;
* VM authority;
* effect execution authority;
* runtime mutation authority;
* audit backend authority;
* host runtime effect path admission.

## 12. Forbidden shortcuts

Future PRs must not:

* treat committed effect record as Host ABI authority;
* treat committed effect record as runtime mutation;
* create HostRuntimeEffectPath directly from Workbench command;
* create HostRuntimeEffectPath directly from renderer affordance;
* call VM from host runtime effect boundary scaffold;
* call Host ABI from host runtime effect boundary scaffold;
* mutate runtime state from host runtime effect boundary scaffold;
* write audit backend records from host runtime effect boundary scaffold;
* collapse host boundary, Host ABI, runtime mutation, and audit backend into one PR;
* hide denied host runtime effect as no-op.

## 13. Required implementation order

Future implementation must proceed in separate PRs:

```text
docs host runtime effect boundary
  -> host runtime effect boundary descriptor scaffold
  -> host runtime effect boundary result scaffold
  -> host runtime effect denial trace scaffold if needed
  -> host runtime effect path descriptor scaffold
  -> Host ABI boundary docs
  -> runtime mutation boundary docs
  -> audit backend boundary docs
```

No PR should combine host boundary, Host ABI, VM, runtime mutation, and audit backend.

## 14. Relationship to full effect trace ladder

The full UI-side effect trace ladder is defined in:

```text
docs/architecture/ui_full_effect_trace_ladder.md
```

That ladder stops at:

```text
InteractionCommittedEffectRecord
```

This document defines the next boundary after that stop point.

## 15. Relationship to committed effect boundary

The committed effect boundary is defined in:

```text
docs/architecture/ui_committed_effect_boundary.md
```

That document states that Host runtime effect path requires a separate boundary.

This document is that boundary definition.

## 16. Validation expectation

Docs-only PR validation:

```text
git diff --check
```

No Rust tests are required for this PR.
