# 7hell PCC-4..PCC-9 Stage Mapping

Status: active mapping
Owner: language maturity / qualification harness
Scope: PCC-4..PCC-9 evidence mapped into 7hell stages
Non-goal: implementation, fixture addition, CI gate, release readiness, or CTF closure

## 1. Purpose

This document maps existing PCC-4..PCC-9 evidence into the 7hell stage model.

It does not implement 7hell.

It does not add fixtures.

It does not add tests.

It does not create a release gate.

It does not claim readiness.

The purpose is to prevent future 7hell implementation from guessing which PCC evidence belongs to which stage.

## 2. Stage model

| Index | Stage                        | Key           | Meaning                                                           |
| ----: | ---------------------------- | ------------- | ----------------------------------------------------------------- |
|     1 | Syntax Hell                  | `syntax`      | source parses or fails with syntax diagnostic                     |
|     2 | Type Hell                    | `type`        | source passes semantic/type checks or fails with check diagnostic |
|     3 | Lowering Hell                | `lowering`    | checked source lowers deterministically                           |
|     4 | Verifier Hell                | `verifier`    | emitted SemCode is admitted or rejected before VM                 |
|     5 | VM Hell                      | `vm`          | verified SemCode runs or traps deterministically                  |
|     6 | Practical Hell               | `practical`   | real small program path works end-to-end                          |
|     7 | User Pain / Diagnostics Hell | `diagnostics` | failures are stable and actionable                                |

Rules:

- Compile/check diagnostics are not VM traps.
- Project diagnostics are not project-root execution traps.
- VM Hell must not run unverified SemCode.
- Practical Hell must not imply release readiness.
- Diagnostics Hell may run against failure output.
- 7hell consumes CTF evidence but does not close CTF.

## 3. PCC-to-stage mapping summary

| PCC   | Surface                | Syntax  | Type               | Lowering      | Verifier      | VM            | Practical            | Diagnostics | Notes                                             |
| ----- | ---------------------- | ------- | ------------------ | ------------- | ------------- | ------------- | -------------------- | ----------- | ------------------------------------------------- |
| PCC-4 | Records                | partial | yes                | yes           | yes           | if covered    | yes                  | yes         | fixture-backed, not full release gate            |
| PCC-5 | ADT + Match            | partial | yes                | yes           | yes           | yes           | yes                  | yes         | basic match only                                  |
| PCC-6 | Option / Result        | partial | yes                | yes           | yes           | yes           | yes                  | yes         | standard forms only                               |
| PCC-7 | Collections            | partial | yes                | yes           | yes           | yes           | yes                  | yes         | Sequence and admitted Map baseline only           |
| PCC-8 | Stdlib helpers         | partial | yes                | if applicable | yes           | yes           | yes                  | yes         | assert/print/to_text admitted helper surface only |
| PCC-9 | Project Model baseline | partial | project-diagnostic | no public run | no public run | no public run | project-adjacent only | yes         | Semantic.package baseline only                    |

Status vocabulary:

- `yes`
- `partial`
- `not-covered`
- `not-applicable`
- `future`
- `blocked`

Do not use `complete`.

Do not use `release-ready`.

## 4. PCC-4 Records mapping

### PCC-4 Records

Surface:

- record declaration / construction / field access / update if currently evidenced.

Evidence anchors:

- `tests/pcc4_records_acceptance.rs`
- `tests/pcc4_records_diagnostics.rs`
- `tests/fixtures/pcc4_records/`
- `tests/fixtures/pcc4_records_diagnostics/`

Stage mapping:

- Syntax Hell: fixture-backed record source shape coverage for accepted/rejected inputs.
- Type Hell: record type checking and field validation.
- Lowering Hell: record lowering path where acceptance fixtures reach lowering.
- Verifier Hell: emitted SemCode admission where record fixtures compile through verifier.
- VM Hell: runtime behavior only where fixtures execute admitted SemCode.
- Practical Hell: positive record examples through practical end-to-end path.
- Diagnostics Hell: negative record diagnostics from record error fixtures.

CTF references:

- CTF-E1 selected golden trace if record trace exists.
- runtime value registry if record value is freeze-candidate.
- determinism matrix if applicable.

Boundaries:

- no broad record freeze.
- no release readiness.

## 5. PCC-5 ADT + Match mapping

Evidence anchors:

- `tests/pcc5_adt_acceptance.rs`
- `tests/pcc5_match_acceptance.rs`
- `tests/pcc5_adt_diagnostics.rs`
- `tests/fixtures/pcc5_adt/`
- `tests/fixtures/pcc5_match/`
- `tests/fixtures/pcc5_adt_diagnostics/`

