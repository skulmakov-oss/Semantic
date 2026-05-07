---
name: semantic
description: Use for the skulmakov-oss/Semantic repository when editing Semantic source, sm-front, sm-sema, sm-ir, sm-emit, sm-verify, sm-runtime-core, sm-vm, smc-cli, SemCode, verifier admission, VM execution, Quad Logic, runtime ownership, quotas, PROMETHEUS ABI/capability/gate/runtime/state/rules/audit crates, UI contract maps, docs/spec, roadmap status, tests, or PR planning. Do not use for unrelated generic Rust cleanup.
---

ROLE: conservative Semantic implementation agent.

REPO:
- Project: Semantic Language.
- Main package: semantic_language.
- Core pipeline:
  source -> frontend -> sema -> IR -> SemCode -> verify -> VM -> PROMETHEUS boundary.
- Public contract source of truth:
  docs/spec/*
- Status vocabulary source of truth:
  docs/roadmap/public_status_model.md

CORE_CRATES:
- sm-front: frontend / parser / source surface.
- sm-sema: semantic analysis / diagnostics.
- sm-ir: IR and canonical SemCode format ownership.
- sm-emit: producer-facing SemCode facade; do not fork format ownership here.
- sm-verify: SemCode admission gate.
- sm-runtime-core: shared runtime vocabulary / quotas.
- sm-vm: deterministic verified SemCode execution.
- smc-cli: canonical public CLI owner.

PROMETHEUS_CRATES:
- prom-abi: host ABI vocabulary.
- prom-cap: capability policy.
- prom-gates: gate descriptors / binding.
- prom-runtime: runtime session orchestration.
- prom-state: semantic state.
- prom-rules: deterministic rule agenda.
- prom-audit: audit / trace / replay records.
- prom-ui*, apps/workbench: operator/application boundary, not compiler/verifier/VM owner.

LEGACY_RULE:
- ton618-core and compatibility paths are historical/support perimeter.
- New architecture must land in the correct owner crate, not legacy shims.

MUST:
- preserve verifier-first execution.
- preserve deterministic VM semantics.
- preserve SemCode version/header discipline.
- preserve capability-gated PROMETHEUS boundary.
- preserve auditability for external effects.
- preserve Quad Logic as N/F/T/S, not bool.
- keep docs/spec synchronized with public contract changes.
- keep release/status wording honest.
- make one logical change per PR.
- add tests when behavior changes.

MUST_NOT:
- execute unchecked SemCode on public routes.
- bypass sm-verify as a public execution path.
- make sm-vm replace verifier admission.
- make sm-verify into parser, optimizer, runtime, or executor.
- fork SemCode format between sm-ir and sm-emit.
- silently reinterpret unsupported SemCode headers.
- repurpose capability bits without version/compat review.
- add direct host effects inside Semantic core.
- smuggle capability policy into VM core by implication.
- collapse Quad Logic N or S into bool silently.
- widen public release claims because code landed on main.
- claim unimplemented or unpromised behavior as stable.
- do broad rewrites when a narrow fix is enough.

STATUS_RULE:
- Published stable: only behavior promised by stable line.
- Qualified limited release: only behavior admitted by evidence.
- Landed on main, not yet promised: implemented but not release-promised.
- Out of scope: intentionally excluded.
- Landed on main does not mean stable.
- If docs conflict on status, treat it as a readiness defect.

SEMCODE_RULES:
- sm-ir owns SemCode header/opcode/capability contract.
- sm-emit exposes producer-facing entrypoints over sm-ir contract.
- sm-verify is required admission before standard execution.
- sm-vm consumes verified SemCode.
- Header/layout/opcode/capability meaning changes require:
  spec update,
  verifier update,
  VM compatibility check,
  tests/golden fixtures where public behavior changes.

VERIFIER_RULES:
- Standard route:
  emit SemCode -> verify_semcode -> execute.
- Verifier checks structure/admission, not runtime policy execution.
- Reject diagnostics must be deterministic for same input.
- Verifier changes need positive and negative tests.
- Public verifier behavior changes require docs/spec/verifier.md sync.

VM_RULES:
- Standard public route:
  verify -> run_verified_semcode* -> execute.
- VM must be deterministic for same verified SemCode, config, and entry.
- VM owns execution, frames, registers, quotas, safe failure reporting.
- VM does not own SemCode binary contract.
- VM does not own capability policy semantics.
- VM must surface verifier rejection distinctly from runtime failure.
- VM must not accept malformed bytecode by best effort.

QUAD_RULES:
- Quad values:
  N = unknown
  F = false
  T = true
  S = conflict
- Source branch control must be explicit.
- Do not silently treat unknown as false.
- Do not erase conflict.
- Any bool conversion must be explicit and tested.

PROMETHEUS_RULES:
- Semantic core remains deterministic and effect-controlled.
- PROMETHEUS boundary owns external interaction.
- Capability, gate, ABI, runtime state, rules, and audit concerns belong in prom-* owner crates.
- UI is an operator/application boundary, not semantic owner.

RUNTIME_OWNERSHIP_RULES:
- Current frozen slice:
  tuple AccessPath,
  direct record-field AccessPath,
  Borrow/Write events,
  OWN0,
  SEMCOD11 tuple ownership,
  SEMCOD12 direct record-field ownership,
  frame-local borrow lifetime,
  overlap write rejection.
- Do not imply support for ADT payload paths, schema paths, partial release, inter-frame borrow persistence, advanced aliasing, or indirect projection unless explicitly implementing and documenting it.

TASK_START:
1. Identify layer:
   frontend | sema | IR | emit | verify | runtime-core | VM | cli | prom-* | UI | docs | tests | legacy.
2. Identify contract impact:
   none | internal | public spec | release/status | SemCode compatibility | verifier admission | VM execution | capability/audit.
3. Identify required tests before editing.

EDIT_POLICY:
- Prefer smallest correct patch.
- Preserve owner boundaries.
- Preserve no_std / feature-gated behavior where present.
- Prefer explicit errors over silent fallback.
- Do not move behavior across crates without architectural reason.
- Do not clean unrelated code in feature/bug PRs.
- If cleanup reveals feature work, split it into a separate PR/issue.

TEST_POLICY:
- verifier change => positive + negative admission tests.
- SemCode layout/header/capability change => golden/compat fixtures.
- VM behavior change => deterministic execution tests.
- quota change => quota limit/usage tests.
- runtime ownership change => overlap and allowed-sibling tests.
- prom-* capability/audit change => denial + audit path tests.
- docs-only status change => check public status vocabulary.
- no_std-sensitive change => cargo check --no-default-features where relevant.

PREFERRED_COMMANDS:
- cargo fmt --check
- cargo test -q
- cargo test -q --test public_api_contracts
- cargo test -q --test runtime_ownership_e2e
- cargo check --no-default-features
- cargo run --bin smc -- check <file.sm>
- cargo run --bin smc -- compile <file.sm> -o <file.smc>
- cargo run --bin smc -- verify <file.smc>
- cargo run --bin smc -- run <file.sm>
- cargo run --bin smc -- run-smc <file.smc>
- cargo run --bin svm -- disasm <file.smc>

OUTPUT:
Summary:
Layer:
Contract impact:
Invariants checked:
Changes:
Tests:
Risks:
Next:
