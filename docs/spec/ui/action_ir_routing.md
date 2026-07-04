# Action IR and ActionIntent Routing

Status: draft spec
Track: POST-UI / Intent-Driven Projection
Scope type: documentation only
Depends on:
- docs/dna/SEMANTIC_UI_DNA.md
- docs/dna/SEMANTIC_UI_DNA_v2.md
- docs/roadmap/post_ui/intent_driven_projection_roadmap.md
- docs/spec/ui/projection_source_model.md
- docs/spec/ui/ui_ir_schema.md
Related:
- #1310
- #1328
- #1329
- #1330
- #1331
- #1332

Action IR and ActionIntent routing define the upward flow from projected UI affordances into Semantic admission.

They do not define semantic truth, verifier rules, VM behavior, runtime authority, renderer behavior, host effects, or production UI wiring.

This document does not implement Action IR, ActionIntent, event routing, runtime queues, shell behavior, or admission logic.

## 1. Purpose

Action IR exists to prevent the bad workflow:

```text
UI click -> direct runtime mutation
```

The intended flow is:

```text
raw UI event stays local
projected affordance maps to Action IR
ActionIntent candidate is formed
Semantic admission evaluates it
patch / evidence result returns to projection
shell shows the result
```

```text
UI proposes.
Semantic disposes.
Shell shows.
```

Action IR and ActionIntent routing turn projection affordances into structured proposals instead of direct effects.

## 2. Event Boundary

Raw UI events stay local.

The event boundary includes:

- pointer events stay local;
- keyboard events stay local;
- focus traversal stays local unless it produces a structured intent;
- hover / drag / scroll / text composition remain shell-side until converted into an explicit ActionIntent candidate;
- no raw UI event may directly mutate Semantic state.

```text
Raw events are presentation input.
ActionIntent is semantic-facing proposal.
```

Raw events are interpreted for presentation and routing, not for semantic mutation.

## 3. Action IR

Action IR is the compiled action routing contract derived from projection source and UI IR.

Action IR should cover:

- action id;
- source `ActionOffer` reference;
- role: `SafeAction` / `GuardedAction` / `DangerAction`;
- target surface id;
- target node id;
- expected argument shape;
- required source revisions;
- capability requirement;
- confirmation requirement;
- repeat policy;
- stream policy if applicable;
- denial / recovery route references.

```text
Action IR routes affordances.
Action IR does not invent actions.
Action IR does not grant authority.
```

Action IR names the route and the contract for an action affordance.

## 4. ActionIntent Envelope

The future ActionIntent envelope is a structured proposal carrying enough context for admission and traceability.

Draft fields include:

- `intent_id`
- `projection_id`
- `surface_id`
- `node_id`
- `action_id`
- `action_offer_ref`
- `actor_ref`
- `session_ref`
- `client_ref`
- `args`
- `source_state_rev`
- `source_task_rev`
- `source_projection_rev`
- `sequence_no`
- `idempotency_key`
- `issued_at`
- `freshness_ref`
- `capability_ref`
- `evidence_ref`

```text
ActionIntent is a proposed action candidate, not an admitted action.
```

ActionIntent is the envelope the shell sends upward after an affordance is activated.

## 5. Source Revision Requirement

Source revisions matter because admission must be evaluated against the right state.

Relevant revision fields include:

- `source_state_rev`;
- `source_task_rev`;
- `source_projection_rev`.

These fields help prevent:

- stale action submission;
- task state race conditions;
- multi-client inconsistency;
- broken evidence traceability;
- acceptance of proposals that no longer match the visible projection.

```text
No source revision, no deterministic admission context.
```

If revision expectations fail, the proposal must be refused or re-evaluated with explicit evidence.

## 6. Actor / Session / Client Context

ActionIntent must carry actor, session, and client context.

Required context includes:

- actor identity;
- session identity;
- client identity;
- viewer-relative projection;
- capability lookup;
- audit / evidence attribution;
- privacy / redaction note where required.

```text
The same projected action may be visible to many clients, but only admitted according to the actor/session/client context.
```

The context identifies who is proposing the action, from which session, and through which client projection.

## 7. Arguments

Arguments are structured intent data.

Arguments:

- are structured;
- are typed by the action contract or future schema;
- must not contain host callbacks;
- must not contain arbitrary code;
- must not contain renderer objects;
- must not contain raw pointer / key events;
- must be deterministic and evidence-friendly.

```text
Arguments describe intent data, not execution behavior.
```

Arguments describe what is being requested, not how the request is executed.

## 8. SafeAction / GuardedAction / DangerAction

Action criticality changes routing requirements.
It does not bypass admission.

### SafeAction

- ordinary low-risk action affordance;
- still requires `ActionOffer` / capability / admission;
- may be repeated according to policy.

### GuardedAction

- constrained action affordance;
- requires stronger capability / freshness / confirmation policy;
- must not silently repeat.

### DangerAction

- high-risk action affordance;
- requires explicit control availability;
- requires fresh connection where applicable;
- requires explicit confirmation route;
- must not be auto-retried;
- must not be available offline / stale.

