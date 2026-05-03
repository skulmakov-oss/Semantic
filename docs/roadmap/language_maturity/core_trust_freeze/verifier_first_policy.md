# CTF-5 — Verifier-First Policy

Status: draft policy
Parent lane: `core_trust_freeze/index.md`

## Purpose

This file protects the public execution route:

```text
SemCode bytes
  → verifier admission
  → verified program
  → VM execution
```

The VM must not become the first and only line of defense for public execution.

## Policy

1. Public `.smc` execution is verifier-first.
2. Any direct VM helper must be internal, test-only, or explicitly documented as non-public.
3. Statically knowable bytecode errors should be verifier rejects, not VM traps.
4. Capability-bearing behavior must be admitted before execution.
5. Debug rendering and disassembly may inspect data, but they must not define language semantics.

## PCC review rules

A PCC PR must update this file if it:

- adds a SemCode section;
- adds or changes opcodes;
- changes verifier admission;
- changes VM entrypoints;
- changes capability-gated behavior;
- adds debug or trace output that could be confused with language output.

## Internal tooling boundary

`debug_render` is allowed only as internal tooling.

```text
debug_render != to_text
debug_render != public output API
debug_render != stdlib conversion
debug_render must not be used by canonical examples
```

`to_text` belongs to PCC-8 Stdlib v0 and must have a public type contract.

## Review checklist

```text
[ ] Does the public route still require verifier admission?
[ ] Did this PR add a VM bypass?
[ ] If yes, is it test-only or explicitly non-public?
[ ] Did this PR add a SemCode feature?
[ ] Does sm-verify know about it?
[ ] Does 7hell verify before run?
[ ] Are debug helpers kept out of the language surface?
```
