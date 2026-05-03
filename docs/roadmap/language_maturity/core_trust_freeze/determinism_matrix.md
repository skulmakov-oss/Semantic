# CTF-3 — Determinism Matrix

Status: draft matrix
Parent lane: `core_trust_freeze/index.md`

## Purpose

This file tracks where Semantic must prove repeated behavior is stable for the same input, config, and admitted program.

Determinism is not a slogan. Each PCC feature must add or reuse concrete fixtures.

## Matrix

| Area | Required deterministic property | Status | PCC owner |
|---|---|---|---|
| Lexer / parser | Same source produces same syntax result / diagnostic | audit-needed | PCC-0.5 |
| Typecheck | Same source produces same type result / diagnostic ordering | audit-needed | PCC-0.5 |
| Lowering | Same typed source produces same IR | audit-needed | PCC-0.5 |
| IR passes | Same IR produces same optimized IR | audit-needed | PCC-0.5 / PCC-2+ |
| SemCode emission | Same IR/options produce same bytes | audit-needed | PCC-0.5 |
| Verifier | Same SemCode/config produces same accept/reject | audit-needed | CTF |
| VM execution | Same verified program/config produces same result/trap | audit-needed | CTF |
| Control flow | Loop execution is stable under same fuel/config | planned | PCC-1 |
| Numeric behavior | Arithmetic result/trap is stable | planned | PCC-2 |
| Text behavior | Text ops result/trap is stable | planned | PCC-3 |
| Records | Field order, access, and value behavior are stable | planned | PCC-4 |
| ADT + match | Variant order and match behavior are stable | planned | PCC-5 |
| Option / Result | Helper and match behavior are stable | planned | PCC-6 |
| Collections | Iteration and lookup ordering are stable | planned | PCC-7 |
| Stdlib | Helper behavior is stable | planned | PCC-8 |
| Project model | Module root and project check order are stable | planned | PCC-9 |

## Minimum repeated-run rule

For each feature phase:

```text
same input
same config
same environment assumptions
  → same check result
  → same emitted SemCode where emission is involved
  → same verifier result
  → same VM result/trap where execution is involved
```

## 7hell coupling

Each 7hell stage should eventually emit deterministic machine-readable output.

Minimum future output fields:

```text
stage
status
input_hash
output_hash
trace_hash, if applicable
diagnostics_hash, if applicable
```

## Review checklist

```text
[ ] Does this PR add a new nondeterminism source?
[ ] Does map/set iteration order affect output?
[ ] Does diagnostic ordering stay stable?
[ ] Does emitted byte order stay stable?
[ ] Does VM result/trap stay stable under repeated run?
[ ] Does 7hell need a deterministic fixture?
```
