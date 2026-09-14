# Module Ownership Map

One concept must have one owner module.

Core ownership:

- lexical model: `sm-front` (consolidated frontend crate owns lexer layer)
- AST and syntax model: `sm-front` (consolidated frontend crate owns AST layer)
- cross-grammar source-surface authority: `sm-front` (Decision F, 2026-09-14 — `SurfaceAuthority`/`resolve_surface_authority`; the sole resolver over a `GrammarAdmission` pair, downstream crates consume it and do not re-derive it)
- parser profiles: `sm-profile`
- compiler semantics: `sm-sema`
- IR and lowering: `sm-ir`
- optimization passes: `sm-ir`
- SemCode binary contract: `sm-ir`
- bytecode admission contract: `sm-verify`
- runtime primitives: `sm-runtime-core`
- VM execution mechanics: `sm-vm`
- CLI orchestration: `smc-cli`

Integration ownership:

- host ABI: `prom-abi`
- capability policy: `prom-cap`
- semantic state: `prom-state`
- rule runtime: `prom-rules`
- orchestration: `prom-runtime`
- audit and replay metadata: `prom-audit`

Ownership rules:

- no public contract may keep two long-term owners;
- pending ownership decisions must be marked explicitly rather than implied as settled;
- `sm-front` is the canonical frontend owner crate; the original split into `sm-lexer` + `sm-ast` was not carried forward — lexer, AST, and source-level type-check layers are intentionally consolidated in `sm-front` for the current `v1` baseline; if a later decision splits them out, that requires explicit crate creation with matching code movement and ownership transfer;
- `sm-opt` is not a canonical owner crate for `v1`; optimizer contract lives in `sm-ir` unless a later architecture decision creates a separate owner with matching code movement;
- `sm-emit` is a producer-facing facade in the current `v1` baseline; the SemCode header/opcode/capability contract is owned by `sm-ir`;
- `smc-cli` is the canonical owner of the public CLI contract in the current `v1` baseline; root `smc` and `svm` binaries are process entrypoints and not second CLI owners;
- the retained non-owning TON618 compatibility perimeter (`ton618_core`, `ton618-core`, `ton618_legacy/`) is not a canonical public owner and must not become a second owner for `sm-*` public contracts;
- `sm-runtime-core` owns shared runtime vocabulary only; it does not own
  verifier admission, VM execution mechanics, or orchestration semantics;
- `sm-vm` consumes SemCode but does not own its format contract;
- `sm-verify` owns admission, `sm-vm` executes only admitted code;
- `prom-state` owns semantic state, not `sm-vm`;
- `prom-rules` owns agenda/conflict logic, not `sm-vm`;
- `prom-runtime` orchestrates verified entrypoints but does not own VM
  execution, runtime traps, or quota semantics.
