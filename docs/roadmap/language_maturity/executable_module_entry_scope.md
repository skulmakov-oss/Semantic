# Executable Module Entry Scope

Status: completed post-qualification blocker-removal checkpoint

## Goal

This narrow follow-up track removed the largest practical-programming blocker
identified after the first `Gate 1` cycle:

- ordinary helper-module executable authoring was blocked because top-level
  executable `Import` was not admitted on the executable source path

The landed result remains intentionally narrow. It is not a reboot of the whole
package or module ecosystem story.

## Status Reading For This Scope Doc

This document records a completed blocker-removal checkpoint on current `main`.

Per `docs/roadmap/public_status_model.md`, it should be read as a track record
for landed behavior and track scope, not as the authority for the current
release-facing verdict.

Current release-facing posture still lives in:

- `docs/roadmap/v1_readiness.md`
- `reports/g1_release_scope_statement.md`

## Why This Track Exists

The first `Gate 1` qualification cycle ended in:

- `qualified limited release`

The main blocker preventing a broader practical-programming claim was not VM
integrity or verifier trust. It was ordinary module-based executable authoring.

Evidence is frozen in:

- `reports/g1_real_program_trial.md`
- `reports/g1_frontend_trust.md`
- `reports/g1_release_scope_statement.md`

Those reports originally showed:

- single-file executable programs are admitted and runnable
- the frontend and execution path are trusted on the admitted contour
- module-based executable entry with top-level `Import` was still blocked at
  the parser/source-contract boundary

## Decision Check

- [x] This is a new explicit post-qualification track with its own scope decision
- [x] This addresses a proven release blocker rather than speculative feature work
- [x] This remains one stream, not a mixture of package, registry, and stdlib expansion
- [x] This can close with a clear done-boundary

## Landed First-Wave Reading

The landed first wave now admits only:

- direct local-path executable helper-module imports
- direct local-path selected executable helper-module imports with optional
  per-symbol alias inside the current function-only helper slice
- one root executable entry module containing `fn main()`
- imported executable declarations needed for ordinary helper-module programs

The goal was to admit ordinary module-based executable authoring without
silently widening into a general package or registry system.

## Landed In First Wave

- top-level `Import` admission on the executable source path
- direct local-path helper-module loading for executable module graphs
- direct local-path selected helper imports for function-only helper modules,
  including deterministic synthesis of the requested public bindings plus the
  required local helper-function call closure
- imported executable declarations for current source items such as:
  - `fn`
  - `record`
  - `enum`
  - `schema`
- deterministic executable module graph loading before semantic checking
- executable-path diagnostics for missing modules, missing selected symbols, and
  blocked out-of-scope import forms
- docs/tests/readiness updates that distinguish the admitted first wave from
  the already-frozen package ecosystem baseline

## Explicit Non-Goals

- external registries
- package manifests as part of this implementation wave
- lockfiles
- semantic version resolution
- generalized module loader redesign
- dynamic imports
- module-level executable statements
- wildcard or public re-export promises for the executable path unless they are
  explicitly admitted in a later scope decision
- namespace-qualified executable access unless it is explicitly admitted in a
  later scope decision

## Honest First-Wave Rules

- this track widens executable source admission, not the host/runtime boundary
- the root executable entrypoint remains `fn main()`
- imports remain deterministic and source-level only
- this wave is about helper-module authoring, not a full ecosystem
- landed package/dependency work on `main` does not automatically mean broader
  executable-module promises are qualified

## Executed Wave Order

### Wave 0 — Governance

- scope checkpoint
- backlog/milestone/readiness sync

### Wave 1 — Parser And Source Admission

- admit top-level `Import` in executable source files
- keep rejected forms explicit where the executable path is still narrower than
  the broader module spec

### Wave 2 — Executable Module Resolution

- build the executable helper-module graph before semantic checking
- make direct local-path helper-module declarations available to the executable
  semantic path through deterministic bundling
- admit direct local-path selected helper imports only for the current
  function-only helper slice
- keep top-level alias, wildcard, and re-export forms explicitly out of scope;
  package-qualified executable imports were later admitted by the bounded
  SSF-06 local package contract

### Wave 3 — Lowering / CLI / End-To-End

- run the admitted executable module graph through:
  - sema
  - IR
  - SemCode
  - verifier
  - VM
- keep diagnostics deterministic for missing or out-of-scope module cases

### Wave 4 — Freeze And Qualification Sync

- docs/spec/tests agree on the admitted executable module contour
- rerun qualification evidence if the practical-programming contour widens

## Qualification Sync Result

The widened admitted contour is now frozen as:

- single-file executable programs
- narrow helper-module executable programs using direct local-path bare imports
- narrow helper-module executable programs using direct local-path selected
  imports over function-only helper modules

The updated Gate 1 evidence keeps the overall decision state at:

- `qualified limited release`

The blocker was removed, but the release contour remains intentionally narrow
because broader executable import forms and full CLI-style authoring are still
outside the admitted contour.

## Acceptance Reading

This track is now done because:

1. ordinary helper-module executable programs are admitted on current `main`,
2. the executable import path is deterministic and tested end to end,
3. docs/spec/readiness language matches the actual admitted contour,
4. the release scope can be widened honestly only after the updated evidence is
   collected.
