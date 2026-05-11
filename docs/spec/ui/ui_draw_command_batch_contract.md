# UI Draw Command Batch Contract

Status: Draft
Track: POST-UI
Depends on:
- `host_runtime_effect_path_boundary.md`
- `ui_effect_envelope_v0.md`
- `ui_capability_taxonomy.md`
- `ui_event_envelope_model.md`
- `ui_frame_lifecycle_contract.md`
Scope: minimal draw command batch contract only
Implementation: out of scope

Related:

- `README.md`
- `../../architecture/ui_full_effect_trace_ladder.md`
- `../../architecture/ui_committed_effect_boundary.md`
- `ui_runtime_adapter_boundary.md`

## 1. Purpose

This document defines the minimal bounded draw command batch accepted by
`SubmitDrawCommands`.

```text
DrawCommandBatch is bounded declarative UI output data.
It is not a renderer, not a GPU command buffer, and not a platform handle.
```

## 2. Relationship to I50-I54

- `I50` defines the host runtime effect path.
- `I51` defines `UiEffectEnvelope` and the `SubmitDrawCommands` effect.
- `I52` defines `UiDrawSubmit` capability.
- `I53` defines deterministic event input.
- `I54` defines frame lifecycle and the active frame requirement.
- `I55` defines the minimal draw command batch shape.

## 3. Core rule

```text
A draw command batch is a bounded declarative list of UI output commands
submitted into an active frame.
```

Rules:

- no raw GPU handles;
- no platform-native renderer objects;
- no shader code;
- no unbounded command list;
- no draw submission outside an active frame.

## 4. DrawCommandBatch shape

The v0 pseudo-shape is:

```text
DrawCommandBatchV0 {
  batch_version: 0,
  batch_id: DrawBatchId,

  target: DrawBatchTarget,
  bounds: DrawBatchBounds,
  commands: [DrawCommandV0],

  style_table?,
  resource_refs?,
  audit_summary?
}
```

### 4.1 `batch_version`

```text
batch_version: 0
```

Rules:

- unknown major version -> `InvalidDrawBatch`;
- no silent reinterpretation between versions;
- `batch_version` is independent from `UiEffectEnvelope` version.

### 4.2 `batch_id`

```text
batch_id: DrawBatchId
```

Rules:

- unique inside one frame;
- not global;
- not persistent;
- used for audit correlation;
- not a capability.

### 4.3 `target`

```text
DrawBatchTarget {
  window_id,
  frame_id
}
```

Rules:

- `window_id` must belong to current runtime session;
- `frame_id` must be active;
- `frame_id` must belong to `window_id`;
- target must match the `SubmitDrawCommands` envelope target.

### 4.4 `bounds`

```text
DrawBatchBounds {
  command_count,
  byte_size?,
  max_text_len?,
  max_resource_refs?
}
```

Rules:

- `command_count` is required;
- `command_count` must be budgeted;
- unbounded batch is invalid;
- runtime profile may set numeric limits later.

## 5. Allowed DrawCommand v0 set

The admitted v0 set is:

- `Clear`
- `Rect`
- `Line`
- `Text`
- `ClipBegin`
- `ClipEnd`

## 6. Command payloads

### 6.1 `Clear`

Purpose:

```text
Set frame background or clear region.
```

Shape:

```text
Clear {
  color
}
```

Rules:

- allowed once or multiple times depending on runtime policy;
- does not expose platform surface;
- color uses deterministic color model.

### 6.2 `Rect`

Purpose:

```text
Draw a filled or stroked rectangle.
```

Shape:

```text
Rect {
  x,
  y,
  width,
  height,
  fill?,
  stroke?,
  radius?
}
```

Rules:

- dimensions must be finite;
- negative width/height are invalid;
- radius is optional;
- style must be inline or a style_table reference.

### 6.3 `Line`

Purpose:

```text
Draw a line segment.
```

Shape:

