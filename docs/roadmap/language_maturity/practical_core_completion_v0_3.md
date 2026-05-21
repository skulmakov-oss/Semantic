# Practical Core Completion v0.3

Status: draft planning gate
Owner: language maturity stream
Scope: Semantic practical language core before readiness completion
Non-goal: release claim widening

## 0. Purpose

Practical Core Completion (PCC) is the intermediate completion phase between the current `main` development line and any later readiness-completion / release-qualification plan.

PCC exists because Semantic already has a strong staged compiler/runtime architecture, verifier-first execution, and PROMETHEUS boundary discipline, but the practical language surface still needs a controlled completion phase before the project can honestly enter final readiness.

PCC v0.3 freezes the operational shape of that phase.

```text
Current Semantic
  ↓
PCC-0 Truth Reset
  ↓
Practical Core Completion phases
  ↕
Core Trust Freeze lane
  ↕
7hell progressive qualification
  ↓
Readiness Completion
  ↓
Release Qualification
```

## 1. Goal

Bring Semantic to the point where small and medium programs can be written without relying on implementation accidents, bypasses, or knowledge of current frontend / VM gaps.

The practical target is:

```text
Practical Semantic =
  control flow
+ usable numbers
+ text
+ records
+ ADT + match
+ Option / Result
+ collections v0
+ minimal stdlib
+ project model v0
+ verified deterministic execution
+ canonical examples
+ 7hell qualification
```

## 2. Non-goals

PCC does not include:

- Workbench implementation;
- UI application capability;
- graphics / rendering;
- browser or mobile targets;
- IDE / LSP as a blocker;
- macro system;
- async / concurrency;
- package registry;
- broad generics / traits beyond readiness need;
- LLM / TinyLM research;
- GPU / Metal / AVX-512 backend work as required path;
- quad hardware accelerator research;
- PROMETHEUS runtime widening without explicit separate scope;
- visual architecture experiments.

Rule:

```text
If a task does not move PCC-0..PCC-9, CTF, or 7hell qualification,
it does not belong to PCC.
```

## 3. Phase map

```text
PCC-0    Truth Reset
PCC-0.5  Feature Matrix Live Audit
PCC-0.6  7hell Skeleton Seed
CTF-0    Core Trust Freeze directory

PCC-1    Control Flow Core
PCC-2    Numeric Core
PCC-3    Text Core
PCC-3.5  Data Carrier Design Note
PCC-4    Records End-to-End
PCC-5    ADT + Basic Match
PCC-6    Option / Result
PCC-7    Collections v0
PCC-8    Stdlib v0
PCC-9    Project Model v0
```

Parallel lanes:

```text
Core Trust Freeze lane
7hell progressive qualification
monthly waypoint review
strict out-of-scope enforcement
```

## 4. PCC-0 — Truth Reset

Purpose:

- stop treating optimistic readiness language as current-state truth;
- separate practical-core completion from readiness completion;
- mark post-PCC work explicitly;
- prevent Workbench / UI / research creep.

Required outcomes:

- readiness-completion plan is marked as post-PCC;
- Wave 2 wording is aligned with practical-core completion and 7hell seed work;
- Workbench remains in dessert track / post-PCC planning;
- global out-of-scope block is visible;
- PCC-0.5 and PCC-0.6 are opened before PCC-1.

DoD:

```text
[ ] current-state wording no longer overclaims trusted readiness
[ ] post-PCC readiness boundary is explicit
[ ] Workbench / UI creep is excluded from PCC
[ ] feature matrix audit is scheduled
[ ] CTF directory exists
[ ] 7hell skeleton task exists
```

## 5. PCC-0.5 — Feature Matrix Live Audit

Purpose:

Verify every uncertain feature status against current `main` before starting new feature work.

Every status such as:

```text
confirmed / partial
landed but needs audit
closed but needs live check
✅ / 🟡
```

must resolve into one of three states:

| Result | Meaning |
|---|---|
| Confirmed Stable | Has test / golden / PR evidence and can be marked stable for PCC planning. |
| Confirmed Partial | Exists, but the missing edge is explicit. |
| Downgraded | Assumption did not survive live audit; feature moves to planned work. |

