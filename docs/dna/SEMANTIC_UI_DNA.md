# Semantic UI DNA

Status: architecture doctrine
Track: POST-UI / Semantic UI Application Boundary
Scope type: documentation only

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

## Close

Semantic UI acknowledges the open-source ecosystem honestly while preserving its own architecture.

It studies systems.
It does not become them.
