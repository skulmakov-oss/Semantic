# SSF-08 / #1718 — Ownership Path-Family Contract Decision

Status: **CONTRACT DECISION ONLY. NO PRODUCTION BEHAVIOR CHANGE.**
Audit/decision baseline SHA: `ef2d8524ddc597f7064edf052dad70fd03575fdf` (`main`, confirmed
current at time of writing; `origin/main` unmoved).

This document answers one question only: **does the Semantic Stable Foundation
(Position A) contour authorize `SequenceIndexStatic` and `AdtPayload` ownership
path components, and if so, under what capability/version authority?** It does
not implement a wire format, a capability bit, a header revision, or any
decoder/verifier/VM change. #1718 remains OPEN after this document. Per the
repository's own "contract decided != implementation repaired" discipline,
closing #1718 requires a follow-up implementation checkpoint (see §10).

## 1. Historical mechanism (2026-08 finding)

`#1718` / FA-04-012 recorded that `sm-ir` emits `AdtPayload`/`SequenceIndexStatic`
ownership path components end-to-end (emit → decode → verify → VM enforcement)
while `docs/spec/semcode.md` froze `SEMCOD11` as a **tuple-only** ownership
transport and no capability bit distinguished these two families from
tuple/record. Three independent module owners (`sm-format`, `sm-verify`,
`sm-runtime-core`) filed matching "Phase A evidence only, no repair" comments
on 2026-08-15/16.

## 2. Current mechanism (post-rev21, re-verified fresh on this baseline)

