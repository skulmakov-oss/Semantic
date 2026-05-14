# Semantic Hello SemCode Representation

Status: planning document for `#477`

## 1. Purpose

This document decides the proposed SemCode representation for future Hello emission planning.

- docs-only
- no SemCode emission
- no opcode implementation
- no verifier changes
- no VM/runtime changes
- no capability/effect changes
- no CLI pipeline integration
- no accepted golden SemCode

## 2. Non-goals

- no Rust code
- no SemCode encoder changes
- no final opcode implementation
- no verifier admission
- no VM/runtime execution
- no capability/effect admission
- no audit implementation
- no CLI pipeline integration
- no `smc check` / `compile` / `verify` / `run` / `run-smc` integration
- no accepted golden SemCode
- no runtime output
- no observe effect
- no print implementation
- no general I/O
- no README/examples rewrite
- no Linguist readiness
- no UI / Workbench / I70

## 3. Input Boundary

Future SemCode input is:

- `HelloIrModule`
- canonical verbose shape only
- ordered IR body:
  - `HelloIrLocalQuad`
  - `HelloIrRequireQuadEq`
  - `HelloIrObserveText`
  - `HelloIrCompleteQuad`

The minimal observe secondary shape is excluded from the first SemCode plan.

This document does not accept the secondary shape.

## 4. Proposed SemCode Conceptual Sequence

Conceptual, non-opcode sequence:

- `declare_local_quad boot = T`
- `require_quad_eq boot T`
- `request_observation_text "Hello, World!"`
- `complete_quad T`

This is:

- conceptual sequence only
- not final opcode names
- not bytecode format
- not executable truth

## 5. SemCode Planning Table

| IR node | Conceptual SemCode role | Required data | Deferred / not decided |
|---|---|---|---|
| `HelloIrLocalQuad` | declare local quad | local symbol and literal | final opcode IDs, encoding layout, const pool handling |
| `HelloIrRequireQuadEq` | require quad equality | require symbol and expected literal | capability token shape, audit event shape |
| `HelloIrObserveText` | request controlled observation text | observation text literal and observation class | final opcode IDs, encoding layout, sink token shape |
| `HelloIrCompleteQuad` | complete explicit quad result | completion quad literal | final opcode IDs, encoding layout, termination marker shape |

## 6. Const / Data Boundary

- text literal storage is not decided here.
- possible options:
  - inline text literal
  - const pool text entry
  - symbol / id reference
- no final choice in this PR unless already obvious from existing SemCode policy.
- quad literals should remain compact / deterministic.
- string encoding and escaping must be specified before implementation.

## 7. Capability / Effect Boundary

- observation must not become generic stdout.
- SemCode representation must preserve controlled observation intent.
- verifier / runtime / capability policy must decide whether observation is admitted.
- capability token / observation sink representation is deferred.
- audit event representation is deferred.

## 8. Determinism Boundary

- instruction / order sequence must preserve IR order.
- no reordering across requirement / observation / completion.
- observation order must be deterministic.
- no host-dependent output semantics in this plan.

## 9. Failure Boundary

Future failure classes:

- IR accepted but SemCode emission unsupported
- text literal encoding unsupported
- observation capability unavailable
- verifier rejects observation request
- runtime sink unavailable
- audit policy missing
- invalid non-canonical IR shape

No failure behavior is implemented here.

## 10. Rejected Alternatives

| alternative | reason rejected |
|---|---|
| emit `print` | canonizes legacy output vocabulary and hides controlled observation |
| emit generic stdout write | collapses observation into host I/O |
| encode observation as ordinary string return | loses controlled observation semantics |
| skip requirement in emitted form | removes precondition / admission boundary |
| fold state / require into constant truth and remove them | erases architecture-bearing Hello shape |
| emit from parser directly | skips the planned lowering boundary |
| accept minimal observe as canonical SemCode first slice | excludes the canonical verbose shape |

## 11. Future Implementation Sequence

Recommended next steps:

- `M-HELLO-4F` - semcode tests plan / expected conceptual shape fixtures, no emission
- `M-HELLO-4G` - semcode emitter skeleton for Hello IR, gated / not runtime-admitted
- `M-HELLO-5A` - docs(policy): verifier/runtime/capability observation policy
- `M-HELLO-5B` - verifier admission plan for Hello observation
- `M-HELLO-5C` - runtime observation sink plan

## 12. Acceptance Checklist

- SemCode conceptual representation documented
- input boundary documented
- minimal observe shape excluded / deferred
- conceptual sequence listed
- planning table added
- const / data boundary documented
- capability / effect boundary preserved
- determinism boundary documented
- failure boundary listed
- rejected alternatives listed
- no code changes
- no SemCode emission / opcode implementation
- no verifier / runtime / capability changes
- no accepted runtime behavior
- `#477` remains open
