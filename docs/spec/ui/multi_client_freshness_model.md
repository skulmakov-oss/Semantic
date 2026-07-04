# Multi-Client and Freshness Projection Model

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
- docs/spec/ui/denial_recovery_projection.md
- docs/spec/ui/task_projection_model.md
Related:
- #1310
- #1328
- #1329
- #1330
- #1331
- #1332
- #1333
- #1334
- #1335
- #1336

Multi-client and freshness projection defines how shared Semantic state is projected differently for different actors, sessions, and clients.

It does not define Semantic truth, task engine behavior, verifier rules, VM behavior, runtime authority, networking implementation, renderer behavior, host effects, or production UI wiring.

This document does not implement multi-client synchronization, networking, freshness tracking, ActionOffer generation, runtime queues, shell behavior, Rust types, compiler behavior, or renderer backend behavior.

## 1. Purpose

This spec exists to prevent the bad workflow:

```text
one client sees a button -> every client assumes the same control authority
```

The intended model is:

```text
Semantic state is authority-owned.
Task state is global where authority says it is global.
Projection is viewer-relative.
ActionOffers are actor/session/client-relative.
Freshness gates control.
Shell displays only the controls valid for that viewer.
```

```text
One task.
Many views.
No shared illusion.
```

Multi-client freshness projection keeps shared authority visible without pretending every viewer has the same control surface.

## 2. Non-Authority Rule

Multi-client projection must not own or redefine:

- Semantic truth;
- task state authority;
- verifier admission;
- capability policy;
- ActionOffer generation;
- networking behavior;
- runtime scheduling;
- VM behavior;
- host effects;
- renderer lifecycle.

```text
Projection may differ per viewer.
Semantic truth does not.
```

Projection is a view contract, not a shared authority layer.

## 3. Viewer-Relative Projection

Viewer-relative projection means the same underlying state may appear differently depending on the viewer context.

It should cover:

- actor identity;
- session identity;
- client identity;
- viewer role / capability context;
- viewer-specific visibility;
- viewer-specific control availability;
- viewer-specific redaction / privacy;
- viewer-specific freshness state;
- same underlying state, different projected affordances.

```text
Viewer-relative projection changes what a viewer may see or do.
It does not fork Semantic truth.
```

Viewer-relative projection is the mechanism that keeps a shared task understandable without claiming uniform control.

## 4. Actor / Session / Client Model

The projection context uses three distinct references:

- `actor_ref` — who is acting;
- `session_ref` — authenticated / session context;
- `client_ref` — concrete UI / client surface.

Optional device / location / channel metadata may be shown if authority provides it.

The projection should also preserve a privacy / redaction note.

```text
The same actor may have multiple sessions.
The same session may have multiple clients.
The same client must not imply authority without actor/session context.
```

These references provide attribution without collapsing all clients into one authority bucket.

## 5. ActionOffers per Actor / Session / Client

ActionOffers are viewer-specific.

They may differ by:

- actor;
- session;
- client.

Rules:

- controls may be visible to one viewer and unavailable to another;
- unavailable controls should show reason where practical;
- UI must not invent controls to make clients look consistent.

```text
Controls are offered by authority per context.
UI does not normalize control authority across clients.
```

ActionOffers may be shared in meaning while still being filtered by viewer context.

## 6. Task State versus Task Controls

Task state may be shared / global where authority says it is global.

Task projection may still be viewer-relative.

Task controls are capability-relative.

Examples:

- one viewer may resume while another may only observe;
- task state changes require authority evidence;
- controls require `ActionOffers` and freshness.

```text
Task state is authority-owned.
Controls are context-owned.
Projection is viewer-relative.
```

The same task can exist globally while control surfaces remain different per viewer.

## 7. No Semantic ExternallyPaused

Do not create semantic status `ExternallyPaused`.

Use this model instead:

```text
Paused is semantic.
Paused-by-whom is evidence.
Can-resume is capability.
How-it-looks is projection.
```

