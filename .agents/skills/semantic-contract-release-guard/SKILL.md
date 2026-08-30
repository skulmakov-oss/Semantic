---
name: semantic-contract-release-guard
description: Domain guard for public API/ABI contracts, binary serialization formats, specification synchronization, and release/status honesty. Enforces the boundary between stable releases and forward-only main behavior.
---

# Semantic Contract & Release Guard

Status: repository-native domain guard
Authority: subordinate to [`AGENTS.md`](../../../AGENTS.md), [`CONSTRAINTS.md`](../../../CONSTRAINTS.md), and [`.harness/current.task.yaml`](../../../.harness/current.task.yaml)

---

## 1. Purpose & Domain Scope

This domain guard governs:
- public crate interfaces, ABI signatures, and module exports across the workspace;
- binary serialization schemas and SemCode format contracts;
- synchronization between normative specifications ([`docs/spec/*`](../../../docs/spec/)) and repository implementations;
- release status claims, stability declarations, and public guarantees.

---

## 2. Release & Stability Truth

### A. Non-Negotiable Release Law

`Landed on main != Stable != Released != Promised`

1. **No Retroactive Widening**: Never widen, redefine, or expand the guarantees of an already published stable release.
2. **Forward-Only Main Scope**: New constructs or features merged to `main` are forward-only until formally qualified, documented, and released.
3. **Evidence-Backed Claims**: Use the authority appropriate to the claim; specifications, tests, and CI do not by themselves prove publication.

### B. Status Authority Routing

- **Vocabulary**: [`docs/roadmap/public_status_model.md`](../../../docs/roadmap/public_status_model.md) defines status vocabulary only.
- **Current release-facing posture**: [`docs/roadmap/v1_readiness.md`](../../../docs/roadmap/v1_readiness.md).
- **Current practical-programming qualification verdict**: [`reports/g1_release_scope_statement.md`](../../../reports/g1_release_scope_statement.md).
- **Published stable / released claim**: verify the relevant tag, GitHub release, assets, and release-governance evidence at the time of the claim.

Do not turn a green CI result, a tag, or behavior merely landed on `main` into a published-stable claim.

---

## 3. Contract Synchronization Discipline

When modifying public APIs, ABIs, or binary formats:

1. **Public API Contracts**: Changes to public structs, enums, functions, traits, or crate-root re-exports require updating and passing `cargo test --test public_api_contracts --quiet`.
2. **Normative Specification Sync**: An intentional change to language grammar, type semantics, verifier checks, or SemCode encoding requires synchronization with the owning normative document in `docs/spec/*`.
3. **Authorization Is Separate**: A synchronization requirement never grants authority to edit a specification. If the needed spec path is outside the active Harness, stop and obtain task-scoped authority.
4. **Implementation Placement vs. Normative Ownership**:
   - Implementation code resides in `sm-format` (format and decoding), `sm-emit` (producer emission), `sm-verify` (admission), and `sm-vm` (execution).
   - If normative specification text and verified implementation ownership disagree, stop and report contract drift. Do not perform an unauthorized migration.

---

## 4. External Process Skills Routing

When designing or reviewing public interfaces and contracts, agents may invoke external process skills:
- **`api-and-interface-design`**: Use for ergonomic, minimal, and backwards-compatible Rust APIs.
- **`doubt-driven-development`**: Use for breaking changes, migration risks, and edge-case contracts.
- **`security-and-hardening`**: Use when contracts cross capability, quota, or serialization boundaries.
- **`code-review-and-quality`**: Use for pre-PR diff hygiene, clippy analysis, and quality validation.

External skills provide methodology; they do not override repository invariants or normative specifications.

---

## 5. Stop Conditions

Stop execution and report a blocker immediately if:
- **Contract Drift**: Public code behavior deviates from normative `docs/spec/*` without authorized specification updates.
- **Unverified Stability Claim**: A PR description, documentation edit, or commit message calls a feature stable or released without the relevant publication evidence.
- **Retroactive Widening**: A change reinterprets an existing release baseline to include new, unadmitted capabilities.
- **Public API Breakage**: Public API contract tests fail and no authorized breaking-change envelope is active.
