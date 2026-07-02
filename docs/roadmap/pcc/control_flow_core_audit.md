# PCC Control Flow Core Audit

Status: working audit note for the practical core contour

## Purpose

This document records the current admitted control-flow surface in Semantic as
observed through canonical and probe samples.

It is intentionally conservative. It does not claim release stability. It does
not widen the language contract. It captures the current practical contour so
the next implementation issues can be scoped cleanly.

## Executive Verdict

The current control-flow surface is practically usable for small programs and
has enough evidence to support canonical examples.

Observed practical anchors:

- [examples/canonical/text_collections_toolbox/](C:\Users\said3\Desktop\EXOcode\EXOcode\examples\canonical\text_collections_toolbox\README.md)
- [examples/canonical/match_control_flow/](C:\Users\said3\Desktop\EXOcode\EXOcode\examples\canonical\match_control_flow\README.md)
- [examples/canonical/option_result_control_flow/](C:\Users\said3\Desktop\EXOcode\EXOcode\examples\canonical\option_result_control_flow\README.md)

Current verdict:

- `if / else` are practical and canonical-safe.
- `match` over `quad` is practical and canonical-safe.
- `match` over `Option(T)` and `Result(T, E)` is practical and canonical-safe
  for current admitted surface.
- `quad` does not imply truthiness.
- current `quad` `match` still requires an explicit `_` arm.
- `fn main()` remains the admitted entrypoint shape.

## Observed Surface

### Control Flow

- `if condition { ... } else { ... }`
- nested `else if`
- `match` expressions over `quad`
- `match` expressions over `Option(T)`
- `match` expressions over `Result(T, E)`
- terminal return paths inside branches
- ordinary nested `if` inside a `match` arm

### Stable Quirks

- `fn main()` must not declare a return type in the current admitted surface.
- current `quad` `match` requires an explicit `_` arm even when `T/F/N/S`
  appear explicitly.
- current `Option` / `Result` matches use explicit namespace constructors.

### Not Yet Canonicalized Here

- `while / loop / break / continue`
- `break value`
- pattern guards in new control-flow examples
- fallthrough semantics
- implicit `quad` truthiness

## Evidence Summary

Canonical examples that demonstrate the current practical contour:

- `text_collections_toolbox`
- `match_control_flow`
- `option_result_control_flow`
- `loop_control_flow`

Probe evidence:

- `examples/pcc_candidates/option_result_control_flow/`

Validation:

- `smc check` passed for the canonical samples and the probe sample
- canonical samples are included in the canonical examples test and smoke
  matrix

## Follow-Up Issues

Recommended next issue pack:

- PCC-CF-1: specify control-flow core contract
- PCC-CF-2: [stabilize match fallback arm policy](match_fallback_arm_policy.md)
- PCC-CF-3: [qualify terminal return paths](terminal_return_paths_policy.md)
- PCC-CF-4: qualify while/loop/break/continue surface
- PCC-CF-5: add negative diagnostics fixtures
- PCC-CF-6: [define 7hell control-flow group](7hell_control_flow_group.md)
- PCC-CF-6B-preflight: [audit 7hell runner structure](7hell_control_flow_runner_audit.md)
- PCC-CF-closeout: [close out the control-flow contour](control_flow_core_closeout.md)

## Non-Goals

- No language widening.
- No claim that loop control is finished.
- No claim that the control-flow contour is release-stable.
- No canonical promotion of `while/loop` until probe evidence exists.
