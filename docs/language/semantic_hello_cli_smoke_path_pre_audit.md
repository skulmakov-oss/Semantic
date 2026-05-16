# Semantic Hello CLI Smoke Path Pre-Audit

Status: inspection-only readiness audit for `#477`

Implementation note: `M-HELLO-12A-1`, `M-HELLO-12A-2`, `M-HELLO-12A-3`,
and `M-HELLO-12A-4` are now implemented as a VM-side non-output controlled
observation event seam, a verifier-side controlled observation admission
seam, a production capability gate seam, and a production audit decision
/ storage seam. The broader CLI controlled observation path is still not
ready.

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
| VM route | emits internal `ControlledObservationEvent` | current VM builtin `print` now records internal controlled observation events in memory without direct stdout; the seam is still isolated from verifier / capability / audit / CLI layers | yes | wire the VM seam into the later verifier / capability / audit / CLI chain |
| capability gate | explicit controlled sink allow / deny | `prom-cap` now exposes a `ControlledObservationSink` capability and a production-manifest-aware helper, but the runtime observation route is still not consuming it | partial | wire the production capability gate into the later observation route |
| audit policy | `record` / `redact` / `no_store` / `deny` decision | `prom-audit` now can represent controlled observation audit decisions and archive them deterministically, but CLI rendering is still blocked | yes | wire the audit seam into the later observation route / CLI envelope |
| CLI source-run | `smc run` uses full controlled route | `smc run` compiles source and runs bytes directly; it does not consume a controlled observation result envelope | no | add a source-run route that only renders approved controlled observation results |
| CLI artifact-run | `run-smc` uses full controlled route | `run-smc` verifies then runs bytes directly; it still does not consume a controlled observation result envelope | no | add a verified-artifact route that only renders approved controlled observation results |

## 4. Current Bypass Risks

Observed risks in code:

- `CAP_STDOUT` is still the verifier capability for builtin `print`
- isolated hello capability / audit modules are not wired into production routing
- `smc run` compiles source and runs bytes directly without a controlled observation envelope
- `run-smc` verifies and then runs bytes directly without a controlled observation envelope
- CLI output is ordinary text output, not controlled observation rendering

Evidence:

- `crates/sm-verify/src/lib.rs:1066-1072` maps builtin `print` to `CAP_STDOUT`
- `crates/smc-cli/src/app.rs:2169-2176` makes `smc run` compile source and call `run_semcode`
- `crates/smc-cli/src/app.rs:2197-2203` makes `run-smc` verify and then call `run_verified_semcode`
- `crates/prom-cap/src/hello_observation_capability.rs` remains an isolated skeleton, not production capability wiring
- `crates/prom-audit/src/hello_observation_audit.rs` remains an isolated skeleton, not production audit storage wiring

## 5. Required Implementation Split

Based on the current code state, the next narrow split should be:

- `M-HELLO-12A-1` - done: VM-side non-output controlled observation event seam
- `M-HELLO-12A-2` - done: verifier-side controlled observation admission seam
- `M-HELLO-12A-3` - done: production capability gate for controlled observation sink
- `M-HELLO-12A-4` - done: audit decision policy wiring for controlled observation
- `M-HELLO-12A-5` - add CLI result envelope rendering for source-run and run-smc separately

## 6. No-Go Decision

`M-HELLO-12A-code is not ready.`

Lower-layer production seams are incomplete or still isolated, even though
`M-HELLO-12A-1`, `M-HELLO-12A-2`, `M-HELLO-12A-3`, and `M-HELLO-12A-4`
are now implemented as narrow seams.

The current code has separate `smc run` and `run-smc` commands, but neither
command is an honest controlled observation CLI implementation yet.

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
- [ ] no code changed
- [ ] `#477` remains open
