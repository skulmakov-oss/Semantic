# Shell Player v0 Boundary Contract

Status: NORMATIVE BOUNDARY FREEZE
Track: UI-DNA2-9A1
Scope: documentation-only ownership and stage contract

Implementation: NOT AUTHORIZED
Rust types: NOT AUTHORIZED
Public API: NOT AUTHORIZED
Bundle activation: NOT AUTHORIZED
Patch application: NOT AUTHORIZED
Interaction runtime: NOT AUTHORIZED
Renderer integration: NOT AUTHORIZED
Backend integration: NOT AUTHORIZED
Gate D: CLOSED
Production promotion: NOT AUTHORIZED

## 1. Purpose

This document freezes the ownership and stage boundary of the future
`prom-ui-runtime::shell_player` module.

It does not define its Rust representation, public API, concrete state model,
patch algorithm, focus algorithm, hit-test algorithm, accessibility encoding,
draw-command encoding, or backend integration.

Shell Player owns local projection playback.

Shell Player does not own Semantic truth.

## 2. Governing stage relationship

```text
verified ProjectionBundle
  → inert bundle loading
  → separate bundle activation decision
  → authorized use of a read-only inert bundle representation
  → Shell Player session construction
  → deterministic local shell transitions
  → backend-neutral draw/session material
  → renderer/backend processing
```

The Shell Player may consume a read-only inert bundle representation only
after a separately owned activation decision authorizes its use.

The Shell Player does not parse, validate, verify, load or activate a ProjectionBundle.

```text
parsed bundle != verified bundle
verified bundle != inert loaded bundle
inert loaded bundle != activated shell session
activation decision != production promotion

local shell state != Semantic truth
patch application != Semantic mutation
hit-test result != action authorization
ActionIntent candidate != admitted action
draw material != pixels
shell transition != backend event loop
```

## 3. Conceptual inputs

The future Shell Player may accept only these conceptual input classes:

- a read-only inert bundle representation;
- a separately owned activation result or authorization context;
- normalized backend-neutral interaction input;
- ordered inert `ProjectionPatch` input;
- caller-supplied deterministic session and viewport context.

This contract does not define Rust types or name a final activation-token
representation.

The Shell Player must not acquire inputs through:

- filesystem access;
- network access;
- the system clock;
- randomness;
- live Semantic reads or subscriptions;
- capability evaluation;
- admission policy;
- native backend polling.

## 4. Conceptual outputs

The future Shell Player may produce only:

- next local shell state;
- focus realization;
- hit-test realization;
- accessibility realization;
- backend-neutral draw/session material;
- an optional `ActionIntent` candidate;
- deterministic diagnostics.

An output is projection/runtime material only.

No output becomes Semantic truth, admission evidence, an accepted action,
renderer pixels, or production-promotion evidence merely because the Shell
Player produced it.

## 5. Local state ownership

The future Shell Player may own:

- local focus state;
- local hover and pressed state;
- local pointer or interaction capture state;
- local projection-playback cursor;
- local surface/session lifecycle state;
- local hit-test realization;
- local accessibility realization;
- local projected-value cache;
- local invalidation or damage bookkeeping;
- backend-neutral draw-command production.

Projected-value caches remain non-authoritative display state. They do not
become sources of Semantic truth.

The Shell Player must not own:

- Semantic state;
- task truth;
- connectivity truth;
- freshness truth;
- capability policy;
- admission decisions;
- action acceptance;
- action denial policy;
- bundle trust;
- bundle activation policy;
- production promotion;
- renderer pixel authority;
- native backend lifecycle.

## 6. ProjectionPatch boundary

The Shell Player is the future owner of `ProjectionPatch` application to local
shell state.

This ownership does not authorize implementation in UI-DNA2-9A1.

`ProjectionPatch` remains inert contract data until a separately authorized
Shell Player implementation applies it.

Patch application may update local projection state only.

Patch application must not mutate Semantic state, admission state, capability
state, task-engine state, or native backend state.

These decisions remain unresolved:

- batch transaction model;
- `Atomic` versus `OrderedPartial` runtime semantics;
- revision and replay-cursor representation;
- stale-patch rejection policy;
- unknown-target handling;
- unknown-operation handling;
- rollback representation;
- diagnostic namespace;
- numeric resource defaults.

