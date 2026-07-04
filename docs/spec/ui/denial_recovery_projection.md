# Denial, Partial Batch, and Recovery Projection

Status: draft spec
Track: POST-UI / Intent-Driven Projection
Scope type: documentation only
Depends on:
- docs/dna/SEMANTIC_UI_DNA.md
- docs/dna/SEMANTIC_UI_DNA_v2.md
- docs/roadmap/post_ui/intent_driven_projection_roadmap.md
- docs/spec/ui/projection_source_model.md
- docs/spec/ui/ui_ir_schema.md
- docs/spec/ui/action_ir_routing.md
- docs/spec/ui/projection_patch_model.md
Related:
- #1310
- #1328
- #1329
- #1330
- #1331
- #1332
- #1333
- #1334

Denial, partial batch, and recovery projection define how refusal and recovery states are shown to users and operators.

They do not define Semantic truth, verifier rules, admission policy, VM behavior, runtime authority, recovery implementation, renderer behavior, host effects, or production UI wiring.

This document does not implement denial handling, recovery handling, batch execution, runtime queues, shell behavior, Rust types, compiler behavior, or renderer backend behavior.

## 1. Purpose

This spec exists to prevent the bad workflow:

```text
button failed -> local UI handler guesses what to do
```

The intended flow is:

```text
ActionIntent is proposed.
Semantic / local boundary accepts or denies.
Denial result is projected through declared routes.
Recovery options come from known contracts.
Shell displays the result.
UI does not improvise policy.
```

```text
Denied is projected, not handled.
Recovery is projected, but never improvised.
```

Denial, partial batch, and recovery projection keep refusal visible without letting UI invent policy.

## 2. Non-Authority Rule

Denial / recovery projection must not own or redefine:

- Semantic truth;
- verifier admission;
- admission policy;
- capability policy;
- recovery policy;
- task engine behavior;
- batch execution behavior;
- VM / runtime behavior;
- host effects;
- renderer lifecycle.

```text
Projection reports denial.
Semantic owns admission.
Recovery options come from authority.
```

Projection shows what happened; authority decides what was allowed.

## 3. Denial Taxonomy

Required result categories include:

- `Accepted`
- `Denied`
- `LocalDenied`
- `AdmissionDenied`
- `CapabilityRejected`
- `StaleRejected`
- `FreshnessRejected`
- `PartialDenied`
- `NotApplied`
- `BatchBreak`
- `PendingUnknown`
- `Quarantined`

Required distinctions:

```text
Denied is not false.
Denied is not NotApplied.
PendingUnknown is not Accepted.
Quarantined is not generic failure.
```

These categories keep refusal, non-application, uncertainty, and quarantine distinct.

## 4. LocalDenied versus AdmissionDenied

`LocalDenied` covers refusal that happens before Semantic admission.

It includes:

- no fresh control channel;
- local shell cannot form valid ActionIntent;
- missing source revision;
- missing actor / session / client context;
- unsupported renderer / shell capability;
- critical action attempted while stale / offline.

`AdmissionDenied` covers refusal returned by Semantic admission.

It includes:

- Semantic admission refused the ActionIntent;
- capability / policy / source-state reason came from the admission boundary;
- evidence should point to the admission result where available.

```text
LocalDenied stops before Semantic admission.
AdmissionDenied comes from Semantic admission.
```

The UI should show where the refusal happened, not blur the boundary.

## 5. Denial Projection Routes

Denial must have a projection route or produce a diagnostic.

Relevant routes and targets include:

- `DenialOutlet`;
- `LocalDenialAnchor`;
- surface-level denial;
- node-level denial;
- action-level denial;
- evidence-linked denial;
- viewer-relative denial;
- privacy / redaction note.

```text
A denial must have a projection route or produce a diagnostic.
```

Denial routes tell the shell where to expose refusal details and recovery choices.

## 6. EvidencePanel and Denial Evidence

Denial evidence should include:

- denial id;
- originating intent id, if any;
- action offer ref, if any;
- actor / session / client refs, if allowed;
- source revisions;
- denial kind;
- denial reason;
- result category;
- timestamp / order trace;
- redaction / privacy note;
- target outlet.

```text
Denial evidence explains what was refused and why, without making UI the authority.
```

Denial evidence makes refusal inspectable without turning the projection into policy.

## 7. Batch Result Model

Batch projection should carry:

- batch id;
- batch mode;
- ordered intents;
- batch-level evidence;
- per-intent evidence;
- source revision expectations;
- atomic versus ordered partial.

```text
Atomic is the default.
OrderedPartial must be explicit.
```

Batch projection must distinguish whole-batch refusal from partially applied ordered work.

## 8. Atomic Batch Behavior

Atomic batches are all-or-nothing from the projection standpoint.

Atomic behavior should state:

- batch either fully accepted or not applied;
- one denial may deny the whole batch;
- no partial semantic effect is claimed;
- UI must not present accepted prefix unless authority says it exists;
- evidence should show denied cause and unapplied intents.

```text
Atomic denial prevents partial truth claims.
```

Atomic mode prevents the UI from inventing partial success.

## 9. OrderedPartial Behavior

OrderedPartial batches may show a truthful prefix and an unapplied suffix.

OrderedPartial behavior should state:

- accepted prefix may become truth;
- denied step is evidence;
- unapplied suffix is explicit;
- UI must distinguish accepted, denied, and not applied;
- OrderedPartial must not be inferred by UI.

```text
Accepted prefix is truth.
Denied step is evidence.
Unapplied suffix is silence made visible.
```

OrderedPartial is a contract choice, not a visual guess.

## 10. Denied versus NotApplied

`Denied` and `NotApplied` must remain distinct.

Requirements:

- `Denied` means evaluated and refused;
- `NotApplied` means not evaluated or not executed due to prior batch break;
- `NotApplied` must not be rendered as denial;
- `NotApplied` must not be rendered as success;
- suffix visibility is required where practical.

```text
Denied != NotApplied.
```

The projection must not collapse refusal and non-execution into one visual state.

## 11. BatchBreak

`BatchBreak` explains where ordered execution stopped.

It should cover:

- break point;
- cause ref;
- denied intent ref;
- accepted prefix refs;
- not-applied suffix refs;
- evidence route;
- recovery route if available.

```text
BatchBreak is a projection event that explains where ordered execution stopped.
```

BatchBreak makes the boundary between applied and unapplied work explicit.

## 12. Recovery Taxonomy

Recovery options include:

- `Dismiss`
- `Acknowledge`
- `Retry`
- `Resume`
- `CancelSuffix`

And:

- `ResumeToken`

Required distinctions:

- `Dismiss` = local presentation only;
- `Acknowledge` = semantic-facing acknowledgement when authority provides it;
- `Retry` = new admitted retry proposal;
- `Resume` = requires `ResumeToken`;
- `CancelSuffix` = records cancellation of unapplied suffix when applicable.

```text
UI may present recovery options.
UI must not invent recovery options.
```

Recovery options come from authoritative contracts, not UI improvisation.

## 13. ResumeToken

`ResumeToken` governs resume permission.

It should cover:

- token id / ref;
- originating denial / batch / task ref;
- allowed actor / session / client scope;
- source revision expectations;
- expiration or invalidation;
- evidence ref;
- one-shot or repeat policy;
- privacy / security note.

```text
No ResumeToken, no Resume.
```

Resume requires a valid authority-bound token.

## 14. Retry Boundary

Retry forms a new proposal.

Retry must:

- create a new ActionIntent or batch;
- pass admission again;
- carry fresh revisions / context;
- not silently repeat `GuardedAction` / `DangerAction`;
- preserve evidence chain to original denial.

```text
Retry is a new proposal, not a continuation of authority.
```

Retry is a fresh request, not a hidden replay.

## 15. Acknowledge versus Dismiss

`Dismiss` and `Acknowledge` are different.

Requirements:

- `Dismiss` changes only local presentation;
- `Acknowledge` may become semantic-facing only if provided by `ActionOffers` / recovery contract;
- UI must not convert dismiss into semantic acknowledgement;
- evidence should distinguish them.

```text
Dismiss hides locally.
Acknowledge records intentionally.
```