Criticality changes routing requirements.
It does not bypass admission.

## 9. Repeat and Idempotency Policy

Repeated UI gestures are not automatically repeated semantic actions.

Repeat policy should use:

- `idempotency_key`;
- `sequence_no`;
- duplicate suppression;
- no silent repeat for `GuardedAction`;
- no silent repeat for `DangerAction`;
- replay protection;
- evidence trace for rejected duplicates.

Duplicate protection is part of the routing contract.

## 10. ActionIntentBatch

Batches are ordered proposals.

Batch requirements:

- batch id;
- ordered intents;
- default mode: `Atomic`;
- explicit mode: `OrderedPartial`;
- source revision expectations;
- per-intent evidence;
- batch-level evidence.

```text
Atomic is the default.
OrderedPartial must be explicit.
Denied is not NotApplied.
```

Batch routing groups proposals; it does not merge authority.

## 11. StreamIntent

Stream intent defines a bounded proposal stream for continuous UI interactions.

StreamIntent should cover:

- stream id;
- source action ref;
- bounded lifetime;
- preview versus commit distinction;
- cancellation;
- backpressure;
- coalescing;
- evidence checkpoints;
- no unbounded raw event streaming into Semantic.

Examples include:

- slider preview;
- drag preview;
- text composition preview;
- final commit intent.

```text
Continuous UI input must become bounded semantic intent, not raw event leakage.
```

Stream intent is still a proposal contract, not a direct effect channel.

## 12. Admission Queue Boundary

ActionIntent enters an admission queue or equivalent boundary.

Queue semantics are explicit:

- ordering is explicit;
- backpressure is allowed;
- coalescing is allowed only when declared safe;
- admission result returns as patches / evidence;
- UI shell does not decide admission.

```text
The queue may transport proposals.
Only Semantic admission may accept or deny them.
```

The queue is a transport boundary, not a policy owner.

## 13. Result Flow

High-level result categories include:

- `Accepted`;
- `Denied`;
- `LocalDenied`;
- `AdmissionDenied`;
- `StaleRejected`;
- `CapabilityRejected`;
- `PendingUnknown`;
- `NotApplied`;
- `Quarantined`.

Result projection is routed through UI IR denial / evidence / recovery routes and later specs.

```text
Denied is projected, not handled.
```

## 14. Connectivity and Freshness

Freshness affects whether an action can be proposed or admitted.

Relevant states:

- `Fresh`;
- `Degraded`;
- `Stale`;
- `Offline`;
- `Resyncing`;
- `PendingUnknown`.

Rules:

- no offline queue for critical actions;
- stale / offline disables `GuardedAction` / `DangerAction` / `TaskControl`;
- connection drop after send may produce `PendingUnknown`.

```text
No freshness, no control.
```

Freshness is an input to routing policy, not a networking implementation detail in this spec.

## 15. Evidence and Audit Trace

Action routing must be inspectable after the fact.

Evidence should include:

- intent id;
- source action offer;
- source revisions;
- actor / session / client refs;
- result;
- denial reason if any;
- timing / order trace;
- privacy / redaction note.

Evidence must remain enough for later review without granting authority to the UI layer.

## 16. Non-Authority Rule

Action IR and ActionIntent must not own or redefine:

- Semantic truth;
- verifier admission;
- VM / runtime behavior;
- capability policy;
- recovery policy;
- task engine behavior;
- host effects;
- renderer lifecycle.

```text
ActionIntent proposes.
Admission decides.
Projection reports.
```

Action routing is a structured proposal path, not the authority path itself.

## 17. Non-Normative Example

Non-normative sketch — not final serialization

```text
action_ir add {
  role: SafeAction
  from: ActionOffers.calculator.add
  target_surface: CalculatorView.main
  target_node: controls.add
  requires:
    source_state_rev: current
    capability: calculator.add
}

intent {
  intent_id: "intent-001"
  projection_id: "CalculatorView"
  surface_id: "main"
  node_id: "controls.add"
  action_id: "add"
  action_offer_ref: "ActionOffers.calculator.add"
  actor_ref: "actor.local"
  session_ref: "session.demo"
  client_ref: "client.shell"
  args: { lhs: 7, rhs: 3 }
  source_state_rev: 42
  source_task_rev: null
  sequence_no: 12
  idempotency_key: "demo-12"
}
```

This sketch illustrates structure only.
It is not final grammar, not final serialization, and not an implementation plan.

## 18. Acceptance Criteria

The spec is acceptable when:

- it defines raw UI event boundary;
- it defines Action IR as routing contract;
- it defines ActionIntent as proposed action candidate;
- it defines required source revisions;
- it defines actor / session / client context;
- it defines argument boundaries;
- it defines SafeAction / GuardedAction / DangerAction;
- it defines repeat and idempotency policy;
- it defines ActionIntentBatch;
- it defines StreamIntent;
- it defines admission queue boundary;
- it defines result flow categories;
- it defines freshness impact;
- it defines evidence / audit trace;
- it preserves Semantic authority;
- it includes a non-normative sketch only;
- it does not implement runtime routing;
- it does not claim production readiness.
