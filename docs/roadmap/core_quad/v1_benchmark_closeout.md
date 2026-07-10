# Quad Logic Engine v1 Benchmark and Qualification Closeout

## Purpose

Complete the v1 qualification track with deterministic, observational relative
benchmarks using the workspace's existing benchmark CLI.

## Issue

- Closes #1412.
- Parent: #1404.

## Starting main commit

`ceee937b87e3b97519a9bbd2ff52fb727a499f31`

## Existing benchmark mechanism

The owner is `semantic-core-bench`, exposed as the `core-bench` CLI. It already
uses `std::time::Instant` and already depends on `semantic-core-quad`; no
external benchmark framework or dependency is added.

## Command

```text
cargo run -p semantic-core-bench --release -- quad-v1
```

The command also remains runnable in debug mode and reports the active build
mode in its header.

## Benchmark inventory

- Register NOT: scalar, SWAR, and default APIs.
- Register AND: scalar, SWAR, and default APIs.
- Default tile XOR lifting.
- In-place XOR for `QuadroBank<64>`.
- In-place XOR for `QuadTileBank<32>`.
- Dense-to-physical-to-dense mask conversion.
- Dense-mask iteration with every visited lane consumed.

Each sample reports API calls as `ops`, elapsed nanoseconds, `ops/s`, `ns/op`,
the applicable structural rate, and a checksum.

## Canonical deterministic vectors

The register schedule uses the same seven vectors as PR #1483:

```text
0x0000_0000_0000_0000
0xFFFF_FFFF_FFFF_FFFF
0x5555_5555_5555_5555
0xAAAA_AAAA_AAAA_AAAA
0x0123_4567_89AB_CDEF
0xE4E4_E4E4_E4E4_E4E4
0xBADC_0FFE_DEAD_BEEF
```

Scheduling is deterministic. Randomness, wall-clock input, environment seeds,
and CPU-capability-dependent operation selection are excluded.

## Relative comparison policy

The register summaries report `swar_vs_scalar` and `default_vs_scalar`. Each
ratio is:

```text
baseline elapsed time / candidate elapsed time
```

A ratio above 1.0 means only that the candidate was faster in that specific
run. It is not a guaranteed speedup. Elapsed nanoseconds are clamped to at
least one so ratios remain finite.

## Checksum and black-box policy

Inputs and consumed results cross `std::hint::black_box`. Every timed iteration
contributes to an emitted checksum. These are optimization barriers, not
correctness, compatibility, or cryptographic promises.

## Performance boundaries

There is no timing threshold and no performance assertion. Output is local
diagnostic evidence only. It is not a cross-machine baseline and makes no
stability claim across hardware, operating systems, toolchains, thermal
states, or background load. No optional CPU feature is required.

## Relationship to PR #1483

PR #1483's downstream integration matrix owns correctness for encoding,
truth-policy invariants, scalar/SWAR/default equivalence, masks, deltas, tiles,
banks, and lattice separation. This benchmark owns only observational
performance evidence and does not select runtime behavior from timings.

## EQUIV deferral

EQUIV is excluded by the closed #1413 compatibility policy and is not part of
the qualified v1 public API. Its absence is policy, not unfinished benchmark
coverage.

## Feature posture and core capsule

- `std`: tested.
- `no_std`: check-qualified through `--no-default-features`.
- `serde`: compile/test-qualified under `--all-features`.
- `semantic-core-capsule`: retained as the minimum downstream smoke consumer.

## #1412 acceptance mapping

| Criterion | Evidence |
| --- | --- |
| State encoding freeze | PR #1483 integration matrix |
| Truth-policy invariants | PR #1483 integration matrix |
| Scalar/SWAR equivalence | PR #1483 integration matrix |
| 4x4 delta matrix | PR #1483 integration matrix |
| Mask model | PR #1483 integration matrix |
| Tile/bank lifting | PR #1483 integration matrix |
| Core capsule smoke | Retained verification command |
| Minimal performance evidence | `quad-v1` benchmark command |
| Relative-output policy | This closeout and relative summary lines |
| Workspace qualification | Local and CI checks for the closeout PR |

## Explicit non-claims

This closeout does not claim stable absolute throughput, universal speedups,
cross-machine comparability, benchmark-driven runtime selection, serialized
cross-version compatibility, EQUIV support, GPU transport, or completion of
#1417 or #1404.

## Remaining core-quad work

- #1417: GPU transport representation and visual adapter boundary.
- #1404: umbrella roadmap closeout.
