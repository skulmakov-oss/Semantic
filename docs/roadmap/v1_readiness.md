# Semantic v1 Readiness

Status: current release-facing posture authority

Read this document using the canonical status vocabulary in:

- `docs/roadmap/public_status_model.md`

Within that authority order, this document is the primary release-facing
reading for the repository.

## Current Posture

The repository currently spans four different factual layers:

- `published stable`
  - no current feature meets this evidence test
- `qualified limited release`
  - the current practical-programming verdict from the completed Gate 1 cycle
- `landed on main, not yet promised`
  - widened surfaces present on current `main` but not yet promoted into either
    a stable line or the qualified contour
- `out of scope`
  - surfaces still intentionally excluded from the current qualified contour and
    candidate Stable Foundation promise

Current top-level reading:

- Semantic is **not** currently positioned as `public release`
- Semantic is currently qualified only for a **limited release** practical
  contour
- current `main` contains wider landed work than the current qualified contour

## Published Stable and Candidates

There is no currently evidenced published stable line. The repository has a
`v1.1.1` git tag, but the decision stored at that tag describes a proposed
stable-tag checkpoint, leaves downloaded-asset smoke for the exact tag as a
blocker, and no corresponding GitHub Release exists.

The current prerelease is:

- `v1.2.0-beta.1`

It should be read as prerelease evidence, not as a stable publication or a
complete description of everything already landed on current `main`.

The intended artifact model and platform-scope rules are defined in:

- `docs/release_artifact_model.md`

## Qualified Limited Release

The completed Gate 1 evidence currently supports a narrow practical-programming
contour documented in:

- `reports/g1_release_scope_statement.md`

That qualified contour includes:

- single-file executable programs on the admitted source surface
- narrow helper-module executable programs using direct local-path bare imports
- narrow helper-module executable programs using direct local-path selected
  imports over function-only helper modules
- rule/state-oriented programs over records, `quad`, and explicit
  `Option` / `Result`
- built-in `Sequence(T)` iteration
- direct-record user-defined `Iterable` dispatch
- verified execution through the admitted
  `source -> sema -> IR -> SemCode -> verifier -> VM` path

This is enough for:

- `qualified limited release`

It is not enough for:

- `public release`

## Application Completeness Benchmark Contour

The completed Application-Completeness evidence supports a practical-programming contour documented in:

- `reports/application_completeness_benchmark_verdict.md`

This contour is **landed and benchmark-qualified on main, not yet promoted to published stable or Gate-1 qualified contour**.

The benchmark-qualified contour includes:

- same-family `i32` relational operators and arithmetic (`+`, `-`, `*`, `/`, `%`)
- mutable locals (`let mut` + reassignments) and loop control exits (`while`, `loop`, `break`, `continue`)
- bounded text usability (`text` literals, same-family equality, `text + text`, explicit `to_text` for admitted scalar families)
- built-in `Sequence(T)` persistent utilities (`len`, `push`, `pop`, `prepend`, `contains`)
- persistent functional `Map(K, V)` lookup tables
- deterministic seeded PRNG (xorshift64)
- narrow stdout observation (`print(text)`) under capability and audit boundaries
- bounded project-root CLI baseline (`semantic.toml` entrypoint resolution)

## Landed On `main`, Not Yet Promised

Current `main` contains widened surfaces beyond the currently qualified
practical-programming contour.

High-signal landed families include:

- schema/boundary-core work
- package baseline work
- ordered sequence surface
- iterable surface
- first-wave closures
- first-wave generics
- runtime ownership for tuple + direct record-field paths
- first-wave UI application boundary
- selected-import executable module entry

These surfaces must stay explicitly unpromoted until a later scope decision and
qualification or release decision promotes them.

## Current Known Limits

The following release-facing limits remain explicit:

- broader executable-module authoring beyond the admitted bare/selected slice
  is not currently qualified
- full CLI application authoring with admitted argv/stdout/file IO is not
  currently qualified
- UI remains outside the current qualified contour
- broader generalized iterable dispatch remains outside the current qualified
  contour
- landed-on-`main` widenings beyond the above admitted contour are not
  automatically part of the stable line and are not automatically qualified

## Current Release Gates

Release-facing truth should be treated as valid only while the relevant
validation remains green:

- `cargo test --workspace`
- boundary and ownership guard tests
- `cargo test --test public_api_contracts`
- release bundle verification
- release asset smoke verification
- compatibility/runtime validation checks used by the current release process

## Next Release-Maintenance Steps

The highest-signal remaining release-maintenance work is:

1. keep release-facing docs aligned with:
   - `docs/roadmap/public_status_model.md`
   - `reports/g1_release_scope_statement.md`
   - actual stable assets and actual current-`main` behavior
2. keep packaged stable assets and smoke validation aligned
3. avoid reopening scope during release-maintenance work
4. require a new explicit scope decision plus Gate amendment/new Gate cycle
   before any broader practical-programming widening is promoted

## Contract Rule

This document must not:

- silently promote landed-on-`main` behavior into the stable line
- silently promote landed-on-`main` behavior into the qualified contour
- blur the distinction between published stable, qualified, landed, and out of
  scope
