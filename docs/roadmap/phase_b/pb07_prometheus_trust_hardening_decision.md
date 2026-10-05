# PB-07 — PROMETHEUS Trust Hardening Decision

Umbrella: #1617 · Base: `9e79f5f2aba1e830795948da02669c09abe75e43`
Issues: #1778, #1779, #1780, #1781, #1782, #1783, #1784, #1785, #1786, #1787
C1 `89641da8237f4fcefb50cf1958a50e4d4003aea7` / `v1.2.0` are not touched.
Out of scope: #1987.

Principle:

- an invalid ABI value cannot acquire semantic meaning;
- an invalid capability contract is a distinct typed failure;
- an invalid snapshot cannot become trusted state;
- an unsupported rule plan has zero prefix effects;
- authority metadata comes from the authority object itself;
- invalid audit evidence cannot be called canonical;
- a hostile declared count is a typed error, never an allocation panic.

## Re-adjudication on current main (`9e79f5f2`)

| Issue | Base evidence | Classification |
|---|---|---|
| #1778 | `prom_abi::AbiValue::Quad(u8)` represents 256 states. `sm-vm::quad_from_abi` already rejects bytes ≥ 4 (#1775), so the defect sits at the ABI vocabulary boundary, not the VM | STILL_REPRODUCED |
| #1779 | `ApplicationHostAbi` docs say a read is "captured by a replay layer before a later write is admitted". `ApplicationVmHost` tracks only `observed: bool`, set after a successful host read, and its error says "preceding captured observation". No replay substrate exists at this seam | STILL_REPRODUCED (overclaiming contract) |
| #1780 | `CapabilityManifest::require` maps a `validate()` failure to `CapabilityDeniedCode::MissingCapability`. `require_hello_observation_sink_capability` maps every checker error to `MissingObservationCapability` | STILL_REPRODUCED |
| #1781 | both Hello evaluators contain `Some(_) => Deny(GenericIoNotAllowed)`, and `unknown_host_channel_is_denied` exists | RESOLVED_BY_PRIOR_CHANGE: PR #1973, squash `19c1fb8b` (2026-09-30), replaced the `_ => {}` fall-through |
| #1782 | `SemanticStateStore::restore` assigns directly. `StateSnapshotArchive::from_canonical_text` accepts single-value `uncertain`, key/record mismatches and record epochs beyond the snapshot. `apply_rollback` installs checkpoint records unvalidated | STILL_REPRODUCED |
| #1783 | `apply` computes `StateEpoch(self.epoch.0 + 1)` unchecked; restore and archive accept `u64::MAX` | STILL_REPRODUCED |
| #1784 | `apply_rule_state_write_effects` / `apply_rule_audit_note_effects` discover an unsupported family inside the effect loop, after earlier effects committed | STILL_REPRODUCED |
| #1785 | `ExecutionSession::{new,kernel_bound}` and `GateExecutionSession::{new,kernel_bound}` take the checker and a `CapabilityManifestMetadata` as independent arguments | STILL_REPRODUCED |
| #1786 | `AuditReplayArchive::to_canonical_text` / `MultiSessionReplayArchive::to_canonical_text` never validate. `replay.session` is not serialized and the reader reconstructs it from `session`, so an A/B provenance contradiction round-trips as A/A | STILL_REPRODUCED |
| #1787 | `Vec::with_capacity(expected_event_count)` / `Vec::with_capacity(expected_session_count)` run on an untrusted parsed count before any structural proof | STILL_REPRODUCED |

Same-root findings repaired with their owner issue:

- `prom-state::decode_resolution` computes `count + 2` on a persisted count, which overflows at `usize::MAX`. It is repaired under #1782 with checked arithmetic, and the capacity is then structurally bounded by the payload length.

Out-of-root finding, reported and not repaired:

- `prom_audit::AuditTrail::record` uses unchecked `next_id += 1`. No PB-07 path depends on it; it is a new residual.

## Answers