The UI may expose both, but it must not confuse them.

## 16. Quarantine Projection

Quarantine is a guarded projection state.

It should cover:

- quarantine result category;
- affected action / batch / task / source refs;
- evidence route;
- allowed recovery only from authority;
- no auto-retry;
- control restrictions;
- visibility to operator.

```text
Quarantine is not generic failure.
Quarantine is a guarded state requiring explicit evidence and authority.
```

Quarantine indicates an explicit protective boundary, not a generic error.

## 17. PendingUnknown Projection

`PendingUnknown` is used when the outcome is not yet known.

It should cover:

- connection drop after send;
- missing admission result;
- broken causal chain;
- resync required;
- no success claim;
- no denial claim unless later evidence arrives;
- control restrictions until freshness restored.

```text
PendingUnknown must not be rendered as success or failure.
```

PendingUnknown is uncertainty, not hidden refusal or hidden acceptance.

## 18. Freshness and Critical Controls

Freshness affects control availability.

Relevant states:

- `Fresh`;
- `Degraded`;
- `Stale`;
- `Offline`;
- `Resyncing`.

Rules:

- no offline queue for critical actions;
- stale / offline disables `GuardedAction` / `DangerAction` / `TaskControl`;
- `LocalDenied` route for stale critical action attempts.

```text
No freshness, no control.
```

Freshness is a control contract, not a UI convenience.

## 19. Accessibility and Operator Readability

Denial / recovery projection must be operator-readable.

Accessibility requirements include:

- denial label;
- denial reason summary;
- result category;
- focus route to `DenialOutlet`;
- recovery option labels;
- criticality;
- non-visual interpretation;
- evidence provenance where practical.

```text
Denial and recovery must be operator-readable, not only visually styled.
```

The operator must be able to understand refusal and recovery without relying on color alone.

## 20. Diagnostics

Diagnostics generated by denial / recovery projection should cover:

- missing `DenialOutlet`;
- missing `RecoveryOutlet`;
- unknown denial kind;
- `NotApplied` rendered as denial;
- denied rendered as false;
- missing evidence ref;
- `Resume` without `ResumeToken`;
- `Retry` without `ActionOffer`;
- `Acknowledge` without authority;
- stale critical control shown as available;
- unsupported recovery option.

```text
Diagnostics are evidence, not silent UI guesses.
```

Diagnostics exist so refusal and recovery projection remains auditable.

## 21. Non-Normative Sketch

Non-normative sketch — not final serialization

```text
denial_result {
  result_id: "denial-001"
  kind: AdmissionDenied
  intent_id: "intent-042"
  action_offer_ref: "ActionOffers.calculator.divide"
  source_state_rev: 44
  reason: "division_by_zero"
  target: DenialOutlet.main
  evidence_ref: "evidence-901"
  recovery: [
    Dismiss,
    Retry from ActionOffers.calculator.divide
  ]
}

batch_result {
  batch_id: "batch-007"
  mode: OrderedPartial
  accepted_prefix: ["intent-001", "intent-002"]
  denied_step: "intent-003"
  not_applied_suffix: ["intent-004", "intent-005"]
  batch_break: "break-003"
  evidence_ref: "evidence-902"
}
```

This sketch illustrates structure only.
It is not final grammar or serialization.

## 22. Acceptance Criteria

The spec is acceptable when:

- it defines denial taxonomy;
- it distinguishes LocalDenied from AdmissionDenied;
- it defines denial projection routes;
- it defines denial evidence;
- it defines batch result model;
- it defines Atomic batch behavior;
- it defines OrderedPartial behavior;
- it distinguishes Denied from NotApplied;
- it defines BatchBreak;
- it defines recovery taxonomy;
- it defines ResumeToken;
- it defines Retry boundary;
- it distinguishes Acknowledge from Dismiss;
- it defines Quarantine projection;
- it defines PendingUnknown projection;
- it defines freshness impact on critical controls;
- it defines accessibility / operator readability;
- it defines diagnostics;
- it preserves Semantic authority;
- it includes a non-normative sketch only;
- it does not implement denial handling;
- it does not implement recovery handling;
- it does not claim production readiness.
