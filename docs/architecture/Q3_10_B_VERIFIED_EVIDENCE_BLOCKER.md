# Q3.10-B VerifiedEvidence — Contract Blocker

Status: BLOCKED — proposal only; no implementation or contract migration.

Baseline: `69cbdb7edb8a5f7df9053cb1a6140f321e5040e9`

## Observed current authority

`sm-verify` issues `VerifiedSemCode` and `VerifiedEntrySemCode` only after
validating the existing SemCode envelope, instruction framing, sections,
capabilities, resources and entry resolution. The current SemCode contract has
no evidence/proof section, issuer namespace, epoch, source-set binding,
proposition identity or parent-proof relation.

The existing public `Evidence`-like carriers are ordinary data. A private Rust
wrapper around them would prevent construction through that Rust type, but it
would not establish that the wrapped claim was derived by an authorized issuer
or that hostile SemCode could not introduce an equivalent claim.

## Exact blocker

The requested guarantees I-01 through I-06 require an admission boundary that
does not exist in the current normative contract. At minimum, one of the
following must be authorized and specified before implementation:

1. A new SemCode proof section and revision/ABI, decoded by `sm-format`,
   independently validated by `sm-verify`, and bound to the artifact's source,
   compiler/contract identity, proposition identity, epoch, policy namespace,
   authorized operation and parent proofs; or
2. A separately specified non-SemCode trusted issuer protocol that creates a
   session-scoped capability handle from independently validated evidence,
   with an explicit rule for how hostile SemCode is prevented from asserting or
   consuming that handle.

Adding only a public/private Rust struct, a digest, a `valid` flag, or a public
integer handle would not satisfy the security property and is therefore not
implemented.

## Migration proposal

Before code changes, the owner should approve a focused contract containing:

- proof section identity and version;
- canonical proposition tuple
  `(kind, ast_owner, semantic_scope, type_obligation)`;
- witness and parent-proof encoding and duplicate rules;
- source/file/source-set and compiler-contract bindings;
- issuer/session/epoch/policy binding;
- deterministic derivation rules for `N/F/T/S`;
- verifier rejection taxonomy for forged, stale, cyclic, downgraded and
  cross-context proofs;
- VM handling, if a runtime value is required;
- legacy SemCode compatibility and the exact revision floor.

The migration must include positive and hostile-SemCode fixtures, replay and
cross-context tests, deterministic artifact rules, and an explicit decision on
whether the feature is experimental or public language/ABI surface.

## Architecture comparison

| Design | Current status | Why it is insufficient/viable |
|---|---|---|
| Source-level opaque nominal type | Not sufficient alone | Frontend rejection does not constrain hand-crafted SemCode or prove issuer authority. |
| Verifier-issued typed handle | Requires contract | Viable only with a specified evidence input, issuer rule, context binding and consumer boundary. |
| Combined source + verifier design | Preferred future direction | Can address source ergonomics and hostile artifacts, but requires the proof section/ABI and full qualification. |

## Decision

No production files, SemCode revision, opcode, verifier admission rule, VM
runtime type, public language rule, dependency, Harness or frozen C0 identity
was changed in Q3.10-B. The correct current result is a contract blocker, not
an opaque-evidence qualification.
