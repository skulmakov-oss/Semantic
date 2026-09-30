# Runtime Boundary Hardening

Status: completed post-stable anti-drift checkpoint

This checkpoint is completed and now serves as frozen baseline history on
current `main`.

## Goal

Freeze the current execution-boundary rules between verifier admission, shared
runtime vocabulary, VM execution, and PROMETHEUS orchestration on `main`.

This checkpoint exists to stop runtime authority from drifting implicitly
between:

- `sm-verify`
- `sm-runtime-core`
- `sm-vm`
- `prom-runtime`

The point of this track is boundary hardening, not new runtime behavior.

## Canonical Reading

The standard execution route is:

`emit -> verify -> run_verified_* -> execute`

In that reading:

- `sm-verify` owns SemCode admission before standard execution
- `sm-runtime-core` owns shared runtime vocabulary used across execution crates
- `sm-vm` owns execution mechanics, frames, quotas, and runtime trap surfacing
- `prom-runtime` orchestrates verified execution sessions only and must not
  become a second execution authority

## Current Landed State

The current `main` already includes:

- verifier-first canonical public VM entrypoints, alongside explicitly named
  raw diagnostic and compatibility APIs that are not trusted execution routes
- explicit `ExecutionConfig` / `ExecutionContext` runtime wiring
- runtime quota enforcement in `sm-vm`
- frame-local runtime ownership tracking and `BorrowWriteConflict`
  enforcement for the admitted tuple + direct record-field slice
- PROMETHEUS execution-session wiring that composes verified entrypoints
  rather than redefining VM semantics

That is enough to freeze the current boundary rules.

## Included In This Freeze

- verifier-before-execution as the public route
- `sm-runtime-core` as shared runtime vocabulary owner only
- `sm-vm` as the sole owner of execution mechanics and runtime trap surfaces
- `prom-runtime` as orchestration glue over verified entrypoints only
- explicit non-goal list for what orchestration does not own

## Explicit Non-Goals

This checkpoint does not include:

- new VM opcodes
- new execution contexts
- new host-call families
- runtime ownership widening beyond the admitted tuple + direct record-field
  slice
- retries, rollback, or compensation semantics
- alternate raw execution authorities outside the verified path

## Freeze Rules

- standard public execution must continue to require verifier admission
- raw or test-only helpers must not become public orchestration boundaries
- `prom-runtime` must not redefine VM trap, quota, or ownership semantics
- `sm-runtime-core` stays a vocabulary crate, not a second VM or verifier owner
- admitted runtime trap surfaces remain owned by `sm-vm`
- execution-boundary changes require explicit spec and compatibility review, not
  silent drift

## Completed Reading

This checkpoint is now complete because:

- `docs/spec/verifier.md`, `docs/spec/vm.md`, and `docs/spec/runtime.md`
  reflect the same verified-only execution route
- architecture docs describe `prom-runtime` as orchestration only
- roadmap docs point to this file as the completed runtime hardening checkpoint
- public VM/runtime docs list the current admitted ownership trap surface
- no document implies a second execution authority outside verified VM entrypoints
