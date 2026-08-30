# AGENTS.md

Status: canonical agent bootstrap and routing authority
Repository: `skulmakov-oss/Semantic`

Welcome to Semantic. This file is the primary entry point and routing table for AI agents and human contributors working on this codebase.

---

## 1. Quick Router & Authority Hierarchy

Before reading code or making modifications, every agent must consult these authorities in priority order:

1. **Hard Invariants**: [`CONSTRAINTS.md`](CONSTRAINTS.md) — Non-negotiable architectural, semantic, and verification invariants.
2. **Task Envelope**: [`.harness/current.task.yaml`](.harness/current.task.yaml) — Active task scope, allowed/forbidden paths, authorizations, and task constraints.
3. **Execution Workflow**: [`docs/agents/WORKFLOW.md`](docs/agents/WORKFLOW.md) — 5-phase lifecycle, toolstack instructions, and agent methodology.
4. **Verification Catalog**: [`docs/agents/VERIFICATION.md`](docs/agents/VERIFICATION.md) — Exact verification commands, Admission Guard modes, and CI parity gates.
5. **Contract Truth**: [`docs/spec/*`](docs/spec/) — Public language, format, verifier, and runtime contracts.
6. **Implementation Era Authority**: [`docs/architecture/bootstrap_transition.md`](docs/architecture/bootstrap_transition.md) — Canonical era sequencing and owner boundaries.

---

## 2. Mandatory Agent Stack

### A. Codebase Memory MCP (Code Discovery & Understanding)
**MANDATORY: Use Codebase Memory MCP graph tools FIRST — before reading raw files or making code changes.**

- **Step 0 — Discover project**:
  ```json
  list_projects()
  ```
- **Step 1 — Understand architecture**:
  ```json
  get_architecture({ "project": "<display_name>" })
  ```
- **Step 2 — Graph query and symbol search**:
  - `search_graph(name_pattern, label, file_pattern)` — Locate functions, types, routes, modules.
  - `trace_call_path(function_name, direction, depth)` — Trace caller and callee chains.
  - `get_code_snippet(qualified_name)` — Retrieve specific symbol implementation.
  - `query_graph(query)` — Run Cypher-like relationship queries.
  - `detect_changes(project)` — Map git diff to affected symbols and risk.

### B. mcp-local-rag (Documentation, Research & History)
**MANDATORY: Use mcp-local-rag for documentation lookup, historical context, specifications, roadmap reports, and external reference material.**

- `query_documents(query)` — Semantic search across indexed repository docs, architecture reports, and specs.
- `status()` — Inspect index and memory status.

### C. obra/superpowers (Primary Execution Methodology)
Follow Superpowers discipline for structured execution:
- `using-superpowers` — Skill discovery and invocation rule before any response or action.
- `brainstorming` — Explore intent, requirements, and design before implementation.
- `writing-plans` / `executing-plans` — Plan-driven execution with review checkpoints.
- `test-driven-development` — Write tests before implementation code.
- `systematic-debugging` — Rigorous root-cause analysis before fixes.
- `verification-before-completion` — Confirm evidence before asserting success.

### D. Core Agent Skills
- `constraint-driven-development` — Validate every proposed change against `CONSTRAINTS.md` and `.harness/current.task.yaml` prior to touching code or docs.
- `source-driven-development` — Ground all statements, designs, and refactors directly in existing source code, tests, and specs.
- `doubt-driven-development` — Systematically doubt assumptions, probe edge cases, test failure modes, and demand fresh proof.
- `semantic` — Domain-specific rules for Semantic crates, SemCode, verifier, VM, and PROMETHEUS boundaries.

### E. Conditional Agent Skills
- `api-and-interface-design` — For designing public APIs, ABI types, and crate interfaces.
- `security-and-hardening` — For boundary validation, sanitization, quota enforcement, and capability checks.
- `code-review-and-quality` — For pre-PR review, clippy validation, and diff hygiene.

---

## 3. Tooling & Orchestration Governance

- **Do Not Use RuFlo**: RuFlo is retired in this repository.
- **Ponytail Is Deferred/Experimental**: Ponytail must remain disabled by default.
- **Harness Scope Enforcement**: All changes must stay within `.harness/current.task.yaml` `allowed_paths`. Run `pwsh -File scripts/harness-check.ps1` before and after changes.

---

## 4. Non-Negotiable Discipline

- **One Logical Change per PR**: Keep changes narrowly scoped to the assigned task.
- **Verifier-First Admission**: Never bypass `sm-verify`; never execute unchecked SemCode on public routes.
- **Total Determinism**: Preserve Quad Logic (`N`/`F`/`T`/`S`), deterministic VM execution, and deterministic diagnostics.
- **Capability Boundaries**: Never add direct filesystem, network, or OS effects inside Semantic core.
- **Tests for Behavior Changes**: Add positive admission and negative rejection tests whenever behavior changes.
- **Landed on Main != Stable**: Code on `main` is not automatically stable or release-promised. Never widen release claims silently.
- **No Completion Claim Without Fresh Evidence**: Always run exact verification commands and report exit codes and outputs.
