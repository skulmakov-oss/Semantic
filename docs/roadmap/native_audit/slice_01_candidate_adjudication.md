# Semantic-native Readiness Audit — Slice 1: Candidate Adjudication

Status: NATIVE-AUDIT-01 — adjudication complete for C-01 … C-06; no repairs
Audit base SHA: `ce1cbd9f7bdd55f932ef9a268a61b00443cf7b0c`
Charter: [`charter.md`](charter.md) · Inventory: [`inventory.md`](inventory.md)

## Summary

| Candidate | Group | Verdict | Finding ID | Highest proven level |
|---|---|---|---|---|
| C-01 | N6 | KEEP (negative) | NONE | crates: workspace build+test (repository integrity); Workbench `.sm`: E0 |
| C-02 | N6 | DISPROVED (negative) | NONE | E0 (path allowlist; generated artifact) |
| C-03 | N2 → N6 owner | DEFER-SCOPE (negative for readiness) | NONE | E0 (embedded by `include_str!` into a CI-built binary with 0 tests) |
| C-04 | N1 | DISPROVED (negative) | NONE | file: E0; byte-identical content: E5+E6+E7+E9 |
| C-05 | N4 | DISPROVED (strong negative) | NONE | n/a (no implementation; no readiness claim) |
| C-06 | N8 | KEEP (negative) | NONE | 6 fixtures E7+E9; 30 fixtures E7 local-only; 1 fixture E7 local+ignored |

Confirmed findings: **0**. No `findings.md` registry is created (charter: no
empty registry for ceremony).

Two **audit-method corrections** to the Slice 0 inventory heuristic were found
while adjudicating (§ Method corrections). They are defects of the audit's own
derivation, not repository readiness findings, and are corrected in
[`inventory_overrides.tsv`](inventory_overrides.tsv) (summarized in `inventory.md` §9) without changing the reproducible Slice 0 query.

---

## C-01 — retired Workbench vs workspace CI

Candidate: C-01
Group: N6
Verdict: KEEP (negative finding)

