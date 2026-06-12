# Mission Control Architecture

## 1. Purpose

Mission Control is a visual observability subsystem for Semantic projects.

It receives semantic analysis events from Admission Guard, Verifier, Semantic Analyzer and future Semantic Studio services, then builds a canonical graph model and projects it into an interactive 3D scene.

## 2. High-level flow

```text
Semantic source / PR diff
        ↓
Admission Guard
        ↓
Semantic Analyzer
        ↓
Verifier / Invariant Scanner / Conflict Scanner
        ↓
Graph Event Emitter
        ↓
Mission Control Core
        ↓
Visual Projection Adapter
        ↓
3D Scene / Inspector / Timeline
```

## 3. Layer model

### 3.1 Producer layer

Event producers:

- Semantic parser
- Semantic resolver
- Admission Guard
- Verifier
- Invariant scanner
- Conflict scanner
- Effect boundary analyzer
- Proof-path tracer

These components emit semantic events. They should not know how the graph is rendered.

### 3.2 Canonical graph layer

The graph layer stores meaning, not visuals.

Canonical node examples:

- module
- namespace
- type
- function
- contract
- invariant
- rule
- effect
- guard
- proof
- conflict
- verdict
- artifact

Canonical edge examples:

- imports
- calls
- reads
- writes
- depends_on
- proves
- violates
- conflicts_with
- guards
- emits
- resolves_to
- contains

### 3.3 Visual projection layer

The visual layer maps semantic meaning to presentation:

- color
- size
- opacity
- pulse
- particles
- arrows
- layer depth
- camera focus
- inspector data

The visual layer is replaceable.

### 3.4 Rendering adapter

Possible v1 renderer:

```text
React + TypeScript + react-force-graph-3d + Three.js
```

Possible mature renderer:

```text
Semantic visual rules -> rendering adapter -> WebGL/WebGPU scene
```

## 4. v1 architecture

v1 is an external prototype.

```text
Admission Guard mock/real stream
        ↓ JSONL / WebSocket / IPC
Mission Control TypeScript graph store
        ↓
react-force-graph-3d
        ↓
3D window
```

v1 validates:

- event protocol
- 3D dependency graph UX
- live updates
- graph replay
- inspector behavior
- node/edge taxonomy
- semantic layer projection

## 5. v2 architecture

v2 is Semantic-native.

```text
MissionControl.semantic
        ↓
Semantic graph rules
        ↓
Semantic visual projection rules
        ↓
Renderer adapter
        ↓
3D scene
```

TypeScript/Rust may still exist as runtime infrastructure, but they must not own the semantic meaning.

## 6. 3D semantic layering

Default Z-axis projection:

```text
Z +300    Admission Guard verdicts
Z +200    Invariants / Contracts
Z +100    Functions / Rules
Z   0     Modules / Types
Z -100    Effects / External boundary
Z -200    Conflicts / Unknown / S-state
```

This makes the graph readable as a spatial causal model:

```text
upper layer  -> proof, admission, guarantees
middle layer -> source structure and logic
lower layer  -> effects, risks, conflicts
```

## 7. Visual semantics

| Semantic state | Visual behavior |
|---|---|
| pending | dim node |
| scanning | soft pulse |
| verified | stable glow |
| warning | yellow halo |
| conflict | red core / vibration |
| unknown | translucent node |
| admitted | final green arc |
| rejected | broken proof path / red outline |

## 8. Navigation contract

A graph node must be traceable back to code.

Example:

```text
node id: inv:no_overlap
file: src/booking.semantic
line: 128
symbol: invariant no_overlap
```

Clicking the node in Mission Control should open the corresponding source location in Semantic Studio.

## 9. Replay contract

Mission Control must support replay of a previous Admission Guard run.

Two data forms are required:

```text
event stream -> live visualization
snapshot     -> saved graph state
```

## 10. Design rule

Mission Control is not a UI decoration.

It is an engineering instrument for observing semantic causality, dependency activation, invariant proof, conflict emergence and Admission Guard decision paths.
