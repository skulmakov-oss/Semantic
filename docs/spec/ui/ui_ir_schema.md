# UI IR Schema

Status: draft spec
Track: POST-UI / Intent-Driven Projection
Scope type: documentation only
Depends on:
- docs/dna/SEMANTIC_UI_DNA.md
- docs/dna/SEMANTIC_UI_DNA_v2.md
- docs/roadmap/post_ui/intent_driven_projection_roadmap.md
- docs/spec/ui/projection_source_model.md
Related:
- #1310
- #1328
- #1329
- #1330
- #1331

UI IR is a deterministic compiled projection artifact.

It is not Semantic source.
It is not renderer code.
It is not runtime authority.
It is not verifier admission.
It is not production UI wiring.

This document does not define Rust types, parser implementation, compiler implementation, runtime patch pipeline, or renderer backend behavior.

## 1. Purpose

UI IR sits in the pipeline as the deterministic compiled form of projection source:

```text
.sm owns meaning.
.proj.sm or equivalent projection source owns presentation intent.
Compiler emits UI IR.
Shell interprets UI IR.
Renderer paints through an adapter.
Semantic admission remains authoritative.
```

UI IR exists to make projection deterministic, inspectable, testable, and renderer-independent.

It is the compiled projection contract, not the semantic contract itself.

## 2. Non-Authority Rule

UI IR must not own or redefine:

- semantic truth;
- verifier rules;
- admission rules;
- VM / runtime behavior;
- capability authority;
- repository truth;
- business logic;
- host effects.

```text
UI IR carries projection structure.
Semantic remains the source of truth.
```

UI IR is a projection artifact, not a policy engine and not a second source of truth.

## 3. Top-Level IR Document Shape

The future UI IR document is expected to contain, at a high level:

- `ir_version`
- `projection_id`
- `source_refs`
- `role_dictionary_version`
- `surfaces`
- `bindings`
- `actions`
- `evidence_routes`
- `denial_routes`
- `recovery_routes`
- `task_contracts`
- `connectivity_policy`
- `accessibility_contract`
- `diagnostics`

This is a structural contract, not final serialization syntax.

## 4. Source References

Source references provide traceability across the projection pipeline.

They should cover:

- originating `.sm` source reference;
- originating projection source reference;
- source hash or revision;
- generated-at compiler identity, if available;
- non-authoritative status.

```text
Source references provide evidence and traceability.
They do not make UI IR canonical semantic truth.
```

Source refs help humans and tools inspect provenance without turning UI IR into the authority layer.

## 5. Surfaces

Surfaces are top-level projected UI units.

Each surface should carry:

- stable surface id;
- surface role;
- title / label;
- criticality;
- visibility policy;
- viewer-relative policy;
- root node reference;
- evidence / denial / recovery outlets;
- connectivity / freshness display policy.

```text
A surface is a projection unit, not an application runtime.
```

Surfaces define what a user or observer sees as one coherent projection area.

## 6. Nodes

Nodes are structural projection units inside a surface.

Each node should carry:

- stable node id;
- role;
- parent / children relationship;
- label;
- binding refs;
- action refs;
- accessibility refs;
- evidence refs;
- state display policy;
- criticality;
- diagnostics.

```text
Nodes are semantic projection structure, not renderer widgets.
```

Nodes organize how projection is assembled and referenced.

## 7. Role Dictionary

UI IR roles are versioned through a role dictionary.

Each role entry should include:

- role name;
- role version;
- role meaning;
- allowed interpretation;
- forbidden interpretation;
- non-visual interpretation requirement.

```text
Every UI IR role must remain interpretable by non-visual surfaces such as CLI, logs, voice UI, or evidence reports.
```

Initial draft vocabulary is seeded from the Projection Source Model:

- `AppSurface`
- `Panel`
- `Section`
- `FieldGroup`
- `TextReadout`
- `NumericReadout`
- `StateBadge`
- `EvidencePanel`
- `DenialOutlet`
- `RecoveryOutlet`
- `TaskPanel`
- `ActionSlot`
- `SafeAction`
- `GuardedAction`
- `DangerAction`
- `ConnectivityBadge`
- `List`
- `ListItem`

Roles remain semantic projection roles, not renderer widgets.

## 8. Bindings

Bindings are read-side projection links from semantic state into UI IR structure.

Bindings should cover:

- state binding;
- evidence binding;
- action offer binding;
- task binding;
- connectivity binding;
- binding id;
- source path / source ref;
- target node;
- revision / epoch requirement;
- stale / unknown behavior.

```text
Bindings observe state.
Bindings do not mutate Semantic state.
```

Bindings are declarative links, not imperative update code.

## 9. Action References

Action references describe projection-side affordance routes.

Each action reference should carry:

- action id;
- source `ActionOffer` reference;
- role: `SafeAction` / `GuardedAction` / `DangerAction`;
- target node / surface;
- capability requirement;
- confirmation / repeat restrictions;
- `source_state_rev` / `source_task_rev` requirement;
- disabled / denied / unavailable projection state.