Nothing about Lane 2 (#1709/#1724/#1725/#1726/#1891, closed) or the #1888
reconciliation (closed) touches this axis — that repair concerned Borrow/Write
**execution-site timing** (which instruction/PC activates or discharges an
event), which is orthogonal to **component-kind capability gating** (which
path shapes a header revision may legally carry). Direct re-inspection of
current `main` confirms the original mechanism is structurally unchanged:

- **Capability vocabulary** (`crates/sm-format/src/local_format.rs:35-36`):
  `CAP_OWNERSHIP_PATHS = 1 << 12` (first carried by `HEADER_V11`),
  `CAP_OWNERSHIP_FIELD_PATHS = 1 << 13` (first carried by `HEADER_V12`, and
  every header since — `HEADER_V20`/rev21 inherits `HEADER_V19.capabilities`
  verbatim, introducing **no new bit**). No third or fourth ownership
  capability bit exists anywhere in the codebase.
- **Header promotion** (`crates/sm-ir/src/legacy_lowering.rs:2565-2579`):
  `has_v11_ownership_events` promotes to (at least) `HEADER_V11` for **any**
  non-empty `ownership_events` list — tuple, field, sequence, and ADT are all
  lumped into this one generic floor. `has_v12_record_field_ownership_events`
  promotes further, specifically, only when a `PathComponent::Field(_)` is
  present. **There is no equivalent `has_v1x_sequence_ownership_events` or
  `has_v1x_adt_payload_ownership_events` function.** A program using only
  `Sequence`/`AdtPayload` ownership paths and no `Field` component is promoted
  no further than the generic `HEADER_V11` floor by this mechanism alone —
  i.e. it can still be emitted, today, on this exact baseline, under a header
  whose own normative text (`docs/spec/semcode.md`, `SEMCOD11` section, freshly
  re-read this checkpoint) says the format "does not claim record, ADT
  payload, schema, or release/lifetime transport beyond the current
  frame-local tuple slice." This answers item 4 of the governing brief
  directly: the defect has **not** disappeared post-rev21, it has moved
  forward unchanged, because rev21 inherits the same two bits.
- **Decode** (`crates/sm-format/src/semcode_decode.rs:488-536`): the
  component-kind `match` admits `OWNERSHIP_PATH_COMPONENT_ADT_PAYLOAD` and
  `OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX` with exactly the same structural
  treatment as `TUPLE_INDEX`/`FIELD_SYMBOL` — no header/capability parameter
  reaches this `match` at all; it is purely a function of which OWN0 bytes are
  present, on any header revision.
- **Verify** (`crates/sm-verify/src/lib.rs:1235-1253`, `1414-1423`): the
  `used_caps` accumulator sets `CAP_OWNERSHIP_PATHS` for `has_ownership_section`
  and `CAP_OWNERSHIP_FIELD_PATHS` for `has_record_field_ownership` (defined,
  literally, as "some component is `FieldSymbol`"). The `AdtPayload` and
  `SequenceIndexStatic` match arms are empty — no capability is set for
  either. Current, live source comments read: "Variant is a global SymbolId;
  it cannot be bounds-checked against the local string table. Structural
  acceptance only." and "Static sequence index ownership is structurally
  accepted." Both are accepted under the bare `CAP_OWNERSHIP_PATHS` bit that
  every header since `HEADER_V11` carries.
- **VM** (`crates/sm-vm/src/semcode_vm.rs:1205-1215`): decoded `AdtPayload`/
  `SequenceIndexStatic` components convert unconditionally into runtime
  `sm_runtime_core::PathComponent` and participate in ordinary
  `access_paths_overlap` enforcement — no capability/header gate at this
  layer either.

**Conclusion: the #1718 mechanism is identical today to 2026-08.** The gap is
not "old artifacts still say SEMCOD11" — it is that **no header/capability
authority, at any revision through `HEADER_V20`, explicitly authorizes these
two component kinds**, while the producer, decoder, verifier, and VM all
already treat them as first-class citizens.

## 3. Four-family evidence matrix

Families are scored independently, per the brief's explicit instruction not to
group Sequence and ADT merely because #1718 filed them together.

| Dimension | `TupleIndex` | `Field` | `SequenceIndexStatic` | `AdtPayload` |
|---|---|---|---|---|
| Source producer | `sm-ir` legacy lowering | `sm-ir` legacy lowering | `sm-ir` legacy lowering | `sm-ir` legacy lowering |
| IR representation | `PathComponent::TupleIndex(u16)` | `PathComponent::Field(SymbolId)` | `PathComponent::SequenceIndexStatic(u32)` | `PathComponent::AdtPayload{variant: SymbolId, index: u16}` |
| Current compiler reachability | Yes, Borrow+Write, real source | Yes, Borrow+Write, real source | **Yes, Borrow+Write, real source** (`sequence_ownership_golden.rs`, both `borrowed_paths` and `write_paths` populated by real compiled indexing) | **Borrow only, real source** (`vm_runs_adt_payload_ownership_positive_e2e_path`, via `match`/`ref`); **Write side has no real-source path** — the language has no mutable ADT-payload reassignment syntax, so `PathComponent::AdtPayload` in `write_paths` is only ever exercised by hand-patched test bytes, never by the compiler |
| Header revision emitted today | `HEADER_V11`+ (generic ownership floor) | `HEADER_V12`+ (dedicated promotion predicate) | `HEADER_V11`+ (generic floor only — **no dedicated predicate**) | `HEADER_V11`+ (generic floor only — **no dedicated predicate**) |
| Capability bit(s) | `CAP_OWNERSHIP_PATHS` | `CAP_OWNERSHIP_PATHS` + `CAP_OWNERSHIP_FIELD_PATHS` | `CAP_OWNERSHIP_PATHS` only (no dedicated bit) | `CAP_OWNERSHIP_PATHS` only (no dedicated bit) |
| Wire component kind | `OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX` | `OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL` | `OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX` | `OWNERSHIP_PATH_COMPONENT_ADT_PAYLOAD` |
| Decoder admission | Structural, unconditional | Structural, unconditional | Structural, unconditional, header-agnostic | Structural, unconditional, header-agnostic; variant `SymbolId` never bounds-checked ("structural acceptance only") |
| Verifier admission | Contributes to `CAP_OWNERSHIP_PATHS` | Contributes to both bits | Contributes to `CAP_OWNERSHIP_PATHS` only — **no dedicated capability requirement** | Contributes to `CAP_OWNERSHIP_PATHS` only — **no dedicated capability requirement**; no variant-identity proof |
| VM representation | `PathComponent::TupleIndex` | `PathComponent::Field` | `PathComponent::SequenceIndexStatic` | `PathComponent::AdtPayload` |
| VM overlap enforcement | `access_paths_overlap`, equality-based | Same | Same, freshly re-verified (§4) | Same, freshly re-verified (§5), but only ever reached via patched test bytes on the Write side |
| Positive E2E (real source) | Yes (D7/golden) | Yes (D7/golden) | **Yes** — `tests/sequence_ownership_golden.rs`, re-run this checkpoint: `ok` | **Yes (Borrow only)** — `vm_runs_adt_payload_ownership_positive_e2e_path`, re-run this checkpoint: `ok` |
| Negative E2E (real source) | Yes | Yes | **Yes, 8 scenarios** — `tests/runtime_ownership_e2e.rs` (same-index, sibling, parent/child, child/parent, 4× dynamic/static interaction), re-run this checkpoint: all 8 `ok` | **No** — frontend has no mutable ADT-payload reassignment syntax (confirmed still true, `semcode_vm.rs` `NOTE(ADT-4)` comment unchanged); substituted by 3 runtime-patched negative tests (`vm_rejects_adt_payload_write_when_borrowed_{same_payload,parent_overlaps_child,child_overlaps_parent}`), re-run this checkpoint: all 3 `ok` |
| Current normative documentation | `runtime_ownership.md` "Current supported slice" lists it | Same | **Not listed** in `runtime_ownership.md`'s supported slice; `semcode.md` explicitly disclaims it at `SEMCOD11`/`SEMCOD12` | **Not listed** in `runtime_ownership.md`'s supported slice; `semcode.md` explicitly disclaims it at `SEMCOD11`/`SEMCOD12`; `docs/architecture/adt_payload_ownership_paths.md` is additionally **stale** — it still says "sm-ir does not yet emit these paths (Pending ADT-2)" and "sm-vm does not yet process these paths (Pending ADT-3)," contradicted by the current, real, passing E2E test above |
| Stable Foundation target status | Frozen, included (Position A) | Frozen, included (Position A) | INCLUDED ONLY AFTER REQUIRED REPAIR (Position A) | INCLUDED ONLY AFTER REQUIRED REPAIR (Position A) |

## 4. Sequence falsification (item 8)

Re-ran, fresh, on this exact baseline (not inferred from old test names):

```
cargo test --test sequence_ownership_golden          -> 1 passed
cargo test --test runtime_ownership_e2e sequence      -> 8 passed
  (same_index_conflict_rejects, sibling_index_write_passes,
   parent_child_conflict_rejects, child_parent_conflict_rejects,
   dynamic_borrow_conflicts_with_static_index_zero_write,
   dynamic_borrow_conflicts_with_static_sibling_write,
   dynamic_borrow_conflicts_with_parent_write,
   parent_borrow_conflicts_with_dynamic_write)
```

Neither #1725 (root-identity) nor #1726/#1891 (event-timing/anchors) weakened
this evidence — both closed defects were in shared machinery this family
depends on, and every Sequence test above runs on the post-#1726/#1891
architecture and passes.

**Verdict: IMPLEMENTED AND QUALIFIED (runtime correctness) but
CONTRACT-UNDECLARED (capability/version authority).** The implementation
evidence bar here is at least as strong as tuple/record's own D7/golden
evidence — the only missing piece is the version/capability authorization
itself, which is exactly what #1718 is about.

## 5. ADT payload falsification (item 9)

Re-ran, fresh: `vm_runs_adt_payload_ownership_positive_e2e_path` (`ok`),
`vm_rejects_adt_payload_write_when_borrowed_same_payload` (`ok`),
`_parent_overlaps_child` (`ok`), `_child_overlaps_parent` (`ok`).

The historical limitation is checked directly against current `main`, not
assumed: `semcode_vm.rs`'s `NOTE(ADT-4)` comment, immediately following the
positive E2E test, is unchanged — "the Semantic frontend does not yet support
mutable bindings or mutable re-assignments of ADT payloads... we will add the
negative E2E test once the language surface is ready." **This limitation
still holds on the current baseline.** No later language work (this session
checked; nothing in the #1888 reconciliation, Lane 2, or any closed issue
since 2026-08 touches ADT-payload mutation syntax) removed it.

Two additional, current-source-confirmed weaknesses, both already named by
the Position A decision record and re-confirmed by direct code reading this
checkpoint:
- The verifier's variant `SymbolId` is never bounds-checked against the local
  string table ("structural acceptance only") — architecturally identical to
  how `Field`'s `SymbolId` is treated (per `#1725`'s finding: both are used
  purely as opaque, root-gated equality keys in `access_paths_overlap`, never
  resolved), so this is not a *unique* ADT defect, but it is a real,
  acknowledged gap the decision record explicitly tracks for this family.
