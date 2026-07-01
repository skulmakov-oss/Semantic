# Quad Surface Lowering Density Audit

Status: docs-only audit note
Owner: language maturity stream
Scope: quad-heavy surface syntax lowering density
Non-goal: optimizer implementation

## 1. Purpose

This document audits whether the proposed quad surface syntax reduces only
visual noise, or also lowers lowering/runtime noise on the current admitted
surface.

The audit is docs-only. It does not approve optimizer implementation, SemCode
format changes, verifier changes, or VM behavior changes.

## 2. Audit Summary

The surface track already improves readability, but the lowering story is mixed.

Observed positive points:

- `is` is a direct predicate alias and lowers to the same equality form as
  `==` against a quad literal.
- `when` lowers to the same canonical nested expression-`if` shape as the
  core form.
- `else if` lowers to nested `else { if ... }` semantics.
- expression-bodied functions lower to the same returned-value block shape as
  an explicit `return`.
- compact `match` over quad or scalar cases keeps the source shorter than a
  manually nested branch chain.

Observed lowering-density risks:

- `match` still emits explicit branch labels, jumps, comparison registers, and
  result staging variables in the IR backend.
- `match` currently uses per-arm `LoadQ` / `LoadI32` plus `CmpEq` / `JmpIf`
  control flow rather than a denser decision-table or jump-table form.
- `match` lowering still carries a dedicated result temporary such as
  `__match_expr_*_result`.
- `when` and `else if` are compact in source, but they do not remove the
  canonical `if`-shaped control-flow cost.
- local inference removes source annotations, not lowering artifacts.
- repeated quad expressions are not common-subexpression eliminated by the
  current surface contract alone.

Audit verdict:

```text
The quad surface track improves source density, but lowering/runtime density
is only partially improved today.
`is`, `when`, and `else if` are thin aliases over existing canonical forms.
`match` is denser than hand-written branching in source, but its current IR
shape still allocates control labels and staging locals.
No optimizer seam is approved by this audit.
```

CTF touched: none
Reason: docs-only audit; no runtime value, trap, determinism, verifier,
capability, or trace change.

## 3. Current Lowering Shape

| Surface form | Current lowering impression | Density outcome |
| --- | --- | --- |
| `x is S` | equality predicate alias | low-noise, already compact |
| `when cond { ... } else { ... }` | nested expression `if` | compact source, same control-flow shape |
| `else if` | nested `else { if ... }` | compact source, same control-flow shape |
| expression-bodied fn | block + final `return` | compact source, same return boundary |
| `match` on quad | compare/jump ladder | compact source, still control-flow heavy |
| `match` on scalar | compare/jump ladder | compact source, still control-flow heavy |
| local inference | source annotations removed | no lowering-density gain by itself |

## 4. Candidate Improvements

The following ideas are candidates only:

1. compact `match` lowering into a more structured decision DAG when profiling
   proves it is worthwhile;
2. remove redundant staging locals for trivial expression-bodied returns when
   the canonical block form can be preserved without extra move noise;
3. consider IR-level CSE for repeated quad expressions rather than in the
   frontend;
4. keep local inference conservative so it remains a source-density feature,
   not a hidden optimization claim.

None of these are approved by this audit.

## 5. Future Profiling Plan

Recommended evidence loop:

1. compile the current quad surface examples through `smc compile`;
2. compare IR instruction counts and emitted SemCode size against the core
   form;
3. keep stable fixture pairs for `if`, `else if`, `when`, `match`, and
   expression-bodied functions;
4. only consider an optimizer or lowering rewrite if the profile repeatedly
   shows a durable reduction in instruction count or staging locals;
5. keep the verifier-first and deterministic execution contracts unchanged.

## 6. Out of Scope

- optimizer implementation;
- SemCode format changes;
- verifier admission changes;
- VM execution changes;
- capability or audit behavior changes;
- P5-A reopening;
- hidden truthiness or hidden coercions.

## 7. Related Docs

- [`docs/roadmap/language_maturity/quad_language_design_roadmap.md`](quad_language_design_roadmap.md)
- [`docs/language/quad_language_design.md`](../../language/quad_language_design.md)
- [`docs/language/quad_surface_syntax_migration.md`](../../language/quad_surface_syntax_migration.md)
- [`docs/language/semantic_language_experience.md`](../../language/semantic_language_experience.md)
