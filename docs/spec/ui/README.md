# UI Spec Subbundle

Status: Draft
Track: POST-UI
Scope: UI runtime effect path boundary and envelope specs only
Implementation: out of scope

Related:

- `host_runtime_effect_path_boundary.md`
- `ui_effect_envelope_v0.md`
- `ui_capability_taxonomy.md`
- `ui_event_envelope_model.md`
- `ui_frame_lifecycle_contract.md`
- `../../architecture/ui_host_runtime_effect_boundary.md`
- `../../architecture/ui_full_effect_trace_ladder.md`

This subbundle collects docs that define the boundary between the Semantic VM,
runtime admission, capability and budget checks, audit intent, UI runtime, and
platform adapters, plus the contracts for the envelopes, capability taxonomy,
deterministic event model, and frame lifecycle that move across that boundary.

It does not add executable UI support.
