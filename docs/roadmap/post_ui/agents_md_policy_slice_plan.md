# AGENTS.md Policy Slice Plan

## Status

Result: READY-WITH-WARNINGS

This is an audit-only / plan-only document.

No files were staged.
No files were committed.
No files were stashed.
No files were restored.
No reset --hard was run.
No git clean was run.
No code/tests/examples/7hell files were changed.
No AGENTS.md changes were made.

## Purpose

Explain why `AGENTS.md` must be handled as a separate docs/tooling policy slice before UI implementation work continues.

## Source repo state

- branch: `codex/ui-native-wgpu-boundary-wording`
- HEAD: `9f75fa2bcead68f3925514c00e9c35e5f8334618`
- origin/main: `64dcb4b3ea689e1381fec757a863370b4b677455`
- HEAD == origin/main: yes
- dirty tree summary: one tracked dirty file plus local untracked residue groups remain
- tracked dirty files: `AGENTS.md`
- untracked residue: local PCC / audit residue and existing post-UI audit notes remain untouched

## AGENTS.md change summary

- file: `AGENTS.md`
- tracked state: tracked dirty
- diff size: 11 inserted lines
- suspected purpose: repo policy / tooling guidance for RuFlo / Codex MCP operating rules
- affected workflow: agent behavior, repository-state inspection discipline, small auditable patches, verifier-first boundaries, no push/commit without instruction
- coupling to UI/PCC/CTF/code/tests/examples/7hell: no direct coupling detected

## Classification matrix

| Criterion | Result | Evidence |
|---|---:|---|
| Policy/tooling intent | PASS | The diff adds a dedicated `RuFlo / Codex MCP operating rules` section and repo-operating guidance. |
| Not random local noise | PASS | The content is structured, policy-like, and aligned with repo/agent workflow rather than incidental text. |
| No UI implementation coupling | PASS | The diff does not touch UI source, renderer, or windowing code and does not reference UI implementation details. |
| No PCC/CTF coupling | PASS | The diff does not modify PCC/CTF artifacts or reference their mechanics directly. |
| No code/tests/examples/7hell coupling | PASS | No source, test, example, or `7hell` files are touched. |
| Safe as separate PR candidate | PASS | The change is policy/tooling-oriented and can be reviewed as a standalone repo-instruction update. |

## Risk assessment

| Risk | Severity | Mitigation |
|---|---:|---|
| Mixing `AGENTS.md` with UI implementation | high | Separate PR only; do not stage with UI slices. |
| Accidentally discarding useful policy | medium/high | No restore without owner approval. |
| Keeping dirty file blocks UI work | medium | Resolve as a separate policy slice before implementation. |
| Agent behavior changes unexpectedly | medium | Review exact wording before any commit. |

## Proposed separate slice

If approved later:

Commit message:

```text
docs(tooling): update AGENTS policy guidance
```

Allowed files:

- `AGENTS.md`

Optional supporting doc if owner approves:

- `docs/roadmap/post_ui/agents_md_policy_slice_plan.md`

Do not include:

- `code/`
- `tests/`
- `examples/`
- `tools/7hell/`
- `docs/roadmap/pcc/`
- post-UI implementation docs
- untracked audit residue

Commit gates for future action:

1. `git diff --cached --name-only` must contain only approved files.
2. No UI implementation files.
3. No PCC/CTF residue.
4. No tests/examples/7hell.
5. `AGENTS.md` wording must be reviewed as repo-level policy/tooling guidance.
6. PR body must state contract impact: none.

## Recommended next step

`prepare separate AGENTS.md policy PR`

## Non-goals

- no UI implementation
- no renderer rewrite
- no backend switch
- no runtime/verifier/VM changes
- no PCC/CTF changes
- no tests/examples/7hell changes
- no cleanup execution

## Final verdict

`AGENTS.md` can be handled as a separate policy/tooling slice.

The diff looks intentionally useful enough to preserve, but it must stay isolated from UI implementation and other post-UI work until it is either committed separately or otherwise resolved through an owner-approved policy path.
