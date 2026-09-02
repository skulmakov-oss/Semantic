# SSF-08 Ownership And Memory Position Decision

Status: SSF-08 architecture decision, accepted
Decision: **Position A — bounded deterministic VM language**
Evidence baseline: `5ddb703b19f6135aaa96e2cc8f1c7d4e8e6baafc`
Issue: #1579 (SSF-08), umbrella #1569, module findings under #1617

This record is an architecture-decision document only. It authorizes no
ownership-implementation repair. It does not close #1579 — SSF-08 remains
active until the repair dependency order below is executed and qualified.

## Executive Decision

Semantic selects **Position A**: ownership protects admitted value paths and
runtime invariants inside a bounded, frame-local, deterministic VM. Semantic
does **not** claim Rust-equivalent lifetime inference, region inference,
general borrow checking, unrestricted alias analysis, or systems-language
memory-safety equivalence.

The falsification attempt required by this task (Section 2 of the governing
prompt) was carried out against current `main` at the evidence baseline
above, across the frontend (`sm-front`), IR/transport (`sm-ir`, `sm-format`),
verifier (`sm-verify`), VM (`sm-vm`), frame/host boundary (`sm-vm`,
`prom-abi`), and resource/quota boundary (`sm-runtime-core`, `sm-vm`).
**Position A survives falsification.** No layer implements, even partially,
a general borrow checker, lifetime/region system, alias inference, partial
release, inter-frame borrow persistence, or heap-aliasing model. The evidence
below is organized so the falsification reasoning is checkable line by line.

One structural fact discovered during this audit is decisive and is stated up
front because it reframes the whole question: **`sm-vm`'s `Value` type has no
shared/aliased representation at all.** `get_reg` (`crates/sm-vm/src/semcode_vm.rs:2917-2930`)
always returns `v.clone()` — a full deep copy. There is no `Rc`/`Arc`/pointer
type anywhere in `crates/sm-vm/` (confirmed by direct search). Semantic's
"ownership" machinery is therefore not, and structurally cannot currently be,
a mechanism that prevents use-after-free or shared-mutable-state data races
the way Rust's borrow checker does — there is no shared mutable memory for it
to protect. It is a **value-domain discipline** (a symbolic path-conflict
convention layered on top of copy-value semantics) that catches a bounded
class of move/borrow *logic* errors deterministically. This is the single
strongest piece of evidence for Position A: the "expanded systems ownership
model" of Position B presupposes exactly the kind of aliasable,
reference-holding memory model that does not exist in this VM today, and
building one is a change of much larger scope than "qualify more paths."

## What Semantic Claims

> Semantic provides a bounded deterministic ownership/path-state model for
> explicitly admitted value paths across its verified VM pipeline
> (source → frontend → IR/SemCode → verifier → VM). Borrow and write events
> are transported through the versioned `OWN0` SemCode section, structurally
> admitted by the verifier before execution, and enforced by the VM as a
> frame-local, deterministic overlap rule over concrete (not indirect,
> not dynamically aliased) value paths: a write is rejected if its path
> exactly equals, is a parent of, or is a child of an active borrowed path in
> the current call frame; sibling paths are always permitted. The active
> borrow set is scoped to the current call frame's lifetime.

## What Semantic Explicitly Does Not Claim

- Rust-equivalent lifetime inference or an explicit lifetime-variable system.
- Region inference or a region graph.
- General borrow checking over arbitrary/indirect/dynamically-aliased paths.
- Unrestricted alias analysis. The one alias-adjacent computation in the
  entire runtime is a fixed structural prefix comparison
  (`access_paths_overlap`, `crates/sm-vm/src/semcode_vm.rs:2881-2887`) — not
  an inference procedure, and there is nothing else to infer over because
  `Value` is never aliased in the first place.
- Systems-language memory-safety equivalence. Semantic has no heap-aliasing
  model, no `unsafe`/escape analysis, and (see Executive Decision) no shared
  mutable memory for an ownership system to protect in the traditional sense.
- Partial borrow release before frame exit, inter-frame borrow persistence,
  or a general alias/region reasoning system.
- Concurrency-aware ownership of any kind (the VM is a single-threaded
  sequential interpreter; no concurrency primitives exist in `sm-vm`).

## Evidence Method

Direct inspection of `crates/sm-front`, `crates/sm-ir`, `crates/sm-format`,
`crates/sm-verify`, `crates/sm-vm`, `crates/sm-runtime-core`, and `prom-abi`
at commit `5ddb703b19f6135aaa96e2cc8f1c7d4e8e6baafc`, cross-referenced
against `docs/spec/runtime_ownership.md`, `docs/spec/semcode.md`,
`docs/spec/verifier.md`, and the 24 GitHub issues named in the governing
task (`#1656`-`#1664`, `#1709`, `#1718`, `#1724`-`#1726`, `#1759`-`#1763`,
`#1770`, `#1771`, `#1773`, `#1775`, `#1778`), plus the repository's own
end-to-end ownership test suite (`tests/runtime_ownership_e2e.rs`,
`tests/tuple_ownership_golden.rs`, `tests/record_field_ownership_golden.rs`,
`tests/sequence_ownership_golden.rs`, `tests/pcc6_option_result_ownership_golden.rs`,
and the ADT-payload tests inline in `crates/sm-vm/src/semcode_vm.rs`). Every
factual claim below carries a file:line citation checked against this exact
commit; no claim is taken from an issue's own description without
independent code verification.

## Current Ownership Pipeline

```text
source
  -> sm-front: ScopeEnv (compile-time move/borrow discipline, advisory-strength)
  -> sm-ir: AccessPath / PathComponent / OwnershipPathEvent (Borrow, Write)
  -> SemCode OWN0 section (SEMCOD11 tuple-only, SEMCOD12 +record-field)
  -> sm-verify: structural admission only (shape/count/capability, no semantics)
  -> sm-vm: Frame-local borrowed_paths / write_paths, BorrowWriteConflict trap
  -> frame exit: borrowed set discarded with the popped Frame (not explicitly cleared)
```

This matches the architecture `docs/spec/runtime_ownership.md` already
documents; the sections below record exactly where implementation matches,
exceeds, or falls short of that documented pipeline.

## Frontend State Model

**Representation** (`crates/sm-front/src/lib.rs`): `ScopeEnv` (line 193-195)
is a `Vec<BTreeMap<SymbolId, ScopeBinding>>`. `ScopeBinding` (line 180-189)
carries `consumed: bool` (whole-variable moved-out flag) and
`path_state: Vec<(PatternPath, PathAvailability)>` (per-path partial
move/borrow state). `CaptureMode::{Move, Borrow}` (`crates/sm-front/src/types.rs:229-233`)
carries its own doc comment stating plainly: *"Mutable borrow, partial move,
lifetime inference, and reborrow are deferred."* — this is the implementers'
own contemporaneous statement of Position A's boundary, written into the type
itself, independent of this decision record.

**IMPLEMENTED**: path-level move/borrow conflict detection for tuple-index
and record-field patterns via `mark_path_state` / `check_path_available` /
`check_capture_allowed` (`lib.rs:278-439`), with prefix-based overlap
matching the same semantics the VM later enforces.

**BROKEN-BUT-OWNED-BY-SSF-08** — the `#1656`-`#1664` cluster, verified
against this exact commit (all nine reproduce as described, see the
frontend-inventory evidence table below): `if`/`loop`/`match`/`if-let`, in
both statement and expression form, clone `ScopeEnv` for each branch body and
either never join the result back (`if`, `loop` — no join exists at all,
enforced structurally in expression form because `infer_expr_type` takes
`&ScopeEnv`, not `&mut`) or join back only the scrutinee's own pattern-plan
state and nothing else a branch body touched (`match`, `if-let` — a real but
narrow join, documented in the code's own comments at `typecheck.rs:12119-12121`
and `typecheck.rs:2446-2447`). `ScopeEnv`'s seven ownership-query APIs
(`mark_consumed`, `is_consumed`, `mark_path_state`, `check_path_available`,
`check_capture_allowed`, `is_const`, `is_mutable`) all fail open on an
unknown `SymbolId` — a lookup miss reads as "available"/"allowed" rather than
erroring (`lib.rs:263-461`), exactly as `#1664` states.

