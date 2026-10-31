# EVR0 B2-1 Structural Contract v1 (revision 27)

Status: **NORMATIVE FOR STRUCTURAL DECODING ONLY**; does NOT admit trusted evidence or execution.
Owner authorization: 2026-10-31 single-pass Q3.10 D6-R4/B2-1 task on `main` after PR #2065.
This document narrows `evr0_proof_section_v0.md`, which remains a proposed *semantic/issuer* contract.

## Format

The only new header is 8 ASCII bytes `SEMCOD26`, epoch 0, revision 27,
`capabilities = HEADER_V25.capabilities`. There is no new opcode or
capability. The mandatory section sequence is precisely:

```text
SEMCOD26[8] | ADT0 (unchanged, mandatory) | EVR0 | function envelopes
EVR0 = "EVR0"[4] | payload_len:u32le | payload[payload_len]
```

Position comes from header revision alone. V0–V25 never sniff EVR0.
Exactly one EVR0 section is allowed; a duplicate EVR0 prefix in the
function-envelope slot is rejected. Payload length must be 1..4194304 bytes.
The section must contain exactly all counted records and no trailing payload.

The payload layout, in field order, is:
```text
schema:u16le (=1), flags:u16le (=0)
source_set:[u8;32], compiler_contract:[u8;32], epoch:u16le
policy_namespace:Bytes16, issuer_namespace:Bytes16
proposition_count:u32le, proof_count:u32le
witness_count:u32le, parent_count:u32le
propositions[proposition_count] | proofs[proof_count] |
witnesses[witness_count] | parents[parent_count]
```

`Bytes16` means `u16le(length)` followed by exactly that many UTF-8 bytes,
length 1..4096. For B2-1, no Unicode normalization policy is inferred from
the bytes beyond strict UTF-8 well-formedness. Bytewise ordering and string
equality are used; casefolding/locale rules are forbidden.

Proposition: `kind:Bytes16`, `ast_owner_id:u32le`,
`semantic_scope_id:u32le`, `type_obligation_id:u32le`. Strict ascending
lexicographic order of the entire tuple, no duplicates.

Proof: `proof_id:u32le`, `proposition_index:u32le`, `rule_id:Bytes16`,
`authority_context:u32le`, `witness_first:u32le`,
`witness_count:u32le`, `parent_first:u32le`, `parent_count:u32le`,
`authorized_operation:u8`, `reserved:[u8;3] (=0)`. Proof IDs are contiguous
0..N-1; proposition indices must be in range; operation 0 or 1 is a
*structural code only*, not a permission. Witness and parent ranges are
contiguous, non-overlapping, owned by proofs in proof order and cover the
complete respective tables.

Witness: `witness_id:u32le`, `proposition_index:u32le`,
`polarity:u8 (0=negative,1=positive)`, `reserved:[u8;3](=0)`,
`file_digest:[u8;32]`, `source_start:u32le`, `source_end:u32le`,
`origin_digest:[u8;32]`. IDs contiguous, proposition index valid,
owned by a matching-proposition proof; start <= end. The serialized digests
and span are **untrusted**; B2-1 cannot check external source-set membership.

Parent table: `target_proof_id:u32le` each. For each containing proof,
parents strictly increase, reference an earlier proof only, and have the
same proposition/authority identity. This defines an acyclic graph;
longest chain depth must not exceed 256 (root depth 1).

## Resource budget and deterministic failure

| Resource | Structural upper bound |
|---|---:|
| Payload bytes | 4,194,304 |
| Bytes16 string bytes | 4,096 |
| Propositions | 65,536 |
| Proofs | 65,536 |
| Witnesses | 262,144 |
| Parents | 262,144 |
| Depth | 256 |

The four count limits are **independent**, constrained also by payload.
Before `Vec::with_capacity`, the decoder must reject counts above limits
or whose *minimum* encoded sizes cannot fit remaining bytes, using checked
arithmetic. A valid payload cannot cause a table to allocate more than
its bounded count; arrays of witnesses are consumed incrementally rather
than retained. Proof and proposition bookkeeping and parent depths use
only bounded allocations. Work is bounded by the payload byte count plus
`O(witness_count * log2(max(1,proof_count)))`, the remaining table scans
being linear. No recursion on untrusted parent DAGs, direct I/O, or
untrusted configured resource multiplier is allowed.

Deterministic error precedence follows wire order: header/ADT0, EVR0 tag
and section length, payload schema/flags, strings/counts, propositions,
proofs/ranges, witnesses, parents/depth, final payload consumption,
then function envelopes. When multiple defects exist, the first reached
canonical check determines the rejection.

**Public B2-1 error API**: `DecodeError::InvalidEvidenceSection {offset,msg}`
with absolute byte offset and a static message; `ResourceLimit` remains
available for existing bytecode constraints. The verifier translates
malformed EVR0 into the existing `VerificationCode::UnsupportedVersion`
until the separate stable issuer/semantic taxonomy is approved, and always
rejects structurally valid revision 27 as an untrusted proof-bearing
revision. Raw VM execution likewise rejects revision 27; CLI may inspect
structural metadata only. This is a *rejection contract*, not
`VerifiedSemCode` issuance or `VerifiedEvidence` issuance.

## Compatibility and out-of-scope

- V0..V25 parsing, code generation and verifier admission remain unchanged.
- Existing compiler continues emitting its current revision; no automatic
  upgrade to SEMCOD26.
- Structural acceptance does not verify source digests, issuer context,
  compiler provenance, rule truth, positive witness polarity or
  authorized runtime operation.
- B2-2/B2-3, `sm-admission` crate, new Cargo dependencies, VM consumption
  of evidence and privileged operations require **separate owner GO**.
- The 249-byte fixture in `evr0_proof_section_v0.md` is structural only;
  its contract digest is synthetic and never a semantic-positive golden.
