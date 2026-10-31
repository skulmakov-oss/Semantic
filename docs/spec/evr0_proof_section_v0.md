# EVR0 Proof Section v0 (Candidate)

Status: **PROPOSED — not implemented and not normative**

This document is the Q3.10-B1 contract candidate. It does not add a header,
capability, opcode, section or verifier rule to the current implementation.

## 1. Integration boundary

The candidate introduces a new header family, provisionally named `SEMCOD26`,
with revision `27`. These names and numbers are reserved only as a proposal;
the owner must approve the revision allocation before B2.

For the proposed revision, the artifact layout is:

```text
header[8] | ADT0 | EVR0 | function-envelope[0..n]
```

`ADT0` remains mandatory and is decoded exactly as today. `EVR0` is mandatory
for the proposed proof-bearing family and is revision-derived, never
content-sniffed. Legacy headers keep their current layout and have no implicit
EVR0 authority. EVR0 is not a capability bit and is not placed inside a
function envelope.

## 2. Section framing

All integers are unsigned little-endian. All length arithmetic is checked before
adding or slicing. A proposed artifact contains exactly one EVR0 section:

```text
EVR0 section :=
    tag:"EVR0"[4]
    payload_len:u32
    payload[payload_len]
```

`payload_len` must be nonzero, no larger than `MAX_EVR0_PAYLOAD` (proposed
4096 KiB), and must end exactly at the first function envelope. Missing,
duplicated, truncated, overflowing or trailing EVR0 bytes reject. Unknown
section tags at this position reject; the decoder never searches for EVR0.

The payload is:

```text
payload :=
    schema:u16                 // exactly 1
    flags:u16                  // exactly 0
    source_set:Digest32
    compiler_contract:Digest32
    epoch:u16
    policy_namespace:Bytes16
    issuer_namespace:Bytes16
    proposition_count:u32
    proof_count:u32
    witness_count:u32
    parent_count:u32
    propositions[proposition_count]
    proofs[proof_count]
    witnesses[witness_count]
    parents[parent_count]
```

`Bytes16` is `byte_len:u16` followed by exactly that many bytes. Lengths are
bounded by `MAX_EVR0_STRING_BYTES` (proposed 4096), and values must be UTF-8
where the field is declared textual. Names are nonempty and canonical UTF-8.

All four count fields are bounded by proposed limits:

| Item | Limit |
|---|---:|
| propositions | 65,536 |
| proofs | 65,536 |
| witnesses | 262,144 |
| parents | 262,144 |
| derivation depth | 256 |
| total payload | 4 MiB |

The verifier must check each multiplication and cursor advance with checked
arithmetic. Any overflow is a deterministic rejection.

## 3. Proposition identity

### Canonical proposition table

The payload contains an explicit proposition table. Its four little-endian
counts are proposition_count, proof_count, witness_count and parent_count,
followed in that order by the proposition, proof, witness and parent arrays.
proposition_index is a zero-based index into this table; it is never a proof
ID, AST node number or source offset. The table is sorted by the complete
proposition tuple below and indices are assigned after sorting. Each proposition
occurs once. Multiple proofs of one proposition use the same proposition_index
and distinct proof IDs or derivations.

Every proof carries the exact identity:

```text
P = (kind, ast_owner_id, semantic_scope_id, type_obligation_id)
```

Encoded as:

```text
Proposition :=
    kind:Bytes16
    ast_owner_id:u32
    semantic_scope_id:u32
    type_obligation_id:u32
```

`kind` is a nonempty canonical UTF-8 identifier from the approved rule
registry. `0xffffffff` is not a wildcard for any numeric identity. Equality is
bytewise over the complete tuple. Text labels, source positions and digests
alone never establish proposition equality.

## 4. Witnesses and derived quad state

Each witness is:

```text
Witness :=
    witness_id:u32
    proposition_index:u32
    polarity:u8                 // 0 = negative, 1 = positive
    reserved:u8[3]              // exactly zero
    file_digest:Digest32
    source_start:u32
    source_end:u32
    origin_digest:Digest32
```

`source_start <= source_end`; byte ranges are checked against the trusted source
origin supplied to the verifier. The manifest must contain the referenced
file digest, and the verifier must validate the full path-to-bytes binding.
`origin_digest` commits to the exact origin
record defined by the issuer context, not to a caller-provided arbitrary label.
Witness IDs are unique and sorted strictly ascending. Within a proposition,
positive and negative witness lists are derived by polarity and sorted by
`witness_id`. The encoded state is never trusted because EVR0 contains no state
byte: the verifier derives it as:

