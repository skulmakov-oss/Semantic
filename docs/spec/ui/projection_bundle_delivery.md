# ProjectionBundle Delivery

Status: draft spec
Track: POST-UI / Intent-Driven Projection
Scope type: documentation only
Depends on:
- docs/dna/SEMANTIC_UI_DNA.md
- docs/dna/SEMANTIC_UI_DNA_v2.md
- docs/roadmap/post_ui/intent_driven_projection_roadmap.md
- docs/spec/ui/projection_source_model.md
- docs/spec/ui/ui_ir_schema.md
- docs/spec/ui/action_ir_routing.md
- docs/spec/ui/projection_patch_model.md
- docs/spec/ui/denial_recovery_projection.md
- docs/spec/ui/task_projection_model.md
- docs/spec/ui/multi_client_freshness_model.md
Related:
- #1310
- #1328
- #1329
- #1330
- #1331
- #1332
- #1333
- #1334
- #1335
- #1336
- #1337

ProjectionBundle delivery defines how compiled projection artifacts are packaged, verified, pinned, selected, and updated.

It does not define Semantic truth, verifier rules, admission policy, VM behavior, runtime authority, networking implementation, renderer implementation, host effects, or production UI wiring.

This document does not implement ProjectionBundle loading, bundle verification, runtime patch pipeline, shell behavior, Rust types, compiler behavior, networking, storage, or renderer backend behavior.

## 1. Purpose

This spec exists to prevent the bad workflow:

```text
runtime streams arbitrary UI tree -> shell renders whatever arrives
```

The intended model is:

```text
Projection source compiles to UI IR and related contracts.
ProjectionBundle packages compiled projection artifacts.
Critical bundles are pinned / preinstalled.
Dynamic bundles are verified before activation.
Runtime traffic after activation is patches and intents.
Shell renders through adapter boundaries.
Semantic authority remains outside the bundle.
```

```text
Critical UI is pinned.
Dynamic UI is verified.
Runtime UI is patched.
```

ProjectionBundle delivery keeps runtime UI honest by making package trust and update boundaries explicit.

## 2. Non-Authority Rule

ProjectionBundle must not own or redefine:

- Semantic truth;
- verifier admission;
- VM / runtime behavior;
- capability policy;
- recovery policy;
- task engine behavior;
- ActionOffer authority;
- networking behavior;
- host effects;
- renderer lifecycle.

```text
Bundle carries projection artifacts.
Semantic remains authority.
Shell interprets only verified projection contracts.
```

ProjectionBundle is a delivery unit, not a Semantic authority unit.

## 3. ProjectionBundle Definition

`ProjectionBundle` is the packaged delivery unit for compiled projection artifacts.

It should cover:

- bundle id;
- bundle version;
- projection id;
- source refs;
- compiled artifacts;
- role dictionary version;
- renderer profile;
- capability requirements;
- freshness requirements;
- hash / signature metadata;
- safety class;
- activation policy;
- evidence refs;
- diagnostics.

```text
ProjectionBundle is a delivery unit, not a Semantic authority unit.
```

The bundle is the unit that can be selected, verified, and activated for projection.

## 4. Bundle Manifest

Bundle manifest fields are expected to include, at a high level:

- `bundle_id`
- `bundle_version`
- `projection_id`
- `source_refs`
- `ui_ir_ref`
- `binding_graph_ref`
- `action_ir_ref`
- `role_dictionary_version`
- `renderer_profile`
- `safety_class`
- `criticality`
- `required_capabilities`
- `freshness_policy`
- `hash`
- `signature`
- `created_by`
- `created_at`
- `compiler_identity`
- `compatibility`
- `activation_policy`
- `update_policy`
- `diagnostics`

This is a structural manifest contract, not final serialization syntax.

## 5. Included Artifacts

A bundle may include or reference:

- UI IR;
- Binding Graph;
- Action IR;
- denial / recovery routes;
- task projection contracts;
- connectivity / freshness policy;
- accessibility contract;
- diagnostics metadata;
- evidence / source refs.

```text
A bundle packages projection contracts.
It does not package arbitrary executable UI authority.
```

The bundle should carry the contracts needed to project the UI faithfully, not a free-form runtime tree.

## 6. Source References and Provenance

Bundle provenance should preserve traceability.

It should cover:

- originating `.sm` refs;
- originating projection source refs;
- source hashes / revisions;
- compiler identity;
- build timestamp;
- bundle hash;
- signature;
- evidence refs;
- non-authoritative status.

```text
Provenance supports trust and audit.
It does not make the bundle Semantic truth.
```

