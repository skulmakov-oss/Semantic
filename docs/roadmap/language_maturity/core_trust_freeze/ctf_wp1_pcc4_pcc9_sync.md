# CTF-WP1 PCC-4..PCC-9 Trust Surface Sync

Status: sync waypoint
Owner: language maturity / execution contract
Scope: CTF impact review after PCC-4..PCC-9 bounded closeout
Non-goal: implementation, release readiness, or CTF closure

## Purpose

This document syncs CTF impact after PCC-4..PCC-9.

PCC-WR1 identified that PCC widened the language surface without automatic CTF sync. This document does not change runtime behavior. This document does not close CTF. This document decides which CTF files need follow-up PRs.

## PCC Surface Summary

| PCC | Surface | Trust impact class | CTF areas to inspect |
| --- | --- | --- | --- |
| PCC-4 | Records | runtime values / diagnostics / determinism | RuntimeValue, traps, verifier, traces |
| PCC-5 | ADT + match | runtime values / control flow / diagnostics | RuntimeValue, traps, verifier, determinism |
| PCC-6 | Option / Result | ADT-like value surface / diagnostics | RuntimeValue, traps, traces |
| PCC-7 | Collections v0 | value surface / traps / determinism / quotas | RuntimeValue, traps, determinism, traces |
| PCC-8 | Stdlib helpers | helper traps / diagnostics / effect boundary | traps, verifier, capability/effect denial |
| PCC-9 | Project Model baseline | manifest/project diagnostics / determinism | determinism, SymbolId/path policy, traces |

## CTF Impact Matrix

| CTF file | Current question | PCC impact | Required action | WP1 verdict |
| --- | --- | --- | --- | --- |
| `runtime_value_registry.md` | Did the runtime value set change? | PCC-4..PCC-9 touched records, ADT, Option/Result, collections, stdlib helper outputs, and project metadata surfaces. | audit | follow-up required |
| `trap_taxonomy.md` | Did failure behavior or trap naming change? | PCC-8D and PCC-9D added deterministic failure surfaces; PCC-7D added collection traps. | audit | sync note added |
| `determinism_matrix.md` | Does repeated execution remain deterministic? | PCC-7 and PCC-9 added container and project-adjacent determinism questions. | audit | follow-up required |
| `symbolid_migration.md` | Did names/symbols/hot-path lookup change? | PCC-4..PCC-9 widened named surfaces and project/package naming questions. | audit | follow-up required |
| `verifier_first_policy.md` | Does public execution still require admission? | PCC fixtures still route through verifier-first paths, but the policy should be reviewed after the new surface set. | audit | sync note added |
| `golden_trace_policy.md` | Are new golden traces required? | PCC fixtures increased the set of stable evidence surfaces, but golden trace policy is not closed by PCC alone. | audit | follow-up required |
| `capability_effect_denial_matrix.md` | Did host/effect/capability behavior change? | PCC-8 and PCC-9 widened docs and helper boundaries, but no capability closure is claimed here. | audit | sync note added |

WP1 verdict allowed values:

- `no change required`
- `sync note added`
- `follow-up required`
- `blocked / evidence missing`

## Expected Audit Conclusions

### RuntimeValue registry

Likely review points:

- records;
- ADT;
- Option / Result;
- Sequence / Map;
- text / stdlib helper outputs;
- project manifest metadata if represented as runtime values.

Do not claim registry complete unless the existing file already supports that claim.

### Trap taxonomy

Likely review points:

- `assert(false)` trap from PCC-8D;
- collection traps from PCC-7D;
- helper misuse diagnostics/traps from PCC-8D;
- project manifest diagnostics from PCC-9D;
- Option/Result diagnostics from PCC-6D;
- ADT/match diagnostics from PCC-5D;
- record diagnostics from PCC-4C.

### Determinism matrix

Likely review points:

- Sequence iteration;
- Map persistent update;
- Map iteration remains open if not closed;
- project manifest parse/order behavior;
- import-resolution determinism;
- module-root policy remains bounded-open.

### SymbolId / hot-path

Likely review points:

- ADT/record/field/constructor names;
- manifest package/module names;
- import alias names;
- debug/rendering names;
- whether any PCC path introduced string-based hot-path behavior.

