# #1726 explicit Borrow execution-site design and proof

Status: proposed design; production implementation is NOT authorized by this document.
Evidence checkout: `codex/1726-own0-v20` at
`8b67cc65a87e361778e7a9c597fcd530d8a524c2`.
Scope: frozen Tuple/Record v0 activation timing. #1891 remains a separate
merge-blocking dependency. No #1718, #1888, ADT lowering, Write-cursor, lifetime,
root-copy, capability, or unrelated section changes.

The owner's current instruction supersedes the target-only design in
[the earlier issue comment](https://github.com/skulmakov-oss/Semantic/issues/1726#issuecomment-5551561220).
The owner accepted two counterexamples: O1 removes unreachable StoreVars while
retaining Borrow events; a lowered-local key identifies a binding, not its
unique introduction instruction. Neither counterexample is treated as permission
to guess or weaken metadata validation.

## Decision

Use an explicit, function-local `ActivationSiteId` attached to the introducing
IR StoreVar, carried by its Borrow event, and relocated to the actual final
instruction-relative PC during emission. Keep StoreVar's executable opcode and
operands unchanged. Retain an explicit FrameEntry mode for the existing ADT scan.

For V20 admission, include a compact reverse anchor table **inside OWN0**,
generated from the annotated StoreVars. Cross-check it against Borrow records.
This costs six bytes per site plus a two-byte count and detects one-sided stale
PC metadata, including retargeting to another StoreVar of the same local. It is
not source authentication; see the precise limitation below.

Invariant:

```text
source Borrow introduction
  = StoreVar carrying ActivationSiteId S
  = Borrow event referring to S
  = surviving emitted StoreVar at relocation[S].pc
  = successful VM execution at that exact function-relative PC
```

The equality concerns explicit association and execution, not equality of the
different representations' raw integers. Site IDs, local keys, string-table
indices, event indices, and instruction PCs remain different identity domains.

## Current repository evidence

Paths and lines below refer to the exact evidence checkout, before implementation.

| Fact | Evidence |
|---|---|
| StoreVar contains only name and source register | `crates/sm-ir/src/legacy_lowering.rs:195` |
| All four producers have the function's mutable LoweredLocalEnv | `legacy_lowering.rs:4838`, `4926`, `5016`, `5912` |
| Reassignments reuse the same lowered key | `legacy_lowering.rs:6414`, regression `15213` |
| LoweredLocalEnv is function-local; lifted functions get a fresh one | `legacy_lowering.rs:10485`, `2524` |
| Cleanup deletes after Ret/Jmp until a label | `crates/sm-ir/src/passes/cleanup.rs:54` |
| CrystalFold reconstructs StoreVar unchanged today | `crates/sm-ir/src/passes/crystalfold.rs:289` |
| Actual output length is available immediately before emission | `legacy_lowering.rs:1526` |
| Verifier records exact decoded instruction boundaries | `crates/sm-verify/src/lib.rs:1058` |
| StoreVar operand decoding knows its target index | `crates/sm-verify/src/lib.rs:3587` |
| Raw VM loading already scans instruction boundaries | `crates/sm-vm/src/semcode_vm.rs:1195` |
| VM retains current instruction PC during dispatch | `semcode_vm.rs:1840` |
| Successful local insertion follows the existing Write guard | `semcode_vm.rs:2395` |

Real source probes cover four producer forms at O0 and O1. Each emits two
StoreVars sharing one exact lowered key and string-table entry, and passes
emission, admission, and execution. A further source probe exercises both paths
of record let-else with a tuple Borrow inside its else-return expression:

```text
Borrow metadata order: outer record root, inner tuple root
StoreVar order:        inner tuple binding, outer record binding
```

This is measured at both O0/O1 in `tests/borrow_target_identity_proof.rs`.
The new design must preserve this separation without moving lexical binding
installation into the initial record-pattern scan.

## Alternatives and cost

| Shape | Correct execution identity | Stale metadata checks | Decision |
|---|---|---|---|
| IR site ID -> OWN0 final PC + target only | Exact PC; no target-name activation | Range, boundary, opcode, target; cannot detect a one-sided stale PC landing on another StoreVar of the same local | Smaller by 6 bytes/site, insufficient for that explicit negative case |
| Same relocation plus reverse `(borrow_event_index, pc)` anchors | Exact PC, independently cross-checked associations | Adds missing/orphan/duplicate/one-sided PC disagreement rejection | Selected |
| Single separate `(site_id, pc, target)` table with Borrow -> site_id references | Exact explicit lookup | Moving the only PC into a table does not itself detect stale same-target PC | Valid basic representation, no advantage over inline PC without an independent association |
| Stable IDs/metadata wrapper on every IR instruction | Can express the contract | Requires changing every pass/instruction access | Unnecessary; annotate only Borrow introduction StoreVars |
| New Borrow-activation opcode, or versioned site operands on every StoreVar | Can express the contract | Still cannot authenticate coordinated artifact rewriting | Rejected: existing PC dispatch and non-executable tables suffice |

A final instruction ordinal can replace a PC only if explicitly relocated and
encoded, then resolved from decoded boundaries. It saves no width here and adds
an ordinal-to-PC mapping. Existing byte-PC infrastructure makes PC the smaller
implementation. The selected event index below is a serialized foreign key, not
an inference from source/event/instruction order.

## Minimal IR representation

Conceptual Rust surface (proposal only):

```rust
struct ActivationSiteId(u32);

enum BorrowActivation {
    FrameEntry,
    AtStore(ActivationSiteId),
}

enum OwnershipPathEventKind {
    Borrow(BorrowActivation),
    Write,
}

// Added to the EXISTING IR StoreVar variant; no new executable opcode:
StoreVar {
    name: String,
    src: u16,
    activation_site: Option<ActivationSiteId>,
}
```

`OwnershipPathEvent.path` remains unchanged. AtStore carries no guessed local
name: the unique annotated StoreVar supplies its exact target during emission.
Write cannot carry activation metadata. FrameEntry is an explicit variant,
never a missing-site fallback.

Allocate IDs from a dedicated checked u32 counter in the existing per-function
lowering identity context (`LoweredLocalEnv` already reaches all four producers).
Keep it separate from `next_id` for local keys. Scope push/pop must not reset it;
branches must share the same counter, not clone it. Lifted functions already
receive a fresh context, so IDs may repeat across functions. Checked overflow is
a compile error. Do not derive IDs from names, source symbols, registers, vector
positions, event counts, PCs, or labels. No global allocator is necessary.

Only introducing StoreVars with an actual in-scope Borrow event receive Some(S).
Other stores, including later reassignments, have None and distinct final PCs;
they do not need IDs in the activation namespace. This is smaller than assigning
an ID to every instruction. Copying S onto another static StoreVar is invalid.

Construction rules:

1. Plain tuple/record: mint S at the existing capture producer; attach S to the
   exact StoreVar being constructed and emit Borrow(AtStore(S)).
2. Tuple let-else: create the pair when constructing the existing deferred bind's
   StoreVar after the pattern checks.
3. Record let-else: reserve S when appending the initial Borrow event. Carry S
   explicitly in that particular deferred-bind record. Install the lowered local
   key at its existing final-loop point; attach S to that exact StoreVar. An
   else-return can create other sites in between without changing the pairing.
4. Mixed move/ref bindings: only ref bindings that produce Borrow metadata carry
   a site. Distinct ref bindings to identical paths get distinct IDs only when
   lowering emits distinct Borrow events. Existing conservative dynamic-root
   deduplication remains one event and one site, paired with the first ref
   introduction that activates it; later refs represented by that event do not
   receive orphan StoreVar markers.
5. The ADT/Option/Result prescan emits Borrow(FrameEntry), including its existing
   dynamic-root behavior. It does not allocate or guess a site. A path's shape is
   NOT used to infer the producer, because an ADT dynamic fallback can be a root.
6. Existing path-production scope and dynamic-path deduplication are unchanged.
   Where tuple lowering already emits one conservative dynamic-root event, pair
   that existing event with its actual first ref introduction; do not add broader
   Sequence/ADT ownership semantics. No event that exists may lose its site.

### Deduplicated dynamic-root co-execution proof

**Two separate claims, not one.** This section originally asserted, in one
breath, both (a) a *lowering control-flow safety* property — if the
dynamic-root dedup branch is ever exercised, a single shared site is sound —
and (b) that "the N:1 source case is reachable." Checkpoint D2b.5
(2026-09-06 reconciliation, prompted by a reviewer catching the contradiction
against D2b's own item-5 finding before D3 could turn it into a runtime
soundness defect) established that only (a) was ever actually verified; (b)
was an unverified assumption stated as fact. The two claims are now kept
distinct.

**(a) Lowering control-flow safety — proven, unchanged, still the governing
invariant.** For each such call, the loop scans pattern items in source
order. It emits the `StoreVar` before handling that item's capture.
`emitted_dynamic_root` starts false and is set only when the first borrowing
item appends the single event; there is no conditional jump, return, or loop
back-edge inside this scan. A later represented borrowing item is therefore
reached only after that first `StoreVar` has been emitted and successfully
executed in the same pass. Discard items use `continue` before any borrow
and cannot become the chosen first item. In let-else lowering, pattern
checks and their failure returns precede the deferred-bind loop; all
deferred stores are emitted only after every check has passed, in the same
linear loop, so failure skips all of them, never a later one alone. A match
arm or branch invokes lowering separately and therefore receives its own
event/site; sites are never shared across arms. Loop re-entry repeats the
same static pair and cannot execute the later store without traversing the
earlier store in that iteration. The model regression
`test_deduplicated_first_site_is_required_for_every_later_ref`
(`tests/borrow_site_design_model.py`) records the forbidden split trace. This
proof is scoped to the frozen producer implementations' own internal control
flow, *if called* with a dynamic-fallback path and 2+ Borrow items — it is a
design-model proof, not an end-to-end compiler/VM run (the model file's own
header says so explicitly), and it never claimed anything about whether
today's frontend can construct such a call.

**(b) Source-level reachability — audited directly (Checkpoint D2b.5) and
found false today.** `SequenceOwnershipPath::DynamicFallback` has exactly
one syntactic root cause anywhere in `sm-ir`: a `SequenceIndex` link (at any
depth) whose index is not a non-negative integer literal
(`sequence_access_path_from_expr`'s only two match arms are `Expr::Var` and
`Expr::SequenceIndex`; every other shape returns `Ok(None)`). Independently,
`sm-front::typecheck::apply_arm_pattern_capture` — a general ownership-
capture safety rule from SSF-08 Lane 1 (#1661/#1663) that predates #1726 and
has nothing to do with it — rejects any capturing pattern whose scrutinee is
projection-shaped (`SequenceIndex`/`RecordField`) and unresolvable by
`expr_access_path`, which resolves a `SequenceIndex` only for a literal
non-negative index — the identical condition. This one function is called,
confirmed by direct grep of every call site, for all four of `Stmt::LetTuple`,
`Stmt::LetRecord`, and their let-else counterparts — i.e. for all four frozen
producers' own entry statements, uniformly, before any of them lower. The
single syntactic cause of `DynamicFallback` and the condition this pre-
existing gate rejects are one and the same condition; there is no second
route to `DynamicFallback` the gate could fail to cover. Verified directly
against real compiled source (not inferred) for all four shapes — plain
tuple, let-else tuple, record, let-else record — each rejected with the
identical typecheck error, before lowering. Locked by
`ssf08_1726_checkpoint_d2b_dynamic_index_ref_capture_is_rejected_before_any_dedup_branch_runs`
(`crates/sm-ir/src/legacy_lowering.rs`).

**Authoritative conclusion: currently unreachable (case B).** No admitted
program today exercises any frozen producer's dynamic-root dedup branch for
a Borrow event. The branch is defensive/dead code under the current source
surface. This does **not** retire proof (a): it remains the governing
invariant the four producers must continue to satisfy, and the one any
future change must re-derive before relaxing SSF-08 Lane 1's gate, adding a
producer, or letting `DynamicFallback` arise by a new route — reusing the
single dynamic-root site without re-proving (a) for whatever new call shape
reaches it would be unsound. D3 does not need to reserve any runtime
provision for "one Borrow event backing more than one live ref" today, but
must not assume that stays true forever without re-checking this section
after any frontend or lowering change that touches `SequenceOwnershipPath`,
`apply_arm_pattern_capture`, or the four producers.

## Structural proof and optimizer coherence

For each function, validate before optimization and again before emission:

- Annotated StoreVar site IDs are unique.
- Every AtStore event has exactly one annotated StoreVar with that ID.
- Every annotated StoreVar has exactly one AtStore event. No orphan markers.
- FrameEntry and Write have no site references.
- No site is attached to another instruction kind.

This is a bijection for the frozen producers (one path per introduction). Reject
duplicate references instead of inventing multi-event activation semantics. The
distinct-ID case preserves repeated identical source paths without deduplication.

The smallest place with authority to erase a pair is
`remove_unreachable_until_label`, while it is actually deleting an annotated
StoreVar under its existing Ret/Jmp proof. Give that transformation access to the
function's ownership events. Validate input before mutation, record the IDs on
the exact instructions it deletes in that branch, then remove only their paired
AtStore events. FrameEntry and Write events are untouched, including unreachable
ones; #1891's Write-cursor problem is not repaired or masked.

Record the pre-pass site/event associations. Post-pass associations must equal
the old associations minus precisely the locally recorded unreachable removals.
Check both sides: a pass losing BOTH marker and event without that proof must
also fail, not appear valid just because the final bijection is empty. Removal
receipts are temporary pass-local data, not a wire section or global tombstone
registry. Emit no partial artifact after an error.

Association conservation alone is insufficient: moving a marker from an
introduction to a same-target reassignment preserves that map. Each existing
pass must also preserve the marker at its actual input instruction origin.
For unreachable cleanup, record the input index whenever the traversal retains
an instruction, and record deleted input indices in the proven-unreachable
branch. Check that the retained sequence equals those exact input instructions,
with ordered, unique origins and complete coverage except the recorded deletions.
These temporary indices are transformation provenance recorded during the
operation, not guessed activation ordinals or serialized site identities.

Other cleanup operations cannot delete StoreVars without an explicit removal
receipt. CrystalFold must check its StoreVar output against that exact match-arm
input for binding name and activation annotation, while retaining its existing
input traversal order. A legitimate source-register rewrite is allowed only
through the pass's existing checked rewrite semantics; the marker may never move
to another input instruction. An end-of-pass site map comparison cannot
substitute for this local check. Dead-arm deletion therefore needs a removal
receipt; without one CrystalFold fails closed.
Check preservation of site associations around CrystalFold without any removal
allowance. Do not renumber sites, preserve dead stores, or move a marker to a
different store. These checks complement inspection of the existing transforms;
they are not a proof that arbitrary future optimizer code preserves semantics.

The current `OptPass::run` and `run_default_opt_passes` return infallible
`OptReport`. The narrow honest error route is to make the existing pass result
fallible and propagate it through the two pass implementations and compiler
callers. Keep OptReport's statistics; do not add a generic optimization framework
or hide validation failure in `changed = false`. Direct public pass invocation
must validate too, so malformed input cannot be optimized away before emission.
Public API contract baselines will need intentional updates in implementation.

O0 validates and retains unreachable instruction/event pairs: they are real
emitted anchors which cannot execute. O1 removes a pair only under the described
local proof. Labels conservatively end that proof; do not add CFG reachability
analysis merely to erase more metadata.

**Implemented and qualified** (Checkpoint C, post-design): `StructuralCleanupPass`
gained an explicit removal receipt from its unreachable-code deletion (the exact
`ActivationSiteId`s of any annotated StoreVar it deleted); the paired Borrow
event is removed by that receipt only, never inferred from "no StoreVar found
for this site". `CrystalFoldPass` gained an exact before/after
(name, activation_site) passthrough check over every StoreVar. Both passes call
`validate_activation_sites` before and after running. `OptPass::run` and
`run_default_opt_passes` are now fallible (`Result<OptReport, OptError>`),
propagated through all production and test call sites. New regression tests:
a positive proof that the original counterexample program
(`return; let (ref left, _) = pair;`) now compiles under O1 with the paired
event coherently removed, plus two fail-closed negative controls (duplicate
site, orphan Borrow-event site).

### Normative boundary for future optimization passes

This coherence proof holds for `StructuralCleanupPass` and `CrystalFoldPass`
specifically because every rewrite either passage performs is one of: keep a
surviving instruction exactly as-is and in its original relative order, drop
it wholesale under a proven local removal receipt, or rewrite a non-StoreVar
instruction's own operands in place. Neither pass clones, splits, merges,
substitutes, re-materializes, or reorders an annotated `StoreVar` relative to
its siblings.

Any future pass capable of cloning, splitting, merging, moving, substituting,
or re-materializing an annotated `StoreVar` must not inherit this proof
automatically. `validate_activation_sites` re-checks that the pre/post
bijection still holds, but it cannot by itself distinguish a legitimate
introduction from a marker coherently (but wrongly) relocated onto a different
same-target instruction — the exact "moving a marker to a same-target
reassignment" hazard named above. Such a pass must define and implement its
own explicit provenance/removal-receipt mechanism, mirroring
`remove_unreachable_until_label`'s, before this checkpoint's coherence claim
extends to it. `ActivationSiteId` is authority-bearing metadata, not an
ordinary IR field a new pass may reposition freely. This is also encoded as a
doc comment on the `OptPass` trait in `crates/sm-ir/src/passes/mod.rs`.

## Emitter relocation and V20 wire layout

During the existing actual instruction-emission loop, immediately before writing
an annotated StoreVar, record:

```text
site_id -> (instr_stream.len(), exact StoreVar string-table target index)
```

The first value is checked into u32. Build it from actual emitted bytes, not
`encoded_size` estimates, debug tables, target searches, or nearby instructions.
Resolve each AtStore event using this map and assign its explicit event-table
index. Generate reverse anchors by visiting the recorded annotated StoreVars,
joining by site ID to that index. Do not generate reverse PCs by copying OWN0
Borrow record PCs. Duplicate/missing associations are hard errors.

Use deterministic event order for the event array and emitted instruction order
for the anchor array; never rely on HashMap iteration for encoding or diagnostics.
All indices and counters use checked conversions. Metadata roots still use
strict StringInterner lookup under #1725. Metadata may not create a missing local.

`MAGIC20 = "SEMCOD20"`, `HEADER_V20.rev = 21` (header suffixes are zero-based;
V19.rev is 20). Capabilities are inherited unchanged from V19.
`SEMCODE_OWNERSHIP_SITE_MIN_REVISION = HEADER_V20.rev`.

All integer fields below are little-endian. The proposed V20 OWN0 grammar is:

```text
"OWN0"
event_count:u16
events[event_count]:
    Borrow: kind:u8=0, activation_mode:u8
        mode=0 (FrameEntry):
            path
        mode=1 (AtStore):
            activation_pc:u32
            target_string_index:u32
            path
    Write: kind:u8=1, path             # unchanged bytes

anchor_count:u16
anchors[anchor_count]:
    borrow_event_index:u16
    activation_pc:u32

path = root_string_index:u32, component_count:u16, components...
# Existing component tags and payload widths are unchanged.
```

Anchor event indices count ALL OWN0 events, including Writes. They are explicit
wire references assigned by the emitter after identity resolution, never an
ordinal-matching heuristic. Each AtStore Borrow has exactly one reverse anchor;
each reverse anchor references exactly one AtStore Borrow. No two such records
may name the same PC in this slice. `anchor_count <= event_count`.

For V20 functions, OWN0 (including a zero anchor count) is mandatory. The existing
DBG0 placement remains unchanged; SIG0 follows OWN0 as before. No new section tag,
new executable opcode, StoreVar operand changes, or unrelated section redesign.
Header selection promotes artifacts containing site-bearing ownership to V20;
ADT-only/default programs can remain V19. A mixed artifact encodes its ADT events
explicitly as mode 0 under V20.

Pre-V20 selects the old Borrow grammar and has no anchor trailer. V20 selects the
new grammar strictly from the header revision. No sniffing between grammars,
retry under an older grammar, or malformed-mode conversion. The old V19 eager
semantics remain unchanged. Existing golden byte/header expectations must be
updated only where the new in-scope format intentionally changes the artifact;
Option/Result observable behavior must stay stable.

## Admission and execution

sm-format checks marker/kind/count/component/truncation structure and explicit
anchor references. Unknown modes, absent fields, duplicate/orphan references, and
count inconsistencies reject. V20 target/root indices must be within that
function's string table. It does not scan executable instruction semantics.

sm-verify reuses its decoded instruction-start collection. For each AtStore:

1. PC is in range and is an actual instruction start, not an operand byte.
2. The opcode is StoreVar, and its decoded target index equals the record target.
3. The independently emitted reverse anchor names this event and the same PC.
4. The mapping is bijective and there are no anchors on FrameEntry/Write.

Use the existing operand walk to collect StoreVar `(pc, target)` facts. A generic
"variable reference" list cannot distinguish LoadVar from StoreVar. Report
InvalidOwnershipSection deterministically. Do not require reachability at O0.

The documented raw VM path must mirror the anchor validation in its existing
function-bytecode validation walk, rejecting with BadFormat before any function
runs. Retain the #1725 string-table-to-runtime-symbol remapping, including target
validation. Trusted execution remains verifier-token-first.

At frame creation, mode-0 paths are active. Mode-1 paths are pending in a
frame-local map keyed by final instruction PC. In the StoreVar handler, AFTER
the existing operand/source/write checks and successful `locals.insert`, use
the original dispatch PC to remove and activate that entry. Do not use target,
the advanced operand cursor, next_pc, event order, or next_write_path.

The existing Write-cursor block stays unchanged. Repeated execution of the same
site removes no further pending entry: activation is idempotent. Reassignments
at other PCs cannot activate it even when their target is identical. Activation
persists through lexical scope/loop exits and ends only with frame exit. New
frames get independent pending/active state; no lifetime analysis is introduced.

## Failure-model boundary: consistency is not source authentication

The selected two-record association rejects an OWN0 PC changed to a same-target
reassignment while its reverse anchor is unchanged, and vice versa. It also
rejects an IR event referencing a missing/duplicate site before cleanup/emission.

If BOTH artifact records are coherently rewritten to another valid same-target
StoreVar, the unsigned artifact declares a different exact activation point and
is structurally admissible. Neither this design, a longer site ID, a checksum
carried in the same artifact, nor a new opcode proves its original source
provenance. A stale relocation implementation that emits both records wrongly
can likewise evade structural cross-checks; constructor/optimizer/emitter
regressions must guard that compiler behavior.

Thus the proof has two separate statements: the compiler preserves the explicit
source association; admission validates the artifact's internally declared exact
anchor. It does NOT promise rejection of every coordinated semantic rewrite of
an unsigned artifact. Requiring that stronger property needs a separate trusted
source/manifest binding decision, outside #1726; it cannot be silently claimed.

## Proof cases and executable evidence

| Case | Why the association holds | Evidence category |
|---|---|---|
| Introduction then one/multiple reassignments | Only the exact introduction owns S; other StoreVars have no activation marker and other PCs | Four real producer probes at O0/O1; model later-store-only traces |
| Shadowing | Distinct constructed sites; local keys remain scope-aware | Model distinct targets and reordered events; current #1724 authority |
| Identical source paths | Events are joined by distinct S, never by path equality | Model duplicate paths with distinct sites |
| Taken / untaken branch | Activation depends only on visiting the admitted PC successfully | Explicit model traces; future V20 source-to-VM regression required |
| Loops | One static S/PC, repeated visits; per-frame pending removal is idempotent | Five repeated model visits; future dynamic VM regression required |
| Deferred record let-else | S travels in its deferred record; event/instruction order may reverse | Real both-branch O0/O1 probe plus reversed model mapping |
| Unreachable introduction after return | UCE collects S from the exact removed instruction, deletes only paired event | **Implemented and real**: `legacy_lowering.rs::ssf08_1726_checkpoint_c_unreachable_borrow_introduction_removed_coherently` compiles the counterexample through the production `compile_program_to_ir_with_options` pipeline at O1 and asserts the paired event is gone, not orphaned; plus model O0 retained/O1 removed |
| Dead reassignment, live introduction | Removed reassignment has no S, so no authority to remove Borrow | Model negative deletion case |
| Reachable site, different O0/O1 byte positions | Same S, recomputed actual PC after removed bytes | Model relocation shifts by seven bytes; pair unchanged |
| Missing/duplicate site, marker on wrong kind | Prevalidation rejects before UCE can hide corruption | Model negative cases |
| Both sides disappear without UCE proof | Pre/post conservation fails despite empty final bijection | Model negative case |
| Marker transferred to same-target reassignment | Exact input-origin preservation rejects it even though site/event map stays equal | Model mutation and forged-origin rejection; independent review finding resolved |
| Truncated/unknown mode, bad index/boundary/opcode, stale anchor | Structural and executable checks reject | Reduced wire/anchor model, not production V20 decoding |
| ADT FrameEntry / Write | No site references or reverse anchors; existing behavior preserved | Model mode separation; baseline PCC6 golden |

`tests/borrow_site_design_model.py` is a standalone standard-library proof model,
not production code. It models site construction, the existing local UCE proof,
actual-byte-length relocation for a small instruction subset, the proposed OWN0
layout, reverse cross-checks, and explicit StoreVar execution traces. It is NOT
a full source compiler, CFG interpreter, V20 decoder, or end-to-end Semantic VM
test. Its jump operands are placeholders because traces are supplied explicitly.
Those limits are intentional and must remain visible in any implementation handoff.

Commands at the evidence checkout:

```powershell
cargo test --test borrow_target_identity_proof -- --nocapture
python tests/borrow_site_design_model.py
cargo check --workspace --all-targets
cargo test -p sm-ir
cargo test -p sm-emit
cargo test -p sm-format
cargo test -p sm-verify --features sm-ir/profile-rust
cargo test -p sm-vm
cargo test -p sm-runtime-core
git diff --check
```

At design qualification: 2 Rust tests passed (eight producer/optimization
combinations plus both paths of the deferred record source at both levels), and
15 Python model tests passed. The older red file `tests/borrow_activation_v20.rs`
is historical reproduction evidence, not a passing V20 suite; its original
unconditional survival assertion is superseded by the owner's revised invariant.

At Checkpoint C qualification (post-design): `cargo test -p sm-ir` 156 passed
(152 Checkpoint B plus 4 new Checkpoint C tests: the coherent-removal proof, two
fail-closed negative controls, and the real end-to-end counterexample
compilation); `sm-emit` 5; `sm-format` 30; `sm-verify` **182** passed (see
canonical-invocation note below); `sm-vm` 118; `sm-runtime-core` **9** passed.
Full root-package `cargo test --tests`: 148 test binaries green, except the 3
pre-existing, intentionally-red `tests/borrow_activation_v20.rs` cases (VM/wire
lazy activation, out of scope until Checkpoint D). `git diff --check` clean.

**Canonical invocation note**: `cargo test -p sm-verify` in total isolation is
NOT a valid qualification invocation for this crate. `sm-verify/Cargo.toml`
deliberately depends on `sm-ir` with `default-features = false, features =
["std"]`, excluding `sm-ir`'s own `profile-rust` default feature. Built alone,
`sm-ir` compiles without `profile-rust`, so every `sm-verify` test that
compiles a `CompileProfile::RustLike` program panics
(`"RustLike profile is disabled at compile time"`) — 182 tests discovered, 122
passed, 60 failed, confirmed by exact binary name (`sm_verify-*.exe`), not a
code regression. `cargo test -p sm-verify --features sm-ir/profile-rust`
(182/182) or building sm-verify alongside sm-ir in the same invocation (e.g.
`cargo test --workspace`, this program's standard full-qualification command
throughout SSF-08) both resolve it correctly. `sm-runtime-core` has no such
dependency and is unaffected (9/9 in isolation or otherwise).

## Implementation perimeter, after explicit authorization

Necessary production surfaces: `sm-ir` event/StoreVar types and four producers;
its two existing optimizer passes and fallible orchestration; emitter site
relocation; `sm-format` V20 OWN0 decoding; `sm-verify` structural site admission;
`sm-vm` raw validation, frame state, and successful-StoreVar activation. Normative
runtime ownership/SemCode/verifier specifications and public API baselines must
be synchronized within the active Harness.

Implementation qualification must replace the proof model with public-boundary
regressions for all cases above, actual O0/O1 source-to-verified-VM execution,
V19/V20 compatibility, existing ownership/PCC6 goldens, corruption and semantic
mutation kills, affected-crate suites, repository R3 checks, exact-head hosted
CI/security, and fresh review. No merge/auto-merge. #1891 stays visible as a
merge blocker regardless of local activation-test success.

Production IR scaffolding has already been introduced in `legacy_lowering.rs` and
`crystalfold.rs`: `ActivationSiteId`, the `activation_site` representation fields
on `IrInstr::StoreVar` and `OwnershipPathEvent`, deterministic per-function site
allocation, and site wiring for the four frozen Tuple/Record producers. No OWN0
V20 wire format, verifier admission, VM activation, or optimizer
removal-coherence implementation has been completed yet. No normative
specification or public issue comment was changed.

## Checkpoint C: implemented and qualified

`StructuralCleanupPass` gained an explicit removal receipt from its unreachable-
code deletion (the exact `ActivationSiteId`s of any annotated StoreVar it
deleted); the paired Borrow event is removed by that receipt only, never
inferred from "no StoreVar found for this site". `CrystalFoldPass` gained an
exact before/after `(name, activation_site)` passthrough check over every
StoreVar. Both passes call `validate_activation_sites` before and after
running. `OptPass::run`/`run_default_opt_passes` are fallible
(`Result<OptReport, OptError>`). New tests: the coherent-removal proof, two
fail-closed negative controls (duplicate site, orphan Borrow-event site), and
a real end-to-end counterexample compilation
(`ssf08_1726_checkpoint_c_unreachable_borrow_introduction_removed_coherently`).
A normative boundary doc comment on the `OptPass` trait (mirrored above,
"Normative boundary for future optimization passes") states this coherence
proof does not automatically extend to a future pass with clone/merge/
substitute/move authority.

## Checkpoint D1: implemented and qualified — IR ActivationSiteId to ExecutableAnchor

VM PC identity, confirmed by direct reading of `sm-vm/src/semcode_vm.rs`:
`Frame.pc: usize` is a byte offset relative to the function's `instr_start`
(`cur = f.instr_start + pc`, then the opcode byte is read from `cur`) — the
same domain `DebugSymbol.pc` already uses.

`ExecutableAnchor(pub u32)` and `BorrowActivationResolved { FrameEntry,
StoreVarSite(ExecutableAnchor) }` were added next to `ActivationSiteId` in
`legacy_lowering.rs`. Resolution happens inside `emit_semcode_function`'s
existing real-emission loop (the one that already builds `dbg` for
`DebugSymbol`s) — the anchor is `instr_stream.len()` at the exact moment
`emit_instr` is about to write that StoreVar's real bytes, never an IR index,
source order, event order, or `encoded_size`'s pre-emission length
prediction. `validate_activation_sites` is called at the top of
`emit_semcode_function` too, closing an O0 gap (O0 runs no optimizer pass, so
nothing previously validated lowering's own direct output before emission).
Fails closed on: two sites resolving to the same StoreVar, a Borrow event
whose site has no surviving anchor ("coherence failure", never "probably
dead" — Checkpoint C's removal receipt is the only sanctioned place a
Borrow-introducing StoreVar may be deleted), and anchor arithmetic overflow
(`checked` via `u32::try_from`).

6 new tests, all real (not model-only): exact anchor resolution verified
against a fully emitted artifact decoded by `sm-format`'s own decoder (not a
hand-computed offset); reassignment irrelevance; two independent introductions
resolve to distinct anchors; two shadowed same-spelling bindings in separate
scopes resolve to distinct anchors; a Borrow inside an `if` branch resolves to
a real static anchor independent of the branch condition's value (static
presence is not dynamic activation — that distinction is Checkpoint D3's,
not D1's); the ADT/Option/Result producer stays `FrameEntry`. Mutation test:
derived the anchor from an IR-instruction ordinal instead of the real
`instr_stream.len()` position — the anchor-verification test failed exactly
as required (wrong byte at the claimed anchor), reverted, reconfirmed green.

No OWN0 wire changes, no `sm-verify` changes, no `sm-vm` changes. `#1891`
untouched. Files changed: `crates/sm-ir/src/legacy_lowering.rs` only (plus the
Checkpoint C files, unchanged further in D1).

**Qualification wording correction**: `cargo test --workspace` is not
"globally green" while `tests/borrow_activation_v20.rs`'s 3 intentionally-red
cases remain red (VM/wire lazy activation, out of scope until Checkpoint D3).
Correct framing:

```text
D1 acceptance envelope       PASS
Affected regressions         PASS
Workspace                    3 known expected-red tests (borrow_activation_v20.rs)
Full #1726 qualification     NOT YET GREEN
```

## Checkpoint D1.5: header-revision allocation, audited — no collision found

**Correction to an earlier claim of mine**: I previously flagged a "naming
collision" between this design's proposed `HEADER_V20` and #1773/SIG0's
revision, based on `sm-format`'s own test names (`decode_pre_rev20_header_...`,
`decode_rev20_header_...`). That flag was wrong — a misreading of test-name
shorthand, not a verified fact. Audited directly on this worktree:

- `crates/sm-format/src/local_format.rs` defines `HEADER_V0` through
  `HEADER_V19` only (grepped `pub const HEADER_V` — nothing higher exists;
  `MAGIC20` does not exist anywhere in the tree either).
- Every `HEADER_Vn` follows `rev = n + 1` exactly (`HEADER_V0.rev = 1`,
  ..., confirmed by direct read of each constant's literal `rev` field).
  Therefore `HEADER_V19.rev = 20`.
- `SEMCODE_SIGNATURE_MIN_REVISION = HEADER_V19.rev` (i.e. **20**) — #1773/SIG0
  requires revision 20, which **is** `HEADER_V19`, not a separate struct. The
  "rev20" in `sm-format`'s test names refers to this rev *value*, not to a
  distinct `HEADER_V20` constant. There is no second definition at rev 20 and
  no existing definition at rev 21.
- `supported_headers()` returns exactly `[HEADER_V0, ..., HEADER_V19]` — the
  decoder accepts nothing above rev 20 today.
- Conclusion: **`HEADER_V20` with `rev = 21` (as originally proposed) is
  genuinely the next free, unclaimed revision.** The original design's
  `MAGIC20 = "SEMCOD20"` / `HEADER_V20.rev = 21` numbers were correct; only my
  later claim that they collided was in error.

**Feature-floor composition, proved from the actual emitter** (`emit_semcode`,
`legacy_lowering.rs` ~line 1296): `opcode_driven_magic` is chosen by a
content-driven `if`/`else if` cascade (`has_v18_qtruth_instr` down to
`has_v1_math_instr`, falling back to `MAGIC0` — i.e. highest matching content
tier wins). Then, unconditionally: `if opcode_driven_header.rev <
SEMCODE_SIGNATURE_MIN_REVISION { MAGIC19 } else { opcode_driven_magic }`.
Because every opcode-driven tier's own rev is below 20 today (`HEADER_V18.rev
= 19`), this comparison always promotes to `MAGIC19` in practice — but the
mechanism itself is a genuine rev-*number* comparison, not per-feature
branching, which is exactly `max(opcode_floor, signature_floor)` written as a
two-way `if`/`else` instead of a literal `.max()` call. `HEADER_V19.capabilities
= HEADER_V18.capabilities` (inherited unchanged) is the established precedent
for a version-identity-only revision bump, which `HEADER_V20` should follow
(no new capability bit, per the original design).

This composition mechanism **already generalizes correctly to a third floor
with zero changes to the SIG0-floor check**: adding a new,
highest-priority `has_v20_ownership_execution_anchor(funcs)` branch (content-
driven — set only when a function's `ownership_events` actually carries a
resolved `StoreVarSite`, mirroring how V1-V18 are each content-gated, *not*
SIG0-style unconditional) would set `opcode_driven_magic = MAGIC20` (rev 21)
for artifacts that need it; the existing `if opcode_driven_header.rev <
SEMCODE_SIGNATURE_MIN_REVISION` check then evaluates false (21 is not < 20)
and correctly keeps `MAGIC20` — proving "ownership-only" and "both" collapse
to the same composed outcome by construction (21 always wins over 20),
exactly as "baseline" and "SIG0-only" already collapse to the same outcome
today (confirmed by a new real test, see below) rather than needing a fourth
distinct case.

**Legacy/new OWN0 grammar gate**: `parse_string_table_debug_and_ownership`
(`crates/sm-format/src/semcode_decode.rs:229`) already receives `header_rev:
u16` as a parameter (confirmed by direct read on this worktree, not assumed
from the stale design draft) — the exact plumbing a `header_rev <
SEMCODE_OWNERSHIP_ANCHOR_MIN_REVISION` (legacy grammar) vs `>=` (anchor
grammar) gate needs already exists, requiring no new parameter threading.
OWN0 *section presence* stays content-sniffed exactly as it is today (looking
for the `"OWN0"` tag, unchanged) — only the *per-event field layout once
found* becomes revision-gated, coexisting with SIG0's own (unrelated,
unconditional) revision-gated presence check without conflict, since both
read from the same single `header_rev` value with no sniffing or fallback
between grammars.

**Naming correction**: rename the design's `SEMCODE_OWNERSHIP_SITE_MIN_REVISION`
to **`SEMCODE_OWNERSHIP_ANCHOR_MIN_REVISION`**, matching this document's own
`ExecutableAnchor` (artifact-side identity) terminology rather than the
superseded "site" framing, which Checkpoint D1 now reserves for
`ActivationSiteId` (compiler-side identity) specifically.

**Code-backed proof, no feature faked**: added
`ssf08_1726_checkpoint_d1_5_signature_floor_promotes_even_the_most_trivial_baseline`
(`legacy_lowering.rs`) — compiles the most trivial possible program
(`fn main() { return; }`) and asserts it still emits `SEMCOD19`, proving
"baseline" and "SIG0-only" are the same real, decoded, observable outcome
today. "Ownership-only" and "both" are not tested (would require implementing
the unauthorized wire feature) — proved instead by the rev-number-comparison
argument above, which needs no new code to hold.

No `sm-format`, `sm-verify`, or `sm-vm` production file was changed for D1.5.
No OWN0 wire, verifier admission, or VM implementation was started.

## Checkpoint D2a: implemented and qualified — rev21 OWN0 wire grammar

Implemented the wire feature D1.5 only proved would compose correctly:
`has_v20_ownership_execution_anchor(funcs)` as the new highest-priority,
content-driven branch in `emit_semcode`'s cascade (true iff any function has
a Borrow event with `activation_site.is_some()`), promoting to
`MAGIC20`/`HEADER_V20` (rev 21). Under rev21+, `emit_ownership_events` writes
`activation_mode: u8` (0 = `ACTIVATION_MODE_FRAME_ENTRY`, 1 =
`ACTIVATION_MODE_STORE_VAR_SITE`) immediately after `kind` for Borrow events
only, plus a `u32` `executable_anchor` when the mode is `StoreVarSite`; Write
events and every field below `activation_mode` are byte-for-byte unchanged
at every revision. `sm-format`'s decoder gates the new field purely on
`header_rev` (no sniffing, no fallback), rejecting an unrecognized mode or a
truncated activation prefix. Legacy (`rev < 21`) OWN0 grammar is untouched.

This unconditionally promotes every existing tuple/record-borrow program to
rev21 (any Borrow event carrying a resolved `ActivationSiteId` now trips the
predicate), which broke 22+ pre-existing tests across four files on
hardcoded byte offsets and header assertions. Accepted as expected per-file,
after re-verifying semantic dry (no drift) for each, not blanket
re-snapshotted — the same discipline as #1724/#1725. One of those fixes
surfaced a real, independent bug: `sequence_index_static_borrow_semcode_bytes()`'s
hardcoded patch offset had silently drifted onto `root_symbol_id`'s low byte
instead of the component-kind byte it claimed to target, once the source
fixture became rev21-shaped — the test would have kept "passing" while
testing something else entirely. Fixed with the correct offset, derived by
walking the grammar rather than re-guessing a second hardcoded number.

Qualification: `cargo test --workspace` green except the 3 known
historical-red `borrow_activation_v20` tests. No VM (`sm-vm`) file changed.

## Checkpoint D2b: implemented and qualified — verifier admission of rev21 StoreVarSite anchors

**Scope.** Semantic admission only: the verifier now proves every rev21
`StoreVarSite(anchor)` Borrow event references a structurally valid,
same-function, canonical `StoreVar` instruction boundary. No VM activation,
no `Frame` state, no #1891 work. `sm-vm` was not touched.

**Canonical-boundary mechanism.** `verify_function_code`
(`crates/sm-verify/src/lib.rs`) already builds `instr_starts: Vec<usize>` in
its one real per-function decode pass — the same set the pre-existing
`#1746` `DebugSymbol.pc` check reuses via `binary_search`. D2b adds a
`store_var_starts: Vec<usize>` accumulator in that identical pass, pushing an
offset only when the just-decoded opcode at that exact canonical boundary is
`Opcode::StoreVar`. This is the deliberate answer to the checkpoint's central
risk: `code[anchor] == StoreVar` alone cannot distinguish a genuine
instruction start from an operand byte that coincidentally holds the same
numeric value. Because `store_var_starts` is a subset of `instr_starts`
built in the same walk, one `store_var_starts.binary_search(&anchor)` proves
both "genuine canonical instruction boundary" and "opcode is StoreVar" at
once, with no second, divergently-maintained decoder.

**Anchor domain.** Function-relative, matching `ExecutableAnchor`'s D1
definition and `sm-vm`'s own `Frame.pc` domain: a byte offset relative to
the owning function's `instr_start`. Same-function enforcement is automatic
by construction, not a separate check — `store_var_starts` is built fresh,
per call, from that one function's own decoded instruction stream; there is
no shared or global table a wrong-function or absolute-artifact offset could
coincidentally match. StoreVar-opcode enforcement is likewise automatic by
construction: `store_var_starts` contains only offsets already proven to
decode as `Opcode::StoreVar` in the same pass.

**Duplicate-anchor policy: reject unconditionally.** Audited fresh (not
reused D1 reasoning) at the actual emission site
(`emit_semcode_function`, `legacy_lowering.rs` ~line 1591): `activation_anchors:
HashMap<ActivationSiteId, ExecutableAnchor>` is populated under
`crate::passes::validate_activation_sites`'s proven per-function bijection
(exactly one `ActivationSiteId` per Borrow event, exactly one per StoreVar
instruction) *and* the emission loop's own insert-once check (fails closed
if any site is claimed twice). Instruction start offsets are strictly
increasing positions in one byte stream, so two distinct StoreVar
instructions can never share a start offset. Conclusion: no frozen producer
can legitimately emit two Borrow events sharing one anchor in one function —
duplicate-anchor rejection defends against a hostile/corrupted artifact
only, never a real compiler output shape. Implemented as an unconditional
check in the new `validate_storevar_site_anchors` helper.

**Item 6 audit (FrameEntry vs StoreVarSite path-family): no heuristic
implemented, by design.** `sequence_access_path_from_expr` is shared,
verbatim, between the frozen Tuple/Record producers and the ADT-match-
scrutinee's own dynamic-fallback prescan — both are capable of producing
bare/short paths, proving path *shape* alone cannot authoritatively
distinguish which producer class emitted a given event. Per the governing
brief's own instruction for exactly this situation, no path-family
enforcement was added; D2b validates only the anchor value on events that
already carry `Some(StoreVarSite(_))`, and never infers producer identity,
or treats a missing/invalid `StoreVarSite` as license to fall back to
FrameEntry.

**Item 5 audit, empirical, not merely absence-of-counterexample.** Probed
whether the frozen producers' own dynamic-root dedup branch (`bind_tuple_items`
/ `bind_record_items` / `bind_let_else_tuple_items`, all guarding
`SequenceOwnershipPath::is_dynamic_fallback()` under `ref` capture) can ever
actually fire for a Borrow event through the currently-admitted frontend.
Result: no. `ref`-capturing a pattern item against a scrutinee whose
sequence index is not a literal (e.g. `pairs[idx]` with a runtime `idx`, as
opposed to `pairs[0]`) is rejected by typecheck itself — uniformly, for the
plain-tuple, let-else-tuple, and record shapes (all three probed directly;
the let-else-record shape shares the same typecheck rule) — before any of
these lowering functions ever run. The dynamic-fallback dedup branch is
therefore defensive/dead code for Borrow events under today's admitted
source surface. Recorded permanently as
`ssf08_1726_checkpoint_d2b_dynamic_index_ref_capture_is_rejected_before_any_dedup_branch_runs`
(`legacy_lowering.rs`).