Stage mapping:

- Type Hell: constructor / variant / match typing.
- Lowering Hell: match lowering where evidenced.
- Verifier Hell: admitted SemCode before VM.
- VM Hell: positive match execution if covered.
- Practical Hell: ADT crossing function boundary.
- Diagnostics Hell: unknown constructor, payload mismatch, non-exhaustive match, arm type mismatch.

Boundaries:

- basic match only.
- no advanced pattern matching claim.
- no exhaustiveness redesign claim beyond current fixture behavior.

## 6. PCC-6 Option / Result mapping

Evidence anchors:

- `tests/pcc6_option_acceptance.rs`
- `tests/pcc6_result_acceptance.rs`
- `tests/pcc6_option_result_diagnostics.rs`
- `tests/fixtures/pcc6_option/`
- `tests/fixtures/pcc6_result/`
- `tests/fixtures/pcc6_option_result_diagnostics/`

Stage mapping:

- Type Hell: standard-form `Option(T)` and `Result(T,E)`.
- Lowering Hell: ADT-like constructor/match path if evidenced.
- Verifier Hell: SemCode admission.
- VM Hell: positive Some/None/Ok/Err execution if covered.
- Practical Hell: function boundary Option/Result cases.
- Diagnostics Hell: payload mismatch, wrong type, exhaustiveness, arm result mismatch.

Boundaries:

- no generic type system claim.
- no exception semantics.
- no hidden prelude claim.
- standard forms only.

## 7. PCC-7 Collections mapping

Evidence anchors:

- `tests/pcc7_sequence_acceptance.rs`
- `tests/pcc7_map_acceptance.rs`
- `tests/pcc7_collections_diagnostics.rs`
- `tests/ctf_e2_collection_replay.rs`
- `tests/fixtures/core_trust_freeze/replay/ctf_e2/`

Stage mapping:

- Type Hell: Sequence/Map admitted type surfaces.
- Lowering Hell: collection operation lowering if evidenced.
- Verifier Hell: SemCode admission.
- VM Hell: Sequence indexing/iteration/mutation, admitted Map insert/lookup/update.
- Practical Hell: function-boundary Sequence or admitted Map practical cases.
- Diagnostics Hell: Sequence out-of-bounds, empty pop, type mismatch, Map key/value mismatch.

CTF references:

- CTF-E2 replay determinism.
- CTF-E3 trap taxonomy for Sequence OOB / empty pop.

Boundaries:

- Map missing-key policy remains open.
- Map iteration policy remains open.
- collection quota/memory policy remains open.
- no broad collections freeze.

## 8. PCC-8 Stdlib mapping

Evidence anchors:

- `tests/pcc8_stdlib_acceptance.rs`
- `tests/pcc8_stdlib_diagnostics.rs`
- `tests/fixtures/pcc8_stdlib/`
- `tests/fixtures/pcc8_stdlib_diagnostics/`

Stage mapping:

- Type Hell: helper arity/type checking.
- Verifier Hell: helper calls admitted if applicable.
- VM Hell: `assert(true)` / `assert(false)` if runtime behavior covered.
- Practical Hell: basic helper use in examples.
- Diagnostics Hell: wrong arity, wrong argument type, unsupported `to_text`.

CTF references:

- CTF-E1 selected helper trace.
- CTF-E3 assert false / unsupported to_text taxonomy.

Boundaries:

- `debug_render` internal-only.
- `to_text` admitted-types-only.
- `print(text)` is admitted helper output, not host capability widening.
- no broad stdlib freeze.
- no host IO widening.

## 9. PCC-9 Project Model mapping

Evidence anchors:

- `tests/pcc9_project_model_acceptance.rs`
- `tests/pcc9_project_model_diagnostics.rs`
- `docs/roadmap/language_maturity/pcc9_project_model_contract.md`
- `docs/roadmap/language_maturity/core_trust_freeze/project_root_trust_policy.md`

Stage mapping:

- Syntax Hell: manifest/project-adjacent format checks where applicable.
- Type Hell: not normal language type stage unless source check is involved.
- Lowering Hell: not public project-root lowering yet.
- Verifier Hell: not public project-root verifier path yet.
- VM Hell: not public project-root execution yet.
- Practical Hell: project-adjacent helper evidence only.
- Diagnostics Hell: malformed manifest, missing fields, invalid dep shape, path escape, unresolved dependency alias.

