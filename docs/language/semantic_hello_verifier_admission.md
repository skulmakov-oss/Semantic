# Semantic Hello Verifier Admission

Status: planning document for `#477`

## 1. Purpose

This document decides the future verifier admission rule for Hello controlled observation.

- docs-only
- no verifier code
- no runtime implementation
- no capability implementation
- no audit implementation
- no SemCode / opcode changes
- no CLI integration
- no accepted runtime behavior

## 2. Non-goals

- no Rust code
- no verifier implementation
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

## 3. Verifier Subject

The verifier subject is the conceptual SemCode planning operation:

```semantic
request_observation_text "Hello, World!"
```

This is:

- conceptual only for now
- not final opcode
- not bytecode format
- not stdout
- not print
- not generic I/O

## 4. Admission Rule Decision

A Hello observation operation is verifier-admissible only if all are true:

- operation kind is controlled observation text request
- payload is a deterministic text literal
- payload encoding is valid under the future text encoding policy
- observation sink capability is declared or present in effective capability context
- operation does not claim generic I/O
- operation is ordered after requirements and before completion
- operation is not used as a return value
- required audit classification is present or explicitly audit-deferred by policy
- resource / budget policy is available or explicitly deferred by policy

This is a future rule decision, not implemented.

## 5. Deny Cases

| case | future verifier result | reason |
|---|---|---|
| missing observation capability | deny admission | capability required but absent |
| generic stdout operation | deny admission | collapses controlled observation into host output |
| print operation | deny admission | legacy output vocabulary is not canonical |
| generic I/O operation | deny admission | violates controlled observation boundary |
| non-text payload | deny admission | first slice is text-literal only |
| nondeterministic payload | deny admission | observation must be deterministic |
| observation before requirement | deny admission | ordering rule violation |
| observation after completion | deny admission | completion terminates the entry slice |
| observation used as return value | deny admission | observation is not a return channel |
| missing audit classification when audit required | deny admission | audit policy must be respected |
| unknown sink | deny admission | sink must be explicit or admitted |
| unsupported text encoding | deny admission | payload encoding must be valid |

## 6. Required Verifier Evidence

| evidence | source | status |
|---|---|---|
| operation kind | conceptual observation request | decided |
| payload type | text literal | decided |
| payload encoding | future text encoding policy | required later |
| sink capability | effective capability context | required later |
| ordering index | source / IR sequence | decided |
| audit classification | policy / admission context | required later |
| budget class | verifier/runtime policy | deferred |
| deterministic replay class | replay / audit policy | deferred |

## 7. Capability Interaction

- verifier must not invent capability
- verifier checks effective capability context
- capability policy remains separate from verifier implementation
- missing capability should fail admission or trap according to later policy
- verifier must not silently downgrade to stdout / print

## 8. Audit Interaction

- verifier must know whether audit is required or audit-deferred
- audit event schema remains deferred
- verifier must not silently admit unaudited observation if policy requires audit
- audit metadata shape is not implemented here

## 9. Ordering / Determinism

- verifier must preserve conceptual sequence order
- requirement must precede observation
- observation must precede completion
- no reordering / folding allowed across observation
- deterministic sequence index may be required later
- replay policy remains deferred

## 10. Rejected Verifier Shortcuts

| shortcut | reason rejected |
|---|---|
| auto-admit Hello because it is harmless | bypasses the verifier admission boundary |
| treat Hello observation as stdout | erases the controlled observation model |
| treat observe as print | canonizes legacy output vocabulary |
| bypass capability for Hello | creates a special-case hole in policy |
| bypass audit for Hello | breaks audit visibility expectations |
| fold requirement away | removes the admission / precondition boundary |
| verify parser / sema shape only and skip operation policy | omits the actual admission decision |
| allow generic I/O because payload is fixed | widens the policy boundary too far |

## 11. Future Implementation Sequence

Recommended next steps:

- `M-HELLO-5C` - docs(runtime): decide runtime observation sink model
- `M-HELLO-5D` - docs(audit): decide audit event shape
- `M-HELLO-5E` - tests(policy): add pending verifier / capability policy fixtures
- `M-HELLO-6A` - verify: add pending verifier admission model behind isolated path
- `M-HELLO-6B` - runtime / capability planning or skeleton only after docs accepted

## 12. Acceptance Checklist

- verifier subject documented
- future admission rule listed
- deny cases listed
- required verifier evidence table added
- capability interaction documented
- audit interaction documented
- ordering / determinism documented
- rejected verifier shortcuts listed
- no code changes
- no verifier / runtime / capability / audit implementation
- no SemCode / opcode changes
- no accepted runtime behavior
- `#477` remains open
