# Semantic UI DNA

Status: architecture doctrine
Track: POST-UI / Semantic UI Application Boundary
Scope type: documentation only

## Doctrine Extension

This document is extended by:

- [`SEMANTIC_UI_DNA_v2.md`](SEMANTIC_UI_DNA_v2.md) - Intent-Driven Projection, zero-glue authoring, UI IR, Action IR, ProjectionBundle delivery, freshness, denial/recovery projection, and multi-client projection.

The v2 doctrine extends this document. It does not replace the original Semantic UI ownership, renderer boundary, authority non-transfer, Quad-state, evidence/trace, and UI state separation principles.

## Core Principle

Semantic UI is a Semantic-native UI architecture.

It is not a fork, clone, wrapper, or derivative UI toolkit.

Semantic UI may study proven open-source UI systems, but Semantic owns its own:

- UI Tree;
- UI AST;
- UI IR;
- state/update/event model;
- capability/effect discipline;
- diagnostics and fault model;
- renderer adapter contract.

## Formula

```text
Semantic UI =
  Semantic-owned UI model
+ Semantic-owned contracts
+ Semantic-owned state/update model
+ Semantic UI AST
+ Semantic UI IR
+ capability/effect rules
+ renderer adapters

External projects =
  architectural references, not owners
```

## Lineage

Semantic UI draws architectural inspiration from:

| Source | Studied area | Accepted influence | Explicit non-adoption |
| --- | --- | --- | --- |
| Slint | declarative UI language, components, properties, callbacks | declarative UI genes, property binding, renderer separation | no `.slint` language copy, no runtime adoption, no license obligations without audit |
| Lapce / Floem | Rust-native editor/workbench architecture | editor/workbench structure, state/layout ideas, performance expectations | no full editor fork, no forced lifecycle adoption |
| Makepad | live design and Rust UI runtime ideas | live design feedback loop, visual iteration, rendering architecture ideas | no runtime import as Semantic owner |
| Zed / GPUI | high-performance editor UX | command routing, workspace/panel UX, editor responsiveness | no GPUI ownership of Semantic UI |
| Tauri | Rust backend plus frontend shell model | shell/bridge pattern, IPC inspiration, MVP adapter idea | no WebView/browser ownership of Semantic state |
| Monaco / CodeMirror | mature code editor surface | temporary editor-layer interaction patterns | no web-only IDE commitment |
| React Flow / Cytoscape / ELK | graph and layout interaction patterns | subgraph, compound node, graph layout ideas | no ownership of Semantic GraphStore |

## Inspiration vs Dependency vs Derivative

Semantic UI distinguishes three levels:

| Level | Meaning | Required handling |
| --- | --- | --- |
| Inspiration | idea, pattern, architecture principle | record in `third_party_influence.md` |
| Dependency | actual crate/npm/library used by the repo | record in `third_party_dependencies.md` with license notes |
| Derivative / fork | copied or modified code | requires explicit license compliance and copyright notices |

Target posture:

```text
Mostly inspiration.
Some dependencies only when deliberately admitted.
No silent derivative code.
```

## Semantic Ownership

Semantic UI owns:

- UI Tree;
- UI AST;
- UI IR;
- state/update/event model;
- action/effect model;
- capability admission;
- diagnostics;
- graph/fault localization model;
- renderer contract.

Renderer backends may exist, but they do not own the Semantic UI model.

## Renderer Boundary

Renderer adapters are implementation boundaries.

Allowed future backend families may include:

- terminal/text prototype;
- HTML/SVG adapter;
- Canvas adapter;
- native adapter;
- GPU-backed adapter.

None of these becomes the Semantic language boundary by default.

## Non-Goals

Semantic UI does not claim:

- browser/DOM ownership;
- WebView ownership;
- widget framework adoption;
- Slint/Floem/Makepad/Zed/Tauri runtime adoption;
- GPU/shader pipeline ownership;
- CSS/layout engine ownership;
- copied third-party code;
- public release widening;
- that external UI projects are dependencies unless listed as dependencies.

## Design Rule

```text
Semantic defines the model.
Renderer adapts to Semantic.
Semantic does not adapt itself to a foreign UI lifecycle.
```

