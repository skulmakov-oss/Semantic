# Semantic Hello Lowering Boundary

Status: planning document for `#477`

## 1. Purpose

This document plans the future lowering boundary for Hello.

- docs-only
- no IR changes
- no lowering implementation
- no SemCode changes
- no verifier/runtime/capability behavior
- no accepted runtime behavior

## 2. Non-goals

- no Rust code
- no IR implementation
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
- no README/examples rewrite
- no Linguist readiness
- no UI / Workbench / I70

## 3. Input Boundary

Future lowering input is the isolated Hello checked AST:

- `HelloCheckedFile`
- `HelloSemaReport`

For the first architecture-bearing lowering candidate, the input is the canonical verbose shape only.

The minimal observe secondary shape remains non-canonical and deferred unless explicitly scoped later.

## 4. Lowering Candidates

| Hello source element | Future IR/lowering role | First-slice plan | Deferred |
|---|---|---|---|
| `entry HelloWorld` | isolated entry function / module boundary candidate | canonical verbose lowering target | broader entry syntax, multi-entry, modules |
| `state boot: quad = T` | local quad binding candidate | lowered as a bounded local state declaration | mutable state, heap state, richer types |
| `require boot == T` | precondition / admission check candidate | lowered as a structural precondition gate | arbitrary expressions, side effects |
| `observe "Hello, World!"` | controlled observation request candidate | lowered as controlled observation request, not general I/O | generic stdout, file/stdin/network I/O |
| `complete T` | explicit completion result candidate | lowered as explicit completion marker | arbitrary return values, implicit completion semantics |

## 5. SemCode Planning Boundary

| concern | allowed in first planning slice | explicitly not allowed |
|---|---|---|
| quad literal | yes | final opcode commitment |
| text literal | yes | general I/O payload semantics |
| local state | yes | broad state-memory model |
| requirement | yes | final verifier semantics |
| observation | yes | generic stdout emission |
| completion | yes | implicit control-flow return rules |
| capability | no | capability policy commitment |
| audit | no | final audit protocol |
| runtime output | no | accepted runtime behavior |
| golden SemCode | no | executable truth claim |

## 6. Placeholder Opcode Warning

This PR must not define final opcodes.

If placeholder names are mentioned, they are planning placeholders only, not SemCode commitments:

- `HELLO_STATE_QUAD`
- `HELLO_REQUIRE_QUAD_EQ`
- `HELLO_OBSERVE_TEXT`
- `HELLO_COMPLETE_QUAD`

## 7. Effect / Capability Boundary

- `observe` must not lower as generic stdout.
- observation lowering must later pass through verifier / runtime / capability design.
- no capability policy is changed here.
- no effect admission is changed here.
- output ordering / audit policy remains future work.

## 8. Failure Boundary

Future failure classes to plan for:

- sema accepted but lowering unsupported
- observation lowering blocked by capability policy
- invalid checked shape
- unsupported secondary shape
- missing completion for architecture-bearing shape
- text literal encoding decision pending
- audit sink unavailable

No failure behavior is implemented here.

## 9. Future Implementation Sequence

Recommended next steps:

- `M-HELLO-4B` - docs(ir): decide exact Hello IR representation
- `M-HELLO-4C` - ir/lowering: add isolated Hello lowering skeleton, no SemCode emission
- `M-HELLO-4D` - semcode: plan Hello SemCode representation
- `M-HELLO-5A` - verifier/runtime/capability observation policy plan
- `M-HELLO-5B+` - implementation only after policy acceptance

## 10. Acceptance Checklist

- lowering boundary documented
- input boundary documented
- canonical verbose shape selected as first lowering candidate
- minimal observe shape deferred / classified
- source element to lowering role table added
- SemCode planning boundary table added
- placeholder opcode warning added
- effect / capability boundary preserved
- failure boundary listed
- no code changes
- no IR / lowering / SemCode implementation
- no verifier / runtime / capability changes
- no accepted runtime behavior
- `#477` remains open
