# Semantic Core Spec Bundle

Status: draft v0

This directory is the canonical specification bundle for the current core
execution contract.

Current documents in this PR:

- `syntax.md` - canonical Rust-like source syntax contract
- `types.md` - source-level type contract and current type-family limits
- `source_semantics.md` - source-level execution and binding semantics
- `diagnostics.md` - source-facing parse, policy, type, and module diagnostics
- `modules.md` - module, import, and re-export contract
- `logos.md` - declarative Logos source-surface contract
- `semcode.md` - SemCode binary contract and compatibility rules
- `profile.md` - `ParserProfile` policy contract
- `verifier.md` - SemCode admission verification contract
- `vm.md` - Semantic VM public execution contract
- `runtime_ownership.md` - frozen tuple + direct record-field runtime ownership contract
- `quotas.md` - runtime quota taxonomy and enforcement contract
- `abi.md` - PROMETHEUS host ABI boundary contract
- `capabilities.md` - capability manifest and denial contract
- `gates.md` - gate registry and binding contract
- `runtime.md` - runtime orchestration session contract
- `state.md` - semantic state model and invariants
- `rules.md` - deterministic rule and agenda contract
- `audit.md` - audit trail and replay metadata contract
- `ui_contract_map.md` - POST-UI Semantic UI contract sketch and ownership map
- `ui_abi_capability_admission.md` - POST-UI ABI/capability admission checklist for future UI operations
- `ui_verifier_admission_metadata.md` - POST-UI verifier-visible metadata plan for future UI operation admission
- `ui/README.md` - POST-UI UI runtime effect path boundary subbundle
- `ui/host_runtime_effect_path_boundary.md` - canonical UI host runtime effect path boundary contract
- `ui/ui_effect_envelope_v0.md` - canonical UI effect envelope v0 contract
- `ui/ui_capability_taxonomy.md` - canonical UI capability taxonomy contract
- `ui/ui_event_envelope_model.md` - canonical deterministic UI event envelope model
- `ui/ui_frame_lifecycle_contract.md` - canonical UI frame lifecycle contract
- `ui/ui_draw_command_batch_contract.md` - canonical minimal draw command batch contract
- `../architecture/ui_native_backend_boundary.md` - native facade transcript boundary and ownership split
- `../architecture/ui_renderer_admission_boundary.md` - renderer admission boundary before implementation
- `../architecture/ui_visual_design_doctrine.md` - Semantic UI visual design doctrine before renderer implementation
- `../architecture/ui_visual_token_system_boundary.md` - Semantic UI visual token system boundary before implementation
- `../architecture/ui_layout_primitive_boundary.md` - Semantic UI layout primitive boundary before implementation
- `../architecture/ui_component_admission_boundary.md` - Semantic UI component admission boundary before implementation
- `../architecture/ui_interaction_input_semantic_boundary.md` - Semantic UI interaction and input semantic boundary before implementation
- `../architecture/ui_focus_selection_semantic_boundary.md` - Semantic UI focus and selection semantic boundary before implementation
- `../architecture/ui_semantic_action_boundary.md` - Semantic UI action boundary before implementation
- `../architecture/ui_effect_request_capability_boundary.md` - Semantic UI effect request and UI capability boundary before implementation
- `../architecture/ui_trace_audit_visual_boundary.md` - Semantic UI trace and audit visual boundary before implementation
- `../architecture/ui_error_denial_quarantine_visual_boundary.md` - Semantic UI error, denial, and quarantine visual boundary before implementation
- `../architecture/ui_recovery_rollback_visual_boundary.md` - Semantic UI recovery and rollback visual boundary before implementation
- `../architecture/ui_renderer_transcript_presentation_boundary.md` - Semantic UI renderer transcript and presentation status boundary before implementation
- `../architecture/ui_workbench_consumption_boundary.md` - Workbench UI consumption boundary before implementation
- `../architecture/ui_simulation_snapshot_boundary.md` - Semantic UI simulation and snapshot boundary before implementation
- `../architecture/ui_boundary_index.md` - canonical Semantic UI boundary index
- `../architecture/ui_implementation_gate.md` - Semantic UI implementation gate before code phase

Adjacent source-surface documents also remain relevant:

- `docs/LANGUAGE.md`

Later PRs may extend this bundle further with richer module, package, CLI,
versioning, and release-facing validation specifications.

Contract precedence:

1. `docs/spec/*` defines the public contract.
2. Code must implement that contract.
3. Architecture and roadmap documents constrain ownership and sequencing around
   that contract.

Blocking rule:

- any public change to SemCode admission, VM execution, quota semantics, or
  `ParserProfile` policy must update the relevant file in this directory in the
  same change series.
