# UI Frame Inspection CLI (`smc look ui frame`)

## Status

Implemented, v0. Issue #1365.

Deterministic, read-only UI frame inspection. It does not open a native
window, does not use a GPU, and does not claim general Semantic UI-source
support beyond what is documented below.

## Relationship to other UI evidence

This is **not** the same thing as `docs/spec/ui/draw_command_snapshot_policy.md`.
That policy governs `experiments/ui-shell-kit`'s textual scene-evidence
snapshots and explicitly must not define `prom-ui` renderer behavior. This
document governs a CLI that inspects the real
`prom_ui_runtime::{DrawFrame, DrawCommand}` types produced by the actual
UI-DNA2 pipeline (`prom-ui` + `prom-ui-runtime`), either loaded from a
canonical snapshot file or captured live through an in-memory backend. The
two are unrelated formats serving unrelated experimental/production tracks.

This is also not `docs/spec/ui/gate_d_activation_policy_v0.md` (the policy
this command *uses*, unmodified) and not the UI-DNA2-10 reference
application itself (`crates/prom-ui-demo`, whose non-native logic this
command reuses via `prom_ui_runtime::reference_contour`).

## Purpose

Make it possible to inspect real `DrawFrame` output -- for an existing
canonical frame snapshot, or for a deterministic, in-memory run of the
current UI-DNA2 reference contour -- without a native window or GPU.

```text
current admitted UI input
  -> deterministic in-memory execution (source mode only)
  -> DrawFrame capture
  -> stable text or canonical draw-json output
```

## Command grammar

```text
smc look ui frame --from <snapshot> [--frame <n>] [--format text|draw-json] [--out <path>]
smc look ui frame <source-file> [--events <script>] [--frame <n>] [--format text|draw-json] [--out <path>]
```

Exactly one primary input is required: `--from <snapshot>` (snapshot mode)
XOR a positional `<source-file>` (source mode). Supplying both, or neither,
or `--events` together with `--from`, is `InvalidArguments`.

| Option | Meaning | Default |
|---|---|---|
| `--from <path>` | Load an existing canonical Frame Snapshot v0 document. Never compiles or runs source, never activates a bundle. | -- |
| `<source-file>` | Projection Source v0 text to compile and run through the real UI-DNA2 pipeline. | -- |
| `--events <path>` | Deterministic Event Script v0 to inject in source mode. Invalid together with `--from`. | no events (only the initial frame is captured) |
| `--frame <n>` | Zero-based captured frame index to inspect. | `0` |
| `--format text\|draw-json` | Stable human-readable text, or canonical machine-readable JSON. | `text` |
| `--out <path>` | Write output atomically to a file instead of stdout. | stdout |

## Accepted source profile (read this before assuming general `.sm` support)

**Source mode accepts Projection Source v0 text** -- the same textual
grammar `prom_ui::shell_bridge::compile_projection_source_to_bundle_v0`
already compiles (`docs/spec/ui/projection_source_grammar_v0.md`), the same
grammar the UI-DNA2-10 reference application
(`crates/prom-ui-demo/src/ui_dna2_reference.rs`) has always used. It is
**not** ordinary Semantic (`.sm`) language source, and ordinary `.sm`
programs do not emit `DrawFrame` values through the general Semantic VM in
this repository today -- this command does not claim otherwise.

Compilation, verification, bounded Gate D activation, and layout are
**generic** over the supplied text: any structurally valid Grammar v0
document compiles, verifies, activates, and lays out its own declared
node/child structure. The one thing that is **not** generic, and does not
change no matter what source text you supply, is the admission grant set:
`prom_ui_runtime::reference_admission::ReferenceContourAdmission` in this
bounded reference contour only ever grants interaction-binding id `12`
(`BUTTON_NODE` in the reference document shape). An interaction targeting
any other node id is always denied -- this is a fixed property of the one
bounded contour Gate D currently authorizes
(`docs/spec/ui/gate_d_activation_policy_v0.md`), not something this CLI
widens, narrows, or works around.

## Source-driven pipeline (every arrow real, existing code)

```text
Projection Source v0 text
  -> prom_ui::shell_bridge::compile_projection_source_to_bundle_v0   (parse, Static UI IR, CollectionAnchor qualification, canonical bytes)
  -> prom_ui::shell_bridge::activate_projection_bundle_v0_gate_d      (structural/cross-artifact/compatibility/self-consistency verification + bounded Gate D activation)
  -> prom_ui_runtime::shell_player::create_shell_session              (Shell Player)
  -> prom_ui_runtime::reference_contour::ReferenceLayout               (deterministic vertical-stack layout, generic over declared children)
  -> deterministic event injection (Event Script v0, or none)
  -> prom_ui_runtime::reference_contour::dispatch_input_event          (same event vocabulary the native reference app has always used)
  -> prom_ui_runtime::reference_contour::render_frame -> DrawFrame
  -> prom_ui_runtime::{DesktopSession<InMemoryBackend>, InMemoryBackend} frame capture
  -> selected frame -> stable inspection output
```