“Externally paused” may be a projected interpretation, but it is not a new semantic task state unless separately approved by Semantic authority.

## 8. Causal Attribution

Visible multi-client changes should be attributable.

Required causal attribution fields include:

- `caused_by`;
- `actor_ref`;
- `session_ref`;
- `client_ref`;
- `intent_ref`;
- `action_offer_ref`;
- `previous_state_rev`;
- `new_state_rev`;
- `previous_task_rev`;
- `new_task_rev`;
- `projection_rev`;
- `evidence_ref`;
- timestamp / order trace.

```text
A visible change should be attributable without making UI the authority.
```

Privacy may redact some actor or client detail, but evidence references should preserve traceability where allowed.

## 9. Freshness States

Freshness taxonomy includes:

- `Fresh`;
- `Degraded`;
- `Stale`;
- `Offline`;
- `Resyncing`;
- `PendingUnknown`.

Required distinctions:

```text
Connected is not enough.
Fresh is required for control.
Offline is not Stale.
PendingUnknown is not Accepted or Denied.
Resyncing is not Fresh.
```

Freshness is a control-quality state, not just a status label.

## 10. Freshness and Control Availability

Freshness gates control availability.

Rules:

- `Fresh` may allow controls when `ActionOffers` and capabilities also allow;
- `Degraded` may allow observation or limited safe controls only if explicitly permitted;
- `Stale` grants observation only by default;
- `Offline` grants cache only by default;
- `Resyncing` disables critical controls until fresh;
- `PendingUnknown` restricts controls until the causal chain is resolved.

```text
No freshness, no control.
```

Freshness gating keeps projected control from outpacing authority confidence.

## 11. Critical Actions

Critical actions include `GuardedAction`, `DangerAction`, and critical task controls.

Rules:

- no stale / offline critical action availability;
- no auto-retry;
- no silent repeat;
- no offline queue for critical actions;
- stale / offline critical attempts route through `LocalDenied`.

```text
Critical controls require fresh confirmed authority context.
```

Critical controls must never look safe simply because the client is connected.

## 12. Offline Queue Rule

There is no offline queue for critical actions.

This means:

- critical actions must not be stored for later implicit send;
- queued safe actions require explicit future spec / authority approval;
- UI must not imply that an offline critical control will execute later;
- stale / offline critical attempts become `LocalDenied` or unavailable projection.

This rule keeps critical authority from being replayed without fresh context.

## 13. PendingUnknown

`PendingUnknown` means the system cannot yet determine the task or control result.

It may happen when:

- connection drops after intent send;
- admission result is unknown;
- task control result is unknown;
- patch causal chain is broken;
- no success claim is available;
- no denial claim is available unless later evidence arrives;
- resync is required;
- controls remain restricted until freshness is restored.

```text
PendingUnknown must not be rendered as success or failure.
```

PendingUnknown is uncertainty, not a disguised completion state.

## 14. Resync Behavior

Resync behavior exists to restore causal confidence before control returns.

It should cover:

- entering `Resyncing`;
- observation-only mode;
- rejecting stale patches;
- requiring snapshot or authority checkpoint;
- resolving `PendingUnknown`;
- re-evaluating `ActionOffers`;
- restoring `Fresh` only after the causal chain is known.

```text
Resync restores causal confidence before control returns.
```

Resync is a controlled recovery mode, not an automatic re-enable.

## 15. Multi-Client Conflicts

Multi-client conflicts should remain visible when they occur.

They may include:

- concurrent intents;
- stale source revisions;
- conflicting task controls;
- capability mismatch.

The viewer may see a conflict state if relevant, along with a conflict evidence route.

```text
Conflict is projected.
Conflict is not silently resolved by UI.
```

UI may surface conflict, but only authority can define the resolution contract.

## 16. Evidence and Audit Trace

Multi-client projection must remain inspectable after the fact.

Evidence should include:

- actor / session / client refs where allowed;
- intent / action refs;
- task refs;
- freshness state;
- source revisions;
- result category;
- causal attribution;
- redaction / privacy note;
- evidence route.

