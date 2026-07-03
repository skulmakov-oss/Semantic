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

## CTF-WP4 PCC-4..PCC-9 Golden Trace Sync

PCC-4..PCC-9 added fixture-backed evidence, but fixtures are not automatically golden traces.

CTF-WP4 does not add trace artifacts.

CTF-WP4 defines which surfaces should be considered for future golden trace mapping.

Golden trace work remains follow-up unless already present.

| PCC | Surface | Existing evidence | Golden trace implication | WP4 status |
| --- | --- | --- | --- | --- |
| PCC-4 | Records | positive + negative record fixtures | candidate for syntax / type / lowering / SemCode / verifier trace mapping | follow-up candidate |
| PCC-5 | ADT + basic match | positive + negative ADT / match fixtures | candidate for type / lowering / verifier / VM trace mapping | follow-up candidate |
| PCC-6 | Option / Result | positive + negative standard-form fixtures | candidate for ADT-like trace mapping | follow-up candidate |
| PCC-7 | Collections v0 | Sequence / Map positive + diagnostics / trap fixtures | candidate for VM trap / determinism trace mapping | follow-up required for collection trap / replay traces |
| PCC-8 | Stdlib helpers | helper positive + diagnostics / trap fixtures | candidate for assert trap, helper diagnostics, to_text boundary traces | follow-up candidate |
| PCC-9 | Project Model baseline | Semantic.package positive + diagnostics fixtures | candidate for CLI / project-adjacent diagnostic trace mapping | follow-up candidate |

Trace boundary notes:

- Golden trace freeze is not required for every PCC fixture immediately.
- Golden traces should be added only when a behavior becomes release-facing or freeze-candidate enough to protect byte, result, or diagnostic stability.
- Compile-time diagnostics traces and VM runtime traces must not be mixed.
- Project-model manifest helper traces are not project-root execution traces.
- 7hell report traces remain future work.
- Project-level 7hell remains open.
- Map missing-key / iteration / quota policy remain open, so golden traces for those edges must not be claimed.

## CTF-E1 Selected Golden Trace Coverage

CTF-E1 promotes only selected PCC fixture-backed surfaces into golden trace evidence.

The first evidence set is representative, not exhaustive:

- PCC-4 Records;
- PCC-5 ADT + basic match;
- PCC-6 Option;
- PCC-7 Sequence;
- PCC-8 stdlib helper boundary.

CTF-E1 adds checked-in golden trace artifacts for selected surfaces only.

Boundaries:

- not all PCC fixtures are golden traces;
- no release-facing freeze;
- no CTF closure;
- no Map missing-key / iteration / quota trace;
- no 7hell report trace;
- no project-root execution trace;
- no semantic.toml trace;
- no package registry / remote dependency trace.

Future CTF-E2 / CTF-E3 still own replay and trap taxonomy evidence.

## CTF-WP6 Project-Root Golden Trace Notes

CTF-WP6 defines the trust policy for future project-root behavior, but it does not add trace artifacts.

Future PCC-9I work may add positive project-root check / run traces, project diagnostics traces, and project-root replay traces when behavior is implemented and stable.

Project manifest helper traces are still not project-root execution traces.

## PCC / CTF Sync Follow-Ups

The closed PCC practical phase identifies canonical anchors that are plausible
future trace candidates:

- `match_control_flow`;
- `option_result_control_flow`;
- `loop_control_flow`;
- `text_core`;
- `collections_core`;
- `stdlib_v0_helpers`.

These are not automatically golden traces.

Current follow-up posture:

```text
SYNC-PASS-WITH-FOLLOWUPS
```

Policy reminders:

- canonical anchors can remain docs/test evidence without becoming golden
  traces immediately;
- negative harness output can stay marker-based without full snapshot locking;
- `print(text)` output should only enter traces if trace policy explicitly
  requires it.
