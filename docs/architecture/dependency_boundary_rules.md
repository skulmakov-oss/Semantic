# Dependency And Boundary Rules

Project zones:

- `Construction`: `sm-front`, `sm-sema`, `sm-ir`, `sm-emit`, `sm-profile`
- `Execution`: `sm-verify`, `sm-runtime-core`, `sm-vm`
- `Integration`: `prom-abi`, `prom-cap`, `prom-runtime`, `prom-state`, `prom-rules`, `prom-gates`, `prom-audit`
- `Shared Foundation / Contract Leaf`: neutral foundational tier (`sm-format` precedent)

Current pending ownership notes:

- `sm-front` consolidates lexer, AST, and source-level type-check in a single crate; the originally planned `sm-lexer` / `sm-ast` split was not carried forward; a future split would require an explicit decision and code move
- optimizer surface is owned by `sm-ir` in the current `v1` baseline; a future `sm-opt` split would require an explicit follow-up decision and code move
- SemCode format contract is owned by `sm-ir` in the current `v1` baseline; `sm-emit` remains a producer-facing facade and compatibility layer
- public CLI contract is owned by `smc-cli` in the current `v1` baseline; root `smc` is a thin entrypoint wrapper over that owner
- the retained non-owning TON618 compatibility perimeter (`ton618_core`, `ton618-core`) must not grow into second owners
- canonical internal diagnostic carrier: pending new dedicated leaf crate named `sm-diagnostic` (`crates/sm-diagnostic`) in the neutral `Shared Foundation / Contract Leaf` tier (precedent: `sm-format`; not part of Construction, Execution, Integration, Host/CLI, or TON618 compatibility perimeter). Ownership decision is frozen (Position B) and carrier contract is frozen (`docs/roadmap/stable_foundation/ssf09_canonical_carrier_contract.md`); implementation is pending and NOT authorized under current task envelope (`dependency_changes: false`).

Compatibility checkpoint:

- the remaining TON618-named perimeter is tracked in
  `docs/roadmap/language_maturity/ton618_compatibility_perimeter_scope.md`

Allowed flow:

`Construction -> Execution -> Integration`

Neutral shared contract leaves (`Shared Foundation / Contract Leaf`) sit below and cross-cut these zones without owning zone-specific compiler/runtime semantics:

```text
                  sm-diagnostic
             (Contract Leaf / Tier 0)
                 ▲     ▲     ▲
                 │     │     │
         Construction Execution Host/Tooling
```

Consumers may depend inward on the shared diagnostic carrier owner (`sm-diagnostic`). The diagnostic carrier owner MUST NOT depend on:
`sm-front`, `sm-sema`, `sm-ir`, `sm-emit`, `sm-verify`, `sm-runtime-core`, `sm-vm`, `smc-cli`, `prom-*`, or `ton618-core`.

Boundary rules:

- construction crates must not depend on VM/runtime state or PROMETHEUS internals;
- execution crates must not reach back into parser/sema internals;
- downstream consumers (`sm-sema`, `sm-ir`, `smc-cli`) MUST consume `sm-front`'s canonical cross-grammar `SurfaceAuthority`/`resolve_surface_authority` (Decision F, 2026-09-14) for any decision that requires comparing a Logos `GrammarAdmission` against a RustLike `GrammarAdmission`; they do not own grammar selection and must not destructure/compare a `GrammarAdmission` pair independently — mechanically enforced by `tests/surface_authority_guard.rs`;
- shared runtime vocabulary stays in `sm-runtime-core`; it must not become a
  second execution authority;
- integration crates must not rewrite compiler or VM semantics;
- integration crates must reach execution only through verified VM entrypoints;
- all host effects must cross ABI and capability checks;
- public contracts require versioning, tests, and spec updates.

Current enforcement note:

- these boundary rules are repository policy now
- CI now enforces baseline ownership and dependency guards for root shims, SemCode owner alignment, optimizer owner alignment, and crate-level forbidden dependency checks
- broader graph visualization, API diff, and release gating remain pending `M6`

Immediate debt markers:

- `ParserProfile` outside `sm-profile` is architectural debt;
- richer `fx` arithmetic beyond the current literal/value transport path is architectural debt;
- `smc-cli`'s `resolve_project_route` (`#1919`) is one tracked, measured pre-Decision-F local cross-grammar resolver, pending its own migration PR to consume `sm-front::resolve_surface_authority` directly — tracked by exact count (not a path allowlist) in `tests/surface_authority_guard.rs`'s `EXPECTED_LEGACY_VIOLATION_COUNTS`;
