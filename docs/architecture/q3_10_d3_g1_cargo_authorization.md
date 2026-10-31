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

## 11. D3-G1-R1 authority state machine

The trusted checker derives its migration mode from the **base** task type,
not from the candidate YAML. In trusted mode it reads the envelope and checker
from the immutable base commit, fetches the candidate only as Git objects, and
evaluates the complete base-to-head path set. The candidate cannot execute its
own checker or authorize its own envelope.

| State | Trusted base task type | Candidate change | Current trusted result | Classification |
|---|---|---|---|---|
| S0 Foundation | `foundation_delivery` | ordinary documentation | admitted when base allow/deny matches | VERIFIED |
| S0 -> S1 bootstrap attempt | `foundation_delivery` | `.harness/current.task.yaml` changed to `governance_migration` | `envelope-immutable` | BLOCKED |
| S0 -> S1 Cargo attempt | `foundation_delivery` | `Cargo.toml`, lockfile or crate manifest | forbidden/outside scope | BLOCKED |
| S1 Governance bootstrap | `governance_migration` already landed in trusted base | envelope-only control-plane change | potentially admitted if transition invariants hold | OWNER DECISION REQUIRED |
| S1 -> S2 scoped envelope | `governance_migration` | one further envelope-only transition | only control-plane paths may change; Cargo payload cannot be in the same transition | PROPOSED / OWNER DECISION REQUIRED |
| S2 scoped Cargo authorization | trusted base contains a narrowly scoped non-migration envelope | D3-G2 Cargo foundation paths | path-admissible only if exact allow/deny rules permit | PROPOSED; content review still required |
| S3 D3-G2 implementation | scoped ordinary task | only authorized Cargo and foundation tests | depends on S2 trusted base | NOT RUN |
| S4 restoration | trusted base contains migration authority | envelope-only restoration to Foundation | possible only as a trusted migration transition | PROPOSED / OWNER DECISION REQUIRED |

### Bootstrap conclusion

There is no ordinary green PR sequence from the current S0 to S1. The base
checker computes `$migration` from the trusted base task type. With the current
`foundation_delivery` base, a changed envelope is rejected before candidate
authorization is considered. A candidate cannot change its own base-side task
type. Therefore S0 -> S1 is **BLOCKED: TRUSTED BOOTSTRAP GAP**.

The earlier generic sequence `S0 -> S1 -> S2 -> S3 -> S4` is not an executable
ordinary PR plan until an owner-controlled trusted base transition establishes
S1. The current PR #2069 is an ordinary documentation payload and does not
establish that transition.

## 12. Isolated checker evidence

The repository fixture `tests/harness_scope_enforcement.ps1` was executed one
case at a time against the checker logic. Each result below is the fixture's
actual result and expected result:

| Scenario | Result | Meaning |
|---|---|---|
| S0 envelope mutation (`N3`) | FAIL as expected | Foundation cannot self-authorize Cargo scope |
| Checker replacement (`N5`) | FAIL as expected | Candidate cannot replace trusted checker |
| Migration payload plus non-envelope file (`N6`) | FAIL as expected | Governance transition is envelope-only |
| Permitted migration envelope (`P2`) | PASS as expected | Existing checker can validate a migration envelope when the trusted base is migration-capable |
| Repeated permitted migration envelope (`P3`) | PASS as expected | Deterministic repeat result |
| Merge gate weakening (`N7`) | FAIL as expected | Protected merge invariant |
| Release/tag weakening (`N8`) | FAIL as expected | Protected release invariant |
| Control-plane expansion (`transition-control-plane`) | FAIL as expected | Migration cannot authorize arbitrary control-plane scope |
| Replacement object Cargo path (`replacement-object`) | FAIL as expected | Forbidden path remains denied |

The existing `landed-transition` fixture returned FAIL with
`envelope-immutable` because the live landed base envelope is Foundation, not
`governance_migration`; its fixture expectation is stale relative to the
current base and is not used as positive evidence.

An attempted direct local trusted invocation for PR #2069 with
`-BaseSha bbf9ff41... -HeadSha 194e5c0...` returned
`checker-provenance executable-must-equal-base-blob`. The Windows checkout's
working copy of the checker is not byte-identical to the immutable base blob,
so this local command is classified **BLOCKED**, not PASS. Hosted
`trusted-scope` for the actual documentation-only PR was successful; that does
not prove a hypothetical Cargo transition.

The trusted workflow itself confirms the relevant authority model: it checks
out the event base, fetches the event head as objects, verifies immutable
identities, and executes the checker from the base checkout. Candidate workflow
or Harness files cannot replace that authority.

## 13. Owner-controlled intervention alternatives

