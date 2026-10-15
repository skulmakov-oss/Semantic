# Semantic VM: Record-Field Consumption for Compiler State (SHF-P2 design & implementation)

Status: **implemented in PR #2048** (SHF-R #2030, finding F2; research track #2037).

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

SHF-P3 (#2039) made the local form linear. The field form stays quadratic without field consumption.

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
| **E** | **Verified liveness-based consumption in the VM**: (E1) release registers at a proven last use; (E2) move a record field out of a local when field-sensitive liveness proves it unobserved until the local is overwritten | `sm-vm` private planner and handlers | **implemented** |

## 4. Implemented Architecture (E1 + E2)

The implementation operates strictly within the VM private planner and execution loop, requiring zero public API, opcode, or wire contract changes.

### 4.1 E1 — Register Last-Use Release via Backward Liveness

- **CFG Analysis.** Backward register liveness is computed over basic blocks derived from bytecode:
  - Successors are fall-through instructions, `Jmp`/`JmpIf` targets, and returns (`Ret`).
  - Liveness state is represented with efficient bitsets over local registers.
- **Consumption in Container Constructors & Operations.**
  - `SequencePush`, `MakeRecord`, `MakeSequence`, `MakeTuple`, `MakeAdt`, `MakeClosure` take dead register operands instead of cloning them.
  - **Repeated-Register Safety Invariant:** When an instruction uses the same register in multiple operand positions (e.g., `MakeSequence [r1, r1, r1]` or `MakeTuple (r1, r1)`), operands are taken *only on their final occurrence* within the operand list (`!srcs[idx + 1..].contains(&src)`). Earlier occurrences clone the value, preventing uninitialized reads or empty values.
  - Registers that become dead at the instruction boundary are set to `RegisterSlot::Uninitialized`, preventing lingering references across loop iterations.

### 4.2 E2 — Sound Field Consumption Across CFG Boundaries

E2 identifies opportunities where a field of a local record can be moved out into a register operand rather than cloned:

- **CFG Boundary & Jump Target Verification:**
  - The window between `LoadVar r_src, s` and `RecordGet r_dst, r_src, T, idx` is checked to ensure no instructions branch into it from outside (`jump_target_in(pc_load + 1..=pc_get)`).
  - The window between `RecordGet` and the restoring `StoreVar s, r_val` is verified similarly: no jumps from outside the window may land within `pc_get + 1..=pc_store`.
  - No control-flow instructions (`Jmp`, `JmpIf`, `Ret`, `Call`, `ClosureCall`) are permitted between `LoadVar` and `RecordGet`.
- **Register Deadness & Multi-Field Projection Proof (`allowed_registers`):**
  - In Semantic's lowering of multi-field record updates (`st with { f1: e1, f2: e2, ..., fn: en }` with 3–15 fields), the compiler emits multiple `LOAD_VAR r, s` instructions before projecting each field with `RECORD_GET`.
  - The analysis collects all registers loaded from `s` within the window up to `pc_store`. For each such register, it verifies that it is *only* read by `RecordGet` for a *different* field index (`f_other != idx`).
  - These registers are whitelisted in `site.allowed_registers`. At runtime, `RecordGet` verifies that any live registers in the frame holding the record type belong either to `src` or to `allowed_registers`. If any unproven register references the record, the VM cleanly falls back to copying.
- **Strict Error Precedence Preservation in `RecordGet`:**
  - Before any field consumption occurs, `RecordGet` always validates the actual source register via `get_reg`.
  - Type matching (`Value::Record`, `type_name == expected_name`) and slot bounds (`index < src_record.slots.len()`) are checked immediately. If validation fails, the VM returns the identical `TypeMismatchRuntime` or `BadFormat` error with exact precedence, before any local modification.
- **Moved-out Placeholder & Observability Guarantee:**
  - The vacated record slot in the local is replaced with `Value::Unit`.
  - E2's soundness proof guarantees that no instruction reads this slot before `StoreVar s, _` overwrites the entire record.
  - Since VM traps and errors are terminal and frames are never exposed after error, moved-out placeholders can never be observed by user code or external callers.

## 5. Complexity & Scaling

With E1 + E2 active:
- Loop updates `st = st with { toks: push(st.toks, i), pos: st.pos + 1 }` achieve amortized $O(1)$ per iteration.
- Multi-field records (tested from 2 to 15 fields) maintain linear runtime scaling without quadratic `Arc::make_mut` buffer duplication.
- Planner overhead is linear with function size and bounded by early-exit filters for functions lacking candidate patterns.

## 6. Verification and Regression Coverage

1. **Repeated Register Invariant:**
   `container_constructor_repeated_register_regression` tests repeated register operands in container constructors (`[x, x, x]`, `(x, x)`), verifying that values are preserved correctly across multiple uses.
2. **Error Precedence Invariant:**
   `record_get_error_precedence_regression` executes `RecordGet` on a non-record source register (i32) in planned bytecode, asserting that `TypeMismatchRuntime` is thrown identically under all steps limits.
3. **Multi-Field Partial `with` Updates:**
   `partial_with_update_multi_field_records_regression` verifies 5-field (`State5`) and 10-field (`State10`) records with partial `with` updates across all steps limits using differential execution (`assert_equivalent_under_every_steps_limit`).
4. **Aliasing and Fallback:**
   `record_field_aliasing_fallback_is_equivalent` tests that when a record is aliased (`let alias = st;`), field consumption gracefully falls back to ordinary copying with 100% value equivalence.
5. **Full Quota Failure Sweeps:**
   Every instruction boundary is swept via `assert_equivalent_under_every_steps_limit` with and without the optimization active, proving deterministic counter and outcome equivalence.