- The Write side of `AdtPayload` has **zero compiler-reachable production
  path** on the current language surface — every `write_paths` entry
  containing `AdtPayload` in the test suite is manufactured by hand-patching
  compiled bytes, never emitted by the compiler itself, because there is no
  source syntax that would trigger it. This is a materially different
  situation from Sequence, where `write_paths` are genuinely populated by
  real compiled indexed writes.

**Verdict: PARTIAL.** Borrow-side: IMPLEMENTED AND QUALIFIED (real source,
real negative-conflict proof via runtime-patched Write against a real Borrow).
Write-side: IMPLEMENTED BUT NOT COMPILER-REACHABLE — proven only by
constructing synthetic bytes, not by any program a user could actually write
today.

## 6. Legacy admission audit (item 10)

Distinguishing the four axes explicitly, as required:

- **Current-compiler-production contract**: on this baseline, a program using
  only `Sequence`/`AdtPayload` ownership paths is promoted to `HEADER_V11` by
  the generic floor (§2) unless something else in the same program forces a
  higher header (e.g. `SEMCOD19`'s unconditional `SIG0` floor, per #1773 —
  the `enum`/`match` positive E2E test above almost certainly compiles under
  `SEMCOD19`/`SEMCOD20` for that unrelated reason, not because Sequence/ADT
  earned a higher floor on their own).
