# Calculator Executable Evidence Ledger

## Status

The `ui-shell-kit` calculator reference shell now has a documented executable evidence chain.
This ledger records what is already proven by merged sandbox tests.
It does not introduce new code, new promotion claims, or production wiring.

## Purpose

Record the current executable evidence state for the calculator reference shell so the POST-UI track can refer to one concise ledger instead of re-deriving the same evidence from the individual test PRs.

## Evidence Chain

- #1319 — calculator interaction behavior evidence
- #1320 — calculator rendered snapshot evidence
- #1321 — calculator motion phase evidence
- #1323 — calculator focus/action trace evidence
- #1324 — calculator hit-test stability evidence

## What Is Covered

- canonical `7 + 3 = 10` interaction path;
- controller / layout / hit-testing route;
- rendered snapshot evidence;
- deterministic motion phases: `Entrance`, `Settling`, `Settled`;
- focus / action trace path;
- hit-test stability for canonical buttons;
- outside-hit behavior;
- sandbox-only render evidence.

## What Is Not Claimed

- not production UI;
- not `prom-ui` integration;
- not Workbench integration;
- not verifier / VM / SemCode behavior;
- not runtime capability behavior;
- not renderer backend decision;
- not pixel-perfect screenshot contract;
- not final animation system;
- not promotion of `ui-shell-kit`.

## Boundary

Docs-only.

No code changes.
No production UI wiring.
No workspace wiring changes.
No promotion decision.

## Relationship to ui-shell-kit Track

- #1310 — parent POST-UI track
- #1316 — initial documentation spine
- #1317 — calculator reference scenario docs
- #1322 — surface inventory
- #1319 / #1320 / #1321 / #1323 / #1324 — executable evidence chain

## Next Candidate Areas

Candidates only, not commitments:

- focus traversal refinement;
- accessibility evidence;
- action trace normalization;
- snapshot golden policy refinement;
- reusable primitive review under the #1315 promotion gate.
