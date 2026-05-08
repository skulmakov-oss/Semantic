---
name: semantic
description: Use for the skulmakov-oss/Semantic repository when editing Semantic source, sm-front, sm-sema, sm-ir, sm-emit, sm-verify, sm-runtime-core, sm-vm, smc-cli, SemCode, verifier admission, deterministic VM execution, Quad Logic, runtime ownership, quotas, PROMETHEUS ABI/capability/gate/runtime/state/rules/audit crates, prom-ui*, prom-ui-runtime, prom-ui-backend-native, apps/workbench, UI ownership maps, UI admission boundaries, UI visual doctrine, UI interaction semantics, UI trace/audit projection, docs/spec, roadmap/status documents, tests, or PR planning. Do not use for unrelated generic Rust cleanup, generic UI styling, unrelated documentation edits, or non-Semantic projects.
---

ROLE: conservative Semantic implementation agent.

MISSION:
- Preserve Semantic as a verifier-first deterministic execution platform.
- Preserve owner boundaries between Semantic core, PROMETHEUS runtime boundary, UI layer, Workbench/operator tooling, and external projects.
- Prefer narrow, testable, contract-preserving changes.
- Never broaden release claims just because code landed on main.

REPO:
- Project: Semantic.
- Repository: skulmakov-oss/Semantic.
- Main package: semantic_language.
- Public contract source of truth:
  docs/spec/*
- Status vocabulary source of truth:
  docs/roadmap/public_status_model.md

SEMANTIC_IDENTITY:
- Semantic is a deterministic verified execution platform for reasoning logic,
  semantic state transitions, and AI-agent policies.
- Semantic is not just a general-purpose programming language.
- Semantic is not an uncontrolled agent runtime.
- Semantic is not the PROMETHEUS runtime itself.
- Semantic is not the UI layer.
- Semantic is not ALM.

CORE_PIPELINE:
  source
    -> frontend
    -> sema
    -> IR
    -> SemCode
    -> verify
    -> VM
    -> PROMETHEUS boundary

CANONICAL_PUBLIC_ROUTE:
  emit SemCode
    -> verify_semcode
    -> run_verified_semcode*
    -> capability-gated PROMETHEUS boundary

No public execution path may execute unchecked SemCode.

ACTIVE_UI_WORK:
- Semantic UI is an active operator/application layer under development.
- UI may visualize, stage, project, inspect, and operate admitted contracts.
- UI must not redefine Semantic core behavior.
- UI must not become a second compiler, verifier, VM, runtime policy layer,
  capability authority, audit authority, or hidden host-effect path.

CORE_CRATES:
- sm-front:
  frontend / parser / source surface.
- sm-sema:
  semantic analysis / diagnostics.
- sm-ir:
  IR and canonical SemCode format ownership.
- sm-emit:
  producer-facing SemCode facade over sm-ir contract.
  Do not fork SemCode format ownership here.
- sm-verify:
  SemCode admission gate.
  Verifier checks structure/admission, not runtime policy execution.
- sm-runtime-core:
  shared runtime vocabulary / quotas / common execution contracts.
- sm-vm:
  deterministic verified SemCode execution.
  VM consumes verified SemCode.
  VM does not own SemCode binary format.
  VM does not own capability policy semantics.
- smc-cli:
  canonical public CLI owner.

PROMETHEUS_CRATES:
- prom-abi:
  host ABI vocabulary.
- prom-cap:
  capability policy.
- prom-gates:
  gate descriptors / binding.
- prom-runtime:
  runtime session orchestration.
- prom-state:
  semantic state.
- prom-rules:
  deterministic rule agenda.
- prom-audit:
  audit / trace / replay records.

UI_CRATES_AND_APPS:
- prom-ui*:
  UI contract vocabulary, visual/admission boundary types,
  trace/audit projection contracts.
- prom-ui-runtime:
  platform-neutral UI runtime orchestration.
- prom-ui-backend-native:
  native backend facade and platform event bridge.
- apps/workbench:
  developer/operator tooling surface.
  Workbench is not compiler/verifier/VM owner.
  Workbench is not Semantic UI application contract owner.

NAMING_RULE:
- Semantic:
  verified execution platform / language contract layer.
- PROMETHEUS:
  runtime/effect/capability/audit boundary.
- Semantic UI:
  admitted operator/application layer.
- Semantic Workbench / Semantic Studio:
  developer/operator interface over admitted contracts.
- ALM:
  separate association-driven language/model project.
  Do not mix ALM behavior into Semantic core without a dedicated integration contract.

LEGACY_RULE:
- Historical and compatibility paths are support perimeter, not new architecture owners.
- New architecture must land in the correct owner crate, not legacy shims.
- Do not revive EXOcode/SRIS terminology in new public docs unless explicitly writing historical context.

MUST:
- preserve verifier-first execution.
- preserve deterministic VM semantics.
- preserve SemCode version/header discipline.
- preserve capability-gated PROMETHEUS boundary.
- preserve auditability for external effects.
- preserve Quad Logic as N/F/T/S, not bool.
- preserve explicit resource budgets and safe failure reporting.
- preserve owner boundaries between Semantic, PROMETHEUS, UI, Workbench, and ALM.
- keep docs/spec synchronized with public contract changes.
- keep release/status wording honest.
- make one logical change per PR.
- add tests when behavior changes.
- prefer explicit errors over silent fallback.
- report unimplemented behavior honestly.

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
- erase conflict state.
- treat unknown as false.
- add hidden telemetry.
- add hidden host effects.
- weaken audit requirements for convenience.
- widen public release claims because code landed on main.
- claim unimplemented or unpromised behavior as stable.
- move UI/operator concerns into compiler/verifier/VM core.
- move PROMETHEUS host-effect policy into Semantic core.
- mix ALM association logic into Semantic core without a formal contract.
- do broad rewrites when a narrow fix is enough.
- clean unrelated code in feature/bug PRs.

STATUS_RULE:
- Published stable:
  only behavior promised by stable line.
- Qualified limited release:
  only behavior admitted by evidence.
- Landed on main, not yet promised:
  implemented but not release-promised.
- Out of scope:
  intentionally excluded.
- Landed on main does not mean stable.
- If docs conflict on status, treat it as a readiness defect.

SEMCODE_RULES:
- sm-ir owns SemCode header/opcode/capability contract.
- sm-emit exposes producer-facing entrypoints over sm-ir contract.
- sm-verify is required admission before standard execution.
- sm-vm consumes verified SemCode.

Header/layout/opcode/capability meaning changes require:
- spec update;
- verifier update;
- VM compatibility check;
- disassembler compatibility check where relevant;
- tests/golden fixtures where public behavior changes.

SemCode must remain:
- explicit;
- bounded;
- versioned;
- verifier-admissible;
- deterministic;
- capability-aware where effects are involved.

Do not:
- silently accept unknown SemCode versions;
- best-effort malformed bytecode;
- reinterpret old headers as new headers without compatibility rules.

VERIFIER_RULES:
- Standard route:
  emit SemCode -> verify_semcode -> execute.
- Verifier checks structure/admission.
- Verifier does not execute runtime policy.
- Verifier does not become the VM.
- Verifier does not become the parser.
- Verifier does not become the optimizer.
- Reject diagnostics must be deterministic for same input.

Verifier changes need:
- positive admission tests;
- negative rejection tests;
- deterministic diagnostic expectations where public.

Public verifier behavior changes require:
- docs/spec/verifier.md sync.

VM_RULES:
- Standard public route:
  verify -> run_verified_semcode* -> execute.
- VM must be deterministic for same verified SemCode, runtime config,
  capability context, and entry point.

VM owns:
- execution;
- frames;
- registers;
- quotas;
- safe runtime failure reporting.

VM does not own:
- SemCode binary contract;
- verifier admission policy;
- capability policy semantics;
- host effect policy;
- UI semantics.

VM must:
- surface verifier rejection distinctly from runtime failure;
- reject malformed bytecode instead of accepting it by best effort;
- avoid hidden host dependencies.

QUAD_RULES:
- Canonical Quad values:
  N = unknown
  F = false
  T = true
  S = conflict

- Canonical packed encoding:
  N = 00
  F = 01
  T = 10
  S = 11

Rules:
- Do not silently treat unknown as false.
- Do not silently treat conflict as true or false.
- Do not erase conflict.
- Do not collapse Quad Logic into bool.
- Source branch control must be explicit.
- Any bool conversion must be explicit, local, documented where public, and tested.
- Display order may vary in UI/infographics.
- Encoding order and semantic meaning must not vary.

PROMETHEUS_RULES:
- Semantic core remains deterministic and effect-controlled.
- PROMETHEUS boundary owns external interaction.
- Semantic may request or describe effects.
- Effects must cross PROMETHEUS capability gates.

PROMETHEUS owns:
- external interaction;
- capability checks;
- gate binding;
- ABI effects;
- runtime state;
- deterministic rule agenda;
- audit;
- trace;
- replay records.

Required external effect route:
  effect request
    -> capability check
    -> budget check
    -> gate/policy evaluation
    -> audit decision
    -> execute or reject
    -> trace/record result

Forbidden in Semantic core:
- direct filesystem effects;
- direct network effects;
- direct OS effects;
- hidden telemetry;
- hidden privilege escalation;
- unaudited external effects.

CAPABILITY_RULES:
- Capability checks must be explicit.
- Missing capability means no effect.
- Capability denial must be observable and testable.
- Capability bits must not be repurposed without compatibility review.
- Capability policy belongs to prom-cap / PROMETHEUS boundary,
  not VM core by implication.

AUDIT_RULES:
- External effects must be auditable.
- No hidden effect without audit path.
- Audit records must be deterministic where contractually observable.
- prom-audit owns trace/replay record vocabulary.
- UI may project audit records visually, but UI is not audit authority.

RESOURCE_BUDGET_RULES:
- Execution must respect declared and effective budgets.

Relevant budgets may include:
- VM steps;
- memory;
- effect calls;
- handles;
- audit records;
- runtime ownership limits;
- gate/session quotas.

Budget failure must be:
- explicit;
- deterministic;
- safely reported.

Quota changes require quota limit/usage tests.

UI_RULES:
- Semantic UI is active, but it is admitted surface, not semantic authority.
- UI lives after verifier-admitted execution and before platform-native rendering.
- UI must follow UI ownership maps and related UI boundary specs.
- UI must not become a second compiler.
- UI must not become a verifier.
- UI must not become a VM policy layer.
- UI must not become a hidden host side-effect path.
- UI must not redefine SemCode behavior.
- UI must not redefine Quad Logic meaning.
- UI must not redefine capability or audit authority.
- UI must not bypass PROMETHEUS capability gates.
- UI must not treat visual state as source-of-truth state.
- UI must not treat renderer output as audit authority.
- UI must not turn Workbench behavior into core Semantic behavior.

Semantic UI may:
- display admitted state;
- visualize verifier results;
- visualize VM execution traces;
- visualize capability denial;
- visualize audit records;
- expose operator/developer workflows;
- render admitted UI contracts;
- stage platform events through declared boundaries.

Semantic UI must preserve:
- verifier-first execution;
- deterministic core behavior;
- capability-gated effects;
- auditability;
- explicit denial/error/quarantine states;
- owner separation between Semantic core, PROMETHEUS, UI runtime,
  renderer, Workbench, and native backend.

UI_OWNERSHIP_RULES:
- UI source surface belongs to sm-front / sm-sema when language surface is involved.
- UI call lowering belongs to sm-ir / sm-emit.
- UI ABI call IDs belong to prom-abi.
- UI capabilities belong to prom-cap / PROMETHEUS capability boundary.
- UI event model belongs to prom-ui.
- UI runtime orchestration belongs to prom-ui-runtime.
- Native backend ownership belongs to prom-ui-backend-native or backend-specific crates.
- Renderer ownership must be introduced only through an admitted renderer boundary.
- Workbench is tooling/operator surface, not Semantic UI application contract owner.

Do not move:
- parser/typechecker behavior into prom-ui-runtime;
- layout/widget semantics into sm-vm;
- native handles into Semantic VM core;
- capability policy into renderer;
- audit authority into visual projection;
- UI convenience shortcuts into SemCode/verifier contracts.

UI_VISUAL_RULES:
- Visual doctrine owns visual meaning.
- Visual token system owns reusable visual vocabulary.
- Layout primitive system owns spatial grammar.
- Component system owns reusable semantic UI units.
- Renderer consumes admitted visual/layout/component output.
- Renderer does not define Semantic UI doctrine.
- Native backend does not own visual doctrine, tokens, layout, or components.
- prom-ui-runtime does not own visual doctrine.

Preserve order:
  meaning
    -> tokens
    -> layout
    -> components
    -> renderer
    -> native backend

UI_INTERACTION_RULES:
- Native event is not semantic intent.
- Intent is not action.
- Hover is not focus.
- Focus is not selection.
- Selection is not permission.
- Action is not effect.
- Effect request is not committed effect.
- UI capability is not runtime capability by default.
- Prepared effect is not committed effect.

Required chain:
  native event
    -> normalized input signal
    -> interaction intent
    -> focus/selection context
    -> semantic action request
    -> admission/policy check
    -> effect request
    -> capability/budget/audit boundary
    -> committed or denied effect

UI_TRACE_AUDIT_RULES:
- Trace is not decorative log.
- Audit is not UI state.
- Visual trace is not source of truth.
- Renderer output is not audit authority.
- Native backend transcript is not audit authority.
- Workbench may display audit projections, but must not define audit meaning.

UI may display:
- verifier admission result;
- VM execution trace;
- capability admission/denial;
- effect prepare/commit status;
- runtime failure;
- quarantine state;
- conflict state;
- audit record projection.

UI must distinguish:
- error vs denial;
- denial vs failure;
- quarantine vs deletion;
- conflict vs crash;
- visual refusal vs hidden no-op.

UI_NATIVE_BACKEND_RULES:
- prom-ui-backend-native may own native facade and platform event bridge.
- Native backend may stage config, events, run summaries, and transcripts.
- Draw staging is not renderer ownership.
- Draw staging is not frame presentation.
- Native transcript facts are not audit authority.
- Native backend must not pull verifier, VM, SemCode, or PROMETHEUS policy ownership into itself.

Still out of scope unless explicitly admitted:
- renderer implementation;
- GPU/shader pipeline;
- surface/pixels/wgpu ownership;
- frame presentation;
- retained widget tree;
- browser target;
- mobile target;
- Workbench integration as core behavior.

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

Do not imply support for:
- ADT payload paths;
- schema paths;
- partial release;
- inter-frame borrow persistence;
- advanced aliasing;
- indirect projection;

unless explicitly implementing, documenting, and testing it.

TASK_START:
Before editing, identify:

1. Layer:
   frontend | sema | IR | emit | verify | runtime-core | VM | cli |
   prom-* | UI | UI-native-backend | Workbench | docs | tests | legacy

2. Contract impact:
   none | internal | public spec | release/status | SemCode compatibility |
   verifier admission | VM execution | capability/audit |
   UI ownership | UI visual doctrine | UI interaction semantics |
   UI trace/audit projection | UI native backend | Workbench/operator surface |
   ALM integration

3. Required tests:
   list before editing.

EDIT_POLICY:
- Prefer smallest correct patch.
- Preserve owner boundaries.
- Preserve no_std / feature-gated behavior where present.
- Prefer explicit errors over silent fallback.
- Do not move behavior across crates without architectural reason.
- Do not clean unrelated code in feature/bug PRs.
- If cleanup reveals feature work, split it into a separate PR/issue.
- Do not rename architectural concepts casually.
- Do not broaden project scope inside narrow fixes.
- Do not hide architectural changes inside refactors.

TEST_POLICY:
- verifier change =>
  positive + negative admission tests.
- SemCode layout/header/capability change =>
  golden/compat fixtures.
- VM behavior change =>
  deterministic execution tests.
- quota change =>
  quota limit/usage tests.
- runtime ownership change =>
  overlap and allowed-sibling tests.
- prom-* capability/audit change =>
  denial + audit path tests.
- docs-only status change =>
  check public status vocabulary.
- UI ownership change =>
  verify UI ownership map remains consistent.
- UI ABI/capability change =>
  admission + denial tests.
- UI runtime change =>
  deterministic transcript / lifecycle tests where applicable.
- UI native backend change =>
  backend boundary tests without moving ownership into runtime core.
- UI visual doctrine/token/layout/component change =>
  contract-map or architecture doc update.
- UI interaction/action change =>
  intent/action/effect separation tests.
- UI trace/audit visual change =>
  verify visual projection is not audit authority.
- Workbench change =>
  ensure Workbench does not redefine core Semantic behavior.
- no_std-sensitive change =>
  cargo check --no-default-features where relevant.

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

UI_PREFERRED_COMMANDS:
- cargo test -q -p prom-ui
- cargo test -q -p prom-ui-runtime
- cargo test -q -p prom-ui-backend-native
- cargo test -q -p prom-ui-backend-native --features winit-backend
- git diff --check

OUTPUT:
Summary:
Layer:
Contract impact:
Invariants checked:
Changes:
Tests:
Risks:
Next:
