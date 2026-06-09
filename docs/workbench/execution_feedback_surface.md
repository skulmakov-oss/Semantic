# Workbench Execution And Feedback Surface

Status: proposed v1 / readiness boundary

## Purpose

Workbench may orchestrate public commands and present their outputs. It must not
reinterpret command truth or absorb compiler, verifier, VM, or runtime ownership.

## 1. Scope

The execution and feedback surface covers command launch, job records, command
output display, diagnostics presentation, read-only inspector views, spec links,
and stale/unknown handling.

This surface is process-based over public repository tools and scripts. It does
not introduce private crate integrations, hidden execution paths, or UI-owned
semantic interpretation.

## 2. Source-of-truth rule

Workbench may request explicit commands and display their captured results.
Command output, exit code, repository documents, and public script output remain
the source of truth.

Allowed source surfaces:

- `smc`
- `svm`
- `cargo`
- public release scripts
- public docs/spec files
- public docs/roadmap files
- captured stdout/stderr
- captured exit codes
- explicit artifact references already produced outside this docs scope

Workbench must not create alternate truth for compiler, verifier, VM, runtime,
diagnostic, release, or compatibility outcomes.

## 3. Surface matrix

| Surface | Purpose | Authoritative input | Allowed derivation | Forbidden behavior | Status |
|---|---|---|---|---|---|
| Command runner | Launch explicit public-tool commands. | User-requested `smc`, `svm`, `cargo`, or public-script command line. | Job record, timestamp, exit-code label. | Run hidden commands, run FullPreflight casually, or rewrite command results. | proposed v1 |
| Jobs history | Preserve process evidence over time. | Captured command metadata, stdout/stderr, exit code, and repository revision. | Grouping, sorting, duration, stale/current labels. | Hide failed jobs or merge unrelated jobs into one truth. | proposed v1 / current beta evidence |
| Command output panel | Display raw and summarized output. | Captured stdout/stderr and exit code. | Summary snippets, anchors, severity grouping when emitted. | Redact failures into pass labels or replace emitted text with optimistic UI wording. | proposed v1 / current beta evidence |
| Diagnostics hub | Present emitted diagnostics. | Compiler, verifier, runtime, CLI, or script diagnostics emitted by public commands. | Group by tool, severity when emitted, file/line/column when present. | Invent diagnostic semantics or convert verifier rejection into runtime trap. | proposed v1 / current beta evidence |
| Error-code lookup | Route known codes to docs. | Emitted diagnostic codes and `docs/spec/diagnostics.md`. | Link labels and lookup grouping. | Create undocumented error categories as authoritative. | proposed v1 |
| Spec/document links | Route output to repository docs. | `docs/spec/*`, `docs/roadmap/*`, and status docs. | Source links and context labels. | Generate alternate specs or separate readiness/compatibility matrices. | proposed v1 |
| Disasm view | Present disassembly output. | Supported public `svm disasm` output. | Formatting, folding, search, line anchors. | Execute SemCode through private APIs or invent instruction meaning. | proposed v1 |
| Verify-result view | Present verification output. | `smc verify` or supported verifier command output. | Status label, sections, links to raw output. | Reinterpret verifier result or override admission failures. | proposed v1 |
| Trace summary view | Present already emitted trace evidence. | Public command trace output when available. | Grouping, filtering, source links. | Invent trace events or runtime behavior. | proposed v1 |
| Quota summary view | Present quota evidence when emitted. | Public command quota summary or runtime output. | Grouping, labels, stale marker. | Invent quota decisions or mask exhaustion. | proposed v1 |
| Capability summary view | Present capability evidence when emitted. | Public command capability summary or denial output. | Grouping, labels, links to docs. | Invent capability admission decisions or edit PROMETHEUS state. | proposed v1 |
| Artifact reference display | Display artifact paths and metadata. | Explicitly produced external artifact references. | Path labels, age, source job link. | Modify artifacts, silently produce artifacts, or treat references as release publication. | proposed v1 |

## 4. Command runner

Allowed command families:

- `smc check`
- `smc run`
- `smc compile`
- `smc verify`
- `smc run-smc`
- `svm disasm` and supported public `svm` commands
- `cargo` commands used by documented local workflow
- public release scripts

Command runner may:

- launch explicit user-requested commands
- record command line
- record working directory
- record start and end time
- capture exit code
- capture stdout/stderr
- attach output to a job record
- mark result `passed`, `failed`, `cancelled`, `stale`, or `unknown`