- **Legacy-decode-compatibility**: `semcode_decode.rs`'s component-kind
  `match` (§2) takes no header parameter — it decodes `AdtPayload`/
  `SequenceIndexStatic` identically whether the surrounding header is
  `HEADER_V11` or `HEADER_V20`.
- **Legacy-verifier-compatibility**: the verifier's capability check (§2) is
  the same regardless of header — any header carrying bare
  `CAP_OWNERSHIP_PATHS` (i.e. **every** header since `HEADER_V11`, all the
  way to `HEADER_V20`) admits both families with no additional requirement.
  A hand-built `SEMCOD11` (bare `HEADER_V11`) artifact carrying an
  `AdtPayload`/`SequenceIndexStatic` component today decodes and verifies
  successfully on this exact baseline, despite `semcode.md`'s own `SEMCOD11`
  section explicitly disclaiming exactly that content.
- **Legacy-runtime-execution-compatibility**: the VM conversion/enforcement
  path (§2) is likewise header-agnostic — once verified, both families
  execute identically regardless of which header revision admitted them.

**Conclusion: this is a single coherent contract gap across all four axes,
not four independent ones.** No header revision, current or legacy, has ever
been granted explicit authority over these two component kinds; the gap does
not narrow by waiting for old headers to age out, because current headers
have the identical hole.

## 7. Capability/version authority (item 5)

Reconstructed from version/spec history, not from decoder behavior (per the
brief's explicit instruction that "a decoder accepting a shape is not
sufficient evidence that the capability contract authorizes it"):

- `CAP_OWNERSHIP_PATHS` (bit 12, `HEADER_V11`): `docs/spec/semcode.md`'s own
  `SEMCOD11` section is the authority, and it is explicit: "promoted contract
  used when emitted program usage requires **tuple-only** ownership path
  metadata transport... does not claim record, ADT payload, schema, or
  release/lifetime transport beyond the current frame-local tuple slice."
  **Normative scope: tuple only.**
- `CAP_OWNERSHIP_FIELD_PATHS` (bit 13, `HEADER_V12`): `semcode.md`'s
  `SEMCOD12` section: "extends the ownership-path component vocabulary with
  `Field(SymbolId)`... does not claim ADT payload, schema, or
  release/lifetime transport beyond the current frame-local tuple+record
  slice." **Normative scope: tuple + direct record field only.**

Neither bit's text was ever revised by any later header (`SEMCOD13`-`SEMCOD19`
each document what *they* add — sequence iteration, `Map` values, `QTruth`,
callable signatures — and none amend the ownership-capability text quoted
above). **Today's normative meaning of both bits is exactly what it was in
2026-08: tuple, and tuple+record. Nothing in the current, freshly-read spec
authorizes Sequence or ADT under either bit.**

