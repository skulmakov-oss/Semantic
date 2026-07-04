# ProjectionBundle Fixture Inventory and Golden Evidence Plan

Status: planning / fixture inventory
Track: POST-UI / Intent-Driven Projection
Scope type: documentation only
Implementation status: blocked
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
- docs/spec/ui/multi_client_freshness_model.md
- docs/spec/ui/projection_bundle_delivery.md
- docs/roadmap/post_ui/ui_shell_kit_projection_alignment.md
- docs/roadmap/post_ui/intent_driven_projection_closeout.md
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
- #1337
- #1338
- #1339
- #1340

This document defines a non-executing fixture inventory and golden evidence plan for future ProjectionBundle work.

It is not a fixture implementation.
It is not a serialization format.
It is not a loader plan.
It is not a runtime plan.
It is not a shell-player plan.
It does not authorize production UI wiring.

Fixtures and golden evidence must come before runtime behavior.
Implementation remains blocked until a separate implementation issue, Harness task, allowed_paths declaration, and review boundary are approved.

## 1. Purpose

This plan exists to prevent the bad workflow:

```text
spec stack closed -> immediately implement loader/runtime/shell player
```

The intended workflow is:

```text
closed specs -> fixture inventory -> golden evidence plan -> fixture-only PRs -> review -> tiny implementation slices later
```

```text
Fixtures before runtime.
Evidence before authority.
No loader first.
```

The plan is a staging map for evidence, not an implementation backdoor.

## 2. Non-Authority Rule

Fixture planning must not own or redefine:

- Semantic truth;
- verifier admission;
- capability / audit authority;
- host-effect permission;
- VM / runtime behavior;
- UI IR execution;
- `ProjectionBundle` verification;
- shell-player behavior;
- renderer backend behavior;
- production UI wiring.

```text
A fixture describes expected evidence.
It does not execute authority.
```

Fixture planning only specifies what evidence should exist when the future implementation is built.

## 3. Fixture Categories

The planned fixture categories are:

- ProjectionBundle manifest fixture;
- UI IR skeleton fixture;
- Role dictionary compatibility fixture;
- Renderer profile compatibility fixture;
- Binding Graph fixture;
- Action IR fixture;
- ActionIntent envelope fixture;
- Patch stream fixture;
- Denial / recovery fixture;
- Task projection fixture;
- Multi-client / freshness fixture;
- Evidence / audit trace fixture;
- Accessibility contract fixture;
- Invalid / negative fixture set.

For each category, the plan should eventually answer:

- purpose;
- future evidence produced;
- boundary;
- not-yet-implemented status.

These categories are intentionally pre-runtime and pre-loader.

### ProjectionBundle Manifest Fixture

Purpose:
- define the expected bundle identity and trust envelope.

Future evidence produced:
- manifest field presence / stability;
- deterministic identity inspection;
- compatibility references.

Boundary:
- non-executing;
- no bundle load;
- no verification execution.

Not-yet-implemented status:
- planned only.

### UI IR Skeleton Fixture

Purpose:
- capture the minimum future UI IR structure expected by the bundle.

Future evidence produced:
- surface / node / role inventory;
- binding references;
- outlet references.

Boundary:
- structure evidence only;
- no UI IR runtime.

Not-yet-implemented status:
- planned only.

### Role Dictionary Compatibility Fixture

Purpose:
- verify the expected role dictionary version relationship.

Future evidence produced:
- compatibility / incompatibility result;
- unsupported role diagnostics.

Boundary:
- compatibility evidence only;
- no silent reinterpretation.

Not-yet-implemented status:
- planned only.

### Renderer Profile Compatibility Fixture

Purpose:
- verify the expected renderer profile against supported roles and surfaces.

Future evidence produced:
- renderer profile match / mismatch result;
- capability/support matrix.

Boundary:
- compatibility evidence only;
- no backend selection.

Not-yet-implemented status:
- planned only.

### Binding Graph Fixture

Purpose:
- capture deterministic dependency edges and dirty-propagation expectations.

Future evidence produced:
- source/target edge inventory;
- revision expectations;
- keyed collection references.

Boundary:
- no patch propagation implementation.

Not-yet-implemented status:
- planned only.

### Action IR Fixture

Purpose:
- capture the future action routing contract.

Future evidence produced:
- affordance route inventory;
- criticality and boundary expectations.

Boundary:
- route evidence only;
- no admission execution.

Not-yet-implemented status:
- planned only.

### ActionIntent Envelope Fixture

Purpose:
- capture the shape of upward action proposals.

Future evidence produced:
- actor/session/client attribution;
- source revision expectations;
- idempotency evidence.

Boundary:
- proposal evidence only;
- no semantic execution.

Not-yet-implemented status:
- planned only.

### Patch Stream Fixture

Purpose:
- capture the expected sequence of deterministic projection updates.

Future evidence produced:
- patch family inventory;
- sequence / stale / resync expectations.

Boundary:
- update evidence only;
- no runtime application.

Not-yet-implemented status:
- planned only.

### Denial / Recovery Fixture

