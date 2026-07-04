# Binding Graph and Projection Patch Model

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
Related:
- #1310
- #1328
- #1329
- #1330
- #1331
- #1332
- #1333

The Binding Graph and Projection Patch Model define deterministic update flow for projected UI state.

They do not define Semantic truth, verifier rules, VM behavior, runtime authority, renderer behavior, networking behavior, host effects, or production UI wiring.

This document does not implement Binding Graph, patch streams, runtime patch queues, shell patch player, Rust types, compiler behavior, or renderer backend behavior.

## 1. Purpose

Projection source describes presentation intent.
UI IR defines static projection structure.
Binding Graph defines dependency edges.
Patch streams update projection state.
Shell applies projection updates.
Renderer paints through an adapter.
Semantic admission remains authoritative.

```text
Static UI IR defines structure.
Binding Graph defines dependencies.
Patch streams update projection deterministically.
```

```text
Compile structure once.
Patch projection forever.
```

Binding Graph and patch streams exist so projected UI can update deterministically without reinterpreting all structure on every change.

## 2. Non-Authority Rule

Binding Graph and patches must not own or redefine:

- Semantic truth;
- verifier admission;
- VM / runtime behavior;
- capability policy;
- recovery policy;
- task engine behavior;
- repository truth;
- business logic;
- host effects;
- renderer lifecycle.

```text
Patches report state changes.
They do not create semantic authority.
```

Binding edges and patch streams reflect authoritative state changes; they do not become authority themselves.

## 3. Static UI IR versus Dynamic Patches

The boundary is:

- UI IR is the static compiled projection structure;
- Binding Graph is derived from UI IR bindings and routes;
- patch streams update values, availability, evidence, freshness, and projection status;
- patch streams must not replace the full UI tree during normal runtime;
- full UI IR replacement requires explicit bundle / version boundary and is not defined here.

```text
Runtime traffic is patches, not arbitrary UI tree streaming.
```

UI IR remains the source of static projection shape.
Patches update that shape incrementally.

## 4. Binding Graph

Binding Graph is the dependency model connecting source observations to projected targets.

Binding Graph should cover:

- graph id;
- projection id;
- source nodes;
- target nodes;
- binding edges;
- dependency kind;
- source revision requirement;
- target projection revision;
- dirty propagation;
- diagnostics;
- evidence refs where practical.

Binding edge categories include:

- state binding;
- evidence binding;
- action offer binding;
- task binding;
- connectivity binding;
- accessibility binding if required.

```text
Binding Graph observes and routes dependencies.
It does not mutate Semantic state.
```

Binding Graph is a deterministic dependency map, not an effect system.

## 5. Source Nodes

Source-side graph concepts include:

- Semantic state source;
- evidence source;
- ActionOffer source;
- task source;
- connectivity / freshness source;
- projection-local source;
- source revision / epoch.

```text
A source node is an observed input, not an authority transfer.
```

Source nodes are observed facts or contracts that feed the projection graph.

## 6. Target Nodes

Target-side graph concepts include:

- UI IR node target;
- surface target;
- outlet target;
- action affordance target;
- evidence outlet target;
- denial / recovery outlet target;
- accessibility target;
- collection target.

```text
A target node receives projection updates.
It does not become the source of semantic truth.
```

Targets are the projected consumers of the dependency graph.

## 7. Patch Envelope

Patch streams share a common envelope shape at a high level.

Draft fields include:

- `patch_id`
- `patch_kind`
- `stream_id`
- `projection_id`
- `surface_id`
- `target_ref`
- `source_ref`
- `source_rev`
- `previous_projection_rev`
- `projection_rev`
- `epoch`
- `sequence_no`
- `causal_ref`
- `evidence_ref`
- `issued_at`
- `staleness_policy`
- `diagnostics`

```text
Every patch must be ordered, attributable, and revision-aware.
```

The envelope makes patch traffic inspectable and deterministic.

## 8. Patch Kinds

Required patch kinds include:

- `SemanticStatePatch`
- `ProjectionPatch`
- `RenderPatch`
- `EvidencePatch`
- `ActionOfferPatch`
- `ConnectivityPatch`

`TaskStatePatch` is reserved for the later task projection model and may be referenced only as a deferred patch family.

Patch kinds describe what class of projection update is being applied.

## 9. SemanticStatePatch

SemanticStatePatch reflects projection-facing state changes.

It should cover:

- source state path / ref;
- source revision;
- target binding;
- value projection;
- Quad-state preservation;
- unknown / conflict propagation;
- stale rejection or stale projection behavior;
- evidence ref where available.

```text
SemanticStatePatch reflects Semantic state.
It does not mutate Semantic state.
```

SemanticStatePatch updates what the projection shows about semantic state.

## 10. ProjectionPatch

ProjectionPatch covers projection-level state changes.

It should cover:

- projection visibility;
- surface availability;
- node availability;
- role interpretation updates;
- local projection status;
- viewer-relative projection updates;
- projection diagnostics.

```text
ProjectionPatch changes projection state, not Semantic state.
```

ProjectionPatch changes how the UI is presented, not what Semantic means.

## 11. RenderPatch

RenderPatch is renderer-independent projection output.

It should cover:

- target node / surface;
- display state;
- state badge / status update;
- focus intent update;
- accessibility update;
- animation phase name only if required.

Forbidden in this spec:

- backend-specific renderer commands;
- pixels;
- GPU commands;
- CSS-like layout;
- manual colors / fonts / themes.

```text
RenderPatch is renderer-independent projection output, not renderer backend code.
```

RenderPatch describes what the shell should prepare for painting, not how a backend paints it.

## 12. EvidencePatch

