# sm-vm VM-M6 Equivalent Source Shape Scalar Movement Comparison

## Status

VM-M6 compares existing source shapes to refine scalar movement diagnosis.

This document does not approve VM optimization, lowering changes, fixture changes, or runtime changes.

## Context

- VM-M3 showed `scalar_movement` dominates.
- VM-M4 showed `LoadVar` / `StoreVar` dominate every fixture.
- VM-M5 mapped likely source/lowering shapes.
- VM-M6 compares existing source shapes and identifies missing controlled pairs.

## Method

- Re-ran the existing profiling workloads.
- Re-inspected source fixtures.
- Compared existing fixtures where possible.
- Classified comparisons by strength:
  - exact equivalent pair
  - near-equivalent comparison
  - scale comparison
  - non-equivalent but informative contrast
  - missing controlled pair
- Did not add fixtures or modify code.

## Commands

- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --nocapture`
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored --nocapture`
- `sed -n '1,240p' crates/sm-vm/tests/fixtures/profiling/quad_logic_storm.sm`
- `sed -n '1,240p' crates/sm-vm/tests/fixtures/profiling/quad_match_dispatch.sm`
- `sed -n '1,240p' crates/sm-vm/tests/fixtures/profiling/fact_merge_kernel.sm`
- `sed -n '1,260p' crates/sm-vm/tests/fixtures/profiling/fact_intersect_kernel.sm`
- `sed -n '1,300p' crates/sm-vm/tests/fixtures/profiling/delta_like_kernel.sm`
- `sed -n '1,360p' crates/sm-vm/tests/fixtures/profiling/andromeda_fact_wave_64.sm`
- `sed -n '1,420p' crates/sm-vm/tests/fixtures/profiling/andromeda_fact_wave_256.sm`
- `git diff --check`
- `cargo fmt --check`

`cargo fmt --check` failed due to pre-existing unrelated formatting drift in `crates/prom-ui-backend-native/src/lib.rs`. This audit did not modify Rust source.

## Existing Comparison Matrix

| Comparison | Type | Fixtures | Load+Store % A | Load+Store % B | Observation | Confidence |
|---|---|---|---:|---:|---|---|
| Andromeda wave scale comparison | scale comparison | `andromeda_fact_wave_64.sm` vs `andromeda_fact_wave_256.sm` | 41.40% | 41.47% | The ratio is effectively stable across scale, which strongly suggests structural source/lowering pressure rather than a tiny-fixture artifact. | Strong |
| Helper-heavy mixed wave vs quad logic storm | non-equivalent but informative contrast | `andromeda_fact_wave_64.sm` vs `quad_logic_storm.sm` | 41.40% | 42.10% | Both stay near 40% despite different source shapes; the similarity suggests temporaries/accumulators dominate more than specific quad operation choice. | Medium |
| Match dispatch vs explicit branch kernels | non-equivalent but informative contrast | `quad_match_dispatch.sm` vs `fact_merge_kernel.sm` / `fact_intersect_kernel.sm` | 39.49% | 29.38% / 23.36% | Match-heavy dispatch sits above the smaller explicit-branch kernels, but fixture size and semantics differ, so this is only a directional signal. | Medium |
| Transition classification vs simpler branch classification | near-equivalent classification contrast | `delta_like_kernel.sm` vs `fact_merge_kernel.sm` / `fact_intersect_kernel.sm` | 34.65% | 29.38% / 23.36% | The old/new transition workload is higher than the simpler branch kernels, supporting transition-local reload as a plausible contributor. | Medium |
| Temporary-heavy source shape | hypothesis from source shape | `quad_logic_storm.sm`, `andromeda_fact_wave_64.sm`, `andromeda_fact_wave_256.sm` | 42.10% / 41.40% / 41.47% | N/A | The current corpus has many named intermediates, but no controlled temporary-light pair. The evidence supports staging pressure only as a hypothesis. | Low |
| Helper-boundary source shape | non-equivalent but informative contrast | `quad_logic_storm.sm`, `quad_match_dispatch.sm`, `delta_like_kernel.sm`, `andromeda_fact_wave_64.sm`, `andromeda_fact_wave_256.sm` | 42.10% / 39.49% / 34.65% / 41.40% / 41.47% | N/A | Helper-heavy workloads still show high scalar movement, but there is no helper-free equivalent pair in the current corpus. | Low |

