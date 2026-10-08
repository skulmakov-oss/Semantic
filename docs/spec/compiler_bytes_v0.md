# Compiler Bytes Buffer Model and Byte Value Contract v0

Status: implemented and qualified under SHF-2A2 (#2015) of #1910.
SHF-2A1 (contract) and SHF-2A2 (implementation/qualification) are COMPLETE.
Not part of `v1.2.0`.

Contract ID: `semantic.compiler.bytes/0.1`

Parent: #1910 (SHF-2 — Compiler-grade Bytes and Endian Helpers) · Checkpoint issue: #2012

Adjacent stable library contract: `semantic.foundation.std/0.1`.

`semantic.compiler.bytes/0.1` is an additive compiler-foundation contract and does not amend or
widen `semantic.foundation.std/0.1`.

## 1. Authority and scope

This document freezes the minimum deterministic **byte value domain** and **in-memory binary buffer
contract** required by a future self-hosted Semantic compiler to accumulate, slice and inspect raw
binary artifacts (such as SemCode binaries).

This contract covers:
- The byte value domain and source-language representation.
- The `Bytes` value family and its semantic identity.
- Raw-byte preservation invariants (including non-UTF-8 octet sequences).
- The six core in-memory deterministic `Bytes` operations.
- The persistence, immutability and indexing rules.
- Boundaries, non-goals and deferred capabilities (endian helpers, file I/O, etc.).
- Qualification plan for subsequent implementation (SHF-2A2).

This document is **contract only**. It changes no compiler frontend, typechecker, IR, SemCode
emitter, verifier or VM execution code.

## 2. Byte value domain and carrier

1. **Domain:** A byte value is an element of the octet domain:
   $$\mathbb{B} = \{ n \in \mathbb{N} \mid 0 \le n \le 255 \}$$
   Every integer in $0 ..= 255$ is a valid byte. Values outside this range ($< 0$ or $> 255$) are
   not valid byte values.
2. **Carrier type:** The source-level carrier for a byte value is **`u32`** (`semantic.compiler.u32/0.1`).
   - There is **no new `u8` numeric type family** introduced by this contract.
   - Introducing a separate `u8` numeric type family would widen the language type system, arithmetic
     hierarchy and conversion rules, which is explicitly non-goal for SHF-2A.
   - Byte values passed to `Bytes` operations or returned from `Bytes` operations are represented as
     plain, unmeasured `u32` integers constrained to the domain $[0, 255]$.
3. **Out-of-range enforcement:** When a `u32` value outside $[0, 255]$ is supplied to an operation
   expecting a byte, the operation fails deterministically (returning `None` or failing closed according
   to its signature, §7.3). No truncation, modular reduction or wrapping (e.g. `val & 0xFF`) is
   permitted.

## 3. The `Bytes` semantic identity

1. **Distinct value family:** `Bytes` is a distinct, language-owned binary value family.
   - `Bytes` is **NOT** `text` (`semantic.compiler.text/0.1`).
   - `Bytes` is **NOT** `Sequence(u32)` or any alias of a generic sequence.
   - `Bytes` is **NOT** an alias or view of string memory.
2. **Binary domain:** A `Bytes` value is a finite, ordered sequence of octets:
   $$b = [o_0, o_1, \dots, o_{L-1}], \quad o_i \in [0, 255]$$
3. **Separation from text:**
   - `text` is strictly validated UTF-8 text (`compiler_text_v0.md` §2).
   - `Bytes` holds raw octets with **no UTF-8 validation**, no Unicode scalar interpretation, no
     character encoding assumptions and no normalization.
   - There is no implicit conversion between `text` and `Bytes`. Conversion requires explicit,
     fallible operations defined in separate contracts.

## 4. Raw-byte preservation invariant

1. **Preservation:** A `Bytes` value stores and preserves arbitrary 8-bit octet sequences without
   modification, replacement, transcoding or filtering.
2. **Boundary octets:** All 256 octet values ($0\text{x}00$ through $0\text{xFF}$ inclusive) are
   valid elements. Specifically, $0\text{x}00$ (NUL), $0\text{x}7F$ (DEL), $0\text{x}80$ and
   $0\text{xFF}$ are normal data bytes and have no terminator or sentinel semantics.
3. **Invalid UTF-8 support:** A `Bytes` buffer may contain octet sequences that are completely invalid
   or illegal under UTF-8 (such as isolated continuation bytes, overlong encodings, or invalid lead
   bytes).
   - **Normative golden test vector:**
     The raw octet sequence:
     $$[0\text{xFF}, 0\text{x}00, 0\text{xC0}, 0\text{xAF}]$$
     MUST be accepted, preserved, concatenated, indexed and sliced without error, transformation or
     decoding traps.
4. **No hidden normalization:** The byte sequence observed by indexing or slicing must be byte-for-byte
   identical to the sequence constructed.

## 5. Persistence, immutability and memory model

1. **Value semantics:** `Bytes` values have pure **value semantics** and are persistent (immutable).
2. **No observable in-place mutation:** Operations such as push or extend produce logically new
   `Bytes` values. No operation can mutate an existing `Bytes` value in place or observe shared buffer
   mutation across references.
3. **Implementation freedom:** The runtime implementation may employ persistent data structures,
   reference counting or copy-on-write buffers internally, provided that value-semantic purity and
   isolation are strictly preserved.
4. **Determinism:** Buffer representations and observable behaviors must be identical across all target
   platforms, memory allocators and optimization levels (`O0` vs optimized).

## 6. Indexing and length model

1. **Unit:** The indexing unit is always a **single byte (octet)**. There are no multi-byte, scalar, or
   word-aligned default indexing strides in core `Bytes`.
2. **Type:** Indices, offsets and lengths are plain **`u32`** values (`semantic.compiler.u32/0.1`).
3. **Range:** For a `Bytes` instance with length $L$:
   - Valid element indices are $0 \le \text{index} < L$.
   - Valid slice endpoints are $0 \le \text{start} \le \text{end} \le L$.
   - Slices use half-open intervals: $[ \text{start}, \text{end} )$.
4. **Length limit:** A `Bytes` buffer has a maximum length of $\text{u32::MAX}$ ($4,294,967,295$ bytes).
   Any construction or extension that would cause the length to exceed $\text{u32::MAX}$ must fail
   closed deterministically with an `ArithmeticOverflow` trap (`R0004`). Silent truncation, wrap-around,
   or host `usize` overflow is strictly forbidden.

## 7. The six core `Bytes` operations

The minimum core surface consists of exactly six deterministic operations:

| Contract ID | Conceptual Signature | Description |
|---|---|---|
| `BYTES-EMPTY` | `bytes_empty() -> Bytes` | Returns an empty `Bytes` buffer of length `0`. |
| `BYTES-LEN` | `bytes_len(bytes: Bytes) -> u32` | Returns the number of octets in the buffer. |
| `BYTES-PUSH` | `bytes_push(bytes: Bytes, value: u32) -> Option(Bytes)` | Appends a single byte if `value <= 255`, else returns `None`. |
| `BYTES-EXTEND` | `bytes_extend(left: Bytes, right: Bytes) -> Bytes` | Appends buffer `right` to `left`. Traps on overflow. |
| `BYTES-GET` | `bytes_get(bytes: Bytes, index: u32) -> Option(u32)` | Returns octet at `index` as `u32`, or `None` if out of bounds. |
| `BYTES-SLICE` | `bytes_slice(bytes: Bytes, start: u32, end: u32) -> Option(Bytes)` | Returns sub-slice $[start, end)$, or `None` if invalid. |

### 7.1 `BYTES-EMPTY`
- **Signature:** `bytes_empty() -> Bytes`
- **Semantics:** Produces an immutable, empty `Bytes` value.
- **Invariants:** `bytes_len(bytes_empty()) == 0u32`.

### 7.2 `BYTES-LEN`
- **Signature:** `bytes_len(bytes: Bytes) -> u32`
- **Semantics:** Returns the exact number of octets currently contained in the buffer.
- **Invariants:** Returns `0u32` for an empty buffer; always $\le \text{u32::MAX}$.

### 7.3 `BYTES-PUSH`
- **Signature:** `bytes_push(bytes: Bytes, value: u32) -> Option(Bytes)`
- **Semantics:**
  - If `value <= 255u32`:
    - If `bytes_len(bytes) == u32::MAX`: traps with `ArithmeticOverflow` (`R0004`).
    - Otherwise: returns `Some(b')` where $b'$ is a new `Bytes` value of length $L + 1$ with octet `value`
      at index $L$.
  - If `value > 255u32`:
    - Returns `None`. The value is outside the byte domain $\mathbb{B}$.
- **Invariants:**
  - Never truncates `value` (e.g. `bytes_push(b, 256u32)` does not push `0`).
  - Pure operation: original buffer `bytes` is unaffected.

### 7.4 `BYTES-EXTEND`
- **Signature:** `bytes_extend(left: Bytes, right: Bytes) -> Bytes`
- **Semantics:**
  - Let $L_1 = \text{bytes\_len}(left)$ and $L_2 = \text{bytes\_len}(right)$.
  - If $L_1 + L_2 > \text{u32::MAX}$ (checked addition via `semantic.compiler.u32/0.1`): traps with
    `ArithmeticOverflow` (`R0004`).
  - Otherwise: returns a new `Bytes` value of length $L_1 + L_2$ containing the concatenation of all
    octets in $left$ followed by all octets in $right$.
- **Invariants:**
  - Associativity: both constructions (`bytes_extend(bytes_extend(a, b), c)` and `bytes_extend(a, bytes_extend(b, c))`)
    produce `Bytes` values with exactly the same ordered octet sequence.
  - Identity: extending a `Bytes` value with empty `Bytes` on either side (`bytes_extend(a, bytes_empty())`
    or `bytes_extend(bytes_empty(), a)`) preserves exactly the original ordered octet sequence of `a`.

### 7.5 `BYTES-GET`
- **Signature:** `bytes_get(bytes: Bytes, index: u32) -> Option(u32)`
- **Semantics:**
  - If `index < bytes_len(bytes)`: returns `Some(octet)` where `octet` is the byte value at `index`
    represented as a `u32` in $0 ..= 255$.
  - If `index >= bytes_len(bytes)`: returns `None`.
- **Invariants:**
  - For empty buffer, returns `None` for all `index`.
  - Never returns `Some(v)` where $v > 255$.

### 7.6 `BYTES-SLICE`
- **Signature:** `bytes_slice(bytes: Bytes, start: u32, end: u32) -> Option(Bytes)`
- **Semantics:**
  - Valid boundary condition:
    $$0 \le start \le end \le \text{bytes\_len}(bytes)$$
  - If the boundary condition is satisfied: returns `Some(sub)` containing the $end - start$ octets
    from index $start$ up to (excluding) $end$.
  - If $start > end$ or $end > \text{bytes\_len}(bytes)$: returns `None`.
- **Invariants:**
  - `bytes_slice(b, 0u32, bytes_len(b))` returns `Some(b)`.
  - For any valid $i \le \text{bytes\_len}(b)$, `bytes_slice(b, i, i)` returns `Some(bytes_empty())`.
  - Slicing operates at arbitrary byte granularity; unlike `text_slice`, there are no UTF-8 character
    boundary restrictions.

## 8. Deferred comparison and non-admitted operations

SHF-2A1 freezes only the six core operations defined in §7. This contract does **NOT** admit or
freeze:
- Source-level `Bytes` equality operators (`==`, `!=`).
- Source-level `Bytes` ordering operators (`<`, `<=`, `>`, `>=`).
- Hashing of `Bytes` values.
- Map-key eligibility for `Bytes`.
- Substring/pattern search, `contains` or finding methods within `Bytes`.
- Prepend, pop, insert, or remove operations.
- Mutable-buffer or in-place modification APIs.
- Capacity, allocation, or reserve manipulation APIs.

Exact ordered-octet identity is mathematically defined by the finite sequence of octets
($A = B \iff \text{bytes\_len}(A) == \text{bytes\_len}(B) \land \forall i, \text{bytes\_get}(A, i) == \text{bytes\_get}(B, i)$),
but source-level comparison, operator admission, and search/container mechanics are explicitly
deferred to separate future contracts. No builtin comparison functions or operator overloads are
admitted or introduced by this contract.

## 9. Non-goals and deferred capabilities

This contract strictly excludes the following, which are explicitly deferred to subsequent checkpoints:

1. **SHF-2B Endian helpers:**
   - Multi-byte encoding/decoding (`u16`, `u32`, `u64`, `i32`) in little-endian or big-endian order
     (e.g. `bytes_push_u32_le`, `bytes_get_u32_le`) are deferred to SHF-2B.
2. **SHF-2C Binary filesystem I/O:**
   - Reading binary files into `Bytes` or writing `Bytes` directly to disk (`fs.read_bytes`,
     `fs.write_bytes`) are deferred to SHF-2C.
3. **SHF-3B Bitwise operators:**
   - Bitwise arithmetic (`&`, `|`, `^`, `~`, `<<`, `>>`) on `u32` or `Bytes` is deferred to SHF-3B.
4. **SHF-9 SemCode binary format encoding:**
   - Serializing compiler AST/IR structures into binary SemCode modules is deferred to SHF-9.
5. **SHF-10 Compiler lexer:**
   - Full lexical scanner implementation is deferred to SHF-10.
6. **New source types / numeric families:**
   - No `u8` or `byte` keyword type is added to the source language.
7. **In-place mutable arrays:**
   - No mutable raw pointers, slices or shared memory buffers.

## 10. Qualification plan (SHF-2A2)

Subsequent implementation under SHF-2A2 must qualify against the following test matrices across the
complete verified pipeline (`smc` $\to$ `sm-verify` $\to$ `sm-vm`) at both `O0` and `O1`:

1. **Empty buffer:**
   - `bytes_empty()` has length `0`.
   - `bytes_get(empty, 0)` returns `None`.
   - `bytes_slice(empty, 0, 0)` returns `Some(empty)`.
2. **Byte domain enforcement:**
   - `bytes_push(b, 0)` through `bytes_push(b, 255)` succeed (`Some`).
   - `bytes_push(b, 256)` and `bytes_push(b, 4294967295)` return `None`.
3. **Raw octet and invalid UTF-8 preservation:**
   - Construct buffer with $[0\text{xFF}, 0\text{x}00, 0\text{xC0}, 0\text{xAF}]$.
   - Verify `bytes_len == 4`.
   - Verify `bytes_get` returns exact values `[255, 0, 192, 175]`.
   - Verify slicing $[1, 3)$ yields $[0\text{x}00, 0\text{xC0}]$.
4. **Concatenation and slicing:**
   - Concatenate non-empty buffers; verify contents and length.
   - Slicing out of bounds ($start > end$ or $end > len$) returns `None`.
5. **Boundary limits:**
   - Extending beyond $\text{u32::MAX}$ traps with `ArithmeticOverflow`.
6. **Optimizer parity and determinism:**
   - Identical execution results between `O0` and `O1`.
   - Constant folding (if implemented) preserves traps and values identically.
