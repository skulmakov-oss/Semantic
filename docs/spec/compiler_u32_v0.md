# Compiler u32 Arithmetic and Ordering Contract v0

Status: normative semantics frozen (SHF-3A1, #2006); implemented and qualified on `main` by
SHF-3A2 (#2008, PR #2009, §13). Not part of `v1.2.0`.

Contract ID: `semantic.compiler.u32/0.1`

Parent: #1910 · Checkpoint issue: #2006

## 1. Authority and scope

This document freezes the semantics of deterministic arithmetic and ordering on **plain,
unmeasured `u32`**, the integer family a compiler written in Semantic uses for byte offsets and
lengths (`compiler_text_v0.md`). It covers:

| Group | Operators |
|---|---|
| Arithmetic | `+` `-` `*` `/` `%` |
| Ordering | `<` `<=` `>` `>=` |
| Equality (existing, unchanged) | `==` `!=` |

SHF-3A2 implements these operators (§13). This does not change `semantic.foundation.std/0.1`,
the `foundation_source_profile_v1.md` stable-candidate profile or the published `v1.2.0` contour:
the surface is additive on current `main`.

Notation: `u32::MAX` in this document means the value `4294967295`; it is not source syntax.
Source vectors spell it `4294967295u32`. `i32::MAX` likewise means `2147483647`.

## 2. Domain

The contract applies only when **both** operands have the plain type `u32`. It does not widen:

- measured numerics (`u32[unit]`); unit-carrying arithmetic stays rejected as today;
- mixed-family arithmetic, ordering or equality (§6);
- implicit or contextual conversion, including literal coercion: an unsuffixed integer
  literal is `i32` in expression position, so `u32` operands are written explicitly
  (`i + 1u32`, `i < len`).

Integer conversions between families are not part of this contract.

## 3. Arithmetic

| Operation | Signature | Result |
|---|---|---|
| Addition | `u32 + u32 -> u32` | exact sum, else trap |
| Subtraction | `u32 - u32 -> u32` | exact difference, else trap |
| Multiplication | `u32 * u32 -> u32` | exact product, else trap |
| Division | `u32 / u32 -> u32` | unsigned quotient, rounded toward zero |
| Remainder | `u32 % u32 -> u32` | unsigned remainder, `a == (a / b) * b + (a % b)` |

There is no unary minus on `u32`: `-x` for `x: u32` is rejected at compile time.

### 3.1 Checked overflow and underflow

For `+`, `-` and `*`, the result is the exact mathematical result when it lies in
`0 ..= u32::MAX`. Otherwise execution fails with the existing deterministic runtime trap
`RuntimeTrap::ArithmeticOverflow` (diagnostic code `R0004`):

| Expression | Outcome |
|---|---|
| `4294967295u32 + 1u32` | `ArithmeticOverflow` |
| `0u32 - 1u32` | `ArithmeticOverflow` (underflow uses the same trap) |
| `4294967295u32 * 2u32` | `ArithmeticOverflow` |

Forbidden: wrapping, saturation, host- or build-dependent behaviour (including Rust
debug-versus-release overflow modes), panics and undefined behaviour.

This **intentionally differs** from the frozen `i32` policy, under which `+`, `-` and `*` wrap
on two's-complement overflow (`foundation_source_profile_v1.md`). The difference is normative:
compiler offsets and lengths must fail closed rather than silently wrap to a different
position.

### 3.2 Division and remainder by zero

`a / 0u32` and `a % 0u32` fail with the existing `RuntimeTrap::DivisionByZero` (diagnostic code
`R0003`) for every `a`, including `0u32`. For a non-zero divisor, unsigned division and
remainder cannot overflow; there is no other trapping case (`u32` has no counterpart to
`i32::MIN / -1`). No new trap taxonomy is introduced.

## 4. Ordering

| Operation | Signature |
|---|---|
| `<` `<=` `>` `>=` | `u32 <op> u32 -> bool` |

Ordering is the ordinary total unsigned numeric order on `0 ..= u32::MAX`. There is no signed
reinterpretation: values above `i32::MAX` are greater than every value at or below it.

| Expression | Result |
|---|---|
| `0u32 < 1u32` | `true` |
| `1u32 < 0u32` | `false` |
| `4294967295u32 > 2147483647u32` | `true` |
| `2147483648u32 > 2147483647u32` | `true` |
| `4294967295u32 >= 4294967295u32` | `true` |

## 5. Equality

`u32 == u32` and `u32 != u32` keep their existing same-family meaning (exact value equality).
This contract lists them as part of the compiler-grade surface and does not redefine them.

## 6. Mixed families

Every arithmetic, ordering or equality operator with one `u32` operand and one operand of a
different family is rejected at compile time, in either operand order. For example:

```text
u32 + i32    i32 + u32    u32 < i32    u32 == i32
```

There is no implicit promotion, no magnitude-based conversion and no host convenience
coercion.

## 7. Optimizer parity

Optimized compilation must have exactly the observable semantics of unoptimized compilation
(`OptLevel::O0` versus optimized levels such as `O1`). Constant folding of `u32` operations must
preserve:

- every result value;
- every overflow and underflow trap;
- every division- and remainder-by-zero trap;
- ordering results.

An optimizer may never turn a trapping operation into a value, a wrapped result or a saturated
result; when it cannot fold an operation without changing its outcome, it must leave it to
runtime.

## 8. Determinism

For identical source and inputs, source → typecheck → IR → SemCode → verifier → VM produces the
same result or the same trap on every platform. Results must not depend on host `usize` width,
CPU overflow behaviour, Rust build profile or platform ABI.

## 9. Qualification plan for SHF-3A

SHF-3A must qualify every vector below through the full verified pipeline (source compiled to
SemCode, admitted by `sm-verify`, executed by `sm-vm`), at both `O0` and an optimizing level.

**Positive (value):**

| Group | Vectors |
|---|---|
| `+` | `0 + 0`, `1 + 1`, `u32::MAX + 0` |
| `-` | `1 - 1`, `u32::MAX - 1` |
| `*` | `0 * u32::MAX`, `1 * u32::MAX` |
| `/` | `0 / 1`, `7 / 3 = 2`, `u32::MAX / 1`, `u32::MAX / u32::MAX = 1` |
| `%` | `0 % 1`, `7 % 3 = 1` |
| Ordering | all four operators around `0`, `1`, `i32::MAX`, `i32::MAX + 1`, `u32::MAX` |

**Traps:** `u32::MAX + 1` and `0 - 1` and `u32::MAX * 2` → `ArithmeticOverflow`;
`x / 0` and `x % 0` → `DivisionByZero`.

**Negative (compile time):** mixed numeric families (§6, both orders); measured `u32`
arithmetic; wrong operand families; unary `-` on `u32`.

**Determinism:** repeated compilation produces byte-identical SemCode; `O0` and optimized builds
produce the same result or the same trap for every vector.

## 10. Relationship to the SHF-1 lexer probe

The SHF-1 §9.4 lexer probe (`compiler_text_v0.md`) needs at least `offset + 1u32` and
`offset < text_len(source)`. SHF-3A implementation therefore unblocks the probe's mechanical
requirements. This contract does not run the probe. SHF-3A has landed, and the formerly blocked
probe is qualified by SHF-1C (#2010) on its PR branch (`compiler_text_v0.md` §9.4).

## 11. Implementation strategy (not frozen here)

Contract-time discovery at the base of this document found no existing `u32` arithmetic or
ordering SemCode opcodes (for example `AddU32` or `CmpU32`); current `i32` arithmetic uses
dedicated `i32` opcodes, and constant folding (`crates/sm-ir/src/passes/crystalfold.rs`) is an
independent implementation of the same policy. The SHF-3A implementation task must determine
independently whether correct semantics require new SemCode opcodes, existing generic numeric
instructions or another admitted mechanism. If new opcodes or any SemCode format change is
required, that task needs explicit R3 owner authorization. This contract changes no frontend,
IR, emitter, format, verifier or VM code.

## 12. Non-goals

Bitwise operators and shifts (`&` `|` `^` `~` `<<` `>>`, SHF-3B); integer conversions between
families; measured `u32` arithmetic; `u32` literal coercion; byte values or buffers (SHF-2); the
compiler lexer (SHF-10); any SemCode format or opcode change; any change to `i32` semantics,
`v1.2.0`, `semantic.foundation.std/0.1` or the C0 pin.

## 13. SHF-3A2 implementation status

Implemented and qualified on `main` (#2008, PR #2009); not part of `v1.2.0`.

| Operation | Opcode | Byte | Minimum header |
|---|---|---|---|
| `a < b`, and `a > b` as `b < a` | `CmpU32Lt` | `0x24` | `SEMCOD23` (rev 24) |
| `a <= b`, and `a >= b` as `b <= a` | `CmpU32Le` | `0x25` | `SEMCOD23` |
| `a + b` | `AddU32` | `0x26` | `SEMCOD23` |
| `a - b` | `SubU32` | `0x27` | `SEMCOD23` |
| `a * b` | `MulU32` | `0x28` | `SEMCOD23` |
| `a / b` | `DivU32` | `0x29` | `SEMCOD23` |
| `a % b` | `ModU32` | `0x2A` | `SEMCOD23` |

- `HEADER_V23` (`SEMCOD23`, revision 24) inherits the `SEMCOD22` capabilities unchanged; no
  capability bit is added. The compiler emits it only for programs that use this opcode family;
  every other artifact keeps the `SEMCOD22` floor. A `SEMCOD22` artifact carrying any of the
  seven opcodes is rejected by the verifier (`Opcode::minimum_semcode_revision`).
- `==` / `!=` still lower to the existing `CmpEq` / `CmpNe`.
- The VM uses checked `u32` operations: `+ - *` outside `0 ..= u32::MAX` trap
  `ArithmeticOverflow`; `/ %` by zero trap `DivisionByZero`. Comparisons use the `u32` values
  directly, with no signed or widened reinterpretation.
- CrystalFold folds a `u32` operation only when its checked result exists; overflow,
  underflow and zero-divisor cases are left to runtime, so `O0` and `O1` agree on every
  result and trap.
- Typecheck admits exactly `u32` with `u32`; mixed families, measured `u32`, unary `-` and
  unsuffixed (`i32`) literals stay rejected.
- Qualification: `tests/shf3a_u32_qualification.rs` covers every §9 vector at `O0` and `O1`,
  plus header selection, the `SEMCOD22` relabel attack, unknown and truncated instructions,
  frozen opcode bytes, byte-identical recompilation and unchanged `i32`/equality/`match`.
- The SHF-1 §9.4 lexer probe is not run by this change; it is the separate SHF-1C checkpoint.
