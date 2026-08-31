# Generics Full Scope

Status: completed M9.1 first-wave post-stable subtrack

> **Current status (corrected — see Post-Close-Out Correction sections
> below):** this "completed" self-report did not hold. Frontend generic
> *function* declaration admission and call-site type inference/substitution
> are implemented end-to-end (#1634, #1648, #1649). Generic record/ADT
> *declarations* are admitted at the frontend only for zero type parameters
> after correction (#1650) — the original claimed applied-type-argument
> surface was never implemented. Wave 3 (IR monomorphisation / lowering) was
> never implemented for generic functions either: the IR/SemCode boundary
> now deterministically rejects every generic function declaration rather
> than silently erasing its type parameters (#1717). No runtime generic
> dispatch or monomorphisation exists anywhere in this repository today.

Related roadmap package:
`docs/roadmap/language_maturity/m8_everyday_expressiveness_roadmap.md`

## Goal

Introduce the first admitted parametric polymorphism surface for Semantic
without silently widening the published `v1.1.1` line and without opening
trait-based abstraction, async, or runtime machinery ahead of schedule.

This is a forward-only language-maturity subtrack for current `main`. It is not
a claim that generics already exist on the published stable line.

## Why This Track Exists

Semantic now has text, packages, collections, and first-class closures on
`main`. All four foundations are completed first-wave baselines. The next
barrier to practical code reuse is the inability to write type-parametric
definitions. Without generics:

- `Sequence(T)` cannot be written generically over user-defined types
- `Closure(T -> U)` cannot be composed generically
- `Option(T)` and `Result(T, E)` cannot be opened to user-defined type families
- record and ADT definitions must be duplicated for each concrete type

This track opens the minimum first-class generic surface without mixing in
trait dispatch, async abstractions, or higher-kinded types.

## Decision Check

- [x] This is a new explicit post-stable track with its own scope decision
- [x] This does not silently widen published `v1.1.1`
- [x] This is one stream, not a mixture of multiple tracks
- [x] This can be closed with a clear done-boundary

## Stable Baseline Before This Track

The current stable line already freezes these facts:

- there are no type parameters in the public language contract
- all concrete types (`i32`, `u32`, `f64`, `fx`, `bool`, `quad`, `text`,
  `Sequence(T)`, `Closure(T -> U)`) use fixed or structurally admitted forms
- `Option(T)` and `Result(T, E)` exist as standard library forms but are not
  user-parameterisable in the published stable baseline
- record and ADT definitions take no type parameters in the published stable
  line
- published `v1.1.1` does not claim user-defined generic types or generic
  functions

That baseline remains the source of truth until this subtrack explicitly lands
its widened contract on `main`.

## Included In This Track

- one first-wave type-parameter family for functions and record/ADT definitions
- a narrow type-parameter spelling for admitted source positions
- deterministic monomorphisation policy
- generic function definitions and call-site instantiation
- generic record and ADT definitions
- docs/spec/tests/compatibility wording for the widened contract

## Explicit Non-Goals

- higher-kinded types
- variance annotations (covariance, contravariance)
- trait/protocol bounds on type parameters (completed in M9.2)
- associated types or type families
- generic closures beyond what first-wave monomorphisation admits
- specialisation or template-based optimisation
- implicit type-class dispatch
- variadic generics
- lifetime or region annotations
- silent widening of published `v1.1.1`

## Intended Wave Order

### Wave 0 — Governance

- scope checkpoint
- roadmap/milestone/plan linkage

### Wave 1 — Owner Layer

- type-parameter syntax ownership
- generic definition and instantiation metadata inventory
- monomorphisation policy boundaries
- explicit typecheck/lowering gap markers before executable admission

### Wave 2 — Source Admission

- parser admission for type-parameter syntax
- sema/type admission for generic definitions and call-site instantiation
- explicit diagnostics for unsupported generic forms

### Wave 3 — Lowering Path

- IR monomorphisation pass
- lowering of generic definitions to concrete SemCode paths
- verifier and VM compatibility for monomorphised output

### Wave 4 — Freeze

- docs/spec/tests/compatibility freeze

## Suggested Narrow PR Plan

1. PR 1: scope checkpoint
2. PR 2: owner-layer type-parameter surface
3. PR 3: parser/sema/type admission
4. PR 4: IR monomorphisation and lowering path
5. PR 5: freeze and close-out

## Initial First-Wave Reading

The first-wave generic contract is intentionally narrow:

- one type-parameter per definition site only
- monomorphisation only (no runtime generic dispatch)
- no trait/protocol bounds in Wave 1–3
- generic functions, records, and ADTs admitted; generic closures follow from
  monomorphisation automatically
- no implicit coercion across generic boundaries

That keeps the track additive over the current concrete type surfaces without
opening a full abstraction system in one step.

## Acceptance Reading

This track is done only when:

- one first-wave type-parameter family is explicit and inspectable
- generic definitions, monomorphisation, and call-site instantiation agree on
  one deterministic first-wave model
- docs/spec/tests describe the same admitted baseline
- published `v1.1.1` and widened `main` are explicitly distinguished

## Non-Commitments After Close-Out

Even after this first wave lands, the repository still does not claim:

- trait/protocol-based generic bounds or dispatch
- higher-kinded types or type constructors
- variance, lifetimes, or region-based memory semantics
- specialisation or template metaprogramming
- that generics were already part of the published `v1.1.1` line

## Merge Gate

Before closing this track:

- [x] code/tests are green
- [x] spec/docs are synced
- [x] public API or golden snapshots are updated if needed
- [x] compatibility/release-facing wording is honest

## Post-Close-Out Correction (SSF-07 #1650)

This track's "completed" status and its "generic record and ADT
definitions [...] admitted" claim (Included In This Track /
Initial First-Wave Reading, above) did not hold: `RecordDecl`/`AdtDecl`
declaration admission existed, but `Type::Record`/`Type::Adt` never gained
applied type arguments, no source syntax existed to apply them, and every
record literal / ADT constructor already unconditionally rejected a
declaration `TypeVar` — so a generic record/ADT declaration was admitted
while every construction of it already failed, a phase-inconsistent,
false-ready surface rather than the deterministic first-wave model this
document's Acceptance Reading requires. Corrected by narrowing canonical
record/ADT admission to zero type parameters (`build_record_table`/
`build_adt_table`); see `docs/spec/foundation_source_profile_v1.md` for the
current normative wording. This entry is left in place as the historical
record of this track's original scope and self-reported completion state,
not retroactively edited. The Wave 3 (IR monomorphisation / lowering)
portion of this track's claimed completion was separately tracked as
FALSE-READY / INERT-CONTRACT and is corrected below by #1717; #1650 addresses
only the frontend record/ADT declaration-admission and type-identity
portion of this document's overclaim. Generic *function* frontend
declaration admission and call-site type inference/substitution, by
contrast, are genuinely closed (#1634, #1648, #1649) and this correction
does not apply to that portion of the track — but "genuinely closed" here
means the frontend admission/inference surface only, not execution: whether
a generic function's declaration is admitted into executable IR/SemCode at
all is a separate contract, addressed below by #1717, and was not closed by
#1634/#1648/#1649.

## Post-Close-Out Correction (SSF-07 #1717)

This track's "completed" status and its "deterministic monomorphisation
policy" / IR monomorphisation pass claims (Included In This Track / Intended
Wave Order / Wave 3, above) did not hold: no IR monomorphisation or
specialization pass exists in `crates/sm-ir/src/passes/` (only
`StructuralCleanup` and `CrystalFold`), and `crates/sm-ir/src/legacy_lowering.rs`
had no check on a function's `type_params` anywhere prior to this
correction. Whether a generic function's declaration reached IR lowering at
all depended entirely on incidental structure: a type parameter referenced
directly or nested in the function's own `params`/`ret` happened to fail,
because lowering reused the non-generic `canonicalize_declared_type`
(unaware of `type_params`) and that function rejects any `TypeVar` it
encounters — but a type parameter declared and never referenced in its own
signature (e.g. `fn marker<T>(x: i32) -> i32`) had no `TypeVar` for that
incidental path to catch, and lowered as an ordinary, non-generic
`IrFunction` with `type_params` silently discarded. That is genuine type
erasure, not partial monomorphisation support, and it is strictly worse than
the direct/nested cases' accidental rejection because it is *inconsistent*:
whether a generic declaration survived to execute depended on where its own
type parameter happened to be written, not on whether the declaration was
generic at all — the same phase-inconsistent, false-ready pattern #1650
found in record/ADT construction, now found at the IR execution boundary
instead of the frontend construction boundary. Corrected by
`ensure_function_is_ir_concrete`, a single admission check run at
`lower_function_to_ir_with_tables` (the shared choke point every public
`compile_program_to_ir*`/`compile_program_to_semcode*`/`lower_function_to_ir`
entrypoint funnels through) that deterministically rejects every generic
function declaration — used or unused type parameter, called or uncalled,
trait-bounded or not — before any canonicalization or lowering work begins;
see `docs/spec/foundation_source_profile_v1.md` for the current normative
wording. This entry is left in place as the historical record of this
track's original scope and self-reported completion state, not
retroactively edited. No IR monomorphisation, specialization, or runtime
generic dispatch has been implemented — that remains a distinct, larger
undertaking, not attempted here. Generic record/ADT declarations remain
separately narrowed to zero type parameters by #1650 and are unaffected by
this correction, which concerns only generic *function* IR/SemCode
execution admission.
