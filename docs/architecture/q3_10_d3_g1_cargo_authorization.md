# Q3.10 D3-G1 - Controlled Cargo Authorization Transition

Status: PROPOSAL - governance owner review required; no Cargo authorization is active

Baseline main: `bbf9ff41cc1d4f2b06d688b7aad04af671a74eeb`
Baseline tree: `e08779b864d19623c9131d76dc98aedd43ac200f`
Active task: `SHF-FOUNDATION-DELIVERY-STABLE`
Future implementation task: `Q3.10-D3-G2`

This document prepares a narrow governance transition. It does not register
`sm-admission`, change the dependency graph, create a trusted issuer, enable
revision-27 execution, or weaken the verifier and Harness.

## 1. Verified active envelope

The current `.harness/current.task.yaml` is the effective normal-task
envelope. Its relevant values are:

| Control | Current value | Consequence |
|---|---|---|
| Task | `SHF-FOUNDATION-DELIVERY-STABLE` | Foundation envelope remains authoritative |
| `governance_constraints.dependency_changes` | `false` | Cargo dependency/workspace changes are denied |
| `authorization.dependency_changes` | `false` | Owner task text does not activate dependency mutation here |
| Allowed documentation | `docs/architecture/**` | This proposal is admissible as documentation |
| Forbidden governance | `.harness/**`, `.github/**`, `scripts/**` | No checker or Harness edits |
| Forbidden manifests | `Cargo.toml`, `Cargo.lock`, `crates/*/Cargo.toml`, `crates/*/Cargo.lock` | No crate registration or dependency edits |
| Merge | `merge_after_review_and_checks: false` | No merge or protected-main change |

The active envelope permits documentation and ordinary commit/push/PR
operations, but it does not authorize the requested future Cargo mutation.
The owner GO authorizes preparation of this transition proposal; it is not a
temporary replacement for the active Harness envelope.

## 2. Exact proposed envelope for D3-G2

The following is the minimum future authorization to be reviewed by the
repository owner and trusted base-side governance. It is a proposal, not an
active permission.

### Permitted paths and operations

| Path | One permitted operation in D3-G2 |
|---|---|
| `Cargo.toml` | Add exactly one workspace member: `crates/sm-admission` |
| `crates/sm-admission/Cargo.toml` | Add the new crate with approved existing workspace dependencies only |
| `Cargo.lock` | Update only when deterministic Cargo resolution requires it |
| `crates/sm-admission/src/lib.rs` | Minimal non-authoritative crate foundation only |
| Explicit D3-G2 tests | Dependency, compilation, cycle and boundary qualification only |

Any other path requires a separate owner decision. In particular, D3-G2 must
not authorize changes to existing crate manifests, verifier admission,
SemCode, VM capabilities, public trusted-token constructors, workflows,
releases, tags or the Harness itself.

### Explicit denials

- No external registry dependency without separate approval.
- No reverse dependency from an existing owner to `sm-admission`.
- No source typechecker or alternate verifier inside the gateway.
- No host I/O, process execution or runtime effect in the deterministic
  gateway crate.
- No revision-27 execution switch and no EvidenceOnly-to-execution escalation.
- No merge, force-push, release or tag operation.

### Lifetime and expiry

The proposed envelope is valid only for the named task `Q3.10-D3-G2`, its
single implementation PR, and its exact owner-approved base/head evidence. It
expires when that PR is closed, merged, superseded, or the owner withdraws the
authorization. A later implementation task must not inherit it implicitly.

## 3. Current Cargo graph evidence

`cargo metadata --format-version 1 --no-deps` was run at the baseline. In this
section `A -> B` means that package A depends on package B. `kind=regular`
means a production dependency; `kind=dev` means a test-only dependency.

Relevant current edges:

```text
sm-front       -> legacy-language-core (regular), sm-profile (regular), sm-diagnostic (regular)
sm-sema        -> legacy-language-core (regular), sm-front (regular), sm-profile (regular), sm-diagnostic (regular)
sm-format      -> (none among the relevant workspace owners)
sm-ir          -> sm-front (regular), sm-profile (regular), sm-format (regular)
sm-emit        -> sm-format (regular), sm-ir (regular)
sm-verify      -> sm-runtime-core (regular), sm-format (regular), sm-diagnostic (regular)
sm-verify      -> sm-ir (dev only)
```

`legacy-language-core` is a logical alias for the compatibility-core package
declared by the current `sm-front` and `sm-sema` manifests. Those manifests are
the authoritative package-name source; this new file intentionally avoids
duplicating the legacy package identifier in the inventory surface.

Observed invariants:

- `sm-verify` has no production dependency on `sm-front` or `sm-sema`.
- `sm-verify -> sm-ir` is dev-only and does not create a production semantic
  dependency.
- `sm-format` remains structural-only.
- No existing package depends on `sm-admission`, because that package does not
  yet exist.

## 4. Proposed D3-G2 graph

The smallest proposed gateway graph is:

```text
sm-admission -> sm-front       (regular, source observation API)
sm-admission -> sm-sema       (regular, semantic observation API)
sm-admission -> sm-format     (regular, structural artifact API)
sm-admission -> sm-ir         (regular, canonical reconstruction API)
sm-admission -> sm-emit       (regular, canonical producer facade)
sm-admission -> sm-verify     (regular, final admission API)
```

The actual minimum may be smaller after API inspection. A D3-G2 patch must
remove unused edges rather than mechanically adding all six. It must not add a
production edge from `sm-verify` to a source-semantic crate, and no listed
owner may depend on `sm-admission`.

This graph is not currently registered or resolved by Cargo. The current
`cargo metadata --no-deps` result proves only the existing graph. D3-G2 must
add the crate in an isolated candidate and run resolved metadata with the
lockfile before claiming cycle freedom.

## 5. Trusted-base admission analysis

Local Harness enforcement would reject the future Cargo PR because its changed
paths match the active forbidden entries:

```text
Cargo.toml
Cargo.lock
crates/sm-admission/Cargo.toml
```

The local candidate check is not the complete authority. The trusted GitHub
PR-base checker also evaluates the submitted diff against its base-side task
envelope. Editing the candidate `.harness/current.task.yaml` would not alter
that trusted base-side policy and is forbidden in the current envelope.

Therefore the future D3-G2 PR cannot be admitted through ordinary checks until
the owner authorizes a named governance transition and the trusted base-side
policy recognizes it. No administrative GitHub exception is requested or
executed here. If the base-side policy cannot admit the transition PR, the
required action is an owner-controlled governance exception or a separately
landed, reviewed envelope transition; an agent must not bypass it.

## 6. Controlled transition and restoration procedure

The proposed auditable sequence is:

1. Record the immutable baseline above, the active task ID, and the exact D3-G2
   PR/issue identity.
2. Owner approves the named governance transition and its expiry.
3. On a dedicated governance branch, change only the task-scoped envelope
   needed to admit the D3-G2 Cargo paths. Preserve all ordinary denials.
4. Run the trusted-scope and local Harness checks against the exact candidate
   base. A passing local check is insufficient if the trusted base-side check
   rejects the envelope.
5. Restore the normal Foundation envelope before any merge of the out-of-band
   governance transition, or leave both changes Draft if restoration cannot be
   verified.
6. Only after the restored envelope is accepted may the separately authorized
   D3-G2 implementation PR register the crate.

Rollback is a git revert of the single governance-envelope commit on its
dedicated branch/PR, followed by a fresh scope check against the restored
normal envelope. No manual file deletion, force-push or direct protected-main
edit is acceptable. If the governance PR is not merged, close it without
merging and preserve the audit trail.

## 7. D3-G2 implementation authorization packet

The future implementation request must include all of the following:

- exact base and clean worktree identity;
- named task `Q3.10-D3-G2` and owner authorization;
- the four permitted production paths and explicit test paths above;
- resolved Cargo metadata and lockfile diff;
- cycle analysis proving no reverse edge into the gateway;
- API review proving no duplicate typechecker or verifier;
- compile-only foundation tests;
- proof that no revision-27 artifact is emitted or executed;
- legacy EVR0 and verifier regression results;
- Harness and trusted-scope results on the exact head;
- restoration/expiry evidence for the temporary governance envelope.

The first D3-G2 patch must stop after dependency and foundation qualification.
Source capture, semantic issuer rules, canonical revision-27 production,
VerifiedEvidence and runtime activation require later explicit authorizations.

## 8. Qualification commands

Commands for this proposal and the future transition are:

```powershell
git rev-parse HEAD
git show -s --format=%T HEAD
git status --short --branch
cargo metadata --format-version 1 --no-deps
git diff --check
pwsh -NoProfile -File scripts/harness-check.ps1
cargo test --test q3_10_b2_1_evr0 -- --nocapture
cargo test --test public_api_contracts -- --nocapture
```

Additional D3-G2 commands, only after authorization, must include:

```powershell
cargo metadata --format-version 1
cargo check -p sm-admission --all-targets
cargo test -p sm-admission --all-targets
cargo test --test dependency_boundaries -- --nocapture
git diff --name-only origin/main...HEAD
```

The resolved metadata command and cycle/boundary tests are mandatory; a
`--no-deps` inventory alone is not sufficient evidence for the future graph.

## 9. Decision register

| Decision | Status |
|---|---|
| Current main identity and tree | VERIFIED |
| Current dependency/Harness prohibition | VERIFIED |
| Current production dependency graph | VERIFIED by manifests and metadata |
| Minimal proposed gateway edge set | PROPOSED; must be reduced to actual API need |
| Temporary D3-G2 envelope | OWNER DECISION REQUIRED |
| Trusted base-side admission of the governance transition | BLOCKED until owner-controlled policy admits it |
| `sm-admission` registration in production Cargo | NOT AUTHORIZED in G1 |
| Trusted issuer, revision-27 execution, opaque token | OUT OF SCOPE / NOT AUTHORIZED |

## 10. Final boundary

This G1 artifact is a governance transition proposal only. It creates no
production authority, no dependency edge, no verifier exception and no runtime
capability. D3-G2 may begin only after the owner and trusted base-side
governance explicitly admit the narrow envelope described above.