## 8. No Silent Mutation Rule — four hypotheticals (item 6)

`docs/spec/semcode.md`'s own rule (freshly re-read): "The following are
forbidden without a documented version change: repurposing an existing
capability bit; changing the meaning of an existing header family; changing
section interpretation while keeping the same public version."

| Hypothetical change | Verdict | Authority |
|---|---|---|
| Reinterpret `CAP_OWNERSHIP_PATHS` to also mean "tuple, or sequence, or ADT" | **FORBIDDEN** | Directly "repurposing an existing capability bit" — the bit's documented scope is tuple-only at `SEMCOD11` (§7). |
| Add Sequence/ADT admission to an already-frozen header revision (e.g. declare `HEADER_V20` "already covers this," no new bit) | **FORBIDDEN** | "Changing the meaning of an existing header family" — `HEADER_V11`-`V19`'s own text disclaims this content; the Version Policy section is explicit: "existing admitted header families remain fixed once they ship on `main`." |
| Change `HEADER_V20`'s semantic meaning in place (reuse unused reserved bits without a new header) | **FORBIDDEN** | Same clause; also violates "capability widening stays additive in the current baseline and must not repurpose existing bits." |
| Change legacy `SEMCOD11`/`SEMCOD12` interpretation (retroactively declare old artifacts were always allowed to carry these paths) | **FORBIDDEN** | Same clause, applied retroactively — historical header semantics do not get expanded after the fact. |

**All four are forbidden.** This has a direct, load-bearing consequence for
the decisions below: **PROMOTE cannot be satisfied by a documentation update
or a capability-bit reinterpretation alone.** It requires genuinely new
version/capability machinery — which is explicitly out of scope for this
checkpoint (item 12) and is deferred to the next implementation checkpoint
(§10).

## 9. Resolving the apparent Position A / #1718 contradiction (item 7)

The brief flags an apparent tension: the decision record's "INCLUDED ONLY
AFTER REQUIRED REPAIR" language versus #1718's own discussion of "capability
promotion OR fail-closed rejection" as both valid remedies. Triangulating
across three independent authorities:

1. **`ssf08_ownership_position_decision.md`** (Position A, frozen): classifies
   both families as "INCLUDED ONLY AFTER REQUIRED REPAIR," naming `#1718`
   (plus the now-closed `#1709`/`#1724`/`#1725`/`#1726`) as the repair gate —
   language that does not itself pick promote-vs-restrict.
2. **`stable_foundation_target_contract.md`** (independent authority, not
   authored as part of this audit trail): its "Candidate Foundation contour"
   lists "bounded `Sequence` and `Map` contracts" and "records, tuples,
   bounded enums/ADTs, `Option`, and `Result`" as part of the target language
   surface, and its "Unresolved decisions and owners" table states plainly:
   "Ownership Position A or B | SSF-08 | Position A selected...;
   implementation repair (#1656-#1664, #1709, #1718, #1724, #1725, #1726)
   remains before qualification closes." This confirms #1718 is anticipated,
   required repair work for SSF-08's *own* qualification — not merely this
   audit's own shorthand.
3. **`semantic_stable_foundation_matrix.md`** (independent maturity ledger):
   its "ADT/schema/sequence/map indirect ownership paths" row status is
   "Landed but unqualified," with the note: "Sequence static-index and ADT
   payload classified INCLUDED ONLY AFTER REQUIRED REPAIR under Position A...
   Map/schema/indirect projection remain deferred or not representable" —
   consistent with, and independent confirmation of, (1).

**Resolution**: there is no real contradiction. "INCLUDED ONLY AFTER REQUIRED
REPAIR" describes the *target-contour status* (these families are candidates
for the stable surface, gated on a repair). "#1718 requires capability
promotion OR fail-closed rejection" describes the *closure mechanism* for the
contract-authority gap itself — either remedy would satisfy AC1/AC2 (the
public claim matching implementation evidence), independent of whether the
families are ultimately promoted or restricted. Both statements are true
simultaneously; they operate at different levels (target-contour
classification vs. contract-mechanism closure). This document's job is to
pick, per family, which of the two closure mechanisms the evidence actually
supports (§11) — that choice was always open, exactly as #1718 itself framed
it; the decision record simply hadn't made it yet.

