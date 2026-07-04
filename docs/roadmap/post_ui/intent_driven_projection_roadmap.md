# Intent-Driven Projection Roadmap

Status: roadmap
Track: POST-UI / Intent-Driven Projection
Scope type: documentation only
Depends on:
- docs/dna/SEMANTIC_UI_DNA.md
- docs/dna/SEMANTIC_UI_DNA_v2.md
Related:
- #1310
- #1327
- #1328
- #1329

This roadmap sequences future work after Semantic UI DNA v2.

It does not implement `.proj.sm`, UI IR, Action IR, ProjectionBundle loading, runtime patches, or production UI wiring.

## Phase 0 — Doctrine Closed

Current completed state:

- `SEMANTIC_UI_DNA_v2.md` exists.
- `SEMANTIC_UI_DNA.md` cross-links to v2.
- Intent-Driven Projection doctrine is discoverable.

Completed by:

- `#1328`
- `#1329`

## Phase 1 — Projection Source Model

Define the future `.proj.sm` or equivalent projection source model.

Must answer:

- file naming: `.proj.sm` versus alternative;
- relation to `.sm`;
- no inline projection in v0 unless separately approved;
- projection intent versus layout pollution;
- minimal role vocabulary;
- accessibility as projection contract.

Output artifact:

- `docs/spec/ui/projection_source_model.md`

No parser implementation.

## Phase 2 — UI IR Schema

Define deterministic UI IR as a compiled artifact.

Must cover:

- surfaces;
- nodes;
- roles;
- bindings;
- evidence outlets;
- denial/recovery routes;
- task projection contracts;
- connectivity policies;
- role dictionary versioning.

Output artifact:

- `docs/spec/ui/ui_ir_schema.md`

No Rust types yet unless separately approved.

## Phase 3 — Action IR and ActionIntent Routing

Define the upward flow.

Must cover:

- raw UI events stay local;
- Action IR routes;
- ActionIntent envelope;
- `source_state_rev` / `source_task_rev`;
- actor/session/client context;
- `ActionIntentBatch`;
- `StreamIntent`;
- `GuardedAction` / `DangerAction` repeat restrictions.

Output artifact:

- `docs/spec/ui/action_ir_routing.md`

No runtime implementation.

## Phase 4 — Binding Graph and Patch Streams

Define the incremental projection update model.

Must cover:

- Static UI IR;
- Binding Graph;
- keyed collections;
- `SemanticStatePatch`;
- `ProjectionPatch`;
- `RenderPatch`;
- `EvidencePatch`;
- `ActionOfferPatch`;
- `ConnectivityPatch`;
- dirty propagation / revisions / epochs.

Output artifact:

- `docs/spec/ui/projection_patch_model.md`

No shell patch player implementation.

## Phase 5 — Denial, Partial Batch, and Recovery Projection

Define refusal and recovery behavior.

Must cover:

- Denied is projected, not handled;
- `LocalDenied` versus `Semantic AdmissionDenied`;
- `PartialDenied`;
- `NotApplied`;
- `BatchBreak`;
- `Dismiss` / `Acknowledge` / `Retry` / `Resume` / `CancelSuffix`;
- `ResumeToken`;
- `EvidencePanel` / `LocalDenialAnchor` routing.

Output artifact:

- `docs/spec/ui/denial_recovery_projection.md`

No UI code.

## Phase 6 — Long-Running Tasks

Define `TaskRecord` projection.

Must cover:

- `TaskRecord`;
- `TaskStatePatch`;
- task phases / progress;
- `AwaitingInput`;
- `allowed_controls` / `ActionOffers`;
- scope locks;
- recovery options;
- task evidence timeline.

Output artifact:

- `docs/spec/ui/task_projection_model.md`

No runtime task engine changes.

## Phase 7 — Multi-Client and Freshness

Define distributed projection safety.

Must cover:

- viewer-relative projection;
- `ActionOffers` per actor/session;
- causal attribution;
- no semantic `ExternallyPaused`;
- `Fresh` / `Degraded` / `Stale` / `Offline` / `Resyncing`;
- No freshness, no control;
- `PendingUnknown`;
- no offline queue for critical actions.

Output artifact:

- `docs/spec/ui/multi_client_freshness_model.md`

No networking implementation.

## Phase 8 — ProjectionBundle Delivery

Define bundle packaging and delivery.

Must cover:

- `ProjectionBundle` manifest;
- pinned critical UI;
- verified dynamic UI;
- runtime traffic as patches / intents;
- bundle hash / signature;
- role dictionary version;
- renderer profile;
- safe update boundaries.

Output artifact:

- `docs/spec/ui/projection_bundle_delivery.md`

No bundle loader implementation.

## Phase 9 — ui-shell-kit Alignment

Define how the current `ui-shell-kit` maps to the future model.

Must cover:

- current role as reference shell / evidence substrate;
- future role as `ProjectionBundle` player seed;
- what existing tests map to:
  - snapshot evidence;
  - focus / action trace;
  - hit-test stability;
  - motion phase evidence;
  - visual smoke bridge;
- what must not be promoted yet.

Output artifact:

- `docs/roadmap/post_ui/ui_shell_kit_projection_alignment.md`

No code changes.

## Gates

No implementation before the corresponding spec exists.

No parser before the projection source model is approved.

No UI IR Rust types before the UI IR schema is approved.

No runtime patch pipeline before the patch model is approved.

No production UI wiring before the ProjectionBundle delivery model is approved.

No `ui-shell-kit` promotion before the alignment doc exists.

## Hard Non-goals

- No code changes.
- No parser implementation.
- No `.proj.sm` implementation.
- No UI IR implementation.
- No Action IR implementation.
- No ProjectionBundle loader.
- No runtime patch pipeline.
- No production UI wiring.
- No Workbench dependency.
- No verifier / VM / SemCode changes.
- No runtime capability widening.
- No renderer backend decision.
- No promotion of `ui-shell-kit`.

## Acceptance Criteria

- the roadmap references UI DNA v2;
- the roadmap sequences work into small phases;
- every phase has a concrete docs / spec artifact;
- implementation is explicitly deferred;
- gates prevent scope creep;
- `ui-shell-kit` remains platform / internal reference infrastructure;
- the roadmap does not claim production readiness.