Claim / confidence source: `prom-ui*` crates are Cargo workspace members, so
`ci.yml` (`cargo test --workspace --all-targets`) builds and tests them; Workbench
`.sm` sources are named by a doc comment in `crates/prom-ui-iced-adapter/src/lib.rs`.
Actual proven behavior: `docs/roadmap/ui_workbench_studio_retirement.md` (active
governance decision, 2026-09-30, #1968/#1862/#1969) states the crates are "Landed
on `main`, unqualified, retired. Crates remain workspace members and keep
compiling and testing as repository integrity; they are not repaired." Workbench
(`examples/workbench_semantic`, `artifacts/workbench/**`) is "Historical evidence
only. Excluded from the root workspace; not an active qualification surface"
(`Cargo.toml` `exclude = ["examples/workbench_semantic"]`). README states the
contour is "not remediated, qualified, or promoted". The Workbench `.sm` files
are not executed anywhere in CI; the only CI-side text naming them is a doc
comment.
Highest evidence level: crates — workspace build + unit tests (repository
integrity, not UI readiness); Workbench `.sm` — E0.
Hosted-CI evidence: `test-std` builds/tests `prom-ui*` crates; no job executes
Workbench `.sm`.
Gap: none. Green CI on these crates is explicitly scoped by the current
authority to repository integrity.
Classification: KEEP
Severity: none
Evidence: `docs/roadmap/ui_workbench_studio_retirement.md` (Retired contour table,
Qualification boundary); README "Native Semantic UI, Workbench, and Semantic
Studio are **retired**"; `Cargo.toml` members/exclude.
Negative evidence / alternative explanation: crate builds in workspace ≠ retired
UI product supported; the distinction is already explicit in the authority.
Repair note: none.

## C-02 — Workbench native-launch smoke

Candidate: C-02
Group: N6
Verdict: DISPROVED (negative finding)

Claim / confidence source: `artifacts/workbench/native-launch-smoke/smoke-project/main.sm`
was `CI_REFERENCED` through `scripts/check_repository_hygiene.py`.
Actual proven behavior: the hygiene script's allowlist names
`artifacts/workbench/native-launch-smoke/report.md` (and other artifact paths);
the inventory matched the `.sm` only through the shared ancestor directory. It
proves path admissibility, not execution. `scripts/workbench_native_launch_smoke.ps1`
*writes* this `main.sm` (`Set-Content … "fn main() { assert(1 == 1); return; }"`)
into a project directory and launches the Workbench executable on it; it is not
named by any workflow. The retirement decision records it as "a historical
manual script; its failure against the current root workspace (#1862) is
recorded and is not repaired".
Highest evidence level: E0 (tracked generated artifact).
Hosted-CI evidence: none (path allowlist only).
Gap: none — no active authority claims this smoke path is current or qualified.
Classification: none (DISPROVED)
Severity: none
Evidence: `scripts/check_repository_hygiene.py` allowlist;
`scripts/workbench_native_launch_smoke.ps1:35,52`;
`ui_workbench_studio_retirement.md` Qualification boundary.
Negative evidence / alternative explanation: historical evidence artifact, not
a readiness signal.
Repair note: none.

## C-03 — quad_logic_calculator

Candidate: C-03
Group: N2 (ownership resolved to the retired N6 UI contour)
Verdict: DEFER-SCOPE (negative for native readiness)

Claim / confidence source: inventory classified the two `.sm` as `MANUAL_ONLY`
via tracked, non-workspace `proj_test/src/main.rs`.
Actual proven behavior:
- `examples/quad_logic_calculator` **is** a workspace member (`Cargo.toml` line 49).
- `src/main.rs:30-31` embeds both sources with `include_str!("quad_calc.proj.sm")`
  and `include_str!("calculator.sm")`; the inventory's literal-path rules missed
  these bare-name embeds.
- The crate is a `[[bin]] quad_calc` that depends on `prom-ui`,
  `prom-ui-runtime`, `prom-ui-backend-native` (retired UI contour).
- `cargo test -p quad_logic_calculator -- --list` lists **0 tests**; CI therefore
  compiles the binary (embedding the text) but never compiles or runs the
  Semantic sources. `compile_program_to_semcode(CALCULATOR_SEMANTIC_SOURCE)`
  executes only when the binary is launched.
- `proj_test/src/main.rs` reads `../examples/experimental/quad_logic_calculator/…`,
  a path that does not exist; it is a stale manual probe.
- No current authority (README, `v1_readiness.md`, feature maturity matrix,
  canonical README, SSF-11 onboarding matrix) names the calculator.
  Historical `.harness/reports/SEMANTIC-WORKBENCH-NATIVE-V0.md` test counts are
  history, not current authority.
Highest evidence level: E0 (embedded in a CI-built binary).
Hosted-CI evidence: build only (`test-std` compiles the binary; 0 tests).
Gap: no current readiness claim exists, so no gap. The example is a UI
demonstration of the retired contour.
Classification: DEFER-SCOPE (owner: retired N6 UI contour)
Severity: none
Evidence: `Cargo.toml:49`; `examples/quad_logic_calculator/Cargo.toml`
dependencies; `src/main.rs:30-31,366`; `proj_test/src/main.rs:2,5`;
`cargo test -p quad_logic_calculator` → `0 passed`.
Negative evidence / alternative explanation: dynamic/embedded load exists
(`include_str!`), but only at build time; the inventory heuristic was
incomplete (method correction M-03), which this slice records.
Repair note: none.

## C-04 — readiness_draft qualified-limited packs (highest priority)

Candidate: C-04
Group: N1
Verdict: DISPROVED (negative finding)

Claim / confidence source: `examples/readiness_draft_canonical/README.md` reads
`cli_batch_core`, `rule_state_decision`, `data_audit_record_iterable`,
`wave2_local_helper_import` as "current reading: `qualified limited release`"
and `module_selected_import_settlement`, `module_selected_import_audit_report`
as "current reading: `out of scope on current main`". The README itself says the
pack "does not by itself widen the release contour" and contains "draft
source-shaped text artifacts for readiness planning".
Actual proven behavior, per inventory entry:

| File | Pack reading | Trace |
|---|---|---|
| `module_selected_import_audit_report/src/risk_policy.sm` | out of scope | no Rust/CI reference to the pack (`git grep readiness_draft_canonical -- '*.rs' '.github/workflows/*.yml'` → none); no qualified claim |
| `module_selected_import_audit_report/src/text_format.sm` | out of scope | same |
| `module_selected_import_settlement/src/rendering.sm` | out of scope | same |
| `module_selected_import_settlement/src/rules.sm` | out of scope | same |
| `wave2_local_helper_import/src/helper.sm` | qualified limited | byte-identical (`cmp`) to `examples/qualification/executable_module_entry/wave2_local_helper_import/src/helper.sm` and to `examples/canonical/wave2_local_helper_import/src/helper.sm` |

For the four "qualified limited" packs, every draft source is byte-identical to
a qualification copy (`cmp` SAME):
`cli_batch_core`, `rule_state_decision`, `data_audit_record_iterable` →
`examples/qualification/g1_real_program_trial/<pack>/src/main.sm`;
`wave2_local_helper_import/src/{main,helper}.sm` →
`examples/qualification/executable_module_entry/wave2_local_helper_import/src/`.
Those copies are executed in CI:
- `tests/g1_real_program_trial.rs` — `smc check` + `smc run` succeed for all four;
- `tests/executable_module_entry.rs::executable_module_entry_wave2_local_helper_import_checks_and_runs`
  — the entry imports `helper.sm`; check+run succeed, so the helper module is
  loaded indirectly through module discovery;
- `tests/g1_execution_integrity.rs::g1_execution_integrity_stage_summaries_match_current_baseline`
  asserts the exact per-stage summary (sema, IR names, `verify:names=…`,
  disasm, `run=ok`) for all four programs, and
  `…_repeated_compiles_are_byte_stable` asserts byte-stable compile/disasm.
Highest evidence level: draft files themselves — E0 (never loaded);
byte-identical content — E5 (verifier) + E6 (VM run) + E7 (exact stage summary
and determinism asserted) + E9 (`test-std`).
Hosted-CI evidence: `test-std` runs the three test targets above.
Gap: none. The out-of-scope packs claim nothing; the qualified-limited packs'
content is executed and asserted through the qualification copies. The draft
README's reading is accurate for the content it describes.
Classification: none (DISPROVED)
Severity: none
Evidence: commands above; local runs: `g1_real_program_trial` 6/6,
`executable_module_entry` 10/10, `g1_execution_integrity` 3/3.
Negative evidence / alternative explanation: a draft copy is not itself a
qualified artifact; qualification rests on the identical qualification copies.
If a draft copy ever diverged from its qualification copy, the README reading
would become unsupported — a future-drift risk, not a present gap.
Repair note: none.

## C-05 — self-hosting direction

Candidate: C-05
Group: N4
Verdict: DISPROVED (strong negative finding)

Claim / confidence source: README, backlog, WBS and feature maturity matrix
call self-hosting "the active strategic direction"; no Semantic compiler source
exists in this repository or in `skulmakov-oss/Semantic-Language` (0 `.sm`).
Actual proven behavior: every current-facing statement is qualified as
direction, not capability:
- README: "This is architecture work, not a release promise."
- `docs/roadmap/backlog.md:94-96`: "an architecture track, not a release promise".
- `docs/status/feature_maturity_matrix.md:58-60`: "architecture work, not a
  release status".
- `docs/roadmap/wbs.md:35-36`: direction plus link to #1910, no readiness wording.
- Issue #1910 (OPEN) defines the goal and a future C0 → C1 → C2 fixed-point
  protocol and lists missing foundations.
- `git grep` for readiness phrasing ("can self-host", "self-hosting is
  ready/supported/qualified/implemented", "bootstrap is ready") over `*.md`
  outside this audit: no match.
Highest evidence level: n/a (no implementation claimed).
Hosted-CI evidence: none; none claimed.
Gap: none.
Classification: none (DISPROVED)
Severity: none
Evidence: files/lines above; #1910 body.
Negative evidence / alternative explanation: an explicit roadmap goal is not
FALSE-READY because implementation has not started.
Repair note: none.

## C-06 — sm-vm profiling fixtures and the ignored test

Candidate: C-06
Group: N8
Verdict: KEEP (negative finding)

Claim / confidence source: 37 profiling fixtures referenced from
`crates/sm-vm/tests/vm_opcode_profile_workloads.rs` (24 `#[test]`, 1 `#[ignore]`).
Actual proven behavior:
- The whole test file is gated by `#![cfg(feature = "vm-profile")]`.
  `vm-profile` is not a default feature of `sm-vm`, no workflow passes it, and no
  workspace member enables it (`cargo metadata`: members enable `sm-vm` only
  with `std` / `disasm`). `cargo test -p sm-vm --test vm_opcode_profile_workloads`
  without the feature runs **0** tests. Hosted CI therefore never compiles this
  file.
- With the feature (local only): 23 passed + 1 ignored; `--ignored` runs the
  remaining 1. Each test does `compile_program_to_semcode` → `verify_semcode_token`
  → `require_entry("main")` → `run_verified_entry_semcode_with_profile` and
  asserts opcode-family properties (E3–E6, with structural opcode-count asserts).
- Six helper-boundary fixtures (`scalar_helper_boundary_{helper,inline}.sm`,
  `g2/scalar_helper_boundary_single_call_{helper,inline}.sm`,
  `g2/scalar_helper_boundary_call_chain_{helper,inline}.sm`) are additionally
  embedded by `crates/sm-vm/src/semcode_vm.rs` test module
  `helper_boundary_result_observation_tests` under default features;
  `helper_boundary_pair_equivalence_harness_matches_terminal_observations`
  compiles, verifies and runs each pair and asserts equal canonicalized terminal
  observables. This runs in hosted `test-std`.

Fixture → test reachability (37 fixtures):

| Fixtures | Test(s) | Ignored | Normal `cargo test` (CI) | Stage / assertion |
|---:|---|---|---|---|
| 6 | `helper_boundary_pair_equivalence_harness_matches_terminal_observations` (lib) + one `vm-profile` pair test each | no | **yes** | compile→verify→run, terminal-observable equivalence asserted (E7, E9) |
| 30 | one `vm-profile` test each (`profile_quad_logic_storm`, `profile_quad_match_dispatch`, `profile_fact_merge_kernel`, `profile_fact_intersect_kernel`, `profile_delta_like_kernel`, `profile_andromeda_fact_wave_64`, the `profile_scalar_*_pair` and `profile_scalar_g2_*_pair` tests, `profile_pulsar_p5a_evidence_probe_pair`, `profile_quad_surface_lowering_against_core_form`) | no | no (feature-gated) | compile→verify→run, opcode-family asserts (E7, local) |
| 1 | `profile_andromeda_fact_wave_256_local` | **yes** | no | same (E7, local, `--ignored`) |

Answers:
- Fixtures reachable **only** from the ignored test: exactly one —
  `andromeda_fact_wave_256.sm`.
- Documentation relying on it labels it consistently as "ignored/local"
  (`docs/roadmap/pulsar_p4h_sm_vm_profile_evidence.md:35`,
  `sm_vm_vm_m3_opcode_family_evidence.md:37`,
  `sm_vm_vm_m4_scalar_movement_audit.md:49`, …), and those records give the
  local command `cargo test -p sm-vm --features vm-profile … -- --ignored`.
  `docs/architecture/svm_verified_execution_core.md:66,86` defines the profiler
  as "feature-gated `vm-profile` local measurement … not production telemetry".
  No qualification authority relies on profiling fixtures.
- The ignored test is an intentionally larger local measurement workload, not a
  qualification gap.
Highest evidence level: 6 fixtures E7+E9; 31 fixtures E7 local only.
Hosted-CI evidence: `test-std` (6 helper-boundary fixtures via lib tests only).
Gap: none against any claim. The Slice 0 inventory's `CI_REFERENCED` for the
31 feature-gated fixtures was an overstatement of the audit itself (M-02).
Classification: KEEP
Severity: none
Evidence: `crates/sm-vm/Cargo.toml` `[features]`; test file line 1 `#![cfg(…)]`;
local runs: default 0 tests; `--features vm-profile` 23 passed / 1 ignored;
`--ignored` 1 passed; `cargo test -p sm-vm --lib helper_boundary_result_observation_tests` 3 passed.
Negative evidence / alternative explanation: one `#[ignore]` did not affect 37
fixtures; the real boundary is the feature gate, which is documented.
Repair note: none.

---

## Method corrections (Slice 0 inventory heuristic)

These are corrections to the audit's own derivation. They do not change any
repository claim, and they are recorded so later slices do not inherit them.

- **M-01 — over-broad `parent/name.sm` suffix.** The suffix `src/main.sm`
  matches any Rust file that names any `…/src/main.sm`. 8 rows were
  `CI_REFERENCED` only through it: the 6 `examples/readiness_draft_canonical/*/src/main.sm`
  and the 2 `examples/pcc_candidates/*/src/main.sm`. `git grep` finds no Rust or
  workflow reference to either directory. Adjudicated evidence:
  `INSPECTION_ONLY` (named only by Markdown).
- **M-02 — feature-gated referrers counted as CI.** A workspace `.rs` file can be
  excluded from CI compilation by `#![cfg(feature = …)]`. 31 `sm-vm` profiling
  fixtures are referenced only by the `vm-profile`-gated test. Adjudicated
  evidence: `MANUAL_ONLY`.
- **M-03 — bare-name `include_str!` missed.** `examples/quad_logic_calculator/src/main.rs`
  embeds `calculator.sm` / `quad_calc.proj.sm` by bare name, which none of the
  anchors matched; the stale `proj_test` match was used instead. Adjudicated
  evidence: `CI_REFERENCED` (build-time embed, E0).

The Slice 0 query and `inventory.tsv` are kept unchanged as the reproducible
heuristic output (Slice-0 raw snapshot — historical derivation, superseded where
adjudicated); the 50 adjudicated overrides live in `inventory_overrides.tsv`,
composed with the raw snapshot by the rule in `inventory.md` §0. Changing the
query would silently re-classify rows that have not been adjudicated.

## Commands used as evidence

```text
git grep readiness_draft_canonical -- '*.rs' '.github/workflows/*.yml'      # none
git grep "pcc_candidates\|readiness_draft_canonical" -- '*.rs' '.github/workflows/*.yml'  # none
cmp <draft> <qualification copy>                                            # SAME ×5
cargo test --test g1_real_program_trial                                     # 6 passed
cargo test --test executable_module_entry                                   # 10 passed
cargo test --test g1_execution_integrity                                    # 3 passed
cargo test -p sm-vm --test vm_opcode_profile_workloads                      # 0 tests (feature off)
cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads           # 23 passed, 1 ignored
cargo test -p sm-vm --features vm-profile --test vm_opcode_profile_workloads -- --ignored  # 1 passed
cargo test -p sm-vm --lib helper_boundary_result_observation_tests          # 3 passed
cargo metadata --format-version 1 --no-deps                                 # no member enables vm-profile
cargo test -p quad_logic_calculator -- --list                               # 0 tests
```

Locally executed ignored/feature-gated tests are investigation evidence only,
not hosted-CI evidence.
