# Application-Completeness Benchmark Verdict

Status: closed evidence report
Program: Semantic application-completeness benchmark pack
Close-out item: PR-F4
Scope: benchmark evidence only

## Verdict

Semantic now supports benchmark-class application logic on the admitted deterministic surface.

This verdict is based on:

- a deterministic headless snake engine;
- a deterministic seeded Q-learning benchmark;
- an explicit external trace adapter contract;
- green check / run / compile / verify paths;
- green CI for the benchmark pack.

This verdict does not mean Semantic is public release.

This verdict does not widen the published stable contour.

This verdict does not make browser UI, DOM, Canvas, WebGL, native windows, file I/O, or argv part of the Semantic language boundary.

## Evidence Summary

| Evidence | Artifact | Status |
|---|---|---:|
| Deterministic headless snake engine | `examples/benchmarks/snake_core.sm` | ✅ landed |
| Snake core test | `tests/snake_core_benchmark.rs` | ✅ landed |
| Deterministic Q-learning benchmark | `examples/benchmarks/snake_learning.sm` | ✅ landed |
| Snake learning test | `tests/snake_learning_benchmark.rs` | ✅ landed |
| Trace adapter contract | `docs/roadmap/snake_trace_adapter_contract.md` | ✅ landed |
| Sample trace | `examples/benchmarks/snake_trace_sample.txt` | ✅ landed |
| Benchmark README | `examples/benchmarks/README.md` | ✅ landed |

## Landed PRs

| PR | Title | Result |
|---|---|---|
| PR-F1 | `examples/tests: add snake_core benchmark` | ✅ landed |
| PR-F2 | `examples/tests: add snake_learning benchmark` | ✅ landed |
| PR-F3 | `docs/examples: add snake trace adapter contract` | ✅ landed |
| PR-F4 | `reports/tests: publish application-completeness benchmark verdict` | ✅ close-out |

## Admitted Surface Exercised

The benchmark pack exercises the following admitted Semantic surfaces:

- `i32` arithmetic and relationals;
- `i32 / i32` and `i32 % i32`;
- `bool` logic;
- `text` literals and text concatenation;
- `to_text(...)`;
- `print(text)`;
- `let mut`;
- reassignment;
- `if`;
- `while`;
- `Sequence(i32)`;
- `len`;
- `contains`;
- `prepend`;
- `pop`;
- `Map(i32, i32)`;
- `map_empty`;
- `map_get`;
- `map_set`;
- `map_contains`;
- deterministic `random_seed`;
- deterministic `random_next_i32`;
- user-defined functions;
- assertions;
- check / run / compile / verify pipeline.

## Snake Core Evidence

Artifact:

```text
examples/benchmarks/snake_core.sm
```

Test:

```text
tests/snake_core_benchmark.rs
```

Expected deterministic output:

```text
snake_core: score=0 steps=200
```

Evidence:

- deterministic 10x10 headless snake engine;
- deterministic seed;
- body represented as `Sequence(i32)`;
- movement through `prepend` and `pop`;
- self-collision through `contains`;
- invariant: `len(snake) == 3 + score`;
- benchmark test performs `check / run / compile / verify`.

## Snake Learning Evidence

Artifact:

```text
examples/benchmarks/snake_learning.sm
```

Test:

```text
tests/snake_learning_benchmark.rs
```

Expected deterministic output:

```text
snake_learning: total_score=8 total_steps=1417 episodes=10
```

Evidence:

- deterministic 10-episode training loop;
- seeded PRNG using `episode * 137 + 42`;
- 5-bit state encoding;
- relative action policy: turn left / straight / turn right;
- Q-table represented as `Map(i32, i32)`;
- integer-scaled Q-values;
- greedy selection after bootstrap episode;
- golden assertions:
  - `total_score == 8`;
  - `total_steps == 1417`;
  - `episode == 10`;
- benchmark test performs `check / run / compile / verify`.

## Trace Adapter Evidence

Artifacts:

```text
docs/roadmap/snake_trace_adapter_contract.md
examples/benchmarks/snake_trace_sample.txt
examples/benchmarks/README.md
```

Evidence:

- one line-oriented trace format is defined;
- `frame ...` lines are specified;
- `summary ...` line is specified;
- cell encoding `cell = x + y * 10` is specified;
- external renderer boundary is explicit.

Boundary:

```text
Semantic emits deterministic text traces.
External renderer consumes those traces.
Renderer is not the Semantic runtime.
```

Browser, DOM, Canvas, WebGL, native windows, animation timing, file I/O, and argv remain outside the Semantic language boundary.

## Validation Commands

Required local validation:

```bash
cargo test -q --test snake_core_benchmark
cargo test -q --test snake_learning_benchmark
git diff --check
```

Recommended full validation:

```bash
cargo test --workspace
```

Manual smoke command for learning benchmark:

```powershell
cargo run -q -p smc-cli -- run examples/benchmarks/snake_learning.sm
```

Expected output:

```text
snake_learning: total_score=8 total_steps=1417 episodes=10
```

## What This Proves

This benchmark pack proves that Semantic can express, verify, and execute deterministic benchmark-class application logic involving:

- mutable state;
- control flow;
- collections;
- seeded randomness;
- persistent lookup state;
- text observation;
- stdout traces;
- end-to-end deterministic assertions.

The practical threshold crossed is:

```text
syntax/runtime demo
↓
verified benchmark-class application substrate
```

## What This Does Not Prove

This close-out does not claim:

- public release readiness;
- general-purpose language completeness;
- browser ownership;
- GUI ownership;
- file I/O support;
- argv support;
- async runtime support;
- full renderer support;
- production optimization;
- stable public API beyond the explicitly documented contour.

## Final Close-Out

The application-completeness benchmark pack is closed when this PR lands.

Final state:

```text
PR-F1 ✅ snake_core benchmark
PR-F2 ✅ snake_learning benchmark
PR-F3 ✅ trace adapter contract
PR-F4 ✅ benchmark verdict
```

Program status after merge:

```text
Application-completeness benchmark pack: CLOSED
```
