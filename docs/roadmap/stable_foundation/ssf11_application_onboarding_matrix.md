# SSF-11 Application and Onboarding Matrix

Status: SSF-11 (#1582) canonical mapping, completed phase evidence (merged
through PR #1976, commit `bd4e34ebe219037bcb5330c1c3666d03dd0ec11d`)
Umbrella: #1569 — Semantic Stable Foundation
Base: `main` at `ea0d6dcacc70cada5b1fc80e23a5560ecddccb94` (SSF-10 / #1581
closed through merged PR #1975)

This document maps every one of the twelve #1582 application families to its
evidence, its owning contract, and its disposition. It is the human-readable
companion of the machine-readable corpus:

- corpus: `examples/qualification/ssf11/corpus.json`
  (schema `semantic.foundation.ssf11.corpus` v1)
- replay test: `tests/ssf11_canonical_applications.rs`
- onboarding/drift guard: `tests/ssf11_onboarding_docs.rs`

This matrix is evidence for SSF-12 (#1583). It is **not** a stable-release
claim, and it does not promote anything. SSF-12 has issued no verdict.

## Disposition vocabulary

| Value | Meaning |
|---|---|
| `PASS-REUSED` | Existing canonical evidence satisfies the family; SSF-11 indexes it and replays it through the public CLI. |
| `PASS-NEW` | SSF-11 adds the missing case(s) under `examples/qualification/ssf11/`, using only already-admitted surface. |
| `EXCLUDED-JUSTIFIED` | The family needs a capability that a prior phase deliberately left outside the Foundation contour. The exclusion carries deterministic boundary evidence. It is not a pass. |
| `RETURN-TO-OWNER` | An unintended gap that belongs to an earlier phase. |
| `BLOCKED` | SSF-11 cannot proceed without an earlier-phase repair. |
| `HISTORICAL-NON-QUALIFYING` | Useful historical evidence only; never Foundation qualification. |

## Twelve-family matrix

Every command below goes through the real `smc` binary. `{entry}` is the
entry path and `{artifact}` is a fresh `.smc` path. "Full pipeline" means
`check`, `compile -o`, `verify`, `run-smc`, `run`.

| ID | Required family | Existing evidence | Stable contract owner | Profile | Maturity | Positive evidence | Negative evidence | Expected observable result | Corpus disposition | Bootstrap-comparable | Qualification role |
|---|---|---|---|---|---|---|---|---|---|---|---|
| F01 | Minimal program | `examples/canonical/*` (none minimal) | SSF-01 language contract; SSF-10 artifact identity | `pure` | Landed and qualified on `main` | `examples/qualification/ssf11/f01_minimal/main.sm`, full pipeline, plus `smc artifact hash` | n/a: the F10 assertion case covers failure | every step exits 0; hash has the `sha256:<64 hex>` shape | PASS-NEW | yes (exit statuses); the artifact digest is Foundation-only | Foundation + onboarding |
| F02 | Quad decision engine | `match_control_flow`, `rule_state_decision` | SSF-01; `docs/spec/types.md` evidence algebra; `branch_condition_quad_rule.md` | `pure` | Landed and qualified on `main` | `f02_quad_decision/main.sm` (exact N/F/T/S dispatch and evidence-plane algebra) plus both reused examples | `f02_quad_bare_condition/main.sm`: `check --format json` gives `E0201` | exit 0; `T && F` is `N`, `T \|\| F` is `S`, `S \|\| T` is `S`, `!T` is `F` | PASS-NEW (plus 2 PASS-REUSED cases) | yes | Foundation |
| F03 | Records / enums / Option / Result | `option_result_control_flow`, `rule_state_decision`, `tests/fixtures/pcc5_*` | SSF-01 / SSF-07 | `pure` | Landed and qualified on `main` | `f03_records_enums_option_result/main.sm` (one coherent program) plus `option_result_control_flow` | `tests/fixtures/pcc5_adt_diagnostics/negative_match_missing_variant.sm`: `E0201` | exit 0 with asserted enum, record, Option and Result outcomes | PASS-NEW | yes | Foundation |
| F04 | Sequence / Map processing | `collections_core`, `text_collections_toolbox` | SSF-03 stdlib v0 / SSF-07 | `pure` | Landed and qualified on `main` | both canonical examples, full pipeline | `tests/fixtures/pcc7_collections_diagnostics/negative_map_key_type_mismatch.sm`: `E0201` | exit 0 | PASS-REUSED | yes | Foundation + onboarding |
| F05 | Generic + trait-based reusable helper | `data_audit_record_iterable` (direct-record `Iterable`) | SSF-07: matrix rows "First-wave generics: IR monomorphisation / execution" (**Roadmap**) and "Broader static traits" (dispatch/UFCS deferred) | `pure` | Generic execution: Roadmap; Iterable slice: Qualified limited release | the admitted trait slice `data_audit_record_iterable` (reused) | `f05_generic_boundary/main.sm`: `check` exits 0 (frontend admits it), `compile`/`run` exit 1 with "concrete IR monomorphisation is not implemented", and no artifact is written | fail-closed before SemCode exists | **EXCLUDED-JUSTIFIED** | no: the only discriminator is English wording without a stable code (see returns) | Boundary |
| F06 | Module and local package composition | `pcc9_project_root_*`, `positive_selected_import`, temp-only fixtures in `tests/ssf06_package_baseline.rs` | SSF-05 Project Model v0; SSF-06 Package Baseline v0 | `pure` | Landed and qualified on `main` | `f06_project` (`semantic.toml`, `lib.sm` import, `smc test`); `f06_packages/app` -> `../mathlib` (`check`/`compile`/`verify`/`run`, deterministic `package inspect`) | `f06_package_missing_dep` (missing manifest), `f06_project_entry_escape` (entry escapes root) | exit 0; `smc test` prints exactly `ok tests/double.sm` | PASS-NEW | yes (fingerprint values not frozen; see note) | Foundation + onboarding |
| F07 | Deterministic structured serialization | none in Semantic source | SSF-03: `std.serde` **Roadmap**, "no Semantic source serialization API or encoding is claimed" | `pure` | Roadmap | n/a | `f07_serialization_boundary/main.sm`: `to_json` is an unknown function (`E0201`) | rejection at check | **EXCLUDED-JUSTIFIED** (deliberate SSF-03 decision, so not RETURN) | yes (the boundary) | Boundary |
| F08 | Controlled CLI file transform | `examples/qualification/ssf04_file_transform.sm`, `tests/ssf04_application_boundary.rs` | SSF-04 `controlled_application_boundary_v0.md` | `cli-file-transform` | Landed and qualified on `main` | `f08_file_transform/main.sm` with `--profile cli-file-transform --root .`: `output.txt` is exactly `transformed:hello\n`; nothing else created; replay-identical audit | `cli-read-only` denies `fs.write`; `../escaped.txt` denied (nothing created outside the root); `pure` denies `args.read` | exact bytes and audit `decision=allow/deny` lines | PASS-NEW | yes | Foundation + onboarding |
| F09 | Contract / verifier rejection | `tests/fixtures/ssf10_compatibility/unsupported_semcode/corrupt.smc` | verifier (`docs/spec/verifier.md`), SSF-10 compatibility | n/a (artifact) | Landed and qualified on `main` | rejection family | new `f09_verifier_rejection/unsupported_header.smc` (`[UnsupportedVersion]`); reused `corrupt.smc` (`[InvalidFunctionName]`); `run-smc` refuses both with empty stdout | `verify` and `run-smc` exit 1 with a stable category | PASS-NEW | yes | Foundation |
| F10 | Deterministic runtime trap / quota | `ssf08_1763` taxonomy, `ctf_e3_trap_taxonomy_regression.rs` | SSF-08 runtime failure taxonomy and quotas | `pure` | Landed and qualified on `main` | rejection family | `division_by_zero.sm` -> `runtime trap: DivisionByZero`; `step_quota.sm` -> `quota exceeded: Steps limit=100000 used=100001`; `assertion_failure.sm` -> `assertion failed`; all verified before execution and replay-identical | exact identifiers, never wall-clock | PASS-NEW | yes | Foundation |
| F11 | Benchmark-class non-trivial application | `examples/benchmarks/snake_core.sm`, `snake_learning.sm` | SSF-08 (#1902 envelope decision) | `pure` | Landed and qualified on `main` | `snake_core.sm` full pipeline; both `run-smc` (verified artifact) and `run` print exactly `snake_core: score=0 steps=200` | `snake_learning.sm` verifies, but `smc run` fails closed under the default envelope with the exact Steps quota; the full run is proven only under the explicit test envelope in `tests/snake_learning_benchmark.rs` | exact output; timing informational only | PASS-REUSED | yes | Foundation |
| F12 | Native Semantic application evidence | retired Workbench / Studio / native UI (`docs/architecture/ui_boundary_index.md`) | retired (#1968, #1862 closed not planned) | n/a | Out of scope | none | none | none | **HISTORICAL-NON-QUALIFYING** | no | Historical only |

### Notes on the exclusions

- **F05.** No IR monomorphisation or specialization pass exists. Since #1717
  every generic function declaration fails at the shared IR lowering
  boundary. SSF-11 does not implement monomorphisation, trait dispatch, UFCS,
  trait objects or associated types. The only executable trait evidence is
  the bounded direct-record `Iterable` slice. For Bootstrap, a conforming
  implementation must reject the generic helper at the same phase (frontend
  admission, then no executable artifact). It must not run it.
- **F07.** SSF-03 deliberately selected no serialization API or encoding.
  JSON emitted by Rust tooling (`smc check --format json`,
  `smc package inspect`) is toolchain output, not Semantic stdlib. A
  hand-written `text` rendering is possible with admitted `to_text` and
  concatenation, but it is not "structured serialization", and SSF-11 does
  not present it as such.
- **F12.** Native Semantic UI, Workbench and Semantic Studio are retired. The
  historical evidence still documents the UI/ABI capability boundary
  (`docs/spec/ui_*`), but it is not part of Foundation qualification. No UI
  code was touched or revived.

### Note on fingerprints and digests

`smc package inspect` fingerprints and `smc artifact hash` digests are
deterministic for identical bytes. The corpus freezes their *shape* and
replay identity, not their values: `examples/qualification/**` has no
line-ending pin, so a CRLF checkout would legitimately change them. The
byte identity of Rust-produced SemCode is SSF-10 artifact-trust evidence. It
is not a Bootstrap equivalence criterion.

## Onboarding topic ownership

| # | Topic | Canonical owner |
|---|---|---|
| 1 | Getting Started | `docs/getting_started.md` |
| 2 | Language Tour | `docs/LANGUAGE.md`, then `docs/spec/source_semantics.md` |
| 3 | Semantic By Example | `docs/examples_index.md`, then `examples/canonical/README.md` and this matrix |
| 4 | Project Model | `docs/spec/project_model_v0.md` |
| 5 | Package Baseline | `docs/spec/package_baseline_v0.md` |
| 6 | Standard Library | `docs/spec/foundation_stdlib_v0.md` |
| 7 | CLI | `docs/spec/cli.md` |
| 8 | Diagnostics | `docs/spec/diagnostics.md`, `docs/spec/diagnostics_machine_schema_v1.md` |
| 9 | Verifier / Runtime | `docs/spec/verifier.md`, `docs/spec/runtime.md`, `docs/spec/quotas.md` |
| 10 | Compatibility / Migration | `docs/roadmap/compatibility_statement.md` |
| 11 | Troubleshooting | `docs/getting_started.md` § Troubleshooting |
| 12 | Release Status | `docs/roadmap/public_status_model.md`, `docs/status/feature_maturity_matrix.md` |

## Earlier-phase returns discovered by SSF-11

None of these block SSF-11. Each one belongs to its owning phase and was not
fixed here.

| Finding | Owning phase | Evidence |
|---|---|---|
| `docs/spec/types.md` and `docs/spec/branch_condition_quad_rule.md` list `known(x)`, `unknown(x)`, `conflict(x)` as readable quad predicates, but `smc check` rejects all three as `unknown function` (`E0201`). Docs and admission have drifted. | SSF-01 | one-line probe `assert(conflict(S));` |
| String literals have no escape processing, and none is documented: `"\n"` is written as the two bytes `\` `n`. The published SSF-04 example `examples/qualification/ssf04_file_transform.sm` appends `"\nsemantic.foundation.application/0.1\n"`, which reads as newlines but emits literal backslashes. | SSF-01 (text literal contract) / SSF-04 (example wording) | `smc run ... --profile cli-file-transform` output bytes |
| The F05 generic-execution rejection has no stable diagnostic code, only the #1717 wording, so the case is not Bootstrap-comparable yet. | SSF-07 / SSF-09 | `f05_generic_boundary` |
| Some failure messages embed absolute host paths (for example the missing-dependency diagnostic, and the `sources[].path` field in `check --format json`). The corpus therefore matches stable substrings only. | SSF-06 / SSF-09 | `f06_package_missing_dep` |
