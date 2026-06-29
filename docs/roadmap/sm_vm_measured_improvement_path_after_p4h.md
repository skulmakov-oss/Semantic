# sm-vm VM-M1 Measured Improvement Path After P4-H

## Status

P4-H blocked direct Pulsar P5-A review for the current workload corpus.

The next measured improvement frontier is `sm-vm` control flow and scalar movement.

This document does not change runtime behavior and does not approve Pulsar runtime promotion.

## P4-H Evidence Summary

Source evidence:

- [Pulsar P4-H sm-vm Opcode Profile Evidence](pulsar_p4h_sm_vm_profile_evidence.md)

Aggregate profile evidence:

| Metric | Count | Ratio |
|---|---:|---:|
| QAnd + QOr | `785 / 55611` | `1.41%` |
| quad logic | `849 / 55611` | `1.53%` |
| quad family | `8551 / 55611` | `15.37%` |
| control flow | `14287 / 55611` | `25.69%` |

Decision:

`P5-A candidate review: BLOCKED`

Reason:

- `QAnd + QOr` did not reach the 15% trigger threshold.
- The dominant costs appear in control flow and scalar movement.
- Batchability is not proven.
- Direct Pulsar runtime promotion is not justified by the current evidence.

Correct reading:

Pulsar remains a verified substrate, but the current `sm-vm` workload evidence does not justify promoting it into runtime hot paths yet.

## Measured VM Improvement Frontier

The next measured frontier is:

- control flow
- scalar movement

Relevant opcode families:

- `LoadVar`
- `StoreVar`
- `Jmp`
- `JmpIf`
- `Call`
- `Ret`
- `LoadI32`
- `CmpEq`
- `CmpNe`
- `BoolAnd`

Some workloads showed meaningful `quad_family` activity, but the pressure was mostly around loading, comparison, branching, and scalar VM movement rather than pure packed quad logic.

High quad-family activity does not equal Pulsar batchability.

## VM-M1 Goals

VM-M1 should answer:

1. Which non-Pulsar VM costs dominate the measured workloads?
2. Are `LoadVar` / `StoreVar` costs caused by lowering shape, register movement, local variable layout, or unavoidable semantics?
3. Are `Jmp` / `JmpIf` costs caused by source-level control flow, match lowering, loop lowering, or VM dispatch shape?
4. Are repeated calls/helper functions inflating `Call` / `Ret` / `LoadVar` / `StoreVar`?
5. Can source lowering reduce instruction count without changing Semantic behavior?
6. Can VM-level representation improve scalar movement without changing SemCode format?
7. Which next technical slice can reduce measured overhead with the lowest trust risk?

## Candidate Investigation Lanes

### Lane A - Opcode-family profile summaries

Goal:
Improve evidence extraction.

Possible slice:

`test(sm-vm): add opcode-family profile summaries`

Allowed:

- test-only helpers;
- family count summaries;
- no runtime behavior change;
- no public performance claim.

Purpose:
Make future evidence reports less manual.

### Lane B - Lowering shape audit

Goal:
Inspect whether source constructs generate excessive instruction counts.

Focus:

- `for` lowering;
- `match quad` lowering;
- nested `if` lowering;
- helper function calls;
- local variable movement.

Possible slice:

`docs(sm-vm): audit lowering shape for profiling workloads`

Allowed:

- docs-only audit;
- maybe disasm evidence;
- no lowering changes.

### Lane C - VM scalar movement audit

Goal:
Inspect `LoadVar` / `StoreVar` pressure.

Questions:

- Are locals stored and loaded more often than needed?
- Are temporary values spilled into locals unnecessarily?
- Are `Value` clones contributing to the cost?
- Are frame/local layouts causing avoidable movement?

Possible slice:

`docs(sm-vm): audit scalar movement in profiled workloads`

No code changes yet.

### Lane D - Control-flow lowering audit

Goal:
Inspect `Jmp` / `JmpIf` pressure.

Questions:

- Is `match quad` lowering branch-heavy?
- Are nested `if` chains generating expected or excessive jumps?
- Does `for` lowering add predictable overhead?
- Can branch shape be improved without semantic changes?

Possible slice:

`docs(sm-vm): audit control-flow lowering in profiled workloads`

### Lane E - Measured micro-improvement candidate

Only after A-D evidence:

A narrow code candidate may be proposed.

Examples:

- reduce redundant load/store in a specific lowering pattern;
- simplify a specific branch pattern;
- improve local temporary handling;
- improve a specific VM instruction path.

Must be measured and reversible.

## Forbidden Paths

This next phase must not:

- reopen Pulsar P5-A without fresh evidence;
- integrate Pulsar into `sm-vm`;
- replace VM execution wholesale;
- change SemCode semantics;
- change verifier admission;
- add production telemetry;
- add hidden runtime counters;
- make public performance claims;
- optimize based only on intuition;
- introduce broad VM refactors;
- change parser/typechecker/lowering just to make one workload look better.

## Evidence Required For VM Improvement PRs

Any future VM improvement PR must include:

1. Baseline profile evidence.
2. Exact workload(s) used.
3. Target opcode family.
4. Explanation of why the target is hot.
5. Boundary statement: no semantic change.
6. Tests proving behavior unchanged.
7. Before/after profile or instruction-count comparison, if code changes are made.
8. Rollback/fallback posture.
9. Statement that P5/Pulsar promotion is not implied.

For code-changing PRs:

`correctness first, measurement second, performance claim last`

## Recommended Next Slice

Recommended next PR:

`test(sm-vm): add opcode-family profile summaries`

Purpose:
Move family metrics from manual documentation into reusable test-local helper output.

Useful local profiling families:

- `quad_logic = QAnd + QOr + QNot + QImpl`
- `quad_family = LoadQ + QAnd + QOr + QNot + QImpl + CmpEq + CmpNe`
- `control_flow = Jmp + JmpIf`
- `scalar_movement = LoadVar + StoreVar`
- `calls = Call + Ret`
- `integer_ops = LoadI32 + AddI32 + SubI32 + MulI32 + DivI32 + ModI32 + CmpI32Lt + CmpI32Le`

These definitions are for local profiling reports, not public runtime API.

## Non-claims

This document does not claim:

- VM performance improved;
- Pulsar is obsolete;
- Pulsar runtime promotion is permanently blocked;
- P5 is cancelled;
- P5-A can never reopen;
- control-flow optimization is approved;
- scalar-movement optimization is approved;
- any code path should be changed without a follow-up measured PR.

Correct statement:

Pulsar P5-A is blocked by current evidence, but may reopen if future workloads or measurements show a strong, batchable quad-state hot path.