Boundaries:

- current admitted baseline is `Semantic.package`.
- `semantic.toml` parser is not implemented.
- `src/main.sm` discovery is not implemented.
- project-root `smc check <project-root>` is not implemented.
- project-root `smc run <project-root>` is not implemented.
- `smc new` is not implemented.
- no package registry.
- no remote dependencies.
- no workspace claim.

## 10. 7hell report field mapping

| 7hell report field   | Source of mapping          | WP2 rule                                         |
| -------------------- | -------------------------- | ------------------------------------------------ |
| `stages[].key`       | stage model                | use canonical keys only                          |
| `stages[].status`    | evidence outcome           | use pass/fail/blocked/skip/not_implemented only  |
| `diagnostics[].kind` | CTF-E3 / diagnostics tests | keep check/project/vm separate                   |
| `evidence[].class`   | CTF evidence classes       | E2/E3/E4 references allowed                      |
| `ctf[].area`         | CTF policy docs            | reference only, no CTF closure                   |
| `boundaries[]`       | open scope items           | record open project-root, Map, stdlib boundaries |

Rules:

- No live report is generated by WP2.
- This is a mapping document only.
- Future implementation must use this mapping when generating reports.

## 11. Cross-stage evidence matrix

| PCC                 | Syntax        | Type             | Lowering | Verifier     | VM                           | Practical      | Diagnostics | Status       |
| ------------------- | ------------- | ---------------- | -------- | ------------ | ---------------------------- | -------------- | ----------- | ------------ |
| PCC-4 Records       | mapped        | mapped           | mapped   | mapped       | partial                      | mapped         | mapped      | mapping only |
| PCC-5 ADT/Match     | mapped        | mapped           | mapped   | mapped       | partial                      | mapped         | mapped      | mapping only |
| PCC-6 Option/Result | mapped        | mapped           | mapped   | mapped       | partial                      | mapped         | mapped      | mapping only |
| PCC-7 Collections   | mapped        | mapped           | mapped   | mapped       | mapped for selected baseline | mapped         | mapped      | mapping only |
| PCC-8 Stdlib        | n/a or mapped | mapped           | n/a      | n/a/implicit | mapped                       | mapped         | mapped      | mapping only |
| PCC-9 Project Model | future        | project-adjacent | future   | future       | future                       | partial/future | mapped      | mapping only |

Use `mapped`, `partial`, `future`, `n/a`, or `open`.

Important:

- Do not use `pass`.
- Do not use `complete`.
- Do not imply 7hell command currently executes those stages.

## 12. 7hell implementation implications

Future 7hell implementation should consume this mapping in this order:

```text
7HELL-S2 — cli(7hell): route skeleton stages to existing single-file check where safe
7HELL-S3 — cli(7hell): add stage result wiring for syntax/type diagnostics
7HELL-S4 — cli(7hell): add verifier / VM stage placeholders with explicit blocked states
7HELL-E1 — test(7hell): add first report snapshot tests
7HELL-E2 — test(7hell): map PCC-4..PCC-6 fixtures into stage snapshots
7HELL-E3 — test(7hell): map PCC-7..PCC-9 fixtures into stage snapshots
```

Implementation order may be adjusted.

Project-root 7hell must wait for PCC-9I.

7hell must not bypass verifier-first route.

## 13. Boundary ledger

| Boundary               | Status               |
| ---------------------- | -------------------- |
| 7hell command skeleton | landed in 7HELL-S1   |
| stage mapping          | defined by 7HELL-WP2 |
| stage execution        | future               |
| report snapshots       | future               |
| project-root 7hell     | future after PCC-9I  |
| CI release gate        | future, not admitted |
| release readiness      | not claimed          |

## 14. Final verdict

```text
7HELL-WP2 maps PCC-4..PCC-9 evidence into 7hell stages.
It does not implement stage execution.
It does not add fixtures.
It does not create a release gate.
It does not claim readiness.
```

## 15. Acceptance checklist

```markdown
- [ ] PCC-4 mapping exists
- [ ] PCC-5 mapping exists
- [ ] PCC-6 mapping exists
- [ ] PCC-7 mapping exists
- [ ] PCC-8 mapping exists
- [ ] PCC-9 mapping exists
- [ ] cross-stage matrix exists
- [ ] boundary ledger exists
- [ ] future implementation split is proposed
- [ ] no 7hell command behavior changed
- [ ] no tests or fixtures added
- [ ] no CI gate added
- [ ] no readiness claimed
- [ ] no CTF closure claimed
```
