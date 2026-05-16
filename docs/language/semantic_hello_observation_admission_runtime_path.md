# Semantic Hello Observation Admission and Runtime Path

Status: docs-only scope note for `#661`

See also:

- [`semantic_hello_controlled_observation_encoding.md`](semantic_hello_controlled_observation_encoding.md)
- [`semantic_hello_observation_admission_shape.md`](semantic_hello_observation_admission_shape.md)
- [`semantic_hello_vm_observation_execution_route.md`](semantic_hello_vm_observation_execution_route.md)
- [`semantic_hello_observation_capability_gate.md`](semantic_hello_observation_capability_gate.md)
- [`semantic_hello_observation_audit_policy.md`](semantic_hello_observation_audit_policy.md)
- [`semantic_hello_implementation_closeout.md`](semantic_hello_implementation_closeout.md)
- [`semantic_hello_cli_smoke_path.md`](semantic_hello_cli_smoke_path.md)

## 1. Purpose

This document scopes the next controlled step after the provisional observation
byte bridge.

- docs-only
- no code changes
- no tests
- no fixtures
- no CLI integration
- no runtime output
- no accepted Hello World behavior
- `#477` remains open

## 2. Scope Model

```text
provisional Hello observation bytes
  ↓
real observation admission / runtime path scope
  ↓
owner map
  ↓
next PR split
```

This is a planning boundary only. It does not claim that any runtime path is
accepted or executable.

## 3. Current Boundary

Accepted today:

- controlled observation remains symbolically scoped
- provisional observation bytes remain gated and non-production
- canonical Hello validation remains narrow
- docs can describe the next admission/runtime split

Not accepted today:

- full `smc run` output
- full `run-smc` output
- general stdout
- print formatting
- implicit scalar-to-text conversion
- file / stdin / network I/O
- broad Host ABI widening
- capability redesign
- audit redesign
- README promotion
- Workbench / UI
- closing `#477`

## 4. Owner Map

| Area | Suggested owner | Notes |
|---|---|---|
| observation byte bridge | `sm-emit` | provisional bridge already exists; no production bytecode claim |
| real observation admission | `sm-verify` | future admission must stay narrow and verifier-first |
| execution of admitted observation | `sm-vm` | owns instruction dispatch / execution behavior for admitted SemCode |
| runtime vocabulary / sink config | `sm-runtime-core` | shared execution config, trap/quota vocabulary, not routing/orchestration owner |
| capability gate | `prom-cap` | route gating only, no broad host-call widening |
| audit decision / storage policy | `prom-audit` | record or defer by explicit policy only |
| CLI smoke path | `smc-cli` | later, after verifier/runtime/capability/audit are accepted |

This table is a planning split, not an implementation claim.

## 5. Next PR Split

- `11B` - production observation admission shape
- `11C` - VM execution route scope for admitted controlled observation
- `11D` - capability gate wiring for the explicit sink route
- `11E` - audit decision/storage policy for controlled observation
- `12A` - CLI smoke path only after the above are accepted

## 6. Not Implemented

- no production verifier admission
- no VM execution
- no runtime routing behavior
- no capability admission redesign
- no audit storage behavior
- no CLI / smc integration
- no user-visible output
- no README / examples claim

## 7. Issue State

- this document does not close `#477`
- this document does not satisfy `#477` acceptance criteria
- `#661` is a scope note only

## 8. Acceptance Checklist

- [ ] docs-only
- [ ] admission/runtime path scope documented
- [ ] owner map documented
- [ ] next PR split documented
- [ ] no code / test / fixture changes
- [ ] no CLI / runtime output
- [ ] no accepted Hello World behavior
- [ ] `#477` remains open
