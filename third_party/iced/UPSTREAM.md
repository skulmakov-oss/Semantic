# Iced — upstream record

Status: admitted third-party dependency (not influence-only), per explicit
repository-owner authorization (chat, 2026-08-02, "WORKBENCH ICED SUBSTRATE
MIGRATION"). See `.harness/current.task.yaml`'s
`owner_directive_2026_08_02` for the exact authorization text.

## Source form

**A — exact crates.io release, pinned.** No vendoring, no Git submodule, no
fork. This is the initial, and so far only, source form used; see FORK RULE
below for when that would change.

## Identity

- Upstream repository: https://github.com/iced-rs/iced
- Upstream crate: `iced` (plus its component crates: `iced_core`,
  `iced_widget`, `iced_renderer`, `iced_wgpu`, `iced_winit`, `iced_runtime`,
  pulled in transitively by the top-level `iced` crate, not depended on
  directly)
- Pinned version: `=0.14.0` (published 2025-12-07 on crates.io — verified
  live against the crates.io API, not assumed from training knowledge)
- License: MIT (verified against the upstream `LICENSE` file at the `0.14`
  tag)
- Internal backend versions this release pulls in: `iced_winit ^0.14.0`,
  `iced_renderer` backed by `wgpu 27.0.1`

## Why this version

`0.14.0` is upstream's newest published release at the time of this task
and is the version the owner directive names ("Iced 0.14-compatible
sources"). Its release notes are also the reason Iced is viable for the
FRAME CAPTURE requirement without depending on OS foreground-window focus:
0.14 explicitly adds "headless mode testing" and "end-to-end testing
capabilities" upstream, rather than requiring a bespoke offscreen-rendering
patch.

## Relationship to this repository's existing native rendering path

This repository already has a hand-rolled native rendering path
(`crates/prom-ui-backend-native`: raw `winit` 0.30.13 + raw `wgpu` 23.0.1 +
`glyphon` 0.7 for text). Iced 0.14 pulls in its *own* `iced_winit`/`wgpu 27`
internally — it owns its own window, event loop, and GPU device; it does
not plug into `NativeBackendWgpuContext`. Both wgpu major versions (23 and
27) and both winit usages coexist in the workspace dependency graph without
conflict (Cargo permits multiple major versions of the same crate; each
consumer gets its own copy). `examples/quad_logic_calculator` and any other
existing `prom-ui-backend-native` consumer are unaffected and continue to
use the pre-existing hand-rolled path unchanged — Iced is additive
infrastructure for the Workbench specifically, not a replacement of
`prom-ui-backend-native` itself.

## Where Iced may be depended on

Only `crates/prom-ui-iced-adapter` (and, transitively, anything it alone
exports) may depend on `iced` or name Iced types. `examples/workbench_semantic`
depends on the public, Iced-free Prom UI component contract
(`prom-ui-iced-adapter`'s `PromNode` types), never on `iced` directly. This
boundary is enforced by a dependency test (see
`crates/prom-ui-iced-adapter/tests/`).

## Features enabled

Default features of the `iced` crate as published (no feature
customization was needed for this task's scope). Recorded here so a future
change to the feature set is a visible diff against this line, not a
silent drift:

```
iced = "=0.14.0"
```

## Local modifications

None. Zero patches, zero vendored/forked source. Per the owner's FORK RULE,
a fork is only justified after a concrete, documented limitation is proven
unmet by the adapter layer or Iced's own advanced/custom-widget API — no
such limitation has been hit yet.

## Update procedure

1. Check crates.io for a newer `0.14.x` patch release or a new minor/major
   line.
2. Read its changelog for breaking changes to the `Application`/`Program`
   API, widget API, or headless/test API surface this adapter depends on.
3. Bump the pinned version in `crates/prom-ui-iced-adapter/Cargo.toml`
   (exact-version pin, not a caret range) in its own change, not bundled
   silently into an unrelated commit.
4. Re-run the full adapter test suite and the Workbench qualification
   sweep before considering the bump safe.
5. Update this file's "Pinned version" and "Why this version" sections.

## Attribution

Iced is Copyright its contributors, licensed under the MIT License. The
full license text is reproduced at
`third_party/iced/LICENSE-MIT` (verbatim copy of the upstream `0.14` tag's
`LICENSE` file, required for MIT attribution compliance since this
repository redistributes/links against it).
