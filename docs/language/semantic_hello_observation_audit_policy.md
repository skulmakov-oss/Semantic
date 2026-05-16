# Semantic Hello Observation Audit Policy

Status: audit-policy scope note for `#477`

See also:

- [`semantic_hello_vm_observation_execution_route.md`](semantic_hello_vm_observation_execution_route.md)
- [`semantic_hello_observation_capability_gate.md`](semantic_hello_observation_capability_gate.md)
- [`semantic_hello_observation_admission_runtime_path.md`](semantic_hello_observation_admission_runtime_path.md)
- [`semantic_hello_observation_admission_shape.md`](semantic_hello_observation_admission_shape.md)
- [`semantic_hello_controlled_observation_encoding.md`](semantic_hello_controlled_observation_encoding.md)
- [`semantic_hello_cli_smoke_path.md`](semantic_hello_cli_smoke_path.md)
- [`semantic_hello_cli_smoke_path_pre_audit.md`](semantic_hello_cli_smoke_path_pre_audit.md)

## 1. Purpose

This document defines the audit decision and storage policy contract for
controlled text observation.

It does not define verifier admission, VM dispatch, capability authorization,
CLI output, final Hello examples, or general stdout behavior.

## 2. Audit Policy Model

```text
admitted ControlledTextObservation
  -> sm-vm internal observation event / sink packet
  -> capability gate allow
  -> audit decision
  -> audit record / explicit no-store policy
  -> later CLI-visible sink
```

The audit layer must not itself write to stdout.

The audit layer decides whether the controlled observation event must be
recorded, denied, redacted, or explicitly passed with a no-store policy.

## 3. Required Audit Record Shape

```text
ControlledObservationAuditRecord
  event_kind: ControlledTextObservation
  class: ControlledText
  sequence_index: deterministic observation order
  policy_ref: v0 policy descriptor or 0
  capability_decision: allow / deny
  audit_decision: record / deny / redact / no_store
  text_ref: admitted text constant reference
  text_digest: optional deterministic digest
```

This is an audit contract shape, not necessarily final Rust API, final storage
schema, or final binary encoding.

## 4. Audit Invariants

The audit-facing contract must require:

- the observation must already be verifier-admitted
- the VM route must already classify it as `ControlledText`
- the capability gate decision must already be explicit
- the audit decision must be explicit, not default-allow
- the audit layer must preserve deterministic sequence order
- the audit layer must not introduce nondeterministic timestamps unless
  explicitly modeled
- the audit layer must not write to stdout
- the audit layer must not authorize file / stdin / network
- the audit layer must not perform formatting / interpolation
- the audit layer must not bypass later CLI policy

## 5. Storage Policy Options

| Policy | Meaning |
|---|---|
| `record` | store an audit record for the controlled observation |
| `deny` | stop the observation route before CLI-visible sink |
| `redact` | store metadata / digest without full text payload |
| `no_store` | explicitly allow no persistent record under v0 policy |

`no_store` must be explicit. Missing audit policy is not equivalent to
`no_store`.

## 6. Denial Matrix

| Case | Expected audit result |
|---|---|
| no audit policy | deny or reject |
| capability decision missing | deny |
| capability denied | deny |
| invalid observation class | deny |
| sequence index missing | deny |
| nondeterministic audit metadata required | deny or out of scope |
| file / stdin / network target appears | deny |
| general stdout target appears | deny |
| record policy selected | audit record required |
| redact policy selected | digest / metadata record required |
| no_store policy selected | explicit no-store decision required |

audit allow does not mean write output in this PR.
audit allow only means the event may proceed to a later CLI smoke path.

## 7. Owner Boundary

| Concern | Owner | M-HELLO-11E action |
|---|---|---|
| admission shape | `sm-verify` | already scoped by 11B |
| VM event route | `sm-vm` | already scoped by 11C |
| capability gate | `prom-cap` | already scoped by 11D |
| audit decision / storage policy | `prom-audit` | define narrow audit policy contract |
| runtime vocabulary / sink config | `sm-runtime-core` | vocabulary only, no dispatch ownership |
| CLI-visible smoke path | `smc-cli` | out of scope until 12A |

## 8. Not Implemented

- no production audit implementation unless trivially local and tested
- no VM execution changes
- no verifier changes
- no capability redesign
- no CLI output
- no stdout writing
- no `run-smc` behavior
- no `smc run` behavior
- no general I/O
- no file / stdin / network
- no formatting / interpolation
- no README / example promotion
- no closure of `#477`

## 9. Next PR Split

- `12A` - CLI smoke path only after verifier / VM route / capability / audit are accepted

Optionally:

- `12B` - user-facing examples / README promotion only after 12A is real and tested

## 10. Issue State

- this document does not close `#477`
- this document does not satisfy `#477` acceptance criteria
- `#661` remains historical planning context only

## 11. Acceptance Checklist

- [ ] audit decision / storage policy contract documented
- [ ] audit-facing invariants documented
- [ ] storage policy options documented
- [ ] denial matrix documented
- [ ] owner boundary documented
- [ ] prom-audit owns only audit decision / storage policy
- [ ] prom-audit does not own VM execution, capability gate, or CLI output
- [ ] no CLI behavior is claimed
- [ ] no stdout / general I/O behavior is claimed
- [ ] `#477` remains open
