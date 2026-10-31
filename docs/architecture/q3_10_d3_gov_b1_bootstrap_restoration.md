# Q3.10 D3-GOV-B1 - Trusted Governance Bootstrap and Restoration Protocol

Status: OWNER DECISION REQUIRED - bootstrap capability is not verified

Baseline main: \`47d52d1a23d709e931a1d2dbcbcbb2f618c5b717\`
Baseline tree: \`8d61520465c3370f1b2a54d90630d53ce1568475\`
Merged predecessor: PR #2069 (\`55c152cf...\`)
Normal task: \`SHF-FOUNDATION-DELIVERY-STABLE\`
Future implementation task: \`Q3.10-D3-G2\`

This is an owner execution packet for a possible, narrowly scoped governance
transition. It performs no protected mutation and does not authorize Cargo
registration, revision-27 admission, issuer activation, merge, release or
administrative bypass.

## 1. Verified trust-root facts

The active trusted base envelope has:

\`\`\`text
task.id = SHF-FOUNDATION-DELIVERY-STABLE
task.type = foundation_delivery
governance_constraints.dependency_changes = false
authorization.dependency_changes = false
authorization.merge_after_review_and_checks = false
scope.forbidden_paths includes .harness/**, .github/**, scripts/**,
  Cargo.toml, Cargo.lock, crates/*/Cargo.toml, crates/*/Cargo.lock
constraints.no_merge_without_owner_go = true
\`\`\`

The trusted checker is taken from the immutable PR base. It evaluates the
complete base-to-head Git diff, uses the base envelope, and fetches the
candidate head only as Git objects. Candidate workflow, checker or envelope
content cannot select the trusted authority.

The active GitHub ruleset was read-only inspected:

\`\`\`text
name: protect-main
id: 16491078
target: branch
ref: refs/heads/main
enforcement: active
required status: trusted-scope (integration 15368)
allowed merge methods: squash, rebase
bypass_actors: []
current_user_can_bypass: never
\`\`\`

The ordinary branch-protection endpoint returned \`404 Branch not protected\`;
the repository ruleset is the effective protection source observed here. No
administrative bypass capability was established.

## 2. State machine

\`\`\`text
S0  PROTECTED FOUNDATION
 |  owner-controlled bootstrap, not ordinary candidate self-authorization
 v
S1  GOVERNANCE-CAPABLE TRUSTED BASE
 |  envelope-only migration transition
 v
S2  TEMPORARY D3-G2 AUTHORIZATION
 |  one bounded implementation PR
 v
S3  SM-ADMISSION FOUNDATION LANDED
 |  separately authorized restoration transition
 v
S4  PROTECTED FOUNDATION RESTORED
\`\`\`

| Edge | Trusted base requirement | Candidate change | Expected trusted result | Status |
|---|---|---|---|---|
| S0 -> S1 | \`foundation_delivery\` | change Harness task to migration | \`envelope-immutable\` | BLOCKED |
| S0 -> Cargo | \`foundation_delivery\` | Cargo manifest/lock/crate manifest | forbidden or outside scope | BLOCKED |
| S1 establishment | trusted \`governance_migration\` base | owner-controlled bootstrap | owner operation unavailable/unverified | NOT VERIFIED |
| S1 -> S2 | trusted migration base | exactly one envelope-only transition | conditionally supported | PROPOSED |
| S2 -> S3 | trusted scoped ordinary base | bounded D3-G2 Cargo/test paths | path-only admission; content gates required | PROPOSED |
| S3 -> S4 | trusted migration-capable base | envelope-only restoration | possible only from suitable trusted base | NOT VERIFIED |
| S4 -> unauthorized Cargo | restored Foundation base | Cargo mutation | forbidden path rejection | PROPOSED |

The checker derives migration mode from the **trusted base task type**. A
candidate cannot turn S0 into S1 by submitting a different task type in its
own YAML. No ordinary green PR path from S0 to S1 has been proven.

## 3. Recommended mechanism and feasibility

### Recommended: B, dedicated trusted governance transition

The minimum safe sequence is:

1. Establish a trusted base whose task type is explicitly
   \`governance_migration\`.
2. Use that base only for the envelope-only transition to S2.
3. Open D3-G2 from the trusted scoped base.
4. Land only the minimal \`sm-admission\` foundation.
5. Execute a separately authorized restoration transition to S4.
6. Prove post-restoration rejection of Cargo, Harness, workflow and
   self-authorization attempts from the restored trusted base.

This is recommended design, not an executed owner operation. The current
repository evidence does not verify that the owner can establish S1.

| Alternative | Assessment |
|---|---|
| A. Ordinary one-time bootstrap PR | Rejected: S0 base checker rejects envelope mutation |
| B. Dedicated trusted governance transition | Recommended, but owner capability and exact admission path are NOT VERIFIED |
| C. Administrative exception | Not authorized; no bypass actor and \`current_user_can_bypass=never\` observed |

Final bootstrap feasibility:

\`\`\`text
BLOCKED - OWNER-CONTROLLED BOOTSTRAP CAPABILITY UNVERIFIED
\`\`\`

No \`--admin\`, ruleset edit, bypass actor, direct protected push or workflow
modification is used.

## 4. Owner execution packet

| Step | Operation | Class | Evidence |
|---|---|---|---|
| 0 | Verify exact main SHA/tree, clean checkout, merged #2069 | AUTOMATABLE | Git/GitHub metadata |
| 1 | Select mechanism and name governance task | OWNER APPROVAL REQUIRED | Durable task record |
| 2 | Establish trusted S1 migration base | PRIVILEGED OWNER ACTION | Trusted main SHA and trusted-scope |
| 3 | Submit S1 -> S2 envelope-only transition | OWNER APPROVAL REQUIRED | One-file migration diff |
| 4 | Confirm S2 exact path scope | AUTOMATABLE | Trusted checker plus content review |
| 5 | Run D3-G2 implementation PR | OWNER APPROVAL REQUIRED | Exact scope, Cargo resolution, tests |
| 6 | Freeze S3 and stop implementation | AUTOMATABLE | Exact candidate evidence |
| 7 | Submit S3 -> S4 restoration transition | PRIVILEGED OWNER ACTION | Trusted migration base and restoration diff |
| 8 | Prove restored negative cases | AUTOMATABLE | Fresh trusted-scope results |
| 9 | Close temporary authorization record | OWNER APPROVAL REQUIRED | Durable closeout; not automatic |

No step grants issuer authority, revision-27 execution or
\`VerifiedEvidence\` construction.

## 5. Exact D3-G2 scope and enforcement

The temporary S2 window may authorize only:

\`\`\`text
Cargo.toml
Cargo.lock                         # only when resolution changes
crates/sm-admission/Cargo.toml
crates/sm-admission/src/lib.rs
explicitly named dependency/boundary tests
\`\`\`

Content is limited to one workspace member, one new manifest, approved
existing workspace dependencies, a compilation-only foundation and boundary
tests. It must not authorize existing crate manifests, arbitrary members,
external dependencies, unrelated versions/features, verifier/VM/SemCode
changes, issuer/token constructors, workflows, scripts, releases, tags or a
change to \`merge_without_owner_go\`.

Three enforcement layers are required:

1. **Trusted path authorization**: exact Git paths, deny precedence, immutable
   checker/workflow controls and identity.
2. **Cargo content/dependency validation**: workspace member count, manifest
   ownership, dependency source/version/features and lockfile delta.
3. **Owner review**: invariants not encoded in a trusted validator.

The current checker provides layer 1 only. An allowlist cannot override matching
forbidden patterns because deny wins, and the checker cannot prove that Cargo
content adds only one member with only approved dependencies. A content
validator is a separate future authorization prerequisite.

## 6. Preflight and transition checks

Before each operation:

\`\`\`powershell
git status --short --branch
git rev-parse HEAD
git show -s --format=%T HEAD
git diff --check
git diff --name-only <trusted-base>...<candidate-head>
cargo metadata --format-version 1 --no-deps
pwsh -NoProfile -File scripts/harness-check.ps1
\`\`\`

For S1/S2/S4 the trusted workflow must prove canonical repository/main,
event base/head identity, checker equality with the base blob, intended
base-side envelope and \`trusted-scope: SUCCESS\`.

Local Harness PASS is informative only and never authorizes a protected
transition.

## 7. Restoration guarantee

The current checker has no timer, PR-closure hook, automatic expiry or
automatic envelope restoration. “One PR only” is procedural unless the
trusted base records the restored Foundation envelope.

S4 is complete only when a fresh trusted main base proves:

\`\`\`text
task.type = foundation_delivery
Cargo.toml and Cargo.lock are forbidden
crates/*/Cargo.toml and crates/*/Cargo.lock are forbidden
.harness/**, .github/** and scripts/** remain protected
merge_without_owner_go remains false
no_release_or_tag remains true
\`\`\`

Fresh post-restoration negative tests must show:

| Candidate | Required result |
|---|---|
| Unauthorized Cargo mutation | trusted-scope REJECT |
| Existing crate manifest | trusted-scope REJECT |
| Additional workspace member | path reject or content-validator reject |
| Candidate Harness edit | trusted-scope REJECT |
| Candidate workflow edit | trusted-scope REJECT |
| Candidate checker replacement | trusted-scope REJECT |
| Candidate self-authorization | trusted-scope REJECT |

If S3 fails, hosted CI fails, approval is withdrawn, or the window remains
open unexpectedly: stop D3-G2, preserve exact evidence, do not merge, and
invoke owner-controlled restoration if available. If no trusted restoration
base exists, remain BLOCKED. Concurrent PRs targeting S1/S2/S3 invalidate
stale transition evidence and require a fresh exact base.

## 8. Isolated qualification evidence

The existing immutable-checker fixture suite demonstrated:

| Test | Result |
|---|---|
| Foundation self-authorization | FAIL as expected |
| Candidate checker replacement | FAIL as expected |
| Migration plus implementation payload | FAIL as expected |
| Migration envelope-only transition | PASS as expected in fixture |
| Merge/release invariant weakening | FAIL as expected |

These disposable fixtures prove checker semantics, not owner ability to
establish S1. A direct local trusted invocation previously failed
\`checker-provenance executable-must-equal-base-blob\` because the Windows
working-copy checker was not byte-identical to the immutable base blob. This is
BLOCKED evidence, not substituted local PASS. The only acceptable proof of
S1/S2/S4 is hosted \`trusted-scope: SUCCESS\` from the immutable event base.

## 9. Recovery and owner decisions

| Failure | Action |
|---|---|
| Bootstrap cannot establish S1 | stop with exact bootstrap blocker |
| S2 check fails | do not submit D3-G2 |
| D3-G2 local/hosted checks fail | no merge; owner decides retry/restoration |
| Approval withdrawn | stop and restore if trusted route exists |
| Window remains open | no further PR; owner closeout required |
| Restoration fails | remain BLOCKED; do not claim Foundation restored |
| Concurrent intermediate PR | freeze and invalidate stale evidence |

Owner must decide:

1. Whether an actual supported operation exists to establish trusted
   \`governance_migration\` S1.
2. The exact privileged actor, operation, base SHA and durable audit record.
3. The S2 envelope, expiry and closeout procedure.
4. Separate D3-G2 authorization after S2 is trusted.
5. A Cargo-content validator or explicit owner-review gate.
6. Required S4 evidence before declaring the temporary window closed.

## 10. Final classification

The protocol is complete as an owner decision packet, but entry from current
trusted S0 is not executable with ordinary PR authority:

\`\`\`text
Q3.10 D3-GOV-B1 - BLOCKED: OWNER-CONTROLLED BOOTSTRAP CAPABILITY UNVERIFIED
\`\`\`

No live Harness change, Cargo transition, issuer activation, revision-27
execution, protection bypass or merge was performed.
