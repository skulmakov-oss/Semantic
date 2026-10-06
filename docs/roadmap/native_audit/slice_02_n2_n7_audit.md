# Semantic-native Readiness Audit — Slice 2: N2 and N7 Audit

Status: NATIVE-AUDIT-02 — N2 AUDITED, N7 AUDITED; UNKNOWN resolved to 0; no repairs
Audit base SHA: `7d3020d4246c006e3e4c1dcb2d719652827942c5`
Charter: [`charter.md`](charter.md) · Inventory: [`inventory.md`](inventory.md) ·
Overrides: [`inventory_overrides.tsv`](inventory_overrides.tsv) ·
Findings: [`findings.md`](findings.md)

Naming note: `tests/legacy_guards.rs::ton618_content_inventory_is_explicit`
freezes the exact list of Markdown files that spell the TON618 compatibility
crate/binary names. This report therefore refers to them descriptively — the
**TON618 legacy CLI binary** (`src/bin/` shim) and the **TON618
compatibility-named primitive crate** — and `inventory.md` (already in that
inventory) carries the exact names. No guard was edited.

## Summary

| Group | Artifacts | Confirmed findings | Negative findings | Remaining UNKNOWN | Status |
|---|---:|---:|---:|---:|---|
| N2 | 7 `.sm` | 1 (NA-N2-001, P3, DEFER-SCOPE) | 6 | 0 | **AUDITED** |
| N7 | 1 `.sm` + `ton618_legacy/**` (22 files, 0 `.sm`) + TON618 compatibility relationship | 0 | 3 | 0 | **AUDITED** |

Former UNKNOWN rows (3) are all resolved; effective UNKNOWN = 0.

Evidence here is either hosted CI (`test-std`, release-asset qualification) or
explicitly marked **local investigation only** (not CI evidence).

---

## N2 — Examples and demonstrations

### N2 inventory (7 `.sm`)

| Path | Owner | Role | Claim level | Current authority |
|---|---|---|---|---|
| `examples/benchmarks/snake_core.sm` | examples / SSF-11 onboarding index | DEMO (F11 benchmark) | QUALIFIED_LIMITED ("Landed and qualified on `main`") | README; `docs/examples_index.md` F11 row; `examples/benchmarks/README.md` |
| `examples/benchmarks/snake_learning.sm` | examples; SSF-08 #1902 decision | DEMO (feature-completeness experiment) | ILLUSTRATIVE | `ssf08_1902_snake_learning_envelope_decision.md` §9 |
| `examples/semantic_policy_overdrive_trace.sm` | release-asset qualification | EXAMPLE (heavy policy trace) | QUALIFIED_LIMITED (release smoke scenario) | `docs/roadmap/release_asset_smoke_matrix.md`; SSF final verdict Scenario 3 |
| `examples/calculator.sm` | none current (historical v1-math example) | EXAMPLE | HISTORICAL | `CHANGELOG.md` (historical entry only) |
| `examples/semantic_policy_overdrive.sm` | none current | EXAMPLE (unwired stress sibling) | UNKNOWN → none claimed | none |
| `examples/quad_logic_calculator/src/calculator.sm` | retired N6 UI contour (C-03) | DEMO | HISTORICAL | none current |
| `examples/quad_logic_calculator/src/quad_calc.proj.sm` | retired N6 UI contour (C-03) | DEMO | HISTORICAL | none current |

### N2 artifact-by-artifact evidence

**`examples/benchmarks/snake_core.sm`** — verdict: KEEP (claim supported).
Execution path: `tests/snake_core_benchmark.rs::snake_core_passes_check_run_compile_verify`
runs `smc check`, `smc run`, `smc compile`, `smc verify`. The program asserts its
own advertised result (`assert(score == 0); assert(steps == 200);`, lines 113-114)
after printing `snake_core: score=0 steps=200`, so a successful CI run proves the
deterministic result. Highest level: **E7 + E9**. The examples-index claim
"Landed and qualified on `main`" with result `score=0 steps=200` is backed.
Benchmark ≠ performance qualification: the index and README describe it as a
deterministic headless benchmark; no performance claim exists.

