# Semantic-native Readiness Audit — Inventory

Status: INVENTORIED — NOT AUDITED (NATIVE-AUDIT-00, Slice 0)
Audit base SHA: `d95bee488d26208f4a3a4215ae4dd34e04d429ee`
Charter: [`charter.md`](charter.md)
Per-file rows (550): [`inventory.tsv`](inventory.tsv)

Every number below is machine-derived from `git ls-files` at the audit SHA by
the query in §7. Nothing is estimated. A row's execution evidence records a
**reference**, never proven execution depth (charter §5, §8).

## 1. Totals

`git ls-files '*.sm' | wc -l` → **550** tracked `.sm` files.

| Group | Files | Role / claim level |
|---|---:|---|
| N1 | 229 | CANONICAL / QUALIFIED_LIMITED 17; QUALIFICATION / INTERNAL_CONTRACT 199; READINESS_DRAFT / UNKNOWN 11; READINESS_DRAFT / EXPERIMENTAL 2 |
| N2 | 7 | DEMO / ILLUSTRATIVE 2; EXAMPLE / UNKNOWN 5 |
| N3 | 0 | — |
| N4 | 0 | — (external repo `Semantic-Language`: 0 `.sm`) |
| N5 | 0 `.sm` | 10 crates (see §5) |
| N6 | 3 | LEGACY / HISTORICAL 3 (+5 `prom-ui*` crates) |
| N7 | 1 | LEGACY / UNKNOWN 1 |
| N8 | 310 | TEST_FIXTURE / INTERNAL_CONTRACT 310 |
| **Total** | **550** | |

Role counts: QUALIFICATION 199, CANONICAL 17, READINESS_DRAFT 13, EXAMPLE 5,
DEMO 2, LEGACY 4, TEST_FIXTURE 310, PRODUCT 0, BOOTSTRAP 0, UNKNOWN 0.

Claim-level counts: INTERNAL_CONTRACT 509, QUALIFIED_LIMITED 17, UNKNOWN 17,
HISTORICAL 3, EXPERIMENTAL 2, ILLUSTRATIVE 2, PROMISED 0.

Claim levels are taken from the owning README/authority: `examples/canonical`
README states its positive pack is inside the `qualified limited release`
contour; `examples/readiness_draft_canonical` README states it is mixed, so
it is `UNKNOWN` per file until adjudicated; `examples/pcc_candidates` README
states "probe samples … intentionally not canonical" (`EXPERIMENTAL`);
`examples/benchmarks` README describes demonstrations (`ILLUSTRATIVE`);
Workbench is retired (`HISTORICAL`). Where no authority states a claim, the
value is `UNKNOWN`.

## 2. Execution evidence

| Evidence | Files |
|---|---:|
| CI_REFERENCED | 540 |
| MANUAL_ONLY | 2 |
| INSPECTION_ONLY | 2 |
| UNKNOWN | 6 |
| NOT_EXECUTED | 0 |

| Group | CI_REFERENCED | MANUAL_ONLY | INSPECTION_ONLY | UNKNOWN |
|---|---:|---:|---:|---:|
| N1 | 224 | 0 | 2 | 3 |
| N2 | 3 | 2 | 0 | 2 |
| N6 | 3 | 0 | 0 | 0 |
| N7 | 0 | 0 | 0 | 1 |
| N8 | 310 | 0 | 0 | 0 |

Derivation rules (exact, see §7):

- A file is *referenced* when its full path, its `parent/name.sm` suffix, or an
  ancestor directory of depth ≥ 2 (excluding the over-broad `examples`,
  `tests`, `tests/fixtures`, `crates`, `crates/sm-vm`, `crates/sm-vm/tests`)
  appears as text in a source.
- `CI_REFERENCED`: referenced by a `.rs` file inside a Cargo workspace member
  (`ci.yml` runs `cargo test --workspace --all-targets`), by a workflow, or by
  a `scripts/*` file a workflow names.
- `MANUAL_ONLY`: referenced only by non-workspace `.rs` or by scripts no
  workflow names.
- `UNKNOWN`: only the file stem appears in workspace `.rs` (possible dynamic
  path construction).