**Scalar / direct-root copy semantics — empirically resolved, not a blocking
finding.** Whole-variable move tracking (`consumed`/`mark_consumed`) has
**zero production call sites** — its only caller anywhere in the crate is a
unit test (`typecheck.rs:10978`). An ordinary `let y = x;` (`Stmt::Let`,
`typecheck.rs:804-866`) never marks `x` consumed; only pattern-driven
destructuring (`bind_tuple_items`/`bind_record_items`) populates
`path_state`. An earlier revision of this record filed that as a Known
Blocking Finding without first determining whether Semantic's actual
contract for plain root rebinding is copy or move. It is **copy**,
confirmed by direct evidence, not inferred from the VM alone:

- `crates/sm-front/src/typecheck.rs:10754-10768` — a test named
  `use_after_move_rejects` (misleadingly, given its actual assertion)
  compiles `fn take_val() -> i32 { return 5; } fn main() { let x: i32 =
  take_val(); let _ = x; let _ = x; return; }` and asserts it **typechecks
  successfully**, with the code's own comment stating the design intent
  directly: *"i32 is Copy — use-after-move semantics only apply to non-Copy
  types. This test just validates the checker doesn't false-positive on
  i32."* No `is_copy`/`Copy`-classification function exists anywhere in
  `sm-front` (confirmed by search) — this comment is the language's only
  explicit statement of the principle, not a mechanically enforced
  Copy/non-Copy split.
- **Full-pipeline empirical reproduction**, run against this exact commit
  via `cargo run -p semantic_language --bin smc -- run <file>` (compile →
  typecheck → IR → SemCode → verify → VM execute, not typecheck alone):
  `let a: i32 = 7; let b = a; assert(a == 7); assert(b == 7);` and the
  analogous pattern for `bool`, `quad`, `f64`, and `text` all typecheck and
  execute to completion (exit code 0, every `assert` passes) — both the
  original and the rebound name remain independently usable and correct
  afterward. Representative scalar families confirmed:
  `i32`, `bool`, `quad`, `f64`, `text`.
- **The same is true for a non-Copy aggregate via *plain* rebinding**:
  `record Point { x: i32, y: i32 } fn main() { let p: Point = Point { x: 1,
  y: 2 }; let q = p; assert(p.x == 1); assert(q.x == 1); return; }` also
  executes to completion, `p` still usable after `let q = p;`. This shows
  the boundary is not "Copy types get copy semantics, non-Copy types get
  move semantics" (no such split is implemented) — it is "a **plain**
  root-to-root `let` rebinding is copy-by-value for every type today;
  **pattern-based destructuring capture** (tuple/record/ADT patterns, match
  arms) is the only mechanism that tracks move/borrow, via `path_state`,
  regardless of what is being destructured." A durable regression proving
  this end-to-end is added at
  `tests/scalar_root_rebind_copy_semantics_e2e.rs` (this PR).

This is consistent with, not a gap in, `runtime_ownership.md`'s own frozen
scope: the supported slice is defined as **tuple `AccessPath`** and **direct
record field `AccessPath`** — i.e. paths with at least one component. A
plain root rebind (`let y = x;` with no destructuring) produces an
`AccessPath` with zero components, which was never inside the promised
tracked surface to begin with. `ScopeBinding.consumed`/`mark_consumed` is
therefore accurately classified as **inert scaffolding for a whole-value
move-tracking feature the current admitted contract does not promise**, not
a defect blocking any invariant this decision record freezes. It is left
in place (not removed) since removing dead code is out of scope for a
docs/positioning PR; a future cleanup may remove or repurpose it, but that
is not an SSF-08 ownership-position blocker. Given the "no shared aliasing
exists" finding above, this also carries no safety implication: the VM's
copy-value semantics make a hypothetical missed whole-value move merely
redundant, never unsafe.