```text
none / none       = N
negative only     = F
positive only     = T
both              = S
```

Duplicate witness IDs, a witness attached to an invalid proposition, or a
cross-file origin mismatch rejects.

## 5. Proofs and derivation

### Parent table and authority reference

Each parent entry is exactly target_proof_id:u32, little-endian. parent_first
and parent_count select the half-open range
parents[parent_first .. parent_first + parent_count]. The range must be in
bounds, strictly increasing by target_proof_id, duplicate-free and owned by
the containing proof. A target must be a proof ID smaller than the containing
proof_id; forward references, self references and cycles reject.

The parent table is globally ordered by containing proof ID and then target
proof ID. Parent ranges are contiguous and deterministic. A parent entry is not
a free-standing proof claim and cannot be reused outside its selected range.

authority_context is an index into the trusted admission context's authority
table, not an authority-bearing caller identifier. The EVR0 number is only a
reference; an unknown or mismatched entry rejects. The resolved trusted entry
binds issuer namespace, policy namespace, epoch, source-set identity, allowed
operation and rule-registry version. No authority is inferred from the number.

Each proof is:

```text
Proof :=
    proof_id:u32
    proposition_index:u32
    rule_id:Bytes16
    authority_context:u32
    witness_first:u32
    witness_count:u32
    parent_first:u32
    parent_count:u32
    authorized_operation:u8
    reserved:u8[3]              // exactly zero
```

`rule_id` must be in the frozen/approved EVR0 rule registry. The referenced
witness and parent ranges must be in bounds, sorted, duplicate-free and owned
by this proof. Parent proof IDs must be strictly less than the child proof ID;
this canonical ordering proves acyclicity without recursive traversal. A parent
must have the same proposition identity and compatible `authority_context`.

`authorized_operation` is an enumerated registry value, not a free-form
permission. Unknown values reject. The verifier recomputes the rule result and
requires the proof's witnesses, parents and operation to match that result.

## 6. Context and issuer binding

### Canonical source identity

Digest32 is SHA-256 over a domain-separated byte sequence. The exact domains
are EVR0-FILE, EVR0-MANIFEST and EVR0-ORIGIN, each followed by one zero byte.
File identity is:

    SHA256(EVR0-FILE || 0 || u32le(path_len) || canonical_path_utf8
           || u64le(byte_len) || source_bytes)

Manifest identity is:

    SHA256(EVR0-MANIFEST || 0 || u32le(file_count)
           || repeated(file_digest || u32le(import_count)
                       || repeated(import_file_digest)))

Origin identity is:

    SHA256(EVR0-ORIGIN || 0 || file_digest || u32le(start)
           || u32le(end) || source_bytes[start:end])

The manifest file list is sorted by canonical project-relative UTF-8 path;
imports are sorted by resolved manifest index. Separator, case, symlink and
root-confinement rules remain those of the frozen bootstrap source-set
contract. EVR0 does not redefine that contract. The manifest binds the
complete imported graph. A same-offset witness from another file has a
different file_digest and fails origin binding.

**Two different source-set identities must never be conflated.** The serialized
EVR0 `source_set:Digest32` is exclusively the raw 32-byte
`SHA256(EVR0-MANIFEST || 0 || ...)` result above. The frozen
`shf0-source-set-v1` identity is independently computed using its unchanged
`SHA256("SHF0-SOURCE-SET" || NUL || "v1" || NUL || u64be(file_count) || ...)` framing and
has its existing `sha256:<lowercase-hex>` presentation. Its digest is **not**
interchangeable with EVR0 `source_set`, and a verifier must never compare the
two bytewise or substitute one for the other. A bootstrap-aware authenticated
issuer context carries both typed values, recomputed from the **same** trusted
files and the frozen exact bootstrap membership/order, and attests the mapping.
If either digest or their shared source-file set does not match, reject as
`EvidenceContextMismatch`. EVR0 does not change the frozen bootstrap contract.

`source_set`, `compiler_contract`, `epoch`, `policy_namespace` and
`authority_context` are admission inputs, not self-authenticating claims.

An EVR0 byte string alone can never create a runtime `VerifiedEvidence` handle.
Admission requires a trusted issuer context supplied out-of-band by an
authorized admission service. That context must bind:

- exact source-set and source-file byte digests;
- the approved compiler and semantic-contract identities;
- epoch and policy namespace;
- issuer authorization and session identity;
- the permitted operation;
- the rule registry version.

No issuer ID, signature-looking field, digest or `valid` flag inside EVR0 is
accepted as issuer authentication. If the issuer context is unavailable or its
binding differs, admission rejects.

