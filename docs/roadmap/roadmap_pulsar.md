# Pulsar Roadmap

Status: v0 substrate baseline closed
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

P4 is the next safe roadmap phase.
It is design and equivalence planning first, not acceleration.

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

### P5 - Runtime Acceleration Candidate

Type: controlled implementation
Depends on: P2 + P4

Goal:

- use Pulsar as an internal acceleration backend only for proven identical operations.

P5 remains blocked until P4 evidence exists.

Allowed first candidates:

| Operation | Why |
| --- | --- |
| conflict scan | direct `mask_s` |
| known mask scan | direct `mask_non_null` |
| state delta | already modeled |
| batch merge | pure OR |
| batch intersect | pure AND |

Not allowed yet:

- symbolic ownership;
- dynamic index precision;
- range ownership;
- iterator ownership;
- new SemCode vocabulary;
- verifier admission changes.

Acceptance:

- old tests still pass;
- new accelerated path has scalar fallback;
- benchmark shows improvement;
- behavior equivalence is proven by tests;
- feature-gated rollout.

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
