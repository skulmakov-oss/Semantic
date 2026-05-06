# Snake Trace Adapter Contract

Status: PR-F3 contract
Program: Semantic application-completeness benchmark pack
Scope type: docs/examples only

## Purpose

This document defines how Semantic snake benchmarks may feed an external visual renderer without making browser, DOM, HTML, CSS, Canvas, WebGL, native windows, or GUI lifecycle part of the Semantic language boundary.

Semantic owns:

- deterministic program execution;
- benchmark logic;
- state transitions;
- admitted runtime surface;
- text trace emission through `print(text)`;
- golden values and assertions;
- `check / run / compile / verify` validation path.

External adapters own:

- reading emitted trace lines;
- parsing frame records;
- rendering cells, snake body, food, score, and step count;
- animation timing;
- browser or desktop UI lifecycle;
- visual styling.

## Boundary rule

```text
Semantic trace is data.
Renderer is an adapter.
Renderer is not the Semantic runtime.
```

## First-wave trace format

The first-wave trace is line-oriented text.

One frame is one line:

```text
frame step=<i32> score=<i32> head=<i32> food=<i32> snake=<cell0,cell1,...>
```

Example:

```text
frame step=0 score=0 head=55 food=42 snake=55,54,53
frame step=1 score=0 head=56 food=42 snake=56,55,54
```

Final summary line:

```text
summary score=<i32> steps=<i32> status=<text>
```

Example:

```text
summary score=0 steps=200 status=completed
```

## Cell encoding

The first-wave snake benchmarks use a 10x10 grid.

Cell encoding:

```text
cell = x + y * 10
```

Where:

- `x` is in `[0, 9]`;
- `y` is in `[0, 9]`;
- `cell` is in `[0, 99]`.

Examples:

```text
x=5, y=5 -> cell=55
x=0, y=0 -> cell=0
x=9, y=9 -> cell=99
```

The renderer may decode with:

```text
x = cell % 10
y = cell / 10
```

## Trace fields

### `frame`

Required fields:

| Field   | Type                       | Meaning                                                              |
| ------- | -------------------------- | -------------------------------------------------------------------- |
| `step`  | `i32`                      | zero-based frame/step number                                         |
| `score` | `i32`                      | score after or at this frame, depending on emitting program contract |
| `head`  | `i32`                      | encoded head cell                                                    |
| `food`  | `i32`                      | encoded food cell                                                    |
| `snake` | comma-separated `i32` list | snake body cells, head first                                         |

### `summary`

Required fields:

| Field    | Type   | Meaning          |
| -------- | ------ | ---------------- |
| `score`  | `i32`  | final score      |
| `steps`  | `i32`  | final step count |
| `status` | `text` | terminal status  |

Recommended status values:

```text
completed
dead
max_steps
sample
```

## Determinism requirements

A trace is deterministic when all of these are fixed:

- Semantic source file;
- SemCode output;
- runtime configuration;
- seed value;
- capability context;
- CLI/runtime version.

The same fixed inputs must produce the same trace lines in the same order.

## Adapter requirements

An external adapter may:

- read stdout;
- read a saved trace text file produced outside Semantic;
- parse `frame ...` lines;
- parse one final `summary ...` line;
- render a 10x10 grid;
- render snake body cells;
- render food cell;
- display score and step count;
- animate frames using its own local timing.

An external adapter must not require:

- Semantic DOM access;
- Semantic browser APIs;
- Semantic Canvas/WebGL APIs;
- Semantic file I/O;
- Semantic argv;
- hidden host calls;
- non-deterministic renderer feedback into Semantic.

## Contract discipline

The trace contract is one-way:

```text
Semantic program -> stdout text trace -> external adapter
```

The adapter must not mutate Semantic runtime state.

The adapter must not be treated as part of the verified Semantic execution boundary.

## Relation to existing benchmarks

`examples/benchmarks/snake_core.sm` proves deterministic game-state logic.

`examples/benchmarks/snake_learning.sm` proves deterministic seeded training logic.

This contract defines how a future trace-producing benchmark may expose frame data for visualization.

This PR does not modify either benchmark.

## Future trace-producing example

A future `examples/benchmarks/snake_trace.sm` may emit lines matching this contract using only already-admitted Semantic features:

- `print(text)`;
- `to_text(...)`;
- `text + text`;
- `i32` arithmetic;
- `Sequence(i32)`;
- `Map(i32, i32)`;
- deterministic PRNG;
- `while`;
- `if`;
- `let mut`;
- reassignment.

Until such an example lands, this document is the adapter contract, not a runtime claim.

## Non-goals

PR-F3 must not introduce:

- file I/O;
- argv;
- renderer runtime;
- browser ownership;
- DOM API;
- Canvas API;
- WebGL API;
- native window API;
- async event loop;
- timing/sleep;
- animation clock;
- bidirectional UI feedback;
- new Semantic syntax;
- new SemCode opcode;
- new capability.

## Acceptance criteria

This PR is complete when:

- `docs/roadmap/snake_trace_adapter_contract.md` exists;
- `examples/benchmarks/snake_trace_sample.txt` exists;
- `examples/benchmarks/README.md` points to the contract;
- `docs/roadmap/application_completeness_pr_ledger.md` marks PR-F3 as landed after merge;
- no runtime/parser/typechecker/lowering/verifier/VM files are changed;
- `git diff --check` passes;
- CI is green.
