---
name: semantic-ui-boundary-guard
description: Domain guard for Semantic UI orchestration, renderer presentation models, interaction semantics, trace/audit visual projections, and native backend facades. Enforces strict separation between UI presentation and Semantic core/runtime authority.
---

# Semantic UI Boundary Guard

Status: repository-native domain guard
Authority: subordinate to [`AGENTS.md`](../../../AGENTS.md), [`CONSTRAINTS.md`](../../../CONSTRAINTS.md), and [`.harness/current.task.yaml`](../../../.harness/current.task.yaml)

---

## 1. Purpose & Domain Scope

This domain guard governs:
- **`prom-ui*`**: UI contract vocabulary, visual boundary types, and trace/audit projection contracts;
- **`prom-ui-runtime`**: Platform-neutral UI runtime orchestration;
- **`prom-ui-backend-native`**: Native backend facade and platform event bridging;
- **`examples/workbench_semantic`**: Canonical native Semantic Workbench (developer and operator tooling surface);
- presentation models, layout solving, interaction pipelines, and visual projections.

---

## 2. Fundamental UI Invariants

1. **Presentation, Not Compiler**: Semantic UI is an operator/presentation layer over admitted contracts. It must not become a compiler, verifier, VM, capability authority, or audit authority.
2. **Visual State Is Not Semantic Truth**: Rendered pixels and presentation models do not define semantic truth.
3. **Renderer Is Not Verifier or Capability Approval**: Renderer output is not verifier admission and cannot approve side effects.
4. **Tooling Is Not Core Contract**: Workbench and Studio are operator interfaces; their internal behaviors do not define core language or runtime contracts.
5. **No Hidden Host Effects**: UI convenience APIs must not bypass PROMETHEUS capability gates or introduce hidden filesystem, network, or OS effects.

---

## 3. Interaction, Local State, and Effects

Native events must be normalized before entering platform-neutral UI contracts. The following distinctions remain mandatory:
- **Native Event is not Semantic Intent**.
- **Hover is not Focus**.
- **Focus is not Selection**.
- **Selection is not Permission**.
- **Action Request is not Effect**.
- **Prepared Effect is not Committed Effect**.

Pointer hover and local layout/presentation updates may remain UI-local. Focus and selection must follow the admitted UI interaction boundaries. Only an interaction that requests a semantic or external action crosses the relevant admission and effect boundary; an external effect then follows the PROMETHEUS route in `CONSTRAINTS.md`.

---

## 4. Visual Architecture Guidance

Visual Doctrine is repository-backed design guidance. The current UI boundary stack, including visual doctrine, tokens, layout primitives, component admission, interaction, actions, effects, renderer presentation, and native/backend boundaries, is owned by [`docs/architecture/ui_boundary_index.md`](../../../docs/architecture/ui_boundary_index.md).

Do not treat an abbreviated visual-doctrine-to-native-backend chain as a complete or exhaustive architectural contract.

---

## 5. Renderer Presentation Model Rules

Renderer presentation layers represent inert, read-only metadata.

### Presentation Inputs and Responsibilities

They may consume the admitted projection/presentation API, node identifiers, trace references, and diagnostic markers to build deterministic display trees. `UiRenderModel` and `UiProjectionArtifact` are verified current public examples, not eternal architectural identities.

### Forbidden Actions

- Must **NOT** execute semantic actions.
- Must **NOT** dispatch unadmitted events.
- Must **NOT** authorize external side effects.
- Must **NOT** rewrite or alter verifier diagnostics.
- Must **NOT** call directly into `sm-verify`, `sm-vm`, or PROMETHEUS gate implementations.

---

## 6. Trace & Audit Visual Projections

- **Visual Projection Is Not Audit Authority**: Visual trace widgets display representations of audit records, while `prom-audit` owns audit and replay record contracts.
- **Explicit State Distinctions**: The UI must distinguish:
  - Error versus Denial;
  - Denial versus Runtime Failure;
  - Quarantine versus Deletion;
  - Conflict State (`S`) versus System Crash.

---

## 7. Stop Conditions

Stop execution and report a blocker immediately if:
- **Compiler/Verifier Migration into UI**: A task attempts to implement parsing, type checking, or verifier admission inside `prom-ui*`, `prom-ui-runtime`, or Workbench.
- **Capability Bypass**: UI code triggers host side effects without the required PROMETHEUS capability/effect route.
- **Audit Authority Violation**: Visual projection state is treated as authoritative proof of execution or audit compliance.
- **Native Handle Leak**: OS window or event handles are introduced into `sm-front`, `sm-sema`, `sm-ir`, `sm-format`, `sm-emit`, `sm-verify`, `sm-runtime-core`, or `sm-vm`.
