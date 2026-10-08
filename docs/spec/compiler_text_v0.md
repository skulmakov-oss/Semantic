# Compiler Text Inspection Contract v0

Status: normative semantics frozen (SHF-1A, #2002); SHF-1B (#2004) implements and qualifies
the seven operations on `main` (§11). Landed on `main` is not published stable: this surface is
not part of `v1.2.0`. SHF-1 overall remains open — the §9.4 lexer probe is blocked on SHF-3.

Contract ID: `semantic.compiler.text/0.1`

Parent: #1910 (SHF-1 — Compiler-grade Text) · Checkpoint issue: #2002

Base library contract: `semantic.foundation.std/0.1` (`std.text`)

## 1. Authority and scope

This document freezes the semantics of a small deterministic **UTF-8 byte-inspection** surface
over the existing `text` type, sufficient for a lexer written in Semantic. It extends `std.text`
(`foundation_stdlib_v0.md`) and does not change any existing text behaviour: literals, equality,
`text + text` and `to_text` keep their current contract.

SHF-1B makes these operations source-visible on `main` under the spellings in §11.
`semantic.foundation.std/0.1` is unchanged and still exposes no indexing, slicing, ordering or
length API (`foundation_stdlib_v0.md`). The new `std.text` revision that carries this contract
into a published library contour, and its identity, remain a separate owner/release decision.

Operation names below (`TEXT-LEN` …) are contract identifiers. Source spellings are fixed in
SHF-1B under the existing builtin naming conventions and must not overload the `Sequence(T)`
builtins `len` / `is_empty`, whose result type and meaning differ.

## 2. The `text` invariant

Every `text` value is a finite sequence of bytes that is **valid UTF-8**. There is no
in-memory `text` state holding invalid UTF-8. The invariant is established at every point where
bytes enter the `text` domain, and nowhere else:

| Ingress | Rule (current authority) |
|---|---|
| Source text literals | Source files are decoded as UTF-8 by the frontend; literals are double-quoted same-line UTF-8 (`types.md` "Text") |
| SemCode string table (`LoadText`) | `sm-format` decodes table strings with strict UTF-8 and rejects invalid bytes (`InvalidUtf8`) before execution |
| Host text reads (`fs.read`, `stdin.read_text`, `args.read`) | Strict UTF-8 decoding; invalid bytes fail as `InvalidInput` (`controlled_application_boundary_v0.md`) |
| `text + text`, `to_text` | Closed over valid UTF-8 by construction |

Every operation in this contract that produces `text` (only `TEXT-SLICE`) is defined so that it
can never manufacture invalid UTF-8 (§6.3). Consequently invalid UTF-8 is **not** a case of these
operations; it is qualified at the ingress boundaries above (§9.3).

Text literal bytes follow the current literal rules, including release limit R2: escape
sequences are not decoded (a backslash followed by `n` is two bytes). Inspection observes the
bytes the literal actually produced and never interprets escapes.

## 3. Index model

- **Unit.** Every index, offset, length and position in this contract is a **byte offset** into
  the UTF-8 encoding. Never a Unicode scalar index, never a grapheme index.
- **Type.** Offsets and lengths are `u32`. This matches existing Semantic practice: diagnostics
  (`diagnostics_machine_schema_v1.md`) and projection source spans already use zero-based,
  half-open UTF-8 byte offsets with `u32` endpoints.
- **Range.** For a text of byte length `L`: byte positions `0 .. L-1` address bytes; the
  **one-past-end** offset `L` addresses no byte (`TEXT-BYTE-AT` gives `None`) but is a valid
  `TEXT-SLICE` bound. No offset greater than `L` is valid anywhere.
- **Scalar boundary.** Offset `i` is a *scalar boundary* iff `i == 0`, `i == L`, or the byte at
  `i` is not a UTF-8 continuation byte (`0x80..=0xBF`).
- **Representable length.** Lengths are reported as `u32`. A text whose byte length exceeds
  `u32::MAX` (4 294 967 295) cannot be described by this contract (§7).

## 4. Determinism and forbidden behaviour

All operations are pure, total over their stated domain, deterministic across platforms, and
depend only on the bytes of their operands. They perform no locale-dependent behaviour, no case
folding, no Unicode normalization, no grapheme segmentation and no host interaction. Two texts
that are canonically equivalent in Unicode but differ in bytes are different for every
operation here. Results never depend on host string-library corner cases: every edge case is
defined below.

## 5. Relationship to other stages

- **SHF-2 (Bytes / binary I/O).** This contract adds no byte type, buffer or binary I/O. A byte
  value read from text is reported as a `u32` in `0..=255`. Reading non-UTF-8 data is not
  possible through `text` and belongs to SHF-2.
- **SHF-3 (integer / index / bit).** This contract only *produces* and *accepts* `u32` values.
  It does not admit `u32` arithmetic, ordering or bit operations. A lexer needs at least
  `u32` increment and ordering to advance and bound an offset; those are SHF-3 work and are a
  recorded dependency of the SHF-1B lexer probe (§9.4), not part of this contract.
- **SHF-10 (lexer).** See §8.

## 6. Operations

### 6.1 `TEXT-LEN` — byte length

`TEXT-LEN(t: text) -> u32`

Returns the number of bytes in the UTF-8 encoding of `t`. Not a scalar count; not a grapheme
count. `TEXT-LEN("") = 0`. Examples: `"A" → 1`, `"é" → 2`, `"€" → 3`, `"🙂" → 4`.

### 6.2 `TEXT-BYTE-AT` — byte at offset

`TEXT-BYTE-AT(t: text, i: u32) -> Option(u32)`

- If `i < TEXT-LEN(t)`: `Some(b)` where `b` is the byte at offset `i`, `0 <= b <= 255`.
- Otherwise (including `i == TEXT-LEN(t)`): `None`.
- No decoding occurs; continuation bytes are returned as they are.

Examples: `"A"` → `[0x41]`; `"é"` → `[0xC3, 0xA9]`; `"🙂"` → `[0xF0, 0x9F, 0x99, 0x82]`.

### 6.3 `TEXT-SLICE` — byte range

`TEXT-SLICE(t: text, start: u32, end: u32) -> Option(text)`

Half-open byte range `[start, end)`. Let `L = TEXT-LEN(t)`.

| Condition | Result |
|---|---|
| `start <= end <= L`, both scalar boundaries | `Some(s)`, `s` = bytes `start .. end` of `t` |
| `start == end` (any scalar boundary `<= L`) | `Some("")` |
| `start = 0`, `end = L` | `Some(t)` |
| `start > end` | `None` |
| `end > L` or `start > L` | `None` |
| `start` or `end` inside a multi-byte scalar | `None` |

Bounds are never rounded, clamped or adjusted to a scalar boundary.

**Why `Option(text)` rather than a trap or `-> text`.** A `text` result must never be invalid
UTF-8 (§2), so an interior-byte bound cannot produce a text. Existing Foundation conventions
reserve runtime traps for failures of already-admitted operations where no total result exists
(`assert(false)`, empty `pop`, division by zero); an out-of-range or non-boundary slice is an
ordinary, data-dependent outcome that a lexer must be able to test for without aborting, and the
language-owned `Option(T)` form (`std.option`) is the existing way to express "no value".
`TEXT-BYTE-AT` and `TEXT-FIND` use the same form.

### 6.4 `TEXT-STARTS-WITH` — prefix test

`TEXT-STARTS-WITH(t: text, prefix: text) -> bool`

`true` iff the bytes of `prefix` equal the first `TEXT-LEN(prefix)` bytes of `t`. Exact,
case-sensitive byte comparison. An empty prefix is a prefix of every text (`true`). A prefix
longer than `t` gives `false`.

### 6.5 `TEXT-ENDS-WITH` — suffix test

`TEXT-ENDS-WITH(t: text, suffix: text) -> bool`

`true` iff the bytes of `suffix` equal the last `TEXT-LEN(suffix)` bytes of `t`. Exact,
case-sensitive byte comparison. An empty suffix is a suffix of every text (`true`). A suffix
longer than `t` gives `false`.

### 6.6 `TEXT-FIND` — first match

`TEXT-FIND(t: text, pattern: text) -> Option(u32)`

- Returns `Some(i)` for the **smallest** byte offset `i` such that the bytes of `pattern` occur
  in `t` starting at `i`; otherwise `None`.
- Matching is exact byte-sequence equality, case-sensitive, no normalization.
- **Empty pattern:** `TEXT-FIND(t, "") = Some(0)` for every `t`, including `""`. (The empty
  sequence occurs at offset 0, consistent with `TEXT-STARTS-WITH(t, "") = true`.)
- Because `t` and `pattern` are both valid UTF-8, any match begins and ends on scalar
  boundaries; a match can never start inside a multi-byte scalar of `t`.
- Only the first match is reported; overlapping occurrences are irrelevant to the result.
- A pattern longer than `t` gives `None`.

### 6.7 `TEXT-IS-EMPTY` — emptiness

`TEXT-IS-EMPTY(t: text) -> bool`

`true` iff `TEXT-LEN(t) == 0`. Whitespace is not empty.

## 7. Errors and bounds

- Wrong argument families or arities are rejected during frontend analysis or lowering, like
  every other builtin (`foundation_stdlib_v0.md` "Errors, effects, and compatibility").
- The `Option` results in §6 are ordinary values, not errors.
- **Length bound gap.** Host text ingress is limited to 16 MiB per call, but the VM imposes no
  maximum on `text` length, and `text + text` can grow a text without bound. If an operation in
  this contract receives a text whose byte length exceeds `u32::MAX`, it must fail with a
  deterministic runtime trap; it must never truncate or wrap a length or offset. Whether to cap
  `text` length globally (affecting `text + text`) is outside this contract and is recorded as an
  open owner decision. SHF-1B uses the existing `RuntimeTrap::ArithmeticOverflow` class for this
  trap (a checked `u32` conversion of a length or offset); no new trap taxonomy is introduced.

## 8. No host-side lexing

These operations are primitive text mechanics only. Their existence grants no permission for the
host, VM or any Rust helper to classify Semantic tokens, recognize keywords, skip comments, parse
numbers, interpret escapes as lexer logic, choose token boundaries or make grammar decisions.
All of that is Semantic compiler code (SHF-10). An implementation that adds any such behaviour
behind these operations violates this contract.

## 9. Qualification schema for SHF-1B

### 9.1 Path

Every vector runs through the full trusted pipeline:
`Semantic source → typecheck → IR → SemCode → sm-verify → sm-vm → observable result`. Direct
calls into Rust helpers are not evidence.

### 9.2 Vectors

| Area | Required cases |
|---|---|
| Texts | `""`; ASCII; a 2-byte scalar (`é`); a 3-byte scalar (`€`); a 4-byte scalar (`🙂`); mixed ASCII + multi-byte |
| `TEXT-LEN` | each text above |
| `TEXT-BYTE-AT` | first byte; last byte; each byte of every multi-byte scalar; one-past-end (`None`); far out of range (`None`); empty text (`None`) |
| `TEXT-SLICE` | empty slice (`start == end`); full slice; valid scalar-boundary slice inside mixed text; `start` inside a scalar (`None`); `end` inside a scalar (`None`); `start > end` (`None`); `end > L` (`None`); `start > L` (`None`); slice of `""` |
| `TEXT-STARTS-WITH` / `TEXT-ENDS-WITH` | empty prefix/suffix; hit; miss; longer than text; multi-byte prefix/suffix |
| `TEXT-FIND` | match at start; middle; end; miss; multi-byte pattern; empty pattern on non-empty and on empty text; pattern longer than text |
| `TEXT-IS-EMPTY` | `""`; whitespace-only text |
| Negative (compile time) | wrong argument family and arity for each operation |

### 9.3 Invalid UTF-8

Because `text` cannot hold invalid UTF-8, invalid input is qualified at ingress, not in the
operations: a SemCode artifact whose string table holds invalid UTF-8 is rejected before
execution; a host text read of invalid bytes fails with `InvalidInput`. SHF-1B records these as
negative vectors of the ingress paths it relies on: the string-table case in
`tests/shf1b_compiler_text_qualification.rs`, and the host case in the existing
`smc-cli` test `application_host::tests::text_budget_and_malformed_utf8_fail_closed`. SHF-1B adds
no ingress path of its own.

### 9.4 Lexer probe

SHF-1 completion requires a deliberately tiny Semantic program that scans a representative
source sample using only these primitives and no host-side tokenization: ASCII whitespace,
ASCII identifier byte classes, single-character punctuation, and correct advancement over
multi-byte UTF-8 scalars. It is not a Semantic lexer and implements no Semantic grammar. Its
offset advancement and bounds checks require `u32` increment and ordering, so the probe depends
on SHF-3 admitting those operations (or an explicitly authorized interim decision); this
dependency is recorded, not resolved, by this contract.

## 10. Non-goals

Regular expressions; locale behaviour; case folding; Unicode normalization; grapheme or scalar
segmentation APIs; scalar-index APIs; text ordering; formatting or interpolation; escape
decoding; byte values or buffers (SHF-2); general `u32` arithmetic, ordering or bit operations
(SHF-3); the compiler lexer (SHF-10); any change to existing `std.text` behaviour.

## 11. SHF-1B implementation status

Qualified on `main` by SHF-1B (#2004); not part of the published `v1.2.0` contour.

| Contract operation | Source spelling | Signature |
|---|---|---|
| `TEXT-LEN` | `text_len` | `(text) -> u32` |
| `TEXT-BYTE-AT` | `text_byte_at` | `(text, u32) -> Option(u32)` |
| `TEXT-SLICE` | `text_slice` | `(text, u32, u32) -> Option(text)` |
| `TEXT-STARTS-WITH` | `text_starts_with` | `(text, text) -> bool` |
| `TEXT-ENDS-WITH` | `text_ends_with` | `(text, text) -> bool` |
| `TEXT-FIND` | `text_find` | `(text, text) -> Option(u32)` |
| `TEXT-IS-EMPTY` | `text_is_empty` | `(text) -> bool` |

- The spellings are reserved language-owned builtin names: a user function cannot take them.
- They lower to the existing name-dispatched `Call` instruction; no SemCode opcode or format
  change was made. Verifier admission requires `CAP_TEXT_VALUES` for a bare call, like
  `to_text`.
- `Option` results are the canonical runtime `Option` value (`None` tag 0, `Some` tag 1), the
  same carrier `Option::Some` / `Option::None` produce, and are matched by ordinary `match`.
- Qualification: `tests/shf1b_compiler_text_qualification.rs` runs every §9.2 vector from source
  through `sm-verify` and verified `sm-vm` execution, plus wrong-arity / wrong-family diagnostics,
  reserved-name rejection, the capability admission check, byte-identical recompilation, and
  the string-table ingress case. The `u32::MAX` boundary of §7 is exercised synthetically in
  `sm-vm` unit tests, because a >4 GiB text is not allocated in CI.
- The §9.4 lexer probe is **not** executed: it is blocked on SHF-3 (`u32` increment and ordering).
