# CTF Waypoint Review after CTF-WP1..WP4

Status: waypoint review
Owner: language maturity / execution contract
Scope: checkpoint after first CTF sync wave
Non-goal: implementation, release readiness, or CTF closure

## Purpose

This document records the waypoint review after CTF-WP1..WP4.

It evaluates what was synchronized after PCC-4..PCC-9.

It identifies remaining trust gaps.

It recommends the next CTF phase.

It does not close CTF.

It does not claim release readiness.

## CTF-WP1..WP4 Evidence Chain

| CTF step | Area | What changed | Status | Remaining open items |
| --- | --- | --- | --- | --- |
| CTF-WP1 | Trust surface sync | reviewed PCC-4..PCC-9 impact across all CTF areas | complete | follow-up split required |
| CTF-WP2 | Runtime values + traps | updated runtime value registry and trap taxonomy notes | complete | final trap class names for some PCC failure surfaces may need future evidence |
| CTF-WP3 | Determinism + verifier-first | updated determinism matrix and verifier-first policy | complete | Map / project-root / semantic.toml determinism remain open |
| CTF-WP4 | Golden traces + capability/effect denial | mapped trace candidates and tightened capability boundary | complete | no golden artifacts yet; capability policy remains docs-level |

Use “complete” only for the waypoint PR itself, not for the full CTF lane.

## What Actually Improved

- CTF lane now has explicit post-PCC sync docs.
- Runtime value registry no longer leaves PCC-4..PCC-9 families as vague planned items.
- Trap taxonomy now distinguishes compile-time diagnostics from VM trap classes.
- Determinism matrix now separates current admitted fixture-backed determinism from future/open project-root and Map edges.
- Verifier-first policy now explicitly covers PCC fixture-backed surfaces.
- Golden trace policy now maps PCC surfaces to future trace candidates without pretending fixtures are golden traces.
- Capability/effect denial matrix now keeps `print(text)`, `to_text`, `debug_render`, file IO, network IO, package registry, and remote dependencies bounded.
- PCC closeout and CTF freeze are now formally separated.

## What Remains Open

### Runtime values

- Some PCC-backed value families are only `freeze-candidate`, not frozen.
- Map remains bounded because missing-key, iteration, and quota policy are open.
- Project manifest metadata is project-adjacent evidence, not necessarily runtime value behavior.

### Trap taxonomy

- PCC compile-time diagnostics must not be promoted into VM traps.
- Collection runtime failure surfaces may need final trap naming evidence.
- Future helper/runtime traps need explicit CTF admission.

### Determinism

- Map missing-key behavior remains unresolved.
- Map iteration policy remains unresolved.
- Collection memory/quota determinism remains open.
- Project-root discovery remains open.
- `semantic.toml` parse/load determinism remains open.
- `src/main.sm` discovery remains open.
- `smc new` output determinism remains open.
- Project-level 7hell determinism remains open.

### Verifier-first

- Project helper tests are not public execution evidence.
- Future project-root `check/run` must preserve:
  `project source -> check -> compile -> verify -> run`
- Future SemCode/opcode/helper expansion must update verifier-first policy.

### Golden traces

- PCC fixtures are not automatically golden traces.
- Golden trace artifacts were not added in WP4.
- Compile-time diagnostic traces and VM runtime traces remain separate.
- Project manifest traces are not project-root execution traces.
- 7hell report traces remain future work.

### Capability / effects

- `print(text)` remains bounded admitted helper output, not host capability widening.
- `to_text` remains admitted-types-only.
- `debug_render` remains internal-only.
- File IO and network IO remain out-of-PCC.
- Package registry and remote dependency behavior remain out of scope.
- Local audit remains distinct from telemetry.

## Risks Discovered

1. Freeze-candidate drift:

   - docs may say freeze-candidate, but without tests it can still drift.

2. Trace gap:

   - many PCC surfaces have fixtures but no golden traces.

3. Trap naming gap:

   - some PCC failure surfaces are fixture-backed but not final trap taxonomy classes.

4. Project model trust gap:

   - project-root behavior, semantic.toml and smc new remain open.

5. Capability interpretation risk:

   - `print(text)` can be mistaken as host IO;
   - `to_text` can drift into reflection;
   - debug_render can drift into public output.

6. Release overclaim risk:

   - PCC + CTF docs sync can be mistaken for release readiness.
   - This must remain explicitly false.

## Governance Decisions

```text
Rule 1:
CTF-WP1..WP4 sync does not close CTF.

Rule 2:
A freeze-candidate CTF entry requires future evidence before release-facing freeze.

Rule 3:
PCC fixtures are not golden traces unless explicitly promoted through golden trace policy.

Rule 4:
Compile-time diagnostics must not be mislabeled as VM traps.

Rule 5:
Future project-root work must update determinism, verifier-first, trace, and capability notes.

Rule 6:
Future stdlib / collection widening must update trap, determinism, trace, and capability notes.

Rule 7:
No release-readiness claim may cite CTF-WP1..WP4 without a later CTF closeout / qualification PR.
```

## Next-Phase Decision

```text
Decision:
Move from CTF docs-sync waypoint into targeted evidence planning.

Immediate next step:
CTF-E1 — test(core-trust-freeze): add golden trace coverage for selected PCC fixture surfaces
```

Alternative if we want one more docs-only step:

```text
CTF-WP5 — docs(core-trust-freeze): define CTF evidence backlog and freeze-candidate promotion rules
```

Default recommendation:

- If we want to start hardening with tests: `CTF-E1`.
- If we want one more control document before tests: `CTF-WP5`.

The document should recommend `CTF-WP5` first if the project wants maximum control before adding trace artifacts.

## Evidence Backlog Preview

| Backlog item | Why needed | Candidate next PR |
| --- | --- | --- |
| Golden trace selection | PCC fixtures are not golden traces | CTF-WP5 / CTF-E1 |
| Golden trace artifacts | Need stable source / type / IR / SemCode / verifier / VM trace samples | CTF-E1 |
| Collection determinism replay | Map / Sequence behavior needs replay evidence | CTF-E2 |
| Trap taxonomy regression | Fixture-backed failure surfaces need stable trap / diagnostic mapping | CTF-E3 |
| Project-root trust policy | Future project-root work must preserve verifier / determinism / capability boundaries | future PCC-9I / CTF follow-up |
| 7hell report shape | Qualification output must become stable before readiness | 7HELL-WP / CTF-E follow-up |

## Final Verdict

```text
CTF-WP1..CTF-WP4 completed the first post-PCC trust-surface synchronization wave.
The CTF lane is better aligned, but not closed.
The project should now move into targeted evidence planning before any release-readiness claim.
```

## Acceptance Checklist

```markdown
- [ ] CTF-WP1..WP4 chain summarized
- [ ] remaining CTF open items listed
- [ ] runtime value / trap / determinism / verifier-first / trace / capability gaps documented
- [ ] overclaim risks documented
- [ ] governance decisions recorded
- [ ] next-phase decision recorded
- [ ] evidence backlog preview added
- [ ] CTF index points to CTF-WR1
- [ ] no CTF closure claimed
- [ ] no release readiness claimed
- [ ] no code changed
- [ ] no tests or fixtures changed
- [ ] no golden trace artifacts added
```

CTF touched: docs only

Reason: CTF waypoint review after WP1..WP4 sync; no runtime value, trap, determinism, verifier, SymbolId, capability, or trace behavior change
