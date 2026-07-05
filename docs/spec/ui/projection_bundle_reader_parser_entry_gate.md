# ProjectionBundle Reader/Parser Entry Gate v0

Status: pre-Level-4 gate
Track: POST-UI / Intent-Driven Projection
Scope type: reader/parser entry boundary
Current claim level: Level 3 only
Level 4 status: not claimed
Implementation status: no reader / no parser / no loader / no runtime
Authority status: non-authorizing

This document defines the gate before any ProjectionBundle reader/parser claim may be made.

It does not define a reader.
It does not define a parser.
It does not define final serialization.
It does not define a loader.
It does not define runtime behavior.
It does not authorize production UI wiring.

```text
Do not build a parser to discover the contract.
Define the contract before building the parser.
```

A reader/parser claim requires reader/parser evidence.
Level 3 evidence cannot be used to claim Level 4 behavior.

## 1. Purpose

This document exists to prevent jumping from fixture evidence directly into parser or loader implementation.

The purpose of this gate is to stop Level 4 from being smuggled in as a convenience refactor.

This gate exists before a reader/parser basis, not as a substitute for one.

## 2. Current State

Current achieved level: Level 3 only.
Level 4 is not claimed.
Reader/parser behavior is not claimed.
Loader behavior is not claimed.
Runtime behavior is not claimed.
Production UI behavior is not claimed.

Current evidence:

- inert fixture anchor
- fixture boundary guard
- POST-UI fixture guard aggregator
- fixture-facing Rust manifest draft types
- compile-only manifest draft guard
- sketch/draft drift guard
- ProjectionBundle Basis v0
- closeout reading-order link

This evidence is sufficient for Level 3 only.

The current sketch reader evidence contour strengthens fixture-facing evidence, but it does not satisfy this gate.

General Level 4 remains not claimed.

A separate reader/parser basis is still required before any general Level 4 reader/parser claim.

## 3. What Level 4 Would Mean

Level 4 means a separately approved reader or parser can consume an approved ProjectionBundle evidence format and produce a deterministic, testable representation without loader, runtime, verification, or production UI authority.

Level 4 does not mean loading.
Level 4 does not mean activation.
Level 4 does not mean verification.
Level 4 does not mean runtime execution.
Level 4 does not mean production readiness.

## 4. Required Reader/Parser Basis Contents

A future reader/parser basis must define at least:

- input status;
- accepted input boundaries;
- rejected input boundaries;
- non-goals;
- determinism rule;
- error model;
- unknown-field policy;
- ordering policy;
- duplicate-field policy;
- placeholder trust policy;
- negative fixtures;
- golden fixtures;
- reader/parser output shape;
- claim level impact;
- authority boundaries.

This document does not define those policies fully.

This document only lists what the future basis must cover.

## 5. Required Evidence Before Level 4

Required evidence before Level 4:

- a separately approved reader/parser basis;
- at least one positive reader/parser fixture;
- at least one negative reader/parser fixture;
- reader/parser rejection tests;
- deterministic output tests;
- placeholder trust rejection evidence;
- no loader activation;
- no runtime activation;
- no production UI wiring.

A PR that adds reader/parser code without this evidence must not claim Level 4.

## 6. Forbidden Before Level 4

- No parser implementation without reader/parser basis.
- No reader implementation without reader/parser basis.
- No loader.
- No runtime reader.
- No activation path.
- No production UI wiring.
- No final serialization claim.
- No public API claim.
- No security claim.
- No verification claim.
- No capability/audit authority widening.
- No Workbench wiring.
- No ui-shell-kit promotion.
- No prom-ui integration.

## 7. Allowed Pre-Level-4 Work

| Allowed work | Why allowed |
| --- | --- |
| Reader/parser basis draft | Defines evidence requirements before implementation. |
| Negative fixture planning | Prepares rejection evidence without creating parser authority. |
| Golden fixture planning | Prepares deterministic evidence without loader/runtime authority. |
| Error model planning | Defines failure boundaries before reader/parser code exists. |
| Entry gate updates | Clarifies what future PRs may claim. |

## 8. Forbidden Claims

| Forbidden claim | Reason |
| --- | --- |
| ProjectionBundle reader exists | No reader implementation exists. |
| ProjectionBundle parser exists | No parser implementation exists. |
| ProjectionBundle format is stable | No final serialization has been selected. |
| ProjectionBundle can be loaded | No loader exists. |
| ProjectionBundle can be activated | No runtime activation path exists. |
| ProjectionBundle is verified | No verification implementation exists. |
| ProjectionBundle is secure | No security proof exists. |
| Level 4 is achieved | Reader/parser evidence does not exist yet. |

## 9. Authority Boundaries

This gate does not move authority between layers.

Reader/parser work may only describe or produce representation.
It must not claim semantic admission authority.
It must not claim capability authority.
It must not claim audit authority.
It must not claim runtime authority.
It must not claim rendering authority.

Semantic owns meaning.
Projection owns presentation intent.
UI IR owns structure.
Action IR owns affordance routing.
Binding Graph owns deterministic dependency mapping.
Patch streams own projection updates.
Shell owns rendering behavior.
Renderer owns pixels.
Verifier / admission owns semantic admission decisions.
Capability / audit authority owns capability checks, host-effect permission, critical action authorization, and audit evidence boundaries.
Runtime owns execution / scheduling only where explicitly specified.

## 10. Exit Criteria From This Gate

Minimum conditions before attempting a future Level 4 PR:

- `ProjectionBundle Basis v0` is linked in the closeout reading order.
- Reader/parser entry gate exists.
- A separate reader/parser basis PR is prepared.
- The reader/parser basis states non-goals and evidence requirements.
- Negative and positive fixture strategy is defined.
- No loader/runtime/production wiring is included.

This document alone does not satisfy Level 4.

## 11. Working Rule for Future PRs

Every future reader/parser-adjacent PR must state whether it changes the claim level.

If it remains Level 3, it must say so.

If it attempts Level 4, it must provide Level 4 evidence.

No PR may imply Level 4 through naming, comments, file placement, or test wording unless Level 4 evidence exists.

## 12. Reader/Parser Basis Link

`docs/spec/ui/projection_bundle_reader_parser_basis.md` defines the basis for any future reader/parser claim.

This gate remains active.

The existence of the basis does not satisfy general Level 4.

The evidence matrix must be complete before any general Level 4 reader/parser behavior may be claimed.
