# Semantic Hello Runtime Observation Sink

Status: planning document for `#477`

## 1. Purpose

This document decides the future runtime observation sink model for Hello controlled observation.

- docs-only
- no runtime implementation
- no sink implementation
- no capability implementation
- no audit implementation
- no verifier implementation
- no SemCode / opcode changes
- no CLI integration
- no accepted runtime behavior

## 2. Non-goals

- no Rust code
- no VM/runtime execution
- no observation sink implementation
- no verifier admission implementation
- no capability/effect admission implementation
- no audit implementation
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

## 3. Runtime Subject

The future runtime subject is an admitted controlled observation event derived from:

```semantic
request_observation_text "Hello, World!"
```

This is:

- received only after future verifier / capability admission
- not stdout
- not print
- not generic I/O
- not return value
- not arbitrary host output

## 4. Sink Model Decision

Runtime should route admitted observation events to an explicit observation sink abstraction.

The sink abstraction should be:

- capability-gated
- deterministic in event ordering
- auditable or audit-deferred by policy
- host-channel neutral
- not hardwired to stdout
- not general file / network / stdin I/O

## 5. Sink Event Shape

| field | purpose | first-slice status | deferred decision |
|---|---|---|---|
| operation kind | identifies the controlled observation operation | decided | final field naming |
| text payload | carries the literal Hello text | decided | payload encoding and storage layout |
| observation class | distinguishes controlled observation from other event classes | decided | class taxonomy expansion |
| sequence index | preserves deterministic order | decided | replay / interleaving contract |
| sink id | identifies the explicit sink instance | required later | concrete identifier shape |
| capability id / policy id | ties routing to admission context | required later | exact policy representation |
| audit correlation id | links runtime event to audit policy | required later | final audit linkage shape |
| runtime context id | binds the event to a runtime session or frame | required later | final context identity model |

Required points:

- operation kind = controlled observation text
- text payload = literal Hello text
- sequence index preserves deterministic order
- sink / capability / audit / context identifiers are deferred

## 6. Runtime Routing Rule

An admitted Hello observation event may be routed only if:

- verifier admission succeeded
- required capability / sink policy is present
- runtime sink is available
- audit requirement is satisfied or explicitly audit-deferred
- event order can be preserved deterministically

If not, runtime must fail according to later failure policy, not silently print.

## 7. Determinism / Ordering

- runtime must preserve observation sequence order
- no reordering across requirement / observation / completion
- no batching that changes visible ordering
- no host-dependent ordering
- deterministic replay policy remains deferred but required before implementation

## 8. Host Channel Boundary

- stdout is not the default sink
- stdout may only be a future host adapter if explicitly admitted by capability / policy
- file / network / stdin are out of scope
- UI / Workbench sinks are out of scope
- runtime sink model must stay host-channel neutral

## 9. Failure Behavior Planning

Future failure classes:

- sink missing
- sink denied by capability
- verifier admission absent
- audit required but unavailable
- sink cannot preserve deterministic order
- text encoding unsupported
- runtime context unavailable
- host adapter unavailable

No failure behavior is implemented here.

## 10. Rejected Runtime Shortcuts

| shortcut | reason rejected |
|---|---|
| print directly to stdout | collapses controlled observation into host output |
| map observe to print | canonizes legacy output vocabulary |
| treat observation as return value | loses the observation sink model |
| route through generic I/O | widens the runtime boundary too far |
| skip capability because Hello is harmless | creates a special-case hole in policy |
| skip audit because payload is fixed | removes audit visibility expectations |
| allow UI / Workbench sink by default | moves host-channel policy into the wrong layer |
| silently drop observation on missing sink | hides policy failure instead of surfacing it |

## 11. Future Implementation Sequence

Recommended next steps:

- `M-HELLO-5D` - docs(audit): decide audit event shape
- `M-HELLO-5E` - tests(policy): add pending verifier / capability / runtime policy fixtures
- `M-HELLO-6A` - verify: add pending verifier admission model behind isolated path
- `M-HELLO-6B` - runtime: add observation sink interface skeleton, no host output
- `M-HELLO-6C` - capability: add observation capability model skeleton

## 12. Acceptance Checklist

- runtime subject documented
- sink model decision recorded
- sink event shape table added
- routing rule documented
- determinism / ordering documented
- host channel boundary preserved
- failure behavior planning added
- rejected runtime shortcuts listed
- no code changes
- no runtime / sink / capability / audit / verifier implementation
- no SemCode / opcode changes
- no accepted runtime behavior
- `#477` remains open

## 13. M-HELLO-6B Implementation Boundary

- isolated runtime observation sink interface skeleton exists
- no host output
- no stdout default
- no print
- no VM / runtime route integration
- no capability / audit behavior
- no CLI pipeline integration
- `#477` remains open

## 14. M-HELLO-8A Implementation Boundary

- isolated runtime observation route skeleton exists
- routes admitted Hello observation to explicit sink only
- no VM / runtime production routing
- no host output
- no stdout default
- no print / general I/O
- no capability admission
- no audit storage
- no CLI / smc integration
- `#477` remains open
