# UI Renderer Admission Boundary

Status: Draft
Track: POST-UI
Purpose: define the renderer admission boundary before implementation

## 1. Goal

This document defines the boundary for admitting a renderer into the Semantic UI stack.

The renderer must be introduced as a separate layer. It must not be smuggled into:

* `NativeBackend`;
* `NativeBackendWinitApp`;
* `NativeBackendWinitAppState`;
* `NativeBackend::run_event_loop(...)`;
* `prom-ui-runtime`;
* `UiBackendAdapter`.

The current draw path is staging/accounting only:

```text
DrawFrame
  -> NativeBackendWinitApp::stage_draw_frame(...)
  -> NativeBackend submitted_frames accounting
  -> NativeBackendWinitAppDrawTranscript
```

This is not rendering and not presentation.

## 2. Current admitted layers

| Layer | Status | Renderer role |
| --- | --- | --- |
| `prom-ui-runtime` | platform-neutral | none |
| `UiBackendAdapter` | staged adapter seam | none |
| `NativeBackend` | staged native backend state/accounting | none |
| `NativeBackendWinitApp` | native facade | none |
| `NativeBackendWinitAppState` | winit `ApplicationHandler` state | none |
| transcript objects | observation/accounting contracts | none |
| renderer | not admitted yet | out of scope |

## 3. Renderer admission principle

Renderer admission requires a separate PR track.

The renderer may only be admitted after its ownership, lifecycle, and transcript boundary are explicit.

Renderer admission must answer:

1. Who owns the renderer?
2. Who owns the native surface?
3. Who owns frame presentation?
4. How is `DrawFrame` translated?
5. How are render errors reported?
6. How is rendering distinguished from staging?
7. How are renderer facts represented in transcript objects?
8. How does renderer lifecycle interact with window lifecycle?

Until those questions are answered, renderer implementation remains out of scope.

## 4. Renderer non-goals before admission

Before renderer admission, the project must not add:

* `wgpu`;
* `pixels`;
* `softbuffer`;
* platform drawing APIs;
* swapchain/surface ownership;
* GPU device/queue ownership;
* frame presentation;
* font rasterization;
* image loading;
* layout engine;
* animation engine;
* widget system.

## 5. Boundary with draw staging

Draw staging means:

```text
DrawFrame accepted into backend-side accounting.
```

Draw staging does not mean:

```text
DrawFrame rendered.
DrawFrame presented.
DrawFrame flushed to native surface.
DrawFrame sent to GPU.
DrawFrame drawn by CPU rasterizer.
```

The current draw transcript records only:

```text
submitted_before
submitted_after
submitted_delta
```

It does not record pixels, surfaces, GPU commands, or presentation.

## 6. Required renderer ownership model

The first renderer implementation must introduce a clear ownership type.

Candidate shape:

```text
NativeBackendWinitRenderer
  -> owns renderer resources
  -> consumes/interprets DrawFrame
  -> returns RendererTranscript
```

The exact type name may change, but the ownership must remain outside:

```text
prom-ui-runtime
UiBackendAdapter
NativeBackend::run_event_loop(...)
```

Renderer-specific ownership must stay inside:

```text
prom-ui-backend-native
```

or a future renderer crate explicitly admitted by a separate boundary PR.

## 7. Required renderer transcript

Renderer admission must add a transcript distinct from draw staging.

Candidate shape:

```text
NativeBackendWinitRendererTranscript
  -> draw_commands_seen
  -> draw_commands_supported
  -> draw_commands_rejected
  -> render_attempted
  -> render_succeeded
  -> frame_presented
  -> renderer_errors
```

Renderer transcript must not be collapsed into `NativeBackendWinitAppDrawTranscript`.

Draw transcript remains staging/accounting.
Renderer transcript represents actual rendering/presentation facts.

## 8. Required fail-closed behavior

Renderer admission must fail closed when:

* no native window/surface is available;
* renderer resources cannot be created;
* unsupported draw command is encountered;
* frame presentation fails;
* lifecycle state is invalid;
* renderer capability is not admitted, if renderer capabilities are added later.

