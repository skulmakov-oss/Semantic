# AGENTS.md Owner Decision Note

## Status

Result: NEEDS-CONTENT-REVIEW

This is an owner-decision note.

No files were staged.
No files were committed.
No files were stashed.
No files were restored.
No reset --hard was run.
No git clean was run.
No code/tests/examples/7hell files were changed.

## Source repo state

- branch: `codex/ui-native-wgpu-boundary-wording`
- HEAD: `9f75fa2bcead68f3925514c00e9c35e5f8334618`
- origin/main: `64dcb4b3ea689e1381fec757a863370b4b677455`
- HEAD == origin/main: yes
- dirty tree summary: one tracked dirty file plus local untracked residue groups remain

## AGENTS.md state

- tracked state: tracked dirty
- summary of diff: appended a new `## RuFlo / Codex MCP operating rules` section with 8 repository-policy bullets
- suspected purpose: local repo policy guidance for RuFlo/Codex MCP usage and post-install operating discipline
- risk: this is a tracked policy change in a repo control file, so it can be mixed into unrelated UI work if not handled explicitly
- recommended owner decision: `NEEDS-CONTENT-REVIEW`

## Options

| Option | Meaning | Risk | When to use |
|---|---|---:|---|
| KEEP-DIRTY-FOR-NOW | leave as local change | low/medium | if not starting implementation yet |
| STASH-LATER-WITH-OWNER-APPROVAL | stash only AGENTS.md later | medium | before clean implementation branch |
| COMMIT-SEPARATELY-LATER | commit as separate docs/tooling PR | medium | if change is intentional repo policy |
| RESTORE-LATER-WITH-OWNER-APPROVAL | discard local change later | medium/high | if change is accidental |
| NEEDS-CONTENT-REVIEW | inspect more before deciding | low | if meaning is unclear |

## Recommended decision

Choose one:

`NEEDS-CONTENT-REVIEW`

## Final verdict

UI implementation remains blocked until `AGENTS.md` is resolved by the owner.

Because the diff is policy-like rather than obviously accidental, the safest current action is to keep it out of any UI implementation branch until the owner confirms whether it should be preserved, stashed, committed separately, or restored.
