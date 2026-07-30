# Qualification Report

## Main Validation Gates
`	ext
Running: pwsh scripts\admission_guard.ps1 -PRReady
Running: pwsh scripts\admission_guard.ps1 -Readiness
Running: pwsh scripts\admission_guard.ps1 -FullPreflight
Running: cargo fmt --all --check
Running: cargo check --workspace --all-targets --all-features --keep-going
Running: cargo clippy --workspace --all-targets --all-features -- -D warnings
Running: cargo test --workspace --all-features
Running: cargo test --test public_api_contracts
Running: cargo test --test golden_semcode
Running: cargo test --test prometheus_runtime_matrix
Running: cargo test --test prometheus_runtime_goldens
Running: cargo test --test prometheus_runtime_negative_goldens
Running: cargo test --test prometheus_runtime_compat_matrix
Running: cargo test -p quad_logic_calculator
Running: cargo test -p semantic-hub
Running: cargo test -p semantic-hub-turbovec

`

## Expanded Smoke Matrix
`	ext
--- 1. minimal ---
compiled 'examples/calculator.sm' -> 'artifacts/zip_inspect/calculator.smc' (380 bytes)
--- 2. f64 builtin with assert ---
compiled 'artifacts/zip_inspect/smoke_f64_assert.sm' -> 'artifacts/zip_inspect/smoke_f64_assert.smc' (124 bytes)
--- 3. quad program ---
compiled 'artifacts/zip_inspect/smoke_quad.sm' -> 'artifacts/zip_inspect/smoke_quad.smc' (76 bytes)
--- 4. policy/state program ---
compiled 'examples/semantic_policy_overdrive_trace.sm' -> 'artifacts/zip_inspect/smoke_policy_trace.smc' (6029 bytes)

`

