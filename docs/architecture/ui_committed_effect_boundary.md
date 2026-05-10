# Semantic UI Committed Effect Boundary

Status: Draft  
Track: POST-UI / I-series  
Purpose: define the boundary for future committed effects before Host ABI calls, VM calls, effect execution, or runtime mutation exist

## 1. Goal

This document defines the boundary for future Semantic UI committed effects.

A committed effect is a future explicit post-commit record derived from a prepared
effect result that has passed the required commit boundary.

It is not arbitrary Host ABI execution.
It is not VM authority.
It is not unbounded runtime mutation.
It is not renderer authority.
It is not Workbench command execution.

Any future runtime mutation must be explicit, capability-gated, auditable, and
separately bounded.

## 2. Position in the ladder

The current implemented scaffold reaches:

```text
InteractionPreparedEffectDescriptor
  -> InteractionPreparedEffectResult
```

The next future boundary is:

```text
InteractionPreparedEffectResult::Prepared
  -> future CommitBoundary
  -> future CommittedEffect
  -> future HostRuntimeEffectPath if separately admitted
```

Only the committed effect boundary is defined here.

## 3. Core separation

```text
prepared effect result is not commit boundary
prepared effect denial is not commit boundary
commit boundary is not committed effect by itself
committed effect is not arbitrary Host ABI authority
committed effect is not VM authority
committed effect is not unbounded runtime mutation
committed effect requires explicit audit visibility
Host ABI path requires separate boundary
```

## 4. Commit input rule

A future commit boundary may only consume:

```text
InteractionPreparedEffectResult::Prepared
```

or a future narrowed equivalent derived from it.

It must not consume directly:

* raw input;
* interaction intent;
* effect request descriptor;
* effect request trace;
* effect request summary;
* UI capability admission descriptor;
* UI capability admission result alone;
* UI capability denial trace;
* runtime capability mapping descriptor;
* runtime capability mapping result alone if denied;
* prepared effect descriptor alone;
* denied prepared effect result;
* renderer affordance;
* Workbench command;
* component callback.

## 5. Required future commit boundary descriptor fields

A future commit boundary descriptor must preserve:

1. prepared effect result id;
2. prepared effect descriptor id;
3. runtime capability mapping result id;
4. runtime capability mapping descriptor id;
5. UI capability admission result id;
6. UI capability admission descriptor id;
7. effect request descriptor id;
8. source admitted action id;
9. dispatch record id;
10. dispatch route id;
11. requested effect kind;
12. declared UI capability;
13. declared runtime capability requirement;
14. runtime capability namespace;
15. lifecycle precondition;
16. target policy;
17. trace requirement;
18. policy gate namespace;
19. scope;
20. audit requirement;
21. future committed effect shape.

## 6. Required future commit statuses

A future commit boundary result must distinguish at minimum:

```text
committed
denied_missing_prepared_effect
denied_lifecycle
denied_target
denied_policy
denied_audit_required
denied_host_boundary
denied_unknown
```

A committed result still does not mean arbitrary Host ABI access.

## 7. Committed effect vs Host ABI

Committed effect is not arbitrary Host ABI execution.

Required future order:

```text
CommittedEffect
  -> future HostRuntimeEffectBoundary
  -> future HostRuntimeEffectPath
```

No committed effect PR may directly add Host ABI calls unless a separate Host
runtime effect boundary already exists and explicitly admits that path.

## 8. Committed effect vs VM

Committed effect is not VM authority.

The UI layer must not call VM directly during commit.

Any future VM relationship must be explicit and separately bounded.

## 9. Committed effect vs runtime mutation

Committed effect is not unbounded runtime mutation.

A committed effect may represent that an effect has passed the UI-side commit
boundary, but actual runtime mutation must remain capability-gated, auditable,
and separately controlled.

## 10. Audit requirement

Committed effects must be visible to audit.

Future committed effect records must preserve:

* source action identity;
* effect request identity;
* capability mapping identity;
* prepared effect identity;
* commit decision identity;
* denial reason if denied;
* target policy;
* runtime capability namespace.

No committed effect should become hidden mutation.

## 11. Workbench and renderer relationship

Workbench may display future committed effect data only through explicit
consumption boundaries.

Renderer may display committed effect state only as presentation.

Neither Workbench nor renderer may define:

* commit authority;
* Host ABI authority;
* VM authority;
* effect execution;
* runtime mutation;
* audit finality.

## 12. Forbidden shortcuts

Future PRs must not:

* treat prepared effect result as committed effect;
* consume denied prepared effect result for commit;
* create committed effect from renderer affordance;
* create committed effect from Workbench command;
* call VM/Host ABI from commit boundary docs/scaffold PR;
* mutate runtime state from committed effect scaffold;
* collapse committed effect, Host ABI, audit backend, and runtime mutation into one PR;
* hide denied commit as no-op;
* bypass audit visibility.

## 13. Required implementation order

Future implementation must proceed in separate PRs:

```text
docs committed effect boundary
  -> commit boundary descriptor scaffold
  -> commit boundary result scaffold
  -> committed effect descriptor scaffold
  -> committed effect record scaffold
  -> committed effect denial trace scaffold if needed
  -> Host runtime effect boundary docs
  -> Host runtime effect scaffold
```

No PR should combine commit, Host ABI, VM, audit backend, and runtime mutation.

## 14. Relationship to prepared effect boundary

The prepared effect boundary is defined in:

```text
docs/architecture/ui_prepared_effect_boundary.md
```

This document starts the next stage after prepared effect result scaffolding.

## 15. Relationship to H8 effect/capability boundary

The general effect request and capability boundary is defined in:

```text
docs/architecture/ui_effect_request_capability_boundary.md
```

This document narrows the I-series committed effect step.

## 16. Validation expectation

Docs-only PR validation:

```text
git diff --check
```

No Rust tests are required for this PR.

## 17. Relationship to full effect trace ladder

The full UI-side effect trace ladder is documented in:

```text
docs/architecture/ui_full_effect_trace_ladder.md
```

The committed effect boundary is the final currently documented UI-side effect stage:

```text
InteractionCommitBoundaryResult::Committed
  -> InteractionCommittedEffectDescriptor
  -> InteractionCommittedEffectRecord
```

The committed effect record is still not Host ABI authority, VM authority, effect execution, runtime mutation, audit backend, or host runtime path.

## 18. Relationship to host runtime effect boundary

The host runtime effect boundary is defined separately in:

```text
docs/architecture/ui_host_runtime_effect_boundary.md
```

The committed effect record remains inert until a future host runtime effect
boundary admits it.

```text
InteractionCommittedEffectRecord
  -> future HostRuntimeEffectBoundary
  -> future HostRuntimeEffectPath
```

Host runtime effect boundary is not Host ABI execution.
Host runtime effect path is not unbounded runtime mutation.
Audit backend writes require a separate boundary.
