# Pulsar Roadmap

Status: v0 substrate baseline closed; P4 shadow equivalence closed and evidence-repaired; P5-A blocked by current profiling evidence
Owner: runtime acceleration / packed-state substrate
Scope type: documentation only

## Purpose

Pulsar is the code name for the fast Semantic compute core for packed quad-state operations.

Main goal:

- give Semantic a fast low-level substrate for mass processing of N/F/T/S states;
- do so without violating the active Core Trust Freeze contour;
- keep the v0 substrate line closed before any shadow-adapter implementation work begins.

Pulsar does not replace the current `sm-*` crate owners.

Current authority boundaries remain:

| Area | Owner |
| --- | --- |
| SemCode format / decode | `sm-format` |
| verifier admission | `sm-verify` |
| VM execution mechanics | `sm-vm` |
| runtime vocabulary | `sm-runtime-core` |
| CLI orchestration | `smc-cli` |
| fast packed-state substrate | `ton618-core` / future `pulsar-*` |

## Pulsar v0 Baseline Closeout

The Pulsar v0 substrate baseline is now merged through the crate-root export closure.

Merged slices:

- `#1193` - register Pulsar implementation roadmap;
- `#1194` - seed Quadro packed-state engine;
- `#1195` - expand Quadro correctness matrix;
- `#1196` - add Quadro microbench harness;
- `#1207` - record Quadro microbench baseline;
- `#1208` - clarify scalar Quadro hot path timings;
- `#1209` - export Quadro substrate at `ton618-core` crate root.

This closeout is documentation only.
It does not claim release readiness, production readiness, full no_std qualification, or any widening of the active Core Trust Freeze contour.

## Architecture Role

Pulsar is not a new language, verifier, or VM.

Pulsar is an internal kernel substrate:

```text
Semantic / SemCode / VM
        -> runtime operation
        -> Pulsar adapter
        -> packed quad-state kernels
```

Main tasks:

- packed quadit storage;
- fast mask extraction;
- conflict scan;
- delta scan;
- batch merge / intersect / inverse;
- SoA event output;
- future acceleration backend for runtime state propagation.

## Must Not Become

Pulsar must not become a second owner of:

- SemCode format;
- verifier admission;
- VM execution authority;
- CLI public contract;
- source language semantics;
- ownership precision beyond the current conservative contour.

Forbidden claims:

- Pulsar is the new Semantic VM.
- Pulsar replaces `sm-vm`.
- Pulsar owns SemCode.
- Pulsar proves full symbolic ownership.
- Pulsar makes Semantic release-ready.
- Pulsar completes no_std qualification.

Correct wording:

- Pulsar is an internal fast packed-state substrate candidate for Semantic runtime acceleration.

## Core Data Model

### Quadit Encoding

```text
N = 00
F = 01
T = 10
S = 11
```

One `u64` stores 32 quadits:

```text
QuadroReg = 64 bits = 32 x 2-bit states
```

### Bit-Plane Interpretation

- LSB plane = `F` bit
- MSB plane = `T` bit

Then:

- `N` = no bits
- `F` = `F` bit
- `T` = `T` bit
- `S` = `F` bit + `T` bit

### Core Operations

| Operation | Meaning | Low-level op |
| --- | --- | --- |
| merge | join / accumulate | OR |
| intersect | meet / filter | AND |
| inverse | swap T/F | bit-plane swap |
| mask_n | unknown/null states | `!F & !T` |
| mask_f | strict false | `F & !T` |
| mask_t | strict true | `T & !F` |
| mask_s | conflict/super | `F & T` |
| delta | state transition events | mask difference |

## Repository Layout

Start without crate sprawl.

### Phase 1 Layout

```text
crates/
  ton618-core/
    src/
      lib.rs
      ids.rs
      source.rs
      diagnostics.rs
      arena.rs
      sigtable.rs
      quadro.rs        # Pulsar seed module
    benches/
      quadro_logic.rs  # optional after Q1
```

### Future Layout, if Pulsar Grows

```text
crates/
  pulsar-core/       # packed quad-state primitives
  pulsar-kernels/    # batch kernels
  pulsar-bench/      # benchmark harness
  pulsar-adapter/    # integration adapters into sm-runtime-core/sm-vm
```

Do not create separate crates in the first phase unless needed.

## Milestones

### P0 - Kernel Boundary Audit

Type: audit / code inspection
Scope: no behavior changes

Goal:

- confirm Pulsar develops as a substrate, not a new authority owner.

Check:

