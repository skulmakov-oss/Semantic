# UI-IMPL-1 Renderer Contract Test Tightening Plan

## Status

Result: READY-WITH-WARNINGS

This is a plan-only document.

No code was changed.
No tests were changed.
No examples/7hell files were changed.
No runtime/verifier/VM/SemCode files were changed.
No PCC/CTF files were changed.
No untracked residue was touched.

## Purpose

Plan the first minimal UI implementation slice after UI re-entry boundary verification.

## Source repo state

- branch: `main`
- HEAD: `fca5e55108fd47246679bc19e32a0579e2dce835`
- origin/main: `fca5e55108fd47246679bc19e32a0579e2dce835`
- HEAD == origin/main: yes
- AGENTS.md state: clean
- tracked dirty state: none
- untracked residue: local PCC/audit residue remains present and untouched

## DNA inspection

- files inspected: `docs/dna/SEMANTIC_UI_DNA.md`
- alignment: UI remains projection/cache, not semantic authority; renderer owns presentation contract, not verifier/runtime semantics; renderer contract tests must not create semantic authority; Unknown / Conflict states must remain visible and not flattened; no hidden host-effect path; no runtime/verifier/VM/SemCode authority transfer; native/backend ownership remains separate
- conflicts: none in the DNA itself
- constraints applied: no implementation code; no tests/examples/7hell; no PCC/CTF touch; no backend switch; no renderer rewrite; no reliance on untracked residue

## Re-entry evidence

| Evidence | Status | Notes |
|---|---:|---|
| native/WGPU wording aligned | verified | Boundary wording was reconciled to acknowledge the admitted feature-gated backend-native WGPU reality. |
| renderer boundary verified | verified | Renderer contract docs match `prom-ui` source reality and keep authority in the projection/presentation layer. |
| windowing boundary verified | verified | Windowing docs match the runtime/native host-bridge split and do not claim semantic authority. |
| AGENTS.md hygiene resolved | verified | AGENTS.md is clean and no longer blocks a new implementation branch. |

## Renderer source inspected

| File | Role | Notes |
|---|---|---|
| `crates/prom-ui/src/renderer.rs` | renderer contract implementation surface | Defines the inert renderer model, node, marker, diagnostics, trace, marker, and inspection presentation APIs. |
| `crates/prom-ui/src/projection.rs` | input projection surface | Supplies the projected artifact and projected node types that feed the renderer contract. |
| `crates/prom-ui/src/tree_bridge.rs` | projection bridge surface | Helps preserve source-to-render mapping without moving authority into the renderer. |

## Renderer tests inspected

| Test file | Current coverage | Gap / opportunity |
|---|---|---|
| `crates/prom-ui/tests/renderer_public_api_lock.rs` | Locks public renderer entrypoint, public types, and accessor surface; includes basic inertness smoke checks. | Best candidate for a narrow assertion-tightening change if a future mismatch is found. |
| `crates/prom-ui/tests/renderer_marker_presentation.rs` | Covers marker presentation as inert renderer-local metadata. | Not the primary slice, but useful context for the seam. |
| `crates/prom-ui/tests/renderer_trace_presentation.rs` | Covers trace presentation as inert renderer-local metadata. | Not the primary slice, but useful context for the seam. |
| `crates/prom-ui/tests/ui_render_model_stability.rs` | Confirms stable rendering model behavior. | Neighboring coverage, but not the first slice boundary. |

## Planning questions

1. What does `renderer_public_api_lock` protect now?
   - It protects the exported renderer entrypoint, public renderer types, and the model/node accessor surface so the abstract presentation contract cannot drift silently.
2. Which renderer public API elements should stay locked?
   - `render_projection_to_model`, `UiRenderModel`, `UiRenderNode`, `UiRenderNodeKind`, `UiRenderMarker`, `UiRenderError`, and the public model/node accessor methods already under test.
