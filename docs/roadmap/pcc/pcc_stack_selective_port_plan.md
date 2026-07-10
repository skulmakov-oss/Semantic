# PCC Stack Selective Port Plan

## Status

Result: `PASS-WITH-WARNINGS`

This is an audit-only selective port plan.

No files were ported.
No cherry-pick was performed.
No merge was performed.
No implementation code was changed.
No tests/examples/7hell files were changed.

## Purpose

Explain that the current source-of-truth repo `Semantic_phase1_prom_ui` is
valid on its own baseline, but does not contain the external PCC Practical Core
stack from PR `#1301`.

The purpose of this plan is to decide how to treat each layer of PR `#1301`
before any future transfer.

## Current source-of-truth repo state

- path: `C:\Users\said3\Desktop\EXOcode\Semantic_phase1_prom_ui`
- branch: `main`
- HEAD: `cbb54af2518943950d3be5d0ed66520a762d1a34`
- main == origin/main: `yes`
- baseline 7hell: `PASS`
- dirty tree: untracked local files are present
- unexpected untracked files:
  - `docs/language/semantic_sugar_track_rfc.md`
  - `docs/roadmap/pcc/local_practical_core_readiness_audit.md`
  - `docs/roadmap/pcc/local_repo_mismatch_audit.md`
  - `docs/roadmap/pcc/pcc_stack_bridge_audit.md`
- safety branch: not present in this repo

## Existing audit references

These local audit references exist and support the mismatch finding:

- `docs/roadmap/pcc/local_practical_core_readiness_audit.md`
- `docs/roadmap/pcc/local_repo_mismatch_audit.md`
- `docs/roadmap/pcc/pcc_stack_bridge_audit.md`

## External PCC stack reference

- PR: `#1301`
- GitHub repo: `skulmakov-oss/Semantic`
- merge SHA: `736b8bb066ea68e7e6d2e79ff300f77117c51561`
- head SHA: `987857f961a2e17141490925f135849a0f6f7ef8`
- changed files: `119`
- commits: `7`
- local availability of merge SHA: `absent`

Claimed external stack:

- PCC Practical Core closeouts
- canonical examples
- negative diagnostics fixtures
- negative harnesses
- 7hell wiring
- PCC / CTF sync pack
- CTF follow-up issue bodies
- Linguist readiness templates
- post-UI docs
- PR 1185 7hell platform contour audit

Important:

- the PR `#1301` stack is external to this local repo line until proven
  compatible
- the merge SHA is not present in the current repo history
- the current repo baseline is valid, but the claimed stack is absent here

## Porting principle

Do not port the external PR as a whole.

Each layer must be classified separately as one of:

- `PORT-CANDIDATE`
- `REBUILD-NATIVELY`
- `EXTERNAL-REFERENCE-ONLY`
- `REJECT / NOT APPLICABLE`
- `NEEDS-COMPATIBILITY-AUDIT`

## Layer classification matrix

| Layer | External source | Current local state | Compatibility risk | Proposed decision | Notes |
|---|---|---|---|---|---|
| PCC closeout docs | PR `#1301` | absent | medium/high | `NEEDS-COMPATIBILITY-AUDIT` | Must match the current repo surface before any transfer. |
| Canonical examples | PR `#1301` | older 5-example pack plus boundary example | high | `REBUILD-NATIVELY` or `PORT-CANDIDATE` after probe | Must pass current `smc check` and match local admitted surface. |
| PCC candidate probes | PR `#1301` | absent | medium | `PORT-CANDIDATE` after path audit | Candidate probes must not be presented as canonical. |
| Negative fixtures | PR `#1301` | absent | high | `NEEDS-COMPATIBILITY-AUDIT` | Diagnostics may differ in this repo line. |
| Negative harnesses | PR `#1301` | absent | high | `REBUILD-NATIVELY` after fixtures pass | Must use the current local test style. |
| 7hell wiring | PR `#1301` | baseline 7hell exists | high | `PORT-CANDIDATE` last | Only after tests and harnesses exist locally. |
| PCC / CTF sync docs | PR `#1301` | absent | high | `REBUILD AFTER PCC STACK EXISTS` | Cannot claim sync before the stack exists locally. |
| CTF issue bodies | PR `#1301` | absent | medium | `EXTERNAL-REFERENCE-ONLY` or `PORT-CANDIDATE` later | Only after CTF sync is valid locally. |
| Linguist readiness templates | PR `#1301` | absent | low/medium | `PORT-CANDIDATE` | Independent from PCC, but must not claim submit-readiness. |
| Post-UI docs | PR `#1301` | absent locally | medium | `EXTERNAL-REFERENCE-ONLY` / separate track | Must not mix with PCC transfer. |
| PR 1185 platform audit | PR `#1301` | absent locally | medium | `EXTERNAL-REFERENCE-ONLY` or `PORT-CANDIDATE` | Needs a relevance check before any transfer. |

