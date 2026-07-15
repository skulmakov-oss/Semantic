# Denial, Recovery, and Freshness Projection V0

Status: normative v0 contract; crate-private implementation and executable qualification carried by this change

Track: POST-UI / UI-DNA2-7A

Owners:

- `prom-ui::denial_recovery` owns result, batch-denial and recovery presentation plus inert patch composition;
- `prom-ui::connectivity_projection` owns freshness presentation and critical-control availability consistency.

This contract authorizes `ProjectionPatch` construction only. Patch application,
runtime mutation, admission execution and Gate D transition remain unauthorized.

## 1. Authority boundary

```text
caller-supplied outcome != admission execution
denial evidence != Semantic truth
recovery offer reference != recovery authority
ResumeToken reference != validated resume permission
freshness carrier != connection truth
projection success != UI mutation
ProjectionPatch construction != patch application
```

The caller supplies the outcome, reason, evidence references, freshness,
control availability, batch result, recovery offers and patch envelope. The
projector validates and maps that evidence. It does not establish that the
evidence is true or authoritative.

## 2. Ownership split

`denial_recovery` owns:

- denial result projection;
- recovery presentation contracts;
- batch denial projection;
- inert `ProjectionPatch` composition.

`connectivity_projection` owns:

- `Fresh`, `Degraded`, `Stale`, `Offline` and `Resyncing` carriers;
- freshness presentation;
- critical-control availability consistency.

Neither module owns admission policy, capability policy, connection truth,
recovery authority, task truth, runtime effects or patch application.

## 3. Freshness projection

The crate-private model contains:

- `FreshnessState`;
- `ControlCriticality` = `Normal | Guarded | Danger | TaskControl`;
- `ProjectedControlAvailability` = `Available | Unavailable | Hidden | Pending | Stale`;
- a route containing a freshness `BindingTarget` and caller-supplied control routes;
- a fragment containing canonical inert operations and the preserved state.

Stable freshness tokens are:

| State | Token |
| --- | --- |
| Fresh | `fresh` |
| Degraded | `degraded` |
| Stale | `stale` |
| Offline | `offline` |
| Resyncing | `resyncing` |

`project_freshness_fragment` MUST:

1. reject a control-route resource excess before proportional allocation;
2. reject an operation-count excess;
3. sort controls by ascending `StaticNodeId`;
4. reject duplicate control nodes;
5. emit one freshness `SetBindingValue` followed by canonical control operations.

For `Stale`, `Offline` and `Resyncing`, a `Guarded`, `Danger` or
`TaskControl` route MUST NOT carry `Available`. A `Normal` control is not
silently changed: its caller-supplied availability is preserved.

Stable diagnostics are:

```text
CFP_RESOURCE_LIMIT_EXCEEDED
CFP_DUPLICATE_CONTROL_NODE
CFP_INVALID_CRITICAL_CONTROL_AVAILABILITY
CFP_OPERATION_LIMIT_EXCEEDED
```

Freshness projection does not calculate connection state or capability and
does not authorize or deny an action.

## 4. Result taxonomy

The exact categories and patch tokens are:

| Category | Token |
| --- | --- |
| Accepted | `accepted` |
| Denied | `denied` |
| LocalDenied | `local_denied` |
| AdmissionDenied | `admission_denied` |
| CapabilityRejected | `capability_rejected` |
| StaleRejected | `stale_rejected` |
| FreshnessRejected | `freshness_rejected` |
| PartialDenied | `partial_denied` |
| NotApplied | `not_applied` |
| BatchBreak | `batch_break` |
| PendingUnknown | `pending_unknown` |
| Quarantined | `quarantined` |

These distinctions are normative:

```text
Denied != NotApplied
PendingUnknown != Accepted
PendingUnknown != Denied
Quarantined != generic failure
LocalDenied != AdmissionDenied
```

`Accepted` permits an empty reason and MUST NOT claim a denied batch. Every
other result requires a non-empty reason. `AdmissionDenied`,
`CapabilityRejected` and `Quarantined` require a caller-supplied
`SemanticEvidenceRef`.

A critical attempt accompanied by `Stale`, `Offline` or `Resyncing` MUST be
classified as `LocalDenied`.

## 5. Routes and caller evidence

`DenialRecoveryRoute` contains:

- result `BindingTarget`;
- reason `BindingTarget`;
- optional batch collection `StaticNodeId`;
- optional recovery collection `StaticNodeId`;
- freshness route.

`DenialRecoveryEvidence` contains:

- exact `ProjectionPatchEnvelope`;
- result category;
- exact reason text;
- optional `SemanticActionRef`;
- optional `SemanticEvidenceRef`;
- `FreshnessState`;
- critical-attempt flag;
- optional batch projection;
- recovery offers.

The projector MUST NOT invent patch, stream, document, revision, epoch or
sequence identity, or any outcome, reason, freshness, batch, recovery or
control-availability value.

## 6. Recovery offers

Recovery kinds and tokens are:

| Kind | Token | Required reference |
| --- | --- | --- |
| Dismiss | `dismiss` | none; local presentation only |
| Acknowledge | `acknowledge` | `SemanticActionRef` |
| Retry | `retry` | `SemanticActionRef`; a new proposal only |
| Resume | `resume` | `SemanticActionRef` and `ReferenceToken` |
| CancelSuffix | `cancel_suffix` | `SemanticActionRef` |

