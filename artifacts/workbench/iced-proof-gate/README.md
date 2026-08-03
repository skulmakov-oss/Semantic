# Iced adapter proof-gate — real evidence

Owner directive section 7 ("FIRST PROOF -- DO NOT MIGRATE EVERYTHING
BLINDLY") required proving the full real pipeline with one small,
application-neutral, non-Workbench fixture before migrating any Workbench
screen. Fixture source:
`crates/prom-ui-iced-adapter/examples/fixture.rs`. Run with:

```
cargo run -p prom-ui-iced-adapter --example fixture
```

## What was actually proven

The real, full loop the directive requires:

```
Semantic/Prom projection -> Iced adapter -> actual Iced widgets ->
native WGPU/Winit frame -> Iced message -> SemanticActionId ->
verified state transition -> updated projection -> updated frame
```

1. `01_initial_render.png` -- the real native window (`workbench_semantic`-
   unrelated: title "Prom UI Iced Adapter -- Proof Fixture"), captured via
   real Win32 `GetWindowRect`/`CopyFromScreen` against the real, live
   process's real window handle (verified as the real foreground window by
   `GetForegroundWindow` beforehand, not assumed). Shows every required
   fixture element in one frame: a `Header` with a `StatusBadge`
   ("counter: 0"), a `SplitPane` with a real draggable-width divider, a
   `SearchField`, a scrollable `ArtifactList` (`TreeRow`s) with real UTF-8
   content including Cyrillic (`отчёт-финал.md`,
   `проверка-юникода.txt`) rendered correctly through Iced's real
   `cosmic-text` shaping, a `DataTable` with `TableColumn`/`TableRow`s and
   `StatusBadge` cells, two `Button`s, and a real wrapped multi-line
   Cyrillic `Text` node.

2. A real mouse click was sent via Win32 `SetCursorPos`/`mouse_event`
   (real OS input injection, not a simulated/in-memory event) to the real
   screen coordinates of the "increment" button.

3. `02_after_increment_click.png` -- the same real window, re-captured
   after that real click, showing the `StatusBadge` now reading
   **"counter: 1"** -- the real click became a real `iced::Message`,
   converted to `PromUiMessage::Action(SemanticActionId(1))`, routed
   through the adapter's `update` into `FixtureApp::dispatch`, which
   incremented real application state, which produced a real new
   `PromNode` tree on the next `view()` call, which the adapter converted
   into a real new `iced::Element` tree and presented in a real new frame.
   This is the complete loop, proven live, not asserted.

## A resolved false alarm (documented for honesty, not silently dropped)

The very first screenshot captured during this proof pass showed an
unexpected teal "selected" highlight on the last artifact row, even
though the fixture's `selected: Option<usize>` state starts `None` (no
row selected). Before accepting the fixture as proof, this was
investigated rather than dismissed: a temporary diagnostic build printed
the real `is_selected` boolean for every row on every `view()` call to
stderr, confirming `is_selected=false` for all 5 rows. A fresh screenshot
of the *same running, unmodified* fixture (no code change, just a later
frame) then showed the highlight gone. Conclusion: a one-off transient
first-paint rendering artifact (most likely stale compositor/video-memory
content from window creation), not a real bug in `PromNode`/`PromState`/
the converter -- confirmed by both the state-level diagnostic and a
second, clean screenshot. The diagnostic code was removed after
confirming this; it is not part of the shipped fixture.