`prom_ui_runtime::reference_contour` is the extraction this issue required:
the deterministic, backend-agnostic core of the UI-DNA2-10 reference
application (previously private to `prom-ui-demo`, which had no `[lib]`
target and could not be depended on) is now a `pub` module of
`prom-ui-runtime`. `prom-ui-demo`'s native application now imports from it
instead of duplicating it; its native windowing loop and console narration
are the only things that remained in `prom-ui-demo`.

No native backend (`prom-ui-backend-native`, winit, wgpu) is anywhere in
`smc-cli`'s dependency graph. Source mode drives
`DesktopSession<InMemoryBackend>` via `tick_in_memory_frame`, the same
headless seam `prom-ui-runtime`'s own tests use.

## Canonical Frame Snapshot v0

Schema identity: `"semantic.ui.frame_snapshot.v0"`, `schema_version: 0`.
Owned by `smc-cli` (`crates/smc-cli/src/ui_frame_snapshot.rs`) -- this is
CLI inspection evidence, not a runtime contract, so it does not live in
`prom-ui`/`prom-ui-runtime`.

One schema, used identically as the `--from` input file and as the
`--format draw-json` output: `draw-json` always emits the **full** captured
or loaded multi-frame document (every frame, not only the selected one),
which is what makes the round-trip property below simple and testable.
`--frame` still governs which frame's commands are shown in `--format text`,
and `FrameNotFound` is checked identically in both formats before any output
is produced.

```json
{
  "schema": "semantic.ui.frame_snapshot.v0",
  "schema_version": 0,
  "frame_count": 1,
  "frames": [
    {
      "index": 0,
      "commands": [
        {"kind": "clear", "color": {"r": 18, "g": 18, "b": 24, "a": 255}},
        {"kind": "fill_rect", "rect": {"x": 20, "y": 20, "width": 560, "height": 110}, "color": {"r": 50, "g": 56, "b": 74, "a": 255}},
        {"kind": "draw_text", "text": "Label (text role)", "x": 32, "y": 44, "color": {"r": 255, "g": 255, "b": 255, "a": 255}}
      ]
    }
  ]
}
```

All three `DrawCommand` variants (`Clear`, `FillRect`, `DrawText`) are
represented with every field, in the exact order shown. No floating-point
fields exist anywhere in this schema -- `DrawFrame`'s real fields are all
integer (`i32`/`u32`/`u8`), so there is no float-normalization ambiguity to
solve.

