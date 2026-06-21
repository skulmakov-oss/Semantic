# Semantic Public Status Model

Status: active vocabulary authority
Primary owners: release discipline, readiness posture, public-facing truth

## Purpose

This document defines the canonical status vocabulary used by readiness-facing
and release-facing Semantic documents.

Its purpose is not to widen scope, reinterpret existing behavior, or change the
current release verdict.

Its purpose is only to ensure that the repository describes the same truth in a
stable and repeatable way.

## Scope

This document governs the wording and placement of status claims in:

- `README.md`
- `docs/roadmap/backlog.md`
- `docs/roadmap/milestones.md`
- `docs/roadmap/v1_readiness.md`
- `docs/roadmap/compatibility_statement.md`
- `docs/roadmap/stable_release_policy.md`
- `reports/g1_*.md`
- stale or completed scope docs that still influence the public reading of the
  repository

This document does not itself declare the current factual release posture.

## Non-Goals

This document does not:

- widen the release contour
- change the current `Gate 1` verdict
- convert current-`main` behavior into release-promised behavior
- rewrite the published stable line
- open a new feature or blocker-removal track

## External Analysis Reading Guardrail

External summaries, AI-generated reviews, public writeups, and third-party
analyses must not infer release status from naming, diagrams, roadmap shorthand,
or historical compatibility paths.

External-facing descriptions should keep these readings explicit:

- Semantic is deterministic verified execution, not general production
  compliance.
- TON618 names are compatibility perimeter, not current platform ownership.
- `CTF` means Core Trust Freeze.
- `7hell` is staged qualification and report shape.
- Current `main` is not automatically stable or qualified release claim.
- Future-fit domains are not current certification, security, or cryptographic
  guarantees.
- Numeric claims require benchmark, report, test, or release-note evidence.

This guardrail does not create a fifth status family. It applies the four
canonical status families below to external-facing interpretation.

## Canonical Status Vocabulary

Only the following four status families should be used for release-facing truth
and readiness-facing truth.

### 1. Published Stable

Definition:

- behavior and artifacts promised by the published stable line

Use this when a document is describing:

- the current stable tag line
- supported stable assets
- stable compatibility promises
- release-facing guarantees already made publicly

Do not use this for:

- behavior that exists only on current `main`
- qualification results that have not yet been promoted to a stable release

### 2. Qualified Limited Release

Definition:

- behavior qualified by the active completed `Gate 1` evidence and explicitly
  admitted into the current practical-programming contour

Use this when a document is describing:

- what the current qualification cycle actually proved
- the admitted practical-programming contour
- the current limited-release claim

Do not use this for:

- all behavior landed on `main`
- stable promises that were never qualified through the current evidence set

### 3. Landed On `main`, Not Yet Promised

Definition:

- behavior that exists on current `main` but is not yet part of the current
  release promise

Use this when a document is describing:

- post-stable landed waves
- widened current-`main` surfaces
- implemented or completed tracks that remain outside the current release
  promise

Do not use this as a synonym for:

- `experimental`
- `stable`
- `qualified limited release`

This category exists precisely to stop landed work from being silently promoted
into a release claim.

### 4. Out Of Scope

Definition:

- behavior intentionally excluded from the current release contour or current
  readiness-critical track

Use this when a document is describing:

- explicit non-goals
- excluded surface families
- behavior that must not be inferred as supported

Do not use this to hide partial support that actually exists on current `main`.
If behavior is landed, it must be described as landed and unpromoted rather
than erased.

## Authority Order

During readiness completion, the release-facing authority order is:

1. `docs/roadmap/public_status_model.md`
   - vocabulary authority only
2. `docs/roadmap/v1_readiness.md`
   - current release-facing posture authority
3. `reports/g1_release_scope_statement.md`
   - current practical-programming qualification verdict authority

No other document may silently override these layers.

## Promotion Rules

### Rule A — Landed Is Not Automatically Promised

A behavior moving into current `main` does not automatically become:

- `qualified limited release`
- or `published stable`

Promotion requires an explicit later decision.

### Rule B — Qualification Is Separate From Stable Publication

A behavior can be:

- `qualified limited release`
- while still not being part of the `published stable` promise

Qualification and stable publication are separate decisions.

### Rule C — Stable Publication Requires Explicit Release Governance

A behavior should be described as `published stable` only when the stable line
and its supporting release artifacts actually promise it.

### Rule D — Out Of Scope Must Stay Honest

If behavior is not supported anywhere, mark it `out of scope`.
If behavior is landed on `main` but unpromoted, do not erase it by calling it
`out of scope`.

## Placement Rules

### `README.md`

`README.md` should:

- present the current high-level posture
- use the canonical vocabulary where release posture is mentioned
- avoid becoming the detailed matrix for every surface

### `docs/roadmap/backlog.md`

`backlog.md` should:

- describe active and inactive workstream state
- not silently broaden the release promise
- distinguish between completed post-stable tracks and active next-focus work

### `docs/roadmap/milestones.md`

`milestones.md` should:

- describe milestone structure and landed checkpoint placement
- not act as the authority for the current release verdict

### `docs/roadmap/v1_readiness.md`

`v1_readiness.md` should:

- state the current release-facing posture
- distinguish stable promises from widened current-`main` behavior
- remain the primary release-facing reading after this vocabulary document

### `docs/roadmap/compatibility_statement.md`

`compatibility_statement.md` should:

- describe compatibility commitments honestly
- avoid implying that all current-`main` behavior is stable or qualified

### `docs/roadmap/stable_release_policy.md`

`stable_release_policy.md` should:

- describe how stable publication moves forward
- not silently inherit widened current-`main` behavior as part of the stable
  line

### `reports/g1_*.md`

Qualification reports should:

- use `qualified limited release` only for what the evidence actually proved
- avoid pretending to summarize the whole repository
- avoid overriding stable publication claims

## Writing Rules

When in doubt, prefer explicit wording such as:

- `published stable v1.1.1`
- `qualified for limited release`
- `landed on current main, not yet promised`
- `explicitly out of scope`

Avoid ambiguous phrases such as:

- `supported` without saying at which layer
- `done` without saying whether that means stable, qualified, or merely landed
- `available` without saying whether it is release-promised

## Conflict Rule

If two release-facing documents appear to describe different readiness states,
the mismatch should be treated as a real readiness defect.

The correct fix is:

- sync the documents
- not invent a fifth status family
- not silently blur the distinction between stable, qualified, landed, and
  excluded behavior

## Operational Reading

This document is the vocabulary authority for the readiness-completion cycle.

The current factual repository posture still lives elsewhere:

- `docs/roadmap/v1_readiness.md`
- `reports/g1_release_scope_statement.md`
- relevant stable-line and compatibility docs

This separation is intentional:

- this document defines the language of status
- later sync steps define the current facts using that language
