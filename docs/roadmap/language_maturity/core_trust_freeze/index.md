# Core Trust Freeze Lane

Status: draft control lane
Owner: language maturity / execution contract
Parent plan: `docs/roadmap/language_maturity/practical_core_completion_v0_3.md`

## Purpose

Core Trust Freeze (CTF) is the parallel trust lane for Practical Core Completion.

CTF exists because each PCC feature can change execution semantics. Runtime values, traps, verifier expectations, SymbolId assumptions, trace outputs, capability behavior, or golden traces must not drift silently while the language surface widens.

CTF is not a final phase after PCC. It runs across PCC.

## Waypoints

- Current sync waypoint: `docs/roadmap/language_maturity/core_trust_freeze/ctf_wp1_pcc4_pcc9_sync.md`
- Runtime / trap follow-up: `CTF-WP2 — RuntimeValue and trap registry sync after PCC`
- Determinism / verifier-first follow-up: `CTF-WP3 — determinism and verifier-first policy sync after PCC`
- Golden trace / capability follow-up: `CTF-WP4 — golden trace and capability/effect denial policy sync after PCC`
- Golden trace evidence: `CTF-E1 — selected PCC golden trace coverage`
- Collection replay evidence: `CTF-E2 — collection determinism replay coverage`
- Trap taxonomy evidence: `CTF-E3 — trap taxonomy regression coverage`
- Capability denial replay evidence: `CTF-E4 — denied-effect replay coverage`
- Waypoint review: `docs/roadmap/language_maturity/core_trust_freeze/ctf_waypoint_review_after_wp1_wp4.md`
- Evidence backlog: `docs/roadmap/language_maturity/core_trust_freeze/ctf_evidence_backlog.md`
- Promotion rules: `docs/roadmap/language_maturity/core_trust_freeze/freeze_candidate_promotion_rules.md`
- Evidence backlog / promotion waypoint: `CTF-WP5 — define CTF evidence backlog and freeze-candidate promotion rules`
- Project-root trust policy: `docs/roadmap/language_maturity/core_trust_freeze/project_root_trust_policy.md`
- Project-root trust waypoint: `CTF-WP6 — docs(core-trust-freeze): define project-root trust policy before PCC-9I`
- 7hell report contract: `docs/roadmap/language_maturity/7hell_report_contract.md`
- 7hell Diagnostics Hell boundary decision: `docs/roadmap/language_maturity/7hell_diagnostics_boundary_decision.md`
- 7hell Diagnostics Hell seam audit: `reports/7hell_diag_report_quality_seam_audit.md`
- 7hell PCC stage mapping: `docs/roadmap/language_maturity/7hell_pcc4_pcc9_stage_mapping.md`
- 7hell initial fixture selection: `docs/roadmap/language_maturity/7hell_initial_fixture_selection.md`
- PCC waypoint review: `docs/roadmap/language_maturity/pcc_waypoint_review_after_pcc4_pcc9.md`

## Files

| File | Owner question |
|---|---|
| `runtime_value_registry.md` | Did the runtime value set change? |
| `trap_taxonomy.md` | Did failure behavior or trap naming change? |
| `determinism_matrix.md` | Does repeated execution remain deterministic? |
| `symbolid_migration.md` | Did the change affect names, symbols, or hot-path lookup? |
| `verifier_first_policy.md` | Does public execution still require admission before VM run? |
| `golden_trace_policy.md` | Does the change require new or updated golden traces? |
| `capability_effect_denial_matrix.md` | Did host/effect/capability behavior change? |
| `project_root_trust_policy.md` | Does future project-root support preserve verifier-first trust? |
| `7hell_report_contract.md` | Does future 7hell reporting remain stable and deterministic? |
| `7hell_diagnostics_boundary_decision.md` | Has Diagnostics Hell been classified without changing execution behavior? |
| `7hell_diag_report_quality_seam_audit.md` | Has the future Diagnostics Hell report-quality seam been located safely? |
| `capability_denial_replay.md` | Has denied-effect replay evidence been planned without widening capability behavior? |

## PR requirement

Every PCC PR must include a CTF note.

Use this form when execution trust files changed:

```text
CTF touched:
  - docs/roadmap/language_maturity/core_trust_freeze/runtime_value_registry.md
  - docs/roadmap/language_maturity/core_trust_freeze/trap_taxonomy.md
Reason:
  New RuntimeValue and trap behavior introduced by feature X.
```

Use this form when there is no trust-surface impact:

```text
CTF touched: none
Reason: docs-only / parser-only / no runtime impact
```

## Minimum review checklist

```text
[ ] RuntimeValue set reviewed
[ ] Trap taxonomy reviewed
[ ] Determinism matrix reviewed
[ ] SymbolId / string hot-path impact reviewed
[ ] Verifier-first path preserved
[ ] Golden trace need reviewed
[ ] Capability/effect denial impact reviewed
```

## Freeze rule

A CTF entry marked `freeze-candidate` must not change without:

1. a reason;
2. a linked PCC phase or issue;
3. updated fixtures or tests where behavior changes;
4. explicit release-status wording if public claims are affected.
