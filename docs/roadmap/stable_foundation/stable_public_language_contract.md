# Stable Public Language Contract — SSF-01

Status: SSF-01 evidence and closure record
Normative contract: `docs/spec/foundation_source_profile_v1.md`
Contract identifier: `semantic.foundation.source/1.1`
Base commit: `4de0b6eb1cd5d8e5dc37989e9b9b95a5a8e07e57`

The contract identifier above tracks the current version of the normative
profile document; it is not frozen to the base commit, which records only
when the underlying SSF-01 evidence was gathered. SSF-07 bumped the profile
to `1.1` (a backward-compatible clarification: no grammar, semantic, or
rejection-behavior change from `1.0`). That clarification did add net-new
executable evidence in the same PR, so this is not a documentation-only
revision: the "`match`, guards, patterns" row below gained a
singleton-range fixture in its primary evidence file
(`tests/match_surface_qualification.rs`) pinning both interval boundaries
(`0` and `i32::MAX`) end to end, backing the newly precise range and
or-pattern claims the profile now documents. The base commit above remains
SSF-01's original gathering anchor; the evidence that actually governs this
contract's standing is whatever the mapped test/fixture files in the table
below currently contain, per the normative profile's Qualification rule,
not a frozen historical snapshot.

## Decision

SSF-01 selects a bounded Rust-like executable contract from `SSF-TARGET-0`.
The selected surface is the smallest current contour with existing end-to-end
evidence sufficient for ordinary deterministic programs. Current-main
extensions outside it remain experimental or deferred; no new language feature
is implemented by this phase.

## Executable evidence map

| Contract family | Outcome | Primary executable evidence | Boundary evidence |
|---|---|---|---|
| Functions, entrypoint, calls, `let`, `const`, return | Included stable candidate | `tests/practical_surface_execution_qualification.rs`, `tests/call_shape_surface_qualification.rs`, `tests/return_assert_surface_qualification.rs` | arity/type/unknown-call and return diagnostics in the same suites |
| Mutable bindings and assignment | Included stable candidate | `tests/mutable_binding_qualification.rs`, application-completeness benchmark | const/type/unknown-target negatives |
| `if`/`else`, blocks, value expressions | Included stable candidate | `tests/pcc1_control_flow_gate.rs`, `tests/pcc1_control_flow_lowering_stability.rs`, practical execution suite | branch/result mismatch diagnostics |
| `match`, guards, patterns | Included stable candidate | `tests/match_surface_qualification.rs`, PCC5/PCC6 acceptance | exhaustiveness, guard type, pattern-family and arm-type negatives, plus compile-phase lowering-rejection fixtures for scalar (`quad`/`i32`/`u32`) and wildcard-present/absent sum-family or-patterns, genuine multi-value and oversized-singleton `i32` range arms, a check-phase suffixed-integer-literal range-bound rejection fixture, and runtime-defect fixtures pinning the `u32` literal/range match trap and the exclusive `5..5` singleton-range miscompilation |
| `while`, `loop`, range/sequence `for`, exits | Included stable candidate | PCC1 suites, PCC7 sequence acceptance, application benchmark | outside-loop and unsupported nested-form negatives |
| Records | Included stable candidate | `tests/pcc4_records_acceptance.rs`, record copy-with qualification, Gate 1 programs | PCC4 and copy-with diagnostics |
| Tuples | Included stable candidate | IR lowering, tuple ownership golden, runtime ownership E2E | overlap/move/path negatives |
| Enums/ADTs | Included stable candidate | `tests/pcc5_adt_acceptance.rs` and match acceptance | PCC5 diagnostics and exhaustiveness negatives |
| `Option` / `Result` | Included stable candidate | `tests/pcc6_option_acceptance.rs`, `tests/pcc6_result_acceptance.rs`, ownership golden | PCC6 type/context/exhaustiveness diagnostics |
| `Sequence` | Included stable candidate | `tests/pcc7_sequence_acceptance.rs` and application benchmark | PCC7 collection diagnostics |
| `Map` | Included stable candidate | `tests/pcc7_map_acceptance.rs` and snake-learning benchmark | key/value/context diagnostics |
| Captureless single-argument short lambdas | Included stable candidate | `tests/short_lambda_surface_qualification.rs`, SemCode/bytecode compatibility | captureful, missing-context, and multi-arg boundary fixtures |
| Direct-record `Iterable` | Included stable candidate | Gate 1 frontend/execution evidence and canonical data-audit example | ADT/schema/generalized dispatch exclusions |
| Bare/selected local helper imports | Included stable candidate | executable-module entry, `tests/import_surface_qualification.rs`, canonical examples | alias/wildcard/re-export/cycle/collision negatives |
| `quad`, `bool`, `unit` | Included stable candidate | Gate 1 programs, quad lowering profile, canonical match example | explicit branch/type diagnostics |
| `i32` | Included stable candidate | `tests/pcc2_numeric_core_gate.rs` and application benchmark | mixed-family/type/division failure evidence |
| narrow `u32` | Included stable candidate | numeric surface qualification | arithmetic/conversion remain explicitly deferred |
| bounded `f64` and `fx` values/arithmetic | Included stable candidate | numeric surface and PCC2 qualification, SemCode/VM evidence | cross-family, measured-operation, and transcendental-math policy exclusions |
| `text` | Included stable candidate | `tests/pcc3_text_core_gate.rs`, canonical text example, application benchmark | indexing/formatting/cross-family negatives |
| Rust-like `when` | Experimental | frontend unit/lowering evidence only | no full public qualification contour |
| Schemas | Experimental | landed schema metadata/tooling evidence | no selected executable Foundation contract |
| Broad generics/traits | Experimental | frontend conformance/unit evidence | only direct-record `Iterable` is included |
| `requires`/`ensures`/`invariant` | Experimental | bounded parser/typecheck/lowering evidence | no complete public full-path qualification contour |
| Measured numeric forms | Experimental | narrow frontend/carrier evidence | measured `fx` arithmetic remains an explicit gap |
| Broader module/package imports | Deferred | landed current-main package/module evidence | owned by SSF-05/SSF-06 |
| Logos | Experimental declarative profile | parse, semantic analysis, and non-executable Logos-IR canonical example | Model B selected by SSF-02; outside this executable contract |

