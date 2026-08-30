---
name: semantic-ui-boundary-guard
description: Domain guard for Semantic UI orchestration, renderer presentation models, interaction semantics, trace/audit visual projections, and native backend facades. Enforces strict separation between UI presentation and Semantic core/runtime authority.
---

# Semantic UI Boundary Guard

Status: repository-native domain guard
Authority: subordinate to [`AGENTS.md`](../../AGENTS.md), [`CONSTRAINTS.md`](../../CONSTRAINTS.md), and [`.harness/current.task.yaml`](../../.harness/current.task.yaml)

---

## 1. Purpose & Domain Scope

This domain guard governs:
- **`prom-ui*`**: UI contract vocabulary, visual boundary types, and trace/audit projection contracts;
- **`prom-ui-runtime`**: Platform-neutral UI runtime orchestration;
- **`prom-ui-backend-native`**: Native backend facade and platform event bridging;
- **`examples/workbench_semantic`**: Canonical native Semantic Workbench (developer & operator tooling surface);
- Presentation models, layout solving, interaction pipelines, and visual projections.

---

## 2. Fundamental UI Invariants

### A. Non-Authority Doctrine
1. **Presentation, Not Compiler**: Semantic UI is an operator/presentation layer over admitted contracts. It must never become a compiler, verifier, VM, capability authority, or audit authority.
2. **Visual State $\neq$ Semantic Truth**: Visual state, rendered pixels, and presentation models do not define semantic truth.
3. **Renderer $\neq$ Verifier / Capability Approval**: Renderer output is not verifier admission and cannot approve side effects.
4. **Tooling $\neq$ Core Contract**: Workbench and Studio are operator interfaces; their internal behaviors do not define core Semantic language or runtime contracts.
5. **No Hidden Host Effects**: UI convenience APIs must never bypass PROMETHEUS capability gates or introduce hidden filesystem, network, or OS effects.

---

## 3. Strict Interaction & Effect Separation

All user and platform interactions must follow the explicit staged pipeline:

```text
Native Event (OS/Window)
        ↓
Normalized Input Signal
        ↓
Interaction Intent
        ↓
Focus / Selection Context
        ↓
Semantic Action Request
        ↓
Admission & Policy Evaluation
        ↓
Effect Request
        ↓
PROMETHEUS Capability, Budget & Audit Gate
        ↓
Committed or Denied Effect
```

### Critical Interaction Rules
- **Native Event is NOT Semantic Intent**: Raw pointer or keyboard events must be explicitly mapped to typed intents.
- **Hover is NOT Focus**: Transient hover state carries no selection or action authority.
- **Focus is NOT Selection**: Focused elements do not imply active selection.
- **Selection is NOT Permission**: Selected items do not have implicit capability authorization.
- **Action is NOT Effect**: Triggering an action request does not bypass capability evaluation.
- **Prepared Effect is NOT Committed Effect**: Staged UI effects remain inert until authorized and executed through the PROMETHEUS boundary.

---

## 4. Visual Architecture & Layering

Preserve the strict dependency hierarchy:
$$\text{Visual Doctrine (Meaning)} \longrightarrow \text{Design Tokens} \longrightarrow \text{Layout Primitives} \longrightarrow \text{Component System} \longrightarrow \text{Renderer} \longrightarrow \text{Native Backend}$$

- **Visual Doctrine**: Defines visual meaning and state representations.
- **Tokens**: Reusable design values (colors, spacing, typography).
- **Layout**: Spatial grammar and bounding constraints.
- **Components**: Reusable composite UI widgets.
- **Renderer**: Consumes admitted visual/layout models; builds inert presentation data.
- **Native Backend**: Bridges platform events and manages native OS windowing handles without capturing runtime core ownership.

---

## 5. Renderer Presentation Model Rules

Renderer presentation layers represent inert, read-only metadata:
- **Allowed Capabilities**: May consume `UiRenderModel`, `UiProjectionArtifact`, node IDs, trace references, and diagnostic markers to build deterministic display trees.
- **Forbidden Actions**:
  - Must **NOT** execute semantic actions.
  - Must **NOT** dispatch unadmitted events.
  - Must **NOT** authorize external side effects.
  - Must **NOT** rewrite or alter verifier diagnostics.
  - Must **NOT** call directly into `sm-verify`, `sm-vm`, or PROMETHEUS gate implementations.

---

## 6. Trace & Audit Visual Projections

- **Visual Projection $\neq$ Audit Authority**: Visual trace widgets display representations of audit records, but `prom-audit` is the sole authority for trace validity and replay integrity.
- **Explicit State Distinctions**: The UI must distinctly present:
  - Error vs. Denial;
  - Denial vs. Runtime Failure;
  - Quarantine vs. Deletion;
  - Conflict State (`S`) vs. System Crash.

---

## 7. Stop Conditions

Stop execution and report a blocker immediately if:
- **Compiler/Verifier Migration into UI**: A task attempts to implement parsing, type checking, or verifier admission inside `prom-ui*`, `prom-ui-runtime`, or Workbench.
- **Capability Bypass**: UI code attempts to trigger host side effects without routing through the PROMETHEUS capability gate pipeline.
- **Audit Authority Violation**: Visual projection state is treated as authoritative proof of execution or audit compliance.
- **Native Handle Leak**: OS window or event handles are leaked into deterministic Semantic core libraries (`sm-*`).
