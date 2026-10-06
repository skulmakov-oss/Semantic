# Semantic-native Readiness Audit — Inventory

Status: N2 and N7 AUDITED (NATIVE-AUDIT-02); other groups INVENTORIED — NOT AUDITED; current evidence reflects NATIVE-AUDIT-01 and NATIVE-AUDIT-02 adjudication
Slice 0 audit base SHA: `d95bee488d26208f4a3a4215ae4dd34e04d429ee`
Charter: [`charter.md`](charter.md)

## 0. Authority model

**Current authoritative per-file evidence** = Slice-0 raw row in
[`inventory.tsv`](inventory.tsv) **+** the adjudicated override for that path in
[`inventory_overrides.tsv`](inventory_overrides.tsv), when one exists.

- [`inventory.tsv`](inventory.tsv) is the **Slice-0 raw heuristic snapshot —
  historical derivation, superseded where adjudicated**. It is immutable and
  reproducible by the query in §7.1. On its own it is never the current
  per-file evidence.
- [`inventory_overrides.tsv`](inventory_overrides.tsv) holds exactly the paths
  whose evidence a later slice proved (56 rows: 50 from NATIVE-AUDIT-01, 6 from
  NATIVE-AUDIT-02). Each row
  records `slice0_evidence`, `current_evidence`, `proven_level` and
  `adjudication_source`. A path appears at most once.
- Composition rule: for each of the 550 raw rows,
  `current_evidence = override.current_evidence` if the path is in the
  override table, else `raw.execution_evidence`. Group, role and claim level
  always come from the raw row. The composition query is §7.2.
- Rows not in the override table keep their Slice-0 value unchanged; they are
  not reclassified by proximity.

Raw values are machine-derived from `git ls-files` at the Slice 0 SHA. Unless a
row's override says otherwise, its execution evidence records a **reference**,
never proven execution depth (charter §5, §8).

## 1. Totals

`git ls-files '*.sm' | wc -l` → **550** tracked `.sm` files.

| Group | Files | Role / claim level |
|---|---:|---|
| N1 | 229 | CANONICAL / QUALIFIED_LIMITED 17; QUALIFICATION / INTERNAL_CONTRACT 199; READINESS_DRAFT / UNKNOWN 11; READINESS_DRAFT / EXPERIMENTAL 2 |
| N2 | 7 | DEMO / ILLUSTRATIVE 2; EXAMPLE / UNKNOWN 5 |
| N3 | 0 | — |
| N4 | 0 | — (external repo `Semantic-Language`: 0 `.sm`) |
| N5 | 0 `.sm` | 11 crates (see §5) |
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

## 2. Execution evidence (current)

Current authoritative totals (raw + overrides, §0):

| Evidence | Files |
|---|---:|
| CI_EXECUTED | 9 |
| CI_REFERENCED | 494 |
| MANUAL_ONLY | 31 |
| INSPECTION_ONLY | 12 |
| NOT_EXECUTED | 4 |
| UNKNOWN | 0 |
| **Total** | **550** |

| Group | CI_EXECUTED | CI_REFERENCED | MANUAL_ONLY | INSPECTION_ONLY | NOT_EXECUTED | UNKNOWN |
|---|---:|---:|---:|---:|---:|---:|
| N1 | 0 | 216 | 0 | 10 | 3 | 0 |
| N2 | 3 | 2 | 0 | 1 | 1 | 0 |
| N6 | 0 | 3 | 0 | 0 | 0 | 0 |
| N7 | 0 | 0 | 0 | 1 | 0 | 0 |
| N8 | 6 | 273 | 31 | 0 | 0 | 0 |

The 494 `CI_REFERENCED` rows are unadjudicated except the 2 `examples/quad_logic_calculator`
rows (C-03, build-time embed): E0 plus a CI-run referrer.

### 2.1 Historical: Slice-0 raw heuristic totals (superseded where adjudicated)

| Evidence | Files |
|---|---:|
| CI_REFERENCED | 540 |
| MANUAL_ONLY | 2 |
| INSPECTION_ONLY | 2 |
| UNKNOWN | 6 |
| NOT_EXECUTED | 0 |

### 2.2 Slice-0 heuristic derivation rules (historical, see §7.1)

Known defects of these rules (M-01 over-broad `parent/name.sm` suffix, M-02
feature-gated referrers counted as CI, M-03 bare-name `include_str!` missed) are
recorded in `slice_01_candidate_adjudication.md`; they are corrected only
through adjudicated overrides, not by redesigning the rules.

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

## 4. Current non-CI and unresolved entries

Current `UNKNOWN`: **0**. The three Slice-0/Slice-1 UNKNOWN rows were resolved
in NATIVE-AUDIT-02 (`slice_02_n2_n7_audit.md`, UNKNOWN resolution):
`assets/legacy_cli/human.sm` → INSPECTION_ONLY, `examples/calculator.sm` →
INSPECTION_ONLY, `examples/semantic_policy_overdrive.sm` → NOT_EXECUTED.

