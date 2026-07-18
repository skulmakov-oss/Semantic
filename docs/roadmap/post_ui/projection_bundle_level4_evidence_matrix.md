# ProjectionBundle Level-4 Evidence Matrix

Status: evidence matrix
Track: POST-UI / Intent-Driven Projection
Scope type: reader/parser promotion tracking
Current achieved level: Level 3 baseline
General Level 4 status: not claimed
Level 5+ status: not claimed

This matrix tracks the remaining evidence needed before any general Level 4 reader/parser claim may be made.

Level 5+ is not claimed.

| Requirement | Evidence file / guard | Current status | Claim impact |
| --- | --- | --- | --- |
| Reader/parser basis exists | `docs/spec/ui/projection_bundle_reader_parser_basis.md` | complete | Defines the basis without claiming general Level 4. |
| Normative logical contract freeze exists | `docs/spec/ui/projection_bundle_v0.md` | complete in UI-DNA2-8A | Freezes logical identity, stage, validation, resource and authority boundaries without claiming general Level 4. |
| Approved input boundary | `docs/spec/ui/projection_bundle_reader_parser_basis.md` + `docs/spec/ui/projection_bundle_v0.md` | partial | Bounded-input categories and logical stage boundaries are defined, but final serialization remains unresolved and parser implementation is blocked. |
| Approved output boundary | `docs/spec/ui/projection_bundle_reader_parser_basis.md` + `docs/spec/ui/projection_bundle_v0.md` | partial | Deterministic logical and inert-output constraints are defined, but general reader/parser behavior and an inert-loader implementation remain unauthorized. |
| Positive deterministic output golden tests | `tests/fixtures/post_ui/projection_bundle/expected/manifest_minimal.reader.out.txt` + `tools/post_ui/check_projection_bundle_sketch_reader_draft.ps1` | complete | Positive sketch output is deterministic and fixture-facing only. |
| Negative deterministic rejection golden tests | `tests/fixtures/post_ui/projection_bundle/expected/negative_pack.reader.out.txt` + `tools/post_ui/check_projection_bundle_sketch_reader_draft.ps1` | complete | Negative pack rejection is deterministic and fixture-facing only. |
| Missing-field rejection tests | `tests/fixtures/post_ui/projection_bundle/invalid/` + `tools/post_ui/check_projection_bundle_sketch_reader_draft.ps1` | complete | Missing required-field cases are covered by the current negative pack. |
| Malformed-field rejection tests | `tests/fixtures/post_ui/projection_bundle/invalid/` + `tools/post_ui/check_projection_bundle_sketch_reader_draft.ps1` | partial | Fixture-facing sketch evidence exists, but general reader/parser behavior remains not claimed. |
| Unknown-field policy tests | `tests/fixtures/post_ui/projection_bundle/invalid/` + `tools/post_ui/check_projection_bundle_sketch_reader_draft.ps1` | partial | Fixture-facing sketch evidence exists, but general reader/parser behavior remains not claimed. |
| Duplicate-field policy tests | `tests/fixtures/post_ui/projection_bundle/invalid/` + `tools/post_ui/check_projection_bundle_sketch_reader_draft.ps1` | partial | Fixture-facing sketch evidence exists, but general reader/parser behavior remains not claimed. |
| Field-ordering policy tests | `tests/fixtures/post_ui/projection_bundle/invalid/` + `tools/post_ui/check_projection_bundle_sketch_reader_draft.ps1` | partial | Fixture-facing sketch evidence exists, but general reader/parser behavior remains not claimed. |
| Placeholder trust rejection tests | future reader/parser fixtures + basis guard | missing | Placeholder trust rejection is not yet proven as a general reader/parser contract. |
| No loader path | `docs/spec/ui/projection_bundle_reader_parser_entry_gate.md` | guarded | Loader behavior remains not claimed and blocked by the gate. |
| No runtime path | `docs/spec/ui/projection_bundle_reader_parser_entry_gate.md` | guarded | Runtime behavior remains not claimed and blocked by the gate. |
| No activation path | `docs/spec/ui/projection_bundle_reader_parser_entry_gate.md` | guarded | Activation behavior remains not claimed and blocked by the gate. |
| No production UI wiring | `docs/spec/ui/projection_bundle_reader_parser_entry_gate.md` | guarded | Production UI behavior remains not claimed and blocked by the gate. |
| Claim-boundary guard | `tools/post_ui/check_projection_bundle_claim_boundaries.ps1` | complete | Forbidden overclaim wording is blocked at the claim boundary. |

General Level 4 is not currently achieved.

The current state contains narrow reader-facing fixture evidence only.

UI-DNA2-8A contract freeze does not achieve general Level 4.
The matrix is a promotion tracker, not a promotion claim.