## 7. Canonical ordering

### Rule registry and independent checks

The EVR0 rule registry is verifier-owned and versioned by
`compiler_contract`. Identifiers are **exact UTF-8 bytes**, not abbreviations
or human-readable aliases. The minimum **candidate** registry is:

| Rule | Proof shape | Independently checked condition |
|---|---|---|
| `POS-WITNESS-1` | one positive witness, zero parents, `authorized_operation=0` | trusted semantic evaluator confirms the precise positive observation for P and source origin |
| `NEG-WITNESS-1` | one negative witness, zero parents, `authorized_operation=0` | trusted semantic evaluator confirms the precise negative observation for P and source origin |
| `JOIN-SAME-P-1` | zero local witnesses, exactly two distinct earlier parent proofs, `authorized_operation=0` | both parents are already admitted for identical P and compatible authenticated context; derive the union of their authenticated witness sets |
| `OP-T-ONLY-1` | zero local witnesses, exactly one earlier admitted parent proof, `authorized_operation=1` | parent state is exactly T, operation 1 is permitted by the trusted authority table, and P/context match |

The sole proposed `authorized_operation` values in this minimal profile are
`0` (evidence derivation only, **no** privileged operation) and `1` (proof-
required T-only operation). Unrecognized values reject. `JOIN-SAME-P-1`
deduplicates an observation by its authenticated witness ID, never by an
untrusted label; conflicting polarities for one P yield S, **not** a malformed
proof. A valid S is rejected only at a T-only operation, as
`EvidenceOperationDenied`.

**Independent semantic verification is mandatory.** The trusted evaluator must
resolve P against the authenticated source set, rebuild the expected semantic
obligation and re-evaluate the polarity from trusted source bytes and the
approved type/rule semantics. It must return the expected (P, file identity,
byte range, polarity, origin digest) observation or reject. A syntactically
valid origin, matching file hash, arbitrary compiler declaration or `rule_id`
is not a proof of the observation. The EVR0 producer is untrusted and must
not provide this evaluator's answer as authority. For external facts not
reproducible from source, a separately authenticated issuer must validate
the observation using a versioned external-fact rule; **none of these four
candidate rules confers generic external-fact authority**.

**Remaining normative dependency:** the precise trusted evaluator interface,
supported proposition kinds and language-semantic algorithms, and issuer
authentication are NOT yet assigned to a qualified implementation. Therefore
this proposed registry describes rejection/derivation boundaries but does
**not** yet authorize acceptance of a general type-obligation proof in B2.
Until those interfaces are explicitly specified and qualified, the verifier
must reject any proof whose underlying observation cannot be independently
established. This open item must remain in the owner decision register.

- Propositions are ordered by `(kind bytes, ast_owner_id, semantic_scope_id,
  type_obligation_id)`.
- Proofs are ordered by `proof_id`, with IDs contiguous from zero.
- Witnesses are ordered by `witness_id`, contiguous from zero.
- Parents are ordered by `proof_id`.
- No map or set is serialized using host iteration order.
- No trailing bytes, padding, alternate integer widths or duplicate sections are
  canonical.

## 8. Reproducible EVR0 section vectors (proposal, not executable qualification)

The following is a **structurally positive** section vector, NOT a
verifier-admitted proof: it intentionally lacks a trusted issuer context and
a qualified semantic evaluator for proposition kind `type`. The expected
result is successful EVR0 *structural decoding* and **failed trust admission**
(`EvidenceIssuerUnavailable`). Never count it as a semantic-positive test.

Exact source fixture (UTF-8, LF, no BOM): `fn main() { return; }\n`
where `\n` denotes the single newline byte `0a`; byte length 22.
Canonical relative path: `probe.sm` (8 UTF-8 bytes). Import count: 0.
Witness span `[0,2)` contains the ASCII bytes `66 6e` (`fn`).

Computed from the domain-separated algorithms in section 6:

```text
file_digest     1e26d7e912d6486d3fbf7cca277320f8636bb83d323c2ea8c2164c124b3cfc09
source_set      20d35f55ccd21e8c2491050ac14c432868517ff28ca5d1c0c9ba7f592711eb52
origin_digest   c9727023caaf7dbfc9fdd1df2bf558fcac39a76b8cd89b1c7292294f9ac75fe2
```

