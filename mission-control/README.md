# Mission Control

Mission Control is the visual observability layer for the Semantic ecosystem.

Its purpose is to show, in real time, how Semantic code, dependencies, invariants, contracts, effects, conflicts, proof paths and Admission Guard verdicts interact with each other.

## Core idea

Mission Control must not be just a dependency viewer.

It should behave as a live 3D semantic instrument:

```text
PR / local change
  -> Admission Guard
  -> Semantic analysis events
  -> canonical graph model
  -> 3D visual projection
  -> inspector / replay / navigation back to code
```

## Version strategy

### v1: external prototype

Mission Control v1 is a technical prototype used to validate the visual model before deep integration into Semantic Studio.

Expected stack:

```text
Desktop shell:        Tauri
Frontend:             React + TypeScript
3D graph:             react-force-graph-3d / Three.js
Transport:            JSONL, WebSocket, or Tauri IPC
Input:                Admission Guard event stream
```

v1 is allowed to use TypeScript/Rust infrastructure because its goal is to test UX, live graph rendering, event protocol and replay behavior.

### v2: Semantic-native module

Mission Control v2 must be written in Semantic.

TypeScript, Rust, WebGL and desktop infrastructure may remain as adapters, but the meaning layer must be Semantic-native:

```text
Mission Control v2
  = Semantic graph logic
  + Semantic visual projection rules
  + Semantic event interpretation
  + external rendering adapter
```

This means the renderer displays the graph, but Semantic owns the meaning.

## Non-negotiable principle

The 3D scene is not the source of truth.

```text
Semantic meaning
  -> canonical graph model
  -> visual projection rules
  -> 3D scene
```

Never the opposite.

## Planned capabilities

- Live 3D dependency graph
- Free camera rotation and zoom
- Admission Guard run visualization
- Invariant and contract tracing
- Conflict and unknown-state highlighting
- Proof-path visualization
- Effect boundary visualization
- Node/edge inspector
- Navigation back to Semantic source code
- Run snapshots and replay
- Future Semantic Studio plugin integration

## Directory map

```text
mission-control/
├─ README.md
├─ docs/
│  ├─ architecture.md
│  └─ roadmap.md
├─ protocol/
│  └─ semantic-graph-event.schema.json
├─ examples/
│  └─ mock-admission-run.jsonl
└─ semantic/
   └─ MissionControl.semantic
```

## Working name

Mission Control is the current working name. Future names may include Semantic Observatory, ProofScope, Dependency Atlas, Verifier Lens or Control Tower.
