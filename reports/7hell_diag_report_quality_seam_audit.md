# 7HELL Diagnostics Report-Quality Seam Audit

Status: audit
Scope: locate safe Diagnostics Hell report-quality seam
Mode: research / readiness investigation
Non-goal: implementation, execution behavior, release readiness, CI gate, CTF closure

## 1. Background

PR #743 decided that Diagnostics Hell / User Pain is a report-quality layer.

Diagnostics Hell should evaluate existing `7hell` reports, not produce new
execution facts. It should not parse source, compile, verify SemCode, run the
VM, invoke Practical qualification, or render host-visible output.

The audit goal is to locate where the future evaluator can read the completed
report structure safely.

This PR does not implement anything.

## 2. Current 7hell report pipeline

The current `smc 7hell` command lives in `src/bin/smc.rs`.

Observed entry and command flow:

- `main` dispatches `7hell` and `seven-hell` before delegating to `smc_cli::main_entry`.
- `run_7hell_command` parses args, calls `execute_7hell_single_file`, prints the rendered output, and maps the render outcome to an exit code.
- `parse_7hell_args` accepts only a single file plus optional `--json`; project flags are rejected.

Observed report model:

- `SevenHellStageStatus` has `Pass`, `Fail`, `Blocked`, and `NotImplemented`.
- `SevenHellResult` currently has `Incomplete` and `Fail`; there is no final `Pass` result.
- `SevenHellStageReport` stores stage index, name, key, status, summary, optional `blocked_by`, and diagnostic IDs.
- `SevenHellDiagnostic` stores ID, stage, diagnostic kind, code, category, message needle, severity, and source location.
- `SevenHellReport` stores target display/normalized paths, result, seven stage reports, diagnostics, boundary status, and boundary reason.

Observed assembly path:

- `execute_7hell_single_file` reads the target source, runs semantic check, compiles to SemCode, verifies SemCode, runs verified SemCode, and then calls `CliPipeline::qualify_controlled_observation_bytes`.
- Each failure point converts the failure into a completed `SevenHellReport` using one of:
  - `build_check_failed_7hell_report`
  - `build_lowering_failed_7hell_report`
  - `build_verifier_failed_7hell_report`
  - `build_vm_failed_7hell_report`
  - `build_practical_failed_7hell_report`
  - `build_practical_passed_7hell_report`
- `stage_report` centralizes individual stage entry construction.
- `failure_stage_summary` turns a structured diagnostic into the stage summary shape.
- `diagnostic_from_check_error`, `diagnostic_from_compile_error`, `diagnostic_from_verifier_error`, `diagnostic_from_vm_error`, and `diagnostic_from_practical_error` convert prior-stage failures into `SevenHellDiagnostic` entries.

Observed rendering path:

- `execute_7hell_single_file` builds a `SevenHellReport`, computes success from the report result, and then calls `render_7hell_report`.
- `render_7hell_report` dispatches to `render_human_7hell_report` or `render_json_7hell_report`.
- `render_human_7hell_report` renders the target, mode, profile, stages, diagnostics, result, and boundary.
- `render_json_7hell_report` renders schema `semantic.7hell.report.v0`, target, profile, result, stages, diagnostics, evidence, CTF, and boundaries.
- `render_json_diagnostics` serializes the diagnostics array.

Observed Practical boundary:

- `CliPipeline::qualify_controlled_observation_bytes` calls `qualify_controlled_observation_envelope`.
- `qualify_controlled_observation_envelope` returns `ControlledObservationQualificationEnvelope` with capability decision, audit results, and hashed observation summaries.
- `render_controlled_observation_envelope` is a separate rendering path that exposes `rendered_lines`; `7hell` does not call it.
- Current tests assert that Practical failure JSON does not contain `rendered_lines`, `raw_text`, `stdout`, or raw example text such as `Hello, World!`.

Observed snapshot coverage:

- `tests/7hell_e1_report_snapshots.rs` covers human and JSON report shape.
- JSON snapshots under `tests/fixtures/7hell_e1/snapshots/` show stable stage names, stage keys, statuses, summaries, diagnostic IDs, diagnostics, and boundary records.
- Failure snapshots cover syntax, type, verifier, VM trap, and Practical failure paths.

## 3. Candidate seams

| Candidate | Description | Files / functions involved | Pros | Risks | Verdict |
| --- | --- | --- | --- | --- | --- |
| A. Before JSON / human rendering | Evaluate the fully built internal `SevenHellReport` before `render_7hell_report` chooses human or JSON output. | `src/bin/smc.rs`: `execute_7hell_single_file`, `SevenHellReport`, `render_7hell_report` | Centralized; reads all prior stage results; drives human and JSON output consistently; adds no execution calls. | Future implementation must avoid mutating execution facts or treating report quality as release readiness. | Accepted as recommended seam. |
| B. During report construction after all prior stages | Evaluate while each `build_*_7hell_report` function constructs the final result. | `src/bin/smc.rs`: `build_*_7hell_report`, `stage_report`, `failure_stage_summary` | Has direct access to each failure cause during construction. | Mixes report-quality policy with execution-stage wiring; duplicates policy across constructors; harder to keep consistent. | Rejected for v0. |
| C. Inside each failure branch | Evaluate diagnostics at each failure point in `execute_7hell_single_file`. | `src/bin/smc.rs`: `execute_7hell_single_file`, `diagnostic_from_*` helpers | Can classify failures close to origin. | Scatters report-quality rules; risks inconsistent PASS/FAIL logic; may tempt extra execution or parsing calls. | Rejected. |
| D. After JSON rendering | Parse or inspect rendered JSON/human output after serialization. | `src/bin/smc.rs`: `render_json_7hell_report`, `render_human_7hell_report`; snapshots | Tests serialized shape directly. | Too late and brittle; duplicates serializer assumptions; human and JSON may diverge; encourages string parsing. | Rejected. |

