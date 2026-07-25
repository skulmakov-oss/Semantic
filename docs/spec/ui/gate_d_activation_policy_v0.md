# Gate D Activation Policy v0 — Bounded UI-DNA2-10 Reference Contour

Status: NORMATIVE, BOUNDED IMPLEMENTATION RECORD
Track: UI-DNA2-10, authorized directly by Issue #1543
Gate D: OPEN WITH LIMITS for the bounded reference contour described below.
General Gate D (unrestricted runtime/admission/dispatch integration) stays
CLOSED — this document does not open it.
Production promotion: PROMOTE WITH LIMITS, bounded strictly to this
contour (`docs/roadmap/post_ui/ui_dna2_implementation_roadmap.md`
UI-DNA2-11). Unrestricted or general production promotion is not
authorized.

## 1. Purpose

Records the exact, machine-checkable policy that opens Gate D for one
bounded, non-critical, deterministic contour: the UI-DNA2-10 reference
application (`cargo run -p prom-ui-demo -- --ui-dna2-reference`). It
authorizes nothing beyond that one contour.

Before Issue #1543, Gate D was documented only as `CLOSED`, with no
partial variant defined anywhere. This is the first place `OPEN WITH
LIMITS` is defined. It doesn't touch Gates A-E
(`ui_dna2_ownership_and_compatibility_freeze.md` §18) or reopen the
already-closed Gate D0 reference/lookup subtrack (roadmap §5).

## 2. Fail-closed, not a runtime toggle

Activation is evaluated once, deterministically, against one candidate
`ProjectionBundle v0` byte sequence, before any Shell Player state exists.
Implementation: `prom_ui::shell_bridge::activate_projection_bundle_v0_gate_d`
(calls `prom_ui::projection_bundle::verify_projection_bundle_v0`
internally). There's no runtime enable flag — a bundle either satisfies
every bound below or activation fails closed with no partial evidence
(no snapshot, no node list, no accessibility evidence).

## 3. Accepted versions

| Field | Accepted value |
| --- | --- |
| ProjectionBundle schema/contract version | `1` / `1`, exact match |
| Static UI IR Artifact schema/contract version | `1` / `1`, exact match |
| Role Dictionary schema/contract version | `RoleDictionary::current()`, exact match |
| Projection Source Grammar | v0 only |

No range, "latest", or fallback negotiation. A mismatch anywhere here is a
Compatibility rejection (`projection_bundle_v0.md` §8).

## 4. Size and count limits

Fixed in `gate_d_reference_contour_limits()` (`prom-ui/src/shell_bridge.rs`),
not caller-configurable:

| Limit | Value |
| --- | --- |
| Max bundle bytes | 262,144 (256 KiB) |
| Max binding / action-route / collection-anchor count | 256 each |
| Max accessibility-entry count | 512 |
| Max Static UI IR input bytes | 262,144 (256 KiB) |
| Max surfaces / nodes / children per node | 16 / 512 / 64 |
| Max role bytes | 64 |

Exceeding any limit is a Resource rejection, checked before the
corresponding data is traversed.

## 5. Trusted verification inputs

No cryptographic algorithm is selected or implemented. Trust here is
**deterministic self-consistency only** — canonicalize the decoded
candidate, re-encode it, compare byte-for-byte against the input
(`projection_bundle_v0.md` §§11, 17). This is not authenticity, integrity,
or admission proof; digest/signature algorithm ownership remains
unresolved and no crypto dependency exists in this workspace.

## 6. Supported surface

- **Patch operations:** unchanged from the existing Shell Player contract —
  `SetBindingValue`, `SetNodeAvailability`, `CollectionInsert`,
  `CollectionUpdate`, `CollectionRemove`, `CollectionMove`, applied only
  through `apply_prepared_patch_submission` /
  `ShellSession::apply_projection_patch_batch`.
- **Renderer:** `Clear`, `FillRect`, `DrawText` (`prom_ui_runtime::DrawCommand`)
  through the existing wgpu pipeline plus the glyph-rendering addition
  landed under #1543. General layout realization and arbitrary shape
  primitives are out of scope.
- **Input:** keyboard/pointer events already translated by the winit
  integration, deterministic hit-testing, and one bounded local
  focus-index model (Tab/Shift+Tab, Enter/Space). Pointer capture is
  limited to the single active gesture already modeled by
  `InteractionIntentKind`. Multi-touch, gesture recognition, and IME
  composition are out of scope.
- **Accessibility:** deterministic evidence derived from the activated
  bundle's Static UI IR and Role Dictionary
  (`derive_accessibility_entries`, `BridgeAccessibilityEntry`) — one role
  and one label per node, available as structured data. Wiring this into
  a platform accessibility tree (`accesskit` or equivalent) is a
  separate, out-of-scope integration; live regions, value ranges, and
  custom actions aren't implemented.

None of the above is widened by Gate D activation itself.

## 7. Excluded contours

Activation under this policy does not authorize: general ProjectionBundle
Level 4/5 reader/parser behavior; cryptographic trust verification;
critical/pinned bundle handling; unrestricted admission/dispatch
integration; action authorization beyond the existing admission boundary
this task wires into; Semantic truth ownership or mutation; production
promotion beyond UI-DNA2-11's recorded decision; Workbench/Semantic
Studio/`ui-shell-kit` integration; or any contour beyond the one bounded
reference application.

## 8. Relationship to existing invariants

This policy doesn't weaken or duplicate any frozen Shell Player invariant.
It runs strictly before `create_shell_session`; once activation succeeds,
the resulting `ActivationTargetSnapshot` flows into the existing,
unmodified Shell Player pipeline (stages 1-9, `PreparedPatchSubmission`,
`ActiveProjectionTargetCatalog`, atomic commit, replay-cursor advancement)
exactly as `shell_player_session_state_v0.md` already specifies.

```text
Gate D activation != Shell Player stage 1-9 transition evaluation
Gate D activation != patch admission != action admission
bundle activation != production promotion
```
