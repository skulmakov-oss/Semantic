# R12 UI Model Seed Plan

Status: Draft
Track: R12 / POST-UI / Semantic UI Model
Scope type: planning / model seed plan
Implementation status: not authorized by this document

## 1. Purpose

This document defines the plan for a future minimal inert Semantic UI model seed.

It does not implement the seed.

It does not authorize UI implementation.

It does not authorize Workbench implementation.

It does not authorize Semantic Studio implementation.

It does not authorize renderer/backend dependency admission.

It does not claim readiness, stability, release readiness, or production readiness.

## 2. Prerequisite Docs Gates

The following docs gates are already in place:

- R12 UI roadmap
- R12 POST-UI milestone map
- R12 Workbench / Semantic Studio pause guard
- R12 Studio-00 anchor map
- R12 Workbench separation check
- third-party dependency register
- Semantic UI DNA

These gates make model seed planning possible.

They do not authorize implementation by themselves.

The future code PR still requires explicit owner approval.

## 3. Current prom-ui Posture

Current `crates/prom-ui` posture from inspection:

- current crate role: boundary / capability / admission scaffolding for Semantic UI
- current boundary posture: action admission, effect request, capability admission, dispatch trace, commit boundary, runtime capability mapping, and related inert descriptors
- current dependency posture: internal workspace dependency on `prom-abi` only
- `UiTree` / `UiAst` / `UiIr` already exist: no
- `Workbench` coupling exists: no direct crate coupling observed in `prom-ui`
- renderer/backend dependency exists: no direct external renderer/backend dependency in `prom-ui`

The crate currently exposes contract-heavy, model-light symbols such as capability kinds, operation IDs, action/effect descriptors, and trace/denial helpers.

It does not yet contain a dedicated inert UI model layer for `UiTree`, `UiAst`, or `UiIr`.

If any of the above is later changed by a code PR, this plan must be updated.

## 4. Target Seed Definition

The future seed is a minimal inert `prom-ui`-local type layer that gives Semantic UI names and structural handles for:

- UI Tree
- UI AST
- UI IR

Inert means data/type definitions only.

The future seed must not include:

- parser integration
- lowering integration
- verifier integration
- VM/runtime integration
- renderer integration
- event loop
- layout engine
- widget framework
- Workbench coupling
- Semantic Studio coupling
- external UI dependency

## 5. Candidate Future Types

| Candidate type | Layer | Purpose | Allowed in first code seed | Explicit non-goals |
| --- | --- | --- | --- | --- |
| `UiNodeId` | shared model identity | stable local identifier for UI nodes | yes, if needed for model scaffolding | no global identity service, no runtime handle |
| `UiTreeId` | UI Tree | stable local identifier for tree instances | yes, if needed for model scaffolding | no renderer state, no session ownership |
| `UiNodeKind` | UI Tree | closed initial node-kind enum | yes, if model needs a minimal kind vocabulary | no dynamic plugin taxonomy, no widget framework taxonomy |
| `UiNode` | UI Tree | inert node record with id, kind, and optional parent/children handles | yes, if tree records are introduced | no semantic authority, no execution semantics |
| `UiTree` | UI Tree | inert tree container | yes, if the seed uses a container type | no traversal engine, no renderer state |
| `UiAstNode` | UI AST | inert AST node candidate | yes, if an AST layer is introduced | no parser integration, no lowering semantics |
| `UiAst` | UI AST | inert AST container candidate | yes, if an AST layer is introduced | no parser front-end, no syntax authority |
| `UiIrNode` | UI IR | inert IR node candidate | yes, if an IR layer is introduced | no lowering backend, no verifier/runtime coupling |
| `UiIr` | UI IR | inert IR container candidate | yes, if an IR layer is introduced | no execution semantics, no VM/runtime ownership |

The names above are planning candidates, not final public API commitments.

The future code PR may narrow the set if needed.

No public API widening beyond the explicit code PR scope is authorized here.

## 6. First Code Seed Scope Proposal

Allowed later:

- add one or more new `prom-ui` modules for the model seed
- add inert local IDs and types
- add simple constructors if needed
- add local invariants
- add unit tests for pure data invariants
- export only minimal symbols if required
- avoid external dependencies

Forbidden later:

- no Workbench changes
- no Semantic Studio changes
- no renderer/backend changes
- no Tauri, React, `winit`, Slint, Floem, Makepad, or Zed dependency admission
- no parser, lowering, verifier, VM, or runtime integration
- no event loop
- no layout algorithm
- no widget system
- no command runner
- no file I/O
- no release or readiness authority
- no GitHub CI authority
- no Local Admission Guard replacement

## 7. Suggested File Layout For Future Code PR

Planning only.

Suggested possible future files:

- `crates/prom-ui/src/model.rs`
- `crates/prom-ui/src/model/tree.rs`
- `crates/prom-ui/src/model/ast.rs`
- `crates/prom-ui/src/model/ir.rs`

Alternative if smaller:

- `crates/prom-ui/src/model.rs` only

The exact layout is not authorized here.

The final layout must be chosen in the future code PR.

This document only defines constraints.

## 8. Invariant Plan

Possible invariants for later tests:

- IDs are transparent / newtype only if compatible with current style
- parent / child relationships are inert handles only
- no graph traversal semantics unless explicitly scoped
- no renderer state
- no runtime state
- no host capability state
- no Workbench state
- no Studio state
- no hidden global state
- no allocation-heavy behavior unless already consistent with crate policy
- no panic-prone constructors for normal data creation

Exact invariants must be specified in the future code PR.

## 9. Test Plan For Future Code PR

Allowed future tests:

- unit tests inside `prom-ui`
- pure data construction tests
- ID equality / ordering / debug stability if relevant
- tree container empty / default behavior if implemented
- AST / IR container empty / default behavior if implemented
- compile-only public export tests if already consistent with project style

Forbidden:

- no Workbench tests
- no renderer tests
- no runtime integration tests
- no VM / verifier tests
- no npm tests
- no FullPreflight unless explicitly scoped later
- no release artifact tests

## 10. Dependency / Legal Posture

The future model seed must add no dependencies.

No third-party UI toolkit is admitted by this plan.

The dependency register remains authoritative for actual manifest dependencies.

License verification remains pending where marked.

Influence register remains separate from dependency register.

No derivative / fork code is authorized.

## 11. Workbench / Studio Separation

The future model seed must not touch Workbench.

The future model seed must not touch Semantic Studio.

Workbench remains presentation / orchestration / tooling only.

Studio remains future planning anchor only.

`#675` remains active.

`#595` does not override `#675`.

## 12. Entry Criteria For Future R12-UI-MODEL-SEED Code PR

Before a code PR for `R12-UI-MODEL-SEED`:

- this model seed plan merged
- all previous docs gates merged
- explicit owner approval to begin code
- `PRReady` gate
- one small code PR only
- changed files restricted to `crates/prom-ui` unless explicitly approved
- no dependencies
- no Workbench / Studio changes
- no compiler / verifier / VM / runtime changes
- no release artifacts
- no production / stable / readiness claim

## 13. Exit Criteria For Future R12-UI-MODEL-SEED Code PR

The future code PR can be considered complete only if:

- changed files are bounded
- model types are inert
- tests pass for the touched crate / scope if run
- no forbidden integration appears
- no dependency changes occur
- no Workbench / Studio files are touched
- no release widening appears
- docs remain consistent with this plan

## 14. Non-Goals

- no UI implementation
- no Workbench implementation
- no Semantic Studio implementation
- no renderer/backend dependency admission
- no browser/WebView ownership
- no widget framework
- no layout engine
- no event loop
- no parser / lowering / verifier / VM / runtime integration
- no command runner
- no release widening
- no stable / production-ready / public-release-ready claim
- no final API commitment
- no dependency addition
- no final legal clearance claim
- no closure or weakening of `#675`

## 15. Final Decision

Final decision:

READY — REQUEST EXPLICIT OWNER APPROVAL BEFORE R12-UI-MODEL-SEED CODE
