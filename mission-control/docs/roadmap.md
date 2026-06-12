# Mission Control Roadmap

## Phase 0 — Foundation

Goal: define Mission Control as a future Semantic Studio observability module.

Deliverables:

- directory skeleton
- README
- architecture document
- graph event schema
- mock Admission Guard stream
- first Semantic-native v2 sketch

## Phase 1 — v1 external prototype

Goal: build a standalone 3D visualizer.

Scope:

- read JSONL mock event stream
- build canonical graph state
- render graph in 3D
- support free camera rotation
- show node and edge inspector
- show run status
- support replay from event stream

Suggested stack:

```text
Tauri + React + TypeScript + react-force-graph-3d
```

Acceptance criteria:

- a mock Admission Guard run builds the graph live
- nodes appear progressively
- edges show direction and relation type
- conflict nodes are visually distinct
- clicking a node opens inspector data
- replay produces the same graph state deterministically

## Phase 2 — real Admission Guard integration

Goal: connect Mission Control to real Semantic analysis output.

Scope:

- Admission Guard emits graph events
- event stream includes file/line metadata
- graph captures dependencies, invariants, effects and conflicts
- final verdict appears as a graph node
- run snapshot can be saved

Acceptance criteria:

- a real PR/check run can be visualized
- every node can trace back to a Semantic source location
- conflict and warning states are represented correctly
- graph can be replayed after the run

## Phase 3 — Semantic Studio plugin shell

Goal: embed Mission Control as a plugin-like panel in Semantic Studio.

Scope:

- plugin host adapter
- workspace integration
- editor navigation
- diagnostics integration
- Admission Guard run selector
- graph panel inside Studio

Acceptance criteria:

- clicking a graph node navigates to source code
- selecting a symbol in editor can focus its graph node
- Admission Guard runs can be viewed inside Studio

## Phase 4 — Mission Control v2 on Semantic

Goal: rewrite Mission Control logic as Semantic-native rules.

Scope:

- MissionControl.semantic
- semantic graph construction rules
- visual projection rules
- event interpretation rules
- conflict and proof-path observers
- adapter boundary to renderer

Acceptance criteria:

- graph logic is expressed in Semantic
- visual projection decisions are expressed in Semantic
- renderer acts only as a technical adapter
- event replay is deterministic through Semantic rules

## Phase 5 — mature Semantic Studio module

Goal: turn Mission Control into a native observability subsystem.

Scope:

- persistent graph snapshots
- graph diff between runs
- proof-path replay
- conflict causality tracing
- CI/PR viewer mode
- plugin API stabilization
- optional advanced rendering backend

Acceptance criteria:

- Mission Control can compare two Admission Guard runs
- proof and conflict paths are explainable
- graph snapshots are reproducible
- Studio and CI views share the same semantic graph model

## Naming notes

Current working name: Mission Control.

Possible future internal modes:

- Dependency Atlas
- ProofScope
- Conflict Radar
- Admission Timeline
- Verifier Lens
- Semantic Observatory
