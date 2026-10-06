# Semantic-native Readiness Audit — Findings Registry

Status: active registry (audit only; no repair authorized)
Charter: [`charter.md`](charter.md)

Findings use native IDs `NA-<group>-<nnn>` (charter §12, §15). Status values:
`OPEN — AUDIT ONLY` until a separate owner-authorized repair slice.

| ID | Group | Classification | Severity | Status | Origin |
|---|---|---|---|---|---|
| NA-N2-001 | N2 | DEFER-SCOPE | P3 | OPEN — AUDIT ONLY | NATIVE-AUDIT-02 |

---

## NA-N2-001 — Examples index still says SSF-12 has issued no verdict

ID: NA-N2-001
Title: Examples index still says SSF-12 has issued no qualification verdict
Group: N2 (claim surface for the F11 benchmark row and the onboarding index)
Candidate origin: NATIVE-AUDIT-02 claim trace (not a Slice 0 candidate)
Classification: DEFER-SCOPE — real stale current-facing statement; it belongs
to the release-posture documentation perimeter (the scope of the #1996
reconciliation), not to native-program readiness.
Severity: P3 — documentation drift with no material readiness overstatement
(it understates; it creates no false confidence about any native program).
Claim: `docs/examples_index.md:79-80` — "Nothing in this index is a
published-stable claim. SSF-12 (#1583) has issued no qualification verdict."
README links this index as the examples guide.
Actual proven evidence: SSF-12 #1583 is CLOSED (2026-10-04);
`reports/semantic_stable_foundation_final_verdict.md` is "the authoritative
qualification and verdict deliverable for SSF-12 (#1583)" and records
ORACLE QUALIFIED WITH EXPLICIT LIMITS / PROMOTE WITH EXPLICIT LIMITS;
`v1.2.0` was published 2026-10-04. `docs/examples_index.md` was last changed in
`a1591bc1` (C0, 2026-10-02), before the verdict.
Gap: a current-facing index states a pre-verdict status that is no longer true.
The first sentence ("not a published-stable claim") remains consistent with the
index's own rows; only the SSF-12 sentence is stale.
Exact evidence: `docs/examples_index.md:78-80`; `gh issue view 1583` (CLOSED
2026-10-04T04:39:13Z); final verdict lines 3, 276, 367; `git log -- docs/examples_index.md`.
Repair note: a release-posture documentation slice could replace the sentence
with the published SSF-12 verdict reference; not authorized here.
Status: OPEN — AUDIT ONLY
