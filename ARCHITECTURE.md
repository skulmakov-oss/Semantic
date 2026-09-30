# Semantic Architecture

Status: current top-level architecture entrypoint

This file is the short architectural map for the current repository.

The canonical detailed architecture lives in:

- `docs/architecture/blueprint.md`
- `docs/architecture/module_ownership_map.md`
- `docs/architecture/dependency_boundary_rules.md`
- `docs/spec/index.md`

## System Shape

Semantic is a deterministic, contract-driven compiler/runtime system with a
separate PROMETHEUS integration layer.

The repository is split into two architectural products:

- `Semantic Core`
  - source frontend
  - semantic analysis
  - IR and lowering
  - SemCode emission
  - verifier admission
  - VM execution
- `PROMETHEUS Integration`
  - ABI
  - capabilities
  - gates
  - semantic state
  - rule execution
  - orchestration
  - audit and replay metadata

## Primary Flow

The canonical staged execution path is:

`frontend -> semantics -> lowering -> IR passes -> emit -> verify -> VM`

Public rule:

- standard execution is verifier-first
- VM execution is not the owner of source semantics
- host/runtime effects stay outside compiler ownership and cross explicit
  boundaries

## Current Owner Split

### Core crates

- `sm-profile` - parser/profile policy
- `sm-front` - lexer, parser, AST, and source-surface typing
- `sm-sema` - semantic analysis, diagnostics, import/export policy
- `sm-ir` - lowering, IR, optimizer ownership, SemCode contract ownership
- `sm-emit` - producer-facing SemCode facade
- `sm-verify` - SemCode admission contract
- `sm-runtime-core` - shared runtime vocabulary and quotas
- `sm-vm` - verified execution and disassembly
- `smc-cli` - canonical public CLI owner

### PROMETHEUS crates

- `prom-abi` - host ABI
- `prom-cap` - capability policy
- `prom-gates` - gate registry and adapters
- `prom-state` - semantic state ownership
- `prom-rules` - rule and agenda ownership
- `prom-runtime` - orchestration over verified entrypoints
- `prom-audit` - audit and replay metadata
- `prom-ui`, `prom-ui-runtime`, `prom-ui-backend-native`, `prom-ui-iced-adapter`,
  `prom-ui-demo` - narrow UI boundary layer (retired contour: code preserved,
  not on the active roadmap; see `docs/roadmap/ui_workbench_studio_retirement.md`)

## Architecture Rules

- one public concept has one owner
- frontend structures must not leak into runtime execution
- IR is the lowered contract boundary between source semantics and binary
  execution
- verifier admission remains a public contract layer, not an internal VM detail
- integration crates must not become alternate execution authorities
- no silent contract mutation is allowed across source, IR, SemCode, verifier,
  or VM layers

## Compatibility Perimeter

The repository still retains a narrow legacy compatibility perimeter.

These compatibility artifacts are not canonical owners of current public
contracts.

## Reading Order

Use this order when reloading repository context:

1. `docs/spec/index.md`
2. `docs/architecture/blueprint.md`
3. `docs/architecture/module_ownership_map.md`
4. `docs/architecture/dependency_boundary_rules.md`
5. `docs/roadmap/repository_truth_audit_2026-04-22.md` if present on the
   current branch
