# Gate D Activation Policy v0 — Bounded UI-DNA2-10 Reference Contour

Status: NORMATIVE, BOUNDED IMPLEMENTATION RECORD
Track: UI-DNA2-10, authorized directly by Issue #1543
Gate D: OPEN WITH LIMITS for the bounded reference contour described below.
General Gate D (unrestricted runtime/admission/dispatch integration across
UI-DNA2): remains CLOSED. This document does not open it.
Production promotion: PROMOTE WITH LIMITS, bounded strictly to this
contour (recorded in
`docs/roadmap/post_ui/ui_dna2_implementation_roadmap.md` UI-DNA2-11).
Unrestricted, critical, or general production promotion remains NOT
AUTHORIZED.

## 1. Purpose

This document records the exact, machine-checkable activation policy that
opens Gate D for one bounded, non-critical, deterministic reference
contour: the UI-DNA2-10 end-to-end reference application
(`cargo run -p prom-ui-demo -- --ui-dna2-reference`). It is not a general
Gate D policy framework and does not authorize any contour beyond the one
described here.

Prior to Issue #1543, Gate D was documented only as `CLOSED`, with no
partial/limited variant defined anywhere in this repository's governance
documents. This document is the first place `OPEN WITH LIMITS` is defined,
under the direct owner authorization Issue #1543 records. It supersedes no
other Gate (`docs/roadmap/post_ui/ui_dna2_ownership_and_compatibility_freeze.md`
section 18's Gates A-E remain otherwise unchanged), and it does not reopen
the separate, already-closed `Gate D0` reference/lookup subtrack described in
`docs/roadmap/post_ui/ui_dna2_implementation_roadmap.md` section 6.

## 2. Fail-closed activation, not a runtime toggle

Gate D activation is evaluated once, deterministically, against one
candidate `ProjectionBundle v0` byte sequence, before any Shell Player
state, `ActiveProjectionTargetCatalog`, or `ShellSession` is constructed.
The implementation is
`prom_ui::shell_bridge::activate_projection_bundle_v0_gate_d` (crates/
prom-ui/src/shell_bridge.rs), which internally calls
`prom_ui::projection_bundle::verify_projection_bundle_v0`. There is no
runtime "enable Gate D" flag: activation either succeeds for a bundle that
satisfies every bound below, or fails closed and produces no partial
evidence.

```text
Gate D fail-closed invariant:
  a rejected bundle produces no ActivationTargetSnapshot,
  no node list, no accessibility evidence, and no Shell Player state.
```

## 3. Accepted versions

| Field | Accepted value |
| --- | --- |
| ProjectionBundle schema version | `SchemaVersion::CURRENT` (`1`) — exact match only |
| ProjectionBundle contract version | `ContractVersion::CURRENT` (`1`) — exact match only |
| Static UI IR Artifact schema/contract version (embedded section) | `1` / `1` — exact match only, enforced by `verify_static_ui_ir_artifact_v1` |
| Role Dictionary schema/contract version | `RoleDictionary::current()`'s versions (`1` / `1`) — exact match only |
| Projection Source Grammar | v0 only (enforced upstream by the parser) |

No range, "latest", or fallback negotiation is implemented. A version
mismatch anywhere in this table is a Compatibility rejection
(`docs/spec/ui/projection_bundle_v0.md` section 8).

## 4. Size and count limits

Fixed in `gate_d_reference_contour_limits()`
(`crates/prom-ui/src/shell_bridge.rs`):

| Limit | Value |
| --- | --- |
| Maximum bundle bytes | 262,144 (256 KiB) |
| Maximum binding count | 256 |
| Maximum action-route count | 256 |
| Maximum collection-anchor count | 256 |
| Maximum accessibility-entry count | 512 |
| Maximum Static UI IR input bytes (embedded section) | 262,144 (256 KiB) |
| Maximum surfaces | 16 |
| Maximum nodes | 512 |
| Maximum children per node | 64 |
| Maximum role bytes | 64 |

These are fixed constants for the bounded reference contour, not
caller-configurable. Exceeding any limit is a Resource rejection, checked
before the corresponding variable-length data is traversed.

## 5. Trusted verification inputs

No cryptographic trust algorithm is selected or implemented. Trust
verification for this bounded contour is **deterministic self-consistency
verification only** (canonicalize the decoded candidate, re-encode it,
compare byte-for-byte against the original input) — see
`docs/spec/ui/projection_bundle_v0.md` section 11 and section 17. This is
not authenticity, integrity, or admission proof. Digest and signature
algorithm ownership remain unresolved (`docs/spec/ui/projection_bundle_v0.md`
section 16, items 4-5); no crypto dependency exists in this workspace.

## 6. Supported patch operation family

Unchanged from the existing Shell Player contract
(`docs/spec/ui/shell_player_session_state_v0.md`): `SetBindingValue`,
`SetNodeAvailability`, `CollectionInsert`, `CollectionUpdate`,
`CollectionRemove`, `CollectionMove`, applied only through
`prom_ui::shell_bridge::apply_prepared_patch_submission` and
`prom_ui_runtime::shell_player::ShellSession::apply_projection_patch_batch`.
Gate D activation does not add or widen this family.

## 7. Supported renderer features

`Clear`, `FillRect`, and `DrawText` draw commands
(`prom_ui_runtime::DrawCommand`), presented through
`prom-ui-backend-native`'s existing wgpu pipeline plus the glyph-rendering
addition landed under Issue #1543. Renderer features beyond these three
draw-command kinds (general layout realization with measured text metrics,
arbitrary shape primitives) are not implemented and not authorized by this
policy.

## 8. Supported input features

Keyboard and pointer events already translated by
`prom-ui-backend-native`'s winit integration
(`InputEventKind::{KeyDown, KeyUp, PointerMoved, PointerDown, PointerUp,
CloseRequested}`), deterministic hit testing against the layout physical
placement model, and one bounded local focus-index model (`Tab`/`Shift+Tab`
traversal, `Enter`/`Space` activation). Pointer capture is limited to the
single active drag/press gesture already modeled by
`InteractionIntentKind`. General multi-touch, gesture recognition, or IME
composition are not implemented and not authorized by this policy.

## 9. Supported accessibility features

Deterministic accessibility evidence derived directly from the activated
bundle's Static UI IR and Role Dictionary
(`prom_ui::projection_bundle::derive_accessibility_entries`,
`prom_ui::shell_bridge::BridgeAccessibilityEntry`): one role (generic
container, label, or button) and one label string per node. This evidence
is real, deterministic, and printed/available as structured data for every
activated bundle. Wiring this evidence into a platform accessibility tree
(`accesskit`/`accesskit_winit` or an equivalent OS-level integration) is
**not implemented** by this bounded contour and not authorized by this
policy; it remains a distinct, separately-scoped future integration. Richer
accessibility state (live regions, value ranges, custom actions) is
likewise not implemented.

## 10. Excluded contours

Gate D activation under this policy explicitly does **not** authorize:

```text
general ProjectionBundle Level 4/5 production reader/parser behavior
cryptographic digest or signature trust verification
critical or pinned bundle handling
unrestricted runtime admission/dispatch integration
action/effect authorization beyond the existing repository-owned admission
  boundary this task wires into (see docs/roadmap/post_ui/ui_dna2_implementation_roadmap.md)
Semantic truth ownership or mutation
production promotion (UI-DNA2-11 decision recorded separately)
Workbench, Semantic Studio, or ui-shell-kit integration
any contour beyond the one bounded UI-DNA2-10 reference application
```

## 11. Relationship to existing invariants

This policy does not weaken, replace, or duplicate any already-frozen
Shell Player invariant. It runs strictly before
`prom_ui_runtime::shell_player::create_shell_session` is called; once
`activate_projection_bundle_v0_gate_d` succeeds, the resulting
`ActivationTargetSnapshot` flows into the existing, unmodified Shell Player
pipeline (stages 1-9, `PreparedPatchSubmission`, `ActiveProjectionTargetCatalog`,
atomic commit, replay-cursor advancement) exactly as
`docs/spec/ui/shell_player_session_state_v0.md` already specifies.

```text
Gate D activation != Shell Player stage 1-9 transition evaluation
Gate D activation != patch admission
Gate D activation != action admission
bundle activation != production promotion
```