DoD:

```text
[ ] no dual-status item remains unresolved
[ ] each stable item has test / fixture / PR evidence
[ ] each partial item names its missing edge
[ ] each missing item is assigned to a PCC phase or excluded
```

## 6. PCC-0.6 — 7hell Skeleton Seed

Purpose:

Create `7hell` early as a diagnostic harness that grows with the language, rather than a final retrospective test.

Initial command shape:

```bash
smc 7hell program.sm
smc seven-hell program.sm
```

Initial stages:

1. Syntax Hell
2. Type Hell
3. Lowering Hell
4. Verifier Hell
5. VM Hell
6. Practical Hell
7. User Pain / Diagnostics Hell

Seed behavior:

- command exists;
- stage report exists;
- JSON output exists;
- shallow checks are acceptable at first;
- each PCC phase adds its fixtures to the relevant stage.

DoD:

```text
[ ] `smc 7hell` command or issue exists
[ ] stage taxonomy is documented
[ ] output contract is drafted
[ ] first fixture group is attached to PCC-1
```

Qualification report contract:

- `docs/roadmap/language_maturity/7hell_report_contract.md`

7HELL-WP2 maps PCC-4..PCC-9 evidence into the 7hell stage model; this is a docs-only mapping and stage execution remains future work.
7HELL-WP3 defines skeleton-to-runner transition rules before S2 implementation.

## 7. CTF-0 — Core Trust Freeze directory

Core Trust Freeze is not a post-PCC phase. It is a parallel lane.

Each language expansion can affect runtime values, traps, verifier rules, determinism, SymbolId assumptions, trace outputs, or capability denial behavior. Therefore each PCC PR must explicitly state whether it touches CTF material.

Directory:

```text
docs/roadmap/language_maturity/core_trust_freeze/
  index.md
  runtime_value_registry.md
  trap_taxonomy.md
  determinism_matrix.md
  symbolid_migration.md
  verifier_first_policy.md
  golden_trace_policy.md
  capability_effect_denial_matrix.md
```

PR rule:

```text
CTF touched:
  - runtime_value_registry.md
  - trap_taxonomy.md
  - determinism_matrix.md
```

or:

```text
CTF touched: none
Reason: docs-only / parser-only / no runtime impact
```

## 8. PCC-1 — Control Flow Core

Scope:

- `while`;
- statement `loop`;
- `break`;
- `continue`;
- return-path and terminal CFG consistency;
- diagnostics for invalid control exits.

Feature DoD:

```text
[ ] loops parse
[ ] loops typecheck
[ ] loops lower to deterministic IR
[ ] verifier accepts valid emitted SemCode
[ ] VM executes canonical fixtures
[ ] invalid exits produce stable diagnostics
```

Trust DoD:

```text
[ ] new IR / opcode shape documented if introduced
[ ] trap taxonomy updated if needed
[ ] determinism matrix gets loop fixtures
[ ] 7hell stages include control-flow fixtures
```

## 9. PCC-2 — Numeric Core

Scope:

- complete practical arithmetic for admitted numeric families;
- relationals and equality consistency;
- division / invalid operation policy;
- numeric traps;
- deterministic behavior fixtures.

Minimum surface:

```text
i32 / u32 / f64 / fx as applicable to current admitted profile
+ - * / where admitted
relations
safe failure behavior
```

Trust DoD:

```text
[ ] numeric RuntimeValue entries are frozen or marked draft
[ ] numeric trap policy is documented
[ ] determinism matrix includes arithmetic fixtures
[ ] verifier rules and VM behavior match
```

## 10. PCC-3 — Text Core

Pre-entry surface boundary reset:
`docs/roadmap/language_maturity/pcc3_text_surface_boundary_reset.md`

Scope:

- text literal;
- equality;
- concat;
- length;
- runtime carrier.

Explicitly not included in PCC-3:

```text
to_text(...)
format(...)
user-facing debug conversion
```

`to_text` belongs to PCC-8 Stdlib v0.

