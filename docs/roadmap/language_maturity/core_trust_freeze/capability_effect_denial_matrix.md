# CTF-7 — Capability / Effect Denial Matrix

Status: draft matrix
Parent lane: `core_trust_freeze/index.md`

## Purpose

This file protects the boundary between deterministic VM execution and host effects.

PCC is mainly a practical language-core phase. It should not silently widen host capabilities, effect behavior, or PROMETHEUS runtime access.

## Matrix

| Effect / boundary class | Status in PCC | Owner | Notes |
|---|---|---|---|
| Pure computation | allowed | sm-* | Must remain deterministic. |
| Debug / trace output | internal only unless scoped | smc-cli / CTF | Must not become language output accidentally. |
| `to_text` / formatting | planned | PCC-8 | Public stdlib, not debug_render. |
| Assertion failure | planned | PCC-8 | Trap / diagnostic policy required. |
| File IO | out-of-pcc | future capability scope | Not part of practical core. |
| Network IO | out-of-pcc | future capability scope | Not part of practical core. |
| Host gate read | out-of-pcc unless already stable | prom-abi / prom-cap | No widening in PCC. |
| Host gate write | out-of-pcc unless already stable | prom-abi / prom-cap | No widening in PCC. |
| Pulse emit | out-of-pcc unless already stable | prom-abi / prom-cap | No widening in PCC. |
| UI event/frame effects | out-of-pcc | UI boundary track | Workbench / UI dessert track. |
| Audit emission | internal runtime boundary | prom-audit | Not a public language output channel. |

## Rules

1. PCC features should default to pure deterministic execution.
2. Any host interaction must be explicit capability work and probably out-of-PCC.
3. Debug output must stay separate from user-facing language output.
4. Local audit is not telemetry and not public language output.
5. Capability denial must be deterministic and reportable.
6. A denied effect must not partially mutate host state.

## PR review checklist

```text
[ ] Does this PR introduce an effect?
[ ] If yes, is it pure, debug-only, stdlib output, or host boundary?
[ ] Does it require capability admission?
[ ] Does verifier know about the capability?
[ ] Is denial deterministic?
[ ] Is host state unchanged on denial?
[ ] Is this actually out-of-PCC?
```
