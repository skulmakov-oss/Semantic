# Hub CLI metadata presentation v0

Status: current `smc hub audit` default-output contract

`smc hub audit --request <request-id>` accepts the original, validated
`HubRequestId` and uses it to find one persisted `HubAuditRecord`. This lookup
is deterministic. The CLI presents a human-readable summary; it is not an
export of the canonical audit record.

The persisted audit format remains defined by
[`hub_api_v0.md`](hub_api_v0.md#10-canonical-audit-encoding). In particular,
the record retains its request, session, and caller fields for provenance.
`request_id` remains the public deterministic lookup handle and is emitted
verbatim by default audit output. Raw `session_id` and `caller_identity` are
not disclosed by the presentation layer.

## Field classification

| Field | Persisted audit | Lookup | Default stdout classification | Evidence |
| --- | --- | --- | --- | --- |
| `request_id` | required | required input | `SAFE_VERBATIM` | `HubAuditRecord` and canonical field 2 retain it; the CLI accepts it for lookup and the public audit response echoes it. |
| `session_id` | required | no | `SENSITIVE_OMIT` | Canonical field 3 is a correlation handle. |
| `caller_identity` | required | no | `SENSITIVE_OMIT` | Canonical field 4 identifies the issuing component or user. |
| `tool_id` | required | no | `SAFE_VERBATIM` | Validated tool identifier and required human inspection context. |
| `tool_version` | required | no | `SAFE_VERBATIM` | Registered descriptor version. |
| `adapter_provenance` | required | no | `SENSITIVE_REDACT` | Canonical field 7 is free text, not a closed public vocabulary. |
| `operation_id` | required | no | `SAFE_VERBATIM` | Validated operation identifier and required inspection context. |
| `execution_mode` | required | no | `SAFE_VERBATIM` | Closed `HubExecutionMode` vocabulary. |
| `determinism` | required | no | `SAFE_VERBATIM` | Closed determinism-class vocabulary. |
| `trust_class` | required | no | `SAFE_VERBATIM` | Closed trust-class vocabulary. |
| `privacy_class` | required | no | `SAFE_VERBATIM` | Closed privacy-class vocabulary; communicates handling classification. |
| `input_digest` | required | no | `SENSITIVE_REDACT` | A non-cryptographic correlation fingerprint with byte length. |
| `output_digest` | required | no | `SENSITIVE_REDACT` | A non-cryptographic correlation fingerprint with byte length. |
| `worker_state_after` | required | no | `SAFE_VERBATIM` | Closed worker-state vocabulary. |
| `status` | required | no | `SAFE_VERBATIM` | Closed reply-status vocabulary. |
| `fault_code` | required | no | `SAFE_VERBATIM` | Closed fault-code vocabulary or `-`. |

`SENSITIVE_REDACT` renders the literal `<redacted>`. `SENSITIVE_OMIT` emits
no line. The presentation contract defines no raw-output flag, alternate JSON
mode, environment switch, stderr route, logging route, or compatibility path.

## Session summary presentation

`HubSessionSummary` retains raw `session_id` and `caller_identity` internally
for session validation and provenance. The external `smc hub session` NDJSON
`session_summary` is a separate presentation object: it omits both fields.
Per-request reply objects retain their public `request_id`, so a consumer can
correlate every result and later use that value for `smc hub audit --request`
without receiving a raw session or caller value.

## Boundary

The command is for default human inspection of tool, operation, execution,
classification, worker, and outcome data. It must not expose raw session
correlation handles, caller identity, unbounded adapter text, or weak
content-correlation fingerprints. The validated `request_id` remains the
public lookup/correlation handle and may be echoed verbatim. Callers retain
the request ID supplied to the command; the audit response may echo that same
validated public handle, and it can be reused for deterministic lookup.
