# Q3.10-D6 — Owner Acceptance of EVR0 Header Direction

Status: **OWNER TECHNICAL GO FOR D6a–d/j; D6h REAFFIRMED; D6e/f/g/i OPEN — NOT ABI FREEZE**
Repository: `skulmakov-oss/Semantic`
Checked worktree HEAD: `69cbdb7edb8a5f7df9053cb1a6140f321e5040e9`
Branch: `feat/q3-10-opaque-evidence-prototype`
Decision provenance: affirmative repository-owner message ("Да"), 2026-10-31,
in direct response to the recommendation to adopt `SEMCOD26` / revision 27,
mandatory `EVR0` after `ADT0`, without new opcodes or capabilities,
**subject to independent R3 review of final limits, error codes and admission**.

## 1. Accepted technical direction

- **D6a**: Future proof-bearing header is `SEMCOD26`, 8-byte ASCII magic,
  epoch 0, revision 27. V25 remains revision 26.
- **D6b**: The new layout is `header[8] | ADT0 | EVR0 | function envelopes`.
  `ADT0` remains required, then exactly one required, version-gated
  `EVR0` section. No tag sniffing in older revisions.
- **D6c**: `EVR0` framing is the four literal bytes `45 56 52 30`,
  followed by `payload_len:u32le` and exactly that many payload bytes.
  The inner schema layout and canonical table rules remain subject to D6f/g
  review and normative spec synchronization.
- **D6d**: No new opcode, no new capability, no reinterpretation of a
  capability bit. The V26 candidate inherits the complete V25 capability
  envelope unchanged.
- **D6e**: Schema 1 with zero flags and the proposition, proof,
  witness and parent tables remains a **candidate**, not expressly
  approved by the owner's brief reply; complete wire-schema freeze requires
  a separate exact contract review.
- **D6h**: Issuer names, `authority_context` and digests in serial EVR0
  remain untrusted claims and cannot themselves grant evidence authority.
- **D6j**: V0..V25 interpretation and admission remain unchanged. No
  downgrade/relabeling or legacy evidence promotion.

The owner approved the **direction**, not a merged binary specification.
No `MAGIC26`, decoder, issuer, `VerifiedEvidence`, interpreter consumer,
new manifest/Cargo file, or public error enum has been implemented.

## 2. Deliberately unresolved normative decisions

- **D6e/f**: Final four-table wire-schema and resource-limit matrix,
  allocation bounds, error precedence
  and worst-case verifier work. See R3 review document.
- **D6g**: Stable public versus private rejection categories, exact Rust API
  compatibility and enum exhaustiveness. See R3 review.
- **D6i**: Gate on newly parsed revision-27 artifacts across canonical
  verifier and raw VM/diagnostic paths; **fail closed** is the safety
  requirement, implementation details still under review.
- **D7**: A new, separately bounded B2-1 production implementation GO.
  The affirmative D6 response alone is not a D7 authorization.
- **D3**: A separate protected governance/dependency transition is still
  needed for the later proposed `sm-admission` crate.

`docs/spec/evr0_proof_section_v0.md` therefore stays labeled **candidate**,
not frozen normative truth. This record is local owner-decision evidence
for architecture and must not be paraphrased as a release/ABI success.

## 3. Containment

No source, ABI, runtime, VM, C0, bootstrap, Harness, Cargo, release,
commit, push, PR or merge modifications are authorized by this record.

**Status: D6 HEADER/LAYOUT DIRECTION ACCEPTED — D6e/f/g/i R3 CLOSEOUT PENDING.**