- `INSPECTION_ONLY`: referenced only by Markdown.
- `NOT_EXECUTED`: referenced nowhere.

Ceiling of these rules: `CI_REFERENCED` is **E0 + a CI-run referrer**. A
referrer may be a doc comment, a hygiene path check, an `#[ignore]`d test, or
a directory sweep that only lints text. CI-executed / test-executed /
parse-only are **not** assigned here; 37 rows have a referrer containing
`#[ignore]` (column `ref_file_has_ignore`), which is a pointer for Slice 1, not
a conclusion. The tested-shape of each referrer is Slice-1 work.

## 3. By major directory

| Directory | Files |
|---|---:|
| `tests/fixtures` | 263 |
| `examples/qualification` | 199 |
| `crates/sm-vm` | 37 |
| `examples/canonical` | 17 |
| `examples/readiness_draft_canonical` | 11 |
| `tests/golden_snapshots` | 5 |
| `tests/golden` | 3 |
| `examples/workbench_semantic` | 2 |
| `examples/quad_logic_calculator` | 2 |
| `examples/pcc_candidates` | 2 |
| `examples/benchmarks` | 2 |
| `examples/*.sm` (top level) | 3 |
| `tests/golden_v1` | 1 |
| `crates/sm-front` | 1 |
| `assets/legacy_cli` | 1 |
| `artifacts/workbench` | 1 |

## 4. Non-CI-referenced and unresolved entries (10)

| Path | Group | Evidence |
|---|---|---|
| `assets/legacy_cli/human.sm` | N7 | UNKNOWN |
| `examples/calculator.sm` | N2 | UNKNOWN |
| `examples/semantic_policy_overdrive.sm` | N2 | UNKNOWN |
| `examples/quad_logic_calculator/src/calculator.sm` | N2 | MANUAL_ONLY (`proj_test/src/main.rs`) |
| `examples/quad_logic_calculator/src/quad_calc.proj.sm` | N2 | MANUAL_ONLY (`proj_test/src/main.rs`) |
| `examples/readiness_draft_canonical/module_selected_import_audit_report/src/risk_policy.sm` | N1 | INSPECTION_ONLY |
| `examples/readiness_draft_canonical/module_selected_import_audit_report/src/text_format.sm` | N1 | INSPECTION_ONLY |
| `examples/readiness_draft_canonical/module_selected_import_settlement/src/rendering.sm` | N1 | UNKNOWN |
| `examples/readiness_draft_canonical/module_selected_import_settlement/src/rules.sm` | N1 | UNKNOWN |
| `examples/readiness_draft_canonical/wave2_local_helper_import/src/helper.sm` | N1 | UNKNOWN |

Unresolved ownership: N2 top-level `examples/*.sm` and `examples/quad_logic_calculator`
(no current-authority document owns them) and N7 `assets/legacy_cli/human.sm`.

## 5. Crate-level perimeter (N5, N6)

| Crate | Group | Workspace member |
|---|---|---|
| `semantic_language` (root) | N5 | yes |
| `semantic-hub`, `semantic-hub-turbovec` | N5 | yes |
| `semantic-core-backend`, `-bench`, `-capsule`, `-exec`, `-quad`, `-runtime` | N5 | yes |
| `core-lab` | N5 | yes |
| `ton618-core` | N5 / N7 (TON618 compatibility reading) | yes |
| `prom-ui`, `prom-ui-runtime`, `prom-ui-backend-native`, `prom-ui-iced-adapter`, `prom-ui-demo` | N6 (retired) | yes |
| `examples/workbench_semantic` | N6 (retired) | **no** (`exclude` in `Cargo.toml`) |

## 6. Coverage matrix