Current non-CI entries in N2 (2): `examples/calculator.sm` (INSPECTION_ONLY)
and `examples/semantic_policy_overdrive.sm` (NOT_EXECUTED); neither carries a
current claim. Current non-CI entry in N7 (1): `assets/legacy_cli/human.sm`
(INSPECTION_ONLY).

Current resolved non-CI entries in N1 (13): 10 `INSPECTION_ONLY` — the 6
`examples/readiness_draft_canonical/*/src/main.sm`, the 2
`examples/pcc_candidates/*/src/main.sm`, and
`module_selected_import_audit_report/src/{risk_policy,text_format}.sm`; 3
`NOT_EXECUTED` — `wave2_local_helper_import/src/helper.sm` and
`module_selected_import_settlement/src/{rendering,rules}.sm` (see
`inventory_overrides.tsv` and C-04).

Current non-CI entries in N8 (31 `MANUAL_ONLY`): the `vm-profile`-gated
`crates/sm-vm/tests/fixtures/profiling/**` fixtures listed in
`inventory_overrides.tsv` (C-06).

Ownership: N2 and N7 owners resolved in NATIVE-AUDIT-02 (`examples/calculator.sm`
and `examples/semantic_policy_overdrive.sm` have no current owner and no
current claim, recorded explicitly; `assets/legacy_cli/human.sm` is N7).
`examples/quad_logic_calculator` ownership was
resolved to the retired N6 UI contour (C-03, DEFER-SCOPE); its raw group label
in `inventory.tsv` is unchanged.

### 4.1 Historical: Slice-0 non-CI-referenced and unresolved entries (10)

Slice 0 listed 10 entries: the 3 current UNKNOWN above,
`examples/quad_logic_calculator/src/{calculator,quad_calc.proj}.sm` (MANUAL_ONLY
via `proj_test`), `module_selected_import_audit_report/src/{risk_policy,text_format}.sm`
(INSPECTION_ONLY), and `module_selected_import_settlement/src/{rendering,rules}.sm`
plus `wave2_local_helper_import/src/helper.sm` (UNKNOWN). Superseded by the
current state above.

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
| N1 Canonical/qualification | 229 `.sm` | qualification contour | 216 CI_REFERENCED, 10 INSPECTION_ONLY, 3 NOT_EXECUTED | QUALIFIED_LIMITED (canonical 17) | INVENTORIED — NOT AUDITED |
| N2 Examples/demos | 7 `.sm` | resolved (calculator → N6, C-03; 2 unowned/unclaimed) | 3 CI_EXECUTED, 2 CI_REFERENCED, 1 INSPECTION_ONLY, 1 NOT_EXECUTED | QUALIFIED_LIMITED (snake_core, overdrive_trace) / ILLUSTRATIVE / HISTORICAL | **AUDITED** (NATIVE-AUDIT-02; NA-N2-001 open) |
| N3 Product | 0 | — | — | none | INVENTORIED — EMPTY |
| N4 Bootstrap | 0 (`#1910` direction; external docs-only repo) | #1910 | none in CI | EXPERIMENTAL direction, "not a release promise" (README) | INVENTORIED — NOT AUDITED |
| N5 Support/composition crates | 11 crates | respective crates | workspace tests in CI | UNKNOWN | INVENTORIED — NOT AUDITED |
| N6 UI (retired) | 3 `.sm` + 5 crates + `examples/workbench_semantic` | retirement decision | 3 CI_REFERENCED (reference only) | HISTORICAL | INVENTORIED — NOT AUDITED |
| N7 Legacy | 1 `.sm` + `ton618_legacy/**` | legacy perimeter | 1 INSPECTION_ONLY | HISTORICAL | **AUDITED** (NATIVE-AUDIT-02; 0 findings) |
| N8 Test fixtures | 310 `.sm` | owning platform tests | 6 CI_EXECUTED, 273 CI_REFERENCED, 31 MANUAL_ONLY | INTERNAL_CONTRACT | INVENTORIED — NOT AUDITED |

## 7. Reproduction queries

### 7.1 Slice-0 raw heuristic query (historical derivation)

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

### 7.2 Current-state composition and validation query

Run from the repository root (Python 3). It composes the raw snapshot with the
override table under the §0 rule and asserts the invariants.

```python
import collections
D = "docs/roadmap/native_audit/"
raw = [l.split("\t") for l in open(D + "inventory.tsv").read().splitlines()[1:]]
ovr = [l.split("\t") for l in open(D + "inventory_overrides.tsv").read().splitlines()[1:]]
paths = [r[0] for r in raw]
assert len(raw) == 550 and len(set(paths)) == 550                  # one raw row per path
assert len({o[0] for o in ovr}) == len(ovr)                        # one override per path
raw_ev = {r[0]: r[4] for r in raw}
assert all(o[0] in raw_ev and o[1] == raw_ev[o[0]] for o in ovr)   # overrides cite the true raw value
cur = dict(raw_ev)
cur.update({o[0]: o[2] for o in ovr})
print(len(ovr), "overrides;", dict(sorted(collections.Counter(cur.values()).items())), "total", len(cur))
```

