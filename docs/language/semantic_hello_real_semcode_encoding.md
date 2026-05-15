# Semantic Hello Real SemCode Encoding

Status: planning document for `#477`

## 1. Purpose

This document finalizes the minimal real SemCode encoding decision for future Hello controlled observation.

- docs-only
- no encoder implementation
- no opcode implementation
- no bytecode format change
- no verifier implementation
- no VM/runtime execution
- no capability admission implementation
- no audit implementation
- no CLI / smc integration
- no accepted golden SemCode
- no runtime output

## 2. Non-goals

- no Rust code
- no SemCode encoder changes
- no final opcode implementation
- no VM dispatch implementation
- no verifier admission implementation
- no runtime sink routing
- no capability/effect admission implementation
- no audit storage implementation
- no CLI pipeline integration
- no `smc check` / `compile` / `verify` / `run` / `run-smc` integration
- no accepted golden SemCode
- no runtime output
- no stdout default
- no print implementation
- no general I/O
- no README/examples rewrite
- no Linguist readiness
- no Workbench / UI / I70

## 3. Input Boundary

The future real SemCode input boundary is:

- canonical verbose Hello IR shape
- from `HelloIrModule`
- after parser / sema / lowering
- not from legacy `fn main` / `print` / `return`
- not from the conceptual text fixture directly

The conceptual emitter remains planning-only.

## 4. Canonical Source-to-SemCode Sequence

Source-level semantic shape:

```text
entry hello {
    state boot = T
    require boot == T
    observe "Hello, World!"
    complete T
}
```

Minimal real SemCode planning sequence:

```text
declare_local_quad boot = T
require_quad_eq boot T
observe_text_literal "Hello, World!"
complete_quad T
```

`observe_text_literal` is a proposed real SemCode-level operation name for planning.

- it is not a final opcode ID in this PR
- it is not implemented in this PR
- it must remain controlled observation, not stdout / print / generic I/O

## 5. Encoding Decision Table

| conceptual line | real SemCode-level decision | notes |
|---|---|---|
| `declare_local_quad boot = T` | deterministic local / register initialization | local quad declaration becomes a stable real encoding primitive |
| `require_quad_eq boot T` | verifier-visible assertion / require operation | requirement remains explicit and ordered before observation |
| `request_observation_text "Hello, World!"` | controlled `observe_text_literal` | stays controlled observation, not stdout / print / generic I/O |
| `complete_quad T` | deterministic completion / result marker | completion remains explicit and ordered after observation |

## 6. Minimal Operation Set

| operation | role | implementation status | future owner |
|---|---|---|---|
| `declare_local_quad` | local quad initialization | decided as future minimal SemCode-level operation | emitter / SemCode encoding |
| `require_quad_eq` | explicit precondition | decided as future minimal SemCode-level operation | verifier admission |
| `observe_text_literal` | controlled observation request | decided as future minimal SemCode-level operation | VM/runtime bridge and capability/audit boundary where relevant |
| `complete_quad` | explicit completion marker | decided as future minimal SemCode-level operation | emitter / SemCode encoding |

## 7. Opcode Strategy Decision

- no numeric opcode IDs are assigned in this PR
- opcode ID allocation must happen in a later implementation PR
- no existing opcode must be repurposed silently
- observation must not be encoded as generic host call by default
- observation must not be encoded as stdout / print
- future encoding may use a dedicated observation opcode or a tightly typed admitted host-call form
- this document recommends dedicated controlled observation semantics unless later constraints force otherwise

## 8. Const / Data Boundary

- `"Hello, World!"` should live as a text literal / const-table entry in the future real encoding
- payload must be deterministic
- no formatting / interpolation
- no implicit scalar-to-text conversion
- no host-dependent payload generation
- payload identity for audit should be derived from literal / ref / hash policy later

## 9. Verifier Requirements

Future verifier must check:

- operation sequence is valid
- requirement precedes observation
- observation precedes completion
- observation operation is controlled text literal only
- no generic I/O operation is present
- capability / audit policy metadata is present or deferred according to policy
- no stdout / print fallback exists
- opcode / operand boundaries are valid
- const text reference is valid

No verifier rule is implemented here.

## 10. Runtime / Capability / Audit Linkage

Future linkage is:

- runtime routes admitted observation to an explicit sink
- capability model controls observation sink permission
- audit event records controlled observation metadata
- none of these are wired here
- SemCode operation must carry enough information later for these policies to act deterministically

## 11. Denied / Rejected Encodings

| rejected encoding | reason |
|---|---|
| encode observation as stdout | collapses controlled observation into host output |
| encode observation as print | reintroduces legacy output vocabulary |
| encode observation as generic I/O | loses the controlled observation boundary |
| encode observation as raw host call without policy type | removes explicit admission semantics |
| bypass verifier because payload is fixed | erases admission control |
| bypass capability because Hello is harmless | creates a special-case security bypass |
| bypass audit because payload is fixed | loses auditability for external observation |
| encode observation as function return value | changes observation into result flow |
| use wall-clock or host-dependent payload source | breaks determinism |
| reuse unrelated opcode ID | creates silent contract drift |

## 12. Future Implementation Sequence

Recommended next steps:

- `M-HELLO-7B` - semcode(hello): add gated real Hello SemCode emission skeleton, not admitted
- `M-HELLO-7C` - verify(hello): add verifier admission for controlled `observe_text_literal`
- `M-HELLO-7D` - tests(verify): reject stdout / print / generic I/O encodings
- `M-HELLO-8A` - runtime: route admitted observation to explicit sink skeleton
- `M-HELLO-8B` - capability: gate observation sink route
- `M-HELLO-8C` - audit: record observation audit event or audit-deferred decision
- `M-HELLO-9A` - CLI pipeline smoke path only after verifier / runtime / capability / audit are accepted

## 13. Acceptance Checklist

- real SemCode input boundary documented
- minimal operation set decided
- conceptual-to-real mapping table added
- opcode strategy documented without numeric opcode IDs
- const / data boundary documented
- verifier requirements documented
- runtime / capability / audit linkage documented
- rejected encodings listed
- no Rust code changes
- no SemCode encoder / opcode implementation
- no verifier / runtime / capability / audit implementation
- no CLI / smc integration
- no accepted runtime behavior
- `#477` remains open

## 14. M-HELLO-7B Implementation Boundary

- isolated real SemCode-level emission skeleton exists
- emits typed / planning SemCode-level operations only
- no real SemCode bytes
- no opcode IDs
- no bytecode format changes
- no verifier admission
- no VM/runtime routing
- no capability / audit behavior
- no CLI pipeline integration
- `#477` remains open

## 15. M-HELLO-7C Implementation Boundary

- isolated verifier admission for Hello real SemCode-level skeleton exists
- admits controlled `observe_text_literal` only
- no real SemCode byte verification
- no numeric opcode IDs
- no bytecode format changes
- no production verifier pipeline integration
- no VM/runtime routing
- no capability / audit behavior
- no CLI / smc integration
- `#477` remains open

## 16. M-HELLO-7D Implementation Boundary

- negative verifier tests exist for forbidden observation encodings
- stdout / print / io.write / file / network / stdin are rejected
- opcode / bytecode markers are rejected
- tests target isolated verifier-admission skeleton only
- no production verifier pipeline integration
- no real SemCode byte verification
- no VM/runtime/capability/audit behavior
- no CLI / smc integration
- `#477` remains open

See also: [`semantic_hello_controlled_observation_encoding.md`](semantic_hello_controlled_observation_encoding.md)
