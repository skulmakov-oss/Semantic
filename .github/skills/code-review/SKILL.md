---
name: code-review
description: Review Semantic pull requests on GitHub. Use Codebase Memory MCP when available; when it is unavailable in GitHub-hosted Copilot code review, use the repository-owner-authorized narrow read-only fallback defined here.
---

# Semantic GitHub Code Review

## Authority and scope

This skill applies **only to GitHub-hosted Copilot pull-request review**.

It is subordinate to:

1. `AGENTS.md`
2. `CONSTRAINTS.md`
3. `.harness/current.task.yaml`
4. `docs/spec/*` and other normative contract documents
5. `docs/agents/WORKFLOW.md`
6. `docs/agents/VERIFICATION.md`

The repository owner explicitly authorizes the fallback below only for the duration and scope of the **current pull-request review invocation** when Codebase Memory MCP is not exposed by the GitHub-hosted review environment.

This is a read-only review authorization. It is **not** authorization for implementation, repository mutation, task-envelope widening, or architectural redesign.

---

## Preferred review path: Codebase Memory MCP available

When Codebase Memory MCP is available, use the canonical workflow from `AGENTS.md` and `.github/instructions/codebase-workflow.instructions.md`:

- discover the known project only when needed;
- obtain the architecture view once per review task when relevant;
- use targeted symbol/call-path queries;
- reconcile findings with the PR diff, normative specs, tests, and active Harness envelope.

Do not use the fallback merely because ordinary repository browsing is more convenient.

---

## Owner-authorized hosted-review fallback

Activate this fallback **only if all of the following are true**:

1. the task is GitHub-hosted Copilot **code review** of a pull request;
2. Codebase Memory MCP is unavailable in that review environment;
3. the work remains read-only and limited to evaluating the current PR.

When activated, the reviewer MAY use only the following evidence sources:

- the current PR diff, changed-file list, PR metadata, and review discussion;
- `AGENTS.md`, `CONSTRAINTS.md`, `.harness/current.task.yaml`;
- `docs/agents/WORKFLOW.md` and `docs/agents/VERIFICATION.md`;
- normative specs and architecture documents directly relevant to the changed paths or claims;
- the complete contents of changed files and files they directly reference;
- existing tests/fixtures that directly exercise the changed behavior;
- targeted GitHub repository search for a **specific symbol, path, identifier, error string, or contract statement** needed to validate a concrete review claim.

This fallback MUST NOT become broad repository reconstruction by search.

---

## Prohibited under fallback

The reviewer MUST NOT:

- claim that Codebase Memory architecture/call-graph evidence was obtained when it was not;
- substitute broad grep/search sweeps for architecture discovery;
- infer subsystem-wide ownership, reachability, or call-graph facts that are not directly supported by the permitted evidence;
- edit files, commit, push, open/close issues, open/merge PRs, or otherwise mutate the repository;
- broaden the active Harness envelope;
- approve architecture or release/stability claims that require evidence unavailable in the hosted review environment;
- weaken a finding merely because the preferred MCP source is unavailable.

If a concrete claim cannot be validated from the permitted evidence, mark that claim as **not validated in hosted review** rather than inventing a substitute proof.

---

## Risk-sensitive behavior

Use the repository R0-R3 model from `docs/agents/WORKFLOW.md` and `docs/agents/VERIFICATION.md`.

### R0 / R1

A hosted fallback review may complete normally when the changed behavior and claims are fully checkable from the diff plus canonical repository documents/tests.

Do **not** report a generic blocker solely because Codebase Memory MCP is unavailable when the PR can be validated from authoritative local evidence.

### R2 / R3

Review all claims that can be checked from the permitted evidence, but clearly identify any architecture-wide, call-graph, verifier/runtime reachability, or cross-boundary conclusion that remains unvalidated without Codebase Memory MCP.

Do not convert an evidence limitation into an invented finding. Do not convert it into a blanket approval either.

---

## Review output contract

When the fallback is used, include a short disclosure in the review summary:

> Codebase Memory MCP was unavailable in the GitHub-hosted review environment. This review used the repository-owner-authorized read-only hosted-review fallback from `.github/skills/code-review/SKILL.md`.

Then:

1. report concrete findings first, with file/line evidence where possible;
2. separate actual findings from evidence limitations;
3. name any specific claim that could not be validated;
4. avoid a generic "needs closer look" status when no unresolved claim actually depends on the missing MCP;
5. if no concrete issue is found, say so while preserving any specific evidence limitation.

The fallback exists to keep GitHub-hosted review useful without pretending that missing MCP evidence exists.
