---
name: semantic-contract-release-guard
description: Domain guard for public API/ABI contracts, binary serialization formats, specification synchronization, and release/status honesty. Enforces the boundary between stable releases and forward-only main behavior.
---

# Semantic Contract & Release Guard

Status: repository-native domain guard
Authority: subordinate to [`AGENTS.md`](../../AGENTS.md), [`CONSTRAINTS.md`](../../CONSTRAINTS.md), and [`.harness/current.task.yaml`](../../.harness/current.task.yaml)

---

## 1. Purpose & Domain Scope

This domain guard governs:
- Public crate interfaces, ABI signatures, and module exports across the workspace;
- Binary serialization schemas and SemCode format contracts;
- Synchronization between normative specifications ([`docs/spec/*`](../../docs/spec/)) and repository implementations;
- Release status claims, stability declarations, and public guarantees ([`docs/roadmap/public_status_model.md`](../../docs/roadmap/public_status_model.md)).

---

## 2. Release & Stability Truth

### A. Non-Negotiable Release Law
$$\text{Landed on \texttt{main}} \neq \text{Stable} \neq \text{Released} \neq \text{Promised}$$

1. **No Retroactive Widening**: Never retroactively widen, redefine, or expand the guarantees of an already published stable release.
2. **Forward-Only Main Scope**: New language constructs, expanded VM features, or compiler optimizations merged to `main` represent forward-only development. They must not be described as part of an existing stable release line until formally qualified, documented, and released.
3. **Evidence-Backed Claims**: Every stability or release claim must be supported by normative specifications, positive qualification suites, negative rejection tests, and passing CI gates.

### B. Standard Status Vocabulary
Per [`docs/roadmap/public_status_model.md`](../../docs/roadmap/public_status_model.md), all status discussions must use unambiguous tiers:
- **`Published stable`**: Formally released version with frozen contracts and backwards-compatibility guarantees.
- **`Qualified limited release`**: Admitted for specific bounded use cases under documented constraints.
- **`Landed on main, not yet promised`**: Implemented in the development tree, but subject to evolution and not part of any released contract.
- **`Out of scope`**: Intentionally excluded or deferred.

*Note: Any conflict between documentation files regarding stability status must be treated as a readiness defect.*

---

## 3. Contract Synchronization Discipline

When modifying public APIs, ABIs, or binary formats:
1. **Public API Contracts**: Changes to public structs, enums, functions, traits, or crate root re-exports require updating and passing `cargo test --test public_api_contracts --quiet`.
2. **Normative Specification Sync**: Any intentional alteration to language grammar, type semantics, verifier checks, or SemCode encoding must be accompanied by updates to the corresponding normative documents in `docs/spec/*`.
3. **Implementation Placement vs. Normative Ownership**:
   - Implementation code resides in `sm-format` (format & decoding), `sm-emit` (producer emission), `sm-verify` (admission), `sm-vm` (execution).
   - Historical specification wording (e.g., in `docs/spec/*`) must not be rewritten without dedicated task authority.
   - If normative specification text and verified implementation ownership disagree, **STOP and report contract drift**. Do not perform unauthorized silent migrations.

---

## 4. External Process Skills Routing

When designing or reviewing public interfaces and contracts, agents may invoke external process skills:
- **`api-and-interface-design`**: Use for designing ergonomic, minimal, and backwards-compatible Rust APIs.
- **`doubt-driven-development`**: Mandatory for probing breaking changes, migration risks, and edge-case contracts.
- **`security-and-hardening`**: Use when contracts cross capability, quota, or serialization boundaries.
- **`code-review-and-quality`**: Use for pre-PR diff hygiene, clippy analysis, and quality validation.

*External skills provide methodology (HOW TO WORK); they do NOT override repository invariants or normative specifications (WHAT SEMANTIC PERMITS).*

---

## 5. Stop Conditions

Stop execution and report a blocker immediately if:
- **Contract Drift**: Public code behavior deviates from normative `docs/spec/*` without authorized specification updates.
- **Unverified Stability Claim**: A PR description, documentation edit, or commit message claims a feature is "stable" or "released" without published evidence.
- **Retroactive Widening**: A change reinterprets an existing release baseline to include new, unadmitted capabilities.
- **Public API Breakage**: Public API contract tests fail and no authorized breaking-change envelope is active.
