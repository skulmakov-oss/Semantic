# CTF-6 — Golden Trace Policy

Status: draft policy
Parent lane: `core_trust_freeze/index.md`

## Purpose

Golden traces are used to prove that source, lowering, SemCode, verifier, and VM behavior remain stable across changes.

This file defines when a PCC feature needs new or updated golden traces.

## Trace classes

| Trace class | Purpose | PCC owner |
|---|---|---|
| Syntax trace | Parser/diagnostic stability | PCC-0.6 / PCC-1+ |
| Type trace | Typecheck/diagnostic stability | PCC-0.6 / PCC-2+ |
| IR trace | Lowering stability | PCC-1+ |
| SemCode trace | Emission stability | PCC-1+ |
| Verifier trace | Admission accept/reject stability | CTF / PCC feature phase |
| VM trace | Runtime result/trap stability | CTF / PCC feature phase |
| 7hell report | End-to-end qualification surface | PCC-0.6+ |

## When a PR must add or update golden traces

A PR must update golden traces when it changes:

- accepted source syntax;
- diagnostic behavior;
- type inference or type compatibility;
- lowering shape;
- optimization output;
- SemCode bytes;
- verifier result;
- VM result or trap;
- runtime value representation;
- capability/effect denial behavior;
- 7hell output contract.

## When a PR should not update golden traces

A PR should not update golden traces when it is:

- documentation-only;
- internal refactor with byte-for-byte stable output;
- test-only cleanup;
- non-semantic formatting.

If traces change unexpectedly, the PR must explain the reason.

## Minimum trace metadata

Future golden trace files should carry:

```text
source name
feature phase
input hash
stage
expected status
expected diagnostic code, if any
expected output hash, if any
runtime config, if execution is involved
```

## Review checklist

```text
[ ] Did any accepted behavior change?
[ ] Did any rejected behavior change?
[ ] Did diagnostics change?
[ ] Did emitted bytes change?
[ ] Did runtime result/trap change?
[ ] Did 7hell report shape change?
[ ] Are trace changes intentional and explained?
```
