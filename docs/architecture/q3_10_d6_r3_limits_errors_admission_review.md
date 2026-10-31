# Q3.10-D6-R3 — Review of Limits, Public Errors and Fail-Closed Admission

Status: **R3 DESIGN REVIEW / CANDIDATE RECOMMENDATIONS — NOT R3 QUALIFICATION**
Repository: `skulmakov-oss/Semantic`
Reviewed base: `69cbdb7edb8a5f7df9053cb1a6140f321e5040e9`
Owner-approved direction: `q3_10_d6_owner_direction_20261031.md`
Inputs: current `sm-format`, `sm-verify`, `sm-vm`, `smc-cli`,
`docs/spec/evr0_proof_section_v0.md`,
`docs/architecture/q3_10_b2_1_d6_abi_freeze_packet.md`.

## 1. D6f — resource envelope and arithmetic

Candidate physical limit: `MAX_EVR0_PAYLOAD = 4,194,304` bytes.
Section overhead = 8 bytes; `Bytes16` uses `u16le` length plus bytes
and each textual field has a **candidate** 4,096-byte limit.

Candidate table count maxima:
```text
propositions <= 65536
proofs       <= 65536
witnesses    <= 262144
parents      <= 262144
depth        <= 256
```

These are independent hard upper bounds, **not guarantees that all
maximal tables fit together**. Each witness is 84 bytes; 262,144 witnesses
alone require 22,020,096 bytes, more than the entire 4 MiB payload.
The actual effective count is limited by remaining payload bytes.
The decoder must:
1. Read and validate the 4 MiB section bound without integer wrap.
2. Validate each count against its per-table maximum before any allocation.
3. Check the minimum required bytes for each counted table against the
   **remaining** payload using `checked_mul`, `checked_add` and
   `checked_sub`; reject before `Vec::with_capacity` when impossible.
4. Parse incrementally, cap aggregate allocated bytes and total work,
   and never allocate from an attacker-provided count alone.
5. Validate exact section consumption and deterministic field ordering;
   fail closed on invalid lengths, UTF-8, duplicate IDs, bad parent ranges,
   nonzero reserved bytes, and trailing bytes.

Proof graph is ordered strictly by earlier parent IDs; cycle prevention
uses that ordering. A proposed depth limit 256 can be verified in one
forward pass with bounded per-node depth storage and checked arithmetic,
not recursion proportional to adversarial input.

**R3 blocker before final D6f freeze:** the candidate does not yet
specify a strict aggregate allocation/work budget under malicious
UTF-8/table inputs or stable error precedence when multiple constraints
fail. Approve explicit units and thresholds before implementation.

## 2. D6g — public enum and diagnostics compatibility

Baseline `sm-format::semcode_decode::DecodeError` is a **public**
enumeration, and `sm-verify::VerificationCode` is also public.
Existing `sm-vm::semcode_vm::decode_and_map_errors` performs an
exhaustive error match; `sm-verify` does the same. B2-1 affects a public
Rust API surface even if no caller gets to mint Evidence.

Recommended narrow contract:
- New structural error(s) must carry deterministic absolute byte offset
  and static, non-producer-controlled explanation, with bounded messages.
- Structural syntax failure, resource exhaustion and unsupported revision
  remain distinct classes; do not map malformed EVR0 to misleading
  `InvalidAdtDescriptorSection`.
- Diagnostic categories for untrusted source/issuer/proof **must not**
  automatically become stable public ABI just because listed in a
  proposed spec. No public `EvidenceHandleForgery` claim is warranted
  in a decoder-only stage.
- Update all exhaustive matches and public API contract tests in the
  **same bounded B2-1 PR**, or do not land the public enum change.
- No bytecode-specific stable success/result codes until qualification.

**R3 blocker before final D6g freeze:** exact public variants, field
shape, mapping from decode to `RejectReport` and raw-VM error are not
yet approved and require a verified compilation/API-contract matrix.

## 3. D6i — rev27 admission and downgrade-safe operation

A new `SEMCOD26` header should be recognized by the structural decoder
only when its exact mandatory `ADT0 | EVR0` layout is valid. This is
*structural acceptance*, not trusted admission.

