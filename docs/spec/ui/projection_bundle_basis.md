# ProjectionBundle Basis v0

Status: evidence basis
Track: POST-UI / Intent-Driven Projection
Scope type: claim boundary
Implementation status: no parser / no loader / no runtime
Authority status: non-authorizing

This document defines what the current ProjectionBundle fixture evidence can and cannot claim.

It does not define a runtime.
It does not define a loader.
It does not define a parser.
It does not define final serialization.
It does not define verification authority.
It does not authorize production UI wiring.

```text
Evidence is not implementation.
A passing guard is not a working system.
A fixture is not a loader contract.
A draft type is not semantic authority.
```

No claim may exceed the evidence that directly supports it.

## 1. Purpose

The purpose of this basis is to prevent wishful thinking from becoming architecture.

This basis exists to prevent overclaiming and to define the current evidence boundary before any parser, loader, runtime, or production integration is allowed.

## 2. Current Evidence Inventory

`tests/fixtures/post_ui/projection_bundle/README.md`
- is: inert fixture-anchor documentation for the ProjectionBundle evidence contour
- is not: executable content, final serialization, parser input, loader input, runtime input, or production UI wiring

`tests/fixtures/post_ui/projection_bundle/manifest_minimal.sketch.md`
- is: inert planning evidence for a minimal ProjectionBundle manifest shape
- is not: final serialization, parser input, loader input, runtime input, verification input

`tools/post_ui/check_projection_bundle_fixtures.ps1`
- is: text-boundary guard for the inert ProjectionBundle fixture anchor
- is not: parser, loader, runtime reader, schema authority, or implementation proof

`tools/post_ui/check_post_ui_fixtures.ps1`
- is: local aggregator for POST-UI fixture guards
- is not: CI wiring, runtime wiring, or evidence of a working ProjectionBundle system

`tools/post_ui/projection_bundle_manifest_draft.rs`
- is: fixture-facing Rust draft types for manifest evidence
- is not: crate API, parser, loader, runtime activation code, or semantic authority

`tools/post_ui/check_projection_bundle_manifest_draft.ps1`
- is: compile-only metadata guard for the Rust draft file
- is not: runtime validation, loader validation, or proof of semantic correctness

`tools/post_ui/check_projection_bundle_manifest_drift.ps1`
- is: text-anchor drift guard between the inert sketch and the Rust draft constant body
- is not: parser verification, Rust AST analysis, loader verification, or runtime verification

`tools/post_ui/projection_bundle_sketch_reader_draft.rs`
- is: fixture-facing executable reader draft for one inert sketch, plus deterministic negative fixture and golden output checks
- is not: general reader/parser behavior, loader behavior, runtime behavior, or production UI wiring
- evidence: the inert positive manifest sketch is accepted; the invalid manifest sketch with `allow_production_activation: true` is rejected; the positive normalized reader output golden fixture matches; the negative rejection report golden fixture matches; the exact golden comparison guard passes
- evidence type: narrow reader-facing fixture evidence

`tools/post_ui/check_projection_bundle_sketch_reader_draft.ps1`
- is: compile-and-run guard for the fixture-facing sketch reader draft and exact golden comparison
- is not: runtime activation, loader validation, verification, or proof of general reader/parser behavior
- evidence: the reader draft guard exits successfully on the accepted positive sketch, the rejected negative sketch, and the emitted golden output pack

`tests/fixtures/post_ui/projection_bundle/expected/manifest_minimal.reader.out.txt`
- is: golden normalized reader output for the accepted positive sketch
- is not: final serialization, parser input, loader input, runtime input, or production UI wiring

`tests/fixtures/post_ui/projection_bundle/expected/negative_pack.reader.out.txt`
- is: golden rejection report for the rejected negative pack
- is not: final serialization, parser input, loader input, runtime input, or production UI wiring

## 3. Claim Levels

Claim Level 0 — Text exists
- A file or literal exists on disk.

Claim Level 1 — Boundary text is present
- The file contains inert-boundary wording and negative claims.

Claim Level 2 — Draft shape compiles
- The fixture-facing Rust draft file compiles as metadata only.

Claim Level 3 — Sketch/draft anchors match
- The sketch and Rust draft constant body share selected literal anchors.

Claim Level 4 — Reader/parser behavior
- A reader or parser can consume the manifest shape correctly.

Claim Level 5 — Loader behavior
- A loader can select, reject, or activate bundles correctly.

