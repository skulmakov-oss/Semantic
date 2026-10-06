# Semantic v1 Readiness

Status: current release-facing posture authority

Read this document using the canonical status vocabulary in:

- `docs/roadmap/public_status_model.md`

Within that authority order, this document is the primary release-facing
reading for the repository.

## Current Posture

The repository currently spans four different factual layers:

- `published stable`
  - `v1.2.0`, the Stable Foundation release at qualified candidate C1, within
    its documented explicit limits
- `qualified limited release`
  - the current practical-programming verdict from the completed Gate 1 cycle
- `landed on main, not yet promised`
  - widened surfaces present on current `main` but not yet promoted into either
    a stable line or the qualified contour
- `out of scope`
  - surfaces still intentionally excluded from the current qualified contour and
    candidate Stable Foundation promise

Current top-level reading:

- `v1.2.0` is the published Stable Foundation release, Windows x64 only, with
  explicit limits
- Semantic is **not** positioned as a general production-ready public release
  beyond that contour
- the Gate 1 practical-programming contour remains a separate
  **qualified limited release** reading
- current `main` is development beyond C1 and contains wider landed work that
  is not part of any release promise

## Published Stable and Candidates

`v1.2.0` is the published stable release: **Semantic v1.2.0 — Stable
Foundation Release**, published on 2026-10-04 from qualified candidate C1
`89641da8237f4fcefb50cf1958a50e4d4003aea7` (`v1.2.0^{}` dereferences to C1).

- Foundation Oracle Verdict: `ORACLE QUALIFIED WITH EXPLICIT LIMITS`
- Stable Foundation Promotion Decision: `PROMOTE WITH EXPLICIT LIMITS`
- release contour: the Stable Foundation contour in `reports/semantic_stable_foundation_final_verdict.md` (section 6)
- platform: Windows x64 (`x86_64-pc-windows-msvc`) only; Linux and macOS
  release binaries are outside the v1.2.0 contour
- artifact trust: assets are explicitly unsigned; trust is bound to the
  published SHA-256 digests
- explicit limits: R1 (Quad helper predicates `known`/`unknown`/`conflict`
  are rejected), R2 (string literals do not process escapes), R3 (generic
  rejection has no stable diagnostic code), R4 (host paths appear in
  diagnostics), and `std.serde` (G-08) is not applicable / excluded

Current `main` is development beyond C1 (including the later Phase-B and
residual hardening). It is not the v1.2.0 release and is not qualified as any
new release; nothing landed after C1 widens the v1.2.0 promise.

History: the earlier `v1.1.1` tag remains an unresolved stable-tag checkpoint
(no GitHub Release), and `v1.2.0-beta.1` was the prerelease that preceded the
Stable Foundation release.

The v1.2.0 release is a statement about C1, not a complete description of
everything already landed on current `main`.

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
- iterable surface beyond the qualified built-in `Sequence(T)` iteration and
  direct-record `Iterable` dispatch
- first-wave closures
- first-wave generics
- runtime ownership for tuple + direct record-field paths
- module and import work beyond the qualified direct local-path bare/selected
  helper-module slice (the selected-import slice itself is qualified above)

These surfaces must stay explicitly unpromoted until a later scope decision and
qualification or release decision promotes them.

The first-wave UI application boundary is also landed on `main`, but it belongs
to the retired native UI / Workbench / Semantic Studio contour
(`docs/roadmap/ui_workbench_studio_retirement.md`): it is not awaiting
promotion and is not on the active roadmap.

SemCode compatibility migration (SSF-09 D2-2, PR #1963): `SEMCOD22`
(revision `23`) activates the mandatory `ADT0` descriptor section, and the
compiler emits it for every artifact. Legacy (`SEMCODE0` to `SEMCOD21`)
artifacts that use descriptor-dependent ADT opcodes are no longer
verifier-admissible; recompilation is the migration, and non-ADT legacy
artifacts are unaffected (`docs/roadmap/compatibility_statement.md`). Evidence:
verifier/VM/decoder admission and rejection tests, re-blessed golden
fixtures, mutations M1-M17 killed, and two adversarial reviews. This does not
promote any surface to the stable line.

## Current Known Limits

The following release-facing limits remain explicit:

- broader executable-module authoring beyond the admitted bare/selected slice
  is not currently qualified
- full CLI application authoring with admitted argv/stdout/file IO is not
  currently qualified
- native UI, Workbench, and Semantic Studio are retired and outside every
  current contour (`docs/roadmap/ui_workbench_studio_retirement.md`)
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