**`examples/benchmarks/snake_learning.sm`** — verdict: KEEP (documented limit).
`tests/snake_learning_benchmark.rs`: `snake_learning_passes_check_compile_verify`
(check/compile/verify) and `snake_learning_completes_under_explicit_high_budget_envelope`
(runs to completion under an explicit budget; the program's embedded assertions
`total_score == 8`, `total_steps == 1417` are the asserted result). Default
`smc run` exceeds the default envelope; this is recorded and classified in
SSF-08 decision #1902 §9 ("a correctness fixture, not a benchmark in the
performance sense"). README advertises only `snake_core`, not `snake_learning`.
Highest level: **E7 + E9** (under the explicit envelope).

**`examples/semantic_policy_overdrive_trace.sm`** — verdict: KEEP (claim supported).
`tests/bytecode_compat.rs::compat_example_semantic_policy_overdrive_trace_runs_on_verified_path`
compiles, verifies, runs, and asserts disassembly content;
`tests/work_layer_guard_tests.rs` verifies it through the CLI;
`tests/quad_surface_lowering_profile.rs` asserts lowering-profile invariance.
Public qualification: `scripts/verify_release_assets.ps1` and
`docs/roadmap/release_asset_smoke_matrix.md` Scenario "Heavy semantic policy
trace"; `reports/semantic_stable_foundation_final_verdict.md:319` records
"Scenario 3 … PASS (compiled, verified, run, disassembled)". Highest level:
**E7 + E9 + E10**.