3. Is there a missing check that can be added without changing production code?
   - Possibly a slightly stricter signature assertion or an additional inertness assertion, but only if it stays inside `renderer_public_api_lock.rs`.
4. Can this slice be test-only?
   - Yes. That is the preferred boundary.
5. Which files may the future implementation slice change?
   - Only `crates/prom-ui/tests/renderer_public_api_lock.rs` for the recommended first option.
6. Which files must it not change?
   - `crates/prom-ui/src/renderer.rs`, runtime crates, backend-native crates, tests outside the renderer contract seam, and all PCC/CTF residue.
7. Which tests should run?
   - `cargo test -p prom-ui --test renderer_public_api_lock` first, then `cargo test -p prom-ui` only if the narrow test reveals broader surface drift.
8. What failure modes are expected?
   - Signature drift, accessor drift, accidental coupling to runtime/backend code, or a test that starts requiring production API changes.
9. What is the rollback plan?
   - Keep the change in `renderer_public_api_lock.rs` only; if it spills into production code, stop and re-scope before merging.
10. Can a branch be opened after this?
    - Yes, if the implementation stays test-only and the diff remains limited to the approved renderer contract seam.

## Proposed future implementation slice

- slice: `UI-IMPL-1 renderer contract test tightening`
- owning crate: `prom-ui`
- expected files: `crates/prom-ui/tests/renderer_public_api_lock.rs`
- forbidden files: `crates/prom-ui/src/renderer.rs`, runtime crates, backend-native crates, PCC/CTF docs, examples, 7hell, untracked residue
- expected validation: `cargo test -p prom-ui --test renderer_public_api_lock`
- acceptance criteria:
  - renderer public surface remains locked to the existing abstract presentation contract
  - no semantic authority transfer is introduced
  - no runtime/verifier/VM/SemCode ownership moves into the renderer seam
  - the diff stays minimal and local to `prom-ui`
- rollback criteria:
  - if the tightening forces broader API changes outside `renderer_public_api_lock.rs`
  - if the change starts pulling in backend/runtime code
  - if the slice cannot be validated with a narrow `prom-ui` test run

## Candidate options

| Option | Status | Expected files | Risk | Decision |
|---|---:|---|---|---|
| Option A test-only assertion tightening | READY | `crates/prom-ui/tests/renderer_public_api_lock.rs` | Low | Preferred first branch boundary |
| Option B test helper tightening | READY-WITH-WARNINGS | `crates/prom-ui/tests/renderer_public_api_lock.rs`, existing helper files only if they already exist | Low/medium | Acceptable only if the helper already exists and no new seam is introduced |
| Option C production API adjustment | NOT RECOMMENDED | `crates/prom-ui/src/renderer.rs` | Medium/high | Avoid unless the test reveals a real mismatch and owner approval is explicit |

## Required gates before implementation

1. Create a dedicated branch from clean `main == origin/main`.
2. Change only approved test file(s).
3. Do not touch production renderer code unless owner approves.
4. Do not touch runtime/verifier/VM/SemCode.
5. Do not touch PCC/CTF residue.
6. Run targeted validation:

```powershell
cargo test -p prom-ui --test renderer_public_api_lock
```

7. If broader renderer tests are affected, run:

```powershell
cargo test -p prom-ui
```

## Non-goals

- no renderer rewrite
- no backend switch
- no native/WGPU implementation change
- no runtime/verifier/VM changes
- no PCC/CTF changes
- no examples/7hell changes
- no cleanup of local residue

## Recommended next step

Choose exactly one:

- start implementation branch for Option A
- run test discovery first
- request owner decision
- block implementation

## Final verdict

Implementation may start, but only as a narrow `prom-ui` renderer-contract test-tightening slice.

The repository is aligned enough for a first implementation branch, while untracked local residue remains intentionally isolated and must stay out of the slice. The first safe step is a minimal contract-hardening PR in `prom-ui`, not a runtime or backend expansion.
