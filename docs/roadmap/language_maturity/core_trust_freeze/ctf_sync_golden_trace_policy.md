# CTF-SYNC-5 - Golden Trace Policy Candidates

## Status

Issue body / docs-only follow-up.

## Title

`ctf: record PCC canonical examples as golden trace candidates`

## Scope

Candidate traces:

- `match_control_flow`
- `option_result_control_flow`
- `loop_control_flow`
- `text_core`
- `collections_core`
- `stdlib_v0_helpers`

## Purpose

Record trace candidates without turning them into active golden-trace
requirements.

## Acceptance

- examples are marked as trace candidates, not active golden requirements
- print output trace policy remains unresolved / future
- negative diagnostics stay broad-marker based for now

## Non-Goals

- no golden trace artifact requirement
- no `print(text)` trace policy finalization
- no negative snapshot lock-in

