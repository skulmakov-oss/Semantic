# PCC Waypoint Review after PCC-4..PCC-9

Status: waypoint review
Owner: language maturity stream
Scope: checkpoint after bounded PCC-4..PCC-9 closeout
Non-goal: feature implementation or release readiness claim

## Purpose

This document records the waypoint review after closing PCC-4..PCC-9.

It evaluates progress, bounded-open items, risks, and the next-phase decision.

It does not reopen any closed PCC scope.

It does not claim full readiness.

## Closed PCC Evidence Chain

| PCC | Area | Closeout meaning | Evidence status | Remaining bounded-open items |
| --- | --- | --- | --- | --- |
| PCC-4 | Records | closed for current record seams and fixtures | closed | broad record ergonomics / future widening if separately scoped |
| PCC-5 | ADT + basic match | closed for current ADT/basic match fixture-backed surface | closed | broader pattern matching / advanced exhaustiveness if separately scoped |
| PCC-6 | Option / Result | closed for standard-form Option/Result | closed | exception semantics / helper expansion out of scope |
| PCC-7 | Collections v0 | closed for fixture-backed sequence/map surface | bounded closed | map missing-key policy, map iteration, memory/quota policy remain open |
| PCC-8 | Stdlib v0 | closed for current admitted helper surface | bounded closed | std.math, universal to_text, formatting macros, IO/capability expansion remain open |
| PCC-9 | Project Model v0 | closed for admitted Semantic.package manifest baseline | bounded closed | project-root check/run, semantic.toml parser, smc new, workspace/registry remain open |

## What Actually Improved

- dedicated live audit docs exist for PCC-4..PCC-9;
- positive fixtures were added for records, ADT, match, Option, Result, Sequence, Map, Stdlib helpers, Project Model manifest baseline;
- negative diagnostics / trap fixtures were added for records, ADT/match, Option/Result, Collections, Stdlib, Project Model;
- public contracts were frozen where needed:
  - Stdlib helper contract;
  - Project Model contract;
- evidence chain is now explicit;
- feature matrix is less speculative;
- open boundaries are explicitly named instead of hidden.

## What Remains Open

### Language / Surface

- advanced match patterns;
- broader records / ADT ergonomics if needed;
- any still-unclosed control-flow or everyday expressiveness items from earlier PCC phases.

### Runtime / Verifier / Trust

- CTF is not closed;
- trap taxonomy may need sync;
- runtime value registry may need sync;
- determinism matrix may need sync;
- verifier-first policy may need sync after PCC changes;
- trace / golden policy may need update.

### Collections / Stdlib

- map missing-key policy;
- map iteration policy;
- collection memory / quota evidence;
- std.math;
- formatting helpers / macros;
- universal to_text remains explicitly rejected.

### Project Model

- project-root `smc check <project-root>`;
- project-root `smc run <project-root>`;
- `semantic.toml` parser / loader;
- `src/main.sm` discovery;
- `smc new`;
- project-level 7hell;
- package registry / dependency resolver / workspace remain out of scope.

### Qualification / Release

- 7hell is not complete;
- release readiness is not claimed;
- CTF lane still needs its own evidence sync.

## Risks Discovered

1. Branch hygiene risk:

   - stale / reused branches can create misleading PRs;
   - PR title / branch / diff scope must match.

2. Overclaim risk:

   - bounded closeouts can be misread as full readiness;
   - docs must retain "current admitted surface" wording.

3. Project Model risk:

   - `semantic.toml` target contract can be mistaken for implemented parser behavior;
   - project-root commands remain future work.

4. Stdlib risk:

   - `to_text` can drift into reflection;
   - `debug_render` can drift into public helper behavior;
   - std.math can be overclaimed too early.

5. Collections risk:

   - map missing-key behavior and iteration policy remain unresolved;
   - memory / quota evidence remains open.

6. CTF risk:

   - PCC widened surface without automatic CTF sync unless explicitly done next.

## Governance Decisions

```text
Rule 1:
A PCC closeout means only what its closeout document says. No broad readiness inference.

Rule 2:
Every future PR must pass branch/title/diff/scope hygiene.

Rule 3:
Any project-root work must be split into implementation PRs before fixture closeout.

Rule 4:
No stdlib widening through docs-only closeout.

Rule 5:
No CTF claim without dedicated CTF evidence.
```

## Next-Phase Decision

```text
Decision:
Move from PCC feature closeout into CTF / qualification synchronization.

Immediate next step:
CTF-WP1 — docs(core-trust-freeze): sync PCC closeout impact across runtime value, traps, verifier, determinism, and trace policy
```

Alternative allowed:

```text
If project management wants another checkpoint first:
PCC-WR2 — docs(pcc): reconcile PCC-0..PCC-9 matrix against CTF readiness
```

Default recommendation remains `CTF-WP1`.

## CTF Impact Preview

| CTF area | Why PCC-4..9 affects it | Required next action |
| --- | --- | --- |
| RuntimeValue registry | records / ADT / Option / Result / collections / text / project metadata touched value surfaces | audit registry |
| Trap taxonomy | assert, collections, stdlib, project diagnostics / traps expanded failure surface | sync taxonomy |
| Determinism matrix | collections / project / module roots require deterministic behavior | sync matrix |
| Verifier-first policy | new accepted surfaces must remain verifier-first | verify evidence |
| Golden trace policy | fixture-backed surfaces should map to stable evidence | sync trace policy |
| Capability / effect denial | stdlib / project boundaries must not widen host effects | confirm no widening |

## Final Verdict

```text
PCC-4..PCC-9 are closed for their bounded current scopes.
The language maturity stream now has enough evidence to move into CTF synchronization and qualification planning.
This waypoint does not claim release readiness.
```

## Acceptance Checklist

- [x] PCC-4..PCC-9 closeout chain summarized
- [x] bounded-open items listed
- [x] overclaim risks documented
- [x] branch hygiene risk documented
- [x] next-phase decision recorded
- [x] CTF impact preview added
- [x] practical_core_completion_v0_3.md points to waypoint review
- [x] no feature statuses overclaimed
- [x] no code changed
- [x] no tests or fixtures changed

CTF touched: none

Reason:

`docs-only waypoint review; no runtime value, trap, determinism, verifier, SymbolId, capability, or trace change`