Internal tooling rule:

```text
debug_render != to_text
debug_render is not part of the language
debug_render is not used in canonical examples
debug_render is not exported as public stdlib API
```

## 11. PCC-3.5 — Data Carrier Design Note

Purpose:

Freeze the value / reference boundary before records and collections diverge.

Minimum decision:

| Type family | PCC v0 policy |
|---|---|
| `record` | value semantics |
| record assignment | copy / move-by-value, no hidden reference |
| `Sequence<T>` | runtime-managed dynamic container |
| `Map<K,V>` | runtime-managed dynamic container |
| collection assignment | explicit policy required before implementation |
| collection mutation | only through defined stdlib/runtime operations |

Rule:

```text
records are not heap containers
collections are not records
```

Out of scope:

- borrowed collections;
- shared mutable collections;
- persistent collections;
- copy-on-write;
- GC policy;
- advanced ownership.

## 12. PCC-4 — Records End-to-End

Scope:

- declaration;
- construction;
- field access;
- assignment semantics according to PCC-3.5;
- lowering;
- runtime carrier;
- verifier / VM path;
- diagnostics.

Live audit:

- `docs/roadmap/language_maturity/pcc4_records_live_audit.md`

Closeout note:

```text
PCC-4A/B/C/D evidence is complete for the current Practical Core scope.
Positive and negative fixture coverage exists.
PCC-4 is closed for current scope.
Further aggregate work moves to PCC-5 ADT + Basic Match, then PCC-6 Option / Result, PCC-7 Collections v0, and PCC-8 Stdlib v0.
```

DoD:

```text
[ ] record examples pass check → compile → verify → run-smc
[ ] invalid field access is diagnosed
[ ] record value semantics are tested
[ ] CTF registry is updated where runtime values change
```

## 13. PCC-5 — ADT + Basic Match

Scope:

- ADT declarations;
- constructors;
- payload access where admitted;
- basic `match` over ADT;
- initial exhaustiveness policy or explicit non-exhaustive limitation;
- lowering and VM path.

Live audit:

- `docs/roadmap/language_maturity/pcc5_adt_match_live_audit.md`

PCC-5 boundary note:

```text
records are closed as nominal value aggregates;
ADT is a separate aggregate family;
collections are not records;
host ABI stays closed to record values.
```

DoD:

```text
[ ] constructor fixtures pass full pipeline
[ ] match fixtures pass full pipeline
[ ] non-supported match shapes fail cleanly
[ ] verifier and VM agree on ADT carrier rules
```

PCC-5 closeout note:

```text
PCC-5A/B/C/D/E evidence chain is complete for the current Practical Core scope.
Positive ADT declaration / constructor fixtures exist.
Positive basic ADT match fixtures exist.
Negative ADT/match diagnostics fixtures exist.
PCC-5 is closed for the current Practical Core scope.
Future aggregate work moves to PCC-6 Option / Result, PCC-7 Collections v0,
and PCC-8 Stdlib v0.
```

## 14. PCC-6 — Option / Result

Scope:

- `Option(T)`;
- `Result(T, E)`;
- constructors;
- match helpers;
- minimal helper functions if needed for examples.

Live audit:

- `docs/roadmap/language_maturity/pcc6_option_result_live_audit.md`

DoD:

```text
[ ] Option fixtures pass full pipeline
[ ] Result fixtures pass full pipeline
[ ] canonical failure-flow examples exist
[ ] diagnostics for invalid payload usage are stable
```

PCC-6 closeout note:

- PCC-6 is closed for the current Practical Core scope.
- Evidence lives in `pcc6_option_result_live_audit.md`.
- Dedicated test suites now cover positive Option standard-form fixtures,
  positive Result standard-form fixtures, and negative Option / Result
  diagnostics.
- Further aggregate work remains assigned to later PCC phases:
  - `PCC-7` Collections v0
  - `PCC-8` Stdlib v0
  - `PCC-9` Project Model v0

## 15. PCC-7 — Collections v0

Scope:

- `Sequence<T>` minimum operations;
- `Map<K,V>` minimum operations if admitted;
- deterministic iteration policy;
- bounds / missing-key behavior;
- memory / quota interaction.

DoD:

```text
[ ] collections have explicit carrier policy
[ ] index / iteration behavior is deterministic
[ ] failure behavior is trap-or-diagnostic stable
[ ] 7hell includes practical collection fixtures
```

Live audit:

- `docs/roadmap/language_maturity/pcc7_collections_live_audit.md`

PCC-7 closeout note:

```text
PCC-7 is closed for the current Practical Core fixture-backed scope.
Evidence lives in `pcc7_collections_live_audit.md`.
Dedicated test suites now cover positive Sequence<T> fixtures, positive
Map<K,V> fixtures, and negative collection diagnostics / trap fixtures.
Bounded open items remain:
- Map missing-key policy;
- Map iteration policy;
- assignment / aliasing policy;
- memory / quota evidence.
Further work remains assigned to later phases or explicit policy tracks:
- `PCC-8` Stdlib v0;
- `PCC-9` Project Model v0;
- optional future collection policy track if needed.
```

Bounded scope note:

```text
PCC-7 closeout does not claim Map missing-key completeness, Map iteration
completeness, assignment / aliasing completeness, or memory / quota
completeness.
Those remain bounded policy or future-work items.
```

## 16. PCC-8 — Stdlib v0

Scope:

- `assert`;
- math helpers;
- text helpers;
- `to_text` for admitted basic types;
- sequence helpers;
- map helpers;
- Option / Result helpers.

Rule:

```text
debug_render remains internal tooling and cannot substitute for to_text.
```

DoD:

```text
[ ] public helper list is documented
[ ] each helper has type contract
[ ] helper failures are diagnostic/trap stable
[ ] canonical examples avoid internal debug helpers
```

Live audit:

- `docs/roadmap/language_maturity/pcc8_stdlib_live_audit.md`
- `docs/roadmap/language_maturity/pcc8_stdlib_public_contract.md`

PCC-8 is closed for the current admitted Stdlib v0 helper surface.
Evidence lives in `pcc8_stdlib_live_audit.md`.
The public helper boundary lives in `pcc8_stdlib_public_contract.md`.
Dedicated test suites now cover positive basic helper fixtures and helper
diagnostics / runtime traps.
Bounded open items remain:

- `std.math`;
- broad stdlib expansion;
- universal reflection / broad `to_text`;
- public `debug_render`;
- formatting macros;
- IO/capability expansion.

Further practical-core work moves to PCC-9 Project Model v0.

## 17. PCC-9 — Project Model v0

Scope:

- `semantic.toml`;
- `src/main.sm`;
- `smc new` if admitted;
- `smc check` project;
- `smc run` project;
- deterministic module roots;
- project-level 7hell.

Roadmap artifacts:

- live audit: `docs/roadmap/language_maturity/pcc9_project_model_live_audit.md`
- contract freeze: `docs/roadmap/language_maturity/pcc9_project_model_contract.md`
- waypoint review: `docs/roadmap/language_maturity/pcc_waypoint_review_after_pcc4_pcc9.md`
- CTF sync waypoint: `docs/roadmap/language_maturity/core_trust_freeze/ctf_wp1_pcc4_pcc9_sync.md`
- CTF follow-up: `CTF-WP2 — docs(core-trust-freeze): update runtime value and trap registry after PCC`
- CTF follow-up: `CTF-WP3 — docs(core-trust-freeze): update determinism and verifier-first evidence after PCC`
- CTF follow-up: `CTF-WP4 — docs(core-trust-freeze): update golden trace and capability denial policy after PCC`
- CTF evidence: `CTF-E1 — test(core-trust-freeze): add golden trace coverage for selected PCC fixture surfaces`
- CTF evidence: `CTF-E2 — test(core-trust-freeze): add collection determinism replay coverage`
- CTF evidence: `CTF-E3 — test(core-trust-freeze): add trap taxonomy regression coverage`
- CTF waypoint: `CTF-WP6 — docs(core-trust-freeze): define project-root trust policy before PCC-9I`
- CTF waypoint review: `docs/roadmap/language_maturity/core_trust_freeze/ctf_waypoint_review_after_wp1_wp4.md`
- CTF backlog / promotion rules: `docs/roadmap/language_maturity/core_trust_freeze/ctf_evidence_backlog.md`
- CTF backlog / promotion rules: `docs/roadmap/language_maturity/core_trust_freeze/freeze_candidate_promotion_rules.md`
- CTF waypoint: `CTF-WP5 — docs(core-trust-freeze): define CTF evidence backlog and freeze-candidate promotion rules`

