# 7HELL-S7 Practical / Observation Seam Audit

Status: audit-only
Scope: locate Practical-stage seam before execution
Non-goal: implementation, host-visible output, project-root, readiness, or CTF closure

## Findings

### Observation API seams

- `sm_vm::run_semcode_collecting_hello_observations(bytes)` runs VM in `ExecutionContext::VerifiedLocal` and returns ordered `HelloObservationEvent` values in memory.
- `smc_cli::render_controlled_observation_envelope(bytes)` is the current higher-level route, but it is not a pure qualification seam: it verifies, runs the VM, evaluates capability, applies audit policy, and then prepares rendered lines for stdout.
- `smc run <input.sm>` compiles source, then calls `render_controlled_observation_envelope`, then prints rendered lines.
- `smc run-smc <input.smc>` reads SemCode bytes, then calls `render_controlled_observation_envelope`, then prints rendered lines.

### CLI run routes

- `cmd_run` is too broad for direct `7hell` reuse because it combines source admission, compile, VM execution, capability policy, audit, and host-visible rendering.
- `cmd_run_smc` is also too broad because it combines `.smc` input, VM execution, capability policy, audit, and host-visible rendering.
- Neither route is a safe Practical-stage seam for `7hell` as-is.

### Capability boundary

- Controlled observation requires `CapabilityKind::ControlledObservationSink`.
- The capability decision path is deterministic in memory, but in the current CLI route it is embedded inside the render envelope.
- Missing capability is represented by `HelloObservationCapabilityDecision::Deny(...)`; there is no dedicated `7hell` Practical report mode that surfaces this without the render path.

### Audit boundary

- `apply_controlled_observation_audit_policy` is deterministic and records monotonic event IDs in `AuditTrail`.
- The current `render_controlled_observation_envelope` path creates an `AuditTrail` in memory and applies policy, but it is coupled to rendering rather than exposed as a standalone qualification summary.
- That makes it suitable for tests, but not yet a narrow `7hell` Practical seam.

### Output policy

- The current route renders `rendered_lines` and prints them in `cmd_run` / `cmd_run_smc`.
- For `7hell`, that is host-visible output and therefore too broad for Practical qualification.
- `7hell` should not reuse raw rendered lines as its Practical report surface.

### Safe candidate route

- A safe future `S7` route would need a new non-rendering practical qualification function that returns a structured in-memory summary.
- That summary should separate:
  - observation collection,
  - capability decision,
  - audit decision,
  - practical qualification result,
  - and any host-visible rendering.

### Rejected routes

- Reject `cmd_run`.
- Reject `cmd_run_smc`.
- Reject shelling out to `smc`.
- Reject direct reuse of `render_controlled_observation_envelope` for `7hell`.
- Reject output that exposes rendered observation lines as the `7hell` Practical stage surface.
- Reject `.smc` temp-file routes.
- Reject cache-pack routes.
- Reject project-root / `semantic.toml`.
- Reject timing, metrics, and absolute-path output.
- Reject any route that treats Practical success as final release readiness.

### Required S7 guardrails

- `S7` may run Practical only after Syntax / Type / Lowering / Verifier / VM pass.
- `S7` must keep `target.kind = "single-file"`.
- `S7` must keep `--project` rejected.
- `S7` must not write `.smc`.
- `S7` must not use cache routes.
- `S7` output must be deterministic.
- `S7` must not expose raw host stdout.
- `S7` must not persist external audit state.
- `S7` must distinguish VM trap, capability denial, audit denial, and observation ordering failure.
- `S7` must not claim release readiness or CTF closure.

## Verdict

S7 verdict: STOP
Reason:
Safe Practical/observation seam not found.
Blocking seam:
- the existing practical route is coupled to host-visible rendering and the broader `cmd_run` / `cmd_run_smc` envelope.
Suggested next split:
`7HELL-S7-SEAM — refactor(cli): expose non-rendering practical qualification envelope`
