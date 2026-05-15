# Semantic Hello CLI Smoke Path

Status: planning document for `#477`

## 1. Purpose

This document records the future CLI smoke path boundary for `#477`.

- docs-only
- test/docs-only bridge harness description
- no production CLI implementation
- no `smc` integration
- no runtime output
- no accepted runtime behavior
- no verifier / capability / audit / SemCode implementation

## 2. Current Status

- isolated CLI smoke harness exists
- no production CLI / `smc` integration
- no runtime output
- no user-visible Hello World claim

## 3. Pipeline Modeled

```text
fixture source
→ parser
→ sema
→ lowering
→ real SemCode-level skeleton
→ isolated verifier admission
→ isolated capability gate
→ isolated runtime route
→ isolated audit decision
```

## 4. Not Implemented

- no `smc check`
- no `smc compile`
- no `smc verify`
- no `smc run`
- no `smc run-smc`
- no VM execution
- no bytecode
- no opcode IDs
- no production runtime route
- no production capability admission
- no AuditTrail storage
- no host output
- no README / examples alignment

## 5. Acceptance Gate For Later Real CLI PR

Later real CLI integration may start only after:

- the skeleton chain stays green
- verifier, runtime, capability, and audit boundaries are stable
- observation routing remains explicit sink only
- negative fixtures stay rejected
- `#477` remains open until real CLI behavior is accepted

## 6. Boundary Note

- `M-HELLO-9A` is not user-visible Hello World
- it is not accepted runtime behavior
- it is a controlled smoke-path harness only
- production CLI remains blocked

See also: [`semantic_hello_implementation_closeout.md`](semantic_hello_implementation_closeout.md)