- where `ton618-core` lives today;
- which modules already exist;
- what counts as primitive substrate;
- where naming confusion can occur;
- which crates must not be touched in the first phase.

Acceptance:

- current CTF contour does not change;
- no claim widening;
- first code slice is selected.

### P1 - Quadro Engine Safety Hardening

Type: code
Primary target: `crates/ton618-core/src/quadro.rs`

Goal:

- make the packed quad-state engine safe as a public API;
- make unsafe / SIMD boundaries explicit and honest.

Changes:

1. Move Quadro logic into a separate module:

   - `crates/ton618-core/src/quadro.rs`

2. Add public exports in `lib.rs`.

3. Split safe and unchecked API:

   - `try_set_by_mask(...)`
   - `set_by_mask_unchecked(...)`
   - `try_bulk_calc_delta(...)`
   - `bulk_calc_delta_unchecked(...)`

4. Remove release-mode safety holes:

   - slice lengths are checked outside `debug_assert`;
   - invalid state does not silently become `N`;
   - invalid mask does not pass through public safe API.

5. Feature cleanup:

   - bench requires `std`;
   - `simd` is optional;
   - `no_std` without `alloc` still compiles where intended.

6. x86 cfg cleanup:

   - x86_64 uses `core::arch::x86_64`;
   - x86 is either explicitly supported or removed from cfg.

Acceptance:

```text
cargo fmt --check
cargo clippy -p ton618-core --all-targets --all-features -- -D warnings
cargo test -p ton618-core --all-features
cargo check -p ton618-core --no-default-features
cargo check -p ton618-core --no-default-features --features alloc
```

Non-goals:

- no VM integration;
- no SemCode changes;
- no verifier changes;
- no CTF scope widening.

### P2 - Quadro Microbench Harness

Type: benchmark / measurement
Depends on: P1

Goal:

- measure real speed of packed quad-state primitives.

Add:

- `crates/ton618-core/benches/quadro_logic.rs`

Benchmark workloads:

| Benchmark | Description |
| --- | --- |
| `qreg_merge` | OR over packed quadits |
| `qreg_intersect` | AND over packed quadits |
| `qreg_inverse` | T/F plane swap |
| `qreg_masks_all` | extract N/F/T/S masks |
| `qreg_calc_delta` | scalar delta |
| `qbank_merge_inplace` | bank batch merge |
| `qbank_calc_delta_soa` | batch delta SoA |
| `vec_u8_baseline_delta` | simple baseline |
| `enum_baseline_delta` | optional enum baseline |

Metrics:

- ns/op
- regs/sec
- quadits/sec
- approximate bytes/sec

Acceptance:

```text
cargo bench -p ton618-core
cargo test -p ton618-core --all-features
```

Benchmark rule:

- do not claim performance without recorded benchmark output.

### P3 - Correctness Vectors

Type: tests
Depends on: P1

Goal:

- prove bit-for-bit correctness of quad algebra.

Add tests for:

- all 16 `merge` truth table combinations;
- all 16 `intersect` combinations;
- inverse truth table;
- mask extraction for N/F/T/S;
- mask alignment;
- delta transitions;
- random scalar vs batch equivalence;
- SIMD vs scalar equivalence when SIMD is enabled.

Acceptance:

```text
cargo test -p ton618-core --all-features
cargo test -p ton618-core --no-default-features --features alloc
```

## Next Step: P4 Shadow Adapter Design

P4 shadow equivalence is closed.
The next safe roadmap phase is local measurement and candidate selection, not acceleration.

Required constraints:

- default runtime behavior must remain unchanged;
- the adapter must be feature-gated;
- no VM authority change;
- no verifier admission change;
- no SemCode format change;
- no CTF contour widening;
- no PROMETHEUS capability or host-boundary change;
- no public performance or release claim.

P4 entry criteria:

- the v0 baseline closeout is merged;
- shadow targets are selected;
- the old-path vs Pulsar-path equivalence strategy is documented;
- the feature name is fixed, recommended: `pulsar-shadow`;
- test-only or shadow-only behavior is confirmed.

Allowed first shadow targets:

| Target | Reason |
| --- | --- |
| conflict mask calculation | direct mask projection with no behavior widening |
| known mask calculation | stable structural comparison target |
| state delta comparison | already modeled by scalar delta outputs |
| batch merge equivalence | pure OR-style structural check |
| batch intersect equivalence | pure AND-style structural check |

P4 mismatch diagnostics:

A shadow mismatch must not rely on a bare equality assertion as the only diagnostic.

