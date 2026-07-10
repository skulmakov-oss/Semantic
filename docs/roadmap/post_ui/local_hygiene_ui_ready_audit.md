# LOCAL-HYGIENE-UI-READY Audit

## Status

Result: NEEDS-OWNER-DECISION

This is an audit-only hygiene checkpoint.

No files were deleted.
No files were moved.
No files were staged.
No files were committed.
No git clean was run.
No reset --hard was run.
No code/tests/examples/7hell files were changed.
No AGENTS.md changes were made.

## Purpose

Explain that UI architecture/docs are ready enough, but implementation readiness is blocked by local checkout hygiene.

## Source repo state

- branch: `codex/ui-native-wgpu-boundary-wording`
- HEAD: `9f75fa2bcead68f3925514c00e9c35e5f8334618`
- origin/main: `64dcb4b3ea689e1381fec757a863370b4b677455`
- HEAD == origin/main: yes
- branch relation: on a feature branch, but synchronized with `origin/main`
- dirty tree summary: one tracked dirty file plus multiple local untracked residue groups
- tracked dirty files: `AGENTS.md`
- untracked residue count: 26

## AGENTS.md classification

| File | State | Classification | Reason | Recommended future action |
|---|---|---|---|---|
| `AGENTS.md` | tracked dirty | DO-NOT-TOUCH | This is an unrelated tracked dirty file and must not be mixed into any UI implementation branch. | Ask owner decision before any stash, branch split, or separate PR. |

## Untracked residue classification

| Path / group | Count | Classification | Reason | Recommended future action |
|---|---:|---|---|---|
| `.agents/config.toml`, `.claude-flow/`, `.mcp.json`, `CLAUDE.md` | 10 | DO-NOT-TOUCH | Local Codex/RuFlo configuration residue is unrelated to the UI slice and must not be swept into implementation cleanup. | Keep local for now; decide separately whether any of these belong in repo or stay local-only. |
| `docs/roadmap/pcc/*.md` | 12 | DO-NOT-TOUCH | PCC / port audit residue is explicitly out of scope for UI implementation readiness. | Preserve locally until the PCC owner decides on disposition. |
| `docs/roadmap/post_ui/ui_reentry_1_renderer_boundary_verification.md`, `docs/roadmap/post_ui/ui_reentry_2_windowing_boundary_verification.md`, `docs/roadmap/post_ui/ui_reentry_3_native_wgpu_reality_alignment.md`, `docs/roadmap/post_ui/ui_reentry_checkpoint.md` | 4 | DO-NOT-TOUCH | These are local audit artifacts and should not be deleted or mixed into hygiene cleanup. | Keep as local audit trail unless the roadmap explicitly asks to publish or supersede them. |

## UI readiness state

| Area | Status | Notes |
|---|---|---|
| native/WGPU wording | aligned | `#1305` merged and the docs now reflect feature-gated backend-native reality. |
| renderer boundary | verified | Renderer boundary verification found no source/docs mismatch requiring an immediate patch. |
| windowing boundary | verified | Windowing boundary verification found the contract-level wording consistent with runtime/native source reality. |
| implementation readiness | blocked | The checkout is not yet safe for a new implementation branch because `AGENTS.md` is still tracked-dirty and must be treated separately. |

## Safe hygiene options

| Option | Description | Pros | Cons |
|---|---|---|---|
| A - No action | Leave the checkout as-is for now. | Preserves all local evidence and avoids accidental mixing. | Implementation cannot start cleanly yet. |
| B - Stash tracked dirty file only | Future option: `git stash push -m "local AGENTS.md holdback before UI implementation" -- AGENTS.md`. | Preserves `AGENTS.md` without committing it. | Must be owner-approved and handled carefully so unrelated residue is not swept in. |
| C - Create safety branch for dirty state | Future option: `git switch -c backup/local-agents-holdback`. | Captures the exact state for later review. | Still requires a separate decision for implementation hygiene. |
| D - Commit `AGENTS.md` separately | Future option only if the file is intentional repo content. | Makes the tracked change explicit and reviewable. | Must be a separate PR and cannot be mixed with UI implementation. |
| E - Sync `main` after preserving dirty file | Future option after owner-approved preservation. | Returns the branch to current `main` without losing the dirty file. | Must not use `reset --hard` while the tracked dirty file is unresolved. |

## Recommended next step

`Ask owner decision on AGENTS.md`

## Non-goals

- no UI implementation
- no renderer rewrite
- no backend switch
- no runtime/verifier/VM changes
- no PCC/CTF changes
- no tests/examples/7hell changes
- no cleanup execution

## Final verdict

UI docs and source are ready enough for the next architectural step, but future implementation remains blocked until `AGENTS.md` is resolved by the owner.

This checkout should not be used for a minimal UI implementation slice until the tracked dirty file is either intentionally kept, separately committed, or isolated in an owner-approved preservation path.
