# First-Class Closures Full Scope

Status: completed M8.4 first-wave post-stable subtrack
Related roadmap package:
`docs/roadmap/language_maturity/m8_everyday_expressiveness_roadmap.md`

## Goal

Introduce the first admitted first-class closure surface for Semantic without
silently widening the published `v1.1.1` line and without opening generics,
traits, or async/runtime machinery ahead of schedule.

This is a forward-only language-maturity subtrack for current `main`. It is not
a claim that first-class closures already exist on the published stable line.

## Stable Baseline Before This Track

The current stable line already freezes these facts:

- short lambdas exist only as immediate-call or pipeline sugar
- short lambdas are not first-class values in the public language contract
- short lambdas are capture-free in the current stable source contract
- the published `v1.1.1` line does not claim closure values, closure types, or
  closure runtime carriers

That baseline remains the source of truth until this subtrack explicitly lands
its widened contract on `main`.

## Included In This Track

- explicit ownership of one first-wave closure value family
- a narrow closure type spelling for admitted source positions
- deterministic immutable capture policy for admitted closure values
- direct invocation of admitted closure values
- local binding, parameter, and return transport for admitted closures
- docs/spec/tests/compatibility wording for the widened contract

## Explicit Non-Goals

- multi-argument closure syntax
- async closures or coroutine semantics
- mutable capture or by-reference capture semantics
- trait/protocol-based callable abstractions
- generic closure signatures or higher-kinded callable machinery
- host-ABI widening for closure values
- silent widening of published `v1.1.1`

## Intended Wave Order

### Wave 0 — Governance

- scope checkpoint
- roadmap/milestone/plan linkage

### Wave 1 — Owner Layer

- closure family ownership
- closure type/literal/capture metadata inventory
- deterministic closure value-model boundaries
- explicit typecheck/lowering gap diagnostics before executable admission

### Wave 2 — Source Admission

- parser admission
- sema/type admission
- explicit diagnostics for unsupported closure forms

### Wave 3 — Execution Path

- lowering/runtime/VM closure carrier path
- deterministic immutable capture materialization
- direct invocation path for admitted closure values

### Wave 4 — Freeze

- docs/spec/tests/compatibility freeze

## Suggested Narrow PR Plan

1. PR 1: scope checkpoint
2. PR 2: owner-layer closure family surface
3. PR 3: parser/sema/type admission
4. PR 4: runtime/VM closure carrier and invocation path
5. PR 5: freeze and close-out

## Initial First-Wave Reading

The first-wave closure contract is intentionally narrow:

- one parameter only
- one closure value family only
- immutable capture only
- direct call syntax only
- no trait-based callable overloading

That keeps the track additive over the current short-lambda sugar without
turning it into a general abstraction system in one step.

## Final First-Wave Reading

Completed `M8.4` first-wave contract on current `main`:

- one first-wave closure family is explicit and owned in the frontend owner
  layer
- `Closure(T -> U)` is admitted in declared source type positions
- standalone first-class closure literals are admitted in contextual
  closure-typed positions
- immutable capture inventory is recorded for admitted standalone closure
  literals
- one canonical runtime closure carrier is admitted for that first-wave family
- direct invocation of admitted closure values is admitted
- closure execution is kept narrow:
  - exactly one positional argument
  - immutable capture only
  - no closure equality
  - no PROMETHEUS host-ABI closure transport

Still intentionally not included after close-out:

- multi-argument closure syntax
- async or mutable-capture closure semantics
- trait/protocol-based callable abstractions
- generic closure signatures
- host-ABI widening for closure values

## Acceptance Reading

This subtrack is done only when:

- one first-wave closure family is explicit and inspectable
- closure values, immutable capture, and direct invocation agree on one
  deterministic first-wave model
- docs/spec/tests describe the same admitted baseline
- published `v1.1.1` and widened `main` are explicitly distinguished

## Non-Commitments After Close-Out

Even after this first wave lands, the repository still does not claim:

- generic callable abstractions
- trait/protocol-based closure polymorphism
- async/generator closure semantics
- multi-argument or variadic closure families
- that first-class closures were already part of the published `v1.1.1` line

## Close-Out Reading

`M8.4` is now frozen as completed first-wave baseline history.

The next language-facing candidate inside the `M8` package is `M9.1
Generics`.

## Post-Close-Out Addendum (SSF-07 #1861)

This track's "Included In This Track" list (above) named "local binding,
parameter, and return transport for admitted closures" as the M8.4
first-wave closure-transport surface. It did not name record fields or ADT
payloads — neither as included nor as an explicit non-goal — because
whether an admitted closure could be stored inside a record or ADT was
never independently investigated at the time. That silence is not itself an
error in this document; the M8.4 historical text above is left unedited as
the accurate record of what M8.4 actually scoped and proved.

SSF-07 #1861 ("storage type admission has a fail-open catch-all for
unsupported type families") separately discovered that `validate_record_declarations`/
`validate_adt_declarations` never routed field/payload types through the
frontend's storage-admission authority (`ensure_storage_type_supported`) at
all — only through `ensure_type_resolved`, which proves a nominal reference
resolves, not that the referenced type is itself storage-admitted. Closing
that gap genuinely widened `ensure_storage_type_supported`'s scope from
local bindings to also cover record fields and ADT payloads, which meant
`Type::Closure`'s admission for those two positions needed its own proof
rather than inheriting M8.4's local-binding evidence by assumption. That
proof was produced directly: record-field and ADT-payload closure storage
now has dedicated end-to-end coverage across the full pipeline — frontend
typecheck (`storage_admission_record_closure_field_typechecks_end_to_end`,
`storage_admission_adt_closure_payload_typechecks_end_to_end` in
`crates/sm-front/src/typecheck.rs`), IR lowering to the expected
`MakeClosure`/`MakeRecord`/`MakeAdt`/`RecordGet`/`ClosureCall` opcode
sequence (`storage_admission_record_closure_field_lowers_to_working_ir`,
`storage_admission_adt_closure_payload_lowers_to_working_ir` in
`crates/sm-ir/src/legacy_lowering.rs`), and full VM execution producing the
mathematically correct result (`storage_admission_record_closure_field_vm_executes_correctly`,
`storage_admission_adt_closure_payload_vm_executes_correctly` in
`crates/sm-vm/src/lib.rs`).

Record-field and ADT-payload closure transport is therefore now a
deliberately proven and admitted SSF-07 widening of closure storage scope
beyond the M8.4 first-wave baseline, not a silent extension. See
`docs/spec/foundation_source_profile_v1.md` for the current normative
storage-admission contract, which documents this alongside the same
evidence-first treatment applied to every other admitted composite
(`Tuple`, `Sequence`, `Map`, `Measured`, `Option`, `Result`, `Record`,
`Adt`).