1. **What is the canonical ABI quad representation?** `prom_abi::AbiQuad { N = 0, F = 1, T = 2, S = 3 }` (`#[repr(u8)]`), and `AbiValue::Quad(AbiQuad)`. The safe public API can no longer construct a malformed quad.
2. **Where can a raw `u8` become `AbiQuad`?** Only through `TryFrom<u8> for AbiQuad`. It maps `0..=3` and rejects everything else with `InvalidAbiQuad(raw)`; there is no masking and no fallback. `AbiQuad::as_u8` is the inverse.
3. **Why does `sm-vm` stay non-normalizing?** It converts `AbiQuad` ↔ `QuadVal` with exhaustive matches and no arithmetic. The old low-bit mask cannot return because no raw byte reaches this boundary any more.
4. **What does `ApplicationHostAbi` guarantee?** Its methods are host-bound observations and host writes. The runtime checks the matching capability before each call. The generic VM bridge admits a write only after at least one preceding *successful* host observation in the same execution. A failed read establishes nothing.
5. **What does it not prove about replay capture?** That any observation was persisted to a replay or audit substrate. A concrete orchestration or replay layer that claims replay capture must prove it separately. Docs and the VM error now say "preceding successful host observation".
6. **How are invalid capability contracts typed?** `CapabilityDeniedCode::InvalidManifestContract(ManifestValidationCode)`. `require_call` keeps that code and attaches the `HostCallId`. The Hello helper maps it to `HelloObservationCapabilityDenial::InvalidCapabilityContract`.
7. **Why is a missing grant distinct from an invalid manifest?** A malformed policy document cannot be interpreted at all. A valid policy that lacks a grant is an interpreted "no". Callers must be able to tell them apart without parsing `message`.
8. **Why is #1781 already resolved?** PR #1973 (`19c1fb8b`) made every unknown `Some(_)` channel `Deny(GenericIoNotAllowed)` in both evaluators. PB-07 adds direct evidence (`socket`, `udp`, `stderr2`, an arbitrary string, and `None` reaching the normal checks) without touching production code.
9. **What is the single snapshot validator?** `StateSnapshot::validate`. It is built on the same `validate_resolution` / non-empty helpers that `apply` uses for updates.
10. **Which ingress and egress paths invoke it?**
    - `SemanticStateStore::restore`, which is now `Result`
    - `StateSnapshotArchive::from_canonical_text`, after structural decode
    - `StateSnapshotArchive::to_canonical_text`, which is now `Result`
    - `apply_rollback`, on the selected checkpoint before any mutation, via the new `StateRollbackCode::InvalidCheckpointSnapshot`
11. **What is the terminal epoch policy?** The store has no frozen mode, so a snapshot whose epoch is `u64::MAX` is invalid (`EpochExhausted`) at every snapshot boundary. `apply` keeps an independent checked advance.
12. **What happens at `u64::MAX`?** `apply` returns `StateValidationCode::EpochExhausted` before any mutation: no panic, no wrap, no saturation.
13. **What does effect-family preflight guarantee?** One helper scans the whole plan before the first effect. An unsupported family returns `UnsupportedEffectFamily` at its first ordinal, with state, epoch, audit trail and agenda unchanged.
14. **Which admitted-plan failure semantics stay unchanged?** An admitted homogeneous plan whose later effect fails semantic validation keeps the existing per-effect semantics: earlier effects stay committed. PB-07 is not a transaction engine.
15. **Where does capability provenance come from?** `CapabilityChecker::manifest_metadata(&self)`, a required method with no default. `CapabilityManifest` returns `self.metadata()`. Session constructors derive the descriptor from the checker.
16. **Can caller metadata disagree with checker metadata?** No. The independent constructor parameter is removed, and the public API snapshot guards against it returning.
17. **What makes an audit archive canonical?**
    - Single session:
      - format version is exact
      - `replay.session == session`
      - `replay.event_count == events.len()`
      - `replay.last_event_id == events.last().id`
      - event ids are exactly `0..N`
    - Multi-session:
      - format version is exact
      - session ordinals are exactly `0..N`
      - every embedded archive is canonical
18. **Can writer and reader disagree on invariants?** No. `AuditReplayArchive::validate` / `MultiSessionReplayArchive::validate` own the semantic invariants. Writers call them before serializing, and readers call them on the parsed candidate.
19. **How are hostile declared counts bounded?** The count must first be structurally possible from the remaining input:
    - events must not exceed the remaining lines minus the replay line;
    - sessions must not exceed half the remaining lines, since each session takes at least two.

    Only then does `Vec::try_reserve` run, and its failure is a typed format error. There is no magic cap. The embedded archive-line count consumes input line by line and fails typed when the input runs out.
20. **Why was no format version bump required?** Valid archives serialize to byte-identical text. Only invalid evidence changed, from silently accepted to rejected.
21. **Which public API changes intentionally differ from C1?**
    - `AbiValue::Quad(AbiQuad)`, and the new `AbiQuad` / `InvalidAbiQuad`
    - `CapabilityDeniedCode::InvalidManifestContract`
    - `HelloObservationCapabilityDenial::InvalidCapabilityContract`
    - `CapabilityChecker::manifest_metadata`
    - session constructors no longer take metadata
    - `SemanticStateStore::restore -> Result`
    - `StateSnapshot::validate`
    - `StateValidationCode::{KeyMismatch, RecordEpochAfterSnapshot, EpochExhausted}`
    - `StateRollbackCode::InvalidCheckpointSnapshot`
    - `StateSnapshotArchive::to_canonical_text -> Result`
    - `AuditReplayArchive::{validate, to_canonical_text -> Result}`
    - `MultiSessionReplayArchive::{validate, to_canonical_text -> Result}`

## Non-goals

- No new replay engine.
- No new audit storage or state model.
- No capability grant language.
- No rule transaction or rollback semantics.
- No `#1987` work.
- No release or tag work.
