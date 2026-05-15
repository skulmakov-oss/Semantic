# Semantic Hello Audit Event

Status: planning document for `#477`

## 1. Purpose

This document decides the future audit event shape for Hello controlled observation.

- docs-only
- no audit implementation
- no runtime implementation
- no sink implementation
- no verifier implementation
- no capability implementation
- no SemCode / opcode changes
- no CLI integration
- no accepted runtime behavior

## 2. Non-goals

- no Rust code
- no audit implementation
- no audit storage implementation
- no VM/runtime execution
- no observation sink implementation
- no verifier admission implementation
- no capability/effect admission implementation
- no SemCode opcode implementation
- no bytecode format change
- no CLI pipeline integration
- no `smc check` / `compile` / `verify` / `run` / `run-smc` integration
- no accepted golden SemCode
- no runtime output
- no stdout default
- no print implementation
- no general I/O
- no README/examples rewrite
- no Linguist readiness
- no UI / Workbench / I70

## 3. Audit Subject

The future audit subject is an admitted controlled observation event derived from:

```semantic
request_observation_text "Hello, World!"
```

This is:

- audit applies only after future verifier / capability / runtime admission
- audit does not itself admit the event
- audit does not execute output
- audit is not stdout
- audit is not print
- audit is not generic I/O

## 4. Audit Event Shape Decision

Proposed future event shape fields:

- event_kind
- operation_kind
- observation_class
- payload_ref or payload_hash
- sink_id
- capability_id / policy_id
- deterministic_sequence_index
- runtime_context_id
- verifier_admission_ref
- audit_policy_class
- timestamp policy / logical time policy

Clarifications:

- exact Rust struct is not decided here
- exact storage format is not decided here
- exact hash algorithm is deferred
- wall-clock timestamps are not required and may be avoided for determinism

## 5. Audit Event Table

| field | purpose | first-slice decision | deferred |
|---|---|---|---|
| event_kind | identifies an observation event | decided | final enum shape |
| operation_kind | names the controlled observation text operation | decided | exact serialization form |
| observation_class | distinguishes controlled observation | decided | class taxonomy expansion |
| payload_ref / payload_hash | records payload without requiring raw host output | decided | final privacy / storage policy |
| sink_id | ties event to the routed sink | required later | concrete identifier shape |
| capability_id / policy_id | ties event to capability / policy context | required later | exact policy representation |
| deterministic_sequence_index | preserves ordering | decided | replay / interleaving contract |
| runtime_context_id | binds event to a runtime session or frame | required later | final context identity model |
| verifier_admission_ref | links to admission result | required later | exact reference form |
| audit_policy_class | classifies audit requirement | required later | final policy taxonomy |
| logical_time / timestamp policy | expresses time policy without nondeterminism by default | deferred | wall-clock vs logical-time choice |

Required points:

- event_kind = observation event
- operation_kind = controlled observation text
- observation_class = controlled
- payload should not require unsafe host output
- deterministic_sequence_index preserves ordering
- timestamp policy must not introduce nondeterminism by default

## 6. Payload Handling

- raw text payload storage is not decided here
- possible options:
  - payload hash only
  - payload reference
  - redacted payload
  - literal payload for dev-only sink
- future implementation must decide privacy / security policy
- audit payload must not become a hidden output channel
- audit payload must not bypass sink / capability policy

## 7. Determinism / Ordering

- audit event order must follow runtime observation sequence
- deterministic sequence index is preferred over host wall-clock time
- no host-dependent ordering
- no reordering across requirement / observation / completion
- replay policy remains deferred but required before implementation

## 8. Capability / Verifier Linkage

- audit event should link to verifier admission result or policy id
- audit event should link to capability / sink policy where available
- audit must not invent capability
- audit must not silently admit events
- missing linkage behavior is deferred

## 9. Failure Behavior Planning

Future failure classes:

- audit required but sink unavailable
- audit required but storage unavailable
- payload hash / ref generation fails
- deterministic sequence index unavailable
- verifier admission reference missing
- capability / policy id unavailable
- audit policy conflict
- audit payload policy violation

No failure behavior is implemented here.

## 10. Rejected Audit Shortcuts

| shortcut | reason rejected |
|---|---|
| skip audit because Hello is harmless | removes audit visibility from admitted observation |
| log raw output as stdout | collapses audit into host output |
| use wall-clock timestamp as sole ordering source | introduces nondeterminism |
| audit after host output instead of before / with routed observation | breaks the observation / audit boundary |
| let audit record imply capability admission | reverses the policy dependency |
| silently drop audit event | hides policy failure instead of surfacing it |
| store payload as hidden side-channel output | creates an unauthorized output channel |
| special-case Hello outside audit policy | creates an exception path that undermines the model |

## 11. Future Implementation Sequence

Recommended next steps:

- `M-HELLO-5E` - tests(policy): add pending verifier / capability / runtime / audit policy fixtures
- `M-HELLO-6A` - verify: add pending verifier admission model behind isolated path
- `M-HELLO-6B` - runtime: add observation sink interface skeleton, no host output
- `M-HELLO-6C` - capability: add observation capability model skeleton
- `M-HELLO-6D` - audit: add audit event skeleton only after policy fixtures

## 12. Acceptance Checklist

- audit subject documented
- audit event shape decision recorded
- audit event field table added
- payload handling boundary documented
- determinism / ordering documented
- capability / verifier linkage documented
- failure behavior planning added
- rejected audit shortcuts listed
- no code changes
- no audit / runtime / sink / capability / verifier implementation
- no SemCode / opcode changes
- no accepted runtime behavior
- `#477` remains open

## 13. M-HELLO-6D Boundary Note

- isolated observation audit event skeleton exists
- no audit storage implementation
- no AuditTrail integration
- no runtime routing
- no host output
- no stdout default
- no verifier / capability behavior
- no CLI pipeline integration
- `#477` remains open
