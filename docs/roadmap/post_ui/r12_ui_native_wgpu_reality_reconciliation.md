# R12 UI Native/WGPU Reality Reconciliation

Status: Audited
Track: POST-UI / R12
Purpose: record the current documentation/code alignment issue around native WGPU rendering reality

## 1. Observation

The current repository contains two different kinds of UI renderer truth that must not be allowed to drift silently:

1. Earlier renderer boundary documents still contain language that says renderer ownership, WGPU surface ownership, GPU command submission, and frame presentation are not admitted yet.
2. The current source tree and recent PR history now contain feature-gated `winit`/`wgpu` native presentation behavior, including a native window path, WGPU context setup, surface configuration, `DrawCommand` translation, render pass submission, and frame presentation.

This is not necessarily an architecture violation. It may simply mean that implementation has advanced past older boundary language.

It is, however, a documentation reality gap and should be reconciled explicitly.

## 2. Why this matters

Semantic UI depends on strict authority separation:

```text
UI displays.
Renderer presents.
Runtime stages.
Verifier admits.
VM executes.
Semantic core owns truth.
```

If the docs say renderer presentation is not admitted while the code already presents frames behind feature gates, a reader cannot easily determine which surface is the current source of truth.

That weakens the main architectural property of the UI track: clear separation between display, rendering, presentation, interaction, semantic action, effect, and audit authority.

## 3. Current reality to reconcile

Current code reality appears to include at least these facts:

```text
prom-ui-backend-native
  -> optional winit-backend feature
  -> optional wgpu-backend feature
  -> NativeBackend staged state/accounting
  -> WinitRunLoopHost path
  -> NativeBackendWgpuContext
  -> NativeBackendPresentationSurface
  -> DrawCommand::Clear / FillRect translation
  -> wgpu render pass
  -> queue.submit(...)
  -> frame.present()
```

Recent UI demo work also shows an application-facing shell path:

```text
UiProjectionArtifact
  -> render model
  -> layout model
  -> physical placement
  -> DrawFrame
  -> NativeBackend
  -> native WGPU demo window
```

The calculator shell demo is correctly framed as UI-local/inert: it displays a calculator-like surface and captures local intent preview, but it does not implement calculator semantics, does not execute Semantic logic, and does not make the UI a source of truth.

## 4. Required reconciliation decision

The project should make one explicit decision in a follow-up boundary/closeout PR:

### Option A — admit the current WGPU path as the new controlled baseline

If the current WGPU presentation path is accepted, update the renderer admission/status docs to say:

```text
WGPU foundation: admitted behind feature gate
native surface/presentation: admitted for demo/baseline scope only
renderer transcript: still requires explicit completion if incomplete
text rendering: not admitted as full renderer capability unless separately documented
UI semantic authority: unchanged, not granted
```

### Option B — quarantine the current WGPU path as experimental

If the current WGPU path is not yet admitted, mark it explicitly as experimental/quarantined and require a future admission PR before treating it as baseline.

```text
WGPU code exists.
WGPU baseline is not yet canonical.
Docs remain authoritative until admission closeout.
Demo behavior must not be cited as admitted renderer contract.
```

### Option C — split the status

If the correct reality is mixed, record it precisely:

```text
WGPU dependency/foundation: admitted
minimal offscreen draw: admitted
native window surface: admitted for demo only
frame presentation: admitted for demo only
renderer transcript: not complete
public UI renderer contract: not stable
```

This is likely the safest reconciliation model because it preserves progress without widening the public contract too far.

## 5. Non-authority invariant

No reconciliation option should grant the renderer authority over Semantic meaning.

The following must remain true:

```text
frame presented != semantic success
render succeeded != action admitted
draw staged != frame visible
calculator shell intent != calculator truth
UI local state != VM state
renderer transcript != audit authority
```

## 6. Suggested follow-up artifact

Create a dedicated closeout or audit document, for example:

```text
docs/roadmap/post_ui/r12_ui_native_wgpu_renderer_reality_audit.md
```

That document should enumerate:

* current code reality;
* current docs reality;
* stale/superseded boundary language;
* admitted vs experimental capabilities;
* no-authority invariants;
* remaining gaps before renderer contract stabilization.

## 7. Explicit non-scope of this note

This note does not:

* change code;
* admit new renderer capability;
* change `UiBackendAdapter`;
* change `prom-ui-runtime`;
* change VM/verifier/SemCode behavior;
* claim stable public renderer contract;
* mark WGPU presentation as fully production-ready.

It only records the need to reconcile code reality and architectural documentation.

## 8. Practical next step

This note has now been superseded by the dedicated reality audit at:

- [r12_ui_native_wgpu_renderer_reality_audit.md](./r12_ui_native_wgpu_renderer_reality_audit.md)

Until a separate closeout updates the older boundary docs, the safest wording is:

```text
The native WGPU path exists behind feature gates and is demonstrated in the UI demo, but the exact renderer admission status must be reconciled against the older boundary documents before it is treated as a widened stable UI contract.
```