## Contract audit result

The included contour has existing positive, negative, lowering, verified
execution, canonical-example, and CI/7hell evidence. No parser, typechecker,
lowering, verifier, or VM implementation gap was demonstrated for that bounded
selection. Consequently SSF-01 makes no compiler/runtime implementation change.

The gaps found by the audit are contract gaps, not defects to fill here:

- the permissive parser profile accepts more than the stable-candidate promise;
- current specs are mostly draft-v0 component references rather than one public
  versioned source contract;
- source contract version and SemCode capability-derived header selection were
  not stated together;
- accepted experimental forms and truly unsupported syntax were easy to blur.

The normative profile document resolves those documentation gaps without
pretending experimental behavior is stable.

## Unsupported and excluded behavior

Three outcomes are intentionally distinct:

1. **Included** — documented stable candidate with mapped end-to-end evidence.
2. **Experimental** — may be accepted on current `main`, but is outside the
   compatibility promise and must be labeled accordingly.
3. **Deferred/unsupported** — either owned by a later phase or deterministically
   rejected by existing canonical diagnostics.

This distinction avoids the dishonest alternative of making the parser reject
working experimental research merely to keep the stable contour small.

## Version and compatibility relationship

- source contract: `semantic.foundation.source/1.1`;
- parser acceptance envelope: `semantic.foundation` / `1.0`;
- SemCode: capability-derived supported header, currently `SEMCODE0` through
  `SEMCOD14`, never chosen solely from profile permission;
- verifier: mandatory admission before standard execution;
- compatibility retention window: deferred to SSF-10;
- promotion: deferred to SSF-12 plus explicit human decision.

## Validation contour

The SSF-01 PR must run:

- the new contract drift guard;
- PCC1 through PCC7 acceptance/diagnostic suites relevant to included rows;
- practical surface, numeric, match, mutable binding, lambda, import, ownership,
  `tests/canonical_examples.rs`, SemCode, verifier, and VM focused suites;
- repository PR-ready, boundary, public API, release-bundle, 7hell, all-target,
  and no-std CI gates.

Skipped checks cannot count as pass.

## SSF-02 entry conditions

SSF-02 may start only after:

1. `semantic.foundation.source/1.1` and this evidence map are reviewed and
   merged;
2. all included rows remain green through exact-head CI;
3. experimental/deferred rows remain explicitly unpromoted in current-facing
   docs and examples;
4. issue #1572 records the exact merge commit and closes;
5. a separate governance update activates only #1573.

SSF-02 selected the declarative Model B relationship without widening the
Rust-like contract selected here. The separate Logos contract is
`semantic.logos.declarative/0.1`.
