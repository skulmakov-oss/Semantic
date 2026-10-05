# PB-05 — IR Safety & SemCode Producer Integrity Decision

Umbrella: #1617 · Base: `4e37e4f577f3978c2b39347cc20a49351d26ae1f`
Issues: #1708, #1711, #1713, #1714, #1715, #1716, #1721, #1723, #1727, #1728, #1729
C1 `89641da8237f4fcefb50cf1958a50e4d4003aea7` / `v1.2.0` are not touched.

## Re-adjudication on current main (`4e37e4f5`)

Every finding was re-run on the base before any change.

| Issue | Reproduction on base | Classification |
|---|---|---|
| #1708 | `alloc` with `next == u16::MAX` panics (debug) / wraps to 0 (release) | STILL_REPRODUCED |
| #1711 | `LoadI32 5; GateWrite; AddI32` folded across the `GateWrite` | STILL_REPRODUCED |
| #1713 | parents `__impl::A::B_C::D` and `__impl::A_B::C::D` both produce `__closure___impl_A_B_C_D_0` | STILL_REPRODUCED |
| #1714 | `semcode.md` lists `SEMCOD15/16/17/20` as an undocumented gap | STILL_REPRODUCED (documentation) |
| #1715 | `semcode.md`, `runtime_ownership.md`, `module_ownership_map.md`, `AGENTS.md` name `sm-ir` as format owner, while the code lives in `sm-format` | STILL_REPRODUCED (documentation) |
| #1716 | the guard snapshots `sm-ir/src/lib.rs` only, so `pub use legacy_lowering::*` and `pub mod passes` hide the real surface | STILL_REPRODUCED |
| #1721 | user `fn random_seed` / `fn qtruth_and` are rejected by the frontend ("is reserved for the language builtin"); the PB-02 `BUILTIN_NAMESPACE` reserves every name lowered as an intrinsic | RESOLVED_BY_PB02 |
| #1723 | `LoadVar r1 "missing"; LoadVar r1 "valid"` → cleanup drops the first, trapping load | STILL_REPRODUCED |
| #1727 | only the 8-byte magic is serialized; capabilities come from `header_spec_from_magic` | SUPERSEDED_BY_CURRENT_CONTRACT |
| #1728 | 1025 functions emit successfully; the decoder then rejects with `ResourceLimit` "too many functions" | STILL_REPRODUCED |
| #1729 | `BoolAnd(false, <i32>)` folds to `LoadBool false`, erasing the VM type trap | STILL_REPRODUCED |

## Answers

