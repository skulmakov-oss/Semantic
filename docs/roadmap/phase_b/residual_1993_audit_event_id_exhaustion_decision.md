# Residual #1993 — Audit Event Identity Exhaustion Decision

- Post-Phase-A residual cleanup. It is not a Phase-B slice and does not reopen PB-07.
- Umbrella: #1617 · Issue: #1993
- Base: `1065f3e9a011af7ab707dc6ba7000ff268998a1e`
- C1 `89641da8237f4fcefb50cf1958a50e4d4003aea7` / `v1.2.0` are not touched.

## Re-adjudication on current main (`1065f3e9`): STILL_REPRODUCED

`AuditTrail::record` computed `AuditEventId(self.next_id)` and then ran `self.next_id += 1`. A unit probe with the private cursor at `u64::MAX - 1` showed:

- The record that issues `AuditEventId(u64::MAX)` panics in a debug build at the `+= 1` ("attempt to add with overflow").
- In a wrapping build the same record appends the `u64::MAX` event and resets the cursor to 0, so the next record re-issues `AuditEventId(0)` and identity is reused.
- Neither outcome is a typed audit error.

## Answers

1. **Who owns live audit event identity?** `prom-audit::AuditTrail`, through a private `AuditEventIdCursor`. No caller guesses, increments, wraps or repairs an id.
2. **What is the event-id domain?** The whole public `AuditEventId(u64)` range, `0..=u64::MAX`, each issued exactly once in order.
3. **Is `u64::MAX` issuable?** Yes. It is the last identity, not a sentinel.
4. **How is the exhausted state represented?** As `next: Option<u64>`. `None` means exhausted, and advancing past `u64::MAX` uses `checked_add`, which yields `None`.
5. **What typed error represents exhaustion?** `AuditTrailError::EventIdExhausted`, with `Display` and `std::error::Error` under `std`. It is the only variant.
6. **Does a failed `record` change events?** No. The identity is admitted (`peek`) before the append.
7. **Does a failed `record` change the cursor?** No. It stays exhausted, and `commit` runs only after a successful append.
8. **How can callers preflight N events?** With `AuditTrail::ensure_record_capacity(n)`. It is side-effect free, uses checked `usize`→`u128` conversion and `u128` remaining-capacity arithmetic, and `n = 0` always returns `Ok`.
9. **How does the runtime avoid mutating state before audit admission?** `apply_update_refresh_agenda` admits one event before `state.apply`. On exhaustion it returns `RuntimeStateAdvanceError::Audit` with state, epoch, transitions and audit all unchanged. State and audit failures keep their own typed owners (`StateValidation` / `Audit`).
10. **How do multi-note batches avoid prefix commits on exhaustion?** After the effect-family preflight (#1784), the state-write and audit-note slices admit capacity for the whole admitted plan before the first effect. Exhaustion yields `RuleEffectExecutionCode::AuditRecordingFailed` with zero prefix effects. `smc-cli` likewise admits the whole controlled-observation batch before its loop.
11. **How does controlled-observation audit propagate exhaustion?** `apply_controlled_observation_audit_policy` returns `Result<ControlledObservationAuditResult, AuditTrailError>`. Exhaustion is an error, never disguised as `NoStore` or `Denied`, and `smc-cli` renders it as a message only at its presentation boundary.
12. **Which APIs became fallible?**
    - `AuditTrail::record`
    - `apply_controlled_observation_audit_policy`
    - the runtime `record_session_started` / `record_session_finished` / `record_rule_activation` / `record_state_transition` helpers
    - `apply_state_update_and_refresh_agenda`, whose error type is now `RuntimeStateAdvanceError`
13. **Why is this not a transaction framework?**
    - It is one admission check before visible effects, sized by a count the caller already knows.
    - There is no rollback, reservation registry or global transaction abstraction.
    - A private `RuntimeAudit` trait implemented by `AuditTrail` lets the ordering be tested without any public counter seam.
14. **Why is no wire or version change needed?**
    - `AuditEventId(pub u64)` and the archive format are unchanged.
    - Valid single- and multi-session canonical text is byte-identical to `1065f3e9` (full escaped bodies compared).
15. **What intentionally differs from C1?** Recording past the last identity now fails typed instead of panicking or wrapping, and the listed APIs return `Result`.
16. **What remains unchanged from PB-07?** Archive canonicality (`validate`, fallible canonical writers), bounded archive loading, effect-family preflight and checker-owned provenance.

## Non-goals

No magic event cap, no reuse of `RuntimeQuotas` for audit identity, no archive redesign, and no transaction framework.
