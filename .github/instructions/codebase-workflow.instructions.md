---
applyTo: '**'
---

# Codebase Memory MCP Integration & Navigation Mechanics

Status: tool/integration instruction
Authority: subordinate to [`AGENTS.md`](../../AGENTS.md), [`CONSTRAINTS.md`](../../CONSTRAINTS.md), and [`docs/agents/WORKFLOW.md`](../../docs/agents/WORKFLOW.md)

This instruction governs Codebase Memory MCP mechanics and code discovery for AI assistants operating in the Semantic repository.

---

## 1. Discovery & Query Workflow

Codebase Memory MCP is mandatory for repository and source code understanding. Use graph queries before modifying code.

### Step 1: Project Identity
- Check if the project identifier is already known in the active session.
- If **not known**: Call `list_projects` once to discover the active project key.
- If **already known**: Do not repeat ceremonial `list_projects` discovery calls.

### Step 2: Architecture Overview
- Call `get_architecture(project)` once at task inception to ground codebase structure, crate boundaries, and hotspots.
- Refresh only when structural or module layout changes justify it.

### Step 3: Targeted Symbol & Call Traversal
Use targeted tools for symbol inspection and call-chain analysis:
- `search_graph(name_pattern, label, file_pattern)` — structured search for types, functions, and modules.
- `trace_call_path(function_name, direction, depth)` — trace callers (`inbound`) or callees (`outbound`).
- `get_code_snippet(qualified_name)` — retrieve symbol source code directly.
- `detect_changes(project)` — map diff impact to affected symbols and risk.

---

## 2. Strict No Autonomous Fallback

If Codebase Memory MCP is unavailable in the execution environment:

1. **STOP immediately**. Do not silently substitute broad grep, file search, approximate embeddings, or raw text scanning.
2. **Report the exact capability blocker** and state what cannot be proven or discovered.
3. **Await explicit repository-owner decision**:
   - Repository owner may authorize a task-scoped, visible, and temporary fallback.
   - Historical sessions where an agent proceeded without MCP tools do not create fallback authority.

---

## 3. Canonical Governance Reference

All repository invariants, crate ownership, verifier-first admission, Quad logic, risk tiers, and verification gates are defined in:
- [`AGENTS.md`](../../AGENTS.md)
- [`CONSTRAINTS.md`](../../CONSTRAINTS.md)
- [`docs/agents/WORKFLOW.md`](../../docs/agents/WORKFLOW.md)
- [`docs/agents/VERIFICATION.md`](../../docs/agents/VERIFICATION.md)
