# Q3.10 D6-R5 - Normative Reconciliation

Status: READINESS REVIEW - no production authorization

Repository baseline: 4207f72551190662c14961deb083bdf3bf33d3d2

This document reconciles D6a-D6j with the merged revision-27 structural
implementation. It does not promote revision 27 to semantic admission or
execution authority.

## Matrix

| Item | Contract | Actual implementation/evidence | Gap or risk | Status |
|---|---|---|---|---|
| D6a header | SEMCOD26, epoch 0, revision 27 | sm-format::HEADER_V26 and SEMCODE_EVR0_MIN_REVISION are 27 | Header is structural only; verifier/VM remain fail-closed | VERIFIED structural / BLOCKED admission |
| D6b placement | Header, ADT0, EVR0, function envelopes | decode_semcode derives sections from revision; no tag sniffing | Semantic meaning is not implemented | VERIFIED structural |
| D6c framing | EVR0 tag, u32le length, exact consumption | evr0 decoder checks framing, bounds and trailing data | Public error mapping needs final qualification | VERIFIED structural |
| D6d capabilities | Preserve existing capabilities; no new opcode/runtime effect | Header inherits existing envelope; VM rejects rev27 | No proof-bearing runtime operation exists | VERIFIED |
| D6e schema | Canonical proposition/proof/witness/parent tables | evr0 decoder bounds-checks tables and ordering | Structural claims are not semantic proofs | VERIFIED structural / PROPOSED semantic |
| D6f limits | 4 MiB payload; 4096-byte field; 65536 propositions/proofs; 262144 witnesses/parents; depth 256 | Decoder constants and checked cursor/count validation implement structural limits | Allocation/work budgets are not physical-memory guarantees | VERIFIED structural / OWNER DECISION REQUIRED budget policy |
| D6g diagnostics | Stable categories, deterministic offsets, public mapping | Decoder has structural/resource errors; verifier/VM map rev27 to unsupported admission | V0033/V0034 catalog/API snapshot alignment remains open | PROPOSED / OWNER DECISION REQUIRED |
| D6h issuer | EVR0 metadata never mints trust | No issuer constructor or VerifiedEvidence token exists | B2-2 needs a trusted gateway and authenticated context | VERIFIED boundary / BLOCKED implementation |
| D6i admission | Rev27 fail-closed until gateway qualification | sm-verify and sm-vm reject rev27; no CLI promotion path | Future gateway must not weaken these paths | VERIFIED fail-closed |
| D6j compatibility | V0-V25 retain interpretation; no downgrade | Legacy gates remain separate; rev27 is not an upgrade path | Legacy regression proof remains required after gateway work | VERIFIED |

## D6f limits

These are bounded parser/admission inputs, not resource guarantees:

- EVR0 payload: 4 MiB;
- length-delimited field: 4096 bytes;
- propositions and proofs: 65,536 each;
- witnesses and parents: 262,144 each;
- derivation depth: 256;
- checked arithmetic and bounded deterministic work.

The effective table limit is the smaller of its count maximum and remaining
payload bytes. The decoder must reject before allocation when the minimum table
footprint cannot fit. A successful bound check does not prove host memory
availability.

## D6g diagnostics

Current source takes precedence over stale documents:

- decoder failures retain deterministic structural class and absolute offset;
- verifier maps revision-27 structural artifacts to unsupported admission;
- VM refuses revision 27;
- smc explain and public API snapshots must not claim semantic admission until
  V0033/V0034 mapping is explicitly qualified.

Proposed precedence for B2-2 is malformed framing/length, then unsupported
revision/policy, then context/issuer mismatch, then semantic rule failure, then
operation denial. This remains a proposal until the public diagnostic contract
is approved.

## D6i transition

The safe transition is: retain structural decode; retain verifier/VM rejection;
add an independently qualified gateway; reconstruct the proposition and
expected EVR0; recompile the exact source; require complete byte equality; then
invoke an explicitly approved revision-27 admission policy and issue a token.

There is no temporary verifier switch, public issuer flag, digest-only shortcut
or legacy-to-rev27 promotion.

## Decision register

| Decision | Classification |
|---|---|
| Existing ADT0/EVR0 placement and legacy gates | VERIFIED existing fact |
| Current rev27 fail-closed verifier/VM behavior | VERIFIED existing fact |
| Structural table and limit interpretation | VERIFIED implementation; semantic authority absent |
| Aggregate allocation/work budget semantics | OWNER DECISION REQUIRED |
| Final V0033/V0034 public mapping | OWNER DECISION REQUIRED |
| Trusted issuer context and gateway ownership | OWNER DECISION REQUIRED |
| sm-admission Cargo transition | BLOCKED by current dependency/governance envelope |
| VM consumption of evidence token | OUT OF SCOPE; separate authorization required |