Purpose:
- capture refusal, partial batch, and recovery expectations.

Future evidence produced:
- denial taxonomy coverage;
- recovery route coverage.

Boundary:
- projected refusal evidence only;
- no denial handling runtime.

Not-yet-implemented status:
- planned only.

### Task Projection Fixture

Purpose:
- capture task state, phase, progress, and control expectations.

Future evidence produced:
- task state / phase inventory;
- control and lock expectations.

Boundary:
- task projection evidence only;
- no task engine behavior.

Not-yet-implemented status:
- planned only.

### Multi-Client / Freshness Fixture

Purpose:
- capture viewer-relative projection and freshness gating expectations.

Future evidence produced:
- viewer-specific visibility;
- control availability differences;
- stale critical action rejection.

Boundary:
- projection evidence only;
- no networking or freshness tracking.

Not-yet-implemented status:
- planned only.

### Evidence / Audit Trace Fixture

Purpose:
- capture the traceability expected from bundles and projections.

Future evidence produced:
- source / bundle / action references;
- result categories;
- redaction notes.

Boundary:
- audit planning evidence only;
- no audit authority creation.

Not-yet-implemented status:
- planned only.

### Accessibility Contract Fixture

Purpose:
- capture non-visual and operator-readable expectations.

Future evidence produced:
- role labels;
- freshness labels;
- control labels;
- denial labels;
- keyboard / focus expectations.

Boundary:
- contract evidence only;
- no renderer implementation.

Not-yet-implemented status:
- planned only.

### Invalid / Negative Fixture Set

Purpose:
- prove that unsupported or unsafe forms are rejected.

Future evidence produced:
- diagnostics coverage for missing / invalid / mismatched inputs.

Boundary:
- negative evidence only;
- no production runtime path.

Not-yet-implemented status:
- planned only.

## 4. ProjectionBundle Manifest Fixture

The future manifest fixture should cover:

- `bundle_id`
- `bundle_version`
- `projection_id`
- `source_refs`
- `ui_ir_ref`
- `binding_graph_ref`
- `action_ir_ref`
- `role_dictionary_version`
- `renderer_profile`
- `safety_class`
- `criticality`
- `required_capabilities`
- `freshness_policy`
- `hash`
- `signature`
- `created_by`
- `created_at`
- `compiler_identity`
- `compatibility`
- `activation_policy`
- `update_policy`
- `diagnostics`

```text
The manifest fixture is non-executing.
It must not trigger bundle loading or verification.
```

The manifest fixture should be sufficient to validate identity and compatibility expectations before any future loader exists.

## 5. UI IR Skeleton Fixture

Future UI IR skeleton fixtures should cover:

- top-level projection id;
- source refs;
- surfaces;
- nodes;
- roles;
- bindings refs;
- action refs;
- evidence routes;
- denial routes;
- recovery routes;
- task contracts;
- accessibility contract;
- diagnostics.

```text
The UI IR fixture is structure evidence, not UI IR runtime.
```

The purpose is to prove the shape of the compiled projection contract, not to execute it.

## 6. Binding Graph Fixture

Future Binding Graph fixtures should cover:

- stable source refs;
- target refs;
- deterministic dependency edges;
- keyed collection refs;
- dirty propagation expectation;
- source revision expectation;
- projection revision expectation.

```text
Binding Graph fixture does not implement patch propagation.
```

These fixtures should make dependency expectations visible without applying updates.

## 7. Action IR and ActionIntent Fixture

Future action fixtures should cover:

- projected affordance;
- `ActionOffer` ref;
- `ActionIntent` envelope;
- actor / session / client refs;
- source state rev;
- source task rev;
- projection rev;
- idempotency key;
- capability ref;
- evidence ref;
- expected local / admission boundary.

```text
Action fixture proposes intent shape.
It does not execute admission.
```

The goal is to preserve the upward routing contract without implementing the route.

## 8. Patch Stream Fixture

Future patch stream fixtures should cover:

- `SemanticStatePatch`;
- `ProjectionPatch`;
- `RenderPatch`;
- `EvidencePatch`;
- `ActionOfferPatch`;
- `ConnectivityPatch`;
- `TaskStatePatch`;
- sequence number;
- source revision;
- projection revision;
- stale patch case;
- resync case.

```text
Patch fixture is expected-update evidence.
It does not implement runtime patch application.
```

These fixtures should make deterministic updates inspectable before any runtime patching exists.

## 9. Denial / Recovery Fixture

Future denial / recovery fixtures should cover:

- `LocalDenied`;
- `AdmissionDenied`;
- `CapabilityRejected`;
- `StaleRejected`;
- `FreshnessRejected`;
- `PartialDenied`;
- `NotApplied`;
- `BatchBreak`;
- `PendingUnknown`;
- `Quarantined`;
- recovery options;
- evidence route.

```text
Denied is projected, not handled.
Recovery is projected, but never improvised.
```

```text
Denial fixture does not implement denial handling.
Recovery fixture does not implement recovery behavior.
```

## 10. Task Projection Fixture