**Canonical encoding**: compact form, no insignificant whitespace anywhere,
fixed key order (`schema`, `schema_version`, `frame_count`, `frames`; per
frame `index`, `commands`; per command `kind` then its fields in the order
above), exactly one trailing `\n`, no other trailing whitespace. The decoder
is a purpose-built strict parser for exactly this schema (not a general JSON
library, matching `smc-cli`'s existing zero-external-JSON-dependency
convention and `prom-ui`'s own hand-rolled-parser style) -- it accepts only
this exact canonical byte shape, so a pretty-printed or key-reordered
reformat of an otherwise-equivalent document is rejected as noncanonical,
not silently normalized.

**Rejected, deterministically, before or during parsing (never after
unbounded allocation)**: oversized input (`MAX_SNAPSHOT_BYTES = 4 MiB`,
checked against file size before read), malformed JSON, trailing bytes
after the document, missing trailing newline, unsupported `schema_version`,
unknown `schema` string, unknown command `kind`, out-of-range integers
(e.g. a color component `> 255`, negative `width`/`height`), a `frame_count`
that does not match the actual `frames` array length, non-sequential or
duplicate `index` values, more than `MAX_FRAMES = 4096` frames, more than
`MAX_COMMANDS_PER_FRAME = 4096` commands in one frame, a `DrawText.text`
longer than `MAX_TEXT_BYTES = 8192` UTF-8 bytes, and any noncanonical
formatting (leading zeros, a `+` sign, insignificant whitespace).

## Deterministic Event Script v0

Schema identity: `"semantic.ui.event_script.v0"`, `schema_version: 0`.
Owned by `smc-cli` (`crates/smc-cli/src/ui_event_script.rs`), source mode
only.

```json
{
  "schema": "semantic.ui.event_script.v0",
  "schema_version": 0,
  "steps": [
    {"events": [{"kind": "pointer_move", "x": 300, "y": 201}]},
    {"events": [{"kind": "pointer_down", "button": 0}]},
    {"events": [{"kind": "key_down", "key_code": 9}, {"kind": "key_down", "key_code": 13}]}
  ]
}
```

**Frozen frame-step model**: each `steps[i]` is delivered as exactly one
batch to one `DesktopSession::tick_in_memory_frame` call and produces
exactly one captured frame (multiple events may share a step -- delivered
together, matching `tick_in_memory_frame`'s own real "whatever was queued
since the last drain" batching; this is the runtime's existing behavior, not
an invented rule). Frame `0` is always the initial state, captured before
any step runs -- **captured frame count is always `steps.len() + 1`**,
deterministically, regardless of script content.

Supported `kind`s are exactly the vocabulary
`prom_ui_runtime::reference_contour::dispatch_input_event` has ever
processed -- the same match arms the native reference application's own
event loop has always used:

| `kind` | Fields | Effect |
|---|---|---|
| `pointer_move` | `x`, `y` (integers) | hover/hit-test update |
| `pointer_down` | `button` (unsigned integer, ignored) | hit-test + trigger (admit or deny) at the hovered node |
| `pointer_up` | `button` | delivered, currently a no-op (same as the native path) |
| `key_down` | `key_code` (unsigned integer) | `9` = Tab (cycle focus), `13`/`32` = Enter/Space (activate focused node), any other code is a no-op |
| `key_up` | `key_code` | delivered, currently a no-op |
| `close` | none | delivered, currently a no-op outside the native run loop's own exit branch (see Non-claims) |

`x`/`y`/`button`/`key_code` **must be JSON integers** (no fractional part),
bounded to `±MAX_COORDINATE_MAGNITUDE = 1_000_000`. This is a deliberate
simplification, not a partial float feature: pointer coordinates are
integers in this format, full stop, so there is no floating-point
parsing/formatting determinism problem to solve at all.

Bounds: `MAX_EVENT_SCRIPT_BYTES = 256 KiB`, `MAX_STEPS = 4096`,
`MAX_EVENTS_PER_STEP = 64`, `MAX_TOTAL_EVENTS = 16384`. An unknown `kind` is
`InvalidEventScript`, never a silent fallback.

## Frame lifecycle and selection

- `--frame` defaults to `0`; `--frame 0` is deterministic in both modes.
- The report always states requested frame, captured frame count, and
  selected frame.
- Requesting an index `>=` the captured frame count is `FrameNotFound` --
  never a panic, never an implicit "last frame," never an empty success.
- `status`/`admission` in the text report describe the state **as of the
  selected frame** (source mode tracks which interaction, if any, was most
  recently denied at each captured frame), not the script's final step --
  selecting an earlier frame with `--frame` never shows a later step's
  outcome.

## Stable text output

Deterministic, golden-testable. Fields: command/version line, mode
(`source`/`snapshot`), input path, source profile (or `n/a (snapshot
mode)`), events path (or `none`), requested frame, captured frame count,
selected frame, `status`, `admission` (last-denied at the selected frame, or
`n/a (snapshot mode)`), ordered draw-command count and listing, `warnings`
(currently always `none` -- the field exists for forward compatibility), and
the non-authority notice. No timestamps, random ids, absolute temp paths,
memory addresses, or `Debug`-formatted internals -- it is a purpose-built
renderer, not `{:?}` output.

## Canonical draw-json output

Exactly the Frame Snapshot v0 document described above: canonical JSON,
stable key/array order, exactly one trailing newline, no other whitespace.
Running the same command twice with the same inputs produces byte-identical
output (verified in `tests/cli_look_ui_frame.rs`). A snapshot written by
source mode with `--out` and re-read with `--from --format draw-json`
reproduces the original bytes exactly.

## Result taxonomy

`Captured` / `Loaded` (success), `Denied` (a successful capture whose
selected frame's most recent interaction was denied -- `Denied != Failed`;
the process still exits `0` and prints a complete report), `InvalidArguments`,
`InvalidSource`, `CompileFailed`, `ActivationDenied` (fuses verification and
Gate D activation, see Non-claims), `InvalidEventScript`, `InvalidSnapshot`,
`UnsupportedVersion`, `ResourceLimitExceeded`, `FrameNotFound`,
`UnsupportedFormat`, `OutputWriteFailed`, `InternalInspectionFault`. Every
non-success status maps to a non-zero exit and an `"<Status>: <message>"`
stderr line; the `<message>` preserves the underlying diagnostic's own
`Display`/`Debug` text rather than re-wording it.

## Resource limits

Every untrusted file (source text, snapshot, event script) is size-checked
via `fs::metadata` **before** it is read into memory:
`MAX_SOURCE_BYTES = 1 MiB`, `MAX_SNAPSHOT_BYTES = 4 MiB`,
`MAX_EVENT_SCRIPT_BYTES = 256 KiB`. Frame/command/text/event/step counts are
bounded as listed above, checked incrementally during decode. All bound
comparisons use safe integer types; no arithmetic on untrusted sizes is
unchecked.

## Atomic `--out`

Writes to a hidden temporary sibling file in the destination's own
directory, `File::sync_all()`s it, then `fs::rename`s it over the
destination (`std::fs::rename` replaces an existing destination atomically
on both Windows, via `MoveFileExW`, and POSIX). On any failure the temporary
file is removed and the pre-existing destination (if any) is left untouched;
the failure is reported as `OutputWriteFailed`, distinct from an inspection
failure.

## Authority and non-claims

- Preview/snapshot output is inspection evidence only: **not** live runtime
  authority, and it does not authorize any action or committed effect. Every
  successful report states this explicitly.
- Snapshot mode never compiles or runs source, never activates a bundle.
- Source mode never opens a native window or constructs a GPU/windowing
  backend; only `InMemoryBackend` is used.
- This command does not widen Gate D, does not widen the fixed
  `BUTTON_NODE`-only admission grant set, and does not add a general
  Projection Source interpreter or a second parser/compiler/verifier/Shell
  Player/admission model -- it drives the existing ones.
- `activate_projection_bundle_v0_gate_d` fuses bundle verification and Gate D
  activation into one fail-closed decision (`GateDActivationError` carries no
  stage detail today); this command's `ActivationDenied` status reflects that
  existing fusion rather than inventing a false verify/activate distinction.
  `VerifyFailed` is not currently producible through this pipeline entry
  point for that reason.
- Replay and staleness rejection
  (`ReferenceContourAdmission`'s revision/epoch/sequence guards) are real
  and proven by direct Rust qualification tests reusing
  `reference_contour::ReferenceState` (mirroring `prom-ui-demo`'s own test
  style) -- they are **not** exposed as an injectable Event Script v0 event
  kind, because the reference contour has no `InputEvent`-driven path that
  submits a stale or replayed action. Inventing one to match illustrative
  text would be exactly the kind of unsupported syntax this feature must
  avoid.
- `close`/Escape events are delivered like any other event in headless
  capture but do not terminate script processing early -- there is no window
  to close, and a script's own finite step list is the sole, deterministic
  bound on captured-frame count.
- No Workbench, Semantic Studio, ALM, or Hub dependency of any kind.

## Fixtures

`tests/fixtures/ui_frame_inspection/`: `valid_source.txt` /
`invalid_source.txt` (Grammar v0 text, valid and deliberately structurally
invalid), `events_admit_then_deny.json` (the canonical interaction
qualification script: hover+click the admitted node, then hover+click a
denied node -- proves hit-test, admitted action, visible `DrawFrame` change,
denied action, and preserved state all in one deterministic run),
`events_tab_and_activate.json` (keyboard focus routing), `events_invalid_kind.json`
/ `events_malformed_numeric.json` (invalid event scripts), `snapshot_valid.json`,
and `snapshot_malformed_json.json` / `snapshot_unsupported_version.json` /
`snapshot_unknown_command.json` / `snapshot_truncated.json` /
`snapshot_trailing_garbage.json` (malformed snapshots). An oversized-snapshot
case is exercised by generating an over-limit document in the test itself
(`ui_frame_snapshot::tests::rejects_oversized_input`) rather than committing
a multi-megabyte fixture file to the repository.

Goldens: `tests/golden_snapshots/ui_frame_inspection/` -- initial-frame text
and draw-json output, a post-admit-frame text output (frame 2 of the
admit-then-deny script), a denial-result text output (frame 4 of the same
script), a frame-not-found diagnostic, and an invalid-snapshot diagnostic.

## Compatibility policy

Both new formats are frozen at `schema_version: 0`. Unknown versions are
rejected, never silently upgraded, downgraded, or guessed. Implementing v0
does not claim long-term stable release status; a canonical format change
requires a new explicit version and an accompanying policy update to this
document.

## Examples

```bash
# Inspect an existing snapshot
smc look ui frame --from evidence/frame.json

# Same snapshot, canonical JSON, written to a file
smc look ui frame --from evidence/frame.json --format draw-json --out evidence/frame.reencoded.json

# Compile and capture the initial frame of a Projection Source document
smc look ui frame ui/reference.projsrc

# Drive a deterministic interaction and inspect a later captured frame
smc look ui frame ui/reference.projsrc --events ui/reference.events.json --frame 2
```