## Semantic Hub Dogfooding
`	ext
vector.turbovec	0.9.0	InProcess	Registered
tool_id: vector.turbovec
name: TurboVec
version: 0.9.0
hub_api_version: 0.1
execution_mode: InProcess
trust_class: InProcessUnisolated
adapter_provenance: semantic-hub-turbovec 0.1.0; turbovec 0.9.0 (MIT, https://github.com/RyanCodrai/turbovec)
operations:
  - vector.index.create determinism=Deterministic mutates_tool_state=true required_capabilities=[VectorIndexCreate,PrivateStorageRead,PrivateStorageWrite]
  - vector.index.describe determinism=Deterministic mutates_tool_state=false required_capabilities=[VectorIndexRead,PrivateStorageRead]
  - vector.index.insert determinism=DeterministicWithSeed mutates_tool_state=true required_capabilities=[VectorIndexMutate,PrivateStorageRead,PrivateStorageWrite]
  - vector.index.remove determinism=Deterministic mutates_tool_state=true required_capabilities=[VectorIndexMutate,PrivateStorageRead,PrivateStorageWrite]
  - vector.search determinism=DeterministicWithSeed mutates_tool_state=false required_capabilities=[VectorSearch,PrivateStorageRead]
  - vector.search.filtered determinism=DeterministicWithSeed mutates_tool_state=false required_capabilities=[VectorFilteredSearch,PrivateStorageRead]
  - vector.index.reset determinism=Deterministic mutates_tool_state=true required_capabilities=[VectorIndexMutate,PrivateStorageRead,PrivateStorageWrite]
  - vector.index.recover determinism=EnvironmentDependent mutates_tool_state=true required_capabilities=[VectorIndexMutate,PrivateStorageRead,PrivateStorageWrite]
{
  "fault_code": null,
  "fault_message": null,
  "logical_sequence": 0,
  "operation_id": "vector.index.create",
  "payload": {
    "bit_width": 4,
    "dim": 8,
    "index": "sample"
  },
  "provenance": {
    "artifact": {
      "digest": "fnv1a64:bb25344d0394b742:18",
      "id": "sample",
      "kind": "turbovec.index"
    },
    "input_digest": "fnv1a64:7934c6dcfeaea117:40",
    "output_digest": "fnv1a64:1bbc0451ce99b63b:40",
    "worker_state_after": "Ready"
  },
  "request_id": "req-1",
  "resource_usage": {
    "input_bytes": 40,
    "output_bytes": 40,
    "wall_time_millis": 35
  },
  "schema_version": 1,
  "status": "Success",
  "tool_id": "vector.turbovec",
  "tool_version": "0.9.0",
  "warnings": []
}
{
  "fault_code": null,
  "fault_message": null,
  "logical_sequence": 1,
  "operation_id": "vector.index.insert",
  "payload": {
    "index": "sample",
    "inserted": 1,
    "len": 1
  },
  "provenance": {
    "artifact": {
      "digest": "fnv1a64:8fec4970133d06a1:98",
      "id": "sample",
      "kind": "turbovec.index"
    },
    "input_digest": "fnv1a64:de4fd118b4913f9c:74",
    "output_digest": "fnv1a64:d0bc278ab8017672:39",
    "worker_state_after": "Ready"
  },
  "request_id": "req-2",
  "resource_usage": {
    "input_bytes": 74,
    "output_bytes": 39,
    "wall_time_millis": 193
  },
  "schema_version": 1,
  "status": "Success",
  "tool_id": "vector.turbovec",
  "tool_version": "0.9.0",
  "warnings": []
}
{
  "fault_code": null,
  "fault_message": null,
  "logical_sequence": 2,
  "operation_id": "vector.search",
  "payload": {
    "hits": [
      [
        {
          "external_id": 1,
          "rank": 0,
          "score": 204.63821
        }
      ]
    ],
    "index": "sample",
    "index_version": 1
  },
  "provenance": {
    "artifact": null,
    "input_digest": "fnv1a64:f2883ec4e6a33dcd:70",
    "output_digest": "fnv1a64:c6ba45a305192ddb:92",
    "worker_state_after": "Ready"
  },
  "request_id": "req-3",
  "resource_usage": {
    "input_bytes": 70,
    "output_bytes": 92,
    "wall_time_millis": 193
  },
  "schema_version": 1,
  "status": "Success",
  "tool_id": "vector.turbovec",
  "tool_version": "0.9.0",
  "warnings": []
}
{
  "fault_code": null,
  "fault_message": null,
  "logical_sequence": 3,
  "operation_id": "vector.search.filtered",
  "payload": {
    "hits": [
      [
        {
          "external_id": 1,
          "rank": 0,
          "score": 204.63821
        }
      ]
    ],
    "index": "sample",
    "index_version": 1
  },
  "provenance": {
    "artifact": null,
    "input_digest": "fnv1a64:af95953335d5ae73:88",
    "output_digest": "fnv1a64:c6ba45a305192ddb:92",
    "worker_state_after": "Ready"
  },
  "request_id": "req-4",
  "resource_usage": {
    "input_bytes": 88,
    "output_bytes": 92,
    "wall_time_millis": 158
  },
  "schema_version": 1,
  "status": "Success",
  "tool_id": "vector.turbovec",
  "tool_version": "0.9.0",
  "warnings": []
}
{
  "fault_code": "ToolDeclaredFailure",
  "fault_message": "ToolDeclaredFailure: InvalidIndexName: index name must not be empty",
  "logical_sequence": 4,
  "operation_id": "vector.index.insert",
  "payload": null,
  "provenance": {
    "artifact": null,
    "input_digest": "fnv1a64:9c8b324c36f64861:36",
    "output_digest": "fnv1a64:cbf29ce484222325:0",
    "worker_state_after": "Ready"
  },
  "request_id": "req-5",
  "resource_usage": {
    "input_bytes": 36,
    "output_bytes": 0,
    "wall_time_millis": 0
  },
  "schema_version": 1,
  "status": "ToolFailed",
  "tool_id": "vector.turbovec",
  "tool_version": "0.9.0",
  "warnings": []
}
ToolDeclaredFailure: ToolDeclaredFailure: InvalidIndexName: index name must not be empty
{
  "fault_code": null,
  "fault_message": null,
  "logical_sequence": 5,
  "operation_id": "vector.search",
  "payload": {
    "hits": [
      [
        {
          "external_id": 1,
          "rank": 0,
          "score": 204.63821
        }
      ]
    ],
    "index": "sample",
    "index_version": 1
  },
  "provenance": {
    "artifact": null,
    "input_digest": "fnv1a64:f2883ec4e6a33dcd:70",
    "output_digest": "fnv1a64:c6ba45a305192ddb:92",
    "worker_state_after": "Ready"
  },
  "request_id": "req-6",
  "resource_usage": {
    "input_bytes": 70,
    "output_bytes": 92,
    "wall_time_millis": 211
  },
  "schema_version": 1,
  "status": "Success",
  "tool_id": "vector.turbovec",
  "tool_version": "0.9.0",
  "warnings": []
}
request_id: req-2
session_id: sess-1
caller_identity: test
tool_id: vector.turbovec
tool_version: 0.9.0
adapter_provenance: semantic-hub-turbovec 0.1.0; turbovec 0.9.0 (MIT, https://github.com/RyanCodrai/turbovec)
operation_id: vector.index.insert
execution_mode: InProcess
determinism: DeterministicWithSeed
trust_class: InProcessUnisolated
privacy_class: ProjectLocal
input_digest: fnv1a64:de4fd118b4913f9c:74
output_digest: fnv1a64:d0bc278ab8017672:39
worker_state_after: Ready
status: Success
fault_code: -

`

## UI Validation
**PENDING:** A human maintainer must run the actual application from the exact release-preparation head and capture real OS screenshots to verify Quad Logic bindings and the GUI calculator.