## 4. Recommended seam

Recommended seam:
Evaluate Diagnostics Hell after the `SevenHellReport` object is fully assembled
and before JSON / human rendering.

The current code already has this seam in `execute_7hell_single_file`: the
internal report is assigned before success is computed and before
`render_7hell_report` is called.

Reason:

- it treats Diagnostics Hell as report-quality assessment;
- it avoids new execution behavior;
- it keeps report evaluation centralized;
- it can inspect all prior stage results;
- it does not require calling parser/check/compiler/verifier/VM/practical paths;
- it can later drive both JSON and human output consistently.

The current code structure supports this seam cleanly for a minimal internal
evaluator. A separate refactor-only seam preparation PR does not appear required
for the first implementation, as long as the evaluator remains a pure read over
`SevenHellReport`.

## 5. Minimal future implementation shape

Future implementation should remain high-level and internal to report quality.

Expected shape:

- introduce a report-quality evaluator function;
- input: completed `SevenHellReport`;
- output: Diagnostics Hell stage result or report-quality finding;
- no execution calls;
- no source parsing;
- no SemCode emission;
- no verifier call;
- no VM run;
- no Practical envelope invocation;
- no host effects.

Names that fit the current code:

- `evaluate_7hell_report_quality(...)`
- `classify_diagnostics_hell(...)`
- `DiagnosticsHellFinding`
- `ReportQualityStatus`

These names are suggestions only. Future implementation should follow the local
naming that best fits the final patch.

## 6. PASS / FAIL / INCOMPLETE audit criteria

PASS:
Report contains enough structured information to understand:

- which stage failed or remained unavailable;
- why it failed or remained unavailable;
- whether the failure is syntax/type/verifier/VM/practical/report boundary;
- that no false release-readiness or CTF closure claim is made.

FAIL:
Report is misleading or unusable:

- missing stage name;
- missing failure reason;
- ambiguous status;
- contradictory readiness claim;
- raw panic-like text exposed as the user-facing report;
- host-visible output leaked into `7hell` report;
- final PASS claimed while Diagnostics Hell remains unavailable;
- report-quality evaluator cannot classify the report safely.

INCOMPLETE:
Diagnostics Hell evaluator is not wired yet, or report structure lacks enough
data to evaluate safely.

## 7. Guardrails

Diagnostics Hell must not:

- call parser/check directly;
- call compiler/lowering directly;
- call verifier directly;
- call VM directly;
- call Practical envelope directly;
- perform host effects;
- inspect raw rendered program output;
- introduce new source execution behavior;
- mutate runtime state;
- create project-root behavior;
- become a release gate in this PR;
- claim CTF closure.

## 8. Risk analysis

Risk 1:
Diagnostics Hell becomes a hidden second runner.

Mitigation:
It must only read completed report objects.

Risk 2:
Report-quality logic is scattered across failure branches.

Mitigation:
Centralize evaluation after report assembly.

Risk 3:
Human and JSON outputs diverge.

Mitigation:
Evaluate internal structure before rendering.

Risk 4:
Diagnostics Hell starts judging aesthetics instead of structure.

Mitigation:
Keep v0 structural: stage, status, reason, diagnostic envelope, readiness
honesty.

Risk 5:
Future PCC diagnostics work is confused with the 7hell report-quality layer.

Mitigation:
7hell Diagnostics Hell v0 only checks report quality; broader diagnostic UX
remains future PCC work.

## 9. Audit verdict

Verdict: GO

Safe seam:
Evaluate Diagnostics Hell as a report-quality layer after `SevenHellReport`
assembly and before JSON / human rendering.

Next package:
`PR-7HELL-DIAG-S1` - cli(7hell): add Diagnostics Hell report-quality evaluator

Reason:
The current `execute_7hell_single_file` flow already has a completed internal
report before rendering. A minimal evaluator can read that report without
introducing new execution behavior. No separate refactor-only seam preparation
PR is required by the current structure.

## 10. CTF statement

CTF touched: none

Reason:
This is a docs-only audit locating the future Diagnostics Hell report-quality
seam. It does not change runtime value semantics, VM trap semantics, verifier
behavior, capability/audit behavior, trace policy, project-root behavior,
release gates, or CTF closure behavior.

## 11. 7hell status impact

- Diagnostics Hell implementation remains future work.
- report-quality seam is audited.
- no command behavior change.
- no execution behavior change.
- no verifier behavior change.
- no VM behavior change.
- no Practical envelope behavior change.
- no project-root behavior.
- no CI gate.
- no release readiness claim.
- no CTF closure.
- next safe work package is identified by audit verdict.
