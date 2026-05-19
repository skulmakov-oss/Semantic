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

## CTF-WP3 PCC-4..PCC-9 Verifier-First Sync

PCC positive execution fixtures must continue to pass through check/compile/verify/run-smc where applicable.

Manifest/project helper tests are not public execution paths and must not be used as verifier bypass evidence.

Stdlib helper behavior must not create a host/capability bypass.

`print(text)` must not become an unreviewed host effect.

`debug_render` remains internal-only.

`to_text` remains admitted-types-only and must not become reflection.

Project-root check/run, when implemented, must enter the same verifier-first route:

```text
project source -> check -> compile -> verify -> run
```

`smc new` must not create executable trust without check/verify.

Any future SemCode/opcode/helper expansion must update verifier-first policy or state why not.

| Surface | Verifier-first implication | WP3 status |
| --- | --- | --- |
| Records | execution fixtures must verify before run | sync note |
| ADT / match | lowering / branching must remain admitted before VM run | sync note |
| Option / Result | standard forms do not bypass verifier | sync note |
| Collections | collection ops / traps must remain verified execution behavior | sync note |
| Stdlib helpers | helper calls must not widen host effects | sync note |
| Project model | helper / manifest tests are not verifier bypass evidence | sync note |
| Project-root future commands | must use verifier-first route | follow-up required |

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
