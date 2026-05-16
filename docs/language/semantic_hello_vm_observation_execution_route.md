# Semantic Hello VM Observation Execution Route

Status: VM-side route scope note for `#477`

See also:

- [`semantic_hello_observation_admission_shape.md`](semantic_hello_observation_admission_shape.md)
- [`semantic_hello_observation_admission_runtime_path.md`](semantic_hello_observation_admission_runtime_path.md)
- [`semantic_hello_controlled_observation_encoding.md`](semantic_hello_controlled_observation_encoding.md)

## 1. Purpose

This document defines the VM-side execution route scope for an already-admitted
controlled text observation.

It does not define verifier admission, capability wiring, audit storage, CLI
output, final Hello examples, or general stdout behavior.

## 2. Execution Route Model

```text
VerifiedProgram
  -> admitted ControlledTextObservation
  -> sm-vm dispatch boundary
  -> internal observation event / sink packet
  -> later capability / audit / CLI layers
```

The VM must not write directly to stdout in this step.

The VM route should produce or model an internal controlled observation event,
not general host output.

## 3. VM Ownership

`sm-vm` owns:

- instruction dispatch
- execution behavior for admitted SemCode
- runtime value handling during execution
- controlled observation route decision after verifier admission

`sm-vm` does not own:

- source parsing
- SemCode emission
- verifier admission rules
- capability policy
- audit persistence
- CLI rendering
- general host I/O

## 4. Runtime-Core Boundary

`sm-runtime-core` may own shared vocabulary such as:

- observation sink mode
- trap kind / quota vocabulary
- execution config field names
- runtime result envelope vocabulary

`sm-runtime-core` must not own the execution route or dispatch behavior.

## 5. Controlled Observation Event Shape

```text
ControlledObservationEvent
  class: ControlledText
  text_ref: admitted text constant reference
  sequence_index: deterministic observation order
  policy_ref: v0 policy descriptor or 0
```

This is a VM route / event shape, not necessarily final Rust API or binary
encoding.

## 6. Execution Invariants

The VM-facing route must require:

- the observation must already be verifier-admitted
- the VM must not reinterpret non-text values as text
- the VM must preserve deterministic order
- the VM must not open file / stdin / network routes
- the VM must not implement formatting / interpolation
- the VM must not bypass future capability / audit layers
- the VM must not claim CLI-visible output in this step

## 7. Rejection / Trap Posture

| Case | Expected posture |
|---|---|
| verifier did not admit observation | VM must not execute it |
| invalid text reference reaches VM | trap or reject as malformed verified program boundary breach |
| non-text value reaches observation route | trap or reject as verifier / format invariant breach |
| file / stdin / network sink reaches VM route | reject / trap; not a controlled observation |
| multiple observations without deterministic sequence | reject / trap or explicitly out of scope |
| CLI sink requested directly | out of scope until 12A |

Do not implement these traps unless they already exist locally and can be used
without broad changes.

## 8. Owner Boundary

| Concern | Owner | M-HELLO-11C action |
|---|---|---|
| verifier admission shape | `sm-verify` | already scoped by 11B |
| VM execution route | `sm-vm` | define VM-side route / event model |
| runtime vocabulary / sink config | `sm-runtime-core` | vocabulary only, no dispatch ownership |
| capability gate | `prom-cap` | out of scope until 11D |
| audit policy | `prom-audit` | out of scope until 11E |
| CLI-visible smoke path | `smc-cli` | out of scope until 12A |

## 9. Not Implemented

- no production VM execution yet unless trivially local and tested
- no stdout writing
- no CLI output
- no `run-smc` behavior
- no `smc run` behavior
- no capability gate wiring
- no audit storage
- no general I/O
- no formatting / interpolation
- no README / example promotion
- no closure of `#477`

## 10. Next PR Split

- `11D` - capability gate wiring for the explicit sink route
- `11E` - audit decision / storage policy for controlled observation
- `12A` - CLI smoke path only after verifier / VM route / capability / audit are accepted

## 11. Issue State

- this document does not close `#477`
- this document does not satisfy `#477` acceptance criteria
- `#661` remains historical planning context only

## 12. Acceptance Checklist

- [ ] VM execution route scope documented
- [ ] VM-facing invariants documented
- [ ] rejection / trap posture documented
- [ ] owner boundary documented
- [ ] sm-runtime-core is not assigned dispatch / routing ownership
- [ ] no CLI behavior is claimed
- [ ] no stdout / general I/O behavior is claimed
- [ ] `#477` remains open