Provenance lets operators inspect what was built and from what inputs without transferring authority to the package itself.

## 7. Role Dictionary Version

Role dictionary handling must be explicit.

It should cover:

- role dictionary id / version;
- compatibility check;
- unsupported role dictionary diagnostic;
- non-visual interpretation requirement;
- no silent role reinterpretation;
- renderer capability mismatch.

```text
No supported role dictionary, no safe projection activation.
```

If the bundle's role vocabulary is not understood, activation must not silently proceed.

## 8. Renderer Profile

Renderer profile describes compatibility expectations.

It should cover:

- renderer family / profile;
- supported role set;
- accessibility support;
- evidence outlet support;
- denial / recovery support;
- task projection support;
- connectivity / freshness support;
- critical action affordance support;
- diagnostics for mismatch.

Forbidden in this spec:

- selecting backend-specific code path;
- embedding GPU commands;
- embedding CSS-like layout;
- manual pixels / colors / fonts / themes;
- declaring renderer authority.

```text
Renderer profile describes compatibility.
It does not transfer ownership to renderer backend.
```

Renderer profile helps choose the right compatible shell path without turning renderer details into authority.

## 9. Safety Classes

Bundle safety classes include:

- `CriticalPinned`
- `VerifiedDynamic`
- `DiagnosticOnly`
- `WorkbenchExperimental`
- `ReadOnlyDashboard`

Required distinctions:

- `CriticalPinned` is preinstalled / pinned for operator / guarded / danger surfaces;
- `VerifiedDynamic` may be loaded only after verification;
- `DiagnosticOnly` is a non-control diagnostic surface;
- `WorkbenchExperimental` is research / tooling only;
- `ReadOnlyDashboard` cannot expose control affordances unless separately approved.

```text
Critical control surfaces require CriticalPinned bundles.
Verification is additional evidence, not a substitute for pinning.
```

Safety class defines activation posture and control expectations.

## 10. Pinned Critical UI

Pinned critical UI is the preinstalled, pinned bundle path for operator-sensitive surfaces.

It should cover:

- preinstalled bundle;
- expected hash / signature;
- approved role dictionary;
- approved renderer profile;
- explicit activation policy;
- no runtime arbitrary replacement;
- safe update boundary;
- operator evidence visibility.

```text
Critical UI is pinned.
```

Pinned critical UI is stable by policy, not by accidental lack of updates.

## 11. Verified Dynamic UI

Verified dynamic UI is a bundle that is trusted only after verification succeeds.

It should cover:

- verification before activation;
- hash / signature check;
- compatibility check;
- capability check;
- role dictionary check;
- renderer profile check;
- safety class check;
- diagnostics on failure;
- no activation on failed verification.

```text
Dynamic UI is verified before it is trusted.
```

Dynamic UI is allowed to change, but not to bypass trust boundaries.

## 12. Runtime Traffic Rule

After bundle activation, runtime traffic is patches and intents, not full UI tree streaming.

Runtime traffic may include:

- `SemanticStatePatch`;
- `ProjectionPatch`;
- `RenderPatch`;
- `EvidencePatch`;
- `ActionOfferPatch`;
- `ConnectivityPatch`;
- `TaskStatePatch`;
- `ActionIntent`;
- `ActionIntentBatch`;
- `StreamIntent`.

```text
Runtime UI is patched.
```

Runtime traffic after activation must preserve the patch / intent model rather than replacing it with arbitrary tree streaming.

## 13. Bundle Activation

Activation is the trust boundary for a bundle.

It should cover:

- bundle selected;
- manifest inspected;
- compatibility verified;
- safety class checked;
- role dictionary supported;
- renderer profile supported;
- evidence / diagnostics recorded;
- projection becomes active only after verification succeeds.

```text
Activation is a trust boundary.
It is not arbitrary UI interpretation.
```

Activation must only occur when the bundle matches the environment and policy contract.

## 14. Safe Update Boundaries

Critical bundle updates must not occur during:

- running guarded task;
- running danger task;
- unresolved batch break;
- `PendingUnknown`;
- quarantine;
- `Resyncing`;
- stale / offline control channel;
- unresolved denial requiring acknowledgement;
- active critical control confirmation.

```text
Critical bundle update waits for safe boundary.
```

Safe update boundaries prevent UI replacement from interrupting unresolved authority-sensitive work.

## 15. Bundle Replacement and Migration

Bundle replacement is a controlled transition.

It should cover:

- old bundle id / version;
- new bundle id / version;
- compatibility check;
- projection rev boundary;
- patch stream boundary;
- resync if needed;
- evidence record;
- rollback option if authority provides it;
- no silent replacement of critical UI.

```text
Bundle replacement is a controlled transition, not runtime UI drift.
```

Migration should be visible, attributable, and bounded.

## 16. Critical Action Affordances

Critical affordance delivery has stricter constraints.

It should cover:

- `GuardedAction`;
- `DangerAction`;
- `TaskControl`;
- confirmation route;
- denial / recovery route;
- freshness requirement;
- no offline / stale availability;
- no unchecked dynamic critical control.

```text
Critical action affordances require verified route, freshness, and authority.
```

Critical affordances must not be smuggled in through an unverified dynamic bundle.

## 17. Read-Only and Diagnostic Surfaces

Weaker surfaces must be clearly bounded.

They should cover:

- read-only dashboard;
- diagnostics surface;
- evidence viewer;
- Workbench experimental surface;
- no control affordances by default;
- clear safety class;
- no implied production readiness.

```text
Read-only projection must not smuggle control authority.
```

Read-only and diagnostic bundles are useful, but they do not automatically become control surfaces.

## 18. Evidence and Audit Trace

Bundle delivery must remain inspectable after the fact.

Evidence should include:

- bundle id / version;
- hash / signature;
- source refs;
- compiler identity;
- verification result;
- activation result;
- update result;
- diagnostics;
- actor / session / client refs where relevant;
- privacy / redaction note.

```text
Bundle delivery must be inspectable after the fact.
```

Evidence keeps delivery operations auditable without making the bundle itself authoritative.

## 19. Privacy and Redaction

Bundle provenance may need privacy filtering.

Rules:

- bundle provenance should remain auditable;
- sensitive source / actor / session / client detail may be redacted where authority requires;
- redaction must not falsify bundle identity or verification state;
- UI must not invent attribution.

```text
Privacy may hide details.
It must not falsify trust evidence.
```

Redaction must preserve the truth of verification and identity at the bundle level.

## 20. Diagnostics

Diagnostics should include:

- missing manifest;
- unsupported bundle version;
- missing UI IR;
- missing Binding Graph;
- missing Action IR;
- unsupported role dictionary;
- renderer profile mismatch;
- missing hash;
- invalid signature;
- safety class violation;
- critical bundle not pinned;
- dynamic bundle not verified;
- unsafe update boundary;
- runtime full tree streaming attempt;
- control affordance in read-only bundle;
- accessibility contract missing;
- evidence route missing.

```text
Diagnostics are evidence, not silent shell guesses.
```

Diagnostics are part of delivery accountability, not an optional debug surface.

## 21. Non-Normative Sketch

Non-normative sketch — not final serialization

```text
projection_bundle {
  bundle_id: "bundle.calculator.v0"
  bundle_version: "0-draft"
  projection_id: "CalculatorView"
  safety_class: VerifiedDynamic
  role_dictionary_version: "ui-roles.0-draft"
  renderer_profile: "semantic-shell.reference"
  artifacts: {
    ui_ir: "ui_ir.calculator"
    binding_graph: "binding_graph.calculator"
    action_ir: "action_ir.calculator"
  }
  trust: {
    hash: "sha256:..."
    signature: "sig:..."
    compiler_identity: "semantic-projection-compiler.0-draft"
  }
  activation_policy: {
    require_verification: true
    allow_runtime_tree_streaming: false
  }
}

bundle_update_boundary {
  from: "bundle.calculator.v0"
  to: "bundle.calculator.v1"
  require_no_pending_unknown: true
  require_no_quarantine: true
  require_fresh_control_channel: true
}
```

This sketch only illustrates delivery shape.
It is not final grammar, not implementation, and not a runtime contract.

## 22. Acceptance Criteria

The spec is acceptable when:

- it defines `ProjectionBundle`;
- it defines bundle manifest fields;
- it defines included artifacts;
- it defines source provenance;
- it defines role dictionary version handling;
- it defines renderer profile;
- it defines safety classes;
- it defines pinned critical UI;
- it defines verified dynamic UI;
- it defines runtime traffic as patches / intents;
- it defines bundle activation;
- it defines safe update boundaries;
- it defines bundle replacement / migration;
- it defines critical action affordance constraints;
- it defines read-only / diagnostic surface constraints;
- it defines evidence / audit trace;
- it defines privacy / redaction boundary;
- it defines diagnostics;
- it preserves Semantic authority;
- it includes a non-normative sketch only;
- it does not implement bundle loading;
- it does not implement verification;
- it does not claim production readiness.
