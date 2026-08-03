# Semantic Workbench — Native Semantic + Prom UI Architecture

Status: landed under `SEMANTIC-WORKBENCH-NATIVE-V0` across three closure
passes (see `.harness/current.task.yaml`). This document describes what
actually ships in `examples/workbench_semantic/`, not an aspiration — every
claim below has a corresponding passing test in
`examples/workbench_semantic/src/main.rs`'s `#[cfg(test)] mod tests`, or in
the module it names. This is the *only* canonical Workbench: the prior
React/TypeScript/Tauri app is archived at `apps/workbench_ts_tauri_legacy`
(non-canonical, reference-only — see its README) and the canonical build has
zero Node/npm/Vite/React/Tauri/WebView dependency, enforced by a real
`cargo tree` check
(`canonical_workbench_has_no_js_or_tauri_dependency_footprint`).

Workbench implementation was paused by governance issue `#675`
(`DIR-UI-PAUSE`) for part of this project's history; the project owner
explicitly reopened the track and issue `#675` was closed 2026-08-01 (see
`.harness/current.task.yaml`'s `authorized_by` field). This document reflects
the reopened, active state.

## What this is

A real native desktop application whose application/domain logic is written
in Semantic (`.sm`) and executed through the verified `sm-vm` route on every
user action — not a Rust reimplementation with `.sm` source kept only for
show. It follows the exact pattern already proven by
`examples/quad_logic_calculator/` (the repository's canonical
"Semantic-authored shell" reference), extended with real host effects
(process execution, filesystem discovery, a real bounded job queue) that the
calculator did not need.

```
click / key event
  -> hit-test (Rust)
  -> SemanticIntent + ReferenceActionInvocation
  -> ReferenceContourAdmission (admit/deny)
  -> WorkbenchAction encoded as a Semantic Value
  -> sm_verify::verify_semcode_token + require_entry("apply_action")
  -> sm_vm::run_verified_function_semcode_with_args   (REAL VM execution)
  -> WorkbenchState decoded back from the VM's return value
  -> projection patch (Jobs Ledger / Diagnostics collections, audit trail)
  -> hand-authored DrawFrame render (native winit + wgpu backend)
```

For actions that need a real host effect (running `smc`/`cargo`), the `.sm`
state machine tracks real async job-queue state (`queued_count`,
`running_count`, `concurrency_limit`, a monotonic `job_counter`/
`last_job_id`) — the actual process spawn happens in a narrow,
capability-gated Rust host bridge (`HostCapabilities::check_spawn`), off the
UI thread, and its real result (exit code / stdout / stderr) is fed back in
as a new `WorkbenchAction` (`JobStarted` / `JobSucceeded` / `JobFailed` /
`JobCancelledQueued` / `JobCancelledRunning`) that goes through the *same*
verified VM path. Semantic never touches a file or a process directly; Rust
never decides job-domain semantics.

## Ownership split

| Layer | Owns |
|---|---|
| `examples/workbench_semantic/src/workbench.sm` | Application state, screen/tab selection, real bounded async job-queue admission (FIFO, concurrency-limited, restart-recovery), error/eval-state semantics. Compiled and executed via verified SemCode, exactly like `calculator.sm`. |
| `examples/workbench_semantic/src/workbench.proj.sm` | Grammar v0 projection source — the admitted node/role tree used for admission and the Jobs Ledger / Diagnostics collection anchors. Uses only the 8 roles in `crates/prom-ui/src/role_dictionary.rs` (`root, surface, numeric_readout, danger_action, evidence_panel, recovery_outlet, text, fragment`) — no invented roles. |
| `prom-ui` / `prom-ui-runtime` / `prom-ui-backend-native` (existing, extended) | UI contract types, projection/admission pipeline, shell-player session, native window + wgpu draw submission, plus two generic reusable additions made *for* this app but owned by the runtime layer, not Workbench: `prom_ui_runtime::clip` (CPU-side clip-rect primitive) and `InputEventKind::TextInput` (real committed UTF-8 text from winit). |
| `examples/workbench_semantic/src/main.rs` | Native window lifecycle, hit-testing, VM invocation plumbing, the editor (including real text selection), rendering, and the job scheduler that drives the queue. |
| `examples/workbench_semantic/src/settings.rs` | Local settings persistence (recent projects, last-open project) — a generic "read/write a small local JSON file" module with no Workbench vocabulary in it (enforced by a test, see Validate). |
| `examples/workbench_semantic/src/host_capabilities.rs` | The capability-gated process-spawn boundary (`HostCapabilities::check_spawn`, `resolve_on_path`) — knows nothing about job kinds or screens, only "which real executable paths and directories may be spawned into" (enforced by the same test). |
| `examples/workbench_semantic/src/diagnostics.rs` | Parsing real `smc 7hell --json` output and real plain-text compiler error output into `DiagnosticEntry` values — no Workbench vocabulary in it either. |

An `architecture drift` test
(`generic_host_modules_contain_no_workbench_specific_vocabulary`) scans
`settings.rs`, `host_capabilities.rs`, `diagnostics.rs`, and
`prom-ui-backend-native/src/lib.rs` for Workbench-specific identifiers
(`WorkbenchState`, `JobKind`, `EnqueueJob`, screen/job field names, etc.) and
fails if any appear — the tripwire for Workbench policy drifting into a
layer meant to stay reusable by any Prom UI application.

## Screens

Eight screens, all navigable from the top tab bar: **Cockpit**, **Jobs**,
**Diagnostics**, **Editor**, **Explorer**, **Spec**, **Readiness**,
**Settings**. Screen selection itself is a verified `.sm` transition
(`SelectScreen`), not local Rust UI state.

## Editor

A real multi-tab (bounded to 6 concurrent tabs) text editor: typing,
backspace, delete-forward, arrow-key/Home/End cursor movement, Enter/newline,
Save (button or Ctrl+S), Save All (Ctrl+Shift+S), Reload, dirty markers, an
unsaved-change guard (Save & Close / Discard / Cancel) before a dirty tab
closes, and **real text selection**: anchor/cursor model, extend via
Shift+Arrow, mouse press-drag-release, and Shift+Click; typed text or Enter
replaces an active selection; Backspace/Delete remove it. Character insertion
is driven by real committed UTF-8 text from winit's `KeyEvent.text`
(`InputEventKind::TextInput`), not a hand-rolled ASCII table — physical key
codes (`physical_key_to_char`) now drive only navigation/shortcut routing.
Diagnostics with a real file/line jump straight to that location in the
editor (`navigate_to_selected_diagnostic`).

## Project Explorer

A real recursive, deterministic, bounded (2000-node safety cap) directory
tree with click-to-expand/collapse, skipping
`target`/`.git`/`node_modules`/`.workbench_evidence`/`dist`. Expand state and
tree navigation are Rust-owned presentation state (per the DNA's own "UI
state is projection/cache, not semantic state" rule) — they are not semantic
policy Semantic needs to decide.

## Command bus / Jobs Ledger

Every job goes through `HostCapabilities::check_spawn` before anything is
spawned: the resolved executable must be on a small allowlist (`smc`/`svm`
next to the running binary, `cargo`/`pwsh` resolved from `PATH`), and the
working directory must canonicalize to somewhere inside the open project
root. A denial produces a `JobDenied` action (distinct from a failed run)
through the same verified state machine — it is never silently dropped.

**A real bounded, concurrency-limited FIFO job queue** — not vocabulary-only.
`enqueue_job` pushes a `Queued` ledger entry and calls
`try_start_queued_jobs`, which starts jobs (oldest-first) while
`running_count < concurrency_limit` (default 1, changeable via Settings),
denying/skipping over-capacity dispatch (`queued_count + running_count >=
20`) rather than silently dropping it. Cancelling a still-`Queued` job
(`JobCancelledQueued`) never spawns it; cancelling a `Running` job
(`JobCancelledRunning`) does a real `Child::kill()`. On restart, any
persisted job still `Queued`/`Running` from a prior process is loaded as
`Interrupted` — never silently resumed, never misreported as complete.

Nine job kinds (operating on the currently selected file, except
`cargo-check`/`cargo-test`/`harness-check` which operate on the project
root):

- **Check** — `smc 7hell <file> --json`: real structured diagnostics
  (`id, stage, code, category, severity, source{file,line,column}`).
- **Compile** — `smc compile <file> -o <evidence>/job_<id>.smc`
- **Run** — `smc run <file>`
- **Verify** — a real two-step pipeline: `smc compile` to a temp `.smc`,
  then `smc verify` on it. Both steps' evidence is preserved.
- **Disasm** — `smc compile` to a temp `.smc`, then `svm disasm` on it.
- **Fmt** — `smc fmt <file>` (real in-place formatting, not `--check`); a
  clean open tab for the same file is reloaded automatically after a
  successful format, a dirty one is left untouched with a warning rather
  than silently overwritten.
- **Cargo Check** / **Cargo Test** — `cargo check` / `cargo test --quiet`
- **Harness Check** — this repository's own real admission gate,
  `pwsh -File scripts/harness-check.ps1` (used by the Readiness Console).

Each job produces a deterministic record (id, kind, target file captured at
request time, argv, cwd, resolved executable, status, exit code, duration,
evidence path) and spools its full raw stdout/stderr to
`<project>/.workbench_evidence/job_<id>_<kind>.log` — the on-screen preview
is bounded (600 chars) but the file on disk is never truncated. The ledger
supports kind/status filtering, job selection, Cancel, Rerun (re-dispatches
the same job kind against the same target file, not whatever file happens to
be selected now), Clear, and is persisted to `<evidence>/history.json` so it
survives a restart (with corrupt-file recovery). A stale completion from an
already-cancelled/superseded job is dropped, never overwriting newer state
(`stale_job_completion_does_not_overwrite_newer_pending_state`).

## Diagnostics Hub

Structured diagnostics from `smc 7hell --json` (family = compiler stage:
syntax/type/lowering/verifier/vm/diagnostics), plus a real best-effort
plain-text fallback parser for `Error [CODE]: message at line L:C` output
from Compile/Verify/Run jobs — both are backed by actually-observed tool
output, not invented spans. Clicking a diagnostic with a real span jumps to
that file/line in the Editor; a diagnostic without a real span states so
instead of guessing.

## Inspectors

Five evidence-backed inspector views on the selected job (Jobs screen, lower
panel): **Verify**, **Disasm**, **Runtime**, **Capability**, **Raw
Evidence**. Each reads the job's real spooled evidence file fresh from disk
(`read_evidence_lines`) rather than re-decoding SemCode itself; an inspector
whose kind doesn't match the selected job says so plainly instead of showing
stale or fabricated content. The Capability inspector shows the real
admit/deny outcome and reason, never collapsing "denied" into "failed".

## Spec Navigator

Read-only, deterministic document tree over `docs/{spec,dna,architecture,
workbench,roadmap,status}` (found by walking up from the project root to the
nearest ancestor containing a `docs/` directory). Opening a document renders
it through the same scrollable-text-panel primitive the editor uses
(read-only). A real substring search box (type to build the query, Enter to
jump to the next match, wrapping) — not a decorative box.

## Readiness Console

Non-authoritative evidence console. The "RUN" button dispatches this
repository's own real `scripts/harness-check.ps1` gate as a real job; the
gate rows show `harness-check`, `check`, `compile`, `verify`, and
`cargo-check`'s most recent real job outcome (landed/passing, failing with
its real exit code, or "not run this session" — never a synthetic
percentage or invented score). Every row links to real job evidence.

## Settings

Local JSON settings file (`%APPDATA%/semantic_workbench/settings.json`):
recent projects (click to switch, real rescan), last-open-project restore on
launch, Clear History (clears the job ledger), Reset Local State (clears
persisted settings and expanded-tree state). No telemetry, no hidden upload.
A corrupt or missing settings file recovers to empty settings rather than
crashing startup.

## Clipping

A generic, reusable CPU-side clip-rect primitive lives in
`prom_ui_runtime::clip` (`ClipRect`, `ClipStack`, `clip_fill_rect`,
`clip_text_origin`) — not a Workbench-only hack, and not GPU scissor-rect
clipping (`DrawFrame` still has no clip-rect draw command; the shared wgpu
renderer batches all fills/text into one draw call per frame, so a real
per-draw hardware scissor would need a renderer restructure, which is out of
scope here). Every scrollable panel (Editor, Jobs, Diagnostics, Explorer,
Spec) routes its per-row draws through this primitive, intersecting against
the panel's visible rect before emitting a fill or text draw — the practical
effect of clipping, verified by 10 unit tests on the primitive itself plus
render-site usage, but implemented as geometric intersection rather than a
hardware scissor rect.

## Build and run

```powershell
cargo build -p workbench_semantic
./target/debug/workbench_semantic.exe [path-to-a-project-directory]
```

If no path is given, the last-open project (from Settings) or the current
working directory is opened. Project discovery (the Cockpit file list used
for job dispatch) is bounded and non-recursive beyond one subdirectory level
(`.sm` files directly under the root plus one level of subfolders), capped
at 12 files — the Explorer tree is fully recursive (see above); this
distinction is deliberate v0 scope, not an oversight.

## Native input path

```
winit event -> prom-ui-backend-native translation -> InputEventKind
  -> WorkbenchApp::handle_input_event  (hit-test / key routing / text input)
  -> Action -> verified apply_action (sm-vm) OR direct host effect (spawn/save/scroll)
  -> WorkbenchState decoded back
  -> DrawFrame render
```

Generic capabilities added to `prom-ui-runtime` / `prom-ui-backend-native`
(not Workbench-only hacks) to make this real: the winit key-code table was
extended from 13 keys to the full alphabet/digits/punctuation/navigation-key
set (including a distinct Alt code); `InputEventKind::Scroll` plus
`WindowEvent::MouseWheel` translation; and `InputEventKind::TextInput`, which
carries winit's real layout/shift/IME-resolved committed text
(`translate_winit_text_input`, `is_insertable_text`) so typed characters —
including real non-ASCII UTF-8 — reach the app exactly as a real OS input
method produced them, not through an invented character table.

Native-adapter-boundary tests (`native_adapter_boundary_*` and the
comprehensive `end_to_end_native_adapter_boundary_full_workbench_session`)
reuse the exact headless mechanism `smc look ui frame --events` uses in
production (`InMemoryBackend` + `DesktopSession::tick_in_memory_frame` +
`backend_mut().extend_events(...)`) to drive real `InputEventKind` values
through the same `DesktopSession`/`UiBackendAdapter` plumbing as the live
winit-backed session, calling the identical `WorkbenchApp::handle_input_event`
production entrypoint — not a direct call to an internal method that bypasses
the adapter. This is the same boundary, with the window/GPU surface swapped
for the repository's own established headless stub; it is not OS-level mouse
automation, which no tool in this environment provides. A separate live
process smoke test (`scripts/workbench_native_launch_smoke.ps1`) launches
the real compiled executable, confirms it opens its real native window/event
loop and stays alive with no panic, and captures the run to
`artifacts/workbench/native-launch-smoke/`.

## Validate

```powershell
cargo test -p workbench_semantic
cargo test -p prom-ui-runtime -p prom-ui-backend-native -p prom-ui-demo -p quad_logic_calculator
cargo fmt -p workbench_semantic --check
cargo clippy -p workbench_semantic
smc check examples/workbench_semantic/src/workbench.sm
pwsh -File scripts/harness-check.ps1
pwsh -File scripts/workbench_native_launch_smoke.ps1
```

37 tests in `workbench_semantic` alone exercise real behavior, not mocks:
real project discovery and tab lifecycle through the verified state machine,
real text selection (drag/shift-arrow/shift-click/typed-replacement), real
`smc`/`svm`/`cargo`/`pwsh` subprocesses (including real two-step
Verify/Disasm pipelines and real in-place Fmt), real parsed diagnostics with
real source navigation, real capability denial, real process cancellation
via `Child::kill()`, a real bounded concurrent job queue (FIFO order,
concurrency limit, restart-recovery to `Interrupted`), real persisted job
history with corrupt-file recovery, real recursive directory scanning, the
generic-host-module architecture-drift check, the no-JS/Tauri-dependency
check (a real `cargo tree` run), and one comprehensive end-to-end scenario
that drives every screen through the real adapter boundary in a single
session and verifies state that genuinely persists across a simulated
restart. All pre-existing tests in every touched crate
(`prom-ui-runtime`/`prom-ui-backend-native`/`prom-ui-demo`/
`quad_logic_calculator`) still pass unchanged.

## Known limits

- **No IME composition UI, but composed/committed text is consumed
  correctly.** `InputEventKind::TextInput` carries winit's already-committed
  text (including real non-ASCII UTF-8, proven with real Cyrillic and mixed
  Unicode in `is_insertable_text_accepts_real_unicode_and_rejects_control_
  characters`), but the editor has no in-progress composition (pre-edit)
  rendering of its own — that is a winit/IME-level UI, not something this
  app draws.
- **Selection/cursor columns are counted in `char`s, not extended grapheme
  clusters.** Multi-codepoint graphemes (e.g. combining marks, some emoji)
  will move the cursor per-codepoint rather than per-visual-character. Real
  enough to author `.sm` source (ASCII-dominant), not a full Unicode text
  editing model.
- **"Clipping" is a CPU-side geometric primitive, not GPU scissor-rect
  clipping** (see Clipping section above) — a deliberate, documented
  trade-off given the shared wgpu renderer's current single-draw-call-per-
  primitive-type batching.
- **Rendering is hand-authored `DrawFrame` calls, not the automatic
  `prom_ui::layout` solving pipeline.** This matches the one proven
  precedent (`quad_logic_calculator`) rather than inventing a new pattern;
  `prom-ui-demo` shows the layout pipeline is real and callable, so routing
  through it remains a reasonable follow-up.
- **Semantic cannot own UI label/toolbar text.** Grammar v0's projection
  source has no string/label/text-content construct (only structural
  projection/revision/epoch/surface/node/role/key/child/order/
  collection_anchor) — all on-screen strings (tab labels, button text,
  status messages) are Rust string literals in `main.rs`'s render functions,
  not driven by `.sm` source. Closing this gap would require inventing new
  Grammar v0 syntax, which the `semantic-source-authoring-guard` skill
  forbids without full-stack admission work spanning the parser/typechecker/
  verifier/projection compiler — out of scope for an application-level task.
  Semantic does own everything the DNA requires it to own: application
  state, transitions, admission policy, and job-domain semantics.
- **Documentation drift found, not fixed here:** `docs/architecture/
  ui_native_backend_boundary.md`, `ui_renderer_admission_boundary.md`, and
  `ui_renderer_transcript_presentation_boundary.md` still say renderer/GPU
  presentation is "not admitted yet," which is factually false against the
  `wgpu_integration` module already shipped in `prom-ui-backend-native` and
  exercised by `quad_logic_calculator` and this app. Left unedited to keep
  this task's diff scoped to Workbench; flagged here per the DNA rule that a
  docs/code status conflict is a readiness defect.

## DNA / governance alignment

Per `.agents/skills/semantic/SKILL.md`'s `DNA_RULES`, `docs/dna` was read
before implementation. `docs/dna/SEMANTIC_UI_DNA.md`'s Semantic UI Maturity
Ladder (steps 1–10) gates "Workbench / Studio product work" on a working
"Semantic-authored shell definition" (step 8) and "bounded app shell
prototype" (step 9) existing first. `examples/quad_logic_calculator` is
exactly that prototype — real `.sm` logic driving a real native shell
through verified VM execution. This Workbench slice reuses that same proven
pattern rather than a new one, and proceeds under the project owner's
explicit authorization, which both `.harness/current.task.yaml` and the
closure of governance issue `#675` now record directly.
