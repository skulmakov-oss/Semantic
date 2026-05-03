# CTF-2 — Trap Taxonomy

Status: draft taxonomy
Parent lane: `core_trust_freeze/index.md`

## Purpose

This file records the stable categories of execution failure that can appear after verifier admission or during VM/runtime execution.

The goal is not to name every internal error. The goal is to prevent feature work from introducing untracked failure classes.

## Draft taxonomy

| Trap class | Status | Typical owner | Notes |
|---|---|---|---|
| Invalid bytecode / malformed SemCode | freeze-candidate | sm-verify | Should be rejected before VM execution. |
| Unknown opcode | freeze-candidate | sm-verify / sm-vm | Public route should reject before dispatch. |
| Invalid jump target | freeze-candidate | sm-verify | Must not become VM-only failure. |
| Call target missing | freeze-candidate | sm-verify | Builtin exception policy must be explicit. |
| Register budget exceeded | freeze-candidate | sm-verify / sm-runtime-core | Runtime quota coupling. |
| Step fuel exceeded | freeze-candidate | sm-runtime-core / sm-vm | Deterministic bounded execution. |
| Stack / frame budget exceeded | audit-needed | sm-runtime-core / sm-vm | Confirm current behavior. |
| Numeric invalid operation | planned | PCC-2 | Division / overflow / unsupported numeric op policy. |
| Text invalid operation | planned | PCC-3 | Bounds / invalid concat / invalid length policy if applicable. |
| Record field error | planned | PCC-4 | Missing field / invalid projection policy. |
| ADT match error | planned | PCC-5 | Non-exhaustive / invalid variant behavior. |
| Option / Result misuse | planned | PCC-6 | Payload access and helper behavior. |
| Collection bounds / missing key | planned | PCC-7 | Sequence index and map lookup behavior. |
| Assertion failure | planned | PCC-8 | Stdlib assert behavior. |
| Capability denied | audit-needed | CTF-7 | Must stay separate from raw VM failure. |
| Host ABI denied / failed | out-of-pcc unless boundary touched | separate boundary scope | Not part of practical core unless explicitly changed. |

## Rules

1. Public execution should prefer verifier rejection before VM trap when the error is statically knowable.
2. VM traps must be deterministic for the same verified program and execution config.
3. Trap names must not be reused for semantically different failures.
4. PCC PRs that add a new failure mode must update this file.
5. Trap output must not depend on `debug_render` as a language feature.

## Review checklist

```text
[ ] Is this error statically rejectable by verifier?
[ ] If runtime-only, is the trap deterministic?
[ ] Is the trap class already represented here?
[ ] Does 7hell need a fixture for it?
[ ] Does the golden trace policy need an update?
```
