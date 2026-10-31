# Q3.10-B1 Contract Review

Status: Q3.10-B1-R2 documentation correction candidate; no ABI implementation, no normative approval.

## Baseline consistency

The current decoder derives `ADT0` presence from header revision and then reads
function envelopes to end-of-artifact. Existing `DBG0`, `OWN0` and `SIG0`
parsing is revision-aware and must remain unchanged. The EVR0 candidate follows
the same revision-derived approach and does not content-sniff legacy artifacts.

## Security conclusion

Model C (hybrid) is the only candidate that covers both reproducible local
derivations and externally established authority. A serialized proof is a
claim until independently checked and bound to an authenticated issuer context.
The resulting handle is a verifier-issued, session-scoped capability and is not
the serialized proof itself.

## Remaining owner decisions

1. Approve allocation of a new SemCode header family/revision for EVR0.
2. Approve the exact trusted issuer-context interface and its authentication
   mechanism.
3. Approve the EVR0 rule registry and whether proof rules are source/semantic
   contract owned or verifier owned.
4. Decide whether VM consumption is required in B2 or whether B2 ends at
   verifier-issued token creation.
5. Approve the proposed limits and rejection-code surface.

## R1 closure findings

- proposition_index is now unambiguous: it indexes one explicit canonical
  proposition table; repeated proofs share that index.
- parent entries are fixed-width u32 proof IDs selected by contiguous ranges;
  only earlier proof IDs are legal.
- authority_context is not an authority-bearing public integer. It is resolved
  only through the authenticated admission context.
- file, manifest and origin identities use domain-separated SHA-256 and bind
  the existing frozen source-set path/import policy.
- the initial rule registry is verifier-owned and requires independent premise
  checks; well-formed bytes and matching digests alone never prove a rule.
- S remains a valid derived conflict state, but T-only operations reject S.
- one positive vector and five malformed-vector expectations are specified
  below the implementation boundary; none has been executed.

## R2 independent audit and correction

- The prior section example used the unknown `POS1` alias. It now uses exact
  `POS-WITNESS-1` (13 UTF-8 bytes), with derivation-only operation 0;
  the proof is 47 bytes, payload 241 bytes, EVR0 section 249 bytes.
- The source fixture is fully stated (`probe.sm`, UTF-8 LF source, [0,2)
  origin) and the proposed section is listed as **249 literal bytes** with
  domain-separated file/manifest/origin hashes. The vector proves only
  structural well-formedness; without a trusted issuer/evaluator, admission
  must reject it. No complete SemCode or authenticated semantic-positive
  vector is claimed.
- `source_set` is the EVR0-MANIFEST digest. Frozen
  `shf0-source-set-v1` is a separate domain with different framing;
  the trusted admission service must recompute both from identical source
  files and validate the typed mapping. No bootstrap format repinning.
- Added explicit limit for `proposition_count` (65,536); four counts now
  each have a bound, independently of total payload bound.
- The minimum proposed rule registry now specifies exact names, proof shapes,
  zero/one authorized-operation semantics, parent requirements, and rejects
  unverifiable witness polarities. **The actual trusted semantic evaluator
  algorithm/interface and authenticated issuer context remain normative
  owner decisions.** A proof hash and matching source span cannot authorize
  type semantics. B2 remains blocked on those decisions.
- The prior R1 register is retained as history; the R2 closure is the current
  evidence. No source/compiler/VM/format admission claim follows from a
  documentation-only patch.

## Validation performed

- Exact baseline worktree inspected: `69cbdb7edb8a5f7df9053cb1a6140f321e5040e9`.
- Current SemCode framing and revision-derived ADT0 behavior inspected.
- No encoder, decoder, verifier, VM, opcode, capability, Cargo or bootstrap
  files modified.
- Golden vectors are proposals only; no B2 implementation tests were run.
- R2 independent machine reconstruction on the owner PC: the documented EVR0
  249-byte section matched byte-for-byte a fresh construction from the specified
  fields, with payload length 241; source, file and origin SHA-256 digests were
  recomputed independently. Section SHA-256:
  `cd91d079b4c5846f8bd4e9edf13fc2136d70fe935d808f6d17fdd1f318bd1870`.
- No encoder, decoder, verifier or runtime qualification is claimed.

## Classification

`Q3.10-B1-R2 DOCUMENTATION CORRECTED — OWNER ABI REVIEW PENDING`

This is not `Q3.10-B QUALIFIED` and does not authorize implementation or merge.
