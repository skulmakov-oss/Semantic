# Local Holdbacks Cleanup Plan

## Status

Result: PASS-WITH-WARNINGS

This is an audit-only cleanup plan.

No files were deleted.
No files were moved.
No files were staged.
No files were committed.
No git clean was run.
No code/tests/examples/7hell files were changed.

## Purpose

Classify remaining local holdbacks after the `#1302` / `#1303` / `#1304`
merge cycle.

## Source repo state

- branch: `codex/pcc-bridge-port-audit-trail`
- HEAD: `7b838f9e8c7035ffc317c0aec3296104033888a6`
- origin/main: `a708a4ab36d86adadfa226dc07bd5dc79287326e`
- main == origin/main: `yes`
- dirty tree summary: 9 untracked audit docs remain
- untracked file count: `9`

## Holdback inventory

| Path | Exists | Tracked state | Size / count summary | Classification | Reason |
|---|---:|---|---|---|---|
| `.codex/` | no | absent | not present in this checkout | ABSENT | The path is not present locally, so there is nothing to clean up here. |
| `.codex-remote-attachments/` | no | absent | not present in this checkout | ABSENT | The path is not present locally, so there is nothing to clean up here. |
| `.env.example` | no | absent | not present in this checkout | ABSENT | The path is not present locally, so no secret/template review is possible here. |
| `docs/dev/` | no | absent | not present in this checkout | ABSENT | The path is not present locally, so there is no docs/dev artifact to classify. |
| `semantic-textmate-grammar/` | no | absent | not present in this checkout; nested repo not detected | ABSENT | The path is not present locally, so it cannot be treated as an in-repo nested grammar checkout. |

## Detailed findings

### `.codex/`

- observed: not present in this checkout.
- risk: none for this repo state; there is no directory to clean or track.
- recommendation: no action.

### `.codex-remote-attachments/`

- observed: not present in this checkout.
- risk: none for this repo state; there is no directory to clean or track.
- recommendation: no action.

### `.env.example`

- observed: not present in this checkout.
- suspicious markers: not checked because the file is absent.
- risk: none in this checkout; there is no local template or secret-bearing file to inspect.
- recommendation: no action.

### `docs/dev/`

- observed: not present in this checkout.
- risk: none for this repo state; there is no dev-doc tree to classify.
- recommendation: no action.

### `semantic-textmate-grammar/`

- observed: not present in this checkout.
- nested git repo: false.
- risk: none for this repo state; there is no nested grammar checkout in this path.
- recommendation: no action.

## Recommended actions

Use only proposals, not actual actions.

- keep the listed holdback paths absent for this checkout;
- do not add missing paths to the repo just for sanitation;
- classify the current untracked audit docs separately if they need a future commit;
- if a future checkout contains any of these paths, inspect them before tracking or deleting.

## Do-not-do list

- do not run `git clean`;
- do not `git add .`;
- do not commit tool state;
- do not commit secrets;
- do not nest an external grammar repo into Semantic;
- do not mix sanitation with PCC/port work.

## Risks

| Risk | Severity | Mitigation |
|---|---:|---|
| deleting useful local artifacts | high | owner approval required |
| committing secrets from `.env.example` | high | inspect markers, no blind tracking |
| committing a nested grammar repo | high | detect `.git`, keep separate |
| mixing tool state into repo | medium/high | ignore or keep local |
| losing dev notes | medium | classify `docs/dev/` separately |

## Recommended next step

Choose one:

- no action, keep the current audit-doc untracked set separate;
- create a `.gitignore` update proposal if the absent paths appear in another checkout;
- ask owner decision if a future checkout contains any of these holdbacks;
- split current untracked audit docs into a separate housekeeping review.

## Final verdict

PASS-WITH-WARNINGS.

The requested holdback paths are absent in this checkout, so there is no cleanup
operation to perform on them. The warning remains because the worktree still has
other untracked audit docs, which should be handled as a separate classification
pass.