Future task projection fixtures should cover:

- task id;
- originating `ActionIntent`;
- task state;
- phase;
- progress certainty;
- allowed controls;
- scope locks;
- evidence timeline;
- viewer-relative controls;
- freshness state.

```text
Task lives in Semantic.
Progress lives in Projection.
Pixels live in Shell.
```

```text
Task fixture does not implement task engine behavior.
```

The future fixture should prove what task projection will look like before task runtime exists.

## 11. Multi-Client / Freshness Fixture

Future multi-client / freshness fixtures should cover:

- same task / state shown to multiple clients;
- actor / session / client refs;
- viewer-relative projection;
- `Fresh`;
- `Degraded`;
- `Stale`;
- `Offline`;
- `Resyncing`;
- `PendingUnknown`;
- control availability differences;
- stale critical action rejection.

```text
One task.
Many views.
No shared illusion.
No freshness, no control.
```

```text
Freshness fixture does not implement networking or freshness tracking.
```

## 12. Evidence / Audit Trace Fixture

Future evidence fixtures should cover:

- source refs;
- bundle refs;
- hash / signature refs;
- action refs;
- actor / session / client refs where allowed;
- capability / audit refs;
- semantic admission result;
- capability result;
- freshness result;
- patch result;
- redaction note;
- diagnostics.

```text
Evidence fixture supports audit planning.
It does not create audit authority.
```

## 13. Accessibility Contract Fixture

Future accessibility fixtures should cover:

- role names;
- non-visual labels;
- control authority labels;
- freshness labels;
- denial labels;
- `PendingUnknown` labels;
- task phase labels;
- evidence route labels;
- keyboard / focus expectations where applicable.

```text
Accessibility fixture is contract evidence, not renderer implementation.
```

The plan should keep accessibility visible as a projection contract, not a style detail.

## 14. Negative Fixture Set

Negative fixture categories should include:

- missing manifest;
- missing UI IR;
- missing Binding Graph;
- missing Action IR;
- unsupported role dictionary;
- renderer profile mismatch;
- invalid signature;
- missing hash;
- critical bundle not pinned;
- dynamic bundle not verified;
- control affordance in read-only bundle;
- stale critical action;
- offline queued critical action;
- `PendingUnknown` rendered as success;
- semantic `ExternallyPaused`;
- capability / audit authority widening.

```text
Negative fixtures prove the boundary.
```

The negative set is essential so future implementation cannot quietly accept the wrong shape.

## 15. Golden Evidence Plan

Future golden evidence should prove:

- deterministic fixture parsing / reading once a parser exists;
- deterministic manifest identity;
- deterministic role dictionary compatibility result;
- deterministic renderer profile compatibility result;
- deterministic UI IR structure snapshot;
- deterministic Binding Graph edge inventory;
- deterministic `ActionIntent` envelope;
- deterministic patch sequence;
- deterministic denial projection;
- deterministic task projection;
- deterministic freshness / control gating;
- deterministic evidence trace;
- deterministic negative diagnostics.

```text
Golden evidence proves expected contracts.
It does not prove production readiness.
```

The point is to anchor future implementation to stable contract evidence before any runtime is allowed to grow.

## 16. File Layout Proposal

Suggested future layout only:

```text
tests/fixtures/post_ui/
  projection_bundle/
    README.md
    manifest_minimal.*
    manifest_invalid_missing_hash.*
    manifest_invalid_critical_unpinned.*
  ui_ir/
    minimal_surface.*
    invalid_unknown_role.*
  action_ir/
    intent_envelope_minimal.*
    intent_stale_source_rev.*
  patch_streams/
    state_patch_sequence.*
    stale_patch_rejected.*
  denial_recovery/
    ordered_partial_batch.*
    pending_unknown.*
  task_projection/
    task_running.*
    task_quarantined.*
  multi_client_freshness/
    same_task_two_viewers.*
    stale_control_rejected.*
  evidence/
    audit_trace_minimal.*
```

Do not create this layout in this PR.
Do not choose final file extensions in this PR.

## 17. First Fixture PR Recommendation

The first future fixture PR should be docs / fixture-only and should add only a README plus one non-executing manifest sketch, if separately approved.

No loader.
No runtime.
No shell player.
No production wiring.
No Rust types.

## 18. Implementation Gates

Implementation remains blocked until:

- fixture inventory approved;
- first fixture PR approved;
- golden evidence expectations approved;
- allowed_paths declared;
- forbidden_paths declared;
- one narrow implementation target selected;
- no production wiring;
- no loader-first work;
- no authority widening.

```text
No runtime until fixtures exist.
No fixtures without evidence.
No evidence without boundaries.
```

## 19. Acceptance Criteria

The plan is acceptable when:

- it defines fixture categories;
- it defines future golden evidence expectations;
- it defines negative fixture categories;
- it proposes future layout without creating files;
- it recommends first fixture PR;
- it keeps implementation blocked;
- it preserves Semantic authority;
- it preserves capability / audit authority;
- it does not create fixtures;
- it does not implement anything;
- it does not claim production readiness.