| Group | Artifacts | Owner | Execution evidence | Public claim | Audit status |
|---|---:|---|---|---|---|
| N1 Canonical/qualification | 229 `.sm` | qualification contour | 224 CI_REFERENCED, 5 unresolved | QUALIFIED_LIMITED (canonical 17) | INVENTORIED — NOT AUDITED |
| N2 Examples/demos | 7 `.sm` | unresolved | 3 CI_REFERENCED, 4 non-CI | ILLUSTRATIVE / UNKNOWN | INVENTORIED — NOT AUDITED |
| N3 Product | 0 | — | — | none | INVENTORIED — EMPTY |
| N4 Bootstrap | 0 (`#1910` direction; external docs-only repo) | #1910 | none in CI | EXPERIMENTAL direction, "not a release promise" (README) | INVENTORIED — NOT AUDITED |
| N5 Support/composition crates | 10 crates | respective crates | workspace tests in CI | UNKNOWN | INVENTORIED — NOT AUDITED |
| N6 UI (retired) | 3 `.sm` + 5 crates + `examples/workbench_semantic` | retirement decision | 3 CI_REFERENCED (reference only) | HISTORICAL | INVENTORIED — NOT AUDITED |
| N7 Legacy | 1 `.sm` + `ton618_legacy/**` | legacy perimeter | 1 UNKNOWN | UNKNOWN | INVENTORIED — NOT AUDITED |
| N8 Test fixtures | 310 `.sm` | owning platform tests | 310 CI_REFERENCED | INTERNAL_CONTRACT | INVENTORIED — NOT AUDITED |

## 7. Reproduction query

Save the block below to a file outside the repository (for example
`$TMP/inventory_query.py`) and run it from the repository root at the audit SHA
with Python 3. Without arguments it prints the counts; with `--tsv` it prints
the content of `inventory.tsv` (normalize to LF). The query is recorded here as
documentation; it is not installed as tooling.

```python
"""Deterministic native .sm inventory for NATIVE-AUDIT-00 (run from repo root)."""
import subprocess, collections, sys

def ls(*pats):
    out = subprocess.run(["git", "ls-files", "-z", "--", *pats], capture_output=True, check=True).stdout
    return [p for p in out.decode().split("\0") if p]

def read(p):
    try:
        return open(p, encoding="utf-8", errors="replace").read()
    except OSError:
        return ""

SM = sorted(ls("*.sm"))
# Cargo workspace members (Cargo.toml) whose all-targets tests ci.yml runs via
# `cargo test --workspace --all-targets`; examples/workbench_semantic is excluded.
WORKSPACE = ("src/", "tests/", "benches/", "build.rs", "crates/", "examples/quad_logic_calculator/")
ALL_RS = {p: read(p) for p in ls("*.rs")}
RS = {p: t for p, t in ALL_RS.items() if p.startswith(WORKSPACE)}
OTHER_RS = {p: t for p, t in ALL_RS.items() if p not in RS}
WF = {p: read(p) for p in ls(".github/workflows/*.yml")}
SCRIPTS = {p: read(p) for p in ls("scripts/*")}
CI = dict(WF)
CI.update({p: t for p, t in SCRIPTS.items() if any(p.rsplit("/", 1)[-1] in w for w in WF.values())})
MANUAL = dict(OTHER_RS)
MANUAL.update({p: t for p, t in SCRIPTS.items() if p not in CI})
MD = {p: read(p) for p in ls("*.md")}
TOO_BROAD = {"examples", "tests", "tests/fixtures", "crates", "crates/sm-vm", "crates/sm-vm/tests"}

def group_role_claim(p):
    if p.startswith("examples/canonical/"): return "N1", "CANONICAL", "QUALIFIED_LIMITED"
    if p.startswith("examples/qualification/"): return "N1", "QUALIFICATION", "INTERNAL_CONTRACT"
    if p.startswith("examples/readiness_draft_canonical/"): return "N1", "READINESS_DRAFT", "UNKNOWN"
    if p.startswith("examples/pcc_candidates/"): return "N1", "READINESS_DRAFT", "EXPERIMENTAL"
    if p.startswith("examples/benchmarks/"): return "N2", "DEMO", "ILLUSTRATIVE"
    if p.startswith("examples/quad_logic_calculator/") or p.startswith("examples/") and p.count("/") == 1:
        return "N2", "EXAMPLE", "UNKNOWN"
    if p.startswith(("examples/workbench_semantic/", "artifacts/workbench/")): return "N6", "LEGACY", "HISTORICAL"
    if p.startswith("assets/legacy_cli/"): return "N7", "LEGACY", "UNKNOWN"
    if p.startswith(("tests/", "crates/")): return "N8", "TEST_FIXTURE", "INTERNAL_CONTRACT"
    return "UNASSIGNED", "UNKNOWN", "UNKNOWN"

def anchors(p):
    parts = p.split("/")
    yield p
    if len(parts) >= 2:
        yield "/".join(parts[-2:])
    for i in range(len(parts) - 1, 1, -1):
        d = "/".join(parts[:i])
        if d not in TOO_BROAD:
            yield d

def evidence(p):
    keys = list(anchors(p))
    rs = sorted(f for f, t in RS.items() if any(k in t for k in keys))
    ci = sorted(f for f, t in CI.items() if any(k in t for k in keys))
    if rs or ci:
        return "CI_REFERENCED", rs + ci
    man = sorted(f for f, t in MANUAL.items() if any(k in t for k in keys))
    if man:
        return "MANUAL_ONLY", man
    stem = p.rsplit("/", 1)[-1][:-3]
    if any(stem in t for t in RS.values()):
        return "UNKNOWN", ["stem-only match in *.rs"]
    if any(k in t for t in MD.values() for k in keys):
        return "INSPECTION_ONLY", []
    return "NOT_EXECUTED", []

rows = []
for p in SM:
    g, r, c = group_role_claim(p)
    e, refs = evidence(p)
    ign = any("#[ignore" in RS.get(f, "") for f in refs)
    rows.append((p, g, r, c, e, "yes" if ign else "no", ";".join(refs[:3]) + (f";+{len(refs)-3}" if len(refs) > 3 else "")))

if "--tsv" in sys.argv:
    print("path\tgroup\trole\tclaim_level\texecution_evidence\tref_file_has_ignore\treferenced_by")
    for row in rows: print("\t".join(row))
else:
    print("total", len(rows))
    for i, name in [(1, "group"), (2, "role"), (4, "evidence"), (3, "claim"), (5, "ignore_in_ref")]:
        print(name, dict(sorted(collections.Counter(r[i] for r in rows).items())))
    print("group x evidence", dict(sorted(collections.Counter((r[1], r[4]) for r in rows).items())))
    print("dir", dict(sorted(collections.Counter("/".join(r[0].split("/")[:2]) for r in rows).items())))
    for r in rows:
        if r[4] != "CI_REFERENCED": print("NONCI", r[4], r[0])
```

