# CORE-QUAD-V1-BENCHMARK-CLOSEOUT

## Starting main commit

`ceee937b87e3b97519a9bbd2ff52fb727a499f31`

## Branch, issue, and parent

- Branch: `core-quad/v1-benchmark-closeout`
- Issue: #1412
- Parent: #1404

## Exact changed-file boundary

```text
.harness/current.task.yaml
.harness/reports/CORE-QUAD-V1-BENCHMARK-CLOSEOUT.md
crates/semantic-core-bench/src/lib.rs
docs/roadmap/core_quad/v1_benchmark_closeout.md
```

## Existing benchmark mechanism reviewed

`semantic-core-bench` already exposes the `core-bench` CLI, dispatches through
`run_benchmark`, uses `std::time::Instant`, and depends on
`semantic-core-quad`. The implementation reuses this mechanism without adding
a framework or dependency. Existing command behavior and the existing `all`
composition remain unchanged.

## New command and benchmark inventory

```text
cargo run -p semantic-core-bench --release -- quad-v1
```

The command reports:

- register NOT through scalar, SWAR, and default APIs;
- register AND through scalar, SWAR, and default APIs;
- default tile XOR;
- in-place XOR for `QuadroBank<64>`;
- in-place XOR for `QuadTileBank<32>`;
- dense/physical mask roundtrip;
- dense-mask iteration;
- relative register NOT and AND summaries.

The production iteration count is 100,000 API calls per sample.

## Deterministic vectors

The exact seven canonical raw vectors from PR #1483 are scheduled in a fixed
order. No randomness, current-time input, environment seed, or detected CPU
capability changes the operation set.

## Optimization-barrier policy

`std::hint::black_box` wraps benchmark inputs and consumed results. Every
timed iteration contributes to an emitted checksum. The checksum prevents
dead-code elimination; it is not a correctness or compatibility promise.
Fixed arrays are initialized outside bank hot loops, and no heap allocation
occurs inside the new timed loops.

## Relative and threshold policy

Relative ratios use `baseline elapsed time / candidate elapsed time`, with
elapsed nanoseconds clamped to at least one. A ratio above 1.0 describes only
that local run. There is no timing threshold, performance assertion,
benchmark-driven behavior selection, universal speedup claim, or cross-machine
comparison claim. No optional CPU feature is required.

## Qualification dependency and EQUIV

PR #1483's public integration matrix owns correctness. This closeout adds only
observational performance evidence. EQUIV remains excluded by the #1413
compatibility policy and is not part of the qualified v1 public API.

## Feature posture and core capsule

- `std`: tested.
- `no_std`: check-qualified through `--no-default-features`.
- `serde`: compile/test-qualified under `--all-features`.
- `semantic-core-capsule`: minimum downstream smoke consumer; 8 tests passed.

## Exact verification commands and results

```text
cargo +1.93.1 fmt --all --check                              PASS
cargo +1.93.1 clippy --workspace --all-targets -- -D warnings PASS
cargo test -p semantic-core-bench --quiet                    PASS (2 tests)
cargo +1.93.1 run --quiet -p semantic-core-bench --release -- quad-v1 PASS
release output required-label and finite-value validation    PASS
cargo test -p semantic-core-quad --test v1_qualification -- --nocapture PASS (9 tests)
cargo test -p semantic-core-quad --quiet                     PASS (134 inline, 9 integration)
cargo check -p semantic-core-quad --no-default-features      PASS
cargo test -p semantic-core-quad --all-features --quiet      PASS (134 inline, 9 integration)
cargo test -p semantic-core-capsule --quiet                   PASS (8 tests)
cargo test --test legacy_guards --quiet                       PASS (10 tests)
cargo test --all-targets --quiet                              PASS
git diff --check                                              PASS
pwsh -NoProfile -ExecutionPolicy Bypass -File scripts/harness-check.ps1 PASS
```

The release command completed and emitted every required benchmark group,
checksums, and finite relative summaries. Local numeric observations are not
recorded as normative values.

## #1412 acceptance mapping

| Criterion | Evidence |
| --- | --- |
| State encoding freeze | PR #1483 integration matrix |
| Truth-policy invariants | PR #1483 integration matrix |
| Scalar/SWAR equivalence | PR #1483 integration matrix |
| 4x4 delta matrix | PR #1483 integration matrix |
| Mask model | PR #1483 integration matrix |
| Tile/bank lifting | PR #1483 integration matrix |
| Core capsule smoke | Retained command, 8 passing tests |
| Minimal performance evidence | `quad-v1` release command |
| Relative-output policy | Relative summary implementation and closeout doc |
| Workspace qualification | Full local command set above and PR CI |

## Remaining work

- #1417 remains open for GPU transport representation.
- #1404 remains open for umbrella roadmap closeout.

## Explicit non-changes

No `semantic-core-quad` file, Cargo file, dependency, public Quad API, spec,
other crate, EQUIV surface, timing threshold, CPU-specific requirement,
visual/GPU code, VM/runtime behavior, or unrelated untracked file was modified
or staged.
