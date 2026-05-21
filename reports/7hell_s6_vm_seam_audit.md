# 7HELL-S6 VM Seam Audit

Status: audit-only
Scope: locate VM-stage seam before execution
Non-goal: implementation, project-root, host effects, readiness, or CTF closure

## Findings

### VM API seams

- `sm_vm::run_verified_semcode(bytes)` is the narrowest safe VM entry for 7hell S6. It does not require host capability plumbing, does not expose controlled observations, and internally enforces verifier-first admission before execution.
- `sm_vm::run_semcode(bytes)` is not the preferred seam for 7hell because it runs raw SemCode under `ExecutionContext::VerifiedLocal` without enforcing verifier-first at the callsite.
- `sm_vm::run_semcode_collecting_hello_observations(bytes)` is too broad for S6 VM-only coverage because it collects host-visible observation events.
- `sm_vm::run_verified_semcode_with_host_and_capabilities(...)` and `sm_vm::run_verified_semcode_with_ui_capabilities(...)` are explicitly broader than 7hell VM stage needs because they pull in host/capability boundaries.

Stable runtime signals available from the VM layer:

- `RuntimeError::Trap(RuntimeTrap::...)` is already a stable runtime failure surface for VM traps.
- `ExecutionConfig::for_context(ExecutionContext::VerifiedLocal)` uses deterministic verified-local quotas.

### CLI run routes

- `cmd_run` compiles source and then routes through `render_controlled_observation_envelope(bytes)`, which combines verifier admission, VM observation collection, capability policy, and audit trail rendering. Too broad for 7hell S6.
- `cmd_run_smc` is the same controlled-observation envelope on prebuilt `.smc` bytes. Also too broad for 7hell S6.
- `render_controlled_observation_envelope(bytes)` is not a VM-only seam; it explicitly performs verifier admission, controlled observation collection, capability gating, and audit rendering.
- `cmd_verify` is verifier-only and does not help with the VM stage by itself.

### Runtime context / quotas

- `ExecutionContext::VerifiedLocal` is sufficient for a future 7hell VM stage that stays source-local and does not require host/capability widening.
- `RuntimeQuotas::verified_local()` is deterministic and already used by the verified-local path.
- Runtime traps are classified through `RuntimeTrap` and surfaced via `RuntimeError::Trap(...)`; that is stable enough for a VM-trap JSON classification.

### Output / observation boundary

- `run_semcode_collecting_hello_observations(bytes)` is deterministic for sequence ordering, but it is an observation route, not a pure VM seam.
- Any host-visible observation should be treated as Practical Hell, not as the VM stage itself.
- For 7hell S6, the VM stage should remain silent: validate execution outcome and traps, but do not use the controlled-observation envelope.

### Safe candidate route

Future S6 route can stay narrow:

```text
read source as UTF-8
-> semantic_check_source
-> compile source to SemCode with fixed options
-> verify_semcode(bytes)
-> run_verified_semcode(bytes)
-> VM Hell PASS/FAIL
-> Practical Hell remains BLOCKED
-> result remains INCOMPLETE on VM success
```

This is safe only for fixtures that do not require host-visible output. If a fixture needs controlled observations or other host-facing behavior, that belongs in a separate seam/refactor, not in S6.

### Rejected routes

- `smc_cli::run(["run", ...])`
- `cmd_run`
- `cmd_run_smc`
- shelling out to `smc run`
- temp `.smc` files
- cache-pack routes
- controlled-observation envelope as the 7hell VM report path
- project-root / `semantic.toml`
- timing/metrics in stable output
- absolute-path leakage
- treating VM success as final release pass

### Required S6 guardrails

- S6 may run VM only after Syntax/Type/Lowering/Verifier pass.
- S6 must keep `target.kind = "single-file"`.
- S6 must keep `--project` rejected.
- S6 must not write `.smc`.
- S6 must not use cache routes.
- S6 output must remain deterministic.
- S6 must not expose host output as Practical Hell.
- S6 must not claim final PASS while Practical remains blocked.
- S6 must add snapshots for VM success and VM runtime traps.
- S6 must classify runtime traps as `vm-trap`, not verifier rejection.
- S6 must not reuse verifier rejection diagnostics for VM failure.

## Verdict

S6 verdict: GO
Reason:
Safe VM seam exists for silent, verified single-file fixtures:
- VM API: `sm_vm::run_verified_semcode(bytes)`
- execution context: `ExecutionContext::VerifiedLocal`
- output policy: no controlled-observation envelope for S6; host-visible output stays out of VM stage
- no project-root: reject `cmd_run` / `cmd_run_smc` / package admission routes
- no host-effect leak: avoid host/capability variants and observation collection
- expected future PR: `7HELL-S6 — cli(7hell): add VM stage execution for selected verified single-file fixtures`

If a future fixture needs host-visible observations, that is a separate seam/refactor and not the VM-only S6 route.
