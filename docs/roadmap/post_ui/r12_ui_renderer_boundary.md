# R12 UI Renderer Boundary

## 1. Purpose

This document defines the boundary for a future R12 UI renderer.

Status note:
the repository already contains a feature-gated native WGPU path and dedicated reality notes at
[ui_reentry_3_native_wgpu_reality_alignment.md](./ui_reentry_3_native_wgpu_reality_alignment.md),
[r12_ui_native_wgpu_reality_reconciliation.md](./r12_ui_native_wgpu_reality_reconciliation.md), and
[r12_ui_native_wgpu_renderer_reality_audit.md](./r12_ui_native_wgpu_renderer_reality_audit.md).
This boundary remains the read-only renderer contract boundary, but it should not be read as claiming that native renderer support is entirely absent or only hypothetical.

Reconciliation note:
the renderer contract is still an abstract UI presentation contract;
the admitted native/WGPU presentation path lives in `prom-ui-backend-native` behind feature gates;
the renderer contract and the backend-native presentation path are separate ownership layers.

It does not implement renderer code.
It defines what a future renderer may consume from UiProjectionArtifact and what it must not infer or execute.

Renderer consumes projection artifacts; renderer does not create semantic authority.

## 2. Closed Basis

Closed basis:
#941 — R12 UI Projection Builder Final Closeout — MERGED
#942 — POST-UI Roadmap Next Lane Selection — MERGED

Projection substrate final state:
- UiIr can be projected to UiProjectionArtifact;
- ValidatedUiIr exists;
- config-aware validation exists;
- diagnostics and trace references exist;
- PropertyCarrier / ActionCarrier / EffectBoundaryMarker are inert classifications;
- public API lock exists;
- renderer backend implementation is absent; the abstract renderer contract is present.

## 3. Renderer Position in Pipeline

UiIr
  -> validate_ir
  -> ValidatedUiIr
  -> project_validated_ir_to_projection
  -> UiProjectionArtifact
  -> future renderer
  -> future render model / draw commands / presentation output

Renderer is downstream of UiProjectionArtifact.
Renderer must not read raw UiIr unless separately authorized.
Renderer must not call verifier/runtime/capability systems.
Renderer must not mutate Semantic state.

## 4. Allowed Renderer Inputs

Allowed future inputs:
- UiProjectionArtifact;
- UiProjectedNode;
- stable projection artifact ID;
- structural projected node ID;
- source/root trace references;
- diagnostics metadata exposed by projection layer;
- inert PropertyCarrier classification;
- inert ActionCarrier classification;
- inert EffectBoundaryMarker classification.

Not allowed as renderer input without separate gate:
- raw UiIr;
- AST;
- parser output;
- verifier internals;
- runtime state;
- VM state;
- capability handles;
- Workbench/Studio state;
- host effects;
- direct file/network/process handles.

## 5. Allowed Renderer Outputs

Allowed future renderer outputs, boundary only:
- inert render model;
- visual tree;
- draw-plan description;
- diagnostic presentation model;
- trace presentation model;
- accessibility/inspection metadata if inert;
- renderer-local cache if invalidatable and non-authoritative.

Forbidden outputs:
- semantic state mutation;
- verifier decision;
- runtime command;
- capability grant;
- event dispatch;
- effect execution;
- Workbench/Studio mutation;
- source rewrite;
- parser/verifier/VM command.

## 6. Renderer Non-Authority Rules

Renderer may display truth. Renderer does not become truth.
Renderer may display conflict. Renderer does not resolve conflict.
Renderer may display unknown. Renderer does not collapse unknown.
Renderer may display actions. Renderer does not execute actions.
Renderer may display effect boundaries. Renderer does not authorize effects.

Renderer must not reinterpret PropertyCarrier as runtime state.
Renderer must not reinterpret ActionCarrier as executable event handler.
Renderer must not reinterpret EffectBoundaryMarker as capability admission.

## 7. Projection Artifact Consumption Rules

Renderer must consume projection artifacts as read-only inputs.
Renderer may derive renderer-local presentation data.
Renderer-local cache must be invalidatable and non-authoritative.
Renderer must preserve projection IDs and trace references when producing inspectable render output.
Renderer must not synthesize missing semantic truth.

Renderer-local state is cache/presentation state, not semantic state.

## 8. Property / Action / EffectBoundary Handling

