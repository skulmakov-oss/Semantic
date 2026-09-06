# Runtime Ownership Specification

Status: frozen tuple+record+sequence+(ADT Borrow-only) v1 (#1718)
Source ownership owner: `sm-front`
IR ownership owner: `sm-ir`
SemCode transport owner: `sm-ir`
Admission owner: `sm-verify`
Execution consumer: `sm-vm`
Shared runtime vocabulary owner: `sm-runtime-core`

## Purpose

This document freezes the current runtime ownership contract for tuple
paths, direct record field paths, `Sequence` static-index paths, and ADT
payload paths in `Borrow` events only.

Current supported slice:

- tuple `AccessPath`
- direct record field `AccessPath`
- `Sequence` static-index `AccessPath` (`SequenceIndexStatic`), both `Borrow`
  and `Write` (#1718)
- ADT payload `AccessPath` (`AdtPayload`), `Borrow` events only - `Write`
  events carrying an `AdtPayload` component are not part of this contract
  and are rejected unconditionally at admission, under every header,
  regardless of capability (#1718)
- `Borrow` and `Write` ownership events for tuple, direct record field, and
  `Sequence` static-index paths; `Borrow`-only for ADT payload paths
- frame-local borrow lifetime
- structural `OWN0` admission before execution
- runtime write rejection for overlapping borrowed tuple, direct record
  field, `Sequence` static-index, and ADT payload paths

This document does not claim a general runtime borrow checker, general ADT
mutation, alias analysis, lifetimes/regions, or Rust-equivalent borrow
checking.

## Public Position

SSF-08 (`docs/roadmap/stable_foundation/ssf08_ownership_position_decision.md`)
formally selects **Position A — bounded deterministic VM language** as the
public ownership/memory claim for Semantic: ownership protects admitted
value paths and runtime invariants inside this bounded, frame-local model.
Semantic does not claim Rust-equivalent lifetime inference, region
inference, general borrow checking, unrestricted alias analysis, or
systems-language memory-safety equivalence. The decision record also names
known implementation gaps inside this frozen ownership slice (OWN0 root
identity and event-timing correctness, since closed - see Lane 2, #1709/
#1724/#1725/#1726/#1891) that were open repair work, not positioning
questions.

## Layer Separation

The current ownership pipeline is intentionally split:

- source/frontend semantics decide where borrow capture exists in source
- IR/lowering preserves only the canonical execution-path contract
- SemCode transports that lowered ownership metadata in `OWN0`
- verifier admits or rejects the `OWN0` payload structurally
- VM enforces the runtime write-path guard over admitted tuple, direct
  record field, `Sequence` static-index, and ADT payload (`Borrow`-only)
  paths

Important rule:

- runtime ownership must not depend on frontend AST or parser-only pattern
  structures

## Canonical Runtime Path

Current runtime path form:

- `AccessPath { root: SymbolId, components: Vec<PathComponent> }`

Current supported component kinds:

- `TupleIndex(u16)`
- `Field(SymbolId)` for direct named record field projection only
- `SequenceIndexStatic(u32)`, admitted in both `Borrow` and `Write` events
  (#1718)
- `AdtPayload { variant: SymbolId, index: u16 }`, admitted in `Borrow` events
  only - a `Write` event carrying this component is rejected
  unconditionally, not merely deferred (#1718)

Current ordering rule:

- path components are ordered from root to leaf
- the same path must serialize, admit, and execute in the same deterministic
  order

Important boundary:

- this document does not approve indirect field selection or broader path
  normalization

## Supported Behavior

Current supported runtime ownership behavior covers tuple paths, direct
record field paths, `Sequence` static-index paths (`Borrow` and `Write`),
and ADT payload paths (`Borrow` only).

Borrow lifetime v0:

- a borrowed tuple, direct record field, `Sequence` static-index, or ADT
  payload path becomes active for the current frame
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
- sibling `Sequence` static indices
- sibling ADT payload variants/indices (evaluated only against a real
  `Borrow`; no genuine `Write(AdtPayload)` artifact is admissible to
  exercise this case in practice - see `## Explicitly Unsupported`)

## Frontend And Lowering Contract

Current source/frontend contract:

- tuple, direct record field, and `Sequence` static-index borrow/write
  intent, and ADT payload borrow intent, must not be erased before lowering
- lowering must preserve enough ownership metadata to recover:
  - borrow event kind
  - write event kind
  - canonical `AccessPath`
  - direct record field projection as `Field(SymbolId)` when present
  - `Sequence` static-index projection as `SequenceIndexStatic(u32)` when
    present
  - ADT payload projection as `AdtPayload { variant, index }` when present
    (`Borrow` events only)

Current lowering contract:

- runtime ownership transport is path-based, not AST-pattern-based
- the lowered contract uses canonical `AccessPath` rooted in `SymbolId`

## SemCode Transport Contract

Current binary contract:

- tuple-only ownership metadata is transported through `SEMCOD11`
- direct record-field `Borrow`/`Write` transport is emitted through `SEMCOD12`
- `Sequence` static-index `Borrow`/`Write` transport, and ADT payload
  `Borrow`-only transport, are emitted through `SEMCOD21` (#1718)
- the ownership section tag is `OWN0`
- each event carries:
  - event kind (`Borrow` or `Write`)
  - root `SymbolId`
  - ordered path components

Current transport scope:

- tuple-only path components admitted end-to-end through `SEMCOD11`
- direct record-field `Borrow`/`Write` transport, encoded as `Field(SymbolId)`,
  admitted end-to-end through `SEMCOD12`
- `Sequence` static-index `Borrow`/`Write` transport, encoded as
  `SequenceIndexStatic(u32)`, admitted end-to-end through `SEMCOD21`
- ADT payload `Borrow`-only transport, encoded as
  `AdtPayload { variant, index }`, admitted end-to-end through `SEMCOD21`;
  the identical component in a `Write` event is rejected unconditionally,
  under every header
- deterministic event order
- `CAP_OWNERSHIP_PATHS` remains the tuple ownership capability family
- `CAP_OWNERSHIP_FIELD_PATHS` marks direct record-field ownership path transport
- `CAP_OWNERSHIP_SEQUENCE_PATHS` marks `Sequence` static-index ownership path
  transport (#1718)
- `CAP_OWNERSHIP_ADT_BORROW_PATHS` marks ADT payload `Borrow`-only ownership
  path transport (#1718)

## Verifier Admission Contract

Current verifier responsibility:

- validate `OWN0` section structure
- validate admitted ownership event kinds
- validate tuple, direct record-field, `Sequence` static-index, and ADT
  payload path payload shape
- validate header/capability consistency for ownership transport,
  independently re-derived from decoded content for each path family
  (#1718) - not delegated to `sm-format`'s own decode-time gate
- admit valid `Borrow(Field)`, `Write(Field)`, `Borrow(SequenceIndexStatic)`,
  `Write(SequenceIndexStatic)`, and `Borrow(AdtPayload)` payloads structurally
- reject `Write(AdtPayload)` unconditionally, before and independent of
  capability accounting, under every header (#1718)
- reject malformed or unsupported ownership payload before execution

Current verifier non-goal:

- do not evaluate borrow overlap policy
- do not execute runtime ownership semantics

## VM Enforcement Contract

Current VM responsibility:

- keep a frame-local set of active borrowed tuple, direct record field,
  `Sequence` static-index, and ADT payload paths
- consume admitted ownership metadata only
- reject overlapping writes at runtime for the supported tuple, direct
  record field, `Sequence` static-index, and ADT payload (`Borrow`-only)
  slice
- surface ownership conflicts through `BorrowWriteConflict`

Current VM non-goals:

- no partial borrow release
- no inter-frame borrow persistence
- no advanced alias inference

Legacy artifact execution (#1891 Checkpoint W2F):

- decoding and verifier admission of a legacy (pre-`SEMCOD20`/rev21) Write
  ownership event remain unchanged - such an event carries a path but no
  executable anchor, and the verifier still structurally admits it
- this decode/verifier compatibility does **not** imply runtime execution
  support: a Write-bearing artifact below the anchor-bearing revision has no
  exact execution authority for the VM to enforce against, and is
  deterministically rejected at runtime construction
  (`crates/sm-vm/src/semcode_vm.rs`, `build_vm_program_view_from_decoded`)
  rather than executed via any residual cursor, root-matching, or scan-ahead
  fallback

## Explicitly Unsupported

The current implemented runtime ownership contract does not claim support for:

- `Write(AdtPayload)` - ADT payload paths are supported in `Borrow` events
  only; a `Write` event carrying an `AdtPayload` component is rejected
  unconditionally at admission, under every header, regardless of
  capability. No source syntax reaches this today (the language has no
  mutable ADT-payload reassignment), and promoting it later requires a new,
  separately authorized contract change - not an incidental relaxation of
  this document or of #1718's own mechanism (see
  `docs/roadmap/stable_foundation/ssf08_1718_path_family_contract_decision.md`)
- `Map`/schema paths
- partial borrow release before frame exit
- advanced aliasing or region reasoning
- inter-frame borrow persistence
- indirect field selection or broader smart path normalization
- general ADT mutation beyond `Borrow`-only payload access, general alias
  analysis, lifetimes/regions, or Rust-equivalent borrow checking

## Honesty Rule

If a behavior is not implemented across:

- lowering
- SemCode transport
- verifier admission
- VM enforcement

then it must remain unsupported here rather than being implied by analogy.