Evidence traceability must survive projection differences across clients.

## 17. Privacy and Redaction

Privacy may hide detail without destroying causality.

Rules:

- viewer may not see full actor / session / client detail;
- redaction must not erase evidence traceability;
- redacted display must not invent false attribution;
- privacy filtering is projection-layer behavior controlled by authority policy.

```text
Privacy may hide details.
It must not falsify causality.
```

Redaction is an exposure control, not a rewrite of the event chain.

## 18. Physical Safety versus Remote UI Safety

Remote UI safety is not the same as physical safety.

Required meaning:

- physical / hardwired safety controls are outside remote UI authority;
- remote UI may project safety-related requests only where authority permits;
- remote `DangerAction` must not be shown available without fresh confirmed control channel;
- physical emergency stop is not equivalent to remote UI emergency request.

```text
Remote UI can request.
Physical safety remains its own authority boundary.
```

This boundary prevents remote projection from being mistaken for physical control.

## 19. Accessibility and Operator Readability

Accessibility for multi-client freshness projection should include:

- freshness label;
- control availability reason;
- viewer-relative capability label;
- `PendingUnknown` label;
- `Resyncing` label;
- conflict label;
- evidence route;
- non-visual interpretation.

```text
Freshness and control authority must be operator-readable, not only visually styled.
```

Accessibility must make control availability explainable across clients.

## 20. Diagnostics

Diagnostics generated by this spec should include:

- missing actor / session / client context;
- control shown without `ActionOffer`;
- critical control shown while stale / offline;
- `PendingUnknown` rendered as success / failure;
- `Resyncing` shown as `Fresh`;
- unavailable control without reason where required;
- privacy redaction removing traceability;
- stale patch accepted as fresh;
- multi-client conflict silently resolved by UI;
- semantic `ExternallyPaused` introduced by projection.

```text
Diagnostics are evidence, not silent UI guesses.
```

Diagnostics are part of projection accountability, not a client-side convenience layer.

## 21. Non-Normative Sketch

Non-normative sketch — not final serialization

```text
viewer_projection {
  projection_id: "TaskPanel"
  task_id: "task-042"
  actor_ref: "actor.operator-a"
  session_ref: "session-a"
  client_ref: "client-control-room"
  freshness: Fresh
  visible_controls: [
    ActionOffers.task.pause,
    ActionOffers.task.cancel
  ]
}

viewer_projection {
  projection_id: "TaskPanel"
  task_id: "task-042"
  actor_ref: "actor.observer-b"
  session_ref: "session-b"
  client_ref: "client-dashboard"
  freshness: Stale
  visible_controls: []
  reason: "observation_only_stale_projection"
}

causal_attribution {
  task_id: "task-042"
  caused_by: "intent-901"
  actor_ref: "actor.operator-a"
  session_ref: "session-a"
  client_ref: "client-control-room"
  previous_task_rev: 10
  new_task_rev: 11
  evidence_ref: "evidence-1001"
}
```

This sketch only illustrates multi-client projection shape.
It is not final grammar, not implementation, and not a runtime contract.

## 22. Acceptance Criteria

The spec is acceptable when:

- it defines viewer-relative projection;
- it defines actor / session / client context;
- it defines `ActionOffers` per actor / session / client;
- it distinguishes task state from task controls;
- it forbids semantic `ExternallyPaused`;
- it defines causal attribution;
- it defines freshness states;
- it defines freshness and control availability rules;
- it defines critical action restrictions;
- it forbids offline queue for critical actions;
- it defines `PendingUnknown`;
- it defines resync behavior;
- it defines multi-client conflict projection;
- it defines evidence / audit trace;
- it defines privacy / redaction boundary;
- it distinguishes physical safety from remote UI safety requests;
- it defines accessibility / operator readability;
- it defines diagnostics;
- it preserves Semantic authority;
- it includes a non-normative sketch only;
- it does not implement networking;
- it does not implement runtime synchronization;
- it does not claim production readiness.
