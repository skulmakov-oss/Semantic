# Semantic Hello Observation Capability Gate

Status: capability-gate scope note for `#477`

See also:

- [`semantic_hello_vm_observation_execution_route.md`](semantic_hello_vm_observation_execution_route.md)
- [`semantic_hello_observation_admission_runtime_path.md`](semantic_hello_observation_admission_runtime_path.md)
- [`semantic_hello_observation_admission_shape.md`](semantic_hello_observation_admission_shape.md)
- [`semantic_hello_observation_audit_policy.md`](semantic_hello_observation_audit_policy.md)
- [`semantic_hello_controlled_observation_encoding.md`](semantic_hello_controlled_observation_encoding.md)
- [`semantic_hello_cli_smoke_path.md`](semantic_hello_cli_smoke_path.md)
- [`semantic_hello_cli_smoke_path_pre_audit.md`](semantic_hello_cli_smoke_path_pre_audit.md)

## 1. Purpose

This document defines the capability gate contract for the explicit controlled
observation sink route.

It does not define verifier admission, VM dispatch, audit storage, CLI output,
final Hello examples, or general stdout behavior.

## 2. Capability Gate Model

```text
admitted ControlledTextObservation
  -> sm-vm internal observation event / sink packet
  -> capability gate
  -> later audit decision
  -> later CLI-visible sink
```

The capability gate must not itself write to stdout.

The capability gate decides whether the controlled observation route may
proceed to later layers.

## 3. Required Capability Shape

```text
ControlledObservationCapability
  kind: ControlledTextObservationSink
  class: ControlledText
  sink: ExplicitObservationSink
  policy_ref: v0 policy descriptor or 0
```

This is a capability contract shape, not necessarily final Rust API, final
binary encoding, or final host ABI.

## 4. Capability Invariants

The capability-facing contract must require:

- the observation must already be verifier-admitted
- the VM route must already classify it as `ControlledText`
- the capability must authorize only the explicit controlled observation sink
- the capability must not authorize general stdout
- the capability must not authorize file / stdin / network
- the capability must not authorize formatting / interpolation
- the capability must not authorize implicit scalar-to-text conversion
- the capability must preserve deterministic observation ordering
- the capability decision must be explicit, not default-allow

## 5. Denial Matrix

| Case | Expected capability result |
|---|---|
| no controlled observation capability | deny |
| capability exists but wrong class | deny |
| capability requests general stdout | deny |
| capability requests file / stdin / network | deny |
| capability requests formatting / interpolation | deny |
| capability attempts scalar-to-text conversion | deny |
| capability bypasses VM event route | deny |
| capability allows explicit controlled text sink | allow for later audit / CLI stages only |

allow does not mean write output in this PR.
allow means the event may proceed to later audit / CLI stages.

## 6. Owner Boundary

| Concern | Owner | M-HELLO-11D action |
|---|---|---|
| admission shape | `sm-verify` | already scoped by 11B |
| VM event route | `sm-vm` | already scoped by 11C |
| runtime vocabulary / sink config | `sm-runtime-core` | vocabulary only, no dispatch ownership |
| capability gate | `prom-cap` | define narrow gate contract |
| audit policy | `prom-audit` | out of scope until 11E |
| CLI-visible smoke path | `smc-cli` | out of scope until 12A |

## 7. Not Implemented

- no production capability implementation unless trivially local and tested
- no VM execution changes
- no verifier changes
- no audit storage
- no CLI output
- no stdout writing
- no `run-smc` behavior
- no `smc run` behavior
- no general I/O
- no formatting / interpolation
- no README / example promotion
- no closure of `#477`

## 8. Next PR Split

- `11E` - audit decision / storage policy for controlled observation
- `12A` - CLI smoke path only after verifier / VM route / capability / audit are accepted

## 9. Issue State

- this document does not close `#477`
- this document does not satisfy `#477` acceptance criteria
- `#661` remains historical planning context only

## 10. Acceptance Checklist

- [ ] capability gate contract documented
- [ ] capability-facing invariants documented
- [ ] denial matrix documented
- [ ] owner boundary documented
- [ ] prom-cap owns only the capability decision
- [ ] prom-cap does not own VM execution, audit storage, or CLI output
- [ ] no CLI behavior is claimed
- [ ] no stdout / general I/O behavior is claimed
- [ ] `#477` remains open
