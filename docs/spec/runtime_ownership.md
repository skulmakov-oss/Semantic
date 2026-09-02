# Runtime Ownership Specification

Status: frozen tuple+record v0
Source ownership owner: `sm-front`
IR ownership owner: `sm-ir`
SemCode transport owner: `sm-ir`
Admission owner: `sm-verify`
Execution consumer: `sm-vm`
Shared runtime vocabulary owner: `sm-runtime-core`

## Purpose

This document freezes the current runtime ownership contract for tuple paths
and direct record field paths.

Current supported slice:

- tuple `AccessPath`
- direct record field `AccessPath`
- `Borrow` and `Write` ownership events for both supported path families
- frame-local borrow lifetime
- structural `OWN0` admission before execution
- runtime write rejection for overlapping borrowed tuple and direct record
  field paths

This document does not claim a general runtime borrow checker.

## Public Position

SSF-08 (`docs/roadmap/stable_foundation/ssf08_ownership_position_decision.md`)
formally selects **Position A — bounded deterministic VM language** as the
public ownership/memory claim for Semantic: ownership protects admitted
value paths and runtime invariants inside this bounded, frame-local model.
Semantic does not claim Rust-equivalent lifetime inference, region
inference, general borrow checking, unrestricted alias analysis, or
systems-language memory-safety equivalence. The decision record also names
known implementation gaps inside this frozen tuple/record slice (OWN0 root
identity and event-timing correctness) that remain open repair work, not
positioning questions.

## Layer Separation

The current ownership pipeline is intentionally split:

- source/frontend semantics decide where borrow capture exists in source
- IR/lowering preserves only the canonical execution-path contract
- SemCode transports that lowered ownership metadata in `OWN0`
- verifier admits or rejects the `OWN0` payload structurally
- VM enforces the runtime write-path guard over admitted tuple and direct
  record field paths

Important rule:

- runtime ownership must not depend on frontend AST or parser-only pattern
  structures

## Canonical Runtime Path

Current runtime path form:

- `AccessPath { root: SymbolId, components: Vec<PathComponent> }`

Current supported component kinds:

- `TupleIndex(u16)`
- `Field(SymbolId)` for direct named record field projection only

Current ordering rule:

- path components are ordered from root to leaf
- the same path must serialize, admit, and execute in the same deterministic
  order

Important boundary:

- this document does not approve indirect field selection or broader path
  normalization

## Supported Behavior

Current supported runtime ownership behavior covers tuple paths and direct
record field paths.

Borrow lifetime v0:

- a borrowed tuple or direct record field path becomes active for the current
  frame
- the active borrowed-path set is cleared when that frame exits

Current runtime write rule:

- a write must be rejected if its target path overlaps an active borrowed path

Current overlap cases that must reject:

- exact path equality
- borrowed parent, written child
- borrowed child, written parent

Current allowed case:

- sibling tuple paths
- sibling direct record fields

## Frontend And Lowering Contract

Current source/frontend contract:

- tuple and direct record field borrow/write intent must not be erased before
  lowering
- lowering must preserve enough ownership metadata to recover:
  - borrow event kind
  - write event kind
  - canonical `AccessPath`
  - direct record field projection as `Field(SymbolId)` when present

Current lowering contract:

- runtime ownership transport is path-based, not AST-pattern-based
- the lowered contract uses canonical `AccessPath` rooted in `SymbolId`

## SemCode Transport Contract

Current binary contract:

- tuple-only ownership metadata is transported through `SEMCOD11`
- direct record-field `Borrow`/`Write` transport is emitted through `SEMCOD12`
- the ownership section tag is `OWN0`
- each event carries:
  - event kind (`Borrow` or `Write`)
  - root `SymbolId`
  - ordered path components

Current transport scope:

- tuple-only path components admitted end-to-end through `SEMCOD11`
- direct record-field `Borrow`/`Write` transport, encoded as `Field(SymbolId)`,
  admitted end-to-end through `SEMCOD12`
- deterministic event order
- `CAP_OWNERSHIP_PATHS` remains the tuple ownership capability family
- `CAP_OWNERSHIP_FIELD_PATHS` marks direct record-field ownership path transport

## Verifier Admission Contract

Current verifier responsibility:

- validate `OWN0` section structure
- validate admitted ownership event kinds
- validate tuple and direct record-field path payload shape
- validate header/capability consistency for ownership transport
- admit valid `Borrow(Field)` and `Write(Field)` payloads structurally
- reject malformed or unsupported record ownership payload before execution

Current verifier non-goal:

- do not evaluate borrow overlap policy
- do not execute runtime ownership semantics

## VM Enforcement Contract

Current VM responsibility:

- keep a frame-local set of active borrowed tuple and direct record field paths
- consume admitted ownership metadata only
- reject overlapping writes at runtime for the supported tuple and direct
  record field slice
- surface ownership conflicts through `BorrowWriteConflict`

Current VM non-goals:

- no partial borrow release
- no inter-frame borrow persistence
- no advanced alias inference

## Explicitly Unsupported

The current implemented runtime ownership contract does not claim support for:

- ADT payload paths
- schema paths
- partial borrow release before frame exit
- advanced aliasing or region reasoning
- inter-frame borrow persistence
- indirect field selection or broader smart path normalization

## Honesty Rule

If a behavior is not implemented across:

- lowering
- SemCode transport
- verifier admission
- VM enforcement

then it must remain unsupported here rather than being implied by analogy.
