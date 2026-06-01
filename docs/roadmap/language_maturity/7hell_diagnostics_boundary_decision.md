# 7HELL Diagnostics Hell Boundary Decision

Status: decision record
Scope: Diagnostics Hell / User Pain boundary
Mode: research / readiness investigation
Non-goal: implementation, release readiness, CI gate, CTF closure

Related:

- `7hell_current_status_after_wr2.md`
- `7hell_waypoint_review_after_s1_s7_e1_e5.md`

The report-quality seam audit is recorded in:

- `../../../reports/7hell_diag_report_quality_seam_audit.md`

## 1. Background

`7hell` has reached an active bounded qualification contour.

Syntax, Type, Lowering, Verifier, VM, and Practical report paths are now present
in bounded form. Practical Hell is active through a non-rendering practical
envelope.

Diagnostics Hell / User Pain remains not implemented.

The post-WR2 status refresh preserved the Diagnostics Hell decision boundary
instead of choosing a path prematurely. The open classification choices were:

- A. Standalone `7HELL-S8` stage
- B. Report-quality layer applied across all failure reports
- C. Future PCC diagnostics track, not part of the current 7hell wave

## 2. Decision

Diagnostics Hell / User Pain is classified as:

B. Report-quality layer applied across all failure reports.

It is not a new execution stage.
It is not a new VM/verifier/runtime path.
It is not a project-root feature.
It is not a release gate yet.

## 3. Rationale

Diagnostics Hell should evaluate the quality of reports already produced by
prior stages.

It should not create another execution path.
It should not call check / compile / verify / run itself.
It should not render host-visible program output.
It should not expose raw observation text.
It should inspect report structure and classify usability.
It should be safe to add without touching VM, verifier, or runtime semantics.

This keeps `7hell` as a measurement contour, not a hidden second runner.

Execution stages produce facts.
Diagnostics Hell evaluates whether those facts are understandable and safe to
expose.

## 4. Rejected alternatives

| Option | Decision | Reason |
| --- | --- | --- |
| A. Standalone `7HELL-S8` stage | rejected for now | It risks implying a new execution stage and may blur the line between report evaluation and pipeline execution. |
| B. Report-quality layer applied across all failure reports | accepted | It is the smallest safe boundary: evaluate report quality without changing execution behavior. |
| C. Future PCC diagnostics track only | rejected as the sole answer | PCC diagnostics work is still needed later, but `7hell` already needs a minimal report-quality boundary now to prevent unreadable or misleading failure reports. |

## 5. Boundary definition

```text
Diagnostics Hell =
  report-quality assessment over existing 7hell stage results
```

Diagnostics Hell may read:

- stage names
- stage statuses
- failure reason
- diagnostic envelope
- report-level messages
- structured metadata already present in the report

Diagnostics Hell must not:

- call parser/check directly
- call compiler/lowering directly
- call verifier directly
- call VM directly
- call Practical envelope directly
- perform host effects
- inspect raw rendered program output
- introduce new source execution behavior
- mutate runtime state
- create project-root behavior
- become a release gate in this PR

## 6. Minimal future acceptance shape

Future Diagnostics Hell PASS / FAIL / INCOMPLETE logic should be defined at the
report-quality layer.

PASS:
A failed or incomplete `7hell` report contains enough structured information for
a developer to understand:

- which stage failed or remained unavailable;
- why it failed or remained unavailable;
- whether the failure is due to syntax/type/verifier/VM/practical/report boundary;
- whether there is no misleading release-readiness claim.

FAIL:
A report has a failure but lacks usable structured diagnostics, for example:

- missing stage name;
- missing failure reason;
- ambiguous status;
- raw panic-like text exposed as user-facing report;
- contradictory readiness claim;
- host-visible output leaked into `7hell` report;
- report claims final readiness while a required stage is not implemented.

INCOMPLETE:
Diagnostics Hell itself has not yet been wired as an active report-quality
evaluator.

## 7. Current status after this decision

After this PR:

- Diagnostics Hell boundary is decided.
- Diagnostics Hell implementation is still not present.
- `7hell` remains not a CI gate.
- `7hell` remains not a release gate.
- no CTF closure is claimed.
- next implementation work should be a separate audit PR.

## 8. Next safe work package

Next package:

`PR-7HELL-DIAG-AUDIT` - audit(7hell): locate report-quality seam for
Diagnostics Hell

Purpose:
Find where the `7hell` report can be evaluated for diagnostics/report quality
without invoking new execution behavior.

Do not implement this in the current PR.

## 9. CTF statement

CTF touched: none

Reason:
This is a docs-only decision record for Diagnostics Hell / User Pain boundary.
It does not change runtime value semantics, VM trap semantics, verifier
behavior, capability/audit behavior, trace policy, project-root behavior,
release gates, or CTF closure behavior.

## 10. 7hell status impact

- Diagnostics Hell boundary is decided as a report-quality layer.
- no command behavior change
- no execution behavior change
- no verifier behavior change
- no VM behavior change
- no Practical envelope behavior change
- no project-root behavior
- no CI gate
- no release readiness claim
- no CTF closure
- next safe work is audit-only
