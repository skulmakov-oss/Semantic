# Semantic UI DNA v2

Status: architecture doctrine extension
Track: POST-UI / Intent-Driven Projection
Scope type: documentation only
Extends: docs/dna/SEMANTIC_UI_DNA.md
Issue: #1327

## Core Manifesto

Semantic UI DNA v2 extends the existing Semantic UI doctrine with an intent-driven projection model.

The core ordering is:

```text
Meaning first.
Intent projection second.
UI IR third.
Rendering last.
```

The ownership chain is:

```text
Semantic owns meaning.
Projection owns presentation intent.
UI IR owns structure.
Shell owns rendering.
Renderer owns pixels.
```

And the safety rule is:

```text
One meaning.
Many projections.
Truth does not move into UI.
```

UI behavior must remain contractually traceable from semantic state to rendered projection.

Semantic UI DNA v2 inherits Quad-state visibility from the original UI DNA.
Projection must not flatten `T` / `F` / `N` / `S`, denial, conflict, quarantine, or unknown into ordinary boolean UI state.

```text
Compile structure once.
Patch projection forever.
```

```text
UI proposes.
Semantic disposes.
Shell shows.
```

This document extends, but does not replace, `docs/dna/SEMANTIC_UI_DNA.md`.

## Zero-Glue Authoring

Semantic applications must not require `.sm` logic plus hand-written Rust UI glue as the default workflow.

The normal developer-facing model is:

```text
.sm       = Semantic Core: state, actions, invariants, admission, contracts
.proj.sm  = Semantic Projection: presentation intent, roles, bindings, surfaces
UI IR     = deterministic compiler output
shell     = ProjectionBundle player
```

Rules:

- `.sm` is meaning authority.
- `.proj.sm` is projection intent, not layout code.
- hand-written Rust UI glue is not the normal workflow.
- External Component Boundary, or ECB, is an escape hatch only.
- ECB does not grant Semantic authority.

The authoring goal is zero-glue by default: meaning is authored once, then projected, compiled, and played by shell infrastructure.

## Intent-Driven Projection

Projection is allowed to describe presentation intent, not pixels.

Allowed projection intent includes:

- `role: NumericReadout`
- `role: DangerAction`
- `role: EvidencePanel`
- `priority: High`
- controls from `ActionOffers`
- recovery outlet
- connectivity policy

Projection must not embed layout pollution or business logic duplication.

Forbidden in projection intent:

- pixels
- absolute coordinates
- manual CSS-like layout
- manual colors/fonts
- business logic duplication
- host effects

Litmus test:

```text
If a projection attribute cannot be interpreted by CLI or voice UI, it is likely renderer/layout pollution.
```

Accessibility labels, focus order, operator-readable roles, and non-visual interpretation are part of projection intent where relevant.
Accessibility is a projection contract, not renderer polish.

Projection expresses what should be shown and how it should be routed, not how a specific renderer must paint it.

## Static UI IR, Binding Graph, and Patch Stream

Semantic UI v2 relies on a deterministic static UI IR pipeline:

```text
Compile-time artifacts:
  Static UI IR
  Binding Graph
  Action IR
  Denial / Recovery routes
  Task projection contracts
  Connectivity policy
  ProjectionBundle

Runtime patch streams:
  SemanticStatePatch
  TaskStatePatch
  ProjectionPatch
  RenderPatch
  EvidencePatch
  ActionOfferPatch
  ConnectivityPatch
```

The Binding Graph records stable relationships between meaning, roles, surfaces, actions, and evidence.

Action IR carries structured action intent, not raw UI event noise.

Key points:

- Static UI IR is deterministic compiler output.
- Binding Graph is the stable mapping layer.
- Action IR is the structured route into admission and meaning.
- Patch streams are delivery artifacts, not ad hoc UI state mutation.
- Keyed collections are mandatory for projected lists.
- Missing stable keys in list projection are a projection check error.

The shell applies patches. The shell does not own meaning.

## Strict Action Routing

Raw UI events stay local.

Only structured `ActionIntent` enters Semantic admission.

`ActionIntent` carries source revision and actor/session context.

`GuardedAction` and `DangerAction` must not repeat silently.

Action routing rules:

- local pointer or key events are interpreted by the projection/shell boundary;
- only structured intent crosses into Semantic admission;
- intent must remain attributable to a source revision;
- actor/session context must remain visible where relevant;
- repeated dangerous actions require explicit handling, not silent replay.

This is a routing contract, not a UI convenience policy.

## Denial Projection

