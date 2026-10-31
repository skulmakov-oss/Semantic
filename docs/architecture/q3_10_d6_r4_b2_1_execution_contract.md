# Q3.10 D6-R4 / B2-1 — bounded implementation contract

Status: OWNER GO for one bounded B2-1 implementation PR; **normative wire-schema freeze pending independent validation**.

## Frozen owner direction
- `SEMCOD26`, epoch 0, revision 27; eight-byte header.
- Mandatory section order `header | ADT0 | EVR0 | functions`.
- EVR0 section framing: ASCII `EVR0`, `u32le payload_len`, exact payload.
- No new opcode, capability, issuer authority, VM effect, or legacy revision reinterpretation.
- V0..V25 must retain existing decoding and execution behavior.

## Safety gates to implement before accepting revision 27
- Structural parsing must be bounded by payload <= 4,194,304 bytes, checked cursor arithmetic, and aggregate allocation/work budgets.
- All tables, canonical ordering, indices, witness and parent ranges, reserved bytes, source spans and proof graph must reject malformed inputs deterministically.
- Public `DecodeError` changes require exhaustive handling in `sm-verify` and `sm-vm`, plus public API tests.
- `sm-verify` must not mint `VerifiedSemCode` for rev27 before a separately qualified issuer.
- Raw `sm-vm` execution must reject rev27 even if the structural decoder succeeds.
- Legacy fixtures and malformed revision-27 fixtures must be included.

## Implementation stages within one PR
1. Independently reproduce and validate the 249-byte EVR0 sample; resolve candidate inconsistencies.
2. Freeze precise wire schema, aggregate budgets, deterministic error precedence and failure mapping.
3. Add revision-gated structural parser and typed, untrusted EVR0 result in `sm-format`.
4. Add minimal fail-closed verifier/VM/CLI adaptations without changing execution semantics.
5. Add golden/negative vectors and public API tests; run workspace, no_std, Harness, R3 qualification and CI.
6. Publish one Draft PR, retain draft until complete qualification and separate OWNER MERGE GO.

## Non-goals
No `sm-admission` crate, Cargo or Harness edits, issuer minting, runtime privilege, SHF0/bootstrap mutation, release/tag, or merge. No claiming a green check without its observed result.

## Current checkpoint
The new branch is `feat/q3-10-b2-1-evr0-decoder` from merged main
`3dd5ddc982d38aae32b5afa9d0c0ce388827bb56`.
This document is the explicit R4 scope contract; production qualification remains open.

## Implemented candidate on this branch (qualification pending)
- `sm-format` exposes a public structural `DecodedEvr0` with four typed tables and a distinct malformed-section error.
- The parser enforces four fixed count maxima, 4 MiB input payload, 16 MiB logical decoded-allocation budget and 8 MiB deterministic work budget. The work metric charges one unit per payload byte and each declared record; the parent-graph pass is already charged once per parent record.
- All four table counts are checked against minimum remaining bytes before any vector growth; section integers and ranges are checked. Table ordering and disjoint contiguous ownership of proof-witness/parent ranges are enforced.
- Graph checks validate earlier-parent-only references, strict increasing parent IDs, matching proposition and authority index, and nonrecursive depth <= 256.
- B2-1 knows four proposed rule identifiers and their structural shapes but **does not authenticate any rule**, source digest, origin, compiler identity, epoch, or authority index.
- Stable verifier diagnostics in this candidate are `V0033` (malformed evidence) and `V0034` (issuer unavailable). They are append-only after `V0032`; never repurpose existing codes.
- A valid rev27 artifact must fail `verify_semcode_token` with `V0034`; raw `sm-vm` byte entry points reject rev27. No binary producer or issuer is added.
- The golden reconstructs a candidate *structural* vector whose source semantic proof is deliberately not authenticated. The canonical ADT0 contains Option/Result; an empty ADT0 is forbidden by existing Semantic baseline.
- Final normative ABI freeze, source-observation authority, exact Unicode canonicalization policy, independently audited work accounting, and qualified issuer remain separate review gates. This record is not a claim of R3 completion.

## Validation record
- Legacy fingerprint: `3dd5ddc982d38aae32b5afa9d0c0ce388827bb56`.
- Structural golden section: 249 bytes, SHA-256 `cd91d079b4c5846f8bd4e9edf13fc2136d70fe935d808f6d17fdd1f318bd1870`.
- Full fixture: 312 bytes (8-byte header, 55-byte canonical built-in ADT0 and 249-byte EVR0); SHA-256 `064c8e8482af430f88b74ed61b3930434cacf2bb7742f64429d017fe9279e360`.
- Local focused suites: EVR0 6/6, public API 89/89, legacy guards 10/10, sm-format 85/85, sm-verify 223/223 = 413 passed, 0 failed (before final branch SHA).
- Full workspace, no_std, Harness, hosted CI and R3 independent adversarial qualification remain separately reportable; passing a focused suite does not imply final ABI/issuer approval.

## External review follow-up / CI dependency (2026-10-31)

- Confirmed by source and regression: `decode_semcode_envelope` delegates to canonical `decode_semcode`. New tests cover a rev27 artifact with actual function envelopes after `EVR0`; no independent or degraded envelope parser is permitted.
- Count overflow reports the absolute offset of its offending `u32le`; proof and witness record rejection reports the record beginning; canonical-parent failures report the specific parent-reference location. All offsets are byte positions in the whole artifact.
- Tables and graph-depth storage use bounded, fallible exact-capacity reservations; strings use a bounded fallible reservation. Resource ceilings do **not** guarantee physical memory availability or prevent host-level allocation failure.
- Independent golden generator checks both section and full artifact SHA-256 digests, rather than only the raw section.
- Regression tests exercise canonical and inverted/duplicate two-parent `JOIN-SAME-P-1`, duplicate/descending propositions, control characters, exact offsets, all truncated prefixes, and single-byte mutations. Focused EVR0 suite is now 11 tests.
- Original hosted CI on B2-1 HEAD `1a3f80382a5a23b23eace6f16f747bc2f9e3261d` failed only `test-std` via `ssf09_editor_baseline::every_producer_code_is_catalogued_and_explainable`: `V0033` lacked a canonical catalog entry. Other relevant jobs (trusted scope, legacy boundary, public API, 7hell, CodeQL and Clippy) passed on that head.
- Catalog parity **landed in `main` via PR #2067** at `ef5ae75bc338cd83b83f68de0ef2e9d3a53c5988` (two files, four insertions). That PR's original trusted-scope check failed because the frozen `foundation_delivery` base omitted the catalog paths. The owner explicitly authorized a one-time administrative ruleset exception; the exception was recorded in PR #2067, used for rebase merge, then removed. GitHub `protect-main` was verified restored to its original active configuration with zero bypass actors. Do not claim the original red check became green. The B2-1 branch was rebased onto that main SHA, so V0033/V0034 catalog coverage is a genuine baseline dependency, not a bypass within B2-1.
- No issuer, source-proof semantics, execution authority, Cargo/Harness changes or normative ABI freeze is granted by this review response.
