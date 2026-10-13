# Semantic VM: Shared Value Storage and Verified Consumption

Status: implemented on branch `perf/vm-p3-cow-consume` (SHF-P3, #2037). Awaiting OWNER review; merge is not authorized by this document.

Decision owner: repository OWNER. A public API change was authorized by OWNER CONTRACT GO (Option B, `Arc`).

## 1. Problem and evidence

`sm-vm` copied composite values deeply on every register read (`get_reg` → `v.clone()`), on every `LOAD_VAR`, and on every `STORE_VAR`. The common accumulator `xs = push(xs, v)` lowers to

```text
LOAD_VAR      rX, xs
...           (compute rV)
SEQUENCE_PUSH rZ, rX, rV
STORE_VAR     xs, rZ
```

and therefore copied the whole growing sequence three times per push. That is quadratic in the length.

Research evidence (SHF-RF, baseline `0373bec5`; packages F2-R0, P3-R0 and P3-R1, verified by SHA-256):
- **F2-R0:** for a 23 KB lexer, token-sequence cloning accounted for about 98% of runtime. The copied-node and allocated-byte exponents were 2.00.
- **P3-R1, Windows 11 and Ubuntu 24.04 (WSL2):** four prototypes, 654 fixtures and 1,819 failure-sweep points per platform, with no semantic mismatch. Results:
  - P2 (consumption only) and P3 (consumption + copy-on-write) are equal on the lexer (about ×50 vs baseline).
  - **On the parser, P2 gives ×1.0 and P3 gives ×29–37.**
  - On the evaluator, P3 gives ×1.5–1.9.
- **Independent native Debian 13 run (curator):** confirmed the lexer results.

## 2. Why P2 alone is insufficient

Consumption only removes copies inside the accepted accumulator pattern. In the parser, the dominant copies come from passing state that contains token sequences through calls and returns, and from field and index reads of composites held in locals. P2 cannot avoid these copies: a callee must receive an independent value while the caller keeps its own. Measured on the parser `b_parse_32`, P2 still copied 3.63 M sequence nodes, against 4.31 M for the baseline.

## 3. Why P3 was selected

Copy-on-write makes *every* logical copy of a sequence O(1). Consumption keeps the accumulator's buffer unique, so it is updated in place. The combination covers both the accumulator pattern (lexer) and the pass-around pattern (parser, evaluator). It was never slower than P2 outside measurement noise, and it reduced peak memory by up to 63%.

## 4. Public API preservation strategy

The frozen public API (`tests/golden_snapshots/public_api/sm_vm_semcode_vm.txt`) contains `Value::Sequence(Vec<Value>)`, `pub Frame.locals: HashMap<SymbolId, Value>` and `pub VM.callstack`.

**A private runtime representation (Option A) cannot deliver the parser benefit without changing these public types.** The sharing must live inside values held in the public `Frame.locals`, and converting at every `LOAD_VAR`/`STORE_VAR` would reintroduce the O(n) copy on the hot path.

**Decision (OWNER CONTRACT GO, Option B):** the single public change is

```diff
-    Sequence(Vec<Value>),
+    Sequence(Arc<Vec<Value>>),
```

- `Arc`, not `Rc`. With `Rc`, `Value`, `Frame` and `VM` lose `Send` and `Sync` (verified: E0277). With `Arc` they keep both. A compile-time guard (`assert_public_runtime_types_are_send_sync`) enforces this in every build.
- `Arc` against `Rc` was measured within noise on the parser, evaluator and lexer workloads (P3-IMPL Option B probe, 218/218 fixtures equivalent).
- Every other public type, function and field is unchanged. SemCode, opcodes, revisions, capabilities and the verifier are unchanged.

**Migration for downstream code that constructs or consumes sequences:**

| Before | After |
|---|---|
| `Value::Sequence(vec)` | `Value::Sequence(Arc::new(vec))` |
| By-value use of the `Vec` (`for x in items`, `items.push(..)`) | `Arc::unwrap_or_clone(items)` or `Arc::make_mut(&mut items)` |

Read-only uses (`items.len()`, `items.iter()`, `items[i]`, pattern matching) compile unchanged through `Deref`. No crate in this workspace outside `sm-vm` constructs `Value::Sequence`; the workspace compiles unchanged.

## 5. Internal value representation

- Only `Sequence` uses shared storage in this change.
- `Text`, `Bytes`, `Tuple`, `Record`, `Map`, `Adt` and `Closure` captures keep their existing owned representation. A logical copy of them is still a deep copy (see §14).
- Nested sequences share at every level: an outer sequence holds `Value::Sequence(Arc<…>)` elements, so copying the outer buffer copies element handles, not the inner buffers.

## 6. Unique ownership and copy-on-write rules

1. **Logical copy.** A logical copy (`get_reg`, `LOAD_VAR` without consumption, argument passing, returns) clones the `Arc`: O(1), shared.
2. **Updates.** Every update goes through `Arc::make_mut`: `SEQUENCE_PUSH`, `SEQUENCE_POP`, and `SEQUENCE_PREPEND` via `Arc::unwrap_or_clone`.
   - If the buffer is uniquely owned, it is updated in place.
   - If it is shared, it is first copied, and the other owners keep the previous logical value.
3. **No shared mutable memory.** A buffer that is shared is never written; it can only be read or copied. There is no `unsafe`, interior mutability or reliance on reference counts beyond `Arc`'s own `make_mut`/`unwrap_or_clone` semantics.

## 7. Static consumption plan and fallback

`ConsumptionPlan` is built once per execution (`exec_loop_with_profile`). It is derived from the same function table the execution runs, which the Z2 split-borrow keeps immutable for the whole execution, and it is passed into the dispatch loop.

**Scope.** It is an owned value. There is no global, thread-local or name-keyed cache that could leak across programs, VM instances or threads.

**Decoder.** It uses the canonical instruction walk. `validate_function_bytecode` is now a thin wrapper over `scan_function_bytecode`, which can also record every `u16` operand. Validation reads, errors and order are unchanged, and there is no second opcode decoder or operand table.

**Acceptance.** A triple `a: LOAD_VAR rX,S` … `b: SEQUENCE_PUSH rZ,rX,rV`, `c: STORE_VAR S,rZ` is accepted only if all of the following hold. Anything not accepted runs the ordinary copying path.

| Condition | Requirement |
|---|---|
| C1 | `rX` appears as an operand in no instruction except `a` and `b`. Every `u16` operand counts, so a collision can only reject an optimization. |
| C2 | `rZ` appears as an operand in no instruction except `b` and `c`. |
| C3 | `c` immediately follows `b`, and there is no `JMP`/`JMP_IF`/`RET` strictly between `a` and `b`. |
| C4 | No jump target `t` with `a < t ≤ c`. |
| C5 | No other `LOAD_VAR`/`STORE_VAR` of `S` strictly between `a` and `c`. |
| C6 | No borrowed path of the function is rooted at `S`. |
| Overlap | `rX ≠ rZ`, and `rV` differs from both. |

**Execution of an accepted triple:**
- `a` moves `S` out of `Frame.locals` into `rX`.
- `b` moves the sequence out of `rX` (`take_reg`, leaving `RegisterSlot::Uninitialized`, never `Value::Unit`).
- `c` moves the result out of `rZ` back into `S`.

**Authority.** The plan never decides admission. The verifier (`sm-verify`) remains the only admission authority, and the plan only chooses a faster implementation of already-admitted code.

### 7.1 SHF-P3-FIX1: linear-time planning

**Defect (original planner, `08637f9`).** For every `SEQUENCE_PUSH` candidate, `plan_function` ran:
- two whole-function operand scans (C1/C2);
- a backward search for the matching `LOAD_VAR`;
- a scan of all jump targets (C4);
- range scans for C3/C5;
- a scan of all borrowed paths (C6).

That is O(candidates × instructions) per function. The plan is built before any Step is charged and again on every execution.

The defect was found by source inspection (ChatGPT, Codex R2) and reproduced by Claude Code on ordinary compiler-generated, verifier-admitted programs. With `max_steps = 1`, 64 functions × 1,300 pushes (3.0 MB) took **3.4 s against 0.29 s pristine (×11.6)**.

A hand-crafted variant that reuses registers was proposed. It remains **UNTESTED**; it is a hypothesis, not a reproduced result.

**Algorithm (FIX1).** One canonical decode (`scan_function_bytecode`), then:
1. **Register-use index.** For every decoded `u16` value, record its distinct instruction PCs, up to three. A fourth distinct PC saturates the entry to "used elsewhere".
   - Three are enough: any value at three distinct PCs occurs outside every two-PC allowance.
   - Repeats of a value within one instruction count once, which is exactly the "any instruction other than `a`, `b`" rule.
   - Every `u16` still counts (registers, symbol ids, counts, immediates), so a collision can only reject.
2. **Jump targets** are sorted once. C4 (`a < t ≤ c`) is a binary search.
3. **Borrowed roots** are collected once into a set (C6).
4. **One forward pass** keeps state that, at candidate `b`, describes exactly the instructions strictly before `b`:
   - the last `LOAD_VAR` index per destination register: the nearest preceding `LOAD_VAR rX`, as before;
   - the last `LOAD_VAR`/`STORE_VAR` index per resolved `SymbolId` (C5);
   - the last `JMP`/`JMP_IF`/`RET` index (C3).

   State is updated only after the candidate at the current index has been evaluated.

**Equivalence to C1–C6** (source argument, mirrored by tests):
- **C1/C2:** the old `any(pc ∉ allowed ∧ u16s ∋ r)` equals "some recorded distinct PC ∉ allowed, or more than three distinct PCs".
- **C3:** the old "barrier in `(a, b)`" equals "the last barrier before `b` is after `a`". `a` is a `LOAD_VAR`, so it is never a barrier itself.
- **C4:** unchanged predicate, evaluated by binary search.
- **C5:** the old "access of `S` in `(a, b)`" equals "the last access of `S` before `b` is not `a`". `a` accesses `S`. `b` is a `SEQUENCE_PUSH` and `c` is adjacent, so `(a, c)` and `(a, b)` contain the same accesses.
- **C6 and register overlap:** unchanged.
- **Fail-closed:** a failed decode, a missing symbol or a missing index entry gives no site.

**Complexity.** O(I + U + B + J log J) time and O(I + U + B + J) space per function, for:
- `I` decoded instructions;
- `U` decoded `u16` operands;
- `B` borrowed paths;
- `J` jump targets.

Planning is linear in total code size apart from the jump-target sort. No step traverses all instructions or all candidates per candidate.

**Scope.** Indexes live only inside `plan_function` during plan construction. The plan stays execution-scoped, as in §7.

**Oracle.** The pre-FIX1 planner is kept verbatim as the test-only `plan_function_reference`. Tests compare the exact site maps on:
- a source corpus;
- shaped bytecode (register reuse, `rX == rV`, operand collisions, jumps into the triple, barriers, backward jumps, intervening accesses);
- 3,000 deterministic random instruction streams.

**Rollback.** Revert the FIX1 commit. The previous planner returns, with identical accepted sites.

## 8. Exception-safety guarantees (revalidated on the current code)

| Path | Validated before the transfer | Commit point | What can fail afterwards |
|---|---|---|---|
| `a` (`LOAD_VAR` take) | symbol lookup (same `UnknownVariable` if absent) | `locals.remove` | the Steps quota on any later instruction; any error while computing `rV` (calls: Calls/Frames/StackDepth/Registers quotas, traps, type errors) |
| `b` (`SEQUENCE_PUSH` take) | `take_reg` has the same errors and order as `get_reg` | `take_reg(rX)` | the sequence type check, `get_reg(rV)`, `write_reg(rZ)` (Registers quota) |
| `c` (`STORE_VAR` take) | the exact-PC write check ran before the instruction (unchanged) | `take_reg(rZ)` → `locals.insert` | nothing; borrow activation is unchanged and runs after the write, as before |

**Why a failure inside a triple is unobservable** (verified against the current code, not inherited from research):
1. Every `RuntimeError` in the dispatch loop propagates with `?`. There is no catch or recover construct in the language or the VM; the only `unwrap_or` in the loop formats an error message.
2. No public function takes or returns a `VM` or `Frame`. The dispatch loop and frame admission are private, and every public `run_*` entry creates its VM internally and returns only `Result` or `MeteredRun` counters.
3. Test-only trap observation records status, return value and trap text, never locals.

**Equivalence.** The instruction stream is unchanged, so Steps are charged identically, and Calls, Frames, StackDepth and Registers checks are untouched. The errors raised at `a`, `b` and `c` are the same errors in the same order. The sweep tests compare planned and unplanned execution under every Steps limit, so a failure lands on every instruction, including inside a consumed triple.

## 9. Interaction with OWN0 ownership metadata

- **Unchanged:** the OWN0 path discipline (borrow and write events, `check_write_execution_site` before each instruction, `StoreVarSite` activation after the write commits).
- **No interaction by construction:** consumption is not applied when a borrowed path is rooted at the consumed local (C6).
- **Logical aliasing is still impossible.** Physical sharing is invisible because shared buffers are immutable. The normative claims of the SSF-08 ownership decision stay true.
- **Proposed amendment** (for separate OWNER approval; `docs/roadmap/**` is outside this change's scope): SSF-08's descriptive sentence "There is no `Rc`/`Arc`/pointer type anywhere in `crates/sm-vm/`" is no longer accurate. Suggested replacement:
  > `sm-vm` shares immutable sequence buffers (`Arc`, copy-on-write); no value is ever mutated while shared, so there is still no shared mutable memory and no observable aliasing.

## 10. Thread safety

- `Value`, `ClosureValue`, `Frame` and `VM` remain `Send + Sync` (compile-time guard).
- The VM itself is still a single-threaded sequential interpreter.
- The consumption plan is per execution and owned, so concurrent executions in separate threads cannot interfere.

## 11. Determinism and quotas

- No quota value, charge point or order changes.
- `Arc` reference counts never influence control flow or results. They only decide whether an update copies first, and both branches produce the same logical value.
- Steps, Calls, peak Frames, effect calls and error texts are identical to the baseline in every qualification run (§12).

## 12. Performance evidence (production build)

Full tables, generated from raw data, are in `docs/architecture/vm_shared_value_storage_and_verified_consumption_evidence.md`.

**Setup.** Production branch (`impl`) against the pristine `main` VM, on identical verified SemCode. Median warm time over 7 interleaved rounds, each platform measured separately:

| Workload | Windows 11 (native) | Ubuntu 24.04 (WSL2) |
|---|---|---|
| Parser `b_parse_8` / `16` / `32` | ×7.8–8.7 / ×15.6–18.5 / ×32–52 | ×7.7–9.0 / ×17.8–18.7 / ×32.6–39.5 |
| Evaluator `b_eval_loop_10…80` | ×1.6–1.8 | ×1.5–2.1 |
| Lexer 720 B → 46 KB | ×2.2 → ×114 | ×2.5 → ×132 |
| Accumulation 1,000 → 100,000 items | ×13 → ×1,026 (100k: reference INCOMPLETE) | ×15 → ×1,336 |
| Not optimized (`map_set_loop`, `record_with_loop`) | ×0.9 / ×1.0 | ×0.88 / ×1.1 |

**Peak memory.**

| Workload | Windows | Ubuntu (WSL2) |
|---|---|---|
| Lexer 23 KB | 54.9 → 13.5 MB | 52.1 → 19.7 MB* |
| Lexer 46 KB | 102.7 → 20.9 MB | 100.5 → 20.0 MB |
| Parser `b_parse_32` | 16.9 → 6.3 MB | — |
| Accumulation 100,000 | — | 71.4 → 19.4 MB |

\* The WSL2 `ru_maxrss` floor is about 19.5 MB per process.

**Correctness, both platforms:**
- 224 fixtures;
- 1,819 failure-sweep points;
- alternating programs in one process;
- 120 bytecode mutations;
- repeated runs.

There is no semantic mismatch with the reference. The only exception is a reference timeout on Windows (`append_100000`), whose result matches the recorded pristine result and passes on Ubuntu.

**Copy-on-write counters** are identical to the P3-R1 research prototype:
- lexer: 7,679 of 7,681 updates unique;
- parser: 318 of 321;
- evaluator: 101 of 347, the rest genuinely shared and copied.

**Plan construction:** at most 3 ms per execution.

**Open item.** A possible slowdown of up to about 8% on the Map-heavy microbenchmark `map_set_loop`, which this change does not optimize. It is not resolvable above measurement noise; see the evidence document.

## 13. Compatibility and migration

- **Source compatibility:** see §4. The only public change is the `Value::Sequence` payload type, and the golden snapshot is updated in the same change.
- **Wire and artifact compatibility:** unchanged. No SemCode, opcode or revision change.
- **Rollback:** revert the two commits (`perf(sm-vm): share sequence storage copy-on-write (Arc)` and `perf(sm-vm): verified consumption of sequence accumulators`). Nothing else depends on them.

## 14. Remaining limitations

- **Copy-on-write covers `Sequence` only.** Text, Bytes, Tuple, Record, Map, Adt and closure captures still copy deeply on logical copy. In the lexer, quadratic **text** copying (about 2.18 GB at 23 KB in research) is now the dominant remaining cost.
- **The plan is conservative.** In research fixtures it accepted about 79% of candidate sites. Rejected: `return push(…)`, self-referencing values such as `push(xs, len(xs))`, and register-number collisions.
- **Planning still runs before Steps are charged** (FIX1 made it linear, not metered). On the largest compiler-generated case it adds about 30% to startup over pristine (§7.1; evidence document, FIX1 section).
- **The hand-crafted register-reuse planning case was never executed (UNTESTED).** FIX1 removes the per-candidate scans that hypothesis relied on.
- **Transaction safety assumes terminal errors.** A future catch or recover feature would require an explicit restore strategy for consumed locals.
- **Platforms:** qualification ran on Windows 11 native and Ubuntu 24.04 under WSL2. Native Linux evidence is limited to the curator's independent Debian 13 run (research scope).