```text
UI IR may route an action affordance.
UI IR must not invent an action or bypass admission.
```

```text
UI proposes.
Semantic disposes.
Shell shows.
```

Action references project where affordances appear and how they are gated.

## 10. Evidence Outlets

Evidence outlets route provenance, trace, and evidence visibility.

Each evidence outlet should carry:

- evidence outlet id;
- source evidence ref;
- target surface / node;
- evidence kind;
- provenance display;
- trace visibility;
- uncertainty display;
- redaction / privacy note if needed.

```text
Evidence outlets display provenance.
They do not become audit authority.
```

Evidence outlets make claims inspectable without promoting projection to authority.

## 11. Denial and Recovery Routes

UI IR must define routed surfaces for denial and recovery.

Relevant concepts include:

- `DenialOutlet`;
- `RecoveryOutlet`;
- `LocalDenied`;
- `AdmissionDenied`;
- `PartialDenied`;
- `NotApplied`;
- `BatchBreak`;
- `Dismiss`;
- `Acknowledge`;
- `Retry`;
- `Resume`;
- `CancelSuffix`;
- `ResumeToken` reference.

```text
Denied is projected, not handled.
Recovery is projected, but never improvised.
```

UI IR only routes known denial / recovery semantics.
It does not invent recovery behavior.

## 12. Task Projection Contracts

Task-related projection contracts should cover:

- `TaskRecord` reference;
- task state binding;
- phase;
- progress;
- `AwaitingInput`;
- `allowed_controls` / `ActionOffers`;
- scope locks;
- evidence timeline;
- completion / failure / quarantine projection.

```text
Task lives in Semantic.
Progress lives in Projection.
Pixels live in Shell.
```

Task projection is a visibility and affordance contract, not a task engine implementation.

## 13. Connectivity and Freshness Policy

Connectivity and freshness are projection concerns with control consequences.

UI IR should cover:

- `Fresh`;
- `Degraded`;
- `Stale`;
- `Offline`;
- `Resyncing`;
- `PendingUnknown`;
- control availability;
- critical action restrictions;
- no offline queue for critical actions.

```text
No freshness, no control.
```

UI IR may declare freshness display and control gating expectations.
It does not implement networking.

## 14. Accessibility Contract

Accessibility is part of the UI IR contract, not renderer polish.

Accessibility requirements should include:

- label;
- role description;
- focus order;
- criticality;
- non-visual interpretation;
- denial / recovery discoverability;
- evidence provenance where practical.

Accessibility must remain visible in the projection contract even when renderer implementations differ.

## 15. Quad-State Preservation

UI IR must preserve:

- `N` — unknown
- `F` — false
- `T` — true
- `S` — conflict

```text
UI IR must not flatten Quad-state into boolean visibility, success/failure, or generic disabled state.
```

Unknown remains unknown.
Conflict remains conflict.

## 16. Keyed Collections

Projected collections require stable keys.

Requirements:

- lists require stable keys;
- `ListItem` requires stable identity;
- missing stable key is a projection check error;
- reordering must preserve identity;
- patch streams depend on stable identity.

```text
No stable key, no deterministic projected collection.
```

Stable keys are part of deterministic projection, not a convenience detail.

## 17. Diagnostics

Diagnostics are part of the IR contract.

UI IR diagnostics should cover:

- missing binding;
- invalid role;
- forbidden role interpretation;
- missing accessibility label;
- missing stable key;
- denied unsafe action route;
- unsupported role dictionary version;
- unresolved source ref;
- renderer capability mismatch.

Diagnostics are evidence, not runtime guesses.

## 18. Non-Normative IR Sketch

Non-normative sketch — not final serialization

```text
ui_ir {
  ir_version: "0-draft"
  projection_id: "CalculatorView"
  role_dictionary_version: "0-draft"

  surface main {
    role: AppSurface
    root: node.display_panel
  }

  node display {
    role: NumericReadout
    bind: state.result
    accessibility.label: "Calculator result"
  }

  action add {
    role: SafeAction
    from: ActionOffers.add
    target: node.add_slot
  }

  outlet evidence {
    role: EvidencePanel
  }

  outlet denial {
    role: DenialOutlet
  }
}
```

This sketch illustrates structure only.
It is not final grammar, not final serialization, and not an implementation plan.

## 19. Acceptance Criteria

The spec is acceptable when:

- it defines UI IR as deterministic compiled artifact;
- it explains relation to `.sm` and projection source;
- it defines top-level IR shape;
- it defines surfaces and nodes;
- it defines role dictionary versioning;
- it defines bindings;
- it defines action references;
- it defines evidence outlets;
- it defines denial / recovery routes;
- it defines task projection contracts;
- it defines connectivity / freshness policy;
- it defines accessibility contract;
- it preserves Quad-state meaning;
- it requires stable keys for collections;
- it includes diagnostics;
- it does not define Rust types;
- it does not implement parser / compiler / runtime;
- it does not claim production readiness.