```text
Line {
  x1,
  y1,
  x2,
  y2,
  stroke
}
```

Rules:

- coordinates must be finite;
- stroke width must be bounded and finite.

### 6.4 `Text`

Purpose:

```text
Draw bounded text.
```

Shape:

```text
Text {
  x,
  y,
  text,
  style
}
```

Rules:

- text length must be bounded;
- text must be normalized UTF-8;
- font selection is logical, not platform-native;
- no font file loading in v0.

### 6.5 `ClipBegin`

Purpose:

```text
Begin a rectangular clipping scope.
```

Shape:

```text
ClipBegin {
  x,
  y,
  width,
  height
}
```

Rules:

- clip stack depth must be bounded;
- clip region must be finite;
- must be balanced by `ClipEnd`.

### 6.6 `ClipEnd`

Purpose:

```text
End current clipping scope.
```

Shape:

```text
ClipEnd
```

Rules:

- `ClipEnd` without `ClipBegin` is invalid;
- batch must not end with unclosed clip scopes.

## 7. Coordinate model

```text
CoordinateSpace = logical window-local coordinates
```

Rules:

- origin is top-left unless future profile says otherwise;
- coordinates are logical, not physical pixels;
- device pixel ratio is platform/runtime concern;
- no direct access to OS DPI APIs from the VM;
- all coordinates must be finite deterministic numeric values.

Numeric model:

- coordinate values are logical scalar numbers;
- exact encoding is a future implementation detail.

## 8. Color and style model

### Color

```text
Color = Rgba8(r, g, b, a)
```

Rules:

- components are 0..255;
- no platform color object;
- no ICC/color-management policy in v0.

### Stroke

```text
Stroke {
  color,
  width
}
```

### Fill

```text
Fill {
  color
}
```

### TextStyle

```text
TextStyle {
  color,
  size,
  family?
}
```

Rules:

- family is logical;
- platform font resolution is adapter/runtime concern;
- no font handles in batch.

## 9. Resource policy

External resources are not admitted in `DrawCommandBatchV0`.

Reserved:

- `Image`
- `Texture`
- `FontFace`
- `SvgPath`

`resource_refs?` is reserved but must be empty in v0.

This prevents early texture/image scope creep.

## 10. Ordering rules

Commands execute in list order.

Later commands may visually cover earlier commands.

`ClipBegin` and `ClipEnd` affect following commands until closed.

Batch order inside one frame follows `SubmitDrawCommands` order.

Across multiple batches in one frame:

```text
SubmitDrawCommands(batch A)
SubmitDrawCommands(batch B)
```

means `A` is ordered before `B`.

## 11. Frame attachment rules

A batch is valid only when:

- submitted through `SubmitDrawCommands`;
- target `frame_id` is active;
- target `window_id` owns `frame_id`;
- envelope has `UiDrawSubmit` capability;
- `DrawSubmission` budget is available;
- frame lifecycle allows submission.

Invalid:

- batch outside active frame;
- batch after `EndFrame`;
- batch before `BeginFrame`;
- batch for different `window_id`;
- batch reusing old `frame_id`.

## 12. Budget hooks

`DrawSubmission` budget may account for:

- `command_count`;
- `batch_count_per_frame`;
- `total_command_count_per_frame`;
- `byte_size`;
- `text_bytes`;
- `clip_stack_depth`;
- `resource_ref_count`.

Exact limits belong to a runtime profile or future implementation.

## 13. Audit summary rules

Audit should not dump every command by default.

It should support a summary:

```text
DrawBatchAuditSummary {
  batch_id,
  frame_id,
  window_id,
  command_count,
  command_kinds,
  byte_size?,
  text_bytes?,
  clip_depth_max?,
}
```

Sensitive rule:

Text command content should not be blindly copied into audit by default.

Reason:

drawn text may contain user-visible or user-provided data.

Audit can store:

- text length;
- hash or digest;
- redacted preview only if policy allows.

## 14. Determinism and replay policy

