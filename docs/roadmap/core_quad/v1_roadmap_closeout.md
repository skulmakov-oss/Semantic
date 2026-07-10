# Quad Logic Engine v1 Roadmap Closeout

## Purpose

This is the evidence-only closeout for the Quad Logic Engine v1 umbrella roadmap. It consolidates the completed child issues, their implementation and qualification evidence, the explicitly separated VM/QTruth follow-up contour, and the remaining work that is outside v1 roadmap scope.

## Issue

This PR closes #1404. It changes documentation and harness evidence only; it does not change code or behavior.

## Starting main commit

The starting `main` commit was `cbcbd19341f8aef37cba3b2f275e7528527f0db7`, the squash merge of PR #1486.

## Roadmap scope

The v1 contour is owned by `semantic-core-quad`, with `semantic-core-capsule` as the downstream smoke consumer and the legacy compatibility crate retained for compatibility only. The contour covers the frozen Quad Logic Frame, scalar truth-map oracle, SWAR equivalence, typed mask bridge, explicit delta split, additive compatibility policy, tile/bank lifting, qualification/relative benchmark evidence, and the separate visual transport boundary.

Landed on `main` means implemented and evidenced. It does not widen serialized compatibility, GPU ABI guarantees, or a universal performance promise.

## Original child issue ledger

All original children except the umbrella were verified by GitHub as `CLOSED / COMPLETED`.

| Issue | State / reason | Closing PR(s) or evidence | Main commit evidence | Acceptance summary |
| --- | --- | --- | --- | --- |
| #1405 Quad Logic Frame v1 spec | CLOSED / COMPLETED | PR #1425 | `563bd1b7f63a5ef46df226b88968e0e503a453d5` | Frozen N/F/T/S encoding, operation families, mask isolation, IMPLIES policy, and compatibility owner. |
| #1406 scalar LUT truth-table layer | CLOSED / COMPLETED | PRs #1426, #1427, #1428, #1429, #1430, #1431 | PR #1426 `73f35f97ed02257ae522503757c12ad076541bf3` | Scalar maps and derived policies provide the correctness oracle; EQUIV remains guarded/deferred. |
| #1407 SWAR backend | CLOSED / COMPLETED | PRs #1432, #1433, #1434, #1435, #1437 | PR #1437 `38393d22fbd10e06ac882b5d087e541740cee566` | SWAR maps were proven against scalar behavior and promoted to qualified default aliases. |
| #1408 dense/physical mask bridge | CLOSED / COMPLETED | PR #1478 | `395641a0648e5e1a8524aed688b10512a14e9b84` | Typed dense and physical masks, explicit conversion, and invalid-mask rejection remove new raw-mask ambiguity. |
| #1409 exact-state / plane-delta split | CLOSED / COMPLETED | PR #1478 | `395641a0648e5e1a8524aed688b10512a14e9b84` | Exact-state and plane deltas are separate; legacy delta surfaces remain documented compatibility APIs. |
| #1410 QImpl / truth-operation semantic alignment | CLOSED / COMPLETED | PR: not resolved in this local audit; issue state verified by GitHub. Related evidence: #1441, #1443, #1445, #1447. | Related merged evidence is recorded in the PR ledger. | Current derived `IMPLIES = NOT(A) join B` is retained explicitly; truth-table and lattice families are not silently conflated. |
| #1411 tile and bank APIs | CLOSED / COMPLETED | PRs #1480, #1481 | #1480 `f925475cf2ba40fca4706c933f9d7b77956ae82f`; #1481 `77c7ac9b23169e2c66a71f098f0a87ff18e13c2f` | Deterministic tile lifting and in-place register/tile bank helpers preserve ordering and allocation behavior. |
| #1412 qualification tests and benchmarks | CLOSED / COMPLETED | PRs #1483, #1484 | #1483 `ceee937b87e3b97519a9bbd2ff52fb727a499f31`; #1484 `890f17439eaaa8f89440cfdd017a8c7afb31f584` | Public qualification matrix and observational relative benchmark closeout provide correctness and performance evidence without gates. |
| #1413 compatibility rollout policy | CLOSED / COMPLETED | PRs #1478, #1482 | #1482 `f563e39015db7d3a0ed0f727666b6481e5c44764` | Additive-first public API policy, feature posture, compatibility names, and EQUIV deferral are documented. |
| #1417 aligned tile and GPU upload layout | CLOSED / COMPLETED | PRs #1479, #1485, #1486 | #1486 `cbcbd19341f8aef37cba3b2f275e7528527f0db7` | Core layout, visual `[u32; 4]` transport, Pod/Zeroable qualification, byte view, and WGSL mirror contract are separated correctly. |

## Extended follow-up issue ledger

