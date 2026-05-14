# Semantic Hello Observation Policy

Status: planning document for `#477`

## 1. Purpose

This document plans the future verifier/runtime/capability policy for Hello controlled observation.

- docs-only
- no verifier implementation
- no runtime implementation
- no capability implementation
- no audit implementation
- no SemCode emission changes
- no CLI integration
- no accepted runtime behavior

## 2. Non-goals

- no Rust code
- no verifier admission implementation
- no VM/runtime execution
- no capability/effect admission implementation
- no audit implementation
- no SemCode opcode implementation
- no bytecode format change
- no CLI pipeline integration
- no `smc check` / `compile` / `verify` / `run` / `run-smc` integration
- no accepted golden SemCode
- no runtime output
- no observe effect implementation
- no print implementation
- no general I/O
- no README/examples rewrite
- no Linguist readiness
- no UI / Workbench / I70

## 3. Observation Subject

The policy subject is the future conceptual operation:

```semantic
request_observation_text "Hello, World!"
```

This is:

- a controlled observation request
- not stdout
- not print
- not generic I/O
- not file / network / stdin
- not host-dependent output

## 4. Admission Principle

- observation must be admitted only through explicit verifier/runtime/capability policy
- default should be deny until capability is present
- observation must be deterministic in ordering
- observation must be auditable or explicitly classified as audit-deferred before implementation
- no silent output
- no effect without policy

## 5. Capability Model Planning

| capability concern | planned policy | deferred decision |
|---|---|---|
| observation sink | explicit sink required | exact sink representation |
| text payload | literal-only in first slice | broader payload forms |
| output ordering | follows SemCode / IR order | replay and interleaving rules |
| audit event | required or audit-deferred | final audit schema |
| runtime host channel | not stdout by default | concrete host channel binding |
| budget / cost | enforced by policy | exact limits and counters |
| deterministic replay | planned | exact replay contract |
| denied capability behavior | trap or fail admission | final denial surface |

## 6. Verifier Boundary

Future verifier checks must ensure:

- the operation is controlled observation, not generic I/O
- the payload is a text literal
- the required capability / sink policy is declared or present

This document does not implement a final verifier rule.
This document does not implement opcode admission.

## 7. Runtime Boundary

Future runtime must:

- not print directly by default
- route observation through a controlled sink
- preserve deterministic order
- record or expose observation according to future audit policy
- not permit arbitrary host I/O
- not treat observation as a return value

## 8. Audit Boundary

- audit event shape is deferred
- possible audit fields:
  - operation kind
  - text payload hash or payload reference
  - sink id
  - deterministic sequence index
  - capability id / policy id
- no audit schema is finalized
- no audit implementation is added

## 9. Failure Behavior Planning

Future failure classes:

- missing observation capability
- sink unavailable
- verifier rejects observation operation
- runtime denies sink
- audit required but unavailable
- text encoding invalid
- budget exceeded
- nondeterministic sink configuration

No failure behavior is implemented here.

## 10. Rejected Policies

| rejected policy | reason |
|---|---|
| allow observation as stdout by default | collapses controlled observation into host output |
| treat observe as print | canonizes legacy output vocabulary |
| allow generic I/O capability | widens the policy boundary too far |
| skip verifier policy for Hello | bypasses the admission layer |
| allow unaudited effect silently | removes audit visibility |
| treat observation as function return | loses controlled observation semantics |
| make Hello special-case bypass verifier/runtime policy | creates an exception path that undermines the model |

## 11. Future Implementation Sequence

Recommended next steps:

- `M-HELLO-5B` - docs(verify): decide verifier admission rule for Hello observation
- `M-HELLO-5C` - docs(runtime): decide runtime observation sink model
- `M-HELLO-5D` - docs(audit): decide audit event shape
- `M-HELLO-5E` - tests(policy): add pending policy fixtures
- `M-HELLO-6A` - implementation only after policy acceptance

## 12. Acceptance Checklist

- policy subject documented
- controlled observation boundary preserved
- admission principle recorded
- capability model planning table added
- verifier boundary documented
- runtime boundary documented
- audit boundary documented
- failure behavior planning added
- rejected policies listed
- no code changes
- no verifier / runtime / capability / audit implementation
- no SemCode / opcode changes
- no accepted runtime behavior
- `#477` remains open