Schema=1, flags=0, compiler_contract=32 bytes of `22` (a fixture
placeholder, **not** approved contract authority), epoch=1,
policy_namespace=`p`, issuer_namespace=`i`, counts
(propositions, proofs, witnesses, parents)=(1,1,1,0).
Proposition: kind=`type`, (ast_owner, scope, obligation)=(1,2,3).
Proof: ID=0, proposition_index=0, exact rule ID=`POS-WITNESS-1`,
authority index=0, witness range [0,1), empty parent range,
**authorized_operation=0** (derivation only). Witness: ID=0, index=0,
positive polarity, file_digest above, source span [0,2), origin_digest above.
All reserved bytes zero.

Offsets from the **payload** start:

| Offset | Length | Meaning |
|---:|---:|---|
| 0 | 2 | schema |
| 2 | 2 | flags |
| 4 | 32 | EVR0 source_set digest |
| 36 | 32 | compiler_contract fixture |
| 68 | 2 | epoch |
| 70 | 3 | policy Bytes16 |
| 73 | 3 | issuer Bytes16 |
| 76 | 16 | four u32 counts |
| 92 | 18 | proposition |
| 110 | 47 | proof with full 13-byte rule name |
| 157 | 84 | witness |
| 241 | 0 | parent table |

`payload_len=241` (`f1 00 00 00` little-endian); section header
is `45 56 52 30 f1 00 00 00`. Total section size is **249 bytes**; section SHA-256 is
`cd91d079b4c5846f8bd4e9edf13fc2136d70fe935d808f6d17fdd1f318bd1870`.
All lines below show **section-relative** decimal offsets and literal
hexadecimal bytes (16 bytes per line except the final line):

```text
000: 45 56 52 30 f1 00 00 00 01 00 00 00 20 d3 5f 55
016: cc d2 1e 8c 24 91 05 0a c1 4c 43 28 68 51 7f f2
032: 8c a5 d1 c0 c9 ba 7f 59 27 11 eb 52 22 22 22 22
048: 22 22 22 22 22 22 22 22 22 22 22 22 22 22 22 22
064: 22 22 22 22 22 22 22 22 22 22 22 22 01 00 01 00
080: 70 01 00 69 01 00 00 00 01 00 00 00 01 00 00 00
096: 00 00 00 00 04 00 74 79 70 65 01 00 00 00 02 00
112: 00 00 03 00 00 00 00 00 00 00 00 00 00 00 0d 00
128: 50 4f 53 2d 57 49 54 4e 45 53 53 2d 31 00 00 00
144: 00 00 00 00 00 01 00 00 00 00 00 00 00 00 00 00
160: 00 00 00 00 00 00 00 00 00 00 00 00 00 01 00 00
176: 00 1e 26 d7 e9 12 d6 48 6d 3f bf 7c ca 27 73 20
192: f8 63 6b b8 3d 32 3c 2e a8 c2 16 4c 12 4b 3c fc
208: 09 00 00 00 00 02 00 00 00 c9 72 70 23 ca af 7d
224: bf c9 fd d1 df 2b f5 58 fc ac 39 a7 6b 8c d8 9b
240: 1c 72 92 29 4f 9a c7 5f e2
```

This is an EVR0 **section**, not a complete SemCode artifact: the surrounding
`SEMCOD26` header, mandatory `ADT0`, function envelope and trusted
semantic authority remain separately versioned/qualified. An executable
semantic-positive golden test cannot be asserted until issuer and evaluator
contracts are approved.

Proposed malformed mutations against the section: `payload_len=f2 00 00 00`
with no extra byte -> decoder `EvidenceTruncated`; proof
`proposition_index=1` -> verifier `EvidenceIdentityMismatch`;
`parent_count=1` with no parent-table bytes -> decoder
`EvidenceTruncated`; parent target=1 for proof ID=0 -> verifier
`EvidenceParentOutOfRange`; structurally canonical bytes without a
trusted issuer -> admission `EvidenceIssuerUnavailable`. These expectations
remain proposals until an actual decoder/verifier and executable tests exist.

## 9. Runtime boundary

**B2-0-R2 owner architectural direction (not implementation):** after an
independent source/typecheck, reproducible canonical compilation and exact
artifact byte match, successful `sm-verify` admission and EVR0/issuer
validation, a separate trusted `sm-admission` gateway may mint a private,
session-bound `VerifiedEvidence` wrapping the real `VerifiedSemCode`.
`sm-verify` alone does not establish source-level proposition truth.
The gateway token has no public constructor and cannot be reconstructed from
its digest, serialized bytes or an untrusted issuer label.

