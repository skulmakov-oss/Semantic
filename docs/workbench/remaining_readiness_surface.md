# Workbench Remaining Readiness Surface

Status: proposed v1 / readiness pack

## Purpose

This document closes the remaining Workbench readiness planning surface after
foundation classification, cockpit input map, and execution/feedback surface.

It defines the remaining v1 surfaces as docs-only readiness boundaries: spec
navigator, editor shell, formatter integration, release console, settings and
workspace, and first-pass closeout checklist.

## 1. Scope

This pack documents remaining Workbench readiness surfaces that are not already
closed by:

- `docs/status/feature_maturity_matrix.md`
- `docs/workbench/cockpit_input_map.md`
- `docs/workbench/execution_feedback_surface.md`

The pack does not implement Workbench behavior. It defines source-of-truth
requirements, allowed presentation behavior, forbidden ownership, and closeout
criteria for the remaining v1 surfaces.

## 2. Source-of-truth rule

Workbench remaining surfaces must present repository truth from public sources.
They must not create a second Semantic contract, a second readiness matrix, a
second release authority, or private UI-only semantics.

Allowed source surfaces:

- `README.md`
- `docs/spec/*`
- `docs/roadmap/*`
- `docs/status/*`
- `docs/workbench/*`
- `docs/examples_index.md`
- `examples/canonical/README.md`
- public `smc`, `svm`, `cargo`, and release-script outputs
- later explicit public Rust facades, if separately introduced

## 3. Remaining surface matrix

| Surface | Purpose | Authoritative input | Allowed behavior | Forbidden behavior | Status |
|---|---|---|---|---|---|
| Spec navigator | Navigate canonical specification documents. | `docs/spec/*` and linked status/roadmap docs. | Tree navigation, path/title search, section links, copy path/anchor. | Edit canonical docs through navigator or generate alternate specs. | proposed v1 |
| Roadmap/status navigator | Navigate roadmap and status posture. | `docs/roadmap/*`, `docs/status/*`, public status model. | Show maturity labels, known-limit links, source paths. | Maintain separate readiness or compatibility matrix. | proposed v1 |
| Workbench docs navigator | Navigate Workbench architecture docs. | `docs/workbench/*`. | Link scope, architecture, view-model, beta, cockpit, and feedback docs. | Treat docs navigation as implementation readiness. | proposed v1 |
| Examples navigator | Route users to proof examples. | `docs/examples_index.md` and `examples/canonical/README.md`. | Show paths, labels, and command links where documented. | Claim examples prove full language or release completeness. | proposed v1 |
| Editor shell | Provide bounded authoring shell. | Files in the selected workspace and public `smc` command outputs. | File tree, open file, tabs, save/reload, dirty markers. | Own parser/typechecker/compiler truth or silently rewrite source. | proposed v1 / current beta evidence |
| File tree | Show workspace files. | Filesystem state for selected workspace. | Browse, open, mark dirty/changed, show path. | Hide dirty repository state or rewrite project model semantics. | proposed v1 |
| Tabs / dirty markers | Track local editing state. | Open buffers and filesystem timestamps. | Show unsaved/changed/stale labels. | Treat UI buffer state as repository truth after failed save. | proposed v1 |
| Current-file check action | Run public check command for current file or project. | Explicit user action and public `smc check` output. | Launch job, show output through execution/feedback rules. | Implement parser/typechecker in UI or convert failures to pass. | proposed v1 |
| Current-file compile action | Run public compile command for current file or project. | Explicit user action and public `smc compile` output. | Launch job, show output/artifact references when emitted. | Silently modify SemCode artifacts or bypass verifier-facing rules. | proposed v1 |
| Formatter integration | Provide formatter entrypoint if public surface exists. | Canonical public formatter command, if present. | Invoke explicitly, show preview/diff/output, preserve failures. | Invent formatter semantics or format through private crate internals. | deferred until public formatter surface exists |
| Release console | Show release/readiness visibility. | public status docs, stable release policy, release checklist, local gate outputs. | Show gate status, bundle/asset smoke visibility, policy links. | Publish, qualify, or compute release truth independently. | proposed v1 |
| Bundle verification visibility | Display bundle verification output when available. | `scripts/verify_release_bundle.ps1` output for explicit candidate bundle. | Show pass/fail/stale/unknown and source output link. | Produce bundles or treat visibility as publication. | proposed v1 |
| Asset smoke visibility | Display asset smoke output when available. | `scripts/verify_release_assets.ps1` output for explicit candidate artifacts. | Show pass/fail/stale/unknown and source output link. | Produce artifacts, create releases, or run smoke automatically. | proposed v1 |
| Release policy links | Keep non-claims visible. | `docs/roadmap/stable_release_policy.md`, `docs/roadmap/release_bundle_checklist.md`, `docs/roadmap/public_status_model.md`. | Display links and status vocabulary. | Claim published stable or production-ready status. | proposed v1 |
| Settings / workspace | Manage local UI preferences and workspace selection. | User-selected local settings and workspace paths. | Store workspace path, recent projects, shell preference, theme, cache location. | Change repository truth, release policy, verifier/runtime behavior, or hide failures. | proposed v1 |
| Recent projects | Show recent local workspaces. | Local UI history. | Open recent path, mark missing/stale paths. | Treat recent path as trusted project without validation. | proposed v1 |
| Command configuration | Configure safe local command presentation. | User settings and documented public command surfaces. | Preferred shell, safe timeout display/config, default working directory. | Add hidden gates or run privileged commands silently. | proposed v1 |
| Local cache policy | Cache presentation data only. | Captured UI presentation state and source revision metadata. | Cache labels/output summaries and mark stale when inputs change. | Make cache source of truth or hide stale/failing evidence. | proposed v1 |
| Workbench first-pass closeout checklist | Track readiness closeout evidence. | Merged Workbench docs and status matrix. | Checklist of first-pass boundary criteria. | Treat checklist completion as stable/product release. | proposed v1 |