| Projection classification | Renderer may do | Renderer must not do |
|---|---|---|
| PropertyCarrier | Display as property-like presentation hint | Treat as runtime state or semantic truth |
| ActionCarrier | Display as action-like affordance marker | Dispatch event, execute command, or call runtime |
| EffectBoundaryMarker | Display effect boundary warning/marker | Grant capability, authorize effect, or call host |

## 9. Diagnostics and Trace Handling

Renderer may display diagnostics.
Renderer may display trace references.
Renderer may link visual nodes to source/projection trace references.
Renderer must not rewrite diagnostics into verifier results.
Renderer must not treat trace references as proof of Semantic truth.

## 10. Quad-State Handling

Renderer must preserve Quad-state meaning when available.

Unknown must remain visually distinguishable from false.
Conflict must remain visually distinguishable from ordinary failure.
Denied must remain distinguishable from false.
Not admitted must remain distinguishable from invalid source.

Renderer must not flatten N/F/T/S into boolean true/false.

Current renderer boundary records this as future requirement.
Current renderer implementation is absent.

## 11. Explicit Non-Scope

No renderer implementation.
No renderer module.
No backend implementation.
No WGPU/winit/Tauri.
No layout engine.
No draw engine.
No event loop.
No event dispatch.
No runtime integration.
No verifier integration.
No VM integration.
No capability admission.
No Workbench/Studio integration.
No source changes.
No dependency additions.

## 12. Forbidden Systems

Forbidden in this boundary:
- renderer/backend source code;
- layout/draw/event code;
- runtime bridge;
- verifier bridge;
- VM bridge;
- capability admission;
- action execution;
- effect execution;
- Workbench/Studio integration;
- dependencies;
- Cargo manifest changes.

## 13. Evidence Matrix

| Claim | Classification | Evidence | Status |
|---|---|---|---|
| R12 projection substrate closed | DOCUMENTED | #941 | PASS |
| renderer boundary selected | DOCUMENTED | #942 | PASS |
| renderer implementation exists | ABSENT / FORBIDDEN | Code audit | PASS |
| renderer may consume UiProjectionArtifact in future | AUTHORIZED_FOR_FUTURE | This doc | PASS |
| renderer may execute actions | FORBIDDEN | This doc | PASS |
| renderer may authorize effects | FORBIDDEN | This doc | PASS |
| renderer may become runtime bridge | FORBIDDEN | This doc | PASS |
| renderer may become verifier admission | FORBIDDEN | This doc | PASS |
| renderer may mutate Semantic state | FORBIDDEN | This doc | PASS |
| renderer-local cache may exist in future | AUTHORIZED_FOR_FUTURE_WITH_BOUNDARY | This doc | PASS |

## 14. Admission Guard Table

| Area | Boundary state | Admission Guard classification | Status |
|---|---|---|---|
| renderer boundary document | Present | ADMITTED | PASS |
| future renderer seed | Planned | AUTHORIZED_FOR_FUTURE | PASS |
| UiProjectionArtifact consumption | Planned | AUTHORIZED_FOR_FUTURE | PASS |
| renderer implementation in this PR | Absent | FORBIDDEN | PASS |
| layout/draw/event | Absent | FORBIDDEN | PASS |
| runtime/verifier/VM | Absent | FORBIDDEN | PASS |
| capability admission | Absent | FORBIDDEN | PASS |
| Workbench/Studio | Absent | FORBIDDEN | PASS |
| source changes | Absent | FORBIDDEN | PASS |
| dependency additions | Absent | FORBIDDEN | PASS |

## 15. Future Renderer Seed Gate

R12-UI-RENDERER-SEED-LINE-FULL-PACKAGE

- define inert renderer-local model if needed;
- consume UiProjectionArtifact read-only;
- preserve projection IDs;
- preserve trace references;
- present diagnostics without rewriting authority;
- classify property/action/effect boundary markers visually;
- add tests proving no runtime/capability/event execution.

Forbidden in future seed unless separately authorized:
- WGPU/winit/Tauri backend;
- layout engine;
- draw engine;
- event dispatch;
- runtime bridge;
- verifier bridge;
- capability admission;
- Workbench/Studio integration.

## 16. Final Decision

Final decision:
BOUNDARY DEFINED — A future R12 UI renderer may be introduced only as a downstream read-only consumer of UiProjectionArtifact.

Renderer may derive inert presentation data, diagnostics presentation, trace presentation, and visual affordance markers.

Renderer must not create semantic truth, execute actions, authorize effects, call runtime/verifier/VM, admit capabilities, or integrate Workbench/Studio.

Renderer implementation remains absent until a separate explicit seed gate.