```text
DrawCommandBatch is deterministic declarative data.
Rendering pixels may be platform-dependent.
Replay must reproduce the same command stream, not necessarily identical pixels
across backends.
```

Semantic determinism means the same VM-visible command batch.
Renderer determinism is out of scope for v0.

## 15. Invalid batch conditions

`InvalidDrawBatch` applies when any of these hold:

- unknown `batch_version`;
- `command_count` is missing;
- `command_count` does not match `commands` length;
- `command_count` exceeds declared budget;
- batch has no active frame;
- `frame_id` / `window_id` mismatch;
- command kind unknown;
- command payload malformed;
- coordinate is non-finite;
- rectangle has negative width or height;
- stroke width is negative or non-finite;
- color component outside 0..255;
- text is invalid UTF-8;
- text length exceeds bound;
- `ClipEnd` without `ClipBegin`;
- unclosed `ClipBegin` at batch end;
- clip stack depth exceeds bound;
- `resource_refs` is non-empty in v0;
- raw OS, GPU, or platform handle appears anywhere;
- command contains callback, function pointer, or raw pointer.

## 16. Forbidden content

A `DrawCommandBatch` must not contain:

- raw OS handles;
- raw GPU handles;
- file descriptors;
- sockets;
- raw pointers;
- callbacks;
- closures;
- platform-native renderer objects;
- shader source;
- compiled shader blobs;
- texture handles;
- image decoder state;
- font handles;
- frontend AST nodes;
- IR objects;
- SemCode mutable references;
- unbounded strings;
- unbounded command arrays.

## 17. Reserved future commands

Not admitted in v0:

- `Image`
- `Texture`
- `Path`
- `Bezier`
- `Gradient`
- `LayerBegin`
- `LayerEnd`
- `TransformPush`
- `TransformPop`
- `OpacityPush`
- `OpacityPop`
- `FontFace`
- `SvgPath`
- `ShaderEffect`
- `CustomDraw`

These require resource, renderer, transform, or backend-specific contracts.

## 18. Extension policy

No new command without:

- command kind;
- payload schema;
- bounds rules;
- budget accounting;
- audit summary policy;
- determinism classification;
- forbidden content check;
- invalid condition list;
- renderer/backend independence review.

No new draw command that requires platform handles.

## 19. Out of scope

This document does not add:

- Rust structs;
- ABI calls;
- VM changes;
- verifier changes;
- `prom-ui-runtime` implementation;
- platform adapter implementation;
- renderer;
- GPU or shader pipeline;
- actual draw command binary encoding;
- font engine;
- image decoder;
- texture upload;
- widget/layout framework;
- retained-mode UI tree;
- scene graph;
- actual event loop;
- tests beyond docs/link checks;
- `.claude/`.

## 20. Acceptance checklist

- `docs/spec/ui/ui_draw_command_batch_contract.md` exists;
- `docs/spec/ui/README.md` links it;
- `docs/spec/index.md` links it;
- `ui_frame_lifecycle_contract.md` cross-links it;
- the document references `I50-I54`;
- `DrawCommandBatchV0` shape is defined;
- allowed command set v0 is defined;
- `Clear` / `Rect` / `Line` / `Text` / `ClipBegin` / `ClipEnd` are specified;
- coordinate model is defined;
- color and style model is defined;
- resource policy says external resources are not admitted in v0;
- ordering rules are defined;
- frame attachment rules are defined;
- budget hooks are defined;
- audit summary rules are defined;
- determinism and replay policy is defined;
- invalid batch conditions are listed;
- forbidden content is listed;
- reserved future commands are listed;
- no code changes;
- no ABI widening;
- no renderer/backend implementation.

## 21. Relationship to runtime adapter boundary

The runtime adapter boundary is defined in:

```text
docs/spec/ui/ui_runtime_adapter_boundary.md
```

This document defines the command batch shape consumed by
`prom-ui-runtime` before adapter dispatch.
