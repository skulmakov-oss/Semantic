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
| `to_text` / formatting | bounded admitted surface | PCC-8 / CTF | Admitted helper scope only; not universal reflection. |
| `assert` failure | admitted trap / diagnostic surface | PCC-8 / CTF | Not a host effect. |
| `print(text)` | admitted helper output | PCC-8 / CTF | CLI-visible output; must not become unreviewed host IO/capability widening. |
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

## CTF-WP4 PCC-4..PCC-9 Capability / Effect Denial Sync

PCC-4..PCC-7 are pure language / value surfaces and do not add host effects.

PCC-8 stdlib helper surface remains bounded:

- `assert` is a trap / diagnostic behavior, not a host effect;
- `print(text)` is admitted helper output but must not become unreviewed host IO / capability widening;
- `to_text` is admitted-types-only conversion, not reflection;
- `debug_render` remains internal-only tooling;
- unsupported `to_text` remains rejected.

PCC-9 project model baseline is tooling / project-adjacent documentation and fixture evidence, not runtime host effect expansion.

No package registry, dependency resolver, remote packages, network IO, filesystem capability, or host ABI behavior is introduced by PCC-9 closeout.

Local audit is not telemetry.

No capability matrix entry is promoted to full release-frozen status by this PR.

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

## CTF-WP6 Project-Root Capability Notes

CTF-WP6 states the boundary for future project-root support before PCC-9I implementation.

Reading admitted project files is tooling input, not language host capability.

Project-root support must not introduce network IO, registry access, remote package fetch, or telemetry.

No project-root capability widening is added by WP6.