The following closed issues were found through the GitHub `Parent: #1404` search but are not original child rows. They are recorded as extended VM, QTruth, lattice, and semantic-boundary follow-up contours, not retroactively reclassified as original children.

| Issues | State / reason | Contour evidence |
| --- | --- | --- |
| #1436, #1438, #1440, #1442, #1444, #1446, #1448, #1450, #1452, #1454, #1456, #1458, #1460, #1462, #1464, #1466, #1468, #1470, #1472, #1474, #1476 | CLOSED; GitHub state audit returned closed for each | Separate lattice/QTruth/VM contracts, opcode inventory, explicit source admission, unsupported-opcode guards, and legacy-contour audits. |

Representative merged evidence includes PR #1441 (opcode semantic mismatch audit), #1443 (lattice truth opcode contract), #1445 (explicit lattice aliases), #1447 (quad opcodes through lattice aliases), #1457 (unsupported QTruth opcode boundary), #1459 (QTruth opcode execution through core maps), #1461/#1463 (explicit QTruth IR representation/encoding), #1465/#1467 (SemCode envelope and admission audits), #1469/#1471 (source intrinsic contract/admission), #1473/#1475 (source execution guard and contour close), and #1477 (legacy lattice contour close). These are explicit, separately reviewed contours; they are not hidden changes in this docs-only PR.

## PR ledger

Known merged PRs for the v1 contour and its directly related follow-ups:

- #1425 — freeze Quad Logic Frame v1 spec — `563bd1b7f63a5ef46df226b88968e0e503a453d5`
- #1426 — logic-frame skeleton, NOT table, and tests — `73f35f97ed02257ae522503757c12ad076541bf3`
- #1427 — scalar AND/OR truth tables — `ca7dbfc32fd73d61247d965257b7698c44e06033`
- #1428 — scalar XOR truth table — `a91733a6b7f7db55b19af290688aa4bbf028644c`
- #1429 — derived scalar IMPLIES truth table — `10a930909990ab3c85c1a45b2bb57b94676c1370`
- #1430 — derived scalar NAND/NOR truth tables — `ac48a6562844ffddc7ab58d6e048bcd61c550e2a`
- #1431 — deferred EQUIV guard — `54fa64b0343f295593a14e54f08bf87c471bb160`
- #1432–#1435 — scalar oracle and SWAR NOT/XOR, AND/OR, IMPLIES/NAND/NOR — merged evidence recorded by GitHub
- #1437 — promote proven SWAR maps to default aliases — `38393d22fbd10e06ac882b5d087e541740cee566`
- #1441, #1443, #1445, #1447 — VM/lattice semantic audit, contract, aliases, and routing — merged follow-up evidence
- #1478 — compatibility, mask bridge, and delta split — `395641a0648e5e1a8524aed688b10512a14e9b84`
- #1479 — QuadTile128 core layout qualification — `a17c3a86c3d2c666b015c32bf3e3d83707299707`
- #1480 — tile truth-map lifting — `f925475cf2ba40fca4706c933f9d7b77956ae82f`
- #1481 — bank truth-map helpers — `77c7ac9b23169e2c66a71f098f0a87ff18e13c2f`
- #1482 — compatibility closeout — `f563e39015db7d3a0ed0f727666b6481e5c44764`
- #1483 — public qualification matrix — `ceee937b87e3b97519a9bbd2ff52fb727a499f31`
- #1484 — relative benchmark closeout — `890f17439eaaa8f89440cfdd017a8c7afb31f584`
- #1485 — GPU tile transport layout — `1644fd8643937427265e39f415ce6b553044f4be`
- #1486 — GPU tile upload closeout — `cbcbd19341f8aef37cba3b2f275e7528527f0db7`

## Spec outcome

`docs/spec/quad_logic_frame_v1.md` is frozen as the semantic frame reference: `N=00`, `F=01`, `T=10`, `S=11`; truth-table, knowledge-lattice, diagnostic, and event/delta families remain distinct. `semantic-core-quad` is canonical and the legacy compatibility crate remains compatibility-only.

## Scalar oracle outcome

The scalar LUT layer is the deterministic reference for truth-map operations. Derived NAND/NOR and retained derived IMPLIES semantics are documented rather than silently replaced.

## SWAR outcome

SWAR implementations were qualified against scalar behavior and promoted through explicit default aliases only after equivalence evidence. No CPU-specific feature is required by the v1 contract.

## Mask bridge outcome

Dense logical masks and packed physical masks are typed separately. Conversion is explicit, invalid physical masks are rejected, and existing `QuadMask32` compatibility behavior is retained.

## Delta split outcome

