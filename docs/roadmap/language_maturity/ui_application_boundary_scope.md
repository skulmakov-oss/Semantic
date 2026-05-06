# UI Application Boundary Scope

Status: completed M7 first-wave post-stable track
Related backlog item: `UI application boundary for Semantic desktop applications`

## Goal

Introduce a narrow UI/application boundary that lets a Semantic program own a
desktop window, process input events, and emit a minimal frame of drawing
commands through an explicit host/runtime contract.

This is a post-stable expansion track. It does not reinterpret the published
`v1.1.1` line as if desktop UI support already shipped there.

## Stable Baseline Before This Track

The current stable line already freezes these facts:

- published `v1.1.1` is a CLI-first toolchain and runtime baseline
- current compiler, verifier, VM, and PROMETHEUS boundary do not admit a
  dedicated UI/window host family
- no crate in the current baseline owns a desktop event loop or frame-oriented
  drawing surface
- no graphics backend library is part of the language-level contract

That stable reading remains the source of truth until this track explicitly
lands a widened post-stable contract on `main`.

## Included In This Track

- explicit ownership of a desktop UI boundary and its narrow runtime surface
- single-window lifecycle ownership for create, run, and close
- deterministic input-event polling and frame/tick ownership
- a minimal drawing command family sufficient for a canonical demo program
- capability/admission wiring for the UI family through the existing boundary
- docs/spec/tests/demo coverage for the widened post-stable contract

## Explicit Non-Goals

- forking `wgpu`
- designing a general widget toolkit or retained UI framework
- browser, mobile, or multi-window targets
- shader, resource-binding, or GPU-pipeline surface design
- CSS/layout/theme systems, accessibility framework, or asset pipeline design
- silently widening `v1.1.1`

## Planned Architecture Reading

The first-wave owner split is expected to stay narrow and explicit:

- `prom-ui`: UI boundary types, capability surface, and admitted operation IDs
- `prom-ui-runtime`: desktop session ownership, input polling, frame lifecycle,
  and backend adapter implementation
- `examples/` or `apps/`: one canonical UI demo application, kept as a
  consumer rather than an owner of the runtime boundary

No backend library becomes a language-level promise in this first wave.
Backend selection stays an internal implementation detail of the UI runtime
owner.

## Milestone Reading

Completed milestone: `M7 UI Application Boundary`

This milestone is complete:

- [x] a Semantic program can open a single desktop window through the admitted
  UI boundary (`DesktopSession<B>` + `UiBackendAdapter`)
- [x] deterministic input polling and frame lifecycle behavior are explicit and
  tested (`EventBuffer`, `FrameToken`, `FrameTokenIssuer`, 10 unit tests)
- [x] a minimal draw-command family is owned by the runtime boundary and
  exercised by a canonical demo (`DrawCommand`, `DrawFrame`, `prom-ui-demo`)
- [x] release-facing docs keep the widened `main` behavior distinct from
  published `v1.1.1`

## PR Waves

### Wave 0 - Governance and Owner Split

- PR 1: scope checkpoint, backlog/blueprint/milestone/WBS sync
- PR 2: owner-layer crate scaffolding and inert UI boundary types

### Wave 1 - Boundary Admission

- PR 3: UI capability taxonomy and operation identity ownership
- PR 4: verifier/VM/runtime denial-path ownership when UI capability is absent

### Wave 2 - Desktop Lifecycle

- PR 5: single-window session ownership and lifecycle API
- PR 6: deterministic event polling and frame-token ownership

### Wave 3 - Minimal Drawing Surface

- PR 7: minimal draw-command family such as clear/rect/text
- PR 8: backend adapter plus one canonical demo application

### Wave 4 - Freeze and Close-Out

- PR 9: docs/spec/tests/golden freeze for the widened contract

One PR still equals one logical step. Waves describe delivery grouping, not a
license to batch unrelated work.

## Acceptance Reading

This track is done only when:

- the admitted UI surface is explicit, inspectable, and capability-gated
- desktop lifecycle, event polling, and frame behavior agree across docs,
  runtime, and tests
- the first-wave draw surface stays narrow and sufficient for one canonical
  demo application
- backend choice remains an internal runtime detail rather than a language
  promise
- release-facing docs distinguish widened `main` from published `v1.1.1`

## Non-Commitments After Close-Out

Even after this track lands, the repository still does not claim:

- a general widget/layout framework
- multi-window, browser, or mobile UI support
- a forked graphics stack
- shader-language ownership
- a promise that UI support is already part of the published `v1.1.1` line

## Semantic UI DNA

The next POST-UI architecture layer is governed by:

- `docs/dna/SEMANTIC_UI_DNA.md`

This keeps the completed M7 first-wave UI boundary distinct from later Semantic UI architecture-selection work.
