# Agent Adapter: `.agents/**` Maintenance & Skill Hygiene

Status: client/subsystem adapter
Authority: subordinate to [`AGENTS.md`](../AGENTS.md), [`CONSTRAINTS.md`](../CONSTRAINTS.md), and [`.harness/current.task.yaml`](../.harness/current.task.yaml)

This file defines maintenance rules and authoring hygiene for agent configurations and repository skills within `.agents/**`.

---

## 1. Governance Routing

All work in this repository is governed by the canonical hierarchy:

1. **Platform & Safety Constraints**
2. **Canonical Router**: [`AGENTS.md`](../AGENTS.md)
3. **Hard Invariants**: [`CONSTRAINTS.md`](../CONSTRAINTS.md)
4. **Task Envelope**: [`.harness/current.task.yaml`](../.harness/current.task.yaml)
5. **Contract Truth**: [`docs/spec/*`](../docs/spec/)
6. **Workflow & Verification**: [`docs/agents/WORKFLOW.md`](../docs/agents/WORKFLOW.md) and [`docs/agents/VERIFICATION.md`](../docs/agents/VERIFICATION.md)

This adapter is subordinate to these authorities and cannot loosen or waive repository rules.

---

## 2. Skill File Hygiene (`.agents/skills/**`)

When maintaining or authoring repository skills:

- **Valid Frontmatter**: Every `SKILL.md` must include valid YAML frontmatter with `name` and `description`.
- **Explicit Scope & Non-Goals**: Define exact domain applicability, constraints, and non-goals.
- **Domain Guard Role**: Repository skills answer *what Semantic permits in a domain*; they do not define general repository governance or override normative specifications (`docs/spec/*`).
- **Invariant Integrity**: Never silently delete, weaken, or reinterpret repository invariants or test requirements in a skill.
- **No Direct Modifications**: Do not modify skill files under `.agents/skills/**` unless explicitly authorized by issue scope (e.g. AGENT-INFRA-03 / #1846).

---

## 3. Toolstack & Discovery Routing

- **Codebase Memory MCP**: Mandatory for repository code discovery.
  - Project Discovery: Call `list_projects` only when project identity is not already known in the session.
  - Architecture: Obtain `get_architecture` once per task/session.
  - Targeted Queries: Use `search_graph`, `trace_call_path`, and `get_code_snippet`.
- **No Autonomous Fallback**: If Codebase Memory MCP is unavailable, agents must not silently substitute ungrounded discovery methods. Follow the fail-closed blocker protocol in `CONSTRAINTS.md` (STOP -> report blocker -> await repository-owner decision).
- **mcp-local-rag**: Mandatory for specifications, historical decision logs, and research.

---

## 4. Verification & Validation Rules

- Scale verification to the task's risk classification (R0–R3) per [`docs/agents/VERIFICATION.md`](../docs/agents/VERIFICATION.md).
- When running workspace checks locally:
  - Format Check: Use `pwsh -File scripts/workspace_fmt_check.ps1` (Windows command-line safe).
  - Clippy: `cargo clippy --workspace --all-targets -- -D warnings`.
  - Boundary & Contracts: `cargo test --test legacy_guards --test public_api_contracts --quiet`.
  - Full Workspace: Run `cargo test --workspace --all-targets` for comprehensive workspace validation (note: local `-CIParity` Step 6 runs default package scope; see `docs/agents/VERIFICATION.md` for known coverage gaps).
