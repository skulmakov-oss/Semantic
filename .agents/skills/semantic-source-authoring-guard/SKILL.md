---
name: semantic-source-authoring-guard
description: Domain guard for authoring Semantic `.sm` source, fixtures, examples, and negative diagnostic probes. Enforces evidence-based source authoring and fail-closed handling of spec-versus-executable drift.
---

# Semantic Source Authoring Guard

Status: repository-native domain guard
Authority: subordinate to [`AGENTS.md`](../../../AGENTS.md), [`CONSTRAINTS.md`](../../../CONSTRAINTS.md), and [`.harness/current.task.yaml`](../../../.harness/current.task.yaml)

---

## 1. Purpose & Scope

This domain guard governs the creation and modification of:
- Semantic `.sm` source files;
- positive and negative test fixtures (`tests/fixtures/**/*.sm`);
- language examples (`examples/**/*.sm`);
- negative diagnostic probes.

### Core Authoring Laws

1. **Never Invent Syntax**: Author only from the source contract and task-relevant executable evidence.
2. **Fixture-First Selection**: Find the closest current fixture or qualification example before authoring new `.sm` code.
3. **No Compiler Widening for Guessed Source**: Never alter compiler, parser, sema, or runtime code merely to make an unconfirmed source form compile.
4. **Diagnostic Integrity**: Negative fixtures must probe deterministic diagnostic boundaries with stable error codes and message fragments.

---

## 2. Authority & Evidence

### Primary Normative Specifications
- [`docs/spec/syntax.md`](../../../docs/spec/syntax.md)
- [`docs/spec/types.md`](../../../docs/spec/types.md)
- [`docs/spec/source_semantics.md`](../../../docs/spec/source_semantics.md)
- [`docs/spec/diagnostics.md`](../../../docs/spec/diagnostics.md)
- [`docs/spec/modules.md`](../../../docs/spec/modules.md)
- [`docs/spec/logos.md`](../../../docs/spec/logos.md)

### Supporting Orientation

[`docs/LANGUAGE.md`](../../../docs/LANGUAGE.md) is a supporting overview. It must stay aligned with the spec bundle and does not define an independent source contract.

### Executable Grounding

Use the closest current qualification test, `tests/fixtures/**/*.sm`, or `examples/**/*.sm` as executable evidence. First determine the layer required by the task: normative source contract, landed current-`main` behavior, qualified limited release, or published stable release.

### Spec vs. Executable Evidence Conflict Rule

If a normative specification and executable fixture/test materially disagree on syntax, type rules, or diagnostics, stop and report contract drift. Do not silently choose either source.

---

## 3. Mandatory Authoring Workflow

```text
1. Identify the target construct and required maturity layer.
        ↓
2. Read the owning normative specification and find the closest executable example.
        ↓
3. If evidence conflicts: STOP and report contract drift.
        ↓
4. If the source form is unconfirmed or malformed: correct the source minimally.
        ↓
5. If the form is admitted by the normative contract but parser, sema, or tests reject it:
   STOP and report implementation/spec drift; do not rewrite valid source to hide it.
        ↓
6. Validate the selected source with the task-relevant command, such as
   cargo run --bin smc -- check <file.sm>.
```

---

## 4. Compact Source Reference

- `assert(condition);` is the statement-level builtin contract form.
- Use only operator/type combinations admitted by the current normative type and source-semantics contracts and supported by current executable evidence.
- Treat feature availability as layer-specific; a form admitted on current `main` is not automatically qualified or published stable.

---

## 5. Forbidden Patterns

Unless explicitly confirmed by the applicable normative contract and executable evidence, do not use:
- invented library functions, pseudo-methods, or syntax;
- implicit type coercions, including `quad -> bool`;
- direct filesystem, network, or OS calls from `.sm` source.

---

## 6. Stop Conditions

Stop execution and report a blocker immediately if:
- **No Confirming Evidence**: The requested language construct has no backing normative or executable evidence.
- **Spec Drift**: A material conflict exists between `docs/spec/*` and fixture/test behavior.
- **Language Widening Required**: Validation would require unauthorized compiler, parser, sema, or runtime changes.
- **Scope Misalignment**: The task is an architectural or compiler change masquerading as a source-only fixture task.