### Verifier-first policy

Likely review points:

- positive PCC fixtures must still go through check/compile/verify/run where applicable;
- project-model manifest helpers are not execution bypasses;
- stdlib helper behavior must not bypass verifier admission.

### Golden trace policy

Likely review points:

- PCC fixtures may need trace policy mapping;
- current fixture evidence may be enough for PCC closeout but not enough for golden trace freeze;
- decide whether CTF-WP2 should add golden trace mapping.

### Capability/effect denial matrix

Likely review points:

- Stdlib helpers did not widen IO/capability surface;
- `print(text)` may be local/CLI-visible but must not become host effect widening unless explicitly admitted;
- project model did not introduce filesystem/host effects beyond tooling docs;
- debug_render remains internal-only.

## Follow-up Split

If WP1 finds only docs-sync needs:

```text
CTF-WP2 — docs(core-trust-freeze): update runtime value and trap registry after PCC
CTF-WP3 — docs(core-trust-freeze): update determinism and verifier-first evidence after PCC
CTF-WP4 — docs(core-trust-freeze): update golden trace and capability denial policy after PCC
```

If WP1 finds evidence gaps:

```text
CTF-E1 — test(core-trust-freeze): add golden trace coverage for PCC fixture surfaces
CTF-E2 — test(core-trust-freeze): add determinism replay coverage for collections/project baselines
CTF-E3 — test(core-trust-freeze): add trap taxonomy regression fixtures
```

Do not implement these in WP1.

## Governance Decisions

```text
Rule 1:
PCC closeout does not imply CTF freeze.

Rule 2:
Any feature surface with runtime value, trap, verifier, determinism, SymbolId, trace, or capability impact must either update CTF docs or explicitly state why not.

Rule 3:
No release-readiness claim may cite PCC closeout without CTF status.

Rule 4:
Project-root and semantic.toml future work must include CTF notes when implemented.

Rule 5:
Stdlib/collection future widening must include trap/determinism/capability review.
```

## Final WP1 Verdict

```text
CTF-WP1 completes the first trust-surface sync after PCC-4..PCC-9.
It does not close CTF.
It identifies which CTF files are aligned and which require follow-up.
```

## Acceptance Checklist

```markdown
- [ ] PCC-4..PCC-9 trust surfaces reviewed
- [ ] runtime value registry impact reviewed
- [ ] trap taxonomy impact reviewed
- [ ] determinism matrix impact reviewed
- [ ] SymbolId / string hot-path impact reviewed
- [ ] verifier-first policy impact reviewed
- [ ] golden trace policy impact reviewed
- [ ] capability/effect denial impact reviewed
- [ ] follow-up split proposed
- [ ] CTF index points to WP1 sync doc
- [ ] no CTF closure claimed
- [ ] no release readiness claimed
- [ ] no code changed
- [ ] no tests or fixtures changed
```

## CTF-WP2 Follow-up

CTF-WP2 updates:

- runtime value registry after PCC-4..PCC-9;
- trap taxonomy notes after PCC diagnostics / trap fixture closeout.

CTF-WP2 does not close CTF and does not claim release readiness.

## CTF-WP3 Follow-up

CTF-WP3 updates:

- determinism matrix after PCC-4..PCC-9;
- verifier-first policy after PCC feature-surface closeout.

CTF-WP3 does not close CTF and does not claim release readiness.

## CTF-WP4 Follow-up

CTF-WP4 updates:

- golden trace policy after PCC-4..PCC-9;
- capability/effect denial matrix after PCC feature-surface closeout.

CTF-WP4 does not add golden trace artifacts.
CTF-WP4 does not close CTF and does not claim release readiness.

CTF-WR1 reviews the first trust-sync wave after WP1..WP4.

## CTF-WP5 Follow-up

CTF-WP5 defines:

- CTF evidence backlog;
- freeze-candidate promotion rules;
- evidence classes;
- next evidence PR order.

CTF-WP5 does not close CTF and does not claim release readiness.

CTF touched: docs only

Reason: docs-only trust-surface sync; no runtime value, trap, determinism, verifier, SymbolId, capability, or trace change
