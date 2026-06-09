# Workbench Cockpit Input Map

Status: proposed v1 / readiness boundary

## Purpose

The overview cockpit presents repository and local command evidence. It does not
compute independent Semantic readiness, release truth, or compatibility truth.

## 1. Scope

The cockpit is a presentation surface for operator visibility. It may aggregate
existing evidence from repository documents, public command outputs, public
scripts, and captured job results.

The cockpit is not a compiler, verifier, VM, runtime, release gate, or policy
authority. It must keep every displayed status explainable by a public input.

## 2. Source-of-truth rule

Cockpit may aggregate and display existing evidence. Cockpit must not create new
project truth.

Allowed source surfaces:

- git state
- local command outputs
- Admission Guard outputs
- release verification script outputs
- `docs/status/feature_maturity_matrix.md`
- `docs/roadmap/public_status_model.md`
- `docs/roadmap/stable_release_policy.md`
- `docs/roadmap/release_bundle_checklist.md`
- `docs/examples_index.md`
- `docs/spec/diagnostics.md`

Cockpit may show:

- last-known status
- staleness marker
- link to source document
- link to command output
- exit code
- summary label derived from command result

Cockpit must not:

- invent readiness score
- override repository status
- promote current-main to stable
- convert failed gates into passed gates
- hide known limitations
- invent release readiness
- treat GitHub CI as authoritative

## 3. Cockpit panel matrix

| Panel | Purpose | Authoritative input | Allowed derivation | Forbidden behavior | Status |
|---|---|---|---|---|---|
| Repository state | Show local repository condition. | git state and working tree status. | Clean/dirty label, detached/branch label, stale marker. | Hide dirty state or infer project readiness from git cleanliness. | proposed v1 |
| Current branch / commit | Anchor displayed evidence to a concrete revision. | git branch and commit. | Short commit label, copyable revision link/text. | Treat branch name as release status. | proposed v1 |
| Baseline / status labels | Show documented maturity posture. | `docs/status/feature_maturity_matrix.md` and `docs/roadmap/public_status_model.md`. | Display labels and source links. | Promote current-main or qualified evidence to published stable. | proposed v1 |
| Latest PRReady result | Show last PR admission evidence when available. | `pwsh scripts/admission_guard.ps1 -PRReady` output captured as a job. | Passed/failed/stale/unknown label from exit code and timestamp. | Treat PRReady as release qualification. | proposed v1 |
| Latest Readiness result | Show stronger readiness evidence when available. | `pwsh scripts/admission_guard.ps1 -Readiness` output captured as a job. | Passed/failed/stale/unknown label from exit code and timestamp. | Treat Readiness as publication or release-ready status. | proposed v1 |
| FullPreflight visibility | Show whether explicit heavy-gate evidence exists. | Explicitly scoped FullPreflight job output, if ever run. | Not-run/stale/passed/failed label from captured output. | Run FullPreflight casually or imply it is ordinary docs validation. | proposed v1 |
| Recent command/job status | Summarize local operations. | Workbench job records for `smc`, `svm`, `cargo`, and public scripts. | Grouping, sorting, elapsed time, exit-code label. | Rewrite command truth or treat failed jobs as passed. | proposed v1 |
| Canonical examples status link | Point to practical proof surface. | `docs/examples_index.md` and canonical example command jobs. | Link, last-known command status, stale marker. | Claim example coverage proves full language completeness. | proposed v1 |
| Readiness/status summary | Present documented readiness contour. | status and roadmap docs. | Compact labels with source links. | Create independent readiness scoring. | proposed v1 |
| Known limits | Keep visible limits in the operator path. | status docs, public status model, v1 readiness docs, and Workbench scope. | Grouping and links to source sections. | Hide active limitations or convert out-of-scope items into supported features. | proposed v1 |
| Diagnostics summary | Show emitted diagnostic evidence. | command output and `docs/spec/diagnostics.md`. | Count by emitted severity, group by source command, stale marker. | Invent diagnostic semantics or rewrite compiler/verifier errors. | proposed v1 |
| Release bundle verification visibility | Show bundle verification evidence if available. | `scripts/verify_release_bundle.ps1` output captured against an explicit candidate bundle. | Passed/failed/stale/unknown label and output link. | Produce bundles or treat bundle visibility as publication. | proposed v1 |
| Asset smoke verification visibility | Show release asset smoke evidence if available. | `scripts/verify_release_assets.ps1` output captured against explicit candidate artifacts. | Passed/failed/stale/unknown label and output link. | Produce artifacts, publish assets, or run smoke automatically. | proposed v1 |
| Release policy / non-claims | Keep release posture explicit. | `docs/roadmap/stable_release_policy.md`, `docs/roadmap/release_bundle_checklist.md`, and `docs/roadmap/public_status_model.md`. | Display policy links and non-claim labels. | Make final release decisions or claim published stable. | proposed v1 |