**`examples/calculator.sm`** (former UNKNOWN) — verdict: KEEP (historical, no claim).
Origin: commit `696f60d0` (2026-02-14, "v1-math: add f64, EXOBYTE1, math opcodes,
builtins, and golden fixture"); `CHANGELOG.md:131-132` lists it as a new example
alongside the golden fixture. It is **byte-identical** (`cmp`) to
`tests/golden_v1/calculator.sm`, which `tests/golden_semcode.rs::golden_v1_calculator`
compiles and compares byte-for-byte against `tests/golden_v1/calculator.smc` in
CI (E4 with asserted golden bytes, E9). The file itself is read by no code; it
is unrelated to `examples/quad_logic_calculator` (different program; the
similar name is not evidence). Local investigation only: `smc compile` →
`smc verify` → `smc run` all exit 0 (E6, not CI). No current README/index
advertises it. Current evidence: `INSPECTION_ONLY` (named only by
`CHANGELOG.md`); highest proven level of the file: E0; identical content: E4 + E9.

**`examples/semantic_policy_overdrive.sm`** (former UNKNOWN) — verdict: KEEP
(unclaimed, unwired).
Origin: commit `b74c36f8` (2026-03-15, "Add semantic policy stress regressions"),
which added this file and its `_trace` sibling but wired only the `_trace`
sibling into `tests/bytecode_compat.rs`. `git log -S` finds no commit that ever
referenced this file from code. Exact path, basename and stem searches over
`*.rs`, `*.ps1`, `*.py`, `*.yml`, `*.toml` and Markdown (outside this audit)
return nothing; no directory sweep covers `examples/` top level. No current
authority claims it. Local investigation only: compile → verify → run exit 0
(E6, not CI). Current evidence: `NOT_EXECUTED`; highest proven level: E0. The
commit message's word "regressions" is history, not a current claim; since
nothing presents the file as a regression test, there is no readiness gap.

**`examples/quad_logic_calculator/src/*.sm`** (2) — C-03 result preserved:
DEFER-SCOPE to the retired N6 UI contour; E0 (build-time `include_str!`, crate
has 0 tests). No contradictory evidence found.

### N2 claims traced

| Claim | Location | Backed by |
|---|---|---|
| snake_core benchmark, `smc run` | README "There is also a deterministic headless Snake benchmark" | E7+E9 (`snake_core_benchmark.rs`) |
| F11 "Landed and qualified on `main`", result `score=0 steps=200` | `docs/examples_index.md` F11 row | E7+E9 (in-program asserts) |
| Heavy policy trace release scenario | `release_asset_smoke_matrix.md`; SSF verdict Scenario 3 | E7+E9+E10 |
| snake_learning default-envelope limit | SSF-08 #1902 decision | tests under explicit envelope (E7+E9) |
| calculator / overdrive / quad_logic_calculator | none current | n/a |
| "SSF-12 (#1583) has issued no qualification verdict" | `docs/examples_index.md:79-80` | **contradicted**: #1583 closed 2026-10-04 with the published verdict → NA-N2-001 |

### N2 findings / negative findings

Confirmed: **NA-N2-001** (P3, DEFER-SCOPE) — stale SSF-12 status sentence in
`docs/examples_index.md`. See [`findings.md`](findings.md). It understates
rather than overstates readiness; the owning perimeter is release-posture
documentation (the #1996 reconciliation scope), so it is deferred, not repaired.

Negative findings (6): snake_core claim backed; snake_learning limit documented;
overdrive_trace qualification backed; calculator.sm historical with identical
CI-golden content; overdrive.sm unclaimed; quad_logic_calculator C-03 unchanged.

### N2 completion verdict

All 7 artifacts adjudicated; owners resolved (two have no current owner and no
claim, recorded as such); all claims traced; evidence levels established;
finding and negative findings recorded. **N2 = AUDITED.**

---

## N7 — Legacy / compatibility

### N7 inventory

| Surface | Position | Evidence |
|---|---|---|
| `assets/legacy_cli/human.sm` (+ `machine.sem`, `profile.json`, `samples.json`) | tracked legacy CLI sample bundle | below |
| `ton618_legacy/**` | 22 tracked files, 0 `.sm`; own `Cargo.toml`; not a workspace member; not built by CI | `Cargo.toml` members; `cargo metadata` |
| TON618 compatibility relationship | retained non-owning perimeter: TON618 legacy CLI binary, TON618 compatibility-named primitive crate, `ton618_legacy/` | `docs/roadmap/language_maturity/ton618_compatibility_perimeter_scope.md`; README "narrow compatibility perimeter … not second owners"; `tests/legacy_guards.rs` |

### N7 artifact/history evidence

**`assets/legacy_cli/human.sm`** (former UNKNOWN) — verdict: KEEP (historical).
History: created as `src/human.sm` in `5f4a64fa` (2026-02-12, "Toolchain v0");
moved by the root cleanup recorded in `docs/legacy-map.md:35` ("root sample data
files moved to assets"). Content is the four-statement "Human layer (L1)"
program (`x = TRUE`, `y = FALSE`, `out = x AND y`, `z = NOT out`).
Current execution paths:
- canonical `smc check` rejects it (`E0008 NO SURFACE CLAIM`) — expected, it is
  not Semantic source (local investigation);
- the TON618 legacy CLI binary's `lang run-human --in assets/legacy_cli/human.sm`
  fails without a profile ("unexpected trailing input" at `AND`), and with
  `--profile assets/legacy_cli/profile.json` fails to load the profile
  (`missing field 'identity'`) (local investigation). The retained
  `profile.json` is the original aliases-only format; `ParserProfile` has
  required `identity` since the `sm-profile` canonicalization (`22c803b4`,
  2026-03-13) and PB-01 (#1985) kept unknown/missing fields fail-closed. The
  sample bundle has therefore been non-loadable by the legacy CLI for months;
  no current document says it is runnable.
- `lang run-machine --in assets/legacy_cli/machine.sem` runs (local
  investigation; exit 0).
- The legacy human-layer language itself is proven in CI by the binary's own
  unit test `language::tests::compile_and_execute_human_program_with_profile`
  (root bin target, default features, run by `cargo test --workspace
  --all-targets`): it trains an alias profile and compiles and executes the
  same four statements, asserting the value of `z` (E7 + E9).
Claims: `docs/legacy-map.md` records the file move only; the legacy CLI usage
text uses `human.sm` / `profile.json` as placeholder names; no current authority
claims the asset bundle is runnable or supported. Owner: N7, unambiguous.
Current evidence: `INSPECTION_ONLY` (named only by `docs/legacy-map.md`);
highest proven level of the file: E0; identical program text: E7 + E9.
Negative finding: the stale `profile.json` incompatibility is real but carries
no readiness claim; it is recorded here, not registered as a finding, and not
repaired.

**`ton618_legacy/**`** — verdict: KEEP. "Retained historical source archive for
the pre-`sm-*` naming era" (perimeter scope doc). Not a workspace member, not
compiled by any workflow, file list frozen by
`legacy_guards::legacy_compatibility_perimeter_is_explicit_and_narrow`. Its
`README.md` is empty (0 bytes); its `Cargo.toml` package name collides with
the compatibility-named primitive crate's name but is never built. No claim of
support exists.

**TON618 compatibility relationship** — verdict: KEEP. The perimeter scope doc
(status: completed post-stable closure track) defines exactly three retained
paths, all "compatibility shims only", with "guard tests hold the allowed
perimeter mechanically". README says the paths "are not second owners of
Semantic architecture". The primitive crate is a workspace member whose tests
run in CI (repository integrity); its internals are outside this group's
question and were not re-audited.

### N7 claims traced

| Claim | Location | Status |
|---|---|---|
| retained non-owning compatibility perimeter, exactly 3 paths | perimeter scope doc; README | matches repository and guards |
| legacy assets moved from root | `docs/legacy-map.md` | historical record; no support claim |
| legacy CLI human-layer usage | TON618 legacy CLI binary usage text | language path proven by unit test (E7+E9); placeholder names only |

### N7 findings / negative findings

Confirmed: none. Negative findings (3): `human.sm` historical sample (stale
profile bundle, no claim); `ton618_legacy/` frozen unbuilt archive;
compatibility perimeter narrow and guarded.

### N7 completion verdict

`human.sm` adjudicated; legacy/compatibility claims traced; TON618 relationship
classified; no unresolved relevant claim; negative findings recorded.
**N7 = AUDITED.**

---

## UNKNOWN resolution

| Path | Owner | Role | Current evidence | Verdict |
|---|---|---|---|---|
| `assets/legacy_cli/human.sm` | N7 legacy perimeter | LEGACY sample | INSPECTION_ONLY (E0; same program E7+E9 via legacy unit test) | KEEP — historical |
| `examples/calculator.sm` | none current (historical example) | EXAMPLE | INSPECTION_ONLY (E0; identical content E4+E9 via `golden_v1_calculator`) | KEEP — historical |
| `examples/semantic_policy_overdrive.sm` | none current | EXAMPLE (unwired) | NOT_EXECUTED (E0; local E6 only) | KEEP — unclaimed |

## Inventory overrides (this slice)

Six rows added to `inventory_overrides.tsv` (source `NATIVE-AUDIT-02 / N2|N7`);
no existing override changed:

| Path | Slice-0 | Current | Proven level |
|---|---|---|---|
| `assets/legacy_cli/human.sm` | UNKNOWN | INSPECTION_ONLY | E0 |
| `examples/calculator.sm` | UNKNOWN | INSPECTION_ONLY | E0 |
| `examples/semantic_policy_overdrive.sm` | UNKNOWN | NOT_EXECUTED | E0 |
| `examples/benchmarks/snake_core.sm` | CI_REFERENCED | CI_EXECUTED | E7+E9 |
| `examples/benchmarks/snake_learning.sm` | CI_REFERENCED | CI_EXECUTED | E7+E9 (explicit envelope) |
| `examples/semantic_policy_overdrive_trace.sm` | CI_REFERENCED | CI_EXECUTED | E7+E9+E10 |

Current totals (raw + 56 overrides): CI_EXECUTED 9, CI_REFERENCED 494,
MANUAL_ONLY 31, INSPECTION_ONLY 12, NOT_EXECUTED 4, UNKNOWN 0 — total 550.

## Remaining audit perimeter

N1, N5, N6, N8: INVENTORIED — NOT AUDITED. N3: INVENTORIED — EMPTY. N4:
strategic/Bootstrap perimeter, not audited in this slice.

## Commands used as evidence

```text
git log --follow / -S ; git show b74c36f8 ; git grep (path, basename, stem)
cmp examples/calculator.sm tests/golden_v1/calculator.sm            # identical
cargo test --test golden_semcode                                    # golden_v1_calculator ok
cargo test --test snake_core_benchmark --test snake_learning_benchmark   # 1 + 2 passed
cargo test --test bytecode_compat --test work_layer_guard_tests --test quad_surface_lowering_profile  # passed
cargo test --bin <TON618 legacy CLI> compile_and_execute_human_program_with_profile  # 1 passed
cargo metadata --format-version 1 --no-deps                         # ton618_legacy not a member
# local investigation only (not CI evidence):
smc compile/verify/run examples/calculator.sm                       # exit 0
smc compile/verify/run examples/semantic_policy_overdrive.sm        # exit 0
smc check assets/legacy_cli/human.sm                                # E0008
<TON618 legacy CLI> lang run-human --in assets/legacy_cli/human.sm [--profile assets/legacy_cli/profile.json]  # exit 1
<TON618 legacy CLI> lang run-machine --in assets/legacy_cli/machine.sem   # exit 0
```