| Alternative | Required action | Trusted-scope behavior | Auditability | Branch-rule bypass |
|---|---|---|---|---|
| A. Narrow governance bootstrap transition | Owner-controlled migration-capable base commit or separately admitted governance PR | Can establish S1, but only through the base-side migration profile | High if one-purpose and reverted/restored | No, if ordinary rules admit it |
| B. Separately reviewed governance base transition | Owner reviews and lands a dedicated envelope transition, then opens D3-G2 from that trusted base | S2 can be evaluated from the landed base | Highest; separates governance from implementation | No, if ruleset accepts it |
| C. One-time administrative merge exception | Owner/admin bypasses the blocked bootstrap check | Would bypass trusted-scope rather than prove it | Lower and requires explicit incident/audit record | Yes; not authorized here |

Alternative C is explicitly not used. No `--admin`, ruleset change, bypass actor,
direct protected push or workflow modification is authorized. A or B is the
smallest supported direction; the current repository does not itself prove
which owner-controlled route is available.

## 14. Exact D3-G2 envelope and enforceability

The proposed D3-G2 candidate envelope is illustrative and must not be landed
without a trusted base transition:

```yaml
task:
  id: Q3.10-D3-G2
  type: foundation_delivery
  mode: active
scope:
  allowed_paths:
    - Cargo.toml
    - Cargo.lock
    - crates/sm-admission/Cargo.toml
    - crates/sm-admission/src/lib.rs
    - tests/dependency_boundaries.rs
    - tests/sm_admission/**
  forbidden_paths:
    - .harness/**
    - .github/**
    - scripts/**
    - crates/*/Cargo.toml
    - crates/*/Cargo.lock
    - bootstrap/**
    - reference/**
    - releases/**
```

This exact illustration is **NOT ENFORCEABLE WITH CURRENT CHECKER/BASE**:

1. The current base does not allow these paths.
2. The current base forbids `Cargo.toml`, `Cargo.lock` and all
   `crates/*/Cargo.toml`.
3. Allow/deny has deny precedence; an allow entry cannot carve an exception out
   of the forbidden manifest pattern.
4. A governance migration candidate is restricted to control-plane paths and
   cannot carry the Cargo payload in the same transition.
5. Even after path admission, the checker does not inspect Cargo TOML content,
   dependency versions, feature meaning or lockfile resolution.

The proposed envelope must therefore be split into a trusted governance
transition and a later ordinary D3-G2 base. A future content validator or
mandatory owner review must enforce the semantic operation boundary.

### Enforcement layers

| Attack | Current path checker | Content validator | Owner review |
|---|---|---|---|
| Existing crate manifest | rejects by forbidden path | not reached | not needed if path remains denied |
| Extra workspace member | path may be allowed by a future exact scope | required | required until validator exists |
| External dependency | not detected by path checker | required | required |
| Version/feature change | not detected | required | required |
| Unauthorized lockfile resolution | path-only; exact content not checked | required | required |
| `.github/**`, `scripts/**`, Harness | rejects by protected path | not reached | not a substitute |
| issuer/token constructor or verifier bypass | outside path-only semantics | required semantic/security tests | required |

The current checker can enforce path scope, immutable workflow/checker,
protected invariants, and base/head identity. It cannot prove that Cargo
content implements only one workspace member with only approved dependencies.

## 15. Restoration and expiration proof

The proposal's “expires after one PR” wording is procedural, not automatic.
The current checker has no timer, PR-closure hook, or automatic restoration
mechanism. PR closure does not mutate the trusted base envelope.

The enforceable restoration invariant is instead:

```text
trusted landed base envelope = normal Foundation envelope
AND Cargo paths are forbidden
=> every later ordinary Cargo PR is rejected by trusted-scope
```

This can be proven only from a new trusted base commit and a fresh negative
trusted-scope run against a Cargo-mutating candidate. Until that restored base
exists, restoration is `BLOCKED` and requires owner action. A written promise
or closing a Draft PR is not proof.

## 16. Updated decision register

| Decision | Status |
|---|---|
| Current S0 Foundation envelope | VERIFIED |
| Candidate cannot self-authorize from S0 | VERIFIED by checker rule and N3/N6 |
| Ordinary S0 -> S1 bootstrap | BLOCKED: TRUSTED BOOTSTRAP GAP |
| Migration envelope-only validation | VERIFIED by P2/P3 fixtures |
| D3-G2 exact path allowlist | PROPOSED |
| D3-G2 exact envelope under current base | NOT ENFORCEABLE |
| Cargo TOML/content semantic restrictions | OWNER DECISION REQUIRED / validator required |
| Trusted base-side governance transition | OWNER DECISION REQUIRED |
| Automatic expiry on PR closure | NOT VERIFIED; not implemented |
| Restoration proof under normal Foundation base | BLOCKED until restored trusted base exists |
| Administrative bypass | NOT AUTHORIZED |
| Issuer, rev27 execution, VerifiedEvidence | OUT OF SCOPE / NOT AUTHORIZED |

## 17. R1 conclusion

Q3.10 D3-G1-R1 does not establish a green ordinary PR path from S0 to S1.
It establishes the exact trusted bootstrap gap and separates path admission from
Cargo-content authorization. The minimum next owner decision is to choose a
supported owner-controlled governance-base transition (A or B above), with an
explicit trusted base commit and restoration plan. Until then D3-G2 remains
blocked and no Cargo authorization is active.