**Common-root hypothesis** (per the governing task's Section 5): confirmed,
with a precise refinement. It is not one function with one bug; the same
"clone env, mutate, discard-or-narrowly-join" idiom recurs at every branch
construct (`if`, loop, match, if-let) × both surface forms (statement,
expression), in two mechanically distinct manifestations:
- **Manifestation A — no join at all**: `if`/`loop`, both forms.
- **Manifestation B — a join exists but is scrutinee-only and `Expr::Var`-only**:
  `match`/`if-let`. `validate_plan_against_scrutinee_state` and
  `apply_plans_to_scrutinee` (`typecheck.rs:15420-15433`, `15523-15541`)
  both guard with `let Expr::Var(name) = ... else { return; }`, silently
  no-op-ing for a projected scrutinee (`#1663`) even though
  `expr_access_path` (`typecheck.rs:15453-15476`) already knows how to
  resolve `RecordField`/literal-`SequenceIndex` projections to a path.

The correct first implementation slice is therefore **one canonical
ownership-state transition/join model** (a real `&mut ScopeEnv` threading
discipline through branch/loop/match/if-let checking, including
expression-form control flow, which currently cannot join by construction),
closing all nine findings through that shared root — not nine independent
patches. This confirms the governing task's own working hypothesis
(Section 17): do not split `#1656`-`#1664` into nine implementation
branches.

## IR / OWN0 Transport

**Representation** (`crates/sm-ir/src/legacy_lowering.rs`): `AccessPath`
(line 367-369, `root: SymbolId, components: Vec<PathComponent>`).
`PathComponent` (line 435-441) already has **four** variants in code today —
`TupleIndex(u16)`, `Field(SymbolId)`, `AdtPayload{variant, index}`,
`SequenceIndexStatic(u32)` — not the two the frozen spec (`runtime_ownership.md`)
documents. `OwnershipPathEvent{kind: Borrow|Write, path}` (line 443-453).
Encode order is provably insertion order (`emit_ownership_events`,
line 2255-2314, iterates a plain `Vec`); order loss happens downstream, at
decode (see `#1726` below).

**A genuinely more-implemented surface than the frozen spec states, verified
end-to-end, not merely "vocabulary exists":**
- **Sequence static index**: `tests/sequence_ownership_golden.rs` is a real
  compile → decode → verify → VM-run positive test, and
  `tests/runtime_ownership_e2e.rs` has a full negative suite for it
  (`runtime_ownership_sequence_same_index_conflict_rejects`,
  `_sibling_index_write_passes`, `_parent_child_conflict_rejects`,
  `_child_parent_conflict_rejects`, plus dynamic-index-vs-static-index
  interaction cases and inner-frame-cleanup — lines 140-343 of that file).
  This is transported under plain `SEMCOD11`/`CAP_OWNERSHIP_PATHS` with no
  distinct capability bit (`crates/sm-format/src/local_format.rs:34-35` only
  defines `CAP_OWNERSHIP_PATHS`/`CAP_OWNERSHIP_FIELD_PATHS`).
- **ADT payload** (fixed variant, single level): a real positive E2E test
  (`vm_runs_adt_payload_ownership_positive_e2e_path`,
  `crates/sm-vm/src/semcode_vm.rs:5310-5358`, real enum/match/`ref` source,
  compiled, decoded, run) and a runtime-patched negative conflict test
  proving `BorrowWriteConflict` fires correctly. The test file's own comment
  (`semcode_vm.rs:5360-5363`) explains a *genuine* negative E2E test (through
  real compiled source, not patched bytes) is currently blocked by an
  unrelated frontend gap — the language does not yet support mutable
  reassignment of an ADT payload — so this specific case is not proof-complete
  by the strongest standard used for tuple/record/sequence.
- The verifier's own admission for `AdtPayload` is explicitly weaker than for
  `Field`/`TupleIndex`: its variant `SymbolId` "cannot be bounds-checked
  against the local string table. Structural acceptance only."
  (`crates/sm-verify/src/lib.rs:1034-1037`, mirrored in
  `crates/sm-format/src/semcode_decode.rs:35`).

Per the governing task's Section 14 instruction to be conservative and not
promote merely because vocabulary exists: this evidence clears a materially
higher bar than "vocabulary exists" for Sequence static index (full
positive+negative E2E, real compiler pipeline throughout) and a real but
qualification-incomplete bar for ADT payload (positive E2E proven, negative
proof not yet reachable through the real compiler, verifier admission
structurally weaker). Neither is promoted into the frozen contract by this
PR (see Included/Deferred Ownership Surface below); both are classified
**INCLUDED ONLY AFTER REQUIRED REPAIR**, precisely because of the specific,
named gaps below (`#1709`, `#1718`, `#1724`, `#1725`, `#1726`), not because
the surface is unproven in principle.

**`#1709`** (nested/value lowering can erase ownership events) — **confirmed,
and it is a bug inside the tuple/record-only contract, not evidence for or
against widening.** `lower_value_block_expr` and its `Stmt::LetTuple`/
`Stmt::LetRecord` arms call `bind_tuple_items`/`bind_record_items` with a
throwaway `&mut Vec::new()` (`legacy_lowering.rs:7015-7025`, `7046-7059`) —
real tuple-index Borrow events are generated and then discarded because
nothing reads the temporary buffer. `lower_expr_with_expected`
(`legacy_lowering.rs:2730-2743`), which every nested expression in this path
routes through, has **no `ownership_events` parameter at all** — it is
structurally incapable of propagating events regardless of what happens
inside it. `lower_loop_expr_stmt`'s fallback arm builds a temporary
`LoweringCtx` with a fresh empty event vec, delegates to the real
event-recording `lower_stmt`, then copies back every field except
`ownership_events` (`legacy_lowering.rs:8613-8644`). **This means a program
using `if`/`loop` in expression position, containing a `let (ref a, b) = t;`
inside, can silently lose the resulting Borrow event — the VM would then
fail to reject a conflicting write it should reject.** This is a real gap
inside the currently-frozen tuple/record contract, independent of any
ADT/Sequence scope question, and is the most safety-relevant finding of this
audit (see Known Blocking Findings).

**`#1718`** (ADT/Sequence path components transported under a tuple-only
contract) — **confirmed exactly, with the precise mechanism.**
`has_v12_record_field_ownership_events` (`legacy_lowering.rs:2243-2253`)
tests only for `PathComponent::Field`; a function whose events are entirely
`AdtPayload`/`SequenceIndexStatic` is emitted under plain `SEMCOD11`. The
decoder parses both component kinds with no header/version gate
(`crates/sm-format/src/semcode_decode.rs:395-417`). The verifier admits them
structurally with an explicit "structural acceptance only" comment and no
corresponding capability flag (`sm-verify/src/lib.rs:1026-1044`). The VM
converts and enforces them identically to tuple/record paths
(`semcode_vm.rs:1101-1139`). Nothing "drops" this vocabulary — it is live,
end to end, today, without the version/capability contract the header
scheme's own design otherwise requires for a promoted surface. **This does
contradict a strict reading of "SEMCOD11 is tuple-only"** — it does not
contradict Position A, since the semantics involved (fixed, statically-known
component index/variant, same frame-local overlap rule) are the same bounded
kind of guarantee, just under-declared at the header/capability layer.

**`#1725`** (frontend `SymbolId` vs. VM OWN0 root identity mismatch) —
**confirmed as a genuine, previously-underspecified identity bug, not a
component-kind-scope question.** The frontend's `SymbolId` is a globally
(program-wide) first-seen-order id from `ton618_core::SigTable::intern`
(`crates/ton618-core/src/sigtable.rs:15-27`). `emit_ownership_events` writes
this raw global id directly to the wire (`legacy_lowering.rs:2283`,
`write_u32_le(out, event.path.root.0)`). But `emit_semcode_function` also
builds a completely separate, **function-local** string interner for
`LoadVar`/`StoreVar` variable names (`legacy_lowering.rs:2347-2364`,
`1381-1411`), and at VM load time `remap_paths`
(`crates/sm-vm/src/semcode_vm.rs:1101-1139`) does:
```rust
let local_root = path.root_symbol_id as usize;
let root = symbol_ids.get(local_root).copied().unwrap_or(SymbolId(path.root_symbol_id));
```
— i.e. it treats the wire's `root_symbol_id` (the raw *global* frontend
`SymbolId`) as an **index into the function-local string table**. These are
two independently-numbered spaces. When the numbering happens to coincide
(the small, single-purpose functions exercised by every current golden test),
the path resolves correctly by construction of the test fixtures, not by a
verified identity contract. When it does not coincide, `.unwrap_or(...)`
silently substitutes the raw global id as if it were already a valid local
`SymbolId`, rather than erroring — masking rather than surfacing the
mismatch. **This is the single highest-priority repair finding in this
audit**: it means the correctness of the *entire* tuple/record ownership
guarantee — not just the ADT/Sequence extension — is unproven in the general
case; only proven for the specific numeric coincidences the current golden
fixtures happen to exercise. See Known Blocking Findings and the repair
order below.

A related, independently-filed identity concern, `#1724` (FA-04-018,
confirmed open, Phase-A-evidence-only per its own text): lexical scopes are
flattened into one textual local namespace at IR lowering
(`IrInstr::StoreVar{name: String}`/`LoadVar{name: String}`), so a shadowed
binding (`let x = ...` inside a branch/arm/loop shadowing an outer `x`) can
leak the inner runtime value to a `LoadVar "x"` that should resolve to the
outer binding after the nested scope exits. This is not itself an ownership
event-loss bug, but it shares the same underlying category as `#1725` —
binding/root identity is not preserved uniformly across the
frontend-to-VM boundary — and reinforces why "OWN0 identity correctness"
needs to be treated as one foundational repair rather than assumed sound.

**`#1726`** (OWN0 event timing/interleaving lost; VM activates borrows too
early) — **confirmed exactly.** Encode preserves true temporal order (a
single `Vec`, iterated in order). Decode
(`crates/sm-format/src/semcode_decode.rs:395-435`) reads events in wire order
but immediately fans each one into one of two separate output vectors by
kind (`OWNERSHIP_EVENT_KIND_BORROW => borrowed_paths.push(...)`,
`OWNERSHIP_EVENT_KIND_WRITE => write_paths.push(...)`) — the merged
cross-kind interleaving is unrecoverable from the decoded structure. VM
`push_frame` (`semcode_vm.rs:2845-2879`) installs the callee's **entire**
static `borrowed_paths` list at frame entry (`f.borrowed_paths.clone()`,
line 2872) — every borrow is active from the first instruction of the
callee, not from the true program-order point where its `Borrow` event
occurred. Only `write_paths` has a positional cursor
(`Frame.next_write_path`); `borrowed_paths` has none. **Directional
consequence**: because every borrow activates too *early* rather than too
*late*, this is a soundness-safe, usability-costly direction — it can only
cause the VM to over-reject (treat a write as conflicting before the borrow
"really" started in program order), never to under-reject a write that
should have been caught. It does not, by itself, create a false-negative
(unsound) gap the way `#1709`'s event loss does.

**Per-issue contradiction assessment** (governing task Section 6): `#1709`
is a bug inside the frozen tuple/record contract (independent of
ADT/Sequence). `#1718` is a real, present-tense widening beyond the header's
declared tuple-only scope — it needs either a capability promotion (to
formally include Sequence/ADT) or a fail-closed rejection (to actually
restrict to tuple/record), not silent continuation. `#1725` and `#1726` are
both **orthogonal to component-kind scope** — they affect `AccessPath.root`
and event ordering for every path kind, including the strictly tuple/record
reading of the spec, and would reproduce identically in a hypothetical build
with Sequence/ADT support entirely removed. None of the four falsifies
Position A; all four are gaps *inside* a bounded model, not evidence of an
unbounded one.

## Verifier Admission