Developers do not write per-button error handlers as the primary model.

Denial is a structured projection event.

A denial may produce:

- local anchor
- rollback preview
- evidence append
- surface status update

`.proj.sm` declares routing and outlets, not business-specific denial logic.

Denied is projected, not handled.

Session-boundary denials such as `ConnectionOffline` are local projection/session denials.
They must be distinguishable from Semantic `AdmissionDenied` results returned by the Semantic core.

Denial projection must preserve why the action was denied, what evidence exists, and what recovery or acknowledgment is available.

## Partial Batch Semantics

Batch work can be projected as partially applied when the contract allows it.

Required distinction:

```text
Denied != NotApplied
```

Batch default is `Atomic`.

`OrderedPartial` must be explicit and safe.

Contract language:

```text
Accepted prefix is truth.
Denied step is evidence.
Unapplied suffix is silence made visible.
```

This keeps the UI honest about what happened in a batch without collapsing everything into success/failure.

## Recovery Projection

Recovery is projected, but never improvised.

Required recovery states:

- `Dismiss`
- `Acknowledge`
- `Retry`
- `Resume`
- `CancelSuffix`

Rules:

- `Resume` requires `ResumeToken`.
- recovery controls come from Semantic recovery options / `ActionOffers`.
- UI must not invent recovery behaviors.

Recovery is a routed contract surface, not a custom per-widget implementation detail.

## Long-Running Tasks

Long-running actions create `TaskRecord`.

Task progress is projected.

Task controls come from Semantic `ActionOffers`.

Tasks lock declared scopes, not the entire UI.

This means:

```text
Task lives in Semantic.
Progress lives in Projection.
Pixels live in Shell.
```

The shell does not re-decide task semantics.

## Multi-Client Projection

Multi-client UI must distinguish truth from viewer-relative presentation.

```text
One task.
Many views.
No shared illusion.
```

```text
Task state is global.
Projection is viewer-relative.
Controls are capability-relative.
```

Do not introduce semantic statuses like `ExternallyPaused`.

Instead:

```text
Paused is semantic.
Paused-by-whom is evidence.
Can-resume is capability.
How-it-looks is projection.
```

This keeps the same task observable from multiple clients without letting one viewer redefine task truth.

## Connectivity and Freshness

Freshness is part of control safety.

Required model:

```text
Fresh connection grants control projection.
Stale connection grants observation only.
Offline grants cache only.
```

Rules:

- no offline queue for critical actions;
- stale/offline disables `GuardedAction` / `DangerAction` / `TaskControl`;
- `PendingUnknown` exists when connection drops after sending intent but before result;
- reconnect requires `Resyncing`;
- `Connected` is not enough; `Fresh` is required.
- physical safety controls remain distinct from remote UI safety requests;
- a remote `DangerAction` must not be presented as available without a fresh confirmed control channel.

No freshness, no control.

## ProjectionBundle Delivery

`ProjectionBundle` is the delivery unit for projection and shell behavior.

Rules:

- critical UI is pinned/preinstalled;
- dynamic UI is signed/verified;
- runtime traffic is patches and intents, not full UI tree streaming;
- dynamic unchecked UI tree streaming is forbidden for critical surfaces;
- critical bundle updates require safe boundaries.

Critical UI is pinned.
Dynamic UI is verified.
Runtime UI is patched.

`ui-shell-kit` may serve as:

- reference shell;
- deterministic evidence surface;
- future `ProjectionBundle` player seed;
- patch applier;
- renderer substrate.

It is platform/internal infrastructure, not the required app authoring framework.

## Hard Non-Goals

This doctrine extension does not introduce:

- code changes;
- production UI wiring;
- `prom-ui` integration;
- Workbench dependency;
- verifier changes;
- VM changes;
- SemCode changes;
- runtime capability widening;
- renderer backend decision;
- promotion of `ui-shell-kit` to production UI;
- mandatory hand-written Rust UI;
- CSS/HTML-like layout language;
- offline queued critical control actions.

It also does not claim pixel-perfect mathematical proof across all renderers.

The correct contract wording is:

```text
UI behavior remains contractually traceable from semantic state to rendered projection.
```

## Close

Semantic UI DNA v2 preserves the original UI DNA and adds an Intent-Driven Projection model for zero-glue authoring.

The intended path is:

- meaning authored once in Semantic;
- projection authored as intent;
- UI IR compiled deterministically;
- shell plays patches and evidence;
- renderer draws pixels last.

This document is a doctrine extension, not an implementation plan.