Expected output at this revision: `56 overrides; {'CI_EXECUTED': 9,
'CI_REFERENCED': 494, 'INSPECTION_ONLY': 12, 'MANUAL_ONLY': 31, 'NOT_EXECUTED': 4}
total 550` (no `UNKNOWN` key: zero rows).

## 8. Candidates observed in Slice 0 — outcomes

C-01 … C-06 were recorded in Slice 0 as `CANDIDATE — NOT ADJUDICATED`. They
were adjudicated in NATIVE-AUDIT-01; full evidence is in
[`slice_01_candidate_adjudication.md`](slice_01_candidate_adjudication.md).

| Candidate (Slice 0 origin) | Outcome |
|---|---|
| C-01 retired Workbench `.sm` / `prom-ui*` in workspace CI | KEEP — retirement decision scopes crate CI to repository integrity |
| C-02 Workbench native-launch smoke `.sm` | DISPROVED — path allowlist + historical manual script (#1862) |
| C-03 `quad_logic_calculator` `.sm` | DEFER-SCOPE to retired UI contour — embedded at build time only, 0 tests, no current claim |
| C-04 readiness_draft qualified-limited packs | DISPROVED — qualified packs are byte-identical to CI-executed qualification copies (E5–E7, E9); other packs are declared out of scope |
| C-05 self-hosting direction | DISPROVED — every current statement says "not a release promise/status" |
| C-06 `sm-vm` profiling fixtures / `#[ignore]` | KEEP — documented `vm-profile` local measurement; exactly 1 fixture is ignored-only |

Confirmed findings: 0 (no `NA-*` IDs assigned).

## 9. Slice 1 adjudicated overrides (summary)

The authoritative per-path data is [`inventory_overrides.tsv`](inventory_overrides.tsv)
(50 rows, `adjudication_source` = `NATIVE-AUDIT-01 / C-xx / M-xx`); this table
summarizes it. Traces are in `slice_01_candidate_adjudication.md`.

| Rows | Slice 0 evidence | Adjudicated evidence | Proven level | Source |
|---:|---|---|---|---|
| 6 `crates/sm-vm/tests/fixtures/profiling/**/scalar_helper_boundary_*` (helper/inline, single-call, call-chain) | CI_REFERENCED | **CI_EXECUTED** | E7 + E9 | C-06 |
| 31 other `crates/sm-vm/tests/fixtures/profiling/**` | CI_REFERENCED | MANUAL_ONLY | E7 local (`--features vm-profile`; 1 also `--ignored`) | C-06, M-02 |
| 6 `examples/readiness_draft_canonical/*/src/main.sm` | CI_REFERENCED | INSPECTION_ONLY | E0 (byte-identical content E7+E9 for 4 packs) | C-04, M-01 |
| 2 `examples/pcc_candidates/*/src/main.sm` | CI_REFERENCED | INSPECTION_ONLY | E0 | M-01 |
| 2 `examples/quad_logic_calculator/src/{calculator,quad_calc.proj}.sm` | MANUAL_ONLY | CI_REFERENCED (build-time `include_str!`) | E0 | C-03, M-03 |
| `examples/readiness_draft_canonical/wave2_local_helper_import/src/helper.sm` | UNKNOWN | NOT_EXECUTED (byte-identical content executed) | E0 (content E7+E9) | C-04 |
| `examples/readiness_draft_canonical/module_selected_import_settlement/src/{rendering,rules}.sm` | UNKNOWN | NOT_EXECUTED (pack declared out of scope) | E0 | C-04 |

Current totals are stated once, in §2.

## 10. Slice 2 adjudicated overrides (summary)

Six rows from NATIVE-AUDIT-02 (`adjudication_source` = `NATIVE-AUDIT-02 / N2|N7`);
traces in [`slice_02_n2_n7_audit.md`](slice_02_n2_n7_audit.md). No Slice 1
override was changed.

| Path | Slice 0 evidence | Adjudicated evidence | Proven level |
|---|---|---|---|
| `assets/legacy_cli/human.sm` | UNKNOWN | INSPECTION_ONLY | E0 (same program E7+E9 via legacy CLI unit test) |
| `examples/calculator.sm` | UNKNOWN | INSPECTION_ONLY | E0 (identical content E4+E9 via `golden_v1_calculator`) |
| `examples/semantic_policy_overdrive.sm` | UNKNOWN | NOT_EXECUTED | E0 (local E6 only) |
| `examples/benchmarks/snake_core.sm` | CI_REFERENCED | CI_EXECUTED | E7+E9 |
| `examples/benchmarks/snake_learning.sm` | CI_REFERENCED | CI_EXECUTED | E7+E9 (explicit envelope) |
| `examples/semantic_policy_overdrive_trace.sm` | CI_REFERENCED | CI_EXECUTED | E7+E9+E10 |

Findings registry: [`findings.md`](findings.md) (NA-N2-001, P3, DEFER-SCOPE).
