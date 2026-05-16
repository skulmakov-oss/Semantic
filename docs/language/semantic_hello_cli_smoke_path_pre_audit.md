# Semantic Hello CLI Smoke Path Pre-Audit

Status: inspection-only readiness audit for `#477`

Implementation note: `M-HELLO-12A-1`, `M-HELLO-12A-2`, `M-HELLO-12A-3`,
`M-HELLO-12A-4`, and `M-HELLO-12A-5` are now implemented as a VM-side
non-output controlled observation event seam, a verifier-side controlled
observation admission seam, a production capability gate seam, a production
audit decision / storage seam, and CLI result envelope rendering for
`smc run` and `run-smc`. The narrow Hello controlled observation route is
functionally present, but the broader `#477` closeout is still pending final
acceptance verification.

See also:

- [`semantic_hello_cli_smoke_path.md`](semantic_hello_cli_smoke_path.md)
- [`semantic_hello_observation_audit_policy.md`](semantic_hello_observation_audit_policy.md)
- [`semantic_hello_observation_capability_gate.md`](semantic_hello_observation_capability_gate.md)
- [`semantic_hello_vm_observation_execution_route.md`](semantic_hello_vm_observation_execution_route.md)
- [`semantic_hello_observation_admission_shape.md`](semantic_hello_observation_admission_shape.md)
- [`semantic_hello_observation_admission_runtime_path.md`](semantic_hello_observation_admission_runtime_path.md)
- [`semantic_hello_controlled_observation_encoding.md`](semantic_hello_controlled_observation_encoding.md)

## 1. Purpose

This document audits readiness for an honest CLI controlled observation
implementation.

It is docs-only and does not implement CLI output.

## 2. Scope

Inspected owners:

- `sm-verify`
- `sm-vm`
- `prom-cap`
- `prom-audit`
- `smc-cli`
- `sm-emit` Hello provisional path

## 3. Readiness Matrix

| Layer | Required for honest 12A-code | Current state | Ready? | Next action |
| --- | --- | --- | --- | --- |
| verifier admission | admits `ControlledTextObservation` shape | Hello-specific controlled observation admission seam now exists in `sm-verify`, but production `verify_semcode` still maps builtin `print` to `CAP_STDOUT` | partial | wire the verifier seam into the later production admission chain |
| VM route | emits internal `ControlledObservationEvent` | current VM builtin `print` now records internal controlled observation events in memory without direct stdout | yes | wire the VM seam into the later verifier / capability / audit / CLI chain |
| capability gate | explicit controlled sink allow / deny | `prom-cap` now exposes a `ControlledObservationSink` capability and a production-manifest-aware helper | yes | keep the explicit controlled sink capability narrow |
| audit policy | `record` / `redact` / `no_store` / `deny` decision | `prom-audit` now can represent controlled observation audit decisions and archive them deterministically | yes | keep audit representation deterministic and bounded |
| CLI source-run | `smc run` uses full controlled route | `smc run` now compiles, verifies, collects controlled observations, applies capability / audit policy, and renders approved payloads for the narrow Hello route | yes | keep the source-run envelope narrow and tested separately |
| CLI artifact-run | `run-smc` uses full controlled route | `run-smc` now verifies, collects controlled observations, applies capability / audit policy, and renders approved payloads for the narrow Hello route | yes | keep the artifact-run envelope narrow and tested separately |

## 4. Current Bypass Risks

Observed risks in code:

- `CAP_STDOUT` is still the verifier capability for builtin `print`
- production verifier admission is still not fully aligned with the controlled observation admission seam
- the narrow CLI smoke path is implemented, but it is still only a Hello-specific envelope and not a broad stdout API
- README / examples promotion is still out of scope

Evidence:

- `crates/sm-verify/src/lib.rs:1066-1072` maps builtin `print` to `CAP_STDOUT`
- `crates/smc-cli/src/app.rs` now renders the narrow controlled observation envelope for `smc run` and `run-smc`
- `crates/prom-cap/src/hello_observation_capability.rs` and `crates/prom-audit/src/hello_observation_audit.rs` now have production-manifest-aware / production-audit seams, but the verifier admission layer still needs full alignment

## 5. Required Implementation Split

Based on the current code state, the next narrow split should be:

- `M-HELLO-12A-1` - done: VM-side non-output controlled observation event seam
- `M-HELLO-12A-2` - done: verifier-side controlled observation admission seam
- `M-HELLO-12A-3` - done: production capability gate for controlled observation sink
- `M-HELLO-12A-4` - done: audit decision policy wiring for controlled observation
- `M-HELLO-12A-5` - done: CLI result envelope rendering for source-run and run-smc separately

## 6. No-Go Decision

M-HELLO-12A-code is now functionally present for the narrow Hello controlled
observation route, but `#477` remains open until final acceptance
verification / closeout.

The current code has separate `smc run` and `run-smc` commands, and both now
render narrow controlled observation envelopes for approved Hello payloads.
The remaining gap is the broader verifier alignment for production `print`
admission and final closeout policy.

## 7. Issue State

- this document does not close `#477`
- this document does not satisfy `#477` acceptance criteria

## 8. Acceptance Checklist

- [ ] verifier seam inspected
- [ ] VM route inspected
- [ ] capability gate inspected
- [ ] audit seam inspected
- [ ] `smc run` route inspected
- [ ] `run-smc` route inspected
- [ ] bypass risks listed
- [ ] next implementation split proposed
- [ ] 12A-5 narrow CLI envelope verified
- [ ] no code changed
- [ ] `#477` remains open