When the baseline path and Pulsar path diverge, the shadow harness must first build a local diagnostic report and only then fail the test or fuzz case.

Required mismatch report fields:

- operation name;
- input case id or fuzz seed, when available;
- register index;
- first differing quadit index, when applicable;
- baseline raw register / mask / delta;
- Pulsar raw register / mask / delta;
- old and new state for delta comparisons;
- CPU feature path, such as scalar, SIMD, AVX2, NEON, or fallback;
- enabled Cargo features;
- compact human-readable summary.

This is local diagnostic evidence only.
It must not introduce telemetry, remote reporting, runtime behavior changes, VM authority changes, verifier changes, SemCode changes, CTF widening, or PROMETHEUS boundary changes.

P4 non-goals:

- runtime acceleration;
- SemCode vocabulary changes;
- verifier logic changes;
- VM dispatch changes;
- symbolic ownership;
- range ownership;
- iterator ownership;
- new public Semantic API claims;
- release readiness wording.

P5 remains blocked until P4 evidence exists:

- shadow equivalence tests must exist;
- old and Pulsar paths must match bit-for-bit;
- a benchmark advantage must be recorded;
- a promotion review must be performed.

### P4 - Shadow Adapter Design

Type: design / equivalence planning
Depends on: P1 + P3

Goal:

- define the shadow calculation path and its evidence gates without changing runtime behavior.

Shape:

```text
existing runtime path -> result A
Pulsar packed path   -> result B
assert_eq!(A, B)
```

First application areas:

- conflict masks;
- known masks;
- state delta;
- simple batch ownership masks.

Acceptance:

- adapter compiled behind a feature flag;
- default runtime behavior unchanged;
- tests compare old path and Pulsar path;
- no public claim widening.

Feature name example:

- `pulsar-shadow`

### P4-A - Shadow Adapter Evidence Contract

Type: docs / evidence contract
Depends on: P4

Goal:

- define how the baseline path and the Pulsar path are compared before any implementation slice expands beyond shadow-only evidence.

Evidence contract:

```text
baseline path A
Pulsar path B
A == B bit-for-bit for the selected target
if A != B -> build a local diagnostic report, then fail the test or fuzz case
```

Required mismatch evidence:

- operation name;
- input case id or fuzz seed, when available;
- register index;
- first differing quadit index, when applicable;
- baseline raw register / mask / delta;
- Pulsar raw register / mask / delta;
- old and new state for delta comparisons;
- CPU feature path, such as scalar, SIMD, AVX2, NEON, or fallback;
- enabled Cargo features;
- compact human-readable summary.

This is local diagnostic evidence only.
It must not introduce telemetry, remote reporting, runtime behavior changes, VM authority changes, verifier changes, SemCode changes, CTF widening, or PROMETHEUS boundary changes.

P4-A first shadow targets:

| Target | Reason |
| --- | --- |
| conflict mask calculation | direct mask projection with no behavior widening |
| known mask calculation | stable structural comparison target |
| state delta comparison | already modeled by scalar delta outputs |
| batch merge equivalence | pure OR-style structural check |
| batch intersect equivalence | pure AND-style structural check |

P4-A non-goals:

- runtime acceleration;
- SemCode vocabulary changes;
- verifier logic changes;
- VM dispatch changes;
- symbolic ownership;
- range ownership;
- iterator ownership;
- new public Semantic API claims;
- release readiness wording.

## P4 Shadow Equivalence Closeout

Status: closed.

P4 completed the shadow/equivalence layer required before any runtime acceleration candidate may be considered.

Completed P4 artifacts:

- evidence contract;
- shadow equivalence harness skeleton;
- mask equivalence coverage;
- state delta equivalence coverage;
- batch merge equivalence coverage;
- batch intersect equivalence coverage;
- deterministic seeded sweep coverage;
- local mismatch diagnostics.

Completed slices:

- `#1210` - Pulsar v0 closeout and P4 scope;
- `#1211` - quad equality / evidence algebra boundary;
- `#1212` - P4-A shadow adapter evidence contract;
- `#1213` - P4-B shadow equivalence harness skeleton;
- `#1214` - P4-C shadow equivalence target pack;
- `#1215` - P4-D deterministic shadow sweep coverage.

P4 evidence covers:

- packed quad state mask extraction;
- conflict mask equivalence;
- known mask equivalence;
- state delta equivalence;
- batch merge equivalence;
- batch intersect equivalence;
- deterministic sweep coverage over valid packed `u64` inputs;
- structured local mismatch diagnostics.

