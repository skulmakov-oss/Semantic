# Semantic Stable Foundation Dependency Map

Status: SSF-00 sequencing authority
Base snapshot: `89a014b66e7c1e40502dbd764c94bf5f9445677f`

## Serial gate

```text
SSF-00 #1571 truth freeze
  -> SSF-01 #1572 language contract
  -> SSF-02 #1573 Rust-like / Logos coherence
  -> SSF-03 #1574 standard library
  -> SSF-04 #1575 application boundary
  -> SSF-05 #1576 project model
  -> SSF-06 #1577 package baseline
  -> SSF-07 #1578 type and abstraction closure
  -> SSF-08 #1579 ownership and memory positioning
  -> SSF-09 #1580 diagnostics and editor baseline
  -> SSF-10 #1581 compatibility and artifact trust
  -> SSF-11 #1582 applications and onboarding
  -> SSF-12 #1583 qualification and promotion verdict
  -> explicit human release decision
```

Only one phase is active. A later phase may be investigated for dependency
evidence, but it may not change code, contracts, issue status, or public claims
until the preceding exit gate is accepted.

## Phase ownership

| Phase | Inputs from prior phase | Owns | Exit evidence | Current state |
|---|---|---|---|---|
| SSF-00 / #1571 | Umbrella #1569 and live repository/release evidence | Canonical matrix, target `SSF-TARGET-0`, dependency map, status drift correction | Three accepted documents, drift guard, exact merge evidence | Completed |
| SSF-01 / #1572 | Accepted target and source rows | Versioned Rust-like public language contract, included/deferred source features | End-to-end positive/negative contract evidence | Completed |
| SSF-02 / #1573 | Frozen executable contract | Rust-like/Logos Model A or B, diagnostics and package interaction | Honest profile behavior and examples | Completed: Model B |
| SSF-03 / #1574 | Frozen language/profile contract | Versioned language-owned stdlib equivalents and builtin boundary | Positive, negative, compatibility tests and canonical index | Completed |
| SSF-04 / #1575 | Stable value/library carriers | Capabilities, profiles, denial/audit/replay contract | Canonical deterministic file-transform path | Completed |
| SSF-05 / #1576 | Language, stdlib, and effect contracts | Canonical manifest/project layout/commands/path identity | Reproducible project fixtures including `smc test` | Completed |
| SSF-06 / #1577 | Canonical project model | Local package graph, provenance-equivalent record, capability inventory | Multi-package reproducibility and root-security evidence | Completed |
| SSF-07 / #1578 | Stable project/package usage needs | Numeric/text/collections/closures/generics/traits/pattern bounds | Ordinary programs without undocumented workarounds | Completed |
| SSF-08 / #1579 | Selected abstraction surface | Ownership Position A/B, value paths, frames, host ownership, quotas | Positive/negative ownership and deterministic failure suite | **Active** |
| SSF-09 / #1580 | Canonical diagnostics and project symbols | Diagnostic schema, formatter, LSP/editor baseline | CLI/LSP parity, idempotence, protocol fixtures | Blocked by SSF-08 |
| SSF-10 / #1581 | Frozen source/tooling/runtime contracts | Compatibility windows, migration, artifact identity/provenance | Inspect/hash/migration evidence and release manifest policy | Blocked by SSF-09 |
| SSF-11 / #1582 | Complete intended contour | Canonical applications and external onboarding | Clean-user executable application pack | Blocked by SSF-10 |
| SSF-12 / #1583 | All phase evidence | Full gate execution and final verdict report | `PROMOTE`, `PROMOTE WITH EXPLICIT LIMITS`, or `DO NOT PROMOTE` | Blocked by SSF-11 |

## Phase transition log

- **SSF-07 → SSF-08** (transition base `437ea872609bda57bb5bf735dfa3f29376377b8c`): SSF-07's four residual findings (#1633, #1639, #1646, #1861) are repaired and merged; a fifth lifecycle-only correction (#1717, already resolved by merged PR #1873) is reconciled without new code; every other open finding in SSF-07's own `FA-02` module is classified RETURN (to SSF-01, SSF-02, SSF-03, SSF-08, or SSF-12) or DEFER, not silently absorbed or dropped. Full evidence: `ssf07_exit_reconciliation_record.md`. #1578 was found closed prematurely (2026-08-30) against its own evidence trail and this map's own "Active" state; it was reopened and is closed properly only once this transition lands.

## Cross-phase dependencies

| Dependency | Producer | Consumers | Rule |
|---|---|---|---|
| Source grammar/profile version | SSF-01 | SSF-02 through SSF-12 | No later syntax invention outside SSF-01 change control. |
| Logos relationship | SSF-02 | SSF-05, SSF-09, SSF-11, SSF-12 | Tooling/examples must not imply execution absent the selected model. |
| Stdlib identity and versions | SSF-03 | SSF-05 through SSF-12 | Package and compatibility work use one canonical library boundary. |
| Capability/profile contract | SSF-04 | SSF-05, SSF-06, SSF-08, SSF-11, SSF-12 | Package metadata never grants authority. |
| Project identity | SSF-05 | SSF-06, SSF-09, SSF-10, SSF-11, SSF-12 | Diagnostics, packages, and artifacts bind to the same root model. |
| Package/provenance identity | SSF-06 | SSF-10 through SSF-12 | No release trust without reproducible dependency identity. |
| Type/abstraction subset | SSF-07 | SSF-08 through SSF-12 | Ownership, tooling, examples, and compatibility cover only included forms. |
| Ownership position | SSF-08 | SSF-10 through SSF-12 | Public claims and negative examples match Position A or B. |
| Diagnostic/editor schema | SSF-09 | SSF-10 through SSF-12 | Migration and editor surfaces reuse canonical diagnostic truth. |
| Compatibility/artifact model | SSF-10 | SSF-11 and SSF-12 | Examples and release assets record exact identities. |
| Canonical applications | SSF-11 | SSF-12 | They are mandatory qualification inputs, not illustrative-only samples. |

## Existing issue relationships

| Existing issue | Relationship | Constraint |
|---|---|---|
| #1376 | Reusable evidence for SSF-06 package work | Does not bypass SSF-05 or pre-authorize package expansion. |
| #1375 | Reusable evidence for SSF-09 language-server work | Legacy/Workbench bridges are not canonical language authority. |
| #1374 | Reusable evidence for SSF-10 compatibility work | Does not establish stable publication by itself. |
| Workbench, Hub, Pulsar, Atlas, ALM, Studio, Andromeda tracks | Separate/deferred | No code or product scope may be mixed into an SSF phase. |

If a later phase discovers a defect owned by an earlier phase, it must reopen
that owner or create a narrow blocking child. It must not silently widen its own
scope.

## Promotion boundary

SSF-12 produces a recommendation artifact. Neither completion of this map,
merging all phase PRs, nor green CI publishes a stable release. Promotion and
release publication require a separate explicit human decision against exact
artifacts and hashes.
