# Semantic Hello IR Representation

Status: planning document for `#477`

## 1. Purpose

This document decides the proposed Hello IR representation for later implementation.

- docs-only
- no IR code
- no lowering implementation
- no SemCode emission
- no verifier/runtime/capability changes
- no accepted runtime behavior

## 2. Non-goals

- no Rust code
- no IR type implementation
- no lowering implementation
- no SemCode emission
- no verifier admission
- no VM/runtime execution
- no capability/effect admission
- no CLI pipeline integration
- no accepted golden SemCode
- no runtime output
- no observe effect
- no print implementation
- no general I/O
- no README/examples rewrite
- no Linguist readiness
- no UI / Workbench / I70

## 3. Input Representation

Future IR input is:

- `HelloCheckedFile`
- `HelloSemaReport`
- canonical verbose shape only
- `canonical_shape = true`
- `architecture_bearing = true`
- `secondary_shape = false`

The minimal observe secondary shape is excluded from the first IR representation.

## 4. Proposed IR Model

Planning-only IR structures:

- `HelloIrModule`
- `HelloIrEntry`
- `HelloIrStmt`
- `HelloIrLocalQuad`
- `HelloIrRequireQuadEq`
- `HelloIrObserveText`
- `HelloIrCompleteQuad`

These names are planning labels, not Rust implementation commitments.

## 5. IR Structure Table

| IR node | Source origin | Required fields | Purpose | First-slice constraints |
|---|---|---|---|---|
| `HelloIrModule` | full checked Hello file | entry | isolated Hello IR container | one entry only |
| `HelloIrEntry` | `entry HelloWorld` | name, body | architecture-bearing entry boundary | no params, no imports, no modules |
| `HelloIrLocalQuad` | `state boot: quad = T` | symbol, quad literal | local quad state | immutable, local only |
| `HelloIrRequireQuadEq` | `require boot == T` | symbol, expected quad literal | structural precondition / admission gate candidate | no effects, declared local symbol only |
| `HelloIrObserveText` | `observe "Hello, World!"` | text literal, observation class | controlled observation request candidate | text literal only, not stdout |
| `HelloIrCompleteQuad` | `complete T` | completion quad literal | explicit completion marker | quad literal only |

## 6. Ordering Model

The first Hello IR slice preserves source order:

1. local state
2. requirement
3. observation
4. completion

- no reordering
- no optimization
- no constant folding
- no effect movement
- observation cannot move across requirement
- completion terminates the entry body

## 7. Symbol Model

- state symbols are local to the entry
- `require` references a local state symbol
- no global symbols
- no imports/modules
- no symbol interning commitment here unless existing IR requires it later
- symbol representation remains implementation detail

## 8. Observation Model

- observation is represented as a controlled observation candidate
- not stdout
- not generic I/O
- not effect execution
- later verifier / runtime / capability policy decides admission
- audit metadata is not finalized here

## 9. Completion Model

- completion is explicit
- completion carries quad literal
- no return value model
- no exceptions
- no early return
- no implicit completion in the first IR slice

## 10. Rejected IR Alternatives

| alternative | reason rejected for first slice |
|---|---|
| lowering `observe` as `print` | canonizes legacy output vocabulary and hides controlled observation |
| lowering observation as generic stdout | collapses observation into host I/O |
| lowering Hello directly to existing function body IR | loses Hello-specific boundary shape |
| treating `complete` as `return` | reintroduces legacy completion vocabulary |
| representing `require` as ordinary assertion | blurs precondition/admission semantics |
| accepting minimal observe shape as first canonical IR | excludes the architecture-bearing verbose shape |
| emitting SemCode directly from parser / sema | skips the planned lowering boundary |

## 11. Relationship to Existing IR

This document does not decide whether the future implementation uses:

- existing general IR with a small extension
- a separate temporary Hello IR
- a lowering adapter from Hello AST to existing IR

It only decides the semantic shape that must be preserved.

## 12. Future Implementation Sequence

Recommended next steps:

- `M-HELLO-4C` - ir/lowering: add isolated Hello IR structs and lowering skeleton, no SemCode emission
- `M-HELLO-4D` - tests(ir): add Hello IR shape tests, no SemCode
- `M-HELLO-4E` - docs(semcode): decide Hello SemCode representation
- `M-HELLO-5A` - docs(policy): plan verifier/runtime/capability observation policy

## 13. Acceptance Checklist

- IR representation proposed
- canonical verbose shape selected
- minimal observe shape excluded / deferred
- IR node table added
- ordering model defined
- symbol model defined
- observation model defined
- completion model defined
- rejected alternatives listed
- relationship to existing IR clarified
- no code changes
- no IR / lowering / SemCode implementation
- no verifier / runtime / capability changes
- no accepted runtime behavior
- `#477` remains open

## 14. M-HELLO-4C Implementation Boundary

- isolated Hello IR structs and lowering skeleton exist
- no SemCode emission
- no verifier / runtime / capability behavior
- no normal compiler pipeline integration
- pending fixtures remain outside accepted runtime truth
