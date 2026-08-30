# [<TASK-ID>] <Short Title>

Closes: <issue link or task id>
Harness Task ID: `<TASK-ID>` (per `.harness/current.task.yaml`)
Risk Classification: <R0-Informational | R1-Private | R2-Boundary | R3-Critical> (per `docs/agents/WORKFLOW.md`)
One PR = one logical change.

---

## 1. Objective & Authority

- **Task Authority**: <Authorized directive, issue link, or task reference>
- **Summary**: <What problem this PR solves and what it accomplishes>

---

## 2. Scope & Boundaries

### What This PR Does
- <item 1>
- <item 2>

### What This PR Does NOT Do (Non-Goals)
- <non-goal 1>
- <non-goal 2>

### Boundary & Contract Confirmation
- [ ] Zero unauthorized compiler/parser/sema/IR/SemCode/verifier/VM/runtime changes
- [ ] Zero unauthorized dependency changes
- [ ] Zero CI workflow modifications
- [ ] Spec/contract updated if public API, format, or runtime semantics changed (or N/A)

### Stable / Release Boundary

Published stable release: `<version or N/A>`

- Published stable contract:
  - `<what the published release actually guarantees>`

- Current `main` after this PR:
  - `<new or changed behavior, if any>`

Release-status confirmation:
- [ ] This PR does not retroactively widen or redefine an already published stable release.
- [ ] Any forward-only behavior exists only on current `main` until explicitly released.
- [ ] Stable/release claims are backed by corresponding published specification/evidence.
- [ ] N/A — this PR cannot affect published/current contract interpretation.

---

## 3. Changed Files

`git diff --name-only origin/main...HEAD` must match authorized paths:
- `<file 1>`
- `<file 2>`

---

## 4. Verification Evidence

### Local Verification Commands
| Command | Exit Code | Summary / Evidence |
|---|---|---|
| `pwsh -File scripts/workspace_fmt_check.ps1` | `<exit-code>` | `<observed result>` |
| `cargo clippy --workspace --all-targets -- -D warnings` | `<exit-code>` | `<observed result>` |
| `cargo test --test legacy_guards --quiet` | `<exit-code>` | `<observed result>` |
| `cargo test --test public_api_contracts --quiet` | `<exit-code>` | `<observed result>` |
| `<additional-focused-command>` | `<exit-code>` | `<observed result>` |

### CI Parity & Known Gaps
- Local gate executed: `[None | -Quick | -PRReady | -CIParity | -Readiness | -FullPreflight]`
- Disclosed local coverage differences vs `.github/workflows/ci.yml` (per `docs/agents/VERIFICATION.md`):
  - Local `-CIParity` tests package scope; full workspace testing requires `cargo test --workspace --all-targets`.
  - Doctests (`cargo test --workspace --doc`), Windows 7hell gate, and SARIF uploads are executed in CI.

### Fallback Authorization
- [ ] No fallback used (standard Codebase Memory MCP / toolstack verified)
- [ ] Task-scoped fallback authorized by repository owner: `<decision-reference>`

---

## 5. Residual Risks & Next Step

- **Residual Risks**: <None | specific temporary overlap or bounded follow-up>
- **Next Authorized Step**: <Next sub-issue or task in sequence>
