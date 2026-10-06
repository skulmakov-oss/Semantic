# Semantic-native Readiness Audit — Charter

Status: active audit charter (NATIVE-AUDIT-00, Slice 0 — charter and inventory)
Audit base SHA: `d95bee488d26208f4a3a4215ae4dd34e04d429ee`
Inventory: [`inventory.md`](inventory.md). Current per-file evidence = Slice-0 raw snapshot [`inventory.tsv`](inventory.tsv) + adjudicated overrides [`inventory_overrides.tsv`](inventory_overrides.tsv) (composition rule: `inventory.md` §0)

This charter fixes the perimeter, the evidence model and the rules for the
Semantic-native readiness audit **before any finding is adjudicated or any
repair begins**. This slice adjudicates nothing, creates no findings and changes
no behavior.

## 1. Purpose

Map every place where native Semantic programs and their surrounding native
surfaces can create a stronger impression of readiness than repository evidence
proves. The object is false confidence: what a native artifact claims, what
actually executes, at which evidence level, and who owns it.

## 2. Relationship to #1617

- [#1617](https://github.com/skulmakov-oss/Semantic/issues/1617) — platform
  readiness self-deception / fail-open audit — is the **historical
  predecessor**. It is not rewritten, reopened, converted or closed by this
  audit.
- #1617 explicitly excluded `prom-ui*`, `semantic-hub*`, `semantic-core-*`,
  `core-lab`, `examples/*`, `examples/workbench_semantic`, `ton618-core` / the
  TON618 legacy perimeter, and the root composition crate `semantic_language`,
  and deferred "native `.sm` programs, products, examples, Bootstrap code and
  UI" to a separate audit.
- The Semantic-native readiness audit is the **separate successor perimeter**
  for that deferred material. It has no Phase-B numbering and no PB-08.
- #1617 findings (FA-*) are out of scope here; a native finding that turns out
  to be a platform defect is recorded as `DEFER-SCOPE` with a pointer, not
  re-adjudicated.

## 3. Immutable boundary

- C0 = `a1591bc1b3c0458ec617d704c4a0dcb9670b1fb5`
- C1 = `89641da8237f4fcefb50cf1958a50e4d4003aea7` = `v1.2.0^{}`
- `v1.2.0` is the published Stable Foundation release, Windows x64 only,
  PROMOTE WITH EXPLICIT LIMITS.
- Current `main` is post-C1 development and is not automatically another
  qualified release. Nothing in this audit widens the `v1.2.0` promise.

## 4. Authority hierarchy

1. Release-facing posture: `docs/roadmap/public_status_model.md` (vocabulary),
   `docs/roadmap/v1_readiness.md` (current posture authority),
   `reports/g1_release_scope_statement.md`.
2. Qualified release evidence:
   `reports/semantic_stable_foundation_final_verdict.md` and its SSF reports.
3. Current maturity: `docs/status/feature_maturity_matrix.md`, README.
4. Retirement decision for the UI contour:
   `docs/roadmap/ui_workbench_studio_retirement.md`.
5. Self-hosting direction: issue
   [#1910](https://github.com/skulmakov-oss/Semantic/issues/1910)
   (architecture work, not a release promise).
6. Per-directory READMEs (e.g. `examples/canonical/README.md`) are **claims to
   be audited**, not authority over the documents above.
7. Historical documents are evidence, not current authority.

## 5. Core distinctions (hard rules)

```text
file exists                 ≠ feature supported
parser accepts program      ≠ program has full semantics
example compiles            ≠ advertised feature is qualified
test exists                 ≠ test is executed in CI
test references a file      ≠ test executes that file
native .sm source exists    ≠ Semantic can self-host
audit coverage completed    ≠ defect-free
```

A higher readiness claim must never be inferred from a lower evidence level.

## 6. Perimeter and ownership groups

The repository evidence supports the suggested groups with two adjustments:
**N8** is added, and **N3/N4** are recorded as empty in this repository.

| Group | Contents | Owner |
|---|---|---|
| N1 — Canonical/qualification programs | `examples/canonical/**`, `examples/qualification/**`, `examples/readiness_draft_canonical/**`, `examples/pcc_candidates/**` | language/qualification contour (README, `v1_readiness.md`) |
| N2 — Examples and demonstrations | `examples/benchmarks/**`, `examples/quad_logic_calculator/**`, top-level `examples/*.sm` | examples (no current-authority owner beyond READMEs) |
| N3 — Product/application programs | none tracked in this repository | — |
| N4 — Bootstrap/self-hosting code | none tracked here; direction #1910; external repo `skulmakov-oss/Semantic-Language` holds docs only (0 `.sm`, open PRs #3 and #6 are documentation) | #1910 / Semantic-Language |
| N5 — Native support/composition crates excluded from #1617 | `semantic-hub`, `semantic-hub-turbovec`, `semantic-core-{backend,bench,capsule,exec,quad,runtime}`, `core-lab`, `ton618-core`, root crate `semantic_language` | respective crates |
| N6 — UI native surfaces | `prom-ui`, `prom-ui-runtime`, `prom-ui-backend-native`, `prom-ui-iced-adapter`, `prom-ui-demo`, `examples/workbench_semantic/**`, `artifacts/workbench/**` | **retired** contour (`ui_workbench_studio_retirement.md`) |
| N7 — Legacy/compatibility | `assets/legacy_cli/**`, `ton618_legacy/**` (no tracked `.sm`), TON618 compatibility reading of `ton618-core` | legacy perimeter |
| N8 — Platform test fixtures | `tests/fixtures/**`, `tests/golden*/**`, `crates/sm-vm/tests/**`, `crates/sm-front/tests/**` | owning platform crate/test (#1617 modules) |

Deviations, with reasons:

- **N8 added.** 310 of 550 tracked `.sm` files are inputs to platform tests of
  crates #1617 already audited. They are not native programs that make product
  claims, but they are the evidence that native claims lean on. They are in
  perimeter only as *evidence carriers*: the audit asks whether they support
  the claims made elsewhere, not whether the platform crates are correct.
- **N3 empty.** No tracked product/application Semantic program exists. N3 is
  kept so a later product cannot enter unaudited.
- **N4 empty in-repo.** Bootstrap is referenced as direction (#1910, README,
  backlog, WBS, feature maturity matrix) and has no executable Semantic source
  anywhere in either repository. Its claim level is `EXPERIMENTAL` direction
  only; no CI executes Bootstrap.
- **N5/N6 contain crates, not `.sm`.** They are counted by crate, not by file;
  their native audit is about claims, not `.sm` coverage.

## 7. Explicit exclusions

- The 18 canonical platform modules audited by #1617 (except as N8 evidence
  carriers).
- UI/product **presentation** readiness. N6 is audited only for whether retired
  material still produces readiness claims or CI signal; the native readiness
  audit does not evaluate, revive or implement UI.
- Bootstrap implementation; other repositories are read-only provenance.
- `experiments/**`, `third_party/**`, `scratch/**`, untracked/local build
  outputs.
- Release assets, tags, C0/C1.

## 8. Evidence ladder

| Level | Meaning |
|---|---|
| E0 | textual presence only (file or reference exists) |
| E1 | parses |
| E2 | semantic/type admission |
| E3 | lowers successfully |
| E4 | SemCode emitted |
| E5 | verifier admits |
| E6 | VM/runtime executes |
| E7 | expected deterministic result asserted |
| E8 | negative/adversarial cases asserted |
| E9 | hosted CI exercises the path |
| E10 | public qualification explicitly covers it |

E9 and E10 are orthogonal confirmations: they raise confidence only for the
level actually exercised (a CI job that only parses a file is E1+E9, not E6).
Slice 0 establishes at most **E0 plus a CI reference**; no file is placed above
E0 by this inventory.

## 9. Claim and role vocabulary

Roles: `CANONICAL`, `QUALIFICATION`, `READINESS_DRAFT`, `EXAMPLE`, `DEMO`,
`PRODUCT`, `BOOTSTRAP`, `LEGACY`, `TEST_FIXTURE`, `UNKNOWN`.

Claim levels: `PROMISED`, `QUALIFIED_LIMITED`, `INTERNAL_CONTRACT`,
`ILLUSTRATIVE`, `EXPERIMENTAL`, `HISTORICAL`, `UNKNOWN`.

Execution evidence (refined from the suggested set, see `inventory.md` §2):
`CI_REFERENCED` (named by a CI-run Rust target or a CI-invoked script),
`MANUAL_ONLY` (named only by non-CI code), `INSPECTION_ONLY` (named only by
Markdown), `NOT_EXECUTED` (named nowhere), `UNKNOWN` (stem-only/dynamic
match). `CI_EXECUTED`, `TEST_EXECUTED` and `PARSE_ONLY` are reserved for
adjudicated evidence in later slices; the inventory does not assign them,
because a reference does not prove execution.

`UNKNOWN` is never promoted by intuition.

## 10. Finding taxonomy

Carried from #1617: `FALSE-READY`, `FAIL-OPEN`, `SEMANTIC-DRIFT`,
`INERT-CONTRACT`, `QUALIFICATION-GAP`, `IDENTITY/TRUST-GAP`, `KEEP`,
`DEFER-SCOPE`. A finding may carry several. No native-specific extension is
needed at Slice 0.

## 11. Severity model (definitions only)

- **P0** — catastrophic: wrong code trusted/executed, verifier/capability
  bypass, or a public release claim directly falsified. Requires owner
  escalation.
- **P1** — native program semantics silently differ from the claimed meaning,
  or a qualified-limited claim rests on evidence that does not run.
- **P2** — unsupported surface presented as supported, qualification gap that
  materially overstates readiness, owner/claim missing for a promoted artifact.
- **P3** — presentation/documentation drift with no material readiness
  implication.

Slice 0 rates nothing.

## 12. Audit rules

- **No repair.** No implementation, test, `.sm`, or gate change in any audit
  slice; repairs require separate owner authorization after the map.
- No findings are created in Slice 0. Anomalies seen while inventorying are
  recorded only as `CANDIDATE — NOT ADJUDICATED`.
- No FA-style IDs. Native finding IDs use `NA-<group>-<nnn>` (e.g.
  `NA-N1-001`) and are assigned only at adjudication.
- Open findings do not block continuation through the forensic audit, unless a
  catastrophic P0 requires owner escalation.
- Every count must be reproducible from repository state at the stated SHA.

## 13. Per-area completion criteria

A group is `AUDITED` only when:

1. every artifact in the group has an adjudicated evidence level (E0–E10);
2. every public/README/doc claim about the group is traced to that evidence;
3. every `UNKNOWN` is resolved or explicitly recorded as unresolvable;
4. every confirmed finding is registered with classification and severity;
5. important negative results (candidates disproved) are recorded.

Status values: `INVENTORIED — NOT AUDITED` → `IN AUDIT` → `AUDITED`.

## 14. Whole-audit Definition of Done

- N1–N8 each `AUDITED` (empty groups confirmed empty at the audit SHA);
- all claims in the authorities of §4 that touch native programs traced;
- all findings registered; severity recorded;
- no repair mixed into the audit;
- final verdict states coverage completion, explicitly **not** defect-freedom.

## 15. Future finding-record format

```text
ID:            NA-<group>-<nnn>
Title:
Classification: FALSE-READY | FAIL-OPEN | SEMANTIC-DRIFT | INERT-CONTRACT |
                QUALIFICATION-GAP | IDENTITY/TRUST-GAP | KEEP | DEFER-SCOPE
Severity:      P0 | P1 | P2 | P3
Claim:         where and what is claimed (file:line / doc anchor)
Evidence level actually proven: E<n> (with the test/CI job that proves it)
Gap:           claimed level vs proven level
Evidence:      exact files, tests, CI jobs, audit SHA
Repair note:   optional one line; no implementation during audit
```
