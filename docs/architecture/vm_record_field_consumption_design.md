# Semantic VM: Record-Field Consumption for Compiler State (SHF-P2 design)

Status: **technical design only** (SHF-R #2030, finding F2; research track #2037). No implementation is authorized by this document. Implementation requires a separate OWNER IMPLEMENTATION GO, its own PR, and is independent of P1 (persistent map storage).

## 1. Problem

The dominant state shape of a compiler written in Semantic is a record that carries growing collections:

```semantic
record Lx { toks: Sequence(i32), pos: i32 }
st = st with { toks: push(st.toks, i), pos: st.pos + 1 };
```

On `main` @ `94c5c50` (native Linux, release, trusted-compiler envelope, posted on #2030):

| n | `seq_push` (local accumulator) | `state_record_seq` (field accumulator) |
|---:|---:|---:|
| 8 000 | 10.5 ms | 930 ms |
| 16 000 | 15.6 ms | 4 162 ms |
| 32 000 | 30.7 ms | 17 073 ms (time exponent 2.04; Steps exponent 1.00) |

SHF-P3 (#2039) made the local form linear. The field form stays quadratic.

## 2. Root cause (verified from the emitted bytecode)

`smc compile` + `smc disasm` of the loop body:

```text
0047: LOAD_VAR      r7,  s1            ; base of `with` — never read again
004c: LOAD_VAR      r8,  s1
0051: RECORD_GET    r9,  r8, Lx, 0     ; st.toks
005a: LOAD_VAR      r10, s2
005f: SEQUENCE_PUSH r11, r9, r10       ; <- buffer is shared here
0066: LOAD_VAR      r12, s1
006b: RECORD_GET    r13, r12, Lx, 1    ; st.pos
0074: LOAD_I32      r14, 1
007b: ADD_I32       r15, r13, r14
0082: MAKE_RECORD   r16, Lx, [r11, r15]
008d: STORE_VAR     s1, r16
```

When `SEQUENCE_PUSH` executes, the `Arc` of the token buffer is held by:

1. the **local `s1`**, i.e. the old record, which is overwritten only at `008d`;
2. **r7** and **r8**, whole-record copies that are dead but never cleared;
3. **r9**, the operand;
4. **r11/r16 of the previous iteration**: registers are not released at their last use, so the previous result stays referenced.

`Arc::make_mut` therefore always copies all n elements. The SHF-P3 planner accepts only `LOAD_VAR rX, S` → `SEQUENCE_PUSH rZ, rX, rV` → `STORE_VAR S, rZ` on a **local** (conditions C1–C6). No condition can admit this shape, because the value pushed to is a *field of* `S` while `S` remains live.

Two independent facts must both be proven to update in place:

- **(R) register facts:** r7, r8, r9 and the previous r11/r16 hold no reference at the push. This is a register-liveness property.
- **(F) field fact:** field 0 of the value in `s1` is not observed between the projection at `0051` and the full overwrite at `008d`, on every path. This is a field-sensitive liveness property of a local.

A syntactic match on the eleven instructions above would establish neither under jumps, aliasing, calls or failures. It is rejected, per the OWNER DIRECTION.

## 3. Options

| | Option | Changes | Verdict |
|---|---|---|---|
| A | Extend the P3 triple matcher to the `RECORD_GET … MAKE_RECORD` shape | planner only | **rejected**: a syntactic pattern; cannot prove (F) across control flow; brittle against lowering changes |
| B | `Arc` storage for record slots | public `Value::Record` | **insufficient**: makes record copies O(1), but the buffer is still shared by the live old record (fact F) |
| C | New `RECORD_TAKE` / `RECORD_SET` opcodes emitted by the compiler | SemCode, verifier, compiler | **rejected for now**: a format and contract change; the sharing by the old local remains unless the compiler also proves (F) |
| D | Language-level in-place update syntax | Semantic-Language | **out of scope** (OWNER: do not modify the language) |
| **E** | **Verified liveness-based consumption in the VM**: (E1) release registers at a proven last use; (E2) move a record field out of a local when field-sensitive liveness proves it unobserved until the local is overwritten | `sm-vm` private planner and handlers | **proposed** |

## 4. Proposed design (E)

E is a generalisation of the SHF-P3 `ConsumptionPlan`. The plan is built once per execution from the verifier-admitted function table, with no global state. Each site carries a proof obligation, and anything not proven keeps the ordinary copying path.

### 4.1 E1 — register last-use release

- **Analysis.** Backward register liveness over the function CFG:
  - basic blocks come from the canonical `scan_function_bytecode` decode;
  - successors are the fall-through, `JMP`/`JMP_IF` targets and `RET` (none);
  - the representation is bitsets over `u16` registers;
  - the fixed point is O(I · R / 64) per pass and O(I · R / 64 · depth) overall.
- **Site.** For an instruction operand `r` read at `pc` where `r ∉ live_out(pc)` and `r` is not also the destination of `pc`: the handler takes `r` instead of `get_reg` cloning it, leaving `RegisterSlot::Uninitialized` exactly as P3 does. Instructions whose result is unused (r7 above) can simply release their destination.
- **Why it is safe.**
  - A register not live after `pc` is never read again on any path before being redefined.
  - Reading an `Uninitialized` slot is already a `BadFormat` error, so a planner bug fails closed instead of producing a wrong value. The differential tests in §6 are designed to catch it before that.
- **Scope.** E1 makes every *temporary* copy disappear. It also subsumes P3's C1/C2/C4 conditions, since those are last-use facts. P3's acceptance tests remain the reference.

### 4.2 E2 — field consumption from a local

- **Site.** `LOAD_VAR rA, S` immediately consumed by `RECORD_GET rB, rA, T, i`, with `rA` dead after it (E1).
- **Field-sensitive analysis.** For every local `S` and field index `i`, compute `field_live(S, i)` backward. A use of `S.i` is:
  - a `RECORD_GET _, rX, T, i` where `rX` is defined by a `LOAD_VAR rX, S` that reaches it (reaching definitions on registers);
  - **or any whole-value use of a register loaded from `S`**: call argument, `MAKE_RECORD`/`MAKE_TUPLE`/`MAKE_ADT` item, `STORE_VAR` of another local, `RET`, comparison, `with`-base copy, sequence push of the record, etc. Such a use conservatively counts as a use of **every** field.

  A full `STORE_VAR S, _` kills all `S.i`.
- **Acceptance.** The site is accepted iff `S.i ∉ field_live` immediately after the `RECORD_GET` **and** no borrowed path is rooted at `S` (the P3 condition C6, OWN0 discipline unchanged).
- **Action.** Move the field value out of the record stored in local `S`. Requirements:
  - The record container itself must be uniquely held. If `S`'s record is shared (for example, also held by another local after `let b = a`), the handler falls back to the ordinary copying path at runtime: the check is a uniqueness test, never an assumption.
  - The vacated slot is filled with a VM-internal placeholder that is never a valid observation. Per E2's proof no instruction reads it before `STORE_VAR S` replaces the whole record.
- **Placeholder choice.** Must be decided before implementation:
  - (a) reuse `Value::Unit`, with a debug-build assertion that a projected-out field is never read;
  - (b) add a private moved-out marker. That requires a non-public slot wrapper, because `RecordCarrier` slots are public `Value`s.

  The recommendation is (a), plus the differential oracle in §6. (b) is the fallback if the OWNER prefers a fail-closed sentinel at the cost of an API discussion.

With E1 + E2 on the bytecode in §2:
- the field moves from `s1` into r9 (E2);
- r7, r8 and r9 are released (E1);
- `SEQUENCE_PUSH` sees a unique buffer and pushes in place;
- `MAKE_RECORD` takes r11 at its last use (E1);
- `STORE_VAR s1, r16` takes r16.

Every iteration is O(1) amortised.

### 4.3 What does not change

- Semantic values, results, error kinds and texts.
- Steps, Calls, Frames and effect charges, and their order.
- SemCode, opcodes, the verifier and capabilities.
- The public API, under placeholder option (a).
- The behaviour of programs whose sites are not accepted.

Every `RuntimeError` is terminal and no public API exposes a `VM` or `Frame` after an error (revalidated in the SHF-P3 ADR §8), so partially-moved state is never observable on failure. This must be re-verified on the code at implementation time.

## 5. Risks and open questions

1. **Analysis cost before Step 1.**
   - Liveness costs O(I · R / 64) per iteration of the fixed point. Field liveness is bounded by (locals × fields used).
   - The planner-cost lessons of SHF-P3-FIX1/FIX2 apply: linear-time construction, an early exit for functions without candidates, and verifier-admitted negative controls.
   - Budget: no measurable regression on the SHF-P3 qualification corpus or on the 3.0 MB / 333k-instruction startup control.
2. **Nested paths.** `st.lexer.toks` is a path of depth 2. The first implementation should support depth 1 only and record depth > 1 as a measured follow-up.
3. **Records held in sequences or maps.** Not covered: the container holds the reference. P1 makes maps cheap to copy but does not move values out of them.
4. **Interaction with P1.** None functionally. With P1, `ctx with { syms: map_set(ctx.syms, …) }` is already O(log n) per update (path copy) without E2. E2 would only reduce its constant factor.

## 6. Qualification plan (for the future implementation PR)

- **Reference oracle.** Run every fixture with the plan disabled and enabled. Outcome, values, Steps, Calls, Frames and error text must be identical:
  - the 224 SHF-P3 fixtures;
  - the 120 seeded programs;
  - new field-accumulator fixtures with aliasing (`let b = st;`), calls, early returns, loops with `break`, nested records, borrowed paths and records stored in sequences.
- **Failure sweeps.** Every Steps/Calls/Frames limit on the field workloads, as in SHF-P3.
- **Planner equivalence.** For E1, compare liveness against a brute-force path-enumeration oracle on small random CFGs. E2 acceptance is checked the same way.
- **Bytecode mutations.** Verifier-admitted mutations of the field-accumulator shape: extra reads of `S.i`, jumps into the window, reads through an aliasing local. All must be rejected by the plan.
- **Performance.** The `state_record_seq` family must reach a time exponent ≈ 1 (target: within ×2 of `seq_push` at 32k). No regression above noise on the existing corpus. Wall-clock figures stay informative only.

## 7. Decision requested

1. Approve option E (E1 + E2, depth-1 fields) for implementation in a separate PR after P1.
2. Choose the placeholder: (a) `Value::Unit` with debug assertion, recommended; or (b) a private sentinel.
3. Confirm that E1 (general last-use release) may land as the first checkpoint of that PR, with its own A/B, before E2.
