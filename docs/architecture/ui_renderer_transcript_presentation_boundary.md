# Semantic UI Renderer Transcript and Presentation Status Boundary

Status: Draft
Track: POST-UI / H-series
Purpose: define renderer transcript and presentation status boundaries before renderer implementation

## 1. Goal

This document defines the boundary between draw staging, render attempts, render success, frame presentation, and semantic success in Semantic UI.

The project must distinguish:

```text
draw staging != render attempted
render attempted != render succeeded
render succeeded != frame presented
frame presented != semantic success
renderer transcript != audit authority
```

Renderer transcript facts must be explicit, inspectable, and separate from semantic action/effect success.

Renderer output must not become semantic authority.

## 2. Relationship to renderer admission boundary

Renderer transcript and presentation status depend on the renderer admission boundary:

```text
docs/architecture/ui_renderer_admission_boundary.md
```

Renderer admission must happen before renderer implementation.

H12 does not admit a renderer.

H12 defines the transcript/status vocabulary required before renderer implementation.

## 3. Layer separation

| Layer                  | Meaning                                           | Owner                            |
| ---------------------- | ------------------------------------------------- | -------------------------------- |
| draw staging           | `DrawFrame` accepted into backend-side accounting | native facade / staging bridge   |
| render attempted       | renderer tried to process staged frame            | future renderer layer            |
| render succeeded       | renderer produced render output                   | future renderer layer            |
| presentation attempted | renderer/backend tried to present frame           | future renderer/native bridge    |
| frame presented        | frame became visible through presentation path    | future renderer/native bridge    |
| renderer transcript    | structured renderer/presentation facts            | future renderer transcript layer |
| semantic success       | UI action/effect succeeded semantically           | action/effect boundary           |
| audit authority        | authoritative audit record                        | audit/runtime boundary           |

This preserves:

```text
Submitted is not rendered.
Rendered is not presented.
Presented is not semantic success.
Renderer transcript is not audit authority.
```

## 4. Draw staging definition

Draw staging means:

```text
DrawFrame accepted into backend-side accounting.
```

Current admitted path:

```text
DrawFrame
  -> NativeBackendWinitApp::stage_draw_frame(...)
  -> NativeBackend submitted_frames accounting
  -> NativeBackendWinitAppDrawTranscript
```

Draw staging does not mean:

```text
render attempted
render succeeded
presentation attempted
frame presented
semantic action succeeded
effect committed
```

`submitted_frames` must not be treated as `presented_frames`.

## 5. Render attempted definition

Render attempted means a future admitted renderer tried to interpret/render a frame.

It may fail before output exists.

Render attempted must record:

* renderer identity;
* input frame identity if available;
* attempted command count if available;
* unsupported command if any;
* resource availability;
* failure stage if failed;
* trace/transcript relation.

H12 does not implement this.

## 6. Render succeeded definition

Render succeeded means the renderer produced a render result.

It does not mean presentation happened.

Examples:

```text
CPU raster output produced
GPU command buffer encoded
surface texture prepared
offscreen frame produced
```

Render success must not be shown as frame presentation.

## 7. Presentation attempted definition

Presentation attempted means the renderer/native presentation path tried to display a rendered frame.

Presentation may fail after render success.

Examples:

```text
surface lost
swapchain invalid
window closed
native presentation rejected
backend not ready
```

Presentation attempted must be distinct from render attempted.

## 8. Frame presented definition

Frame presented means a frame was successfully submitted through the presentation path and accepted as visible/presented by the admitted presentation boundary.

Frame presented does not mean:

```text
semantic action succeeded
effect committed
audit succeeded
trace authority established
```

Frame presentation is visual output status only.

## 9. Semantic success boundary

Semantic success is owned by semantic action/effect boundaries, not renderer output.

Example:

```text
ui.action.commit_effect
  -> effect request
  -> capability admission
  -> prepared effect
  -> commit boundary
  -> committed effect
```

Renderer may display this result.

Renderer presentation does not create this result.

## 10. Renderer transcript definition

Renderer transcript is a structured record of renderer/presentation facts.

Candidate fields:

```text
draw_frames_seen
draw_frames_staged
render_attempted
render_succeeded
render_failed
presentation_attempted
frame_presented
presentation_failed
unsupported_commands
renderer_errors
surface_status
window_status
```

These names are not API commitments.

They define conceptual vocabulary for future implementation.

## 11. Renderer transcript vs audit

Renderer transcript is not audit authority by default.

Renderer transcript may be displayed in trace UI.

Renderer transcript may be mapped into trace/audit only through explicit future boundary.

The UI must not imply:

```text
renderer transcript visible == audit record exists
frame presented == effect succeeded
render failed == semantic action failed
```

## 12. Renderer transcript vs native backend transcript

Native backend transcript may describe:

```text
window lifecycle
event loop lifecycle
event translation
draw staging
native window close
```

Renderer transcript may describe:

```text
render lifecycle
renderer resources
surface/presentation lifecycle
frame presentation status
```

These transcript domains must not be collapsed.

## 13. Renderer status vocabulary

Future renderer/presentation status must distinguish at least:

```text
not_admitted
not_initialized
draw_staged
render_attempted
render_succeeded
render_failed
presentation_attempted
frame_presented
presentation_failed
surface_unavailable
window_unavailable
unsupported_draw_command
```

These states must not collapse into one generic rendering state.

## 14. Error/denial/quarantine relationship

Renderer failure may produce status facts.

Renderer denial means renderer operation was refused before attempt.

Renderer failure means renderer operation was attempted and failed.

Renderer quarantine may isolate a frame, surface, command, or renderer state for safe handling.

These states are governed by:

```text
docs/architecture/ui_error_denial_quarantine_visual_boundary.md
```

Renderer status must preserve those distinctions.

## 15. Recovery/rollback relationship

Renderer recovery must distinguish:

```text
retry render
retry presentation
recreate surface
drop staged frame
keep last presented frame
continue without presentation
quarantine renderer state
```

These are governed by:

```text
docs/architecture/ui_recovery_rollback_visual_boundary.md
```

Retry render must not blindly re-present an unsafe frame.

## 16. Trace/audit relationship

Renderer transcript may be projected into trace UI.

Trace UI may show:

```text
draw staged
render attempted
render failed
frame presented
presentation failed
```

But trace projection must not make renderer transcript the source of truth for audit.

Audit mapping requires explicit boundary.

## 17. Component relationship

Components may expose renderer transcript/presentation status surfaces.

Candidate components:

```text
RendererTranscriptView
DrawStagingStatusView
RendererAttemptView
FramePresentationStatusView
SurfaceStatusView
PresentationFailureView
```

Components must not invent renderer status meaning.

They display admitted renderer transcript facts.

## 18. Layout relationship

Layout primitives may provide renderer status regions.

Examples:

```text
TraceLane
InspectorPane
EffectLane
OverlaySurface
StatePanel
```

Layout primitives must not own renderer status semantics.

## 19. Native backend relationship

Native backend may participate in presentation only after an admitted renderer/native presentation boundary exists.

Native backend must not decide:

* render success;
* semantic success;
* audit authority;
* renderer transcript meaning;
* effect success.

Native backend may expose native surface/window facts if admitted later.

## 20. Workbench relationship

Workbench may display renderer transcript and presentation status.

Workbench must not define core renderer transcript semantics.

Workbench-specific renderer views require:

* Workbench-local projection namespace; or
* explicit admission into core renderer transcript contract; or
* separate boundary document.

## 21. Required visual distinction table

Future visual implementation must distinguish:

| Condition           | Not equivalent to  | Required visibility                        |
| ------------------- | ------------------ | ------------------------------------------ |
| draw staged         | rendered/presented | staging/accounting only                    |
| render attempted    | render succeeded   | attempted stage                            |
| render succeeded    | frame presented    | output exists, presentation not guaranteed |
| frame presented     | semantic success   | visual output only                         |
| presentation failed | render failed      | presentation stage                         |
| renderer transcript | audit record       | projection only unless mapped              |
| unsupported command | renderer crash     | unsupported input reason                   |

## 22. Forbidden shortcuts

The system must not:

* treat submitted frame count as presented frame count;
* treat draw staging as render attempt;
* treat render success as presentation success;
* treat frame presentation as semantic action success;
* treat renderer transcript as audit authority by default;
* let renderer define semantic success;
* let native backend define renderer transcript meaning;
* hide presentation failure behind successful render;
* collapse renderer error, denial, and failure;
* bypass trace/projection boundaries for renderer facts.

## 23. Required admission rule

A future renderer transcript/presentation implementation PR must define:

1. renderer transcript type;
2. renderer status vocabulary;
3. draw staging relation;
4. render attempt relation;
5. presentation relation;
6. failure/denial/quarantine behavior;
7. trace projection relation;
8. audit mapping if any;
9. component/layout projection;
10. renderer/native ownership boundary;
11. tests/snapshots where applicable.

No renderer transcript field should be added only because it is convenient for logging.

## 24. Future implementation shape

H12 does not mandate implementation.

Possible future shapes:

```text
docs/spec/ui_renderer_transcript.md
crates/prom-ui-renderer/
crates/prom-ui-renderer-transcript/
crates/prom-ui-presentation/
apps/workbench renderer transcript view
renderer-local presentation status map
```

Any implementation must preserve:

```text
DrawFrame
  -> draw staging
  -> render attempt
  -> render result
  -> presentation attempt
  -> presentation result
  -> renderer transcript
  -> optional trace projection
```

## 25. Current decision

Renderer transcript and presentation status are not implemented in H12.

H12 only defines the boundary.

Current admitted visual/interaction/action architecture:

```text
visual doctrine
  -> visual token boundary
  -> layout primitive boundary
  -> component admission boundary
  -> interaction/input semantic boundary
  -> focus/selection semantic boundary
  -> semantic action boundary
  -> effect request / UI capability boundary
  -> trace/audit visual boundary
  -> error/denial/quarantine visual boundary
  -> recovery/rollback visual boundary
  -> renderer transcript / presentation status boundary
```

Not yet admitted:

```text
renderer transcript structs
presentation status structs
renderer implementation
surface ownership
wgpu/pixels/softbuffer
frame presentation
presented frame counter
renderer trace mapping
renderer audit mapping
Workbench renderer transcript UI
native backend presentation authority
```
