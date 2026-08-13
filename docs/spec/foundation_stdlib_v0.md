# Semantic Foundation Standard Library v0

Status: SSF-03 candidate contract; not published stable

Contract ID: `semantic.foundation.std/0.1`

Base source contract: `semantic.foundation.source/1.1`

This document is the canonical Standard Library v0 index for the Stable
Foundation contour. It freezes the smallest already-implemented library
surface needed by ordinary Foundation programs. It does not promote the
repository, language, or library to Published Stable.

## Binding model

Foundation Source 1.0 does not admit namespace-qualified `std.*` imports.
Consequently the names below are public documentation family identities, not
import paths. Their callable surface is the listed canonical language-owned
equivalent: a builtin, operator, type, constructor, or match form already
owned by the frontend, lowering, verifier, and VM.

`std.*` source spellings remain reserved and deterministically rejected. A
future importable module facade belongs to SSF-05/SSF-06 and must delegate to
this authority rather than copy it.

| Family identity | State in `semantic.foundation.std/0.1` | Canonical language-owned surface |
|---|---|---|
| `std.core` | Selected | statement-only `assert(bool)` |
| `std.quad` | Selected | `qtruth_and`, `qtruth_or`, `qtruth_not`, `qtruth_impl` |
| `std.math` | Selected | `sqrt`, `abs` |
| `std.text` | Selected | `text + text`, text equality, `to_text(text|bool|i32|u32|quad)` |
| `std.seq` | Selected | `len`, `is_empty`, `contains`, `push`, `prepend`, `pop` over `Sequence(T)` |
| `std.map` | Selected | `map_empty`, `map_contains`, `map_get`, `map_set` over `Map(K, V)` |
| `std.option` | Selected | `Option(T)`, `Option::Some`, `Option::None`, exhaustive match |
| `std.result` | Selected | `Result(T, E)`, `Result::Ok`, `Result::Err`, exhaustive match |
| `std.serde` | Deferred | no selected API or encoding |
| `std.rand` | Selected | `random_seed`, `random_next_i32` |

Only Selected rows are compatibility candidates. Deferred rows are named so
that absence cannot be mistaken for an undocumented promise.

## Selected API semantics

### `std.core`

`assert(condition)` accepts exactly one positional `bool` and is
statement-only. `true` continues execution. `false` produces the canonical
deterministic assertion runtime trap. It performs no host effect.

### `std.quad`

The four `qtruth_*` functions operate on the two independent truth and falsity
evidence planes through the canonical quad truth-map instructions. `N`, `F`,
`T`, and `S` remain distinct. In particular, neither `N` nor `S` is normalized
to `bool`, and no hidden lattice/truth-map substitution is permitted.

For operands `a = (a.t, a.f)` and `b = (b.t, b.f)`, the exact maps are:

- `qtruth_and(a, b) = (a.t & b.t, a.f | b.f)`;
- `qtruth_or(a, b) = (a.t | b.t, a.f & b.f)`;
- `qtruth_not(a) = (a.f, a.t)`;
- `qtruth_impl(a, b) = qtruth_not(a) lattice_join b`, where lattice join is
  plane-wise OR. This frozen compatibility rule means `qtruth_impl(T, T) = S`.

The legacy quad operators and their lattice semantics are separate language
operations. They are not aliases for this family.

### `std.math`

`sqrt` and `abs` are `f64 -> f64`. `abs` is a sign-bit clear with no rounding
involved and is cross-platform bit-exact for every `f64` input, including
`NaN` and infinities.

`sqrt` is correctly-rounded per IEEE-754 for `x >= 0.0`: IEEE 754 mandates
`sqrt` as one of the operations required to be correctly rounded, so the
result is bit-exact and uniquely determined across every conformant
platform for this domain, matching Rust `f64::sqrt`'s existing behavior.
For every other admitted `f64` input — `x < 0.0`, and `x` itself already
`NaN` (reachable via a nested call such as `sqrt(sqrt(0.0 - 1.0))`) — `sqrt`
returns `NaN` (matching current behavior, not a new choice), but the
compatibility surface only promises the semantic property that the result
is `NaN` (observable as `sqrt(x) != sqrt(x)`) — the exact `NaN` bit pattern
(payload and sign bit) is explicitly **not** part of the frozen contract,
since IEEE 754 does not mandate a specific `NaN` payload and different
platforms/architectures can legitimately produce different bit patterns for
the same operation, including when propagating an already-`NaN` input.
Programs must not depend on `NaN` bit-pattern stability.

Neither performs a host effect or consults VM quota state.

`sqrt` and `abs` are reserved function names: a top-level program function
sharing either name is rejected deterministically at typecheck time
(`function name '...' is reserved for the std.math standard library
surface`, mirroring how host-boundary builtins like `stdout_write` are
already reserved). Without this, a user-defined `fn sqrt(...)` would
typecheck but be silently shadowed at runtime by the builtin, since VM call
dispatch intercepts these names by bare string match before function-frame
dispatch — promoting these names to a compatibility promise requires ruling
that failure mode out, not just documenting the intended behavior.

`sin`, `cos`, `tan`, and `pow` remain Deferred (see "Deferred families"
below): their bit-exact output depends on the underlying platform/libm
transcendental implementation, so their cross-platform determinism policy is
not yet qualified.

### `std.text`

`text` carries valid UTF-8. Concatenation preserves the left then right byte
sequence; equality is exact and performs no locale folding or Unicode
normalization. This version exposes no indexing, slicing, ordering, or length
API, so no code-point-versus-byte indexing promise is implied.

`to_text` is defined only for `text`, `bool`, `i32`, `u32`, and `quad`:

- `text` is returned unchanged;
- booleans are `true` or `false`;
- integers use canonical base-10 text;
- quad values are exactly `N`, `F`, `T`, or `S`.

Records and collections are rejected. `print` is deliberately excluded: its
controlled observation/capability boundary is an SSF-04 input, not a pure text
helper.

SSF-07 froze this boundary with test evidence against `+`: `i32`, `u32`,
`bool`, `quad`, `Map`, and `Record` operands are tested as representative
cases and all reject with the same "text concatenation currently admits
only text + text operands" message, while a `Sequence` operand is rejected
earlier by the unrelated ordered-sequence operator gate with a distinct
message ("ordered sequence values are not part of the current M8.3 Wave 1
operator surface") — both paths are pinned as distinct cases in
`tests/ssf07_text_family_freeze.rs`. The remaining admitted families
(`f64`, `fx`, `unit`, tuples, `Option`, `Result`) are not separately
tested: the `if lt == Type::Text || rt == Type::Text` check in
`typecheck.rs` is unconditional on the other operand's type, so any of
them would exercise the identical source line as the tested cases, not a
different one — confirmed empirically (`text + f64`, `text + (i32, i32)`,
and `text + <unit-returning call>` were checked against the actual `smc`
CLI and all produced the same message) rather than assumed. `to_text`'s
existing record/map/sequence/arity rejections are also pinned in
`tests/ssf07_text_family_freeze.rs`. `std.seq` and `std.map` (including
whether `Sequence(T)` indexing belongs inside the `std.seq` family
boundary) remain open for a follow-up slice.

### `std.seq`

Sequences have observable left-to-right index and iteration order.
`push` appends, `prepend` inserts at index zero, and `pop` returns a new
sequence without the final element. These operations are persistent: the
input value is not modified. `pop` on an empty sequence traps
deterministically.

`len` returns `i32`; `is_empty` tests zero length. `contains` uses exact value
equality and is selected only for `i32`, `u32`, `bool`, `text`, and `quad`
elements. Capacity and quota failures remain VM/runtime policy, not hidden
fallback behavior.

### `std.map`

Maps are persistent values. `map_empty()` requires a contextual `Map(K, V)`
type. `map_set` returns a new map, `map_contains` reports key presence, and
`map_get` returns the stored value or its explicit default.

Selected key families are `i32`, `u32`, `bool`, `text`, and `quad`, using exact
same-family equality. This version exposes no iteration or ordering API and
makes no observable map-order promise. It also selects no removal, mutation,
or serialization API.

### `std.option` and `std.result`

These are language-owned standard forms, not user-defined library ADTs.
`Option::Some` carries one value and `Option::None` carries none.
`Result::Ok` and `Result::Err` each carry one contextually typed value. Match
uses the frozen Foundation exhaustiveness and type-compatibility rules. No
unwrap, conversion, ordering, or formatting helpers are selected.

### `std.rand`

This family is deterministic VM state, never host entropy and never
cryptographic randomness.

- `random_seed(seed: i32) -> ()` uses the Rust widening cast `seed as u64`
  (negative values sign-extend modulo 2^64); zero becomes one.
- `random_next_i32(lo: i32, hi: i32) -> i32` requires `lo < hi` and returns a
  value in `[lo, hi)`.
- The algorithm is `xorshift64/13-7-17`: `x ^= x << 13`, `x ^= x >> 7`, then
  `x ^= x << 17`, with wrapping `u64` operations.
- Range selection is `lo + (next_u64 % (hi - lo))`, with the span evaluated in
  `i64`.
- An unseeded VM stream behaves as if seeded with one.

The algorithm and range mapping are part of contract `0.1`; changing either
requires a new contract version and migration decision in SSF-10.

## Deferred families

`std.math` selects only `sqrt` and `abs` (see "Selected API semantics"
above); its `sin`, `cos`, `tan`, and `pow` builtins remain
current-main/experimental because their exact cross-platform determinism and
compatibility policy is not qualified. Foundation numeric operators remain
governed by the source contract and
SSF-07.

`std.serde` selects no API and therefore no wire encoding. JSON use inside the
Rust implementation, CLI, Hub, or test infrastructure is not a Semantic source
library. A future serialization family must first freeze one deterministic
encoding, UTF-8 rules, map ordering, duplicate-key behavior, and rejection
semantics.

## Errors, effects, and compatibility

Type/arity/context errors reject during frontend analysis or lowering.
Malformed or capability-inconsistent artifacts reject before VM execution.
Only failures of already-admitted operations, such as `assert(false)`, empty
`pop`, or an invalid random range, are runtime traps/errors.

No selected API reads time, environment variables, arguments, stdin, files,
network, processes, or host entropy. No selected API writes host output.

Contract `0.1` is the first compatibility baseline. SSF-10 owns its retention
window; until then, compatibility means that the names and semantics above
cannot change silently on the Stable Foundation branch. Additive or breaking
changes require an explicit contract revision and renewed evidence.

## Evidence and runnable example

The per-family positive, negative, and compatibility anchors are recorded in
`docs/roadmap/stable_foundation/standard_library_v0_evidence.md`.

The canonical executable example is
`examples/canonical/stdlib_v0_helpers/src/main.sm`. It uses the public
language-owned names because `std.*` imports are intentionally outside
Foundation Source 1.0.

## SSF-04 entry conditions

SSF-04 may start only after this contract and its evidence map are reviewed,
green on the exact PR head, merged, and recorded on issue #1574. SSF-04 must
then define capability names, grants, denial, audit, and replay behavior. It
may use `print(text)` as existing evidence, but must not reinterpret this pure
stdlib contract as host authority.
