# GitHub Copilot Instructions: Semantic Repository

Status: client bootstrap adapter
Authority: subordinate to [`AGENTS.md`](../AGENTS.md), [`CONSTRAINTS.md`](../CONSTRAINTS.md), and [`.harness/current.task.yaml`](../.harness/current.task.yaml)

Welcome to Semantic (`skulmakov-oss/Semantic`). This file bootstraps GitHub Copilot into the canonical governance and toolstack of the repository.

---

## 1. Canonical Governance Routing

Before proposing changes, Copilot must follow the repository authority hierarchy:

1. **Platform & Safety Constraints**
2. **Canonical Entry Point**: [`AGENTS.md`](../AGENTS.md) — primary bootstrap and toolstack router.
3. **Hard Invariants**: [`CONSTRAINTS.md`](../CONSTRAINTS.md) — non-negotiable architectural and semantic laws.
4. **Active Task Envelope**: [`.harness/current.task.yaml`](../.harness/current.task.yaml) — allowed paths, task scope, and authorizations.
5. **Execution & Verification**: [`docs/agents/WORKFLOW.md`](../docs/agents/WORKFLOW.md) and [`docs/agents/VERIFICATION.md`](../docs/agents/VERIFICATION.md).

Copilot instructions are subordinate to these documents and cannot loosen or waive repository constraints.

---

## 2. Codebase Memory MCP Workflow

**Mandatory for repository code discovery and navigation.**

- **Disciplined Ingestion**:
  - `list_projects`: Call *only* when the exact project key is not already known for the current session. Do not repeat ceremonial discovery.
  - `get_architecture`: Obtain once per task/session; refresh only when structural changes justify it.
- **Targeted Symbol & Path Queries**:
  - `search_graph(name_pattern, label, file_pattern)` — locate functions, types, and modules.
  - `trace_call_path(function_name, direction, depth)` — trace caller/callee chains.
  - `get_code_snippet(qualified_name)` — retrieve symbol source code.
  - `detect_changes(project)` — map diffs to affected symbols and risk.
- **Strict No Autonomous Fallback**:
  - If Codebase Memory MCP is unavailable, do **not** silently fall back to grep, local-only discovery, or raw text scanning.
  - Follow the fail-closed blocker protocol in `CONSTRAINTS.md`:
    $$\text{capability unavailable} \rightarrow \text{STOP} \rightarrow \text{report exact blocker} \rightarrow \text{owner decides}$$
  - Only the repository owner may authorize a task-scoped, visible, and temporary fallback.

---

## 3. Core Repository Invariants

- **Verifier-First Trusted Execution**: Never bypass `sm-verify` on canonical production execution routes.
- **Total Determinism**: Preserve Quad Logic (`N = Null`, `F = Strict False`, `T = Strict True`, `S = Conflict / Super` per `docs/spec/quad_logic_frame_v1.md`), deterministic VM execution, and deterministic diagnostics.
- **Capability Boundaries**: Never add direct filesystem, network, or OS effects inside deterministic core libraries (`sm-front`, `sm-sema`, `sm-ir`, `sm-format`, `sm-emit`, `sm-verify`, `sm-runtime-core`, `sm-vm`). Host-facing CLI (`smc-cli`) performs authorized host operations but cannot own language semantics or verifier policy.
- **Scope Discipline**: Respect the active `.harness/current.task.yaml` envelope. Validate staged/committed files against allowed paths.
- **Evidence Before Assertions**: Run appropriate verification commands and report exact exit codes and output logs per `docs/agents/VERIFICATION.md`.