A renderer failure must not silently become a successful draw transcript.

## 9. Required tests before renderer implementation is considered admitted

The first renderer implementation PR must include tests for:

* renderer availability behind feature gate;
* renderer starts uninitialized;
* renderer refuses missing surface/window state;
* renderer distinguishes staging from rendering;
* unsupported draw command handling;
* render transcript shape;
* no changes to `prom-ui-runtime`;
* no changes to `UiBackendAdapter`;
* no changes to `NativeBackend::run_event_loop(...)`.

Manual/native tests may be ignored when a real desktop session is required.

## 10. Forbidden shortcuts

Renderer implementation must not:

* add `wgpu` directly to `prom-ui-runtime`;
* make `UiBackendAdapter::draw_frame(...)` perform real native rendering without an admitted renderer boundary;
* make `NativeBackend::run_event_loop(...)` own renderer lifecycle;
* store renderer handles in `DesktopSession`;
* treat `submitted_frames` as presented frames;
* treat draw transcript as render transcript;
* bypass lifecycle or capability boundaries.

## 11. Current decision

Renderer is not admitted yet.

Current admitted native UI path remains:

```text
NativeBackendWinitApp
  -> NativeBackendWinitAppState
  -> EventLoop::run_app(...)
  -> summary/transcript
```

Current admitted draw path remains:

```text
DrawFrame
  -> stage_draw_frame(...)
  -> submitted_frames accounting
  -> draw transcript
```

## Visual doctrine dependency

Renderer admission must follow the Semantic UI visual doctrine:

```text
docs/architecture/ui_visual_design_doctrine.md
```

Renderer implementation must not introduce visual meaning on its own.

The renderer executes admitted visual grammar.
It does not own Semantic UI meaning.

## Token dependency

Renderer admission depends on the visual token system boundary:

```text
docs/architecture/ui_visual_token_system_boundary.md
```

Renderer implementation must consume admitted visual tokens.

Renderer implementation must not invent token semantics.

## Layout dependency

Renderer admission depends on the layout primitive boundary:

```text
docs/architecture/ui_layout_primitive_boundary.md
```

Renderer implementation must consume admitted layout output.

Renderer implementation must not invent layout semantics.

## Component dependency

Renderer admission depends on the component admission boundary:

```text
docs/architecture/ui_component_admission_boundary.md
```

Renderer implementation must consume admitted component/layout output.

Renderer implementation must not invent component semantics.

## Interaction dependency

Renderer admission does not grant renderer ownership of interaction semantics.

Renderer may support input plumbing only after the interaction/input semantic boundary is admitted:

```text
docs/architecture/ui_interaction_input_semantic_boundary.md
```

Renderer must not treat native input or hit-test output as admitted semantic action.

## Focus and selection dependency

Renderer hit-testing or visual focus indication must not define semantic focus or selection.

Focus and selection semantics are defined separately in:

```text
docs/architecture/ui_focus_selection_semantic_boundary.md
```

Renderer must not treat hit-test output as semantic selection or admitted action target.

## Semantic action dependency

Renderer admission does not grant renderer ownership of Semantic UI actions.

Semantic UI actions are defined separately in:

```text
docs/architecture/ui_semantic_action_boundary.md
```

Renderer may display admitted action affordances, but must not decide action meaning or admission.

## Effect and capability dependency

Renderer admission does not grant renderer authority to perform effects or grant capabilities.

Effect requests and UI capabilities are defined separately in:

```text
docs/architecture/ui_effect_request_capability_boundary.md
```

Renderer may display admitted effect/capability state, but must not decide or perform effects.

## Trace and audit visual dependency

Renderer admission does not grant renderer authority over trace/audit meaning.

Trace/audit visual boundaries are defined separately in:

```text
docs/architecture/ui_trace_audit_visual_boundary.md
```

Renderer may render trace projections, but must not define trace truth or audit authority.

Renderer admission starts only after this boundary is accepted.