## 10. Falsification of RESTRICT/DEFER for both families

Before selecting PROMOTE, an honest attempt was made to prove RESTRICT or
DEFER is actually correct, per item 14's requirement that "a decision without
an attempted contradiction is incomplete."

**Attempted case for RESTRICT/DEFER:** ADT's Write-side is compiler-unreachable,
its verifier admission is structurally weaker, and Sequence/ADT were both
explicitly *not* promoted by the Position A record. A fail-closed rejection
of both families under all current headers would be the more conservative,
"nothing new admitted" choice, and would not need any new wire-format work at
all — just tightening the verifier.

**Why this fails:** implementing that rejection *today*, under current headers,
would immediately break already-passing, real, compiler-driven tests:
`tests/sequence_ownership_golden.rs` (positive), all 8
`runtime_ownership_e2e.rs` sequence conflict/interaction tests, and
`vm_runs_adt_payload_ownership_positive_e2e_path` — every one of these compiles
genuine Semantic source (indexed sequence writes/borrows; an `enum`/`match`/
`ref` program) through the *real* pipeline and gets a header that (per §6)
carries only `CAP_OWNERSHIP_PATHS`. Rejecting that header/component
combination outright means these real, working programs would stop
compiling-and-running successfully — a behavioral regression to already-shipped,
tested functionality. Item 14 names this exact failure mode: "real E2E
evidence already makes removal/narrowing dishonest." It applies here for both
families as a whole (each has at least one real, stable-looking example that
depends on the current admission behavior), which also independently rules
out DEFER's "no stable examples depend on it" condition (item 11) — that
condition fails for both families, not just one.

**Conclusion: RESTRICT/DEFER is falsified for both families as a whole.**
(This does not mean every sub-case is equally strong — see the Write-side
caveat for ADT below — but a family-level RESTRICT/DEFER cannot be adopted
without regressing real, current, passing behavior.)

## 11. Falsification of PROMOTE, and final decisions

**Attempted case against PROMOTE (Sequence):** implementation too incomplete?
No — 1 positive + 8 negative/interaction real-source tests, all freshly
re-run and passing, at least as strong as tuple/record's own bar. Semantics
differ from Position A? No — overlap enforcement uses the same
`access_paths_overlap` machinery already accepted for tuple/record. Verifier
cannot safely admit it? No structural weakness found — `SequenceIndexStatic`
carries a plain `u32` index, no unresolved `SymbolId` concern applies. Identity
unstable? No — closed by #1725/#1726, confirmed by the fresh re-runs in §4.
Negative conflict behavior unproven? No — 8 real scenarios, including
dynamic/static interaction, all passing. **This falsification attempt does
not survive.**

**Attempted case against PROMOTE (ADT):** implementation too incomplete? **Yes,
partially** — the Write side has no compiler-reachable path at all (§5), so
promoting the capability would authorize wire content (`AdtPayload` in
`write_paths`) that the current toolchain can only ever produce by hand-patching
bytes, not by compiling real source. Verifier cannot safely admit it? A real,
open, named weakness exists (unbounded variant `SymbolId`) — architecturally
identical to `Field`'s already-accepted treatment, but explicitly and
repeatedly flagged by the decision record as a still-open concern for this
family. **This falsification attempt partially survives** — it does not
defeat promotion of the concept (the Borrow side is fully qualified, real,
and — per §10 — already relied upon by a real passing test), but it does mean
ADT's promotion carries materially more required companion work than
Sequence's before AC1/AC2/AC5 can be marked satisfied.

**Decisions:**

