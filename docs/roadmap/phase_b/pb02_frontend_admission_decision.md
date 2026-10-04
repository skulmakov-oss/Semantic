# PB-02 — Frontend Admission Decision

Status: decided (owner decision on #1645, 2026-10-04)
Scope: Phase-B PB-02, Module 02 `sm-front`: #1636, #1640–#1645, #1652, #1654, #1655, #1666
Historical audit umbrella: #1617 · Previous slice: PB-01 (#1985)
Base: `main` `10d1eafaa8b8508120c60416ad1412a654f4260a`
Reference oracle (unchanged): C1 `89641da8237f4fcefb50cf1958a50e4d4003aea7` = `v1.2.0`

Invariant: if `sm-front` admits a program, no later phase reinterprets an
admitted declaration or expression under a different frontend meaning.

## Base reproductions (all confirmed at `10d1eafa` before any change)

| Issue | Base behavior |
|---|---|
| #1636 | `fn f(x: i32, x: f64)` parsed and `build_fn_table` admitted it |
| #1640 | `qvec[99999999999999999999999]` → `QVec(32)` |
| #1641 | `qvec`, `qvec[]` → `QVec(32)` |
| #1642 | `qvec[8)`, `qvec(8]` → `QVec(8)` |
| #1643 | Logos `qvec` → `QVec(32)`; Logos `qvec[8]` rejected as a unit annotation |
| #1644 | `System A():` + `System B():` → `system = B` |
| #1645 | `Import "dep.sm" as D` absent from `LogosProgram`; loader re-scanned raw source |
| #1652 | `let _ = nope();`, `missing_var`, `main(1)`, `len(1, 2)` all typechecked OK |
| #1654 | user `fn qtruth_and` admitted; `main` lowered to `QTruthAnd`, user body never called |
| #1655 | user `fn len(x: i32)` admitted; `len(5)` rejected by the builtin `len` rule |
| #1666 | `cargo check -p sm-front --no-default-features --features alloc`: 75 errors; manifest forced `sm-profile/std` |

## Decisions

| # | Question | Decision |
|---|---|---|
| 1 | Canonical frontend admission boundary? | `sm-front` parse (source syntax), `build_*_table` (declaration admission), typecheck (expressions). Each fails closed with `FrontendError`. |
| 2 | Where are duplicate parameters rejected? | In the parser parameter loops (`parse_function`, which covers functions and impl methods, and `parse_trait_method_sig`), before any `FnSig`/`ScopeEnv` exists. |
| 3 | RustLike qvec syntax | `qvec(N)` or `qvec[N]` with the matching closer. Both spellings were already in use (`docs/spec/types.md`, existing tests). |
| 4 | Logos qvec syntax | `qvec[N]` only, per `docs/LOGOS_GRAMMAR_v0_1.md`. The grammar's optional dimension is narrowed to required. |
| 5 | Omitted dimension allowed anywhere? | No. No authority ever defined `32`; it was a parser fallback. `N >= 1` and must fit `usize`. |
| 6 | Where is a default specified? | Nowhere (no default exists). |
| 7 | What owns builtin/helper names? | The private `BUILTIN_NAMESPACE` table in `sm-front/src/lib.rs`, one entry per name with a `BuiltinNamePolicy`. |
| 8 | Reserved names | Application boundary (`args_read`, `stdin_read_text`, `stdout_write`, `stderr_write`, `path_inspect`, `fs_read_text`, `fs_write_text`, `time_duration_ms`); `qtruth_and/or/not/impl`; `len`, `push`, `prepend`, `contains`, `is_empty`, `pop`, `map_empty`, `map_contains`, `map_get`, `map_set`, `print`, `to_text`, `random_seed`, `random_next_i32`. |
| 9 | Can users shadow builtins? | Only `UserShadowable` names: math `sin cos tan sqrt abs pow` (user-first contract #1653/#1750) and `assert` (user-first in typecheck and IR via `is_builtin_assert_name`). |
| 10 | How do qtruth calls stay phase-consistent? | `build_fn_table` rejects user `qtruth_*`, so any `qtruth_*` call reaching `sm-ir` is the language intrinsic. Lowering is unchanged; its name match is a consumer of the registry, not an admission authority. QTruth truth tables and IR opcodes are unchanged. |
| 11 | Model-B Logos contract | Unchanged. Logos is a declarative/inspection profile with no compile, VM or SemCode path (`tests/ssf_logos_profile_boundary.rs` stays green). |
| 12 | Logos Import | Supported and owned by `sm-sema` (option A). The parser preserves every accepted directive in `LogosProgram::imports` (verbatim text, span, mark) with no added grammar. `sm-sema` consumes exactly those nodes via `parse_import_directive`, and an unusable preserved directive fails with `E0239` instead of being skipped. SSF-09 Decision E outcomes (T5–T9) are unchanged. |
| 13 | Does `sm-front` keep alloc/no_std support? | Yes, restored. |
| 14 | Evidence | The alloc check passes; `cargo tree -e features` shows `sm-profile` with only `alloc` (no `std`, no serde) on that path; new CI step `Check sm-front (alloc only)` in `check-no-std`. |
| 15 | Intentional differences from C1 | Every new rejection maps to an issue: duplicate parameters (#1636); invalid, missing, empty or mismatched qvec dimensions (#1640–#1643); a second Logos System (#1644); an unusable preserved Logos Import (#1645); invalid discarded expressions (#1652); user functions named after Reserved builtins (#1654, #1655). |
| 16 | Preserved exactly | All repository sources and examples (none declare a Reserved name or use a bare/invalid qvec); math/`assert` user shadowing; builtin call typing and lowering; Decision E admission outcomes; Logos inspection; `std` builds. |

## Cross-crate changes

- `sm-sema`: the loader and export/cycle passes consume `LogosProgram::imports` instead of re-scanning raw source (#1645). `parse_import_directives` is now a convenience wrapper over the single `parse_import_directive`, and is not used by the loader. One fixture that paired raw text containing `Import` with a `LogosProgram` parsed from different text was corrected.
- `sm-ir`: not modified. The structural guard `builtin_namespace_registry_covers_every_dispatched_name` scans its name matches.
- `.github/workflows/ci.yml`: one `check-no-std` step (#1666).

## Out of scope / residual

- Logos `Pulse`/`Profile` directives are still skipped to end of line. This is the same accept-and-drop pattern, but it is outside #1645 and is reported as a new finding.
- `sm-sema` alloc qualification (#1701) is a later batch.
