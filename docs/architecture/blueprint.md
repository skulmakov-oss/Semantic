# Semantic Architecture Blueprint

Semantic is a deterministic, contract-driven compiler/runtime system.

Semantic is split into two architectural products:

- `Semantic Core`: source, frontend, semantic analysis, IR, optimization, SemCode, verifier, VM.
- `PROMETHEUS Integration`: ABI, capabilities, semantic runtime, state/rule execution, audit.

The core execution rule is:

`frontend -> semantics -> lowering -> IR passes -> emit -> verifier -> VM`

Current repository state:

- active core crates: `sm-front`, `sm-sema`, `sm-ir`, `sm-emit`, `sm-verify`, `sm-runtime-core`, `sm-vm`, `sm-profile`
- active integration crates: `prom-abi`, `prom-cap`, `prom-gates`, `prom-runtime`, `prom-state`, `prom-rules`, `prom-audit`
- `prom-*` crates remain separate from compiler and VM internals

Current repository limits that remain within the published stable `v1` line:

- richer `fx` arithmetic remains intentionally narrower than the `f64` surface in the canonical Rust-like execution path
- optimizer surface is fixed to `sm-ir` for the current `v1`; no separate `sm-opt` owner is planned inside the current baseline
- SemCode format surface is fixed to `sm-ir` for the current `v1`; `sm-emit` remains a producer facade over that contract
- SemCode version and capability widening must stay explicit and additive; older
  admitted header families remain fixed once published on `main`
- public CLI surface is fixed to `smc-cli` for the current `v1`; root `smc` remains a thin process entrypoint over the canonical CLI owner
- PROMETHEUS `v1` scope is fixed to the current narrow ABI/capability/gate boundary; wider planned calls remain post-`v1`
- stable release packaging policy remains narrower than the long-term planned distribution story

Current release-line state:

- `main` carries the active narrow `v1` stable line
- release validation runs through boundary guards, public API inventory, runtime matrix/goldens, and the release-bundle verifier
- published stable releases are expected to ship `smc.exe`, `svm.exe`, and a bundled Windows archive

Planned post-stable UI application boundary:

- UI is treated as a host/runtime boundary product, not as an extension of the
  compiler core
- the planned first-wave owner split is:
  - `prom-ui` for boundary types, capabilities, and admitted UI operation IDs
  - `prom-ui-runtime` for desktop lifecycle, event polling, frame ownership,
    and backend adapter implementation
  - `examples/` or `apps/` for demo consumers, not runtime ownership
- the first-wave UI contract is expected to stay narrow:
  - single-window desktop lifecycle
  - input polling
  - frame begin/end ownership
  - minimal draw-command surface
- no graphics backend library becomes a language-level promise in the first
  wave; backend choice remains an internal runtime detail
- the planning checkpoint for this track is
  `docs/roadmap/language_maturity/ui_application_boundary_scope.md`

Non-negotiable architecture rules:

- compiler semantics and runtime semantics must stay separate;
- IR is the lowered execution-contract boundary between source semantics and
  emitted binary contract;
- no frontend/AST structures may leak into runtime;
- runtime operates only on lowered execution contracts;
- verifier enforces structure before execution and is a public admission layer,
  not an internal VM detail;
- orchestration layers must call verified VM entrypoints only and must not
  become alternate execution authorities;
- no silent contract mutation is allowed across source, IR, SemCode, or VM
  layers;
- determinism is mandatory across all stages;
- VM mechanics and semantic state/rule logic must stay separate;
- all host effects must cross a formal ABI boundary;
- desktop UI, if admitted, must stay behind an explicit host/runtime boundary
  and must not leak backend ownership into compiler or VM crates.

## Runtime Ownership (Execution Contract)

Status: DONE (v1 slice)

Runtime ownership is implemented as a lowered execution contract, not as a full
ownership system.

### Representation

Ownership is expressed via canonical `AccessPath`:

- `root: SymbolId`
- `components`:
  - `TupleIndex`
  - `Field(SymbolId)` for direct record-field access only

### Semantics

- borrow events register paths in frame-local state
- write operations check overlap against active borrows

Overlap rules:

- exact -> reject
- parent-child -> reject
- child-parent -> reject
- siblings -> allowed

### Lifetime Model

- borrow lifetime is frame-local until frame exit

### Guarantees

- deterministic enforcement
- no frontend structure leakage into runtime
- explicit runtime trap on violation through `BorrowWriteConflict`

### Supported Scope

- tuple paths
- direct record-field paths

### Out Of Scope

- ADT payload paths
- nested generalized path systems
- non-frame-local lifetimes
- partial release
- alias/reborrow models
- indirect field projections beyond direct record-field access