`ExactStateDelta32` and `PlaneDelta32` distinguish exact N/F/T/S transitions from truth/falsity-plane membership changes. Legacy `StateDelta32` and related names remain compatibility surfaces with documented meaning.

## Tile and bank lifting outcome

`QuadTile128`, `QuadroBank<N>`, and `QuadTileBank<N>` have deterministic map/lifting APIs and in-place bank helpers. Ordering, indexing, and allocation-free behavior remain intact.

## Compatibility policy outcome

The v1 policy is additive-first. Existing public names are preserved; ambiguous names are documented before any future deprecation; no EQUIV API is part of qualified v1. Serialized cross-version compatibility is not claimed.

## Qualification and benchmark outcome

PR #1483 supplies the public qualification matrix. PR #1484 adds local, relative, observational benchmark evidence using the existing `core-bench` mechanism. No timing threshold, universal speedup, cross-machine baseline, or CPU-feature requirement is claimed.

## GPU transport boundary outcome

PRs #1479, #1485, and #1486 keep `QuadTile128` as canonical core storage and `GpuQuadTile128` in the visual/backend crate. The transport uses `[u32; 4]`, static layout checks, Pod/Zeroable, a read-only byte view, and a WGSL mirror string. No WGPU buffer or renderer integration is implied.

## VM/source semantics boundary

The v1 frame retains current derived IMPLIES semantics and does not silently map source `QImpl` to a different primitive table. Later QTruth/lattice work was tracked through explicit issue and PR contours with admission and unsupported-boundary evidence. This closeout changes no VM, verifier, source, IR, opcode, or runtime file.

## Compatibility and public API posture

The qualified v1 public surface is additive and explicitly named. Core semantic ownership remains in `semantic-core-quad`; the legacy compatibility crate remains compatibility-only; visual transport remains outside core. No public item is removed by this closeout.

## Feature posture

The core feature posture remains `std` tested, `no_std` check-qualified for `semantic-core-quad`, and `serde` qualified under all features. The visual `wgpu-backend` path is optional and does not enter the core crate's dependency surface.

## Core capsule posture

`semantic-core-capsule` remains the minimum downstream smoke consumer. Its passing test suite is integration evidence for the v1 contour, not a claim of total external compatibility.

## No-default warning posture

The pre-existing `prom-ui-backend-native --no-default-features` compile blocker remains outside this roadmap closeout and is not claimed as a pass. It does not affect the core no-default feature posture or the dependency-isolation evidence recorded by the visual transport slices.

## Non-changes

This PR modifies no Rust code, tests, Cargo files, lockfile, specs, workflows, scripts, VM, verifier, runtime, visual transport, WGPU behavior, or public APIs. It does not reopen or create issues, and it does not claim future renderer/runtime, richer visual tile, QTruth ergonomics, or Atlas kernel work complete.

## Definition of Done mapping

| #1404 criterion | Status | Evidence / reason |
| --- | --- | --- |
| All child issues closed or explicitly deferred | PASS | #1405–#1413 and #1417 are CLOSED / COMPLETED; EQUIV is explicitly deferred by #1413 policy. |
| Quad Logic Frame v1 documented | PASS | Frozen `docs/spec/quad_logic_frame_v1.md` and PR #1425. |
| Public API compatibility preserved or shimmed | PASS | Additive mask/delta/tile/bank APIs and retained compatibility names in #1478/#1482. |
| LUT/SWAR equivalence tested | PASS | Scalar oracle, SWAR qualification, public matrix #1483, and core tests. |
| Exact-state and plane-delta tested | PASS | Explicit delta types and transition matrix evidence in #1478/#1483. |
| `semantic-core-capsule` remains green | PASS | Required local capsule verification passes. |
| No VM semantic changes without explicit decision | PASS | IMPLIES policy is explicit; QTruth/lattice changes were separately scoped and evidenced; this PR is docs-only. |
| No legacy compatibility ownership regression | PASS | The legacy compatibility crate remains compatibility-only; canonical ownership stays in `semantic-core-quad`. |
| No UI/WGPU widening inside `semantic-core-quad` | PASS | GPU transport is visual/backend-owned; no core WGPU/bytemuck dependency was added. |

This mapping is the reason this PR closes #1404: every original child is closed or explicitly policy-deferred, implementation and qualification evidence is landed, and the remaining contours are named rather than silently included.

## Remaining work outside #1404

- The pre-existing `prom-ui` native no-default compile blocker.
- Future renderer/runtime upload integration.
- Future richer `GpuQuadTileVisual` transport, if separately approved.
- Future QTruth language ergonomics, if tracked separately from the closed explicit-admission contours.
- Atlas kernel work outside Semantic core-quad v1.

These items are outside this roadmap closeout and are not reopened here.