## Source-Shape Comparison Notes

### Scale comparison: Andromeda 64 vs 256

- This is the strongest existing comparison because the source shape is intentionally similar and only the scale changes.
- `Load+Store` remains stable at `41.40%` vs `41.47%`.
- The stability argues for structural source/lowering pressure.
- The stability also argues against the 256 workload being a tiny-fixture artifact.

### Helper-heavy mixed wave vs quad logic storm

- Both workloads are looped, helper-shaped, and full of intermediate quad values.
- The `Load+Store` ratio stays near 40% in both.
- This suggests temporaries and accumulators matter more than the specific quad operation mix.
- The workloads are not semantically equivalent, so this is only an informative contrast.

### Match dispatch vs if/branch kernels

- `quad_match_dispatch.sm` is the strongest match-heavy workload in the corpus.
- It sits above the explicit branch kernels, but the kernels are smaller and semantically different.
- The gap is still useful because it keeps match discriminant reloads on the suspect list.

### Transition classification vs simpler branch classification

- `delta_like_kernel.sm` is the clearest old/new transition-classification workload.
- It is higher than the simpler branch kernels, which is consistent with reloads of both old and new values.
- This supports transition-local reload as a stronger candidate than generic branching alone.

### Temporary-heavy source shape

- The corpus contains many intermediate `let` bindings and named values.
- Those intermediates are frequently reused later in the same block or loop body.
- There is no controlled temporary-light equivalent pair in the current corpus.
- This remains a hypothesis until a targeted comparison exists.

### Helper-boundary source shape

- Several workloads use helper functions to generate states or dispatch behavior.
- The helper-heavy workloads still show high scalar movement.
- There is no helper-free semantic equivalent pair in the current corpus.
- This remains a hypothesis until a controlled helper-free variant exists.

### Missing exact equivalents

- helper-heavy vs helper-light same semantics
- temporary-heavy vs temporary-light same semantics
- match vs if-chain same semantics
- looped accumulator vs unrolled same semantics
- branch-local counters vs return-value classification same semantics

These are the kinds of controlled pairs that would make the next audit materially stronger.

## Source-Shape Suspect Ranking

| Suspect | Evidence | Existing controlled pair? | Confidence | Next evidence needed |
|---|---|---:|---|---|
| Loop scale stability | The Andromeda 64 vs 256 comparison preserves the same movement ratio at a different scale. | Yes, scale comparison | Strong | A true controlled source pair with the same semantics but less helper/counter noise. |
| Accumulator read-modify-write | Every workload maintains counters or running totals that are repeatedly read and written. | No | Strong | A pair that isolates accumulator updates from other sources of movement. |
| Old/new transition local reload | The transition classification workload is higher than the simpler branch kernels. | No | Medium | A controlled transition vs simpler-classification pair. |
| Branch discriminant reload | Match-heavy and branch-heavy workloads suggest the same value is reloaded for repeated tests. | No | Medium | Equivalent match-vs-if variants with the same semantic goal. |
| Helper argument/result staging | Helper-shaped workloads remain high even when calls themselves are not dominant. | No | Medium | A helper-free equivalent or a source-shape variant with the same semantics. |
| Temporary expression staging | The corpus has many named intermediates, but no temporary-light equivalent pair. | No | Low | A source variant that removes or reduces intermediates without changing semantics. |

## Missing Controlled Fixtures