PCC-9 is closed for the current admitted manifest / project-adjacent baseline.
The current evidence is bounded to the existing package-manifest baseline and
project-adjacent helpers; project-root `check` / `run` and `smc new` remain
explicit follow-up work unless separately evidenced.

Checkpoint outcome:

- bounded current manifest / project-adjacent baseline is closed;
- project-root workflow, `semantic.toml` parser / loader, `smc new`, package
  registry, dependency resolver, and workspace remain open;
- the next step is CTF synchronization and qualification planning.

DoD:

```text
[ ] minimal project layout is documented
[ ] project check/run works or is explicitly scoped as follow-up
[x] project-level diagnostics are stable for the admitted manifest baseline
[x] project fixtures exist for the admitted manifest baseline
```

PCC-9 remains closed only for the current admitted manifest / project-adjacent
baseline. The remaining project-root flow, `semantic.toml` parser / loader,
`smc new`, and project-level 7hell remain explicit follow-up work.

## 18. Waypoint review rule

PCC uses a 4-week waypoint review. It is not a decorative status meeting; it is a control gate.

Outcomes:

```text
A. On track
   → continue current phase

B. Behind, scope still valid
   → continue with updated schedule / risk

C. Behind, scope too large
   → cut scope to minimum viable
   → move overflow to post-PCC
   → or rebuild the PCC forecast
```

Hard rule:

```text
If a PCC phase fails to move by DoD for two consecutive waypoint reviews,
that phase scope must be explicitly reviewed.
```

Allowed scope decisions:

- reduce to minimum viable;
- split into sub-phase;
- move nonessential pieces to post-PCC;
- pause and rebaseline PCC forecast;
- downgrade a formerly assumed requirement.

## 19. Planning envelope

These estimates are planning envelopes, not promises.

| Phase | Envelope |
|---|---:|
| PCC-0 / 0.5 / 0.6 / CTF-0 | 1–2 weeks |
| PCC-1 Control Flow | 2–3 weeks |
| PCC-2 Numeric Core | 3–4 weeks |
| PCC-3 Text Core | 1–2 weeks |
| PCC-3.5 Carrier Note | 2–3 days |
| PCC-4 Records | 3–4 weeks |
| PCC-5 ADT + Match | 4–6 weeks |
| PCC-6 Option / Result | 1–2 weeks |
| PCC-7 Collections | 4–6 weeks |
| PCC-8 Stdlib | 2–3 weeks |
| PCC-9 Project Model | 2–3 weeks |
| CTF overhead | +20–30% per feature phase |

Realistic total horizon:

```text
minimum: 5–6 months
realistic: 6–9 months
with production shifts / breaks: 9–12 months
```

## 20. Gate before PCC-1

PCC-1 is conditionally unblockable after the PCC-0 blocker audit gate is
closed and maintainers accept the audited state.

```text
PCC-0 Truth Reset: landed
PCC-0.5 Feature Matrix Live Audit: landed
PCC-0.6 7hell Skeleton Seed: landed
CTF-0 Core Trust Freeze directory: landed
```

This is intentional. PCC-1 starts only after the project has one current
state, one audit map, one diagnostic harness seed, and one trust-lane registry,
and after maintainers accept the closed gate state.

## 21. Final formula

```text
PCC v0.3 =
  Truth Reset
+ Live Audit
+ Practical Core Phases
+ parallel Core Trust Freeze
+ early 7hell
+ strict out-of-scope
+ waypoint decision rule
+ no debug_render public creep
```