## 8. Candidates observed — NOT ADJUDICATED

Recorded only so Slice 1 starts from them. None is a finding; none is rated or
repaired.

- **CANDIDATE — NOT ADJUDICATED (C-01).** Retired Workbench sources
  `examples/workbench_semantic/src/*.sm` are `CI_REFERENCED` only through a doc
  comment in `crates/prom-ui-iced-adapter/src/lib.rs`; the retired `prom-ui*`
  crates remain Cargo workspace members whose tests `ci.yml` runs. Question:
  does CI signal from a retired contour read as current readiness?
- **CANDIDATE — NOT ADJUDICATED (C-02).** `artifacts/workbench/native-launch-smoke/smoke-project/main.sm`
  is referenced only by `scripts/check_repository_hygiene.py` (a path policy
  check) and the non-CI `scripts/workbench_native_launch_smoke.ps1`.
- **CANDIDATE — NOT ADJUDICATED (C-03).** `examples/quad_logic_calculator` is a
  workspace member, but its `.sm` sources are named only by the tracked,
  non-workspace `proj_test/src/main.rs`.
- **CANDIDATE — NOT ADJUDICATED (C-04).** `examples/readiness_draft_canonical/README.md`
  places some packs in the `qualified limited release` contour, while five of
  its module sources have no CI reference (§4).
- **CANDIDATE — NOT ADJUDICATED (C-05).** Bootstrap/self-hosting is stated as
  "the active strategic direction" in README, backlog, WBS and the feature
  maturity matrix (README adds "not a release promise"), with zero Semantic
  compiler source in either repository.
- **CANDIDATE — NOT ADJUDICATED (C-06).** 37 `crates/sm-vm` profiling fixtures
  are referenced only from `crates/sm-vm/tests/vm_opcode_profile_workloads.rs`,
  which has 24 `#[test]` and 1 `#[ignore]`; whether any fixture is reachable
  only through the ignored test is unknown.