Claim Level 6 — Runtime behavior
- A runtime can activate and use ProjectionBundle behavior correctly.

Claim Level 7 — Production UI behavior
- Production UI wiring is safe, correct, and ready for release use.

Current achieved level: Level 3 baseline.

Levels 4–7 are not claimed.

## 4. Narrow Reader Evidence Note

After the sketch reader draft and golden output pack, the evidence contour includes a narrow reader-facing fixture evidence:

- positive normalized reader output golden fixture;
- negative rejection report golden fixture;
- exact golden comparison guard;
- the inert positive manifest sketch is accepted;
- the invalid manifest sketch with `allow_production_activation: true` is rejected;
- the check is fixture-facing and test-only.

Current achieved level remains: Level 3 baseline.

Level 4 is not generally achieved.

It does not claim general Level 4 reader/parser behavior.
It does not claim loader behavior.
It does not claim runtime behavior.
It does not claim verification behavior.
It does not claim production UI behavior.

## 5. What Current Guards Prove

The current guards prove only repo-local evidence conditions, such as:

- required fixture files exist;
- required inert-boundary text is present;
- forbidden schema/runtime/code artifacts are absent from the current fixture directory;
- the fixture-facing Rust draft file compiles as metadata;
- the manifest sketch and Rust draft constant body share selected literal anchors.

These checks prove only that these checked conditions hold at the checked revision.

## 6. What Current Guards Do Not Prove

The current guards do not prove that:

- ProjectionBundle works.
- any parser is correct.
- any loader is correct.
- any runtime behavior is correct.
- bundle verification is correct.
- hash or signature handling is correct.
- semantic admission is correct.
- UI IR is correct.
- Action IR is correct.
- Binding Graph is correct.
- patch stream behavior is correct.
- shell-player behavior is correct.
- production UI behavior is correct.
- security is proven.
- compatibility with future serialization is proven.

## 7. Allowed Claims

| Claim | Allowed wording |
| --- | --- |
| ProjectionBundle fixture anchor exists | An inert fixture anchor exists. |
| Fixture guard passes | The current fixture anchor satisfies the current boundary text checks. |
| Rust draft compiles | The fixture-facing draft type file compiles as standalone metadata. |
| Sketch and draft align | The sketch and Rust draft constant body share selected literal anchors. |

## 8. Forbidden Claims

| Forbidden claim | Reason |
| --- | --- |
| ProjectionBundle is implemented | A loader/parser/runtime does not exist. |
| ProjectionBundle is verified | No verification implementation exists. |
| The manifest format is stable | No final serialization has been selected. |
| The guard proves security | The guard checks only crude evidence anchors. |
| The Rust draft is the public API | The draft is fixture-facing only and not a crate. |
| Runtime can consume this bundle | No runtime reader or activation path exists. |

## 9. Authority Boundaries

Semantic owns meaning.
Projection owns presentation intent.
UI IR owns structure.
Action IR owns affordance routing.
Binding Graph owns deterministic dependency mapping.
Patch streams own projection updates.
Shell owns rendering behavior.
Renderer owns pixels.
Verifier / admission owns semantic admission decisions.
Capability / audit authority owns capability checks, host-effect permission, critical action authorization, and audit evidence boundaries.
Runtime owns execution / scheduling only where explicitly specified.

```text
This basis document does not move authority between layers.
```

## 10. Promotion Gates

Before Level 4:
- a separately approved reader/parser basis;
- explicit non-goals;
- negative fixtures;
- reader/parser tests;
- no runtime activation.

Before Level 5:
- loader basis;
- loader rejection tests;
- invalid bundle tests;
- trust placeholder rejection;
- no production UI wiring.

Before Level 6:
- runtime basis;
- activation policy;
- freshness policy enforcement;
- denial/recovery evidence;
- no silent authority transfer.

Before Level 7:
- production UI admission basis;
- capability/audit integration basis;
- user-visible denial behavior;
- rollback/fallback evidence.

## 11. Non-Goals

This document is not a parser specification.
This document is not a serialization specification.
This document is not a loader contract.
This document is not a runtime contract.
This document is not a security proof.
This document is not a production readiness claim.

## 12. Working Rule for Future PRs

Every future ProjectionBundle PR must state which claim level it changes.

If a PR increases the claim level, it must add evidence for that level.

If no new evidence is added, the PR must not increase the claim level.

Do not use words like implemented, verified, secure, runtime-prepared, or production-prepared unless the corresponding basis level and evidence exist.