**Confirmed: purely structural.** `sm-verify` never compares one path
against another (`grep` for `overlap`/`conflict`/`alias` in
`crates/sm-verify/src/lib.rs` returns only two unrelated hits about the
verifier's own analysis-pass memory budgeting). Its one ownership-content
loop (`lib.rs:1026-1044`) exists solely to set a capability tally flag
(`has_record_field_ownership`), not to reason about path meaning.

- **Path overlap**: not evaluated by the verifier (computed only at VM
  runtime, `access_paths_overlap`, `semcode_vm.rs:2881-2887`).
- **Lifetime**: not evaluated (frame-scoped lifetime is a VM runtime concept —
  `Frame.borrowed_paths` populated on push, discarded on pop).
- **Release**: not evaluated, and there is no wire representation for it —
  the OWN0 event-kind space is exhaustively `{Borrow=0, Write=1}`
  (`crates/sm-format/src/local_format.rs:52-53`); no release/end-borrow/drop
  event kind exists at all.
- **Inter-frame aliasing**: not evaluated — the verifier's only cross-function
  pass is `CALL` arity/signature matching, unrelated to ownership state; path
  content is carried opaquely through a private field the verifier itself
  never reads.
- **Structural-only confirmation**: `VerifiedFunction`, the type sm-verify
  actually returns as its verified result (`lib.rs:204-209`), carries **no
  ownership-path fields at all** — path content survives only in an opaque
  pass-through field for the VM to consume later. The decoder's own comment
  (`semcode_decode.rs:35`) states an `AdtPayload` variant id "cannot be
  bounds-checked against the local string table. Structural acceptance
  only" — an explicit acknowledgment that semantic validity is out of the
  verifier's scope.
- **Concrete pre-execution rejection surface** (all real, tested): missing
  OWN0 under a header that requires it; invalid/unsupported event-kind byte;
  invalid/unsupported path-component-kind byte; truncated path payloads
  (count/kind/root/component fields cut short); record-field/ADT/Sequence
  payload present without the matching capability bit; header advertises a
  capability with no corresponding section. All are byte-shape, tag, count,
  or capability-bit checks — none compare two paths to each other.
- **Left entirely to the VM**: overlap, lifetime/scope, release, inter-frame
  aliasing, and symbol/root resolution (the verifier never resolves a
  `root_symbol_id` to anything; only the VM's `remap_paths` does, which is
  exactly where `#1725`'s bug lives).

This is not "static borrow checking" under any accurate description; the
verifier's OWN0 role is shape/version/capability admission, precisely as
`runtime_ownership.md` already claims.

## VM Enforcement

**Confirmed, with one refinement not previously documented.** `Frame`
(`semcode_vm.rs:76-85`) carries `borrowed_paths: Vec<AccessPath>` (installed
wholesale from the callee's static list on `push_frame`, line 2872) and
`next_write_path: usize` (a cursor into the function's static `write_paths`
list, reset to 0 per frame, line 2873). `push_frame` never merges anything
from the caller's frame into the callee's — inter-frame borrow persistence is
not merely undocumented, it is structurally impossible given how the callee
frame is constructed (confirmed by direct inspection, not absence of a
keyword). Frame exit clears the active set implicitly, by the whole `Frame`
struct (and its `borrowed_paths` field) being dropped when `Vec::pop`
removes it from the call stack (`Opcode::Ret`, line 2666-2693) — there is no
explicit `.clear()` call anywhere in the file; the guarantee holds by Rust's
own ordinary drop semantics, not by a dedicated ownership-release routine.

`BorrowWriteConflict` has exactly one call site
(`ensure_write_path_allowed`, called from the `Opcode::StoreVar` handler,
line 2377). **Refinement**: the check only fires on a `StoreVar` whose target
symbol matches the *next expected entry* in the function's static
`write_paths` sequence (`frame.next_write_path`, lines 2363-2378) — i.e. the
mechanism is a compile-time-known, cursor-advanced sequence match, not a
general "check every write against every active borrow" scan. A `StoreVar`
to a local that is not the next expected write-path entry (e.g. a first
assignment, or a local outside the declared write-path sequence) is not
checked by this mechanism at all. This is architecturally consistent with a
bounded, statically-planned enforcement model (further evidence for Position
A, since it is explicitly not a dynamic, general-purpose scanner) but is
worth stating precisely rather than describing as "the VM checks every
write."

`access_paths_overlap` (`semcode_vm.rs:2881-2887`) is one fixed function:
same root, then component-prefix comparison up to the shorter length. This
single rule implements exact-equality, parent/child (both directions), and
(by returning `false`) sibling-allowed, uniformly, with no separate branch
per case.

**Confirmed absent** (targeted mechanism search, not keyword-absence
inference): partial release; lifetime variables / explicit lifetime
representation; region graph or region inference; general alias inference
beyond the one fixed overlap rule (and, per the Executive Decision, there is
no aliased memory to infer over regardless); inter-frame borrow persistence
(contradicted directly by `push_frame`'s per-callee-only construction);
general heap ownership tracking (no `Rc`/`Arc`/heap-cell types anywhere in
the crate); unsafe/escape analysis (no `unsafe` blocks, no escape-analysis
code); concurrency-aware ownership (the VM is single-threaded; its one
`thread_local!` is a `#[cfg(test)]` scratch cell for capturing terminal
output in tests, unrelated to ownership).

## Frame / Call Boundary

`push_frame` (`semcode_vm.rs:2845-2879`) is the frame-entry point;
`validate_call_arguments` (line 2817-2842) now enforces declared-signature
arity/type at frame entry for every argument, fused into `push_frame` itself
(not just `Opcode::Call`) specifically so `ClosureCall` and host/verified-entry
routes cannot bypass it (`#1773`, confirmed already repaired — see below).
`Value` (`semcode_vm.rs:46-61`) carries no borrow/ownership tag; ownership
bookkeeping lives entirely outside `Value`, in the frame-local `AccessPath`
lists keyed by `SymbolId`. Register storage now distinguishes
"never written" from "written" via `RegisterSlot::{Uninitialized, Value}`
(`#1770`, confirmed already repaired).

## Host Boundary

Host-ABI conversion is `value_to_abi`/`value_from_abi`
(`semcode_vm.rs:2700-2756`). `value_from_abi`'s `Quad` conversion now routes
through `quad_from_abi`, which rejects any byte outside `0..=3`
(`#1775`, confirmed already repaired) rather than masking it into a valid
quad. The ABI type itself, `prom_abi::AbiValue::Quad(u8)`
(`crates/prom-abi/src/lib.rs:130-139`), remains an unchecked raw `u8` carrier
with no `TryFrom`/checked constructor (`#1778`, confirmed **still open**) —
the issue's own text frames this as two separate root causes (a producer-side
type-level gap and a consumer-side validation gap) and treats the VM-side fix
as one defensive layer, not a substitute for closing the type itself.

## Resource/Failure Boundary

Confirmed as a **separate boundary from ownership**, correctly so: `#1759`
(`max_steps`/`max_calls` quotas defined but never enforced — zero
`QuotaKind::Steps`/`QuotaKind::Calls` call sites in `semcode_vm.rs`), `#1760`
(`trace_enabled`/`max_trace_entries` inert — no consumer), `#1761`
(`ConstPool` quota inert — no consumer), `#1762` (`ExecutionConfig` allows
`context` and `quotas` to disagree; the VM never reads `config.context` at
all), and `#1763` (`RuntimeTrap` documents eight variants; only four —
`AssertionFailed`, `BorrowWriteConflict`, `ArithmeticOverflow`,
`DivisionByZero` — are ever actually constructed; the rest of the VM's real
failure surface lives in parallel top-level `RuntimeError` variants) are all
**confirmed still present** at this commit. None of the five concerns who
owns a value's memory; all five are execution-resource-ceiling or
failure-vocabulary-drift issues. `BorrowWriteConflict` itself is the one
point of contact between the two boundaries (it is a `RuntimeTrap` variant
that is actually constructed, from ownership enforcement) — everything else
in this cluster is orthogonal to SSF-08's ownership scope and should be
resolved on its own track, not folded into ownership repair work.

Position A is bounded in the two senses the governing task asks this record
to keep separate: (1) ownership semantics operate over a bounded,
statically-admitted path/value model, evidenced above; (2) execution has an
explicit (if partly unenforced) resource-quota vocabulary, evidenced here —
these are different contracts and this record does not conflate them.

## Position A

See "What Semantic Claims" / "Does Not Claim" above. Implementation fit:
resolved for plain scalar/direct-root rebinding (confirmed copy-by-value,
no open defect). Strong for the golden-covered scenarios in tuple paths and
direct record-field paths (frozen, documented, D7-qualified positive and
negative end-to-end evidence), with the caveat that
`#1709`/`#1724`/`#1725`/`#1726` apply to these paths too and general-case
correctness beyond the tested scenarios remains open (see Included
Ownership Surface). Partial fit for Sequence static-index and ADT-payload
paths (implemented and largely tested, but blocked from promotion by
`#1709`/`#1718`/`#1724`/`#1725`/`#1726`). No fit claimed, and none needed, for
indirect/dynamic paths, Map paths, or schema paths — these are correctly
absent end to end (frontend never produces a path for them;
`expr_access_path`, `crates/sm-front/src/typecheck.rs:15453-15476`, only
resolves `Var`/`RecordField`/literal-`SequenceIndex`), and the repository's
own `runtime_ownership_unsupported_paths_do_not_silently_claim_support` test
(`tests/runtime_ownership_e2e.rs:569-582`) proves indirect/nested record
projection is rejected deterministically at compile time today, not
"best-effort normalized."

## Position B

Rejected for this phase. The minimum real system Position B would require,
grounded in what is actually absent today (not hypothetically difficult):

- A lifetime/region representation, since none exists anywhere in `sm-vm`
  today (confirmed absent by direct mechanism search, not by argument from
  difficulty).
- Borrow lifetime propagation beyond "active for the current call frame,
  installed wholesale at frame entry" — today's model has no per-event
  activation point (`#1726`) and no partial release.
- Inter-frame ownership rules — today's `push_frame` never derives a callee's
  borrow set from the caller's; building one is a new mechanism, not an
  extension of an existing partial one.
- A genuine alias model — today there is no aliasable memory for one to
  operate over (`Value::clone()` is a full deep copy throughout); Position B
  would require introducing shared/reference-counted value representation
  into the VM before an alias model would even have something to reason
  about. This is a foundational VM-architecture change, not a bounded
  ownership-module change.
- A verifier semantic expansion from pure structural admission into actual
  borrow-policy evaluation (overlap, lifetime, release) — today's verifier
  deliberately leaves 100% of that to the VM (confirmed, see Verifier
  Admission above).
- Release semantics and a release event kind in the wire format — today's
  OWN0 event-kind space is exhaustively `{Borrow, Write}`; adding release
  is a SemCode version-family change, not a policy toggle.
- Call-transfer and host-transfer ownership rules — today, ownership state
  does not cross a call boundary in either direction (confirmed:
  `push_frame` installs only the callee's own static list; `Ret` returns
  only a value and a register index).
- Verifier-side path-identity and bounds validation strong enough to make
  `#1725`'s class of bug structurally impossible, not just individually
  patched.

None of this is "more tests." Each item is a new mechanism or a new wire
contract. Position B is not selected because current `main`, even counting
every landed-but-unqualified surface found in this audit, implements none of
these — it implements more *path-family* breadth than the frozen spec
documents, not a different *kind* of ownership system.

## Decision Matrix

| Dimension | Position A | Position B |
|---|---|---|
| Public claim | Bounded, frame-local, deterministic path/value discipline over admitted paths | General systems-ownership model with borrow/lifetime/alias guarantees |
| Current implementation fit | Resolved for scalar; scope settled and golden-scenarios qualified but general case open for tuple/record (`#1709`/`#1724`/`#1725`/`#1726`); partial for Sequence/ADT; correctly absent for indirect/Map/schema | Not implemented at any layer; no lifetime/region/alias/release mechanism exists |
| Missing implementation | Heterogeneous, bounded work across three layers, not a flat count — see Repair Dependency Order for the full DAG: **Frontend**, canonical `ScopeEnv` transition/join model closing `#1656`-`#1664`; **IR/transport**, `#1709` (event preservation), `#1724` (lexical binding identity), `#1725` (OWN0 root identity), `#1726` (OWN0 event timing); **Contour/version**, `#1718` (path-family capability alignment) | Lifetime/region representation, alias model, release semantics, inter-frame rules, shared-value VM architecture — all from scratch |
| Verifier impact | None required; structural admission already matches the claim | Full semantic-policy expansion (overlap, lifetime, release evaluation) |
| VM impact | Close the named IR/transport bugs within the existing frame-local model; `#1724`'s fix lives in IR lowering (name generation) rather than VM code proper, but must land before VM-observed local identity is trustworthy under shadowing | New activation-timing model, partial release, inter-frame merge, shared-value representation |
| SemCode/OWN0 impact | Promote or fail-closed-reject Sequence/ADT under an explicit capability (`#1718`); fix root identity (`#1725`) | New event kind (release), new path-identity contract, likely new header family |
| Frontend complexity | One join-model fix closes nine findings (`#1656`-`#1664`) | Same fix required, plus mutable-borrow/reborrow/partial-move source semantics (explicitly deferred by `CaptureMode`'s own doc comment) |
| Aggregate/path requirements | Tuple, record field frozen; Sequence static-index and ADT payload gated behind named repairs | Indirect/dynamic paths, Map, schema — none representable without new frontend+IR+verifier work |
| Frame semantics | Borrow lifetime = current call frame (already true; timing bug is orthogonal) | Cross-frame persistence, partial release before frame exit |
| Host semantics | No ownership crosses the host boundary today (unaffected by this decision) | Would require host-transfer ownership rules, not currently scoped |
| Lifetime/region model | None; not required by any currently-qualified feature | Full representation required from zero |
| Alias model | One fixed structural overlap rule; no aliasable memory exists to model | Requires introducing shared/reference-counted values into the VM first |
| Release semantics | Implicit, via frame-pop/Rust-drop; no explicit release event | Explicit release event kind, SemCode version-family change |
| Determinism proof burden | Already carried by existing goldens + `runtime_ownership_e2e.rs` | Every new mechanism needs its own determinism proof from scratch |
| Stable Foundation necessity | Sufficient for #1579's acceptance criteria (deterministic agreement, explicit taxonomy, no overclaim) | Not required by any current Stable Foundation target row |
| Risk of false-ready claim | Low if `#1709`/`#1718`/`#1724`/`#1725`/`#1726` are closed before any promotion | High — no current evidence supports advertising it, and this audit found active correctness gaps even in the frozen tuple/record slice |

## Selected Position

**Position A**, formally selected for SSF-08. The wording in "What Semantic
Claims"/"Does Not Claim" above is the normative public statement.

## Normative Invariants

**Ownership fact preservation.** An ownership fact may be transformed,
joined, transported, activated, or rejected. It must never disappear merely
because execution crossed a control-flow, lowering, serialization, frame, or
host boundary. (Currently violated by `#1656`-`#1664` at the frontend layer
and by `#1709` at the IR layer — both are Known Blocking Findings, not
repaired by this PR.)

**Fail closed.** Cannot prove ownership state → deterministic rejection.
Never: cannot find binding/path/state → assume Available. (Currently
violated by `ScopeEnv`'s seven fail-open query APIs, per `#1664` — a Known
Blocking Finding, not repaired by this PR. The VM/verifier layers already
fail closed: unsupported path shapes and malformed OWN0 payloads reject
deterministically, per the Verifier Admission rejection-surface table
above.)

**Conservative joins.** For branch/alternative control flow, post-state may
claim availability only if availability is valid for every reachable
predecessor state. Uncertainty never restores availability. (This is the
target invariant for the `#1656`-`#1664` repair; it does not yet hold, since
no join exists at all for `if`/`loop`.)

**Path identity.** The path validated by source/frontend ownership must
denote the same path after lowering, serialization, verification, and
runtime decoding. (Currently violated by `#1725` — a Known Blocking Finding
and the highest-priority repair identified by this audit.)

**Event timing.** Borrow/Write semantic order must survive every transport
boundary. (Currently violated by `#1726` — a Known Blocking Finding. Its
failure direction is soundness-safe/over-conservative, not unsound, per the
IR/OWN0 Transport section above.)

**Frame lifetime.** Position A promises: borrow lifetime = current call
frame, exactly as `runtime_ownership.md` already states. No release or
cross-frame persistence is promised or implied.

**Unsupported path forms.** Any path family outside the selected contour
must reject deterministically, not best-effort normalize, skip ownership
work, or silently downgrade to root-only semantics. (Already true today for
indirect/nested projection, per `runtime_ownership_unsupported_paths_do_not_silently_claim_support`.
`#1718`'s gap is that Sequence/ADT paths are accepted rather than rejected
under a header that does not declare them — the fix may be either explicit
promotion under a new capability or explicit fail-closed rejection; this
record does not select between those two remedies, since that is
implementation, not positioning.)

## Included Ownership Surface

| Family | Classification |
|---|---|
| Scalar / direct root | INCLUDED IN SSF-08 — frozen as **copy-by-value** for plain (non-destructuring) root rebinding, confirmed by full-pipeline empirical reproduction (see Frontend State Model); no unresolved defect in this invariant. Pattern-based destructuring capture of a scalar (e.g. as an ADT/tuple/record payload item) is covered by the Tuple/Record/ADT rows below, not this row. |
| Tuple | INCLUDED IN SSF-08 — scope is frozen and settled (unambiguous since before this decision); the golden-covered scenarios (sibling/same-path/parent-child/child-parent write, multi-frame cleanup) are D7-qualified and pass. **Caveat, not a scope question**: #1709 (nested `if`/`loop`-expression lowering can drop these exact events), #1724 (lexical binding identity — a shadowed tuple binding can resolve to the wrong runtime slot), #1725 (OWN0 root-identity correctness), and #1726 (event-timing correctness) all apply to tuple paths too, not only Sequence/ADT — general-case correctness beyond the tested scenarios is not yet proven and requires those four repairs. |
| Direct record field | INCLUDED IN SSF-08 — same scope and same caveat as Tuple: golden-covered scenarios (positive+negative E2E) pass; #1709/#1724/#1725/#1726 apply here too and must close before the guarantee generalizes beyond the tested cases. |
| Sequence static index | INCLUDED ONLY AFTER REQUIRED REPAIR (`#1718` capability alignment, `#1724`, `#1725`, `#1726`; already has stronger E2E proof — including dynamic-vs-static interaction cases — than the frozen spec currently credits it for) |
| ADT payload (fixed variant, single level) | INCLUDED ONLY AFTER REQUIRED REPAIR (`#1718`, `#1724`, `#1725`, `#1726`, plus closing the verifier's SymbolId bounds-check gap and reaching a genuine compiler-driven negative E2E test once the frontend supports mutable ADT-payload reassignment) |

## Deferred Ownership Surface

| Family | Classification |
|---|---|
| Map path | DEFERRED (no frontend `AccessPath` resolution exists for Map keys/values; not attempted by any current code path) |
| Schema path | NOT REPRESENTABLE (schema declarations are compile-time-only metadata with no executable runtime carrier at all, per `docs/spec/source_semantics.md`; there is no value for an ownership path to denote) |
| Indirect/dynamic projection (nested record/tuple/ADT projection, computed index) | DEFERRED, already fails closed today (`indirect_record_projection_source` in `tests/runtime_ownership_e2e.rs` is proven to reject at compile time) |
| Mutable borrow, partial move, reborrow, general lifetime inference | DEFERRED — explicit non-goal, stated in `CaptureMode`'s own doc comment prior to this decision |

## Known Blocking Findings

These are documented, classified, and left unrepaired by this PR, per the
governing task's hard scope.

1. **`#1725`** (highest priority) — frontend global `SymbolId` is
   reinterpreted as a function-local string-table index at VM decode time,
   masked by a silent `.unwrap_or` fallback rather than erroring. Affects
   every path kind, including the frozen tuple/record slice. Root identity
   correctness for the *entire* ownership guarantee is currently proven only
   for the specific numeric coincidences existing golden fixtures happen to
   exercise, not for the general case.
2. **`#1709`** — nested value-block-expr/loop-expr lowering can silently
   drop real, in-contract tuple/record Borrow events (missing
   `ownership_events` threading through `lower_expr_with_expected` and the
   loop-expression fallback path). This is the one finding in this audit
   with a genuine false-negative (unsound) direction: the VM can fail to
   reject a write it should reject, because the Borrow event never reached
   OWN0.
3. **`#1726`** — OWN0 decode discards Borrow/Write interleaving; the VM
   activates a callee's entire borrow set at frame entry rather than at the
   true program-order point. Soundness-safe direction (over-rejection), not
   unsound, but a real correctness gap relative to the documented "active
   for the current frame" semantics, which implies activation at the point
   of borrow, not merely "sometime in this frame."
4. **`#1718`** — Sequence static-index and ADT-payload path components are
   live end-to-end under a header (`SEMCOD11`) and capability
   (`CAP_OWNERSHIP_PATHS`) that document only tuple paths, with no dedicated
   capability bit and (for ADT) weaker verifier admission (no SymbolId
   bounds check).
5. **`#1656`-`#1664`** — the frontend `ScopeEnv` join-model gap (see
   Frontend State Model above); nine findings, one shared root, plus the
   seven-API fail-open gap (`#1664`) and the `expr_access_path`-vs-join
   coverage gap (`#1663`).
6. **`#1724`** — lexical scope flattening at IR lowering can leak a shadowed
   inner binding's value to an outer `LoadVar` of the same textual name
   after the inner scope exits. Related to, but analytically distinct from,
   `#1725`: a name-collision/shadowing defect rather than a
   numbering-space mismatch, both under the general heading of "binding
   identity is not preserved uniformly across the frontend-to-VM boundary."
   Routed to Repair Dependency Order Lane 2b, SSF-08-owned — see the
   Blocker-To-Route Audit table below for the ownership reasoning.
Item 7 in an earlier revision of this record ("whole-value `consumed`
tracking is dead code") is **removed**, not renumbered away silently: it was
investigated to completion (see Frontend State Model above) and resolved as
**not a blocking finding**. Plain root rebinding is copy-by-value by design
(evidenced, not inferred), so `consumed`/`mark_consumed` being unused is
inert scaffolding for an unpromised feature, not a gap in a promised
invariant.

## Repair Dependency Order

Verified against the evidence above, refining the governing task's working
hypothesis into a DAG rather than a strict chain — `#1725`/`#1726` are
orthogonal to the `#1718` component-kind-scope question (they affect every
path kind uniformly), so they do not strictly require `#1718` first:

```text
Lane 1 (frontend, compile-time discipline):
  1. canonical ScopeEnv ownership-state transition + join model
       (closes #1656-#1664 through one shared root)

Lane 2 (IR / binding identity / transport, runtime-safety-relevant):
  2a. #1709 nested-lowering event preservation
        (independent of Lane 1; may proceed in parallel)
  2b. #1724 lexical binding identity preservation
        (preserve the outer/inner lexical binding distinction through IR
         lowering and VM-local storage; independent of Lane 1's join model)
  2c. #1725 OWN0 root-identity correctness
        (independent of #1718; establishes a trustworthy identity contract
         for every path kind, including the already-frozen tuple/record slice)
  2d. #1726 OWN0 event timing
        (depends on 2c: fixing activation timing for a symbol whose identity
         is not yet trustworthy proves nothing)

  2b/2c sequencing: #1724 and #1725 are analytically distinct bugs, not one
  bug under two numbers — #1724 is lexical *binding* identity (a shadowed
  inner `let x` can leak its value to an outer `LoadVar "x"` after scope
  exit, because `IrInstr::StoreVar`/`LoadVar` key storage by bare textual
  name, `legacy_lowering.rs:191-198`, with no per-scope mangling); #1725 is
  ownership-*path-root* identity (the wire's global frontend `SymbolId` is
  reinterpreted as a function-local string-table index at VM decode,
  `semcode_vm.rs:1105-1109`). Their fixes touch different code: #1724's is a
  frontend/IR lowering change (how names are generated before emission);
  #1725's is a decode/remap change (how the wire's numeric id is resolved
  at VM load). Neither implementation calls into the other, so **2b and 2c
  may be implemented in parallel**. But #1725's fix can only be *validated*
  as a general-case identity guarantee — not merely "the off-by-one mapping
  bug is gone" — against a per-function local-identifier space that itself
  does not conflate two different lexical bindings sharing a name. If #1724
  is still open, a "correctly mapped" `#1725` fix can still resolve to the
  wrong runtime slot under shadowing, because the underlying storage slot
  is already ambiguous before OWN0 remapping ever runs. So: **#1724 should
  close before #1725's general-case qualification evidence is trusted**,
  even though the two may be coded in parallel — matching the 2b-before-2c
  ordering above.

Lane 3 (contour/versioning decision):
  3. #1718 path-family / SemCode capability alignment
        (decide promote-under-new-capability vs. fail-closed-reject for
         Sequence/ADT; independent of Lane 2, but Sequence/ADT cannot be
         promoted into the Included surface until both this AND Lane 2 close)

Lane 4 (frame/call/host, mostly already closed):
  4. frame/call/value trust boundary monitoring (#1770/#1771/#1773/#1775 —
     already repaired; no action needed) + #1778 (prom-abi typed Quad,
     still open, independent track)

Lane 5 (resource/failure, independent of ownership):
  5. #1759-#1763 quota enforcement + failure-taxonomy alignment
        (does not block ownership qualification; tracked separately)

  -> end-to-end SSF-08 qualification only after Lanes 1-3 close
     (Lanes 4-5 do not block ownership qualification specifically, but do
      block SSF-08's own full acceptance criteria, which also covers host
      and quota boundaries)
```

`ScopeBinding.consumed`/`mark_consumed` is **not** a lane in this DAG. Per
the Frontend State Model and Correction Log above, it is inert scaffolding
for a whole-root move feature the current admitted contract does not
promise (plain root rebinding is frozen as copy-by-value). It is not
required implementation for SSF-08 Position A. Removing or repurposing it
is optional future cleanup, tracked outside SSF-08's repair DAG, not a
blocker for this phase's exit gate.

### Blocker-To-Route Audit

Every item this record investigated maps to exactly one route. No item may
appear in Known Blocking Findings without a route, and no route may be
assigned without the item appearing somewhere in this record's evidence.

| Item | In Known Blocking Findings? | Route | Where |
|---|---|---|---|
| `#1656`-`#1664` | Yes | REPAIR LANE | Lane 1 |
| `#1709` | Yes | REPAIR LANE | Lane 2a |
| `#1724` | Yes | REPAIR LANE | Lane 2b |
| `#1725` | Yes | REPAIR LANE | Lane 2c |
| `#1726` | Yes | REPAIR LANE | Lane 2d |
| `#1718` | Yes | REPAIR LANE | Lane 3 |
| `#1770` | No | ALREADY CLOSED | Frame/Host Boundary — `RegisterSlot::Uninitialized` confirms the fix in code, not just issue state |
| `#1771` | No | ALREADY CLOSED | Frame/Host Boundary — `MapGet`'s `?` propagation confirms the fix in code |
| `#1773` | No | ALREADY CLOSED | Frame/Host Boundary — `validate_call_arguments` in `push_frame` confirms the fix in code |
| `#1775` | No | ALREADY CLOSED | Frame/Host Boundary — `quad_from_abi`'s `0..=3` domain check confirms the fix in code |
| `#1778` | No | INDEPENDENT SSF-08 TRACK | Lane 4 — host canonicality (`prom-abi`'s `AbiValue::Quad(u8)`), not ownership |
| `#1759`-`#1763` | No | INDEPENDENT SSF-08 TRACK | Lane 5 — resource/failure boundary, not ownership (see Resource/Failure Boundary section) |
| `consumed`/`mark_consumed` | No (removed, see Correction Log) | (not a GitHub issue; no route needed) | Resolved as inert, unpromised scaffolding — see Frontend State Model |

`#1724` is classified **REPAIR LANE, SSF-08-owned (Option A)**, not RETURN.
Reasoning: it is filed under `FA-04` (the `sm-ir` module), not `FA-02` (the
`sm-front` module the completed SSF-07 exit reconciliation already
triaged), so it was never in scope for that reconciliation to claim or
release. It is not a source-language-contract question either — the
completed SSF-01 contract already correctly *defines* lexical block scoping
(`docs/spec/source_semantics.md`'s "Scope And Binding Rules"); `#1724` is an
*implementation* defect in how `sm-ir` lowering honors that already-settled
contract, cross-cutting frontend `ScopeEnv` → IR local storage → VM
locals — squarely inside SSF-08's own "value paths, frames" ownership scope,
which cannot make a trustworthy binding-identity promise while the
underlying lexical-identity guarantee it depends on is unproven. This
matches the working classification already recorded in prior SSF
dependency-mapping conversation history (CROSS into `sm-ir`/OWN0/VM,
alongside `#1709`/`#1718`/`#1725`/`#1726`), independently re-derived here
from the code rather than assumed from that history.

## Public Wording Rules

- Never use "memory safe"/"memory-safe" for Semantic's ownership guarantee
  without the explicit scope qualifier already used in this record (bounded,
  frame-local, value-domain discipline over admitted paths — not shared-memory
  protection, because no shared/aliased memory representation exists).
- Never describe the verifier's OWN0 handling as "borrow checking" or
  "static borrow checking" — it is shape/version/capability admission only,
  confirmed with zero semantic path reasoning.
- Never claim "Rust-like ownership," "Rust ownership," or "systems ownership"
  for Semantic without the explicit non-claims list from this record attached
  in the same breath.
- "Landed and qualified on `main`" is not a stable-contract promise; Sequence
  static-index and ADT-payload ownership are landed and *substantially*
  tested but remain **INCLUDED ONLY AFTER REQUIRED REPAIR** per this record
  — current-facing docs must not describe them as part of the frozen
  contract until `#1718`/`#1725`/`#1726` close.
- "Frame-local borrow lifetime" must not be read as "activates precisely at
  the point of borrow" until `#1726` closes; today it means "active for the
  entire callee frame, from entry."

## Qualification Required Before SSF-08 Exit

- Lanes 1-3 of the repair dependency order above close, each with positive
  and deterministic-negative fixtures for every included form.
- `#1725`'s identity contract is proven for the general case (not just
  current golden-fixture numeric coincidences) before any promotion of
  Sequence/ADT ownership into the frozen contract.
- `#1718` is resolved one way or the other (promote-with-capability or
  fail-closed-reject) — "silently accepted under the wrong header" is not an
  acceptable end state for SSF-08 exit.
- Verifier/runtime agreement is re-demonstrated after Lane 2 changes (the
  existing `runtime_ownership_e2e.rs` determinism-across-runs tests are the
  right shape to extend, not replace).
- `#1759`-`#1763` (quota/taxonomy) and `#1778` (host ABI canonicality) are
  closed or explicitly re-scoped out of SSF-08's own exit gate by an
  explicit follow-up decision — this record does not resolve them, since
  they are a different boundary (see Resource/Failure Boundary above), but
  SSF-08's own acceptance criteria (`.harness/current.task.yaml`) names
  "memory/resource quotas and deterministic failure taxonomy" as in scope.

## Non-Goals

Everything listed under "What Semantic Explicitly Does Not Claim" above,
plus: this record does not implement any repair; does not decide the
`#1718` promote-vs-reject question (that is an implementation decision for
the Lane 3 repair slice); does not touch production Rust in `sm-front`,
`sm-ir`, `sm-format`, `sm-verify`, or `sm-vm`; does not open or close any of
the 24 issues investigated; does not modify CI/workflow configuration; does
not authorize merging this PR.

## Correction Log

Recorded per owner review, before merge, on this same PR. Entries 1-2 are
from the review of the initial revision (commit `3ac20fe7`); entry 3 is
from a second review of that correction (commit `f3e172bc`); entry 4 is
from a third review of that correction (commit `3ad8507f`):

1. **Quota/taxonomy current-facing contradiction** — this decision record's
   own Resource/Failure Boundary evidence (`#1759`-`#1763`, confirmed still
   present) directly contradicted
   `docs/roadmap/stable_foundation/semantic_stable_foundation_matrix.md`'s
   "Quotas/fuel and trap taxonomy" row, which read **Landed and qualified on
   `main`**. Corrected to **Landed but unqualified**, using the matrix's own
   existing status vocabulary (no new status invented), with the evidence
   column naming the specific enforced-vs-inert quota split and the
   taxonomy-drift gap, and the routing column pointing to `#1759`-`#1763`/
   SSF-08. This is a genuine current-facing overclaim this PR corrects — the
   "Public-claim audit" count of 0 corrected in the original PR body was
   wrong and is corrected in the PR description.
2. **Scalar/direct-root classification asserted without full empirical
   proof** — the initial revision classified scalar/direct-root as
   "INCLUDED IN SSF-08 (already qualified)" while simultaneously filing a
   "whole-value `consumed` tracking is dead code" Known Blocking Finding
   against the same invariant, without first determining whether Semantic's
   actual contract is copy or move. Resolved empirically (see Frontend
   State Model): plain root rebinding is copy-by-value by design, evidenced
   by the `use_after_move_rejects` test's own comment and by full-pipeline
   execution of representative scalar families plus a non-Copy aggregate.
   The blocking finding is removed (not silently dropped — recorded as
   resolved); a durable regression is added at
   `tests/scalar_root_rebind_copy_semantics_e2e.rs`. The same re-check also
   surfaced that Tuple and Direct record field carried the identical
   "qualified + blocking defect in the same invariant" shape via
   `#1709`/`#1725`/`#1726` (which apply to every path kind, not only
   Sequence/ADT) — both rows now carry an explicit caveat rather than
   reading as unqualified endorsements.
3. **Two residual echoes of the pre-Correction-2 model, plus the caveat from
   Correction 2 left unresolved at the maturity-status level** — a second
   owner review (commit `f3e172bc`) found: (a) the Decision Matrix's
   "Missing implementation" cell and the Repair Dependency Order's Lane 1
   description both still listed the whole-value `consumed` gap as
   something to fix, contradicting Correction 2's own resolution that it is
   inert, unpromised scaffolding — both references removed, with an
   explicit note added stating `consumed`/`mark_consumed` is not part of
   this DAG and is optional future cleanup, not an SSF-08 blocker; (b) more
   substantively, `semantic_stable_foundation_matrix.md`'s "OWN0
   tuple/direct-record paths" row still read **Landed and qualified on
   `main`** — an unqualified maturity claim — even though this same record
   proves `#1709` is a false-negative-direction bug inside that exact
   frozen contract and `#1725`/`#1726` violate this record's own normative
   Path-identity/Event-timing invariants for every path kind including
   tuple/record. A routing-column caveat is not sufficient when the status
   column itself is the overclaim. Corrected to **Landed but unqualified**,
   preserving the D7/golden-scenario evidence in the evidence column and
   naming `#1709`/`#1725`/`#1726` as the general-case blockers — the same
   fix shape as Correction 1, applied to the row Correction 1 did not
   touch. The Decision Matrix's "Current implementation fit" row was
   tightened to match.
4. **`#1724` declared blocking with no repair route** — a third owner
   review found `#1724` listed in Known Blocking Findings while the Repair
   Dependency Order contained no lane for it, an inconsistency with the
   record's own classification. Resolved: `#1724` is a genuine SSF-08-owned
   finding (Option A, not RETURN — see the Blocker-To-Route Audit's
   reasoning), added to the DAG as Lane 2b, ahead of `#1725` (Lane 2c) —
   implementable in parallel with `#1725`, but `#1725`'s general-case
   (shadowing-inclusive) qualification evidence is only trustworthy once
   `#1724` closes, since both bugs trace to the same root cause
   (`sm-ir`'s per-function locals being identified by bare textual name).
   The Decision Matrix's "Missing implementation" row, which the same
   review correctly flagged as reducible to a misleading count even before
   accounting for `#1724`, is now a heterogeneous three-layer breakdown
   instead of a number. A Blocker-To-Route Audit table was added, covering
   every one of the 24 investigated issues plus `consumed`/`mark_consumed`,
   so no future revision can let a declared blocker silently lose its
   route again. In the course of this pass, an unrelated miscount ("20
   GitHub issues" where the cited list is 24) was also found and corrected
   — not owner-flagged, caught during this same audit discipline.

Position A is unchanged by all four corrections.

## Evidence Index

- `docs/spec/runtime_ownership.md` — current frozen tuple+record v0 contract
  (CURRENT AUTHORITY; already closely aligned with Position A; not modified
  by this PR beyond the pointer added below).
- `docs/spec/semcode.md` — SEMCOD11/SEMCOD12 header/capability contract,
  including the pre-existing documented gap that admission at those
  revisions only proves *some* function has OWN0, not every function
  (CURRENT AUTHORITY; explicitly out of scope, predates this decision).
- `docs/spec/verifier.md` — structural-admission-only framing for ownership
  (CURRENT AUTHORITY; consistent with verifier-agent findings).
- `crates/sm-front/src/lib.rs`, `crates/sm-front/src/typecheck.rs`,
  `crates/sm-front/src/types.rs` — `ScopeEnv`, `CaptureMode`, pattern-plan
  machinery (PRIMARY SOURCE EVIDENCE).
- `crates/sm-ir/src/legacy_lowering.rs` — `AccessPath`, `PathComponent`,
  `OwnershipPathEvent`, OWN0 emission (PRIMARY SOURCE EVIDENCE).
- `crates/sm-format/src/semcode_decode.rs`, `crates/sm-format/src/local_format.rs` —
  OWN0 decode, capability constants (PRIMARY SOURCE EVIDENCE).
- `crates/sm-verify/src/lib.rs` — structural admission (PRIMARY SOURCE
  EVIDENCE).
- `crates/sm-vm/src/semcode_vm.rs` — `Frame`, `push_frame`, overlap
  enforcement, host ABI conversion (PRIMARY SOURCE EVIDENCE).
- `crates/sm-runtime-core/src/lib.rs` — `PathComponent`, `QuotaKind`,
  `RuntimeTrap`, `ExecutionConfig` (PRIMARY SOURCE EVIDENCE).
- `crates/prom-abi/src/lib.rs` — `AbiValue::Quad(u8)` (PRIMARY SOURCE
  EVIDENCE).
- `tests/runtime_ownership_e2e.rs`, `tests/tuple_ownership_golden.rs`,
  `tests/record_field_ownership_golden.rs`, `tests/sequence_ownership_golden.rs`,
  `tests/pcc6_option_result_ownership_golden.rs`,
  `tests/scalar_root_rebind_copy_semantics_e2e.rs` — end-to-end positive and
  deterministic-negative ownership proof (PRIMARY SOURCE EVIDENCE).
- `docs/roadmap/stable_foundation/semantic_stable_foundation_matrix.md`,
  `stable_foundation_target_contract.md`, `stable_foundation_dependency_map.md`,
  `ssf07_exit_reconciliation_record.md` — SSF governance authorities
  (CURRENT AUTHORITY).
- Issues `#1656`-`#1664`, `#1709`, `#1718`, `#1724`-`#1726`, `#1759`-`#1763`,
  `#1770`, `#1771`, `#1773`, `#1775`, `#1778` — read via `gh issue view` and
  cross-checked against code at the evidence baseline commit (HISTORICAL
  RECORD / phase-A audit evidence, not repaired by this PR).
