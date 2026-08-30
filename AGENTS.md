# AGENTS.md

Status: canonical agent bootstrap and routing authority
Repository: `skulmakov-oss/Semantic`

Welcome to Semantic. This file is the primary entry point and routing table for AI agents and human contributors working on this codebase.

---

## 1. Quick Router & Authority Precedence

Before reading code or proposing changes, every agent must consult these authorities in priority order:

1. **Platform & Safety Constraints** — Non-negotiable environment and safety invariants.
2. **Bootstrap & Router**: `AGENTS.md` (this file) — Canonical repository entry point and toolstack routing.
3. **Hard Invariants**: [`CONSTRAINTS.md`](CONSTRAINTS.md) — Non-negotiable architectural, semantic, determinism, and verification laws.
4. **Task Envelope**: [`.harness/current.task.yaml`](.harness/current.task.yaml) — Active task scope, allowed/forbidden paths, authorizations, and task constraints.
5. **Contract Truth**: [`docs/spec/*`](docs/spec/) and [`docs/architecture/bootstrap_transition.md`](docs/architecture/bootstrap_transition.md) — Public language, format, verifier, runtime contracts, and implementation-era authority.
6. **Execution Methodology**: [`docs/agents/WORKFLOW.md`](docs/agents/WORKFLOW.md) and [`docs/agents/VERIFICATION.md`](docs/agents/VERIFICATION.md) — 5-phase lifecycle, toolstack rules, and verification catalog.

A lower layer in this hierarchy may make rules stricter, but may never loosen or waive an upper-layer rule.
$$\text{Effective Authority} = \text{Repository Invariants} \cap \text{Task Authority}$$

---

## 2. Core Repository Crate Ownership

Ground all changes in the canonical ownership boundaries of the repository:

- **`sm-front`**: Frontend / parser / AST / source surface and syntax errors.
- **`sm-sema`**: Semantic analysis / type checking / compile-time diagnostics.
- **`sm-ir`**: Intermediate Representation (IR) and lowering passes.
- **`sm-format`**: Canonical SemCode binary format and decoding contract.
- **`sm-emit`**: Emission / producer-facing facade over the SemCode format.
- **`sm-verify`**: Verifier admission gate (structure, layout, and bytecode rules).
- **`sm-runtime-core`**: Shared runtime vocabulary, common execution types, and quotas.
- **`sm-vm`**: Deterministic execution of verifier-admitted SemCode.
- **`smc-cli`**: Canonical public CLI owner.
- **`prom-*`**: PROMETHEUS host ABI, capability policy, gate descriptors, runtime sessions, rules, and audit logging.
- **`prom-ui*`**: Platform-neutral UI orchestration, presentation models, and backend facades.

---

## 3. Mandatory Agent Stack

### A. Codebase Memory MCP (Code Discovery & Navigation)
**Mandatory for repository code discovery and navigation.**

- **Disciplined Ingestion**:
  - Call `list_projects` only when the exact project key is not already known for the current task/session.
  - Obtain `get_architecture` once per task, and refresh only when structural changes justify it.
  - Avoid ritualistic repeated discovery calls on every turn.
- **Targeted Queries**:
  - `search_graph(name_pattern, label, file_pattern)` — locate functions, types, and modules.
  - `trace_call_path(function_name, direction, depth)` — trace caller/callee chains.
  - `get_code_snippet(qualified_name)` — retrieve symbol source code.
  - `detect_changes(project)` — map git diffs to affected symbols and risk.

### B. mcp-local-rag (Documentation, Research & History)
**Mandatory for specifications, historical decision logs, architecture reports, and external references.**

- RAG outputs are retrieval evidence, not authority: reconcile retrieval against current code, tests, and specs.
- `query_documents(query)` — search indexed documentation.
- `status()` — inspect indexing coverage.

### C. obra/superpowers (Primary Execution Methodology)
When available, Superpowers owns the primary planning, TDD, debugging, and verification workflow. Superpowers operates **inside** Semantic governance and Harness authority, never above them.
- `using-superpowers`, `brainstorming`, `writing-plans`, `executing-plans`, `test-driven-development`, `systematic-debugging`, `verification-before-completion`.

### D. Selected Agent Skills Routing
- **`CONSTRAINTS.md`**: Always authoritative across every step (does not require ritual skill invocation for simple edits).
- **`source-driven-development`**: Use whenever verifying current source behavior, API contracts, or external dependencies.
- **`doubt-driven-development`**: Mandatory for Critical (R3) and selected Boundary (R2) changes; challenge assumptions, probe failure modes, and demand fresh proof.
- **`semantic`**: Enforces domain-specific rules for Semantic crates, SemCode, verifier admission, VM execution, and PROMETHEUS boundaries.

### E. Conditional Agent Skills
- **`api-and-interface-design`**: Required for public API, ABI, or crate interface changes.
- **`security-and-hardening`**: Required for capability, sanitization, quota, or boundary modifications.
- **`code-review-and-quality`**: Required for pre-PR diff hygiene, clippy analysis, and quality validation.

---

## 4. Tooling & Orchestration Governance

- **Do Not Use RuFlo**: RuFlo is retired in this repository.
- **Ponytail Is Deferred/Experimental**: Ponytail must remain disabled by default.
- **Harness Scope Enforcement**:
  - Run `pwsh -File scripts/harness-check.ps1` before committing to validate working-tree and staged changes against `.harness/current.task.yaml`.
  - Validate committed PR changes with `git diff --name-only origin/main...HEAD`.

---

## 5. Non-Negotiable Discipline

- **One Logical Change per PR**: Narrowly scoped to the assigned task.
- **Verifier-First Admission**: Never bypass `sm-verify`; never execute unchecked SemCode.
- **Total Determinism**: Preserve Quad Logic (`N`/`F`/`T`/`S`), deterministic VM execution, and deterministic diagnostics.
- **Capability Boundaries**: Never add direct filesystem, network, or OS effects inside Semantic core.
- **Tests for Behavior Changes**: Add positive admission and negative rejection tests whenever behavior changes. Never weaken or delete tests for CI.
- **Landed on Main != Stable**: Code on `main` is not automatically stable or release-promised. Never widen release claims silently.
- **No Completion Claim Without Fresh Evidence**: Always run exact verification commands and report exit codes and outputs.