1. **What does `sm-ir` own?** The IR (`IrInstr`, `IrFunction`), lowering, the optimizer passes (`sm-ir::passes`) and the SemCode *producer*: it selects a header revision from emitted usage and writes bytes in the `sm-format` layout. It does not own the format.
2. **What does `sm-format` own?** The SemCode format: magics, header specs and capability envelopes (`local_format`), opcodes (`semcode_format`), structural limits and decoding (`semcode_decode`). The dependency is `sm-ir → sm-format`; `sm-format` does not depend on `sm-ir`.
3. **Is register allocation a producer structural invariant or a runtime quota?** It is a producer structural invariant: a register id must be representable in the IR register type (`u16`). `RuntimeQuotas` is a separate downstream execution policy and is not used here.
4. **Exact register exhaustion behavior.** `alloc` uses `checked_add`. Allocating id `u16::MAX` fails with a deterministic `FrontendError` ("IR register allocation exhausted…"), and the counter is left at `u16::MAX`, so it never wraps and never reuses 0. Every call site propagates the error with `?`. This includes `lower_impossible_match_trap`, which became fallible.
5. **What is an optimizer barrier?** Whatever the exhaustive `is_crystalfold_barrier` says: `Label`, `Jmp`, `JmpIf`, `Ret`, `Assert`, `Call`, `ClosureCall`, `GateRead`, `GateWrite`, `PulseEmit`, `StateQuery`, `StateUpdate`, `EventPost`, `ClockRead`, `RngSeed`, `RngNextI32`. The match has no wildcard, so a new `IrInstr` variant does not compile until it is classified. Constant state is cleared before any barrier is processed.
6. **May the optimizer eliminate a possibly trapping instruction without proven operand types?** No. Raw/public IR is untyped, so a rewrite may drop an instruction only when every operand is a known constant of the right type.
7. **Which CrystalFold identities are safe over raw IR?** Only two-sided folds, where both operands are tracked constants of the instruction's own type. The VM wraps `i32`, so the wrapping arithmetic folds match it. All 29 one-sided identity and annihilator rewrites (Bool/Quad/i32/f64/fx) were removed.
8. **How are closure helper names made injective?** Each `::` segment of the parent name is encoded as `<len>s<segment>`, then `_<id>` is appended (`main` → `__closure_4smain_0`). A length-prefixed encoding is injective, and `::` boundaries can no longer merge with `_` inside segments.
9. **Does the producer guarantee globally unique function names?** Yes. `validate_producer_module_limits` rejects a duplicate function name before any byte is emitted. The decoder and VM are not relied on to catch it.
10. **Which frontend builtin names may reach intrinsic lowering?** Only names that `sm-front`'s `BUILTIN_NAMESPACE` marks Reserved: `random_seed`, `random_next_i32`, `qtruth_and`, `qtruth_or`, `qtruth_not`, `qtruth_impl`. `tests/pb05_ir_safety.rs` reads these names from the lowering source and asks the frontend about each. `sm-ir` keeps no registry of its own.
11. **Is #1721 already resolved by PB-02?** Yes. There is direct proof on base and on the branch, and no production change was needed (see 10).
12. **Who owns the public SemCode header family?** `sm-format`. The docs now say so (#1715). `semcode.md` documents `SEMCOD15`/`16`/`17`/`20` from the `sm-format` definitions without inventing semantics (#1714).
13. **What do capability bits mean?** Each one names part of the header revision's admission envelope. The artifact stores no bitset. The envelope is selected by the magic, may be wider than what the artifact uses, and the verifier checks every opcode against it. No format change was made, and no exact-use model was introduced (#1727).
14. **Who owns structural decoder maxima?** `sm-format` (`sm_format::semcode_decode::MAX_*`). `sm-ir` and its tests import those constants rather than copying the numbers.
15. **Must emitter success imply decoder acceptance for producer-controlled dimensions?** Yes. The producer rejects input over `MAX_FUNCTIONS`, `MAX_STRING_LEN` (function names and string-table entries), `MAX_STRINGS_PER_FUNCTION` and `MAX_DEBUG_SYMBOLS_PER_FUNCTION`. `MAX_SIGNATURE_PARAMETERS_PER_FUNCTION` was already enforced on base. That is every `MAX_*` in `semcode_decode`, so there is no scope expansion and no new mismatch to report. The tests emit at each limit, decode the result, and check that limit + 1 fails.
16. **What does `public-api-guard` promise for `sm-ir`?** Any change to a `pub` item in `lib.rs`, `error.rs`, `legacy_lowering.rs`, `adt_descriptors.rs`, `hello_ir.rs`, `hello_semcode.rs` or `passes/{mod,crystalfold,cleanup}.rs` changes a reviewed snapshot. That covers the IR data types, compile and emit entry points, `IrModule`/`OptPass`/pass types and the re-exports. It uses the existing per-file extractor; no new parser was added.
17. **What intentionally changes from C1?**
    - Register exhaustion is now an error instead of a panic or wrap.
    - Closure helper names use the injective form.
    - Over-limit and duplicate-name modules are producer errors.
    - CrystalFold no longer folds across the barriers listed in 5, and no longer performs one-sided rewrites.
    - Cleanup keeps every `LoadVar`.
    - Optimized output can be larger for programs that previously relied on one-sided rewrites. Their observable results do not change.
18. **What C1 behavior is unchanged?**
    - The SemCode wire format, magics, header selection order and capability envelopes.
    - Verifier admission and VM semantics.
    - Every well-typed program's results.
    - The frontend builtin policy.
    - C1 and `v1.2.0` themselves.

## Non-goals

- No `sm-format`, `sm-verify` or `sm-vm` change.
- No runtime-quota work.
- No #1987 work.
- No exact-use capability redesign.
- No rewrite of historical records (`semcode_format_authority_split_plan.md` stays as written).
