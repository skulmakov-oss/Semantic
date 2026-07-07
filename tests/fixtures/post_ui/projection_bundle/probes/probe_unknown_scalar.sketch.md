# Minimal ProjectionBundle Manifest Sketch

Status: non-executing sketch
Track: POST-UI / Intent-Driven Projection
Scope type: fixture sketch only
Serialization status: not final
Execution status: non-executing
Implementation status: blocked

This sketch is not final serialization.
It must not be parsed by production code.
It must not be loaded by a ProjectionBundle loader.
It must not be treated as a runtime contract.
It exists only as planning evidence.

```text
projection_bundle {
  bundle_id: "bundle.example.minimal"
  bundle_version: "0-sketch"
  projection_id: "ExampleMinimalProjection"

  source_refs: [
    "semantic.source.example",
    "projection.source.example"
  ]

  artifacts {
    ui_ir_ref: "ui_ir.example.minimal"
    binding_graph_ref: "binding_graph.example.minimal"
    action_ir_ref: "action_ir.example.minimal"
  }

  compatibility {
    role_dictionary_version: "ui-roles.0-sketch"
    renderer_profile: "semantic-shell.reference-sketch"
  }

  safety {
    unknown_scalar: "test"
    safety_class: "VerifiedDynamic"
    criticality: "NonCritical"
    required_capabilities: []
    freshness_policy: "FreshForControl"
  }

  trust {
    hash: "sha256:SKETCH-NOT-A-REAL-HASH"
    signature: "signature:SKETCH-NOT-A-REAL-SIGNATURE"
    created_by: "semantic-projection-compiler.SKETCH"
    created_at: "not-a-real-timestamp"
    compiler_identity: "semantic-projection-compiler.0-sketch"
  }

  activation_policy {
    require_verification: true
    allow_runtime_tree_streaming: false
    allow_production_activation: false
  }

  update_policy {
    require_safe_update_boundary: true
    allow_critical_update_during_pending_unknown: false
    allow_critical_update_during_quarantine: false
  }

  diagnostics {
    expected: []
  }
}
```

This sketch intentionally uses placeholder identity, hash, signature, compiler, and timestamp values.

These placeholders must not pass future verification.

## Boundary Notes

A future reader may use this sketch only as fixture evidence after a separate approved task.
A future loader must not treat this sketch as loadable.
A future verifier must reject the placeholder hash and signature if verification is implemented.
A future runtime must not activate this sketch.