## 4. Command and job inputs

Allowed job states:

- `queued`
- `running`
- `passed`
- `failed`
- `cancelled`
- `stale`
- `unknown`

Jobs may store and display:

- command line
- working directory
- start and end time
- exit code
- stdout/stderr summary
- artifact path reference if already produced externally
- source surface: `smc`, `svm`, `cargo`, or public scripts

Jobs must not:

- rewrite verifier results
- rewrite compiler diagnostics
- modify SemCode artifacts silently
- treat failure as pass
- run FullPreflight without explicit user scope

## 5. Release and asset-smoke inputs

Cockpit may display release visibility only.

Allowed:

- PRReady status if available
- Readiness status if available
- FullPreflight status if explicitly run
- release bundle verification output if available
- asset smoke output if available
- links to `docs/roadmap/stable_release_policy.md`
- links to `docs/roadmap/release_bundle_checklist.md`
- links to `docs/roadmap/public_status_model.md`

Forbidden:

- produce release artifacts
- create release tags
- create GitHub releases
- publish stable status
- make final release decision
- run release gates automatically

## 6. Diagnostics and known-limits inputs

Diagnostics summary may derive from emitted command output only.

Allowed:

- count by severity if emitted
- group by source command
- link to `docs/spec/diagnostics.md`
- link to relevant file path when present
- show stale diagnostics marker

Forbidden:

- invent diagnostic semantics
- claim unsupported syntax is supported
- hide failing diagnostics
- rewrite compiler/verifier errors into different truth

Known limits panel may link to:

- `docs/status/feature_maturity_matrix.md`
- `docs/roadmap/public_status_model.md`
- `docs/roadmap/v1_readiness.md`
- `docs/workbench/scope.md`

## 7. View-model derivation rules

View models may derive:

- display labels
- grouping
- sorting
- staleness markers
- last-known status
- links to source documents

View models must not derive:

- new Semantic meaning
- new verifier result
- new runtime result
- new release truth
- new compatibility status
- new capability decision

Every cockpit view-model field should be traceable to a public source document,
public command output, public script output, or explicit job record.

## 8. Staleness and uncertainty

Cockpit must distinguish current evidence from last-known evidence.

Use `stale` when a displayed result is tied to an older branch, commit, command
run, release candidate, or artifact set. Use `unknown` when no reliable input is
available. Unknown and stale states must remain visible; they must not be
collapsed into pass or ready labels.

## 9. Forbidden ownership

Cockpit must not own:

- parser
- typechecker
- compiler semantics
- verifier semantics
- VM semantics
- runtime semantics
- ABI semantics
- capability semantics
- quota semantics
- release qualification truth
- private crate internals
- private PROMETHEUS state

## 10. Non-claims

Cockpit readiness does not mean Workbench is stable.

Cockpit readiness does not mean production-ready.

Cockpit readiness does not mean public release.

Cockpit does not replace terminal gates.

Cockpit does not replace local Admission Guard.

Cockpit does not widen Semantic language/runtime behavior.

Cockpit does not imply stable runtime ABI or binary ISA.

Cockpit does not make GitHub CI authoritative.

## 11. Acceptance criteria

The cockpit input map is acceptable when:

- every cockpit panel has an authoritative input
- every cockpit panel has forbidden behavior
- cockpit remains presentation-only
- command/job state is process-based
- release visibility is not release authority
- diagnostics are emitted-truth presentation
- no private crate coupling is introduced
- no UI implementation is changed
- no release claim is made
