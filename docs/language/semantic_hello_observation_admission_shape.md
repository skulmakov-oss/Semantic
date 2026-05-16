# Semantic Hello Observation Admission Shape

Status: production-facing admission shape note for `#477`

See also:

- [`semantic_hello_observation_admission_runtime_path.md`](semantic_hello_observation_admission_runtime_path.md)
- [`semantic_hello_vm_observation_execution_route.md`](semantic_hello_vm_observation_execution_route.md)
- [`semantic_hello_controlled_observation_encoding.md`](semantic_hello_controlled_observation_encoding.md)

## 1. Purpose

This document defines the production-facing verifier admission shape for
controlled text observation.

It does not define VM execution, CLI output, capability wiring, audit behavior,
or final user-facing Hello behavior.

## 2. Admission Shape

```text
ControlledTextObservationAdmission
  kind: ControlledTextObservation
  text_ref: const text index
  observation_class: ControlledText
  sequence_index: deterministic observation order
  policy_ref: v0 policy descriptor or 0
```

This is a contract shape, not necessarily final binary encoding.

## 3. Admission Invariants

The verifier-facing admission contract must require:

- kind must be `ControlledTextObservation`
- text_ref must point to a valid text constant
- text payload must be text, not `i32` / `u32` / quad / `fx` / `bool`
- observation_class must be `ControlledText`
- sequence_index must be deterministic
- policy_ref must be present or explicitly v0-zero
- no file / stdin / network sink is allowed
- no formatting / interpolation is admitted
- no implicit scalar-to-text conversion is admitted

The scope split is:

- shape validity -> verifier-facing rule
- canonical Hello text -> current M-HELLO fixture scope

Do not permanently encode `Hello, World!` as the only possible production text
unless current architecture already requires that.

## 4. Rejection Matrix

| Case | Expected result |
|---|---|
| missing observation record | reject |
| observation before required state check | reject |
| observation after completion | reject |
| non-text constant | reject |
| text ref out of range | reject |
| stdout / print / io.write as raw sink | reject |
| file / stdin / network sink marker | reject |
| formatting / interpolation payload | reject |
| implicit scalar-to-text conversion | reject |
| multiple observations without sequence policy | reject or explicitly out of scope |

## 5. Owner Boundary

| Concern | Owner | M-HELLO-11B action |
|---|---|---|
| provisional bridge | `sm-emit` | already exists; no production encoder integration |
| admission shape | `sm-verify` | define verifier-facing contract |
| execution behavior | `sm-vm` | out of scope until 11C |
| runtime vocabulary / sink config | `sm-runtime-core` | vocabulary / config only, no routing ownership |
| capability gate | `prom-cap` | out of scope until 11D |
| audit policy | `prom-audit` | out of scope until 11E |
| CLI-visible smoke path | `smc-cli` | out of scope until 12A |

## 6. Not Implemented

- no production SemCode encoder integration
- no final numeric opcode ID assignment
- no shared bytecode format change
- no VM execution
- no runtime output
- no `smc run` behavior
- no `run-smc` behavior
- no capability gate wiring
- no audit storage
- no README / example promotion
- no closure of `#477`

## 7. Next PR Split

- `11C` - VM execution route scope for admitted controlled observation
- `11D` - capability gate wiring for the explicit sink route
- `11E` - audit decision / storage policy for controlled observation
- `12A` - CLI smoke path only after verifier / runtime / capability / audit are accepted

## 8. Issue State

- this document does not close `#477`
- this document does not satisfy `#477` acceptance criteria
- `#661` remains historical planning context only

## 9. Acceptance Checklist

- [ ] production-facing admission shape documented
- [ ] verifier-facing invariants documented
- [ ] rejection matrix documented
- [ ] owner boundary documented
- [ ] no code / test / fixture changes
- [ ] no runtime / CLI behavior claimed
- [ ] no accepted Hello World behavior
- [ ] `#477` remains open