The first SemCode-backed proposition is limited to single-file, no-import
RustLike whole-program typechecking with an independently established T
observation. F from typecheck failure remains a source-level diagnostic,
not an executable SemCode-backed `VerifiedEvidence` token; S cannot be
constructed from that deterministic proposition at one fixed context.
This narrows the B2 admission goal without changing the general quad logic
model for other future proposition kinds.

**There is no B2 VM consumer, runtime effect or T-only privileged operation.**
The EVR0 `authorized_operation` byte is never itself permission. Future
T-only consumers require a separate owner-approved runtime contract;
S/N/F cannot be coerced to T and legacy SemCode receives no evidence
authority. Section 9 remains a candidate pending D6 ABI freeze, issuer
integration and actual executable qualification.

## 10. Legacy and downgrade rules

- Existing revisions retain their exact current decoding and execution behavior.
- A legacy artifact has no VerifiedEvidence authority, even if bytes resemble
  EVR0.
- A proposed EVR0 artifact missing EVR0, with a modified header, stripped
  proof, altered source identity or unknown revision is rejected.
- An old verifier rejects the proposed header as unsupported.
- A new verifier accepts legacy artifacts only on the legacy path and never
  upgrades them to evidence-bearing artifacts.
- Re-encoding or relabeling an artifact cannot create proof authority.

The B2 contract must enumerate proof-required operations. This candidate
requires EVR0 plus an admitted T-only proof for OP-T-ONLY-1. Legacy execution
and operations not marked proof-required remain valid on their existing path.
A valid S state is not malformed, but it is rejected as EvidenceOperationDenied
when supplied to a T-only operation. Stripping EVR0 removes the required
admission input and cannot authorize that operation.

## 11. Stable rejection classes

The proposed implementation exposes deterministic categories:

`UnsupportedEvidenceRevision`, `MissingEvidenceSection`,
`DuplicateEvidenceSection`, `EvidenceTruncated`, `EvidenceLengthOverflow`,
`EvidenceNonCanonicalOrder`, `EvidenceUnknownRule`, `EvidenceIdentityMismatch`,
`EvidenceWitnessMismatch`, `EvidenceDuplicateWitness`,
`EvidenceParentOutOfRange`, `EvidenceParentCycle`, `EvidenceContextMismatch`,
`EvidenceIssuerUnavailable`, `EvidenceIssuerMismatch`,
`EvidenceOperationDenied`, `EvidenceStateConflict`, `EvidenceDowngrade`, and
`EvidenceHandleForgery`.

Decoder errors are structural. Verifier errors are semantic/admission failures.
Issuer errors are trusted-context failures. VM errors are invalid-token or
session failures. A malformed or unauthorised proof never reaches the VM.

## 12. Golden-vector design (proposed, not executed)

The B2 test set must include:

1. One canonical T proof with one positive witness.
2. N, F and S derivations from a generic abstract proposition in a pure
   rule-checker fixture; do not represent F or S as SemCode-backed outcomes
   of the first single-file whole-program typecheck kind.
3. Duplicate proof/witness IDs.
4. Cross-proposition parent and JOIN attempts.
5. Wrong source/file digest and same-offset different-file witness.
6. Wrong epoch, namespace, operation and compiler identity.
7. Forward and cyclic parent references.
8. Truncation at every length/count boundary.
9. EVR0 stripping, header relabeling and legacy re-interpretation.
10. Correct bytes presented without a trusted issuer context.

These are test designs, not evidence of executed qualification.

## B2-1 implementation-note appendix (non-authoritative until qualification)

The 249-byte section above can be embedded in a **structurally decodable**
rev27 SemCode artifact only after the mandatory canonical `ADT0` table:
`Option(None/0, Some/1)` and `Result(Ok/1, Err/1)`. An empty
`ADT0` section is rejected by the existing strict decoder. The B2-1 golden
vector therefore has layout `SEMCOD26[8] | ADT0[55] | EVR0[249]`,
**312 bytes**, with no function envelopes (structural inspection only).
SHA-256 of this complete candidate vector:
`064c8e8482af430f88b74ed61b3930434cacf2bb7742f64429d017fe9279e360`.

The reproducible generator and fixed binary fixtures live in
`tests/vectors/q3_10_evr0_generate.py`,
`tests/vectors/q3_10_evr0_section.bin` and
`tests/vectors/q3_10_evr0_structural_only.smc`.
Only the raw section has the original 249-byte digest specified above.
These fixtures do not validate the toy compiler contract, proposition
origin, source semantics or issuer. Neither verifier nor VM may consume
this revision as an execution authority. The independent ABI freeze,
evaluator and issuer remain **not qualified** in B2-1.
