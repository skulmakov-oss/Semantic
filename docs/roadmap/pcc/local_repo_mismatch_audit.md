# Local Repo Mismatch Audit

## Status

Overall result: `FAIL`

## Executive summary

The local repository at `C:\Users\said3\Desktop\EXOcode\Semantic_phase1_prom_ui`
does not contain the Practical Core stack that was described as having been
merged in PR `#1301`.

The repository is internally healthy on its current baseline:

- `main == origin/main`
- `HEAD = cbb54af2518943950d3be5d0ed66520a762d1a34`
- the current `7hell` runner passes

But the requested PCC/CTF closeout files, fixtures, harnesses, and sync pack are
absent here. That means the prior claim is not supported by this local repo.

## Git state

- branch: `main`
- HEAD: `cbb54af2518943950d3be5d0ed66520a762d1a34`
- main == origin/main: `yes`
- safety branch: `not present in this repo`
- dirty tree: `docs/language/semantic_sugar_track_rfc.md` is untracked
- holdbacks: the expected holdback paths from the other workspace are not
  present as tracked files in this repo snapshot

## Evidence matrix

| Area | Result | Evidence | Notes |
|---|---:|---|---|
| PR #1301 merge evidence | `FAIL` | `git cat-file -t 736b8bb066ea68e7e6d2e79ff300f77117c51561` | The merge SHA from the other workspace does not exist in this repository. |
| Practical Core closeouts | `FAIL` | `rg --files docs/roadmap/pcc` | The requested `control_flow_core_closeout.md`, `text_core_closeout.md`, `collections_core_closeout.md`, `stdlib_v0_closeout.md`, and `practical_core_phase_checkpoint.md` are absent. |
| Canonical examples | `FAIL` | `examples/canonical/README.md`, `docs/examples_index.md`, `tests/canonical_examples.rs`, `tests/cli_public_smoke_matrix.rs` | This repo’s canonical pack is the older 5-example pack, not the requested PCC stack. |
| Negative diagnostics fixtures | `FAIL` | `tests/fixtures/pcc` missing | No requested PCC fixture corpus exists in this repo snapshot. |
| Negative harnesses | `FAIL` | `cargo test -q --test pcc_control_flow_negative` and peers | The requested harness targets do not exist here. |
| 7hell wiring | `PASS-WITH-WARNINGS` | `tools/7hell/run.ps1`, `tools/7hell/run.sh`, `tools/7hell/README.md`, `docs/roadmap/pcc/7hell_mini_runner.md` | The local runner passes, but it is the older 7hell contour, not the requested PCC negative-diagnostics wiring. |
| Full 7hell run | `PASS` | `powershell -ExecutionPolicy Bypass -File .\\tools\\7hell\\run.ps1` | Full local run completed successfully with exit code `0`. |
| PCC / CTF sync | `FAIL` | `docs/roadmap/pcc/pcc_ctf_sync_checkpoint.md`, `docs/roadmap/pcc/pcc_ctf_sync_closeout.md`, `docs/roadmap/language_maturity/core_trust_freeze/ctf_sync_closeout.md` missing | The requested sync pack is absent from this repo. |
| CTF issue pack | `FAIL` | `docs/roadmap/issues/issue_ctf_sync_*` missing | The requested execution-handle files are absent here. |
| Linguist readiness | `FAIL` | `docs/roadmap/issues/issue_linguist_*` missing | The requested Linguist readiness templates are absent here. |
| Post-UI docs separation | `UNKNOWN / NOT VERIFIED` | `docs/roadmap/post_ui/` not present in this repo snapshot | The requested post-UI docs were not found in this local repo, so separation could not be checked here. |
| Holdbacks excluded | `PASS-WITH-WARNINGS` | `git status --short`, `git ls-files .codex .codex-remote-attachments .env.example docs/dev semantic-textmate-grammar` | The expected holdbacks from the other workspace are not tracked here, but this repo also has an unexpected untracked file: `docs/language/semantic_sugar_track_rfc.md`. |

## Validation results

- `git status --short --branch`
  - result: `PASS`
  - note: `main...origin/main`
- `git log --oneline -n 12`
  - result: `PASS`
  - note: history is the older `cbb54af` baseline, not the requested `736b8bb0` stack
- `git rev-parse HEAD`
  - result: `PASS`
  - value: `cbb54af2518943950d3be5d0ed66520a762d1a34`
- `git rev-parse main`
  - result: `PASS`
  - value: `cbb54af2518943950d3be5d0ed66520a762d1a34`
- `git rev-parse origin/main`
  - result: `PASS`
  - value: `cbb54af2518943950d3be5d0ed66520a762d1a34`
- `git branch --show-current`
  - result: `PASS`
  - value: `main`
- `git ls-files --others --exclude-standard`
  - result: `PASS`
  - note: only `docs/language/semantic_sugar_track_rfc.md`
- `git ls-files .codex .codex-remote-attachments .env.example docs/dev semantic-textmate-grammar`
  - result: `PASS`
  - note: none of the expected holdbacks are tracked here
- `cargo test -q --test canonical_examples`
  - result: `PASS`
- `cargo test -q --test cli_public_smoke_matrix`
  - result: `PASS`
- `cargo test -q --test pcc_control_flow_negative`
  - result: `FAIL`
  - note: no such test target exists in this repo
- `cargo test -q --test pcc_text_negative`
  - result: `FAIL`
  - note: no such test target exists in this repo
- `cargo test -q --test pcc_collections_negative`
  - result: `FAIL`
  - note: no such test target exists in this repo
- `cargo test -q --test pcc_stdlib_negative`
  - result: `FAIL`
  - note: no such test target exists in this repo
- `powershell -ExecutionPolicy Bypass -File .\\tools\\7hell\\run.ps1`
  - result: `PASS`
  - note: full run completed successfully on this baseline

## Claims checked

- checked Practical Core contour:
  - `FAIL`
  - reason: the requested Practical Core closeout artifacts are not present in this local repo
- closed canonical examples:
  - `FAIL`
  - reason: the requested canonical examples are absent
- negative diagnostics:
  - `FAIL`
  - reason: the requested negative fixture corpora and harnesses are absent
- 7hell qualification:
  - `PASS-WITH-WARNINGS`
  - reason: the runner passes, but not the requested PCC negative-diagnostics wiring
- CTF sync with follow-ups:
  - `FAIL`
  - reason: the requested sync pack is absent

## Unsupported or overclaimed statements

- `PR #1301 was merged in this repository` is not supported here.
- `HEAD = 736b8bb0` is false for this repository.
- `backup/main-before-origin-sync` is not present here.
- `Semantic has a verified Practical Core contour` is not supported by this repo snapshot.
- `closed canonical examples` for the requested PCC stack are not supported.
- `negative diagnostics` for the requested PCC stack are not supported.
- `PCC / CTF sync with follow-ups` in the requested form is not supported.

## Remaining risks

- The local repo contains one unexpected untracked docs file:
  - `docs/language/semantic_sugar_track_rfc.md`
- The local repository baseline differs materially from the workspace that
  contained the merged PCC/CTF stack.
- The current repo state should not be used to infer the existence of the
  requested PCC closeout stack.

## Final verdict

`FAIL`

The requested Practical Core contour is not present in this local repository,
so the claim is not verified here. The repo’s existing baseline is green on its
own terms, but that is a different statement from the one being audited.