## Self-hosted App Shell DNA

Semantic UI is not merely a UI model; it must become the foundation from which Semantic-authored application shells can be defined.

Workbench and Semantic Studio must not become standalone product-level applications until Semantic can define and drive application UI shells through Semantic-owned UI model, UI AST / IR, admission rules, and renderer adapter contract.

Workbench and Semantic Studio are future Semantic UI applications, not external centers of gravity.

This strengthens the existing `#675` pause and does not close or weaken it.

## Bootstrap Tooling DNA

Temporary tooling may exist during bootstrap.

Temporary Workbench surfaces may remain only as bounded presentation, orchestration, diagnostics, reports, and documentation tooling.

Such tooling is scaffolding, not product UI.

Temporary tooling must not become Semantic UI owner, semantic authority, release authority, Studio substitute, or Local Admission Guard replacement.

React and Tauri may exist as current tooling shell dependencies, but they must not become the strategic Semantic UI architecture.

## Authority Non-Transfer DNA

UI may display truth. UI does not become truth.

No UI surface may acquire authority over:

- semantic meaning;
- verifier admission;
- VM/runtime behavior;
- compiler/parser/typechecker behavior;
- release readiness;
- Local Admission Guard;
- canonical documentation truth;
- Semantic UI model ownership;
- GitHub CI authority.

UI presents evidence and projections.

UI does not become the source of semantic, release, or admission truth.

## Quad-State UI DNA

Semantic UI must preserve uncertainty and conflict as first-class visible states.

- `N` - unknown
- `F` - false
- `T` - true
- `S` - conflict

unknown is not absent.

conflict is not merely failure.

denied is not false.

not admitted is not equivalent to invalid source.

UI must not flatten Quad-state meaning into ordinary boolean UI status.

## Evidence / Trace DNA

Every meaningful UI claim should be traceable to source, admission, diagnostic, runtime, repository, or governance evidence.

Distinguish:

- raw output
- repository document
- verifier result
- admission verdict
- diagnostic source
- runtime observation
- cached UI projection
- UI interpretation

UI should make evidence provenance visible where practical.

UI claims without evidence must not be presented as authoritative truth.

## UI State Separation DNA

Distinguish:

- Semantic state
- runtime state
- admission state
- repository truth
- UI state
- presentation cache
- view-model projection

UI state is projection/cache, not semantic state.

UI state may help interaction and presentation.

UI state must not redefine Semantic state or repository truth.

## Fault / Denial / Recovery DNA

Fault, denial, conflict, and recovery are first-class UI states.

- denial
- quarantine
- rollback
- conflict
- partial admission
- capability rejection
- runtime fault
- diagnostic uncertainty

These states must not be hidden behind generic success/failure UI.

Semantic UI should preserve the reason, boundary, and evidence for each denial or fault when available.

## Operator Surface DNA

Semantic UI is an operator surface, not an authority replacement.

UI assists the operator.

UI does not replace admission, verification, governance, or Local Admission Guard.

The operator should be able to see what happened, why it happened, what evidence exists, what was blocked, what is allowed, and what is forbidden.

## Semantic UI Maturity Ladder

1. UI model names and inert types.
2. UI Tree.
3. UI AST.
4. UI IR.
5. capability/effect admission model.
6. diagnostics/fault model.
7. renderer adapter contract.
8. Semantic-authored shell definition.
9. bounded app shell prototype.
10. Workbench / Studio only later as Semantic UI applications.

Workbench / Studio product work before Semantic-authored shell capability is forbidden unless explicitly superseded by governance.

Future model seed work belongs only to foundation stages, not product application stages.

## R12 Expanded Formula

```text
Semantic UI =
  Semantic-owned UI model
+ UI Tree
+ UI AST
+ UI IR
+ state/update/event discipline
+ capability/effect admission
+ quad-state visibility
+ evidence/trace discipline
+ fault/denial/recovery visibility
+ diagnostics/fault model
+ renderer adapter contract
+ self-hosted app shell capability
+ strict authority non-transfer
```

## Close

Semantic UI acknowledges the open-source ecosystem honestly while preserving its own architecture.

It studies systems.
It does not become them.