P4 increases confidence that Pulsar packed-state operations match scalar/reference baselines for the covered operations.

P4 closeout does not claim:

- runtime activation;
- `sm-vm` integration;
- verifier integration;
- SemCode format change;
- default execution behavior change;
- production acceleration;
- public performance claim;
- release readiness;
- P5 approval.

P4 complete does not mean P5 approved.

### P4 Evidence Repair

P4 shadow equivalence was later evidence-repaired by PR `#1237`.

The repair completed the remaining diagnostic and batch-path coverage gaps:

- `ShadowMismatchReport` records CPU feature path.
- `ShadowMismatchReport` records enabled Cargo features.
- alloc-gated seeded sweep exercises `QuadroBank::merge_inplace`.
- alloc-gated seeded sweep exercises `QuadroBank::intersect_inplace`.

This repair remains shadow/test-only.
It does not approve runtime activation, P5-A, P5-B, VM integration, SemCode changes, verifier changes, or public performance claims.

## P5 Promotion Gates

P5 may begin only after all of the following are true:

1. P4 closeout is merged.
2. A local `sm-vm` profiling / measurement harness exists.
3. The candidate hot path is measured, not guessed.
4. The selected candidate has a scalar authority path.
5. A feature-gated Pulsar candidate path is specified.
6. Runtime-level equivalence tests are planned.
7. Fallback behavior is documented.
8. No verifier, SemCode, CTF, or PROMETHEUS boundary is widened.
9. No public performance claim is made before benchmark evidence exists.
10. A promotion review explicitly approves the candidate.

P5 is blocked until a measured hot path exists.

P5-A remains blocked after `#1237`.

Reason:

- `#1237` repaired P4 shadow evidence quality.
- `#1237` did not provide new runtime hot-path measurements.
- `#1237` did not show a meaningful, batchable quad-state VM hot path.
- `#1237` did not approve runtime integration.

P5-A may reopen only from fresh measured runtime evidence.

The `15%` value used in P4-H is a local conservative review heuristic for that evidence report, not a canonical Pulsar promotion gate.

The canonical P5-A rule remains:

- select exactly one narrow candidate;
- base selection on measured VM/runtime evidence;
- require a meaningful hot path;
- require scalar authority path;
- require feature-gated Pulsar candidate path;
- require runtime-level equivalence test plan;
- require fallback documentation;
- require explicit promotion review.

### P5-A Candidate Selection Rule

P5-A must select exactly one narrow acceleration candidate based on measured VM/runtime evidence.

See also: [Pulsar P5-A Expected Candidate Path](pulsar_p5a_expected_candidate_path.md).

A candidate must not be selected because it is architecturally attractive or theoretically fast.
It must be selected because profiling shows it is a meaningful hot path.

Candidate selection must include:

- operation name;
- crate / module;
- current scalar path;
- expected Pulsar replacement path;
- reason it is hot;
- measurement evidence;
- fallback path;
- feature gate;
- equivalence test plan.

Potential P5 candidates may include, but are not limited to:

- repeated quad logical operations;
- quad mask extraction in runtime-heavy scenarios;
- batch-like quad state transitions;
- state delta calculation if measured hot;
- merge / intersect patterns if measured hot.

These are candidates only.
None are approved until measured.

P5 must not start with:

- replacing `sm-vm` execution wholesale;
- making Pulsar the VM authority;
- changing SemCode vocabulary;
- changing verifier admission;
- changing public Semantic behavior;
- changing default runtime behavior without a feature gate;
- adding SIMD before scalar runtime equivalence exists;
- claiming production performance;
- widening CTF or PROMETHEUS boundaries.

The next technical step after this closeout is:

P4-F / P5-pre - local `sm-vm` opcode profiling harness.

Before P5-A candidate selection, the project needs a deterministic local VM profiling harness that can collect opcode execution counts and scenario-level measurement data without introducing production telemetry or runtime behavior changes.

This should be local profiling, not production telemetry.

### P5 - Runtime Acceleration Candidate

P5 remains a blocked future implementation phase.
It may only begin after the gates above are satisfied and a measured hot path selects exactly one narrow candidate.

### P6 - Promotion Review

Type: trust review
Depends on: P5

Goal:

- decide whether Pulsar can be treated as part of a frozen internal implementation contour without widening public trust claims.

Promotion criteria:

- correctness vectors green;
- shadow-mode equivalence green;
- benchmark advantage recorded;
- unsafe boundaries reviewed;
- no new authority ownership introduced;
- public docs do not claim release readiness.

If promoted:

- Pulsar becomes an internal acceleration backend for selected operations.

If not promoted:

- Pulsar remains an experimental substrate.

## Safety Rules

### Public Safe API

Public safe API must not rely only on `debug_assert`.

Bad:

```rust
pub fn set_by_mask(mask, state) { debug_assert!(valid); ... }
```

Good:

```rust
pub fn try_set_by_mask(mask, state) -> Result<Self, Error>
```

Hot path:

```rust
unsafe fn set_by_mask_unchecked(...)
```

### Unsafe Boundary

All unsafe SIMD functions must be internal and called only after:

- length checks;
- alignment decision;
- architecture feature check;
- fallback path exists.

### Alignment

Default SIMD path should remain unaligned-safe.

Aligned path may be added only as:

- `aligned_unchecked`

and only called when alignment is proven.

### Feature Policy

- `std` = runtime detection / benchmarking / timing
- `alloc` = `Vec` / `String` backed structures
- `simd` = explicit acceleration backend
- `bench` = requires `std`

## Benchmark Policy

Do not accept benchmark results without:

- CPU model;
- target triple;
- Rust version;
- build flags;
- features;
- debug / release mode;
- iteration count;
- input size;
- baseline comparison.

Recommended command:

```text
cargo bench -p ton618-core --features "std simd bench"
```

Optional native CPU run:

```powershell
$env:RUSTFLAGS="-C target-cpu=native"
cargo bench -p ton618-core --features "std simd bench"
```

## Integration Policy with CTF

Current Core Trust Freeze remains active only for the conservative verified core contour.

Pulsar work must not widen:

- SemCode format authority;
- verifier admission;
- VM execution authority;
- runtime ownership semantics;
- public release claims.

Pulsar integration must go through:

```text
standalone module
-> correctness tests
-> benchmarks
-> shadow adapter
-> promotion review
-> controlled acceleration
```

## First Three Implementation Slices

### Slice 1 - PULSAR-Q1 Safety Seed

Likely touched files:

- `crates/ton618-core/src/lib.rs`
- `crates/ton618-core/src/quadro.rs`
- `crates/ton618-core/Cargo.toml`

Goal:

- introduce Quadro engine safely, with checked API and tests.

Do not touch:

- `sm-vm`
- `sm-verify`
- `sm-format`
- `sm-runtime-core`
- SemCode
- CI workflows
- `README.md`

### Slice 2 - PULSAR-Q2 Correctness Matrix

Likely touched files:

- `crates/ton618-core/src/quadro.rs`
- `crates/ton618-core/tests/quadro_logic.rs`

Goal:

- truth tables, mask tests, delta tests, scalar / batch equivalence.

### Slice 3 - PULSAR-Q3 Bench Harness

Likely touched files:

- `crates/ton618-core/benches/quadro_logic.rs`
- `crates/ton618-core/Cargo.toml`

Goal:

- measure packed engine against scalar baseline workloads.

## Definition of Done for Pulsar v0

Pulsar v0 is complete when:

- Quadro engine exists as an isolated module;
- safe API is checked;
- unsafe SIMD boundary is narrow;
- `no_std` / `alloc` posture is preserved;
- correctness matrix passes;
- scalar and batch paths match;
- benchmark harness exists;
- benchmark output reports quadits/sec;
- no current CTF authority boundary is widened;
- no public release / no_std / symbolic precision claim is added.

## Related Docs

- CTF overview: [docs/roadmap/language_maturity/core_trust_freeze/index.md](language_maturity/core_trust_freeze/index.md)
- CTF readiness map: [docs/roadmap/pcc/core_trust_freeze_final_readiness_map.md](pcc/core_trust_freeze_final_readiness_map.md)
- CTF active declaration: [docs/roadmap/pcc/core_trust_freeze_active_declaration.md](pcc/core_trust_freeze_active_declaration.md)
- CTF final review: [docs/roadmap/pcc/core_trust_freeze_declaration_final_review.md](pcc/core_trust_freeze_declaration_final_review.md)
- CTF draft declaration: [docs/roadmap/pcc/core_trust_freeze_declaration_draft.md](pcc/core_trust_freeze_declaration_draft.md)

## Final Principle

Pulsar should be treated as:

- a fast internal semantic-state engine

not as:

- a new public Semantic authority.

The goal is not to replace the frozen core.

The goal is to make the frozen core faster from underneath.