- **Sequence (`SequenceIndexStatic`): PROMOTE.** Allocate explicit
  version/capability authority; the compiler's emitter should require it
  whenever a `SequenceIndexStatic` component is present (mirroring
  `has_v12_record_field_ownership_events`'s pattern); decoder/verifier must
  gate on the new capability, not the bare `CAP_OWNERSHIP_PATHS` bit; VM
  consumption is unchanged, since correctness is already proven; artifacts
  below the new capability/revision carrying this component kind must fail
  closed at admission (mirroring the #1891/W2F legacy-rejection precedent).
- **ADT payload (`AdtPayload`): PROMOTE**, on the same mechanism as Sequence,
  **with an explicit required companion condition**: the next implementation
  checkpoint must make an explicit, documented decision about the Write side
  specifically — either (a) accept it on the same architectural-trust basis
  already extended to `Field`'s unresolved `SymbolId` (per #1725's finding
  that this is sound because the value is used only as an opaque, root-gated
  equality key, never resolved), and promote Borrow+Write together, tracking
  the real-source negative E2E test as a still-open, frontend-blocked item
  (not a #1718 blocker, since it was never a #1718 blocker for tuple/record's
  own inclusion either); or (b) restrict Write-side `AdtPayload` admission
  specifically until a real-source negative test exists, promoting Borrow
  only. This document does not pick between (a) and (b) — that is
  implementation-encoding detail explicitly out of scope here (item 12) —
  but it must not be silently defaulted; the next checkpoint must decide it
  and record the reasoning.

Both decisions require genuinely new version/capability machinery per §8 —
neither can be satisfied by documentation alone.

## 12. Explicit non-goals of this document

Per item 12 of the governing brief, this document does **not** invent, name,
or select:

- a new header magic (`MAGIC21`/`HEADER_V21`) or numeric revision (`rev22`);
- a new capability bit number, or reuse of an existing bit's numeric value;
- any wire byte layout for a new component-kind/capability pairing;
- any downgrade/compatibility behavior for artifacts between the old and new
  contract;
- any mutation of `HEADER_V20`'s existing semantics.

All of the above are implementation-encoding decisions reserved for the next
checkpoint, once this contract decision is approved.

## 13. Effect on #1579

- **AC1** ("public ownership/memory claim matches implementation evidence"):
  still **NOT SATISFIED** for Sequence/ADT — the contract decision is now
  frozen, but no capability/version machinery exists yet to make the claim
  true. `docs/spec/semcode.md`, `runtime_ownership.md`, and
  `docs/architecture/adt_payload_ownership_paths.md` (confirmed stale, §3)
  all still need normative updates once implementation lands.
- **AC2** ("aggregate/path/frame/host ownership behavior is documented and
  tested"): behavior is tested (very thoroughly, for Sequence in particular)
  but **not yet documented** as part of the stable contour — still NOT
  SATISFIED for these two families.
- **AC5** ("verifier and runtime agree on admitted/rejected ownership
  states"): verifier and runtime already agree with each other (both admit
  and enforce these paths identically) — the disagreement is between
  implementation and the *written* capability contract, which is exactly
  what #1718 tracks. Still NOT SATISFIED until the new capability/version
  gate is implemented and legacy (pre-promotion) headers are proven to fail
  closed for these component kinds (mirroring the #1891/W2F precedent).

**#1718 ready to close: NO.** A contract decision was reached; no enforcement
of it exists yet.

## 14. Required next implementation checkpoint(s)

A single, bounded implementation checkpoint, scoped to:

1. Allocate a new capability bit and a new header revision (naming/numbering
   deferred to that checkpoint) gating `SequenceIndexStatic`/`AdtPayload`
   admission, following the `has_v12_record_field_ownership_events` pattern.
2. Update the emitter's header-promotion predicates to require the new
   capability whenever either component kind is present.
3. Update `sm-format`/`sm-verify` to require the new capability specifically
   for these component kinds (not just the generic `CAP_OWNERSHIP_PATHS` bit).
4. Add legacy-rejection tests: an artifact carrying these component kinds
   under any header below the new revision must fail closed at verification
   (mirroring `tests/write_execution_site_e2e.rs`'s
   `legacy_pre_rev21_write_bearing_artifact_rejected_at_runtime` precedent
   from #1891/W2F).
5. Decide and implement the ADT Write-side companion question from §11
   (accept on architectural-trust grounds, or restrict Write-side admission
   until a real-source negative test exists).
6. Update `docs/spec/semcode.md`, `docs/spec/verifier.md`,
   `docs/spec/runtime_ownership.md`'s "Current supported slice", and correct
   the stale `docs/architecture/adt_payload_ownership_paths.md`.
7. Update golden/compatibility fixtures and `ssf08_closure_audit.md`'s AC1/
   AC2/AC5 rows only once this lands and is qualified.

**Wait for explicit GO before implementing the selected contract.**