**Failure-model boundary**, stated in code on `VerificationCode::InvalidOwnershipAnchor`
and restated here so it is never read as stronger than it is: D2b proves the
anchor is a genuine, in-range, correctly-typed `StoreVar` instruction
boundary belonging to the declaring function — an artifact-level structural
fact. It does **not** prove that instruction is the compiler's original
source-level introduction site rather than some other admissible `StoreVar`
a hostile producer chose to point at instead. The current rev21 wire format
carries no additional provenance for that stronger claim; reconstructing
source-level ownership provenance from bytecode alone is out of scope for
this code path and was not faked.

**Tests added.**
*Positive* (`crates/sm-verify/src/lib.rs`, `crates/sm-ir/src/legacy_lowering.rs`,
reusing `tests/borrow_target_identity_proof.rs` where it already covers the
ground): plain Tuple/Record StoreVarSite (pre-existing, re-confirmed under
the new check), mixed StoreVarSite+FrameEntry in one artifact, shadowed
same-spelling bindings resolve to two distinct anchors (verified from the
verifier's own decoded view, not just "did not reject"), a FrameEntry-only
program stays below the rev21 anchor grammar entirely (rev20/legacy
unchanged). Let-else tuple/record and introduction-then-reassignment
(anchor still at the introduction site) are exercised end-to-end — compile →
`verify_semcode_token` → `sm_vm::run_verified_entry_semcode`, at both O0 and
O1 — by the pre-existing `frozen_borrow_targets_do_not_uniquely_identify_store_sites`,
confirmed passing under the new check rather than duplicated.
*Fail-closed* (`crates/sm-verify/src/lib.rs`, all built by corrupting one
anchor field of a real compiled artifact via a byte-level OWN0 walker that
self-verifies its own offset against the real decoder before trusting it):
out-of-range anchor, anchor exactly at the function code end, anchor one
byte into a real StoreVar instruction, anchor at a genuine non-StoreVar
instruction boundary (`Ret`), duplicate anchor across two Borrow events in
one function. Cross-function anchor is not a separate byte-level test —
proved by construction (`store_var_starts`'s per-call, per-function scope;
see above). *Mutation test* (item 12): retargeted a real, working anchor to
an operand byte of a `LoadF64` instruction whose value was deliberately set
to equal `Opcode::StoreVar`'s own opcode byte; confirmed rejection. No
shared state to restore — every fixture is a fresh, local `Vec<u8>`.

**Files changed:** `crates/sm-verify/src/lib.rs` (new
`VerificationCode::InvalidOwnershipAnchor`, `store_var_starts`,
`validate_storevar_site_anchors`, and the test additions above),
`crates/sm-ir/src/legacy_lowering.rs` (the item-5 audit test),
`tests/public_api_contracts.rs` and
`tests/golden_snapshots/public_api/sm_verify_lib.txt` (the new public
`VerificationCode` variant, snapshot diff verified to contain only that one
addition).

**Qualification:** `cargo check --workspace --all-targets` clean;
`cargo test -p sm-verify --features sm-ir/profile-rust` 191 passed (182
baseline + 9 new D2b tests); `cargo test -p sm-format` 40 passed unchanged;
`cargo test -p sm-ir --features profile-rust` 166 passed (165 baseline + 1
new D2b audit test); `cargo test --workspace --no-fail-fast` green except
the same 3 known historical-red `borrow_activation_v20` tests, no other
failure; `git diff --check` clean (pre-existing LF/CRLF warnings only, no
new whitespace errors).

D3 (VM dynamic activation) was not started. #1891, #1718, #1888, `Frame`
pending-borrow state, `next_write_path`, ownership lifetime/release rules,
and the D2a wire grammar were not modified.

## Checkpoint D3: implemented and qualified — VM dynamic Borrow activation

**Scope.** Runtime activation semantics only. `sm-vm/src/semcode_vm.rs`
(`Frame`, `FunctionBytecode`, `push_frame`, the `StoreVar` opcode handler).
`#1891`'s `write_paths`/`next_write_path` mechanism, lexical/NLL release,
partial-Borrow release, and any new lifetime analysis were not touched.

**Representation.** `FunctionBytecode.borrowed_paths` changed from
`Vec<AccessPath>` to `Vec<BorrowedPath>` (`{ path, activation }`), where
`BorrowActivation` is `FrameEntry` or `StoreVarSite(usize)` (frame-relative,
the identical `Frame.pc`/`ExecutableAnchor` domain D1 established and D2b
verified against - never a string-table index, symbol, or ordinal). `Frame`
replaced its single eager `borrowed_paths` field with `active_borrowed_paths`
(paths already active - what `ensure_write_path_allowed` checks writes
against) and a private `pending_site_borrows: Vec<PendingSiteBorrow>`
(`{ anchor, path }`), frame-local, never global/static.

**Initialization (`push_frame`).** Every `BorrowedPath` is sorted once, by
its own explicit mode: `FrameEntry` goes straight into
`active_borrowed_paths` (unchanged eager behavior for every rev20 artifact
and every explicit rev21 `FrameEntry` event - decode already collapses
"no activation field" and "explicit FrameEntry" to the same
`BorrowActivation::FrameEntry` before the VM ever sees a difference);
`StoreVarSite(anchor)` goes into `pending_site_borrows`. Never inferred from
`AccessPath` shape - the mode comes only from the already-verified wire tag.

**Activation point.** Inside the `StoreVar` opcode handler, strictly after
`locals.insert(symbol, val)` (the write's own commit) and after the
pre-existing `next_write_path`/`ensure_write_path_allowed` write-conflict
check (unchanged, still checked against `active_borrowed_paths`): a
`position` lookup by `pending.anchor == pc`, where `pc` is captured at the
very top of the dispatch loop before any operand byte is read - the same
"opcode-byte position of the instruction being executed" `Frame.pc` has
always meant. A match is `remove`d from pending and pushed into
`active_borrowed_paths`; no match is a no-op (covers both "not this
instruction" and "already activated by an earlier visit to this same
anchor," making repeated loop visits idempotent by construction, not by an
added case). `next_write_path` is never read or advanced by this code, and
its own logic never reads Borrow-activation state beyond the pre-existing
conflict check - orthogonal in both directions, per the #1891 boundary.

**Ordering proof re: successful commit.** By construction (code order), not
by a runtime flag. Whether a distinguishing test exists (StoreVar failing
after being dispatched but before its write commits) was checked directly:
`get_reg`'s only failure modes (uninitialized/out-of-range register) are
already excluded by verification's register-liveness dataflow pass for any
admitted artifact; the `next_write_path` conflict check structurally never
applies to an introduction (it only fires when `locals.contains_key`, i.e.
the symbol is already bound - true only for a *re*assignment, never a first
introduction, since introductions are exactly the StoreVars D1 gives an
`ActivationSiteId`); `locals.insert` is infallible. Empirically confirmed,
not just argued: temporarily moving the activation block to *before* the
commit line and re-running the full `sm-vm` suite (118/118) and
`borrow_activation_v20.rs` (8/9, the same pre-existing unrelated failure)
produced **zero observable difference** - no admitted program in the suite
can even exercise the ordering distinction, because nothing can fail
between decode and commit for an introduction. Documented here per the
brief's own escape hatch rather than inventing a failure path that doesn't
exist.

**Reassignment (item 5/12.E).** `only_introduction_pc_activates_not_reassignment`
(`tests/borrow_activation_v20.rs`): introduction, then reassignment of the
same bound name, then a write to the borrowed root - rejects, proving the
borrow activated at the introduction and reassignment did nothing further
(no crash, no double-activation, no change in outcome).

**Branches (item 6/12.C-D).** Pre-existing `branch_execution_controls_borrow_activation`
now passes both arms: untaken → pending, later write succeeds; taken →
active, later write rejects. New `post_activation_write_conflicts` isolates
the taken-equivalent case without the branch.

**Loops (item 7/12.F).** `loop_activates_on_first_executed_visit_not_merely_because_the_anchor_exists`:
an anchor skipped on iteration 0 and executed on iteration 1 activates on
that later visit, not before. `loop_repeated_anchor_visits_are_harmless`:
the same static anchor executing on every iteration activates once, then is
a no-op on every later visit (never re-added, never an error).

**Mixed modes (item 8/12.G) and rev20 (item 9/12.H).**
`mixed_frame_entry_and_store_var_site_activation_in_one_function`: an ADT
`FrameEntry` event and a Tuple `StoreVarSite` event in one rev21 function
each keep their own mode (lazy tuple write-before-borrow still succeeds; the
eager ADT borrow still rejects a write to its root regardless of order).
`rev20_legacy_borrow_stays_eager_regardless_of_which_arm_ran`: an ADT-only
program (never promoted past the anchor floor - asserted directly on the
magic bytes) keeps exactly the pre-D3 eager behavior.

**Historical-red transition (item 13), fully closed.** Ran
`tests/borrow_activation_v20.rs` unchanged first, as instructed. Before D3:
3 known red. After D3's runtime implementation: 2 green
(`write_before_borrow_succeeds`, `branch_execution_controls_borrow_activation`),
1 (`optimizer_preserves_borrow_target_store`) still red for a reason
unrelated to VM runtime activation - investigated and STOPped on rather
than silently accepted or silently rewritten; see "The stale third test,
reconciled" below. After the authorized test-only fix: all 3 historical
tests, and the full workspace, are green.

**The stale third test, reconciled (D3 replay, test-only fix).**
`optimizer_preserves_borrow_target_store` failed at
`assert_eq!(o0[0].ownership_events.len(), 1); assert_eq!(o0[0].ownership_events, o1[0].ownership_events);`
(expected O1's event list to equal O0's, i.e. also 1 event) - but its own
source (`let pair = (1,2); return; let (ref left,_) = pair;`) is the
*exact* program `ssf08_1726_checkpoint_c_unreachable_borrow_introduction_removed_coherently`
(`crates/sm-ir/src/legacy_lowering.rs`, part of the already-accepted,
already-qualified Checkpoint C work) uses to assert the opposite:
`o1_main.ownership_events.is_empty()`, i.e. the unreachable Borrow
introduction and its paired event must be coherently removed together at
O1, not preserved. Confirmed directly (not assumed): running this exact
source through both checkpoints' assertions showed O1 genuinely produces
zero events - coherent removal, not an orphaned event with a dangling
target (the failure mode the old test's own comment described and guarded
against, just on the wrong side of the contract). The old assertion was
simply stale: written before Checkpoint C's coherent-removal design was
accepted, never reconciled afterward.

Renamed to `optimizer_removes_unreachable_borrow_and_anchor_coherently` and
rewritten (`tests/borrow_activation_v20.rs`) to prove the accepted
Checkpoint C invariant instead of the stale one, on the *same, unmodified*
counterexample source: O0 (no cleanup) may still carry the dead,
annotated introduction and its Borrow event; O1 must have removed the
annotated StoreVar, its paired Borrow event, and leave no orphan
`activation_site` on either side; both optimization levels must still
compile, verify, and run correctly end to end. Production code was not
touched for this fix - only the test's own name and assertions changed, on
the same source program, so it remains a real regression against the
original counterexample rather than a weakened one.

**Mutation results (item 14).** Mutation 1 (restore all-eager-at-push_frame):
killed - `write_before_borrow_succeeds`,
`branch_execution_controls_borrow_activation`, and
`mixed_frame_entry_and_store_var_site_activation_in_one_function` all fail.
Mutation 2 (activate by root symbol instead of exact anchor PC): killed -
the same three tests fail; `only_introduction_pc_activates_not_reassignment`
does *not* catch it (analyzed and confirmed: in this language, a name's
reassignment can never execute before its own introduction, so
symbol-keyed and PC-keyed matching coincide for that specific scenario -
the mutation is still caught by the other three, so the suite as a whole
detects it; the introduction/reassignment test alone does not, and that
limitation is recorded here rather than silently claimed otherwise).
Mutation 3 (activate before commit): killed nothing, by design - see the
ordering-proof paragraph above; documented per the brief's explicit
allowance rather than fabricated. All three mutations were reverted; the
tree matches this section's description of the shipped code.

**Files changed:** `crates/sm-vm/src/semcode_vm.rs` (`BorrowActivation`,
`BorrowedPath`, `PendingSiteBorrow`, `Frame`/`FunctionBytecode` field
changes, `push_frame`, the `StoreVar` opcode handler, and 4 pre-existing
push/exit tests updated to assert pending-vs-active state instead of
uniform eager activation - a required update given D3 changes what those
tests' own real, unmodified `compile_program_to_semcode` fixtures actually
produce, not a rewrite of intent); `tests/borrow_activation_v20.rs` (7 new
D3 tests, the pre-existing `write_before_borrow_succeeds` and
`branch_execution_controls_borrow_activation` untouched, plus the one
authorized test-only reconciliation renaming
`optimizer_preserves_borrow_target_store` to
`optimizer_removes_unreachable_borrow_and_anchor_coherently`);
`tests/golden_snapshots/public_api/sm_vm_semcode_vm.txt` (public API
snapshot, diff verified to contain only the intended new/renamed items).

**Qualification (final replay, after the stale-test fix):**
`cargo check --workspace --all-targets` clean; `cargo test --test
borrow_activation_v20` 9/9 - all green, including the 3 original historical
tests; `cargo test -p sm-vm` 118/118; `cargo test -p sm-verify --features
sm-ir/profile-rust` 191/191 unchanged; `cargo test -p sm-format` 40/40
unchanged; `cargo test -p sm-ir` 166/166 unchanged; `cargo test --workspace
--no-fail-fast` fully green, zero failures anywhere; `git diff --check`
clean (pre-existing LF/CRLF warnings only).

**Checkpoint D3 = PASS. #1726's implementation is complete** (Checkpoints
A through D3, including the D2b.5 reconciliation, all PASS; the one stale
test contradiction is resolved). `#1891` remains completely untouched and
is the sole remaining merge blocker - #1726 does not merge until #1891 is
independently closed. No new #1726 architecture work should follow this
point; the next work item is #1891 itself.
