# Semantic Workbench Native v0.2.0-beta1 Release Notes

## Status

Component-scoped native Workbench milestone. Not a stable Semantic
platform release and not a public Workbench API promise.

## Generation change

`semantic-workbench-v0.2.0-beta1` replaces the former TS/Tauri
Workbench direction (`semantic-workbench-v0.1.0-beta1`, March 2026)
with the native Semantic + Prom UI + private Iced substrate
architecture.

The old TS/Tauri implementation remains archived at
`apps/workbench_ts_tauri_legacy` as a non-authoritative reference and
is not the production Workbench path.

## What this release introduces

- native Semantic Workbench (`examples/workbench_semantic`), production
  entry point unconditionally on the Iced path -- no migration flag, no
  hidden legacy fallback;
- private `crates/prom-ui-iced-adapter` Iced 0.14 substrate, the only
  crate in the workspace authorized to depend on Iced directly,
  enforced by real dependency-boundary tests;
- Semantic-owned `PromNode` UI projection (a 26-role component
  contract);
- Cockpit, Jobs, Diagnostics, Editor, Explorer, Spec Navigator,
  Readiness, Settings;
- responsive native layout (header / nav rail / explorer / main /
  inspector / ledger);
- draggable split panes -- all three Workbench dividers, real
  `iced::widget::pane_grid` pointer capture, persistent ratios,
  non-zero-minimum clamping;
- UTF-8 text editing with selection, scrolling, and clipping;
- deterministic WGPU frame capture for qualification evidence;
- a real, live job-completion path: the production window now polls
  for async job completion via a generic, periodic `on_tick` hook
  (fixed post-review -- see Known limitations for context);
- qualification evidence: 64 adapter/Workbench tests, a 1621-test
  canonical UI regression suite, deterministic screenshots, and a
  drag-verification sweep.

## Authority boundary

```text
Semantic application state and verified actions
        ↓
Semantic UI projection
        ↓
public Prom UI contracts
        ↓
private prom-ui-iced-adapter
        ↓
Iced 0.14 / WGPU / native platform
```

Iced owns generic UI mechanics only. Semantic remains responsible for:

- domain state;
- command intent;
- action admission;
- verification;
- VM transitions;
- evidence;
- diagnostics;
- readiness;
- Jobs Ledger state.

### Verified action route

```text
Iced event
→ SemanticActionId
→ admission / verifier / VM
→ Semantic state transition
→ updated projection
→ updated native view
```

## Qualification

Run from a clean worktree of the exact merged `main` commit
(`b7f2327b0ce1037db6ee2c454b7e8a42b086f10d`):

- `cargo fmt --all --check` -- clean;
- `cargo test -p prom-ui-iced-adapter -p workbench_semantic` -- 64
  passed, 0 failed;
- `cargo clippy --workspace --all-targets -- -D warnings` (the exact
  CI `pr-ready` command) -- clean;
- `cargo check --workspace` -- clean;
- `scripts/harness-check.ps1` -- `[harness] ok`;
- canonical UI regression suite (`quad_logic_calculator`, `prom-ui`,
  `prom-ui-runtime`, `prom-ui-backend-native`) -- 1621 passed, 0
  failed;
- `cargo build -p workbench_semantic --release` -- built and smoke
  tested (real window opens, Cockpit renders, navigation works, a
  divider drags, process exits cleanly).

## Supported release asset

- Windows x86_64 native executable package.

Linux and macOS are not built, packaged, or claimed as supported by
this release.

## Known limitations

- autonomous Semantic Agent is not included;
- Semantic Studio productization is not included;
- the pre-Iced `DesktopSession`/`render_frame`/`hit_targets`/
  `WorkbenchLayout` renderer source remains in the repository,
  unreachable from the production entry point but not deleted --
  removal is intentionally a separate, future pass;
- `Overflow::Ellipsis`/`ScrollX` render as plain horizontal clipping
  (no literal "…" glyph, no live horizontal scroll offset yet);
- no compact-mode explorer/inspector collapse at narrow widths;
- split-pane ratios are session-only, not persisted across a restart;
- no stable public Workbench API is promised;
- this is an early component release, not a platform-wide stability
  claim.

## Source

- Pull request: [#1567](https://github.com/skulmakov-oss/Semantic/pull/1567)
- Commit: `b7f2327b0ce1037db6ee2c454b7e8a42b086f10d`
- Tag: `semantic-workbench-v0.2.0-beta1`
- Screenshot manifest: `artifacts/workbench/screenshots/manifest.json`
- Harness report:
  `.harness/reports/SEMANTIC-WORKBENCH-NATIVE-V0.md`