Initial B2-1 safety posture: **rev27 is not executable or trusted**.
- Canonical `sm-verify::verify_semcode_token` MUST reject rev27 at an
  explicit deterministic evidence-issuer-unavailable or evidence-not-
  admitted gate. It must NOT mint ordinary `VerifiedSemCode` for rev27
  merely because legacy instruction checks succeeded.
- Raw/diagnostic `sm-vm` APIs MUST not silently begin executing rev27
  after decoder support is added. Reject rev27 until a separate and
  explicitly approved consumer qualification; disassembly/structural
  inspection may remain non-executing when safe.
- `smc-cli` artifact identity/inspection may show rev27 metadata but
  must mark it `structural only / evidence not admitted`.
- V0..V25 preserve exact existing decoding and execution behavior. Any
  EVR0-like legacy bytes remain legacy data under the old revision rules.
- Revision downgrade/header swapping/re-encoding **cannot** promote
  evidence; reject missing proof when evidence required.

**R3 blocker before final D6i implementation:** approved placement of
fail-closed guards in both canonical and raw/diagnostic routes, and tests
demonstrating that decoding rev27 is not trusted execution.

## 4. B2-1 scope delta discovered in actual code

The `DecodedSemCode` public struct is destructured by:
- `crates/sm-verify/src/lib.rs`;
- `crates/smc-cli/src/artifact_identity.rs`.

The `DecodeError` public enum is exhaustively matched by:
- `crates/sm-verify/src/lib.rs`;
- `crates/sm-vm/src/semcode_vm.rs`.

New header is selected from `crates/sm-format/src/local_format.rs` and
legacy producer version choice remains in `sm-ir`.

Minimum B2-1 integration scope: `sm-format` decoder, minimal
`sm-verify` fail-closed gate, `sm-vm` explicit rev27 rejection and
error mapping, `smc-cli` exact public-struct adaptation, and
`tests/public_api_contracts.rs` plus relevant format/legacy fixtures.
No new VM opcode, interpreter token consumption, compiler emitter,
Cargo changes or new gateway in B2-1.

## 5. Test-first R3 closeout required before normative freeze

| Gate | Required evidence |
|---|---|
| Layout | Independent byte reconstruction of current 249-byte EVR0 section, and complete header+ADT0+EVR0+function vector |
| Decoder | Bounded/truncated/overflow/malformed counts, nonzero reserve, canonical table order and duplicate/cycle hostile cases |
| Public Rust API | Exhaustive match changes compile in `sm-verify`, `sm-vm`, `smc-cli`; `cargo test --test public_api_contracts --quiet` |
| Legacy | Old V0..V25 vectors remain identical; unknown/relabelled header rejects |
| Trust | Rev27 decoder-positive does not mint `VerifiedSemCode` / `VerifiedEvidence` or run in VM |
| R3 tools | FullPreflight admission guard, 7hell CI, public API gate, Harness, fresh adversarial review |
| Reproducibility | Exact commit/HEAD, tool versions, vector digests, exit codes and rejected adversarial inputs |

No implementation test was performed during this **documentation-only**
review. Green Harness on a staged documentation file must not be
misrepresented as R3 qualification of public API and new SemCode parser.

## 6. Review verdict and narrow next action

- D6a–d/j: owner selected the header/layout/no-opcode/compatibility
  direction. D6h reiterates prior trusted-issuer boundary. D6e four-table
  payload remains proposed; no production code issued.
- D6f: recommended fail-safe physical/count/depth limits, **work and
  allocation budget still require final contract**.
- D6g: reject taxonomy must be frozen before a public enum changes.
- D6i: default recommended **fail-closed rev27 in verifier and raw VM**
  until a separate issuer/consumer GO.
- D7: B2-1 production implementation authorization remains separate.
- D3: protected Cargo/new-crate authorization remains separate.

**Status: Q3.10-D6-R3 PRE-FREEZE REVIEW COMPLETE —
D6e/f/g/i FINAL ABI DECISIONS PENDING; B2-1 NOT STARTED.**