Command runner must not:

- run hidden commands without user intent
- run FullPreflight casually
- run release gates automatically
- alter SemCode artifacts silently
- rewrite command results
- convert failure to pass
- treat GitHub CI as authoritative

## 5. Jobs history

Allowed job states:

- `queued`
- `running`
- `passed`
- `failed`
- `cancelled`
- `stale`
- `unknown`

Jobs history may store:

- command line
- working directory
- tool family
- start and end time
- duration
- exit code
- stdout/stderr summary
- full output reference
- artifact path reference if already produced externally
- source branch/commit if known

Jobs history must distinguish:

- current result
- last-known result
- stale result
- unknown result
- cancelled result

Jobs history must not:

- treat stale result as current
- hide failed jobs
- invent missing outputs
- merge unrelated command results into one truth

## 6. Diagnostics hub

Diagnostics hub may present emitted diagnostics only.

Allowed:

- group by tool/source command
- group by severity when emitted
- show file/line/column when present
- show diagnostic code when present
- link to `docs/spec/diagnostics.md`
- link to related spec/roadmap sections
- show raw command output link
- show stale marker when diagnostic belongs to old run

Forbidden:

- invent diagnostic semantics
- rewrite compiler/verifier/runtime errors
- claim unsupported syntax is supported
- hide failing diagnostics
- silence boundary failures
- convert verifier rejection into runtime trap or vice versa

## 7. Inspector views

Inspector views are read-only presentations over existing outputs.

Allowed views:

- disasm view
- verify-result view
- trace summary view
- quota summary view
- capability summary view
- artifact metadata/reference view

Allowed inputs:

- `smc verify` output
- `smc run-smc` output
- `svm disasm` output
- runtime trace output if already emitted by public command
- quota/capability summaries if already emitted by public command
- release artifact references if already produced by explicit external scope

Forbidden:

- embed VM execution authority
- execute SemCode through private APIs
- reinterpret verifier result
- invent trace events
- invent quota/capability decisions
- modify artifacts

## 8. Spec links and documentation routing

Workbench may link to:

- `docs/spec/cli.md`
- `docs/spec/diagnostics.md`
- `docs/spec/verifier.md`
- `docs/spec/semcode.md`
- `docs/spec/vm.md`
- `docs/spec/quotas.md`
- `docs/roadmap/public_status_model.md`
- `docs/status/feature_maturity_matrix.md`

Workbench must not:

- generate alternate specs
- maintain separate compatibility matrix
- maintain separate readiness matrix
- hide repository docs behind optimistic UI wording

## 9. Output truth and staleness

Definitions:

- `current`: tied to the current branch/commit and current working tree assumptions.
- `last-known`: previously captured evidence whose exact inputs are still visible.
- `stale`: evidence captured before branch, commit, working tree, command config, or artifact set changed.
- `unknown`: no reliable evidence is available.
- `failed`: captured command or gate exited with failure.
- `cancelled`: command was intentionally stopped before completion.

Rules:

- A result is current only when tied to the current branch/commit and current working tree assumptions.
- A result is stale when branch, commit, working tree, command config, or artifact set changed after capture.
- Unknown must remain visible when no reliable evidence exists.
- Failed output must remain visible and must not be collapsed into unknown.
- Cancelled output must remain distinct from failed and unknown.

## 10. Forbidden ownership

Workbench execution/feedback surface must not own:

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

## 11. Non-claims

Execution/feedback readiness does not mean Workbench is stable.

Execution/feedback readiness does not mean production-ready.

Execution/feedback readiness does not mean public release.

Workbench command runner does not replace terminal workflows.

Workbench jobs history does not replace local Admission Guard.

Workbench diagnostics hub does not redefine diagnostics.

Workbench inspectors do not replace verifier/VM/runtime authority.

Workbench does not widen Semantic language/runtime behavior.

Workbench does not imply stable runtime ABI or binary ISA.

Workbench does not make GitHub CI authoritative.

## 12. Acceptance criteria

The execution/feedback surface is acceptable when:

- every command surface uses public tools or scripts
- every output has a traceable source
- jobs history preserves failures and stale states
- diagnostics remain emitted-truth presentation
- inspectors remain read-only output views
- no private crate coupling is introduced
- no UI implementation is changed
- no release claim is made
