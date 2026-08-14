# SSF-07 Pattern/Match Qualification Matrix

Status: SSF-07 supplementary evidence detail for `match`/pattern forms
Normative contract: `docs/spec/foundation_source_profile_v1.md`
Contract identifier: `semantic.foundation.source/1.2`

This is a granular, per-form supplement to the summary "`match`, guards,
patterns" row in `stable_public_language_contract.md`'s executable evidence
map. It exists because that row's normal granularity (one contract family,
one evidence-file pointer) cannot show which individual scrutinee/pattern
*forms* within `match` are qualified through every phase versus rejected at
a specific phase — exactly the distinction whose absence let three real
phase-consistency defects (`u32` match, the `5..5` exclusive-range
miscompilation, and or-pattern lowering failure) go undetected through
several rounds of documentation-only review before SSF-07's corrective pass
found and fixed them.

A row is marked **Included** only when every phase through `Run` is PASS
and pinned by an executable fixture in `tests/match_surface_qualification.rs`
(not merely a frontend/typecheck unit test — a frontend-only PASS is not
executable qualification, per the normative profile's Qualification rule).
A row is marked **Unsupported** only when it fails deterministically at a
named phase, before a runnable artifact with incorrect or unsupported
semantics could be produced, and that rejection is pinned by a fixture.

Columns: **Parse** / **Typecheck** / **Exhaustiveness** (sum-family
coverage reasoning specifically, not merely "did typecheck pass") /
**Lower** / **Verify** / **Run** / **Classification**.

## `i32` scrutinee forms

| Form | Parse | Typecheck | Exhaustiveness | Lower | Verify | Run | Classification | Fixture |
|---|---|---|---|---|---|---|---|---|
| Plain literal (`5 => {}`) | PASS | PASS | n/a (needs `_`) | PASS | PASS | PASS | **Included** | `positive_i32_plain_literal_match` |
| Inclusive singleton (`5..=5`) | PASS | PASS | n/a | PASS | PASS | PASS | **Included** | `positive_i32_singleton_range_match` |
| Zero boundary (`0..=0` / `0 =>`) | PASS | PASS | n/a | PASS | PASS | PASS | **Included** | `positive_i32_singleton_range_match` |
| `i32::MAX` boundary (`2147483647..=2147483647`) | PASS | PASS | n/a | PASS | PASS | PASS | **Included** | `positive_i32_singleton_range_match` |
| Exclusive singleton (`5..5`) | PASS | PASS | n/a | **FAIL** (`integer range match pattern lowering is not yet implemented in the IR backend`) | — | — | **Unsupported** (lowering, deterministic) | `negative_i32_exclusive_singleton_range_lowering_rejected` |
| Inclusive multi-value (`1..=5`) | PASS | PASS | n/a | **FAIL** (same message as above) | — | — | **Unsupported** (lowering, deterministic) | `negative_i32_multivalue_range_lowering_rejected` |
| Exclusive multi-value (`1..5`) | PASS | PASS | n/a | **FAIL** (same message as above) | — | — | **Unsupported** (lowering, deterministic) | `negative_i32_exclusive_multivalue_range_lowering_rejected` |
| Oversized bound (`2147483648..=2147483648`) | PASS | PASS | n/a | **FAIL** (`integer match pattern literal is outside i32 range`) | — | — | **Unsupported** (lowering, deterministic) | `negative_i32_oversized_singleton_range_lowering_rejected` |
| Negative bound (`-5..=-5`) | **FAIL** (`E0000: expected match pattern`) | — | — | — | — | — | **Unsupported** (parser, deterministic) | `negative_i32_negative_bound_range_pattern_rejected` |
| Suffixed range bound (`5i32..=5i32`) | **FAIL** (`range pattern bound does not accept a type suffix; use a plain integer`) | — | — | — | — | — | **Unsupported** (parser, deterministic) | `negative_i32_range_pattern_suffixed_bound_rejected` |
| Or-pattern (`1 \| 2`) | PASS | **FAIL** (`or-pattern match arms ('A \| B') are not supported; ...`) | — | — | — | — | **Unsupported** (typecheck, deterministic) | `negative_i32_or_pattern_lowering_rejected` |

## `u32` scrutinee forms

| Form | Parse | Typecheck | Exhaustiveness | Lower | Verify | Run | Classification | Fixture |
|---|---|---|---|---|---|---|---|---|
| Suffixed plain literal (`0u32 =>`, `5u32 =>`) | PASS | PASS | n/a | PASS | PASS | PASS | **Included** | `positive_u32_match_full_domain` (`classify_literal`) |
| Unsuffixed plain literal (`0 =>`, `5 =>`) — distinct parser branch from the suffixed and range forms | PASS | PASS | n/a | PASS | PASS | PASS | **Included** | `positive_u32_match_full_domain` (`classify_literal_unsuffixed`) |
| `2147483647` (`i32::MAX`) | PASS | PASS | n/a | PASS | PASS | PASS | **Included** | `positive_u32_match_full_domain` |
| `2147483648` (`i32::MAX + 1`) | PASS | PASS | n/a | PASS | PASS | PASS | **Included** | `positive_u32_match_full_domain` |
| `4294967295` (`u32::MAX`) | PASS | PASS | n/a | PASS | PASS | PASS | **Included** | `positive_u32_match_full_domain` |
| Inclusive singleton range (`5..=5`) | PASS | PASS | n/a | PASS | PASS | PASS | **Included** | `positive_u32_match_full_domain` |
| Range at `u32::MAX` (`4294967295..=4294967295`) | PASS | PASS | n/a | PASS | PASS | PASS | **Included** | `positive_u32_match_full_domain` |
| Exclusive singleton (`5..5`) | PASS | PASS | n/a | **FAIL** (same "not yet implemented" message as `i32`) | — | — | **Unsupported** (lowering, deterministic) | `negative_u32_exclusive_singleton_range_lowering_rejected` |
| Multi-value (`1..=5`) | PASS | PASS | n/a | **FAIL** (same "not yet implemented" message) | — | — | **Unsupported** (lowering, deterministic) | `negative_u32_multivalue_range_lowering_rejected` |
| Oversized bound (`4294967296..=4294967296`, `u32::MAX + 1`) | PASS | PASS | n/a | **FAIL** (`integer match pattern literal is outside u32 range`) | — | — | **Unsupported** (lowering, deterministic) | `negative_u32_oversized_singleton_range_lowering_rejected` |
| Or-pattern (`1u32 \| 2u32`) | PASS | **FAIL** (`or-pattern match arms ('A \| B') are not supported; ...`) | — | — | — | — | **Unsupported** (typecheck, deterministic) | `negative_u32_or_pattern_lowering_rejected` |

Negative-bound and type-suffixed-range-bound restrictions apply identically
to `u32` as to `i32` (both are unconditional parser-level checks that do not
inspect scrutinee type); see the `i32` table above for their evidence.

## `quad` scrutinee or-patterns

| Form | Parse | Typecheck | Exhaustiveness | Lower | Verify | Run | Classification | Fixture |
|---|---|---|---|---|---|---|---|---|
| Or-pattern (`N \| F`) | PASS | **FAIL** (`or-pattern match arms ('A \| B') are not supported; ...`) | — | — | — | — | **Unsupported** (typecheck, deterministic) | `negative_quad_or_pattern_lowering_rejected` |

Every other `quad` match form (literal arms, guards, exhaustiveness) is
Included and was already qualified before SSF-07; see the summary evidence
row for its fixtures.

## Sum-family (`enum`/`Option(T)`/`Result(T, E)`) or-patterns

| Family | Wildcard present? | Parse | Typecheck | Lower | Classification | Fixture |
|---|---|---|---|---|---|---|
| enum | no | PASS | **FAIL** | — | **Unsupported** (typecheck, deterministic) | `negative_enum_or_pattern_no_wildcard_lowering_rejected` |
| enum | yes | PASS | **FAIL** | — | **Unsupported** (typecheck, deterministic) | `negative_enum_or_pattern_with_wildcard_lowering_rejected` |
| `Option(T)` | no | PASS | **FAIL** | — | **Unsupported** (typecheck, deterministic) | `negative_option_or_pattern_no_wildcard_lowering_rejected` |
| `Option(T)` | yes | PASS | **FAIL** | — | **Unsupported** (typecheck, deterministic) | `negative_option_or_pattern_with_wildcard_lowering_rejected` |
| `Result(T, E)` | no | PASS | **FAIL** | — | **Unsupported** (typecheck, deterministic) | `negative_result_or_pattern_no_wildcard_lowering_rejected` |
| `Result(T, E)` | yes | PASS | **FAIL** | — | **Unsupported** (typecheck, deterministic) | `negative_result_or_pattern_with_wildcard_lowering_rejected` |

All six rows above fail with the identical diagnostic — "or-pattern match
arms ('A | B') are not supported; split into separate arms with identical
bodies instead" — from `build_and_apply_match_plan` in
`crates/sm-front/src/typecheck.rs`. Before SSF-07's fix this rejection
happened at the lowering phase instead, with a diagnostic that varied by
family and by whether a wildcard arm was present (see the profile's
version-`1.2` note for the full before/after account). `if let` uses a
separate entry point (`build_match_pattern_plan` called directly, not
through `build_and_apply_match_plan`) and is unaffected by this rejection.

## Validation contour

This matrix's rows are exercised end to end by:

- `tests/match_surface_qualification.rs` — `match_surface_positive_fixtures_run_end_to_end`
  (Included rows: full `check` → `compile` → `verify` → `run-smc` pipeline),
  `match_surface_negative_fixtures_reject_deterministically` (typecheck-phase
  Unsupported rows), and
  `match_surface_range_lowering_rejection_fixtures_reject_at_compile_phase`
  (lowering-phase Unsupported rows).
- `crates/sm-front/src/typecheck.rs`'s unit tests for the or-pattern
  rejection's typecheck-time behavior specifically (e.g.
  `or_pattern_match_arm_rejects_even_when_it_would_be_exhaustive`).
- `crates/sm-ir/src/legacy_lowering.rs`'s existing lowering unit-test suite,
  run unchanged to confirm no regression outside the pattern-lowering paths
  this phase touched.

Skipped checks do not count as pass, per this repository's standing
qualification discipline.
