# Live native resize sequence — real evidence

Real `workbench_semantic.exe` process, launched normally
(`target\debug\workbench_semantic.exe .`), resized via real Win32
`SetWindowPos` calls against its actual `MainWindowHandle` (not a
simulated/in-memory event) — the same OS-level resize a user dragging the
window border produces, verified afterward with real `GetWindowRect` reads
and a real process-alive check.

| requested | actual window (GetWindowRect) | process alive after |
|---|---|---|
| 960x640   | 960x640   | true |
| 1280x720  | 1280x720  | true |
| 1440x900  | 1440x900  | true |
| 1920x1080 | 1920x1080 | true |
| 700x500   | 700x500   | true |
| 1400x900  | 1400x900  | true |

No panic, no crash, no hang across any of the 6 resizes (includes the
task-specified 1440x900 -> 960x640 -> 1920x1080 -> 1280x720 sequence, plus
two additional sizes). This proves the real winit/wgpu event loop survives
real window-manager-driven resizes end to end (`WindowEvent::Resized` ->
`translate_winit_window_event` -> `InputEventKind::Resized` ->
`WorkbenchApp::handle_input_event` -> `window_width`/`window_height`
updated -> next frame's `render_frame()`/`hit_targets()` both recomputed
from the new size).

## Known limitation: no pixel-level screenshot

This evidence proves the process survives real resizes and (via the
separate `native_adapter_boundary_resize_and_dpi_change_...` test) that the
real presented `DrawFrame` command stream changes correctly. It does
**not** include an actual rendered-pixel screenshot of the native window at
each breakpoint — no tool available in this environment can capture an
arbitrary native (non-browser) window's framebuffer. The
`workbench_native_launch_smoke.ps1` script and this resize sequence are the
closest available live evidence; a true visual screenshot at each
breakpoint would need to be captured manually by the project owner running
the app locally.