## Required compatibility gates

Before any future port, require these gates.

### Gate 1: Source alignment

- confirm current repo remote;
- confirm current branch;
- confirm current baseline tests;
- confirm current syntax/admitted surface.

### Gate 2: Per-layer diff audit

For each candidate layer:

- list exact files;
- state why it belongs to the current repo;
- check whether an equivalent local file already exists;
- determine whether the port is additive, conflicting, or obsolete.

### Gate 3: Example compatibility

For any `.sm` example:

- run `cargo run --bin smc -- check <file>`;
- do not mark canonical until it passes;
- do not add it to the smoke matrix until canonical status is justified.

### Gate 4: Negative diagnostics compatibility

For each negative fixture:

- run `smc check`;
- record the actual local marker;
- do not reuse old markers blindly;
- build a harness only after the marker audit.

### Gate 5: 7hell compatibility

Only wire into `tools/7hell` after:

- examples pass;
- negative harnesses pass;
- the local runner structure is inspected;
- no new runner architecture is introduced.

### Gate 6: CTF sync validity

Only create PCC / CTF sync docs after:

- the PCC stack exists locally;
- canonical examples pass;
- negative harnesses pass;
- 7hell qualification is local;
- trust wording matches actual local evidence.

## Proposed port order

Recommended order:

1. Linguist readiness templates, if independent and still useful.
2. PCC candidate probes as an external audit trail.
3. Canonical examples as local probes first, not canonical.
4. Negative fixture corpus after a local diagnostics probe.
5. Negative harnesses after markers are confirmed.
6. Canonical promotion and smoke matrix wiring.
7. 7hell wiring.
8. PCC closeouts.
9. PCC / CTF sync docs.
10. CTF issue bodies.
11. Final local readiness audit.

## Explicit non-goals

This plan does not:

- import PR `#1301`;
- claim PCC readiness;
- claim CTF sync;
- claim 7hell qualification for the missing stack;
- change current repo behavior;
- alter VM / verifier / SemCode / capability behavior;
- introduce new language surface.

## Risk register

| Risk | Severity | Mitigation |
|---|---:|---|
| Blindly porting stale files | high | Require per-file compatibility audit. |
| Reusing diagnostics markers from another repo line | high | Re-probe markers locally. |
| Claiming CTF sync before the PCC stack exists | high | CTF sync only after local qualification. |
| Mixing post-UI with PCC transfer | medium | Keep it as a separate track. |
| Treating candidates as canonical | medium | Require promotion criteria. |
| Overclaiming Linguist readiness | medium | Preserve not-submit-ready wording. |

## Recommended next step

Recommended next action:

`PCC-PORT-0`: create a per-file inventory of PR `#1301` layers as an external
reference, without copying files.

Do not cherry-pick.

Do not port files until the per-file inventory and compatibility gates are
approved.

## Final verdict

Result: `PASS-WITH-WARNINGS`

Why:

- the current repo is valid and baseline-healthy;
- the external PCC Practical Core stack is absent locally;
- the selective port plan can be written safely as an audit-only document;
- a full transfer is not approved;
- the next step is a per-file external inventory, not a port.