## 7. Interaction and action boundary

```text
normalized interaction
  → local hit testing and focus handling
  → local route lookup
  → optional ActionIntent candidate
  → separately owned admission boundary
```

The Shell Player may construct an `ActionIntent` candidate.

The Shell Player does not admit, accept, deny, or dispatch an action.

Local route lookup does not grant authority. A click, key press, focus event,
or hit-test match is not action admission.

## 8. Accessibility and draw seam

Shell Player owns accessibility realization of the active local projection.

Shell Player emits backend-neutral draw/session material through the
separately owned draw seam.

Renderer and backend layers own pixel production and native event-loop work.

```text
accessibility realization != Semantic truth
draw command != renderer authority
draw seam != backend implementation
```

This contract does not select:

- an accessibility-tree encoding;
- a draw-command encoding;
- a layout algorithm;
- a font system;
- a text-shaping implementation;
- a GPU API;
- a windowing API;
- a native event-loop API.

## 9. Determinism and resource posture

A future implementation must be deterministic for identical:

- inert bundle representation;
- activation context;
- local shell state;
- normalized interaction input;
- ordered patch input;
- session/viewport context;
- caller-supplied limits.

It must not depend on:

- wall-clock time;
- absolute filesystem paths;
- host-generated IDs;
- nondeterministic map iteration;
- OS-specific ordering;
- ambient global state.

Future caller-supplied limit categories are:

- maximum active nodes;
- maximum focusable nodes;
- maximum hit-test entries;
- maximum accessibility nodes;
- maximum patches per transition;
- maximum draw commands per transition;
- maximum diagnostic count;
- maximum projected text bytes;
- maximum local session-state bytes.

UI-DNA2-9A1 assigns no numeric values.

## 10. Evidence posture

`experiments/ui-shell-kit` is experimental evidence only.

Calculator shell documents and snapshots are reference evidence only.

Experimental behavior may inform future design after explicit review.
Experimental behavior is not automatically promoted into the canonical Shell
Player contract.

## 11. Explicit non-goals

- no Rust implementation;
- no `prom-ui-runtime::shell_player` module creation;
- no public API;
- no `ShellSession` type;
- no patch application;
- no bundle activation;
- no `ActionIntent` runtime emission;
- no admission integration;
- no dispatcher integration;
- no renderer integration;
- no backend integration;
- no event loop;
- no Workbench integration;
- no Semantic Studio integration;
- no Gate D movement;
- no production promotion.

## 12. Unresolved decisions blocking implementation

- exact activated-session input contract;
- exact local shell-state representation;
- patch transaction and rollback model;
- patch revision and replay-cursor model;
- focus traversal policy;
- pointer capture policy;
- hit-test coordinate model;
- layout ownership details;
- accessibility realization encoding;
- draw-seam contract;
- draw-command encoding;
- `ActionIntent` emission contract;
- diagnostic namespace;
- caller-supplied resource-limit contract;
- error and partial-transition policy.

## 13. Ownership and dependency posture

```text
Semantic owns meaning.
Projection owns presentation intent.
UI IR owns structure.
Shell owns local projection playback and rendering preparation.
Renderer owns pixels.
```

Required dependency direction:

```text
prom-ui-backend-native
  → prom-ui-runtime
  → prom-ui
```

Forbidden dependency direction:

```text
prom-ui
  → prom-ui-runtime
```

## 14. Final status

UI-DNA2-9A1 freezes the ownership and stage boundary only.

UI-DNA2-9A1 does not authorize Shell Player implementation.

```text
Shell Player implementation: NOT AUTHORIZED
ProjectionPatch application: NOT AUTHORIZED
Bundle activation: NOT AUTHORIZED
Renderer integration: NOT AUTHORIZED
Backend integration: NOT AUTHORIZED
Gate D: CLOSED
Production promotion: NOT AUTHORIZED
NEXT AUTHORIZED IMPLEMENTATION SLICE = NONE
```

Gate D remains closed.

NEXT AUTHORIZED IMPLEMENTATION SLICE = NONE.