| Missing pair | Purpose | Why needed | Candidate future slice |
|---|---|---|---|
| Helper-heavy vs helper-light same semantics | Isolate helper boundary movement | The current corpus suggests helper staging may matter, but the effect is not isolated | `VM-M7: test(sm-vm): add scalar movement source-shape comparison fixtures` |
| Temporary-heavy vs temporary-light same semantics | Isolate expression staging | The current corpus has intermediates, but no controlled light variant | `VM-M7: test(sm-vm): add scalar movement source-shape comparison fixtures` |
| Match vs if-chain same semantics | Isolate discriminant reload | The current match-heavy and if-heavy examples are not semantically equivalent | `VM-M7: test(sm-vm): add scalar movement source-shape comparison fixtures` |
| Looped accumulator vs unrolled same semantics | Isolate loop-carried local traffic | The scale comparison is strong, but it does not isolate the loop effect by itself | `VM-M7: test(sm-vm): add scalar movement source-shape comparison fixtures` |
| Branch-local counters vs return-value classification same semantics | Isolate branch result staging | The current branch kernels are informative but not controlled equivalents | `VM-M7: test(sm-vm): add scalar movement source-shape comparison fixtures` |

## Interpretation

- The scale comparison strongly suggests structural scalar movement pressure.
- The existing corpus does not contain a controlled helper-free equivalent.
- The existing corpus does not contain a controlled temporary-light equivalent.
- The existing corpus does not contain a controlled match-vs-if semantic equivalent.
- Transition classification looks like a stronger candidate than generic branching alone.
- Helper and temporary pressure remain hypotheses until controlled comparisons exist.
- No optimization is approved.

## What This Audit Suggests

- Structural scalar movement is real and stable.
- Loop scale and accumulator churn are the strongest evidence-backed suspects.
- Transition classification and match dispatch remain worthwhile follow-up suspects.
- Helper and temporary staging are plausible but still need controlled pairs.
- The next useful step is to define a micro-fixture matrix before any code change.

## What This Audit Does Not Prove

- No code change is safe yet.
- No speedup is guaranteed.
- No SemCode, lowering, or VM change is approved.
- No exact optimization target is proven.

## Recommended Next Slice

VM-M7: docs(sm-vm): specify scalar movement micro-fixture matrix

This is the cleanest next step because the current corpus still has several missing controlled pairs. If the matrix is accepted, the follow-up implementation slice can add only the narrow fixtures needed to separate helper, temporary, match/if, loop, and accumulator effects.

## Non-claims

This document does not claim:

- VM performance improved;
- runtime behavior changed;
- verifier behavior changed;
- SemCode format changed;
- parser/typechecker/lowering changed;
- fixtures changed;
- Pulsar P5-A is reopened;
- scalar optimization is approved;
- any code change is safe without follow-up proof.

## Validation

Commands run:

- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --nocapture`
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored --nocapture`
- `sed -n '1,240p' crates/sm-vm/tests/fixtures/profiling/quad_logic_storm.sm`
- `sed -n '1,240p' crates/sm-vm/tests/fixtures/profiling/quad_match_dispatch.sm`
- `sed -n '1,240p' crates/sm-vm/tests/fixtures/profiling/fact_merge_kernel.sm`
- `sed -n '1,260p' crates/sm-vm/tests/fixtures/profiling/fact_intersect_kernel.sm`
- `sed -n '1,300p' crates/sm-vm/tests/fixtures/profiling/delta_like_kernel.sm`
- `sed -n '1,360p' crates/sm-vm/tests/fixtures/profiling/andromeda_fact_wave_64.sm`
- `sed -n '1,420p' crates/sm-vm/tests/fixtures/profiling/andromeda_fact_wave_256.sm`
- `git diff --check`
- `cargo fmt --check`

Results:

- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --nocapture` passed.
- `cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored --nocapture` passed.
- `git diff --check` passed, with only LF/CRLF warnings on the new roadmap docs.
- `cargo fmt --check` failed due to pre-existing unrelated formatting drift in `crates/prom-ui-backend-native/src/lib.rs`.
