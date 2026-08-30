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
4. **Explicit Repository-Owner Governance Authorization** — Used only to authorize a controlled governance-envelope transition when the task itself changes agent governance/Harness files that the current normal-task envelope does not permit.
5. **Task Envelope**: [`.harness/current.task.yaml`](.harness/current.task.yaml) — Normal active task scope, allowed/forbidden paths, authorizations, and task constraints.
6. **Contract Truth**: [`docs/spec/*`](docs/spec/) and [`docs/architecture/bootstrap_transition.md`](docs/architecture/bootstrap_transition.md) — Public language, format, verifier, runtime contracts, and implementation-era authority.
7. **Execution Methodology**: [`docs/agents/WORKFLOW.md`](docs/agents/WORKFLOW.md) and [`docs/agents/VERIFICATION.md`](docs/agents/VERIFICATION.md) — 5-phase lifecycle, toolstack rules, and verification catalog.
8. **Repository-Native Domain Skills**: [`.agents/skills/*`](.agents/skills/) — Semantic-specific implementation and authoring guards routed by task surface.

A lower layer in this hierarchy may make rules stricter, but may never loosen or waive an upper-layer rule.
$$\text{Effective Authority} = \text{Repository Invariants} \cap \text{Task Authority}$$

For governance-maintenance work, an agent may not self-authorize by broadening the Harness. Follow the controlled transition rules in `CONSTRAINTS.md` and `docs/agents/WORKFLOW.md`.

---

## 2. Core Repository Crate Ownership

Ground all changes in the canonical ownership boundaries of the repository:

- **Deterministic Core Libraries**:
  - **`sm-front`**: Frontend / parser / AST / source surface and syntax errors.
  - **`sm-sema`**: Semantic analysis / type checking / compile-time diagnostics.
  - **`sm-ir`**: Intermediate Representation (IR) and lowering passes.
  - **`sm-format`**: Canonical SemCode binary format and decoding contract.
  - **`sm-emit`**: Emission / producer-facing facade over the SemCode format.
  - **`sm-verify`**: Verifier admission gate (structure, layout, and bytecode rules).
  - **`sm-runtime-core`**: Shared runtime vocabulary, common execution types, and quotas.
  - **`sm-vm`**: Deterministic execution engine. Canonical trusted execution consumes verifier-admitted SemCode; explicitly documented raw/diagnostic APIs remain outside that trusted route.
- **Host-Facing Adapters & Platform Boundaries**:
  - **`smc-cli`**: Canonical public CLI owner; performs authorized host I/O without owning language semantics or verifier policy.
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

### D. External Agent Skills Routing
External process skills answer **how to work**; they do not own Semantic architecture or contracts.

- **`CONSTRAINTS.md`**: Always authoritative across every step (does not require ritual skill invocation for simple edits).
- **`source-driven-development`**: Use whenever verifying current source behavior, API contracts, or external dependencies.
- **`doubt-driven-development`**: Mandatory for Critical (R3) and selected Boundary (R2) changes; challenge assumptions, probe failure modes, and demand fresh proof.

### E. Repository-Native Semantic Skills
Repository-native skills answer **what Semantic permits within a specific domain** and are subordinate to `AGENTS.md`, `CONSTRAINTS.md`, the active Harness envelope, and normative specs.

- **`semantic`** — [`.agents/skills/semantic/SKILL.md`](.agents/skills/semantic/SKILL.md): current broad Semantic domain router/guard for compiler, SemCode, verifier, VM, runtime, PROMETHEUS, UI, ownership, status, tests, and Semantic-specific architecture work. Until #1846 decomposes this skill, use it as the general Semantic domain layer where its task description applies.
- **`semantic-source-authoring-guard`** — [`.agents/skills/semantic-source-authoring-guard/SKILL.md`](.agents/skills/semantic-source-authoring-guard/SKILL.md): **mandatory for any task that creates or modifies Semantic `.sm` source, fixtures, examples, or negative diagnostic probes.** Use it together with `semantic` when the task also changes architecture, compiler/runtime behavior, verifier/VM behavior, or repository contracts.

Do not silently resolve a conflict between a repository-native skill and higher authority. If a skill conflicts with `CONSTRAINTS.md`, the active Harness, normative `docs/spec/*`, or current verified repository evidence, stop and report the drift instead of choosing whichever source is more convenient.

Planned under #1846: reduce the broad `semantic` skill into a slim router plus specialized domain guards (verifier/runtime, contract/release, and UI boundary) while preserving the existing source-authoring guard.

### F. Conditional Agent Skills
- **`api-and-interface-design`**: Required for public API, ABI, or crate interface changes.
- **`security-and-hardening`**: Required for capability, sanitization, quota, or boundary modifications.
- **`code-review-and-quality`**: Required for pre-PR diff hygiene, clippy analysis, and quality validation.

---

## 4. Tooling & Orchestration Governance

- **Do Not Use RuFlo**: RuFlo is retired in this repository.
- **Ponytail Is Deferred/Experimental**: Ponytail must remain disabled by default.
- **Harness Scope Enforcement**:
  - Inspect untracked files first with `git ls-files --others --exclude-standard`; untracked paths are not visible to the current `harness-check.ps1` implementation.
  - Stage intended new files before running `pwsh -File scripts/harness-check.ps1`, so newly created files participate in the staged-path check.
  - Run `pwsh -File scripts/harness-check.ps1` before committing to validate staged/tracked changes against `.harness/current.task.yaml`.
  - Validate committed PR changes with `git diff --name-only origin/main...HEAD`.

---

## 5. Non-Negotiable Discipline

- **One Logical Change per PR**: Narrowly scoped to the assigned task.
- **Verifier-First Trusted Execution**: Never bypass `sm-verify` on canonical/trusted production execution routes. Explicitly documented raw/diagnostic `run_semcode*`-style APIs may remain unverified only within their narrow non-canonical diagnostic/compatibility perimeter and must never drift into trusted execution.
- **Total Determinism**: Preserve Quad Logic (`N`/`F`/`T`/`S`), deterministic VM execution, and deterministic diagnostics.
- **Capability Boundaries**: Never add direct filesystem, network, or OS effects inside deterministic Semantic core libraries. Host-facing adapters (`smc-cli`) perform authorized host operations but cannot define language semantics or verifier policy.
- **Tests for Behavior Changes**: Add positive admission and negative rejection tests whenever behavior changes. Never weaken or delete tests for CI.
- **Landed on Main != Stable**: Code on `main` is not automatically stable or release-promised. Never widen release claims silently.
- **No Completion Claim Without Fresh Evidence**: Always run exact verification commands and report exit codes and outputs.