EvidencePatch routes evidence visibility and provenance updates.

It should cover:

- evidence id / ref;
- evidence kind;
- source provenance;
- target outlet;
- uncertainty display;
- trace visibility;
- privacy / redaction note;
- append versus replace behavior.

```text
EvidencePatch displays provenance.
It does not become audit authority.
```

EvidencePatch keeps evidence visible without transferring authority to the projection layer.

## 13. ActionOfferPatch

ActionOfferPatch updates projected affordance availability.

It should cover:

- ActionOffer ref;
- target `ActionSlot` / action node;
- role: `SafeAction` / `GuardedAction` / `DangerAction`;
- available / unavailable / denied / stale / pending state;
- capability requirement;
- freshness requirement;
- confirmation requirement;
- repeat policy update;
- denial route ref.

```text
ActionOfferPatch changes projected affordance availability.
It does not grant authority or bypass admission.
```

ActionOfferPatch is about visibility and gating of action affordances.

## 14. ConnectivityPatch

ConnectivityPatch updates freshness and connectivity projection.

It should cover:

- `Fresh`;
- `Degraded`;
- `Stale`;
- `Offline`;
- `Resyncing`;
- `PendingUnknown`;
- control availability impact;
- critical action gating;
- no offline queue for critical actions;
- evidence / ref if known.

```text
No freshness, no control.
```

ConnectivityPatch expresses what the projection knows about control readiness.

## 15. Dirty Propagation

Dirty propagation must be deterministic and evidence-friendly.

Required behavior:

- source change marks dependent bindings dirty;
- dirty bindings produce target patch candidates;
- propagation order must be deterministic;
- coalescing is allowed only when declared safe;
- diagnostics must record skipped / unresolved bindings;
- stale sources must not silently update critical controls.

```text
Dirty propagation must be deterministic and evidence-friendly.
```

Dirty propagation is the update scheduler for projection dependencies.

## 16. Revisions and Epochs

Patch streams are revision-aware.

Relevant fields include:

- `source_rev`;
- `projection_rev`;
- `previous_projection_rev`;
- `epoch`;
- `sequence_no`.

Requirements:

- monotonic expectations;
- stale patch rejection;
- replay rejection;
- resync boundary;
- `PendingUnknown` after broken causal chain.

```text
No revision chain, no deterministic patch application.
```

Revisions and epochs preserve deterministic patch ordering and replay safety.

## 17. Keyed Collections

Projected collections require stable identity.

Requirements:

- stable collection id;
- stable item key;
- insert;
- remove;
- update;
- move / reorder;
- identity preservation;
- missing key diagnostic;
- no anonymous list patching.

```text
No stable key, no deterministic projected collection.
```

Collection patches must preserve identity across incremental updates.

## 18. Quad-State Propagation

Patch streams must preserve:

- `N` — unknown
- `F` — false
- `T` — true
- `S` — conflict

```text
Patch streams must not flatten Quad-state into boolean visibility, success/failure, or generic disabled state.
```

Unknown propagation and conflict propagation must remain visible.
Denial is not false.
Stale is not unknown unless explicitly represented.

## 19. Diagnostics

Binding and patch model diagnostics should cover:

- missing source ref;
- missing target ref;
- unresolved binding;
- invalid role target;
- missing stable collection key;
- stale patch;
- replayed patch;
- unsupported patch kind;
- unsupported role dictionary version;
- unsafe coalescing attempt;
- critical control update without freshness;
- renderer capability mismatch.

```text
Diagnostics are evidence, not silent runtime guesses.
```

Diagnostics are part of the projection contract, not an implementation accident.

## 20. Resync Behavior

Resyncing is a high-level projection state for broken or incomplete causal chains.

Rules:

- projection may enter observation-only mode;
- control affordances may be disabled;
- stale patches may be rejected;
- patch stream may require snapshot boundary;
- `PendingUnknown` may be used when intent result is unknown.

```text
Connected is not enough.
Fresh is required for control.
```

Resync behavior protects projection determinism when causal continuity breaks.

## 21. Non-Normative Patch Sketch

Non-normative sketch — not final serialization

```text
binding_graph CalculatorView {
  source state.result rev state:42
  target node.display
  edge state.result -> node.display kind SemanticState
}

patch SemanticStatePatch {
  patch_id: "patch-001"
  projection_id: "CalculatorView"
  target_ref: "node.display"
  source_ref: "state.result"
  source_rev: 43
  previous_projection_rev: 12
  projection_rev: 13
  epoch: 1
  value: { quad: T, data: 10 }
}

patch ActionOfferPatch {
  patch_id: "patch-002"
  projection_id: "CalculatorView"
  target_ref: "node.add"
  source_ref: "ActionOffers.calculator.add"
  projection_rev: 14
  availability: "available"
  freshness: "Fresh"
}
```

This sketch illustrates structure only.
It is not final grammar, not final serialization, and not an implementation plan.

## 22. Acceptance Criteria

The spec is acceptable when:

- it defines Binding Graph purpose;
- it defines static UI IR versus dynamic patches;
- it defines source and target graph concepts;
- it defines patch envelope fields;
- it defines SemanticStatePatch;
- it defines ProjectionPatch;
- it defines RenderPatch;
- it defines EvidencePatch;
- it defines ActionOfferPatch;
- it defines ConnectivityPatch;
- it defines dirty propagation;
- it defines revisions and epochs;
- it defines keyed collection patching;
- it preserves Quad-state meaning;
- it defines diagnostics;
- it defines high-level resync behavior;
- it includes a non-normative sketch only;
- it does not implement patch streams;
- it does not implement shell patch player;
- it does not claim production readiness.