## 4. Spec navigator

The spec navigator is read-only navigation over public repository docs.

Allowed sources:

- `README.md`
- `docs/spec/*`
- `docs/roadmap/*`
- `docs/status/*`
- `docs/workbench/*`
- `docs/examples_index.md`
- `examples/canonical/README.md`

Allowed behavior:

- tree navigation
- title/path search
- section links
- maturity labels
- known-limit links
- copy path / copy anchor

Forbidden:

- edit canonical docs through navigator
- generate alternate specs
- maintain separate readiness matrix
- maintain separate compatibility matrix
- hide repository docs behind optimistic UI wording

## 5. Editor shell

The editor shell is an authoring shell only. Syntax highlighting may be used as
presentation, but it must not become source of truth for grammar or type
validity.

Allowed behavior:

- file tree
- open file
- multi-tab shell
- save/reload
- dirty markers
- current-file check action through public `smc` command
- current-file compile action through public `smc` command
- show output through `docs/workbench/execution_feedback_surface.md` rules

Forbidden:

- own parser semantics
- own typechecker semantics
- own compiler diagnostics
- silently rewrite source
- silently format on save unless explicitly configured
- claim unsupported syntax is valid
- couple to private crate internals

## 6. Formatter integration

Formatter integration is a public formatter surface only.

Allowed:

- invoke canonical formatter command if one exists as public surface
- show diff / preview if output is available
- require explicit user action or explicit setting
- preserve failure output

Forbidden:

- invent formatter semantics
- silently change semantic meaning
- format through private crate internals
- hide formatter failures
- treat formatter success as verifier success

Current stance:

Formatter integration is deferred until a public formatter surface exists. This
document does not implement formatter behavior or imply a formatter is already
available.

## 7. Release console

The release console is visibility only, not authority.

Allowed:

- show PRReady status if available
- show Readiness status if available
- show FullPreflight status only if explicitly run
- show release bundle verification output if available
- show asset smoke output if available
- link `docs/roadmap/stable_release_policy.md`
- link `docs/roadmap/release_bundle_checklist.md`
- link `docs/roadmap/public_status_model.md`
- show final release decision as human-controlled

Forbidden:

- run FullPreflight casually
- produce release artifacts
- create release tags
- create GitHub releases
- publish stable status
- make final release decision automatically
- treat GitHub CI as authoritative

## 8. Settings and workspace

Settings and workspace surfaces are local UI settings only.

Allowed:

- workspace path
- recent projects
- preferred shell
- command timeout display/config where safe
- theme/UI preferences
- local cache location
- clear local cache

Forbidden:

- change repository truth
- change release policy
- change verifier/runtime behavior
- store private secrets unless explicitly scoped later
- hide dirty repository state
- hide failed command evidence

Cache policy:

- cache is presentation cache only
- cache must be invalidated or marked stale when branch/commit/worktree changes
- cache must not become source of truth

## 9. Workbench first-pass closeout checklist

- [ ] foundation classification is merged
- [ ] cockpit input map is merged
- [ ] execution/feedback surface is merged
- [ ] remaining readiness surface is merged
- [ ] spec navigator stays read-only
- [ ] editor shell stays authoring shell only
- [ ] formatter integration is public-surface only or deferred
- [ ] release console stays visibility-only
- [ ] settings/workspace do not alter repository truth
- [ ] no `apps/workbench` implementation changes are required for first-pass docs closeout
- [ ] no parser/typechecker/verifier/VM/runtime ownership moves into Workbench
- [ ] no private crate coupling is introduced
- [ ] no stable/production/release-ready claim is made

## 10. Forbidden ownership

Workbench remaining surfaces must not own:

- parser semantics
- typechecker semantics
- compiler semantics
- verifier semantics
- SemCode admission truth
- VM execution semantics
- runtime semantics
- trap taxonomy
- quota semantics
- capability semantics
- release qualification truth
- private crate internals
- private PROMETHEUS state
- stable release decision

## 11. Non-claims

Workbench readiness first pass does not mean Workbench is stable.

Workbench readiness first pass does not mean production-ready.

Workbench readiness first pass does not mean public release.

Workbench does not replace terminal workflows.

Workbench does not replace local Admission Guard.

Workbench does not replace repository docs.

Workbench does not widen Semantic language/runtime behavior.

Workbench does not imply stable runtime ABI or binary ISA.

Workbench does not make GitHub CI authoritative.

## 12. Acceptance criteria

The remaining readiness surface is acceptable when:

- every remaining surface has a public source of truth
- every remaining surface has forbidden behavior
- spec navigator is read-only
- editor shell does not own parser/typechecker/compiler truth
- formatter integration is public-surface only or deferred
- release console is visibility-only
- settings/workspace cannot alter repository truth
- no `apps/workbench` implementation is changed
- no release claim is made