Each offer contains `order: u32`, `CollectionKey`, kind, optional action
reference and optional resume-token reference.

Offers are sorted by `(order, CollectionKey, kind)`. Duplicate order or key
is invalid. Non-empty offers require a recovery collection route. The
projector presents references but never executes a recovery option.

```text
reference possession != authority
```

## 7. Batch projection

Each batch item contains `order: u32`, `CollectionKey` and exact result
category. Items are canonicalized by `(order, CollectionKey)` and emitted as
`CollectionInsert` operations.

### 7.1 Atomic

Valid atomic evidence is either:

- all items `Accepted`; or
- no item `Accepted`, with one or more denial-like or `NotApplied` outcomes.

Atomic evidence MUST NOT claim an accepted prefix after a denial.

### 7.2 OrderedPartial

Valid ordered-partial evidence contains:

1. zero or more `Accepted` prefix items;
2. exactly one denial-like break item;
3. zero or more `NotApplied` suffix items.

Denial-like break categories are `Denied`, `LocalDenied`, `AdmissionDenied`,
`CapabilityRejected`, `StaleRejected`, `FreshnessRejected` and `Quarantined`.
The declared break order MUST equal the break item's order.

Top-level `PartialDenied`, `BatchBreak` and `NotApplied` require batch
evidence. Non-empty batch evidence requires a batch collection route.

## 8. Deterministic stages and diagnostics

The top-level projection stage order is:

1. `ResourcePreflight`;
2. `RouteValidation`;
3. `FreshnessValidation`;
4. `OutcomeValidation`;
5. `BatchValidation`;
6. `RecoveryValidation`;
7. `OperationConstruction`;
8. `PatchValidation`.

Only diagnostics from the earliest failing stage are returned. Diagnostics
within that stage are sorted canonically and deduplicated.

Stable denial/recovery codes are:

```text
DRP_RESOURCE_LIMIT_EXCEEDED
DRP_MISSING_REASON
DRP_MISSING_EVIDENCE_REF
DRP_INVALID_FRESHNESS_OUTCOME
DRP_DUPLICATE_BATCH_ORDER
DRP_DUPLICATE_BATCH_KEY
DRP_INVALID_ATOMIC_BATCH
DRP_INVALID_ORDERED_PARTIAL_BATCH
DRP_BATCH_BREAK_MISMATCH
DRP_MISSING_BATCH_ROUTE
DRP_DUPLICATE_RECOVERY_ORDER
DRP_DUPLICATE_RECOVERY_KEY
DRP_MISSING_RECOVERY_ROUTE
DRP_RECOVERY_ACTION_REF_MISSING
DRP_RESUME_TOKEN_MISSING
DRP_OPERATION_LIMIT_EXCEEDED
DRP_PATCH_REJECTED
```

`DRP_PATCH_REJECTED` retains the original `ProjectionPatchDiagnostics`
without flattening or losing coordinates. Every failure returns no partial
projection artifact.

## 9. Resource limits

The caller supplies:

- maximum reason bytes;
- maximum batch items;
- maximum recovery offers;
- maximum control routes;
- maximum total operations;
- maximum total projected text bytes.

Input lengths and projected text are checked before proportional output
allocation. Operation capacity is checked before operation construction.
All aggregate arithmetic is checked. The implementation uses deterministic
`alloc` collections only and MUST NOT use randomized hashing, unsafe code or
panic on caller-controlled input.

Input permutation MUST NOT change success, diagnostics or canonical output.

## 10. Patch construction

`project_denial_recovery` constructs operations in this exact order:

1. result `SetBindingValue`;
2. reason `SetBindingValue`, when non-empty;
3. freshness `SetBindingValue`;
4. canonical batch `CollectionInsert` operations;
5. canonical critical-control `SetNodeAvailability` operations;
6. canonical recovery `CollectionInsert` operations.

The exact caller reason is preserved without normalization, truncation or
rewriting. The complete operation list and caller envelope are passed to
`ProjectionPatch::new`.

The inert artifact retains the validated patch, optional action and evidence
references, canonical recovery metadata, optional canonical batch metadata
and operation count. It exposes no apply, execute or dispatch method.

## 11. Implementation mapping

- implementation: `crates/prom-ui/src/denial_recovery.rs`;
- freshness owner: `crates/prom-ui/src/connectivity_projection.rs`;
- executable qualification: `crates/prom-ui/src/ui_dna2_denial_recovery_qualification_tests.rs`;
- inert patch vocabulary: `crates/prom-ui/src/projection_patch.rs`.

The implementation is crate-private and compatible with `no_std + alloc`.

## 12. Explicit non-goals and final posture

This contract does not authorize:

- patch application;
- admission execution or capability evaluation;
- recovery execution;
- live Semantic or connectivity reads;
- task projection;
- runtime or renderer integration;
- public APIs;
- filesystem or runtime loading;
- Gate D transition;
- production promotion.

```text
contract = FROZEN BY THIS CHANGE
crate-private implementation and qualification = CARRIED BY THIS CHANGE
patch construction = INCLUDED
patch application = NOT AUTHORIZED
public API = ABSENT
runtime integration = NOT AUTHORIZED
Gate D = CLOSED
production promotion = NOT AUTHORIZED
FOLLOW-ON AUTHORIZED IMPLEMENTATION SLICE = NONE
```
