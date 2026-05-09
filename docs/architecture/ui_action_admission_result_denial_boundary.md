# Semantic UI Action Admission Result and Denial Trace Boundary

Status: Draft
Track: POST-UI / I-series
Purpose: define the boundary for future action admission results and denial traces before admitted actions or dispatchers exist

## 1. Goal

This document defines the future boundary for Semantic UI action admission results and denial traces.

An action admission result is the future decision record produced after evaluating an action admission descriptor.

A denial trace is the future visible explanation for a denied action candidate.

This document does not implement admission.
It does not implement denial traces.
It does not create admitted actions.
It does not dispatch actions.
It does not request effects.
It does not call the VM.
It does not call Host ABI.
It does not mutate runtime state.

## 2. Position in the ladder

The current implemented scaffold ends at:

```text
InteractionActionBindingTraceReport
  -> InteractionActionAdmissionDescriptor
```

The next future boundary is:

```text
InteractionActionAdmissionDescriptor
  -> future InteractionActionAdmissionResult
  -> future InteractionActionDenialTrace
  -> future AdmittedSemanticUiAction
```

Only the result/denial boundary is defined here.

## 3. Core separation

```text
descriptor is not result
result is not execution
admitted result is not dispatched action
denied result is not hidden no-op
denial trace is not audit authority by itself
action is not effect
effect requires separate admission
```

## 4. Admission result shape

A future admission result must distinguish at minimum:

```text
admitted
denied_missing_target
denied_lifecycle
denied_capability
denied_policy
denied_effect_boundary
denied_unknown
```

The result must preserve:

1. descriptor identity;
2. action name;
3. source intent;
4. binding identity;
5. decision status;
6. denial reason if denied;
7. trace requirement;
8. effect relationship;
9. policy gate namespace.

## 5. Denial trace requirement

Denied action candidates must be visible when the denial affects user expectation or semantic UI state.

A future denial trace must record:

1. denied action candidate;
2. source intent kind;
3. descriptor id;
4. denial reason;
5. missing requirement class;
6. target requirement status;
7. lifecycle requirement status;
8. capability requirement status;
9. policy gate namespace;
10. whether retry may be meaningful.

Denied candidates must not silently disappear.

## 6. Denial reason classes

Future denial reasons should be explicit:

| Reason                   | Meaning                                                   |
| ------------------------ | --------------------------------------------------------- |
| `MissingTarget`          | required target is absent                                 |
| `InvalidTargetOwnership` | target exists but is not admissible                       |
| `LifecycleBlocked`       | lifecycle requirement is not satisfied                    |
| `CapabilityMissing`      | required UI capability is absent                          |
| `PolicyDenied`           | policy gate refuses admission                             |
| `EffectBoundaryRequired` | candidate implies effect path requiring separate boundary |
| `Unknown`                | denial reason is known to be unresolved                   |

## 7. Admitted result is not execution

An admitted result only means:

```text
descriptor checks passed
  -> admitted semantic UI action may be constructed later
```

It does not mean:

```text
dispatch now
execute now
request effect now
call VM now
mutate runtime now
```

## 8. Relationship to descriptor boundary

The descriptor boundary is defined in:

```text
docs/architecture/ui_action_admission_descriptor_boundary.md
```

Descriptor:

```text
describes required checks
```

Result:

```text
records future decision outcome
```

Denial trace:

```text
makes denied outcome visible
```

These must stay separate.

## 9. Relationship to future admitted action

A future admitted action object may only be created after an admitted result.

Required future order:

```text
ActionAdmissionDescriptor
  -> ActionAdmissionResult
  -> AdmittedSemanticUiAction
  -> Dispatcher
```

No PR should combine all of these.

## 10. Effect relationship

A successful admission result does not authorize effects.

Required future order:

```text
admitted semantic UI action
  -> optional effect request
  -> effect/capability admission
  -> effect execution
```

No result may silently produce external effects.

## 11. Forbidden shortcuts

Future PRs must not:

* treat descriptor as result;
* treat admitted result as dispatched action;
* hide denied result as no-op;
* create admitted action without result;
* create dispatcher in result PR;
* create effect request in result PR;
* collapse denial trace into logging only;
* let renderer affordance define admission result;
* let Workbench command define admission result;
* call VM/Host ABI from result or denial boundary;
* mutate runtime state from result construction.

## 12. Required implementation order

Future implementation must proceed in separate PRs:

```text
docs result/denial boundary
  -> admission result enum scaffold
  -> denial trace scaffold
  -> admitted action object scaffold
  -> dispatcher scaffold
  -> effect request bridge
```

## 14. Admitted semantic action object dependency

Admitted semantic action object boundary is defined separately in:

```text
docs/architecture/ui_admitted_semantic_action_boundary.md
```

The result layer stops before action object construction.

```text
InteractionActionAdmissionResult::Admitted
  -> future AdmittedSemanticUiAction
```

Admitted result is not dispatched action.
Denied result must not construct admitted action.
Action object is not effect.

## 15. Validation expectation

Docs-only PR validation:

```text
git diff --check
```

No Rust tests are required for this PR.
