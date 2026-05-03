# CTF-4 — SymbolId Migration Registry

Status: draft registry
Parent lane: `core_trust_freeze/index.md`

## Purpose

This file tracks whether names and symbol-like identifiers remain off the runtime hot path where compact IDs are required.

PCC must not accidentally reintroduce string-based execution paths while adding practical language features.

## Registry

| Area | Expected form | Status | Notes |
|---|---|---|---|
| Source identifiers | source text / interned source symbols | audit-needed | Construction-layer concern. |
| Type names | canonical type IDs or stable names before runtime | audit-needed | Must not leak into VM hot path accidentally. |
| Function names in SemCode | stable table / verified references | audit-needed | Confirm current verifier/VM behavior. |
| Runtime symbols | `SymbolId` / runtime symbol table | audit-needed | Must be deterministic. |
| Debug names | append-only debug map | audit-needed | Debug only; not semantic lookup. |
| Record fields | field ID or stable field descriptor | planned | PCC-4. |
| ADT variants | variant ID or stable descriptor | planned | PCC-5. |
| Option / Result variants | stable canonical variant IDs | planned | PCC-6. |
| Collection keys | explicit key policy | planned | PCC-7. |

## Rules

1. Strings may exist for source, diagnostics, and debug rendering.
2. Runtime dispatch must not depend on user-facing strings unless explicitly accepted and documented.
3. Debug names are not semantic identity.
4. Any new practical feature that introduces names must state where the canonical ID is created.
5. If a PR introduces string lookup in VM execution, it must explain why and mark the debt.

## Review checklist

```text
[ ] Does this feature introduce new named entities?
[ ] Are those names resolved before runtime?
[ ] Does VM execution use compact IDs or verified table indexes?
[ ] Are debug names clearly separated from semantic identity?
[ ] Does this require a new registry row?
```
