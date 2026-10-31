# Q3.10-B2-1 — D6 Normative ABI Freeze Packet (Owner Review)

Status: **PROPOSAL ONLY — D6 NOT APPROVED, NO B2-1 CODE AUTHORIZED**
Repository: `skulmakov-oss/Semantic`
Inspection HEAD: `69cbdb7edb8a5f7df9053cb1a6140f321e5040e9`
Parent decision: `docs/architecture/q3_10_b2_0_r2_owner_architecture_acceptance.md`.
Existing candidate: `docs/spec/evr0_proof_section_v0.md`.

## 1. Proposed D6 decisions for the repository owner

| ID | Exact candidate | Status |
|---|---|---|
| D6a header | `SEMCOD26`, epoch 0, revision 27 | PROPOSED only |
| D6b section ordering | 8-byte header, mandatory legacy `ADT0` table, then exactly one mandatory `EVR0`, then ordinary function envelopes | PROPOSED |
| D6c framing | `EVR0` ASCII tag + `u32le(payload_len)` + exactly the counted payload; no sniffing; no trailing section bytes | PROPOSED |
| D6d capabilities | Preserve the existing V25 capability envelope, no new opcode, no proof-bearing capability bit or runtime effect in B2-1 | PROPOSED |
| D6e schema | `schema=1`, `flags=0`, four bounded canonical tables of propositions/proofs/witnesses/parents and exact fixed fields from B1-R2 | PROPOSED |
| D6f resource limits | Proposed 4 MiB payload, 4096-byte `Bytes16`, propositions/proofs <= 65,536, witnesses/parents <= 262,144, depth <= 256, checked arithmetic and work budgets | NEED R3 budget review |
| D6g error surface | Dedicated structural error for missing/truncated/noncanonical EVR0; map to existing public `RejectReport` until public taxonomy approved | NEED API review |
| D6h issuer | EVR0's issuer label/index is only untrusted metadata; structurally decoded bytes do not mint `VerifiedEvidence` | ARCHITECTURAL D1-D5 |
| D6i admission of rev27 | Fail closed in canonical verifier admission until the qualified gateway and semantic checks are available | SECURITY RECOMMENDATION |
| D6j compatibility | V0..V25 retain exact existing byte interpretation and trusted execution; unsupported header is never relabeled or guessed | HARD INVARIANT |

Do not turn this proposal into a stable ABI just because tests eventually pass.
`docs/spec/evr0_proof_section_v0.md` is still candidate text and requires
normative reconciliation, owner sign-off and golden qualification.

## 2. Actual repository cross-crate impact

The B2-1 structural slice cannot be asserted as `sm-format`-only.

| File / surface | Observed baseline use | Minimum future action (subject to D6 GO) |
|---|---|---|
| `crates/sm-format/src/local_format.rs` | Magic and supported-header list through `MAGIC25`; `ADT0` revision gate | Add version-gated new header/section constants only under approved D6 |
| `crates/sm-format/src/semcode_decode.rs` | `DecodedSemCode {header, adt_descriptors, functions}`; single canonical decoder | Introduce untrusted EVR0 decode output while preserving prior revisions |
| `crates/sm-verify/src/lib.rs` | Exact destructuring of `DecodedSemCode` and exhaustive `DecodeError` mapping | Minimal new-field/error plumbing and **fail-closed** proof-family admission gate; no issuer mint |
| `crates/sm-vm/src/semcode_vm.rs` | Exhaustive `DecodeError` mapping in raw/diagnostic decode | Narrow mapping to deterministic rejection; no new VM token/operation consumption |
| `crates/smc-cli/src/artifact_identity.rs` | Exact `DecodedSemCode` destructuring | Minimal structural display/destructuring compatibility; no evidence authority |
| `crates/sm-ir/src/legacy_lowering.rs` | Producer currently chooses among existing headers, including MAGIC25 | Do **not** automatically promote producer to rev27 in decoder-only stage |
| `tests/public_api_contracts.rs` | Checks supported header set and public API | Update only under documented public API/ABI change, test real compatibility |

Required `DecodeError` enum expansion affects both verifier and VM even
when VM execution semantics remain untouched. It is misleading to claim
that structural decode can change an exported enum without consumer impact.

## 3. Proposed B2-1 scope and non-goals

**IN:** exact approved binary layout, checked decoder, 249-byte section
fixture, full SemCode structural fixture under candidate header, malformed
and legacy fixtures, static parser protection, explicit downstream
non-authorization behavior, public-contract compatibility checks.

**OUT:** an EVR0 issuer that invents trust, a `VerifiedEvidence` token,
source typecheck replay, new `sm-admission` crate, Cargo edits, VM runtime
consumption, new opcode/capability, bootstrap/SHF0 contract mutations.

The producer should continue to emit existing headers until a separate
producer + issuer implementation GO. Manually crafted version-27
structural golden vectors may be decoder-positive but must not be
treated as compiler-generated or verifier-admitted semantically.
The baseline EVR0 section vector is **249 bytes**, payload 241 bytes.
Its `kind=type`, origin span `[0,2)` and fake `compiler_contract`
do **not** prove the later whole-program typecheck proposition.

Potential design issue to resolve at D6: a new header recognized by
`sm-format` could automatically reach current raw VM decoding via
`decode_semcode_envelope`. Review whether revision 27 raw execution
must be rejected until B2-3, or can remain explicitly diagnostic and
non-authoritative. Do not let a structural decoder change silently
broaden trusted VM execution.

## 4. Strict test contract

- Header/section position: exact `ADT0 | EVR0` order, one section,
  exact length, missing section, duplicates, header relabel and legacy
  false positives. V25 and earlier remain byte-for-byte behaviorally
  identical (do not sniff a legacy tag).
- Binary canonicality: reserved bits/bytes, UTF-8 `Bytes16`,
  tables/counts, parent order, zero-based IDs, out-of-bounds indices,
  duplicate ranges, noncanonical strings, arithmetic overflow,
  payload truncation and trailing bytes.
- Fuzz/hostile test: mutate every length, count, pointer and table,
  ensure bounded work and no panic. Distinguish malformed parse from
  unavailable issuer authority in failure diagnostics.
- Compatibility matrix: `sm-format`, `sm-verify`, raw diagnostics,
  `smc-cli` artifact identity and public API tests against V0..V25
  and proposed V26.
- Trust boundary: **zero** structural vectors can instantiate
  `VerifiedEvidence`; no B2-1 artifact is accepted as source-bound
  proof. Existing `VerifiedSemCode` admission remains unchanged for
  legacy artifacts.
- R3 qualification per repo guard: workspace tests, `cargo test --test
  public_api_contracts --quiet`, no_std where relevant, admission guard
  FullPreflight, 7hell CI, adversarial fresh review and staged Harness
  test. Report command exit codes and exact tested commit.

## 5. Owner decision remaining

**D6 ABI FREEZE OWNER GO** must state either:

- Approve the exact version/ordering/schema and specifically approved
  resource/error/admission semantics, with attached normative spec;
  separately issue D7 B2-1 task-scoped implementation GO; OR
- Return precise changed fields/limits, rerun independent layout/hash
  review, and keep production implementation blocked.

**D3 separate governance** remains necessary for the future
`sm-admission` crate, `Cargo.toml` and potential `Cargo.lock`.
D6/B2-1 does **not** authorize that transition and cannot alter Harness
by implication.

**Status: Q3.10-B2-1 D6 REVIEW PACKET PREPARED — ABI NOT FROZEN.**
