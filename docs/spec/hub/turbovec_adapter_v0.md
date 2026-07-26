# Semantic Hub v0 Reference Adapter: `vector.turbovec`

Status: draft v0
Owner crate: `semantic-hub-turbovec`

This document specifies the `vector.turbovec` Semantic Hub tool: a reference
adapter wrapping the third-party `turbovec` vector-quantization crate behind
the generic `semantic-hub` `HubTool` contract. It is the first Hub adapter
with a persisted, mutable, non-trivial backing store, and demonstrates that
the Hub's capability, determinism-classification, and failure-taxonomy
contracts hold for a dependency the Hub does not control.

```text
crates/semantic-hub-turbovec/
  Cargo.toml
  src/lib.rs             tool descriptor, HubTool impl, per-operation handlers
  src/payload.rs         JSON request/reply types, bound checks
  src/storage.rs         IndexName validation, ScopedStorage
  tests/determinism_qualification.rs
```

## 1. Dependency, license, features

`Cargo.toml` pins `turbovec = "=0.9.0"` -- an exact pin, not a caret range, so
a newer upstream release never lands silently.

- Source: crates.io; upstream repository <https://github.com/RyanCodrai/turbovec>.
- License: MIT. Upstream description: "Fast vector quantization with 2-4 bit
  compression and SIMD search."
- `rust-version = "1.70"` is turbovec's own declared MSRV.
- `#[cfg(not(target_pointer_width = "64"))] compile_error!` -- turbovec
  requires a 64-bit target and will not build on a 32-bit one; this adapter
  inherits that requirement transitively.

The adapter crate itself is `license = "Apache-2.0"`, matching the rest of
this repository. An Apache-2.0 crate depending on an MIT dependency is a
normal, compatible arrangement.

No non-default `turbovec` Cargo features are enabled by this adapter. turbovec's
own direct dependencies: `faer 0.20` (linear algebra / QR decomposition, used
to build the deterministic rotation matrix -- Section 9), `ndarray 0.17`
(with turbovec's own `"blas"` feature conditionally enabled on Linux/macOS
only, not Windows -- turbovec's choice, not configured by this adapter),
`ordered-float 4`, `rand 0.8` + `rand_chacha 0.3` + `rand_distr 0.4` (a
seeded RNG stack used only to construct the fixed rotation matrix at index-
shape granularity, never for anything nondeterministic at query time, and
never seeded directly by this adapter), `rayon 1.10` (internal data-
parallelism), `statrs 0.17`.

This adapter's own direct dependencies are `semantic-hub`, `turbovec`,
`serde`, `serde_json`. No network, HTTP, or process-spawning crate appears
anywhere in the dependency graph this adapter introduces.

## 2. Supported operations

Seven operations, exact capability lists and determinism classes from the
tool descriptor in `src/lib.rs`:

| Operation | Required capabilities | Determinism class | Mutates state |
| --- | --- | --- | --- |
| `vector.index.create` | `VectorIndexCreate`, `PrivateStorageRead`, `PrivateStorageWrite` | `Deterministic` | yes |
| `vector.index.describe` | `VectorIndexRead`, `PrivateStorageRead` | `Deterministic` | no |
| `vector.index.insert` | `VectorIndexMutate`, `PrivateStorageRead`, `PrivateStorageWrite` | `DeterministicWithSeed` | yes |
| `vector.index.remove` | `VectorIndexMutate`, `PrivateStorageRead`, `PrivateStorageWrite` | `Deterministic` | yes |
| `vector.search` | `VectorSearch`, `PrivateStorageRead` | `DeterministicWithSeed` | no |
| `vector.search.filtered` | `VectorFilteredSearch`, `PrivateStorageRead` | `DeterministicWithSeed` | no |
| `vector.index.reset` | `VectorIndexMutate`, `PrivateStorageRead`, `PrivateStorageWrite` | `Deterministic` | yes |

`DeterministicWithSeed` marks the three operations that run a turbovec
floating-point kernel (`insert`, `search`, `search.filtered`) -- see Section
9 for what that classification does and does not claim.
`Deterministic` marks the four with no floating-point kernel involved.
Missing capabilities are rejected by the generic Hub admission path before
this adapter's handler code runs; the adapter does not re-check them.

## 3. Request and reply payloads

All seven operations exchange flat JSON objects via `serde_json`. Standard
JSON has no `NaN`/`Infinity` literal, so an invalid coordinate can only ever
arrive as a finite-looking number; magnitude/finiteness is caught by the
validation layer in Section 6, not by parsing.

```text
vector.index.create
  request  { "index": string, "dim": u64, "bit_width": u64 (default 4) }
  reply    { "index": string, "dim": u64, "bit_width": u64 }

vector.index.describe
  request  { "index": string }
  reply    { "index": string, "dim": u64, "bit_width": u64, "len": u64 }

vector.index.insert
  request  { "index": string, "vectors": [[f32, ...], ...], "ids": [u64, ...] }
  reply    { "index": string, "inserted": u64, "len": u64 }

vector.index.remove
  request  { "index": string, "ids": [u64, ...] }
  reply    { "index": string, "removed": [u64, ...], "not_found": [u64, ...], "len": u64 }

vector.search
  request  { "index": string, "queries": [[f32, ...], ...], "k": u64 }
  reply    { "index": string, "index_version": u32, "hits": [[SearchHit, ...], ...] }

vector.search.filtered
  request  { "index": string, "queries": [[f32, ...], ...], "k": u64,
             "allowed_ids": [u64, ...] }
  reply    { "index": string, "index_version": u32, "hits": [[SearchHit, ...], ...] }

vector.index.reset
  request  { "index": string }
  reply    { "index": string, "dim": u64, "bit_width": u64 }

SearchHit { "external_id": u64, "score": f32, "rank": u64 }
```

`vectors`/`ids` in insert must be equal length (checked before touching
turbovec). Removing an absent id is not an error -- it is reported in
`not_found` rather than failing the request. `allowed_ids` is the only field
distinguishing a filtered request from a plain `vector.search` request; the
reply shape is identical either way. `hits` is a list of lists: outer index
is the query's position in `queries`, inner index is rank within that
query's top-k.

`index_version` in the reply is currently a static placeholder value of `1`
on every reply -- there is no real content-based index versioning in v0.
This is stated here as a known limitation, not a real version counter
callers should rely on for change detection.

## 4. Index lifecycle

Four operations mutate a persisted index: `create`, `insert`, `remove`,
`reset`. There is no fifth "clear"/"truncate to empty" primitive, because
`turbovec::IdMapIndex` does not expose one natively.

- `create` fails if an index of that name already exists, or if the
  persisted-index-count ceiling (Section 5) is already reached; otherwise it
  constructs a fresh, empty `IdMapIndex` of the requested shape and persists
  it.
- `insert` adds vectors with caller-supplied external ids to an existing
  index.
- `remove` deletes ids from an existing index; absent ids are reported, not
  rejected.
- `reset` loads the existing index only to read back its `dim` and
  `bit_width`, constructs a brand-new empty `IdMapIndex` of that same shape,
  and atomically overwrites the persisted file with it. This is deliberate:
  turbovec has no in-place clear/truncate call, so "reset" here means "fresh
  empty index of the same shape," not a hidden fast-path truncation of
  internal state.

## 5. Limits

| Limit | Value | Enforced by |
| --- | --- | --- |
| `dim` | positive multiple of 8, up to `MAX_DIM` = 65536 | turbovec (`ConstructError::DimNotPositiveMultipleOf8` / `DimTooLarge`) |
| `bit_width` | one of `{2, 3, 4}` | turbovec (`ConstructError::BitWidthOutOfRange`) |
| persisted index count | `MAX_INDEX_COUNT` = 256 per scoped directory | this adapter (`storage.rs`) |
| coordinate magnitude | finite and `< 1e16` (`MAX_INPUT_MAGNITUDE`) | turbovec, reused by this adapter for query pre-validation |
| index name length | 1 to 64 bytes | this adapter (`storage.rs`) |

`MAX_DIM` exists in turbovec itself specifically to stop an untrusted,
arbitrarily large `dim` from driving a multi-gigabyte `dim x dim`
rotation-matrix allocation -- a resource-exhaustion concern turbovec's own
authors documented, not something added on top by this adapter.
`MAX_INPUT_MAGNITUDE = 1e16` exists because f32 sum-of-squares in the norm
computation can overflow to `+Inf` near that magnitude for dimensions up to
65536; a coordinate at or beyond it is rejected rather than silently
corrupting the index (Section 6).

`MAX_INDEX_COUNT` is counted by counting `.tvim` files in the scoped
directory; a read error counts as zero rather than failing the caller, since
this is only an admission ceiling, not a source of truth. Per-request batch
bounds on id/vector count (insert, remove) and on total result count
(`queries.len() * k`, not `k` alone -- search) are also enforced by this
adapter ahead of any turbovec call, so an attacker-controlled count in the
request cannot drive unbounded allocation before validation runs.

## 6. Vector and query validation

This adapter uses `turbovec::IdMapIndex` exclusively -- never the lower-level
positional `TurboQuantIndex` directly -- because `IdMapIndex` provides stable
`u64` external ids that survive removes. The positional index's own
`swap_remove` moves the last vector into the deleted slot, invalidating
positional references; `IdMapIndex` hides this behind a bidirectional
id-to-slot map.

**Insert path** (`IdMapIndex::add_with_ids_2d`) is entirely typed-`Result`,
never panicking. turbovec's `AddError` variants: `DimMismatch{existing,got}`,
`DimNotMultipleOf8`, `DimTooLarge{dim,max}`,
`VectorBufferNotMultipleOfDim{vectors_len,dim}`,
`IdsCountMismatch{expected,got}`, `IdAlreadyPresent(u64)` (rejected both
cross-batch and against existing index content, validated up front so a
partial failure never leaves ghost id-table entries), and
`InvalidInputValue{vector_index,coord_index,value}` (a coordinate is NaN,
Inf, or `>= 1e16`). This adapter pre-checks, before calling turbovec at all:
batch size against its own bound (`TooManyVectors`), an ids/vectors length
mismatch (`IdsVectorsCountMismatch`), and a per-vector dimension mismatch
against the index's declared dim (`VectorDimensionMismatch`). Whatever
`AddError` turbovec still returns after those checks (in practice
`IdAlreadyPresent` or `InvalidInputValue`) is surfaced as a generic
`InsertRejected` error carrying turbovec's message text; this adapter does
not re-split that remaining surface into more specific codes.

`InvalidInputValue` matters beyond simple rejection: NaN/Inf poison the
per-vector scale so the slot exists in `.len()` but is never reachable via
search, and overflow-to-`+Inf` can make a corrupted slot incorrectly win
every top-k query. Rejecting it at insert time keeps a poisoned or
falsely-dominant slot out of the index entirely.

**Search path** (`IdMapIndex::search` / `search_with_allowlist`) has **no**
typed-error variant -- unlike insert, it panics if: `queries.len()` is not a
multiple of `dim`; any query coordinate is non-finite or `>= 1e16`
(checked internally via turbovec's own exported
`first_invalid_coord(values, dim)`); the allowlist is empty
(`assert!(!ids.is_empty())`); or the allowlist contains an id not currently
in the index (`panic!("id {id} in allowlist is not present in index")`).

This adapter pre-validates every one of these before calling turbovec,
converting each into a typed error instead of letting it reach a panic:

| Condition | This adapter's error code |
| --- | --- |
| query length doesn't match index dim | `VectorDimensionMismatch` |
| query list itself is empty | `EmptyQuery` |
| non-finite or over-magnitude query coordinate | `InvalidQueryValue` |
| `allowed_ids` empty (filtered search only) | `EmptyFilter` |
| `allowed_ids` contains an id not present in the index | `UnknownFilterId` |

The non-finite/over-magnitude check calls turbovec's own exported
`first_invalid_coord` directly rather than reimplementing the `1e16`
threshold -- reusing turbovec's exact validation, not duplicating it. The
unknown-filter-id check calls `IdMapIndex::contains(id)` for every allowlist
id before search is invoked. The generic Hub runtime also wraps dispatch in
`catch_unwind`, but that is defense-in-depth only: this adapter's own
pre-validation is the primary mechanism, and was verified by real tests to
prevent all four panic conditions above from ever reaching a panic in
practice.

## 7. Filter behavior

`vector.search.filtered` restricts search to a caller-supplied set of
external ids (`allowed_ids`):

- the allowlist must be non-empty (`EmptyFilter` otherwise);
- every id in the allowlist must already exist in the index
  (`UnknownFilterId` on the first one that doesn't);
- ids outside the allowlist are excluded from the result entirely, not just
  down-ranked;
- the reply shape is identical to plain `vector.search`.

## 8. Ordering and tie-breaking

Search results within a query's row are ordered by turbovec's own internal
ranking; this adapter does not re-sort, re-score, or break ties itself --
`rank` in each `SearchHit` reflects position in the array turbovec returns.

The exact rule turbovec uses to break a tie between equal-scoring
candidates is turbovec's own internal behavior. It is not documented and not
guaranteed by this adapter. What was verified (Section 9) is only that the
ordering is stable and repeatable across identical repeated queries against
the same data -- not what the rule itself is.

## 9. Measured determinism guarantee

**Rotation determinism.** turbovec's `rotation.rs` defines
`const ROTATION_SEED: u64 = 42;` -- a fixed constant, not derived from OS
entropy or wall-clock time. `make_rotation_matrix(dim)` seeds
`ChaCha8Rng::seed_from_u64(ROTATION_SEED)`, builds a Gaussian random matrix,
and takes its QR decomposition (via `faer`) with a sign-correction step,
producing a deterministic `dim x dim` orthogonal matrix for a given `dim`.
This is why `insert`/`search`/`search.filtered` are classified
`DeterministicWithSeed` rather than plain `Deterministic`: the seed is the
fixed constant `42`, not a caller-supplied value, and not the wall clock or
environment.

**Empirical qualification.** `crates/semantic-hub-turbovec/tests/determinism_qualification.rs`
contains three passing tests, run on this development machine:

1. `repeated_search_on_the_same_loaded_index_is_byte_identical` -- 10
   repeated identical searches against one in-memory `TurboVecAdapter`
   instance produce byte-for-byte identical serialized JSON replies.
2. `search_after_reloading_the_index_from_disk_matches_the_original` --
   three separate `TurboVecAdapter` instances, each freshly constructed and
   reloading the same persisted `.tvim` file from scratch (simulating three
   separate `smc hub invoke` CLI process invocations, the real v0 usage
   pattern since there is no long-running Hub daemon), all produce
   byte-identical replies to the same query.
3. `exact_duplicate_vectors_produce_a_stable_tie_break_order_across_repeats`
   -- five bit-identical inserted vectors, searched repeatedly, always
   return the same tie-break ordering. The rule producing that order is not
   documented or guaranteed (Section 8); only stability/repeatability was
   verified.

**Explicitly out of scope.** Cross-machine or cross-CPU-backend byte
identity was **not** tested. turbovec dispatches SIMD kernels at runtime via
`is_x86_feature_detected!` (AVX-512BW on modern x86, falling back to AVX2
then scalar; NEON on ARM); a different CPU could take a different code path
and, in principle, produce different floating-point rounding. This was not
exercised on a second machine or CPU in this pass.

Two claims are worth keeping separate: **byte-identity** (exact score and
order match, bit for bit, across repeats -- the stronger claim, and what the
three tests above actually measured, but only on one machine/build/
toolchain) versus **ranking-stability** (same top-k membership regardless of
exact score bits or tie order -- a weaker, more portable property that this
pass does not separately claim holds across machines; it simply was not the
property measured). Determinism claims in this document are scoped to this
development machine, this build, this toolchain -- not a cross-machine or
cross-CPU-backend reproducibility claim.

## 10. Persistence model

There is no separate explicit save/load verb in the Hub-facing API. Every
mutating call (`create`, `insert`, `remove`, `reset`) loads-or-initializes
the `IdMapIndex`, mutates it in memory, and atomically rewrites the whole
index to `<data_dir>/vector.turbovec/<name>.tvim`: write to a temp file
(named with the process id and a nanosecond timestamp to avoid collisions),
then `fs::rename` over the final path, so a concurrent reader never observes
a partial or torn write. Read operations (`describe`, `search`,
`search.filtered`) load fresh from that file on every call.

This follows directly from how the Hub CLI is used: one short-lived process
per `smc hub invoke`, with no in-memory cache surviving across invocations
in v0 -- the `.tvim` file on disk **is** the adapter's actual state between
invocations, not a mirror of some other source of truth. `.tvim` is
turbovec's own persistence format (quantized codes plus the id-map side
table), written and loaded through `IdMapIndex::write`/`IdMapIndex::load`,
and round-trips exactly.

## 11. CPU and backend assumptions

turbovec requires a 64-bit target and will not compile on a 32-bit one.
It dispatches its SIMD kernels at runtime per the running CPU's detected
feature set (AVX-512BW / AVX2 / scalar fallback on x86; NEON on ARM); this
adapter does not pin, configure, or override that dispatch. Determinism
claims (Section 9) are conditioned on running the same build on the same
machine -- a different CPU backend is explicitly out of scope for the
current qualification.

## 12. Failure behavior

Every operation returns a typed `HubToolError` with a stable string code on
failure rather than propagating a raw turbovec panic or an untyped error.
Codes observed across the seven handlers: `MalformedInput`,
`InvalidIndexName`, `IndexAlreadyExists`, `TooManyIndexes`,
`InvalidIndexParameters`, `IndexLoadFailed`, `StorageUnavailable`,
`IndexWriteFailed`, `TooManyVectors`, `IdsVectorsCountMismatch`,
`VectorDimensionMismatch`, `InsertRejected`, `TooManyResults`, `EmptyQuery`,
`InvalidQueryValue`, `EmptyFilter`, `UnknownFilterId`. None of these are
reached by unwinding a panic; each is a normal `Result::Err` from this
adapter's own handler code.

Adversarial cases verified live against the built `smc` binary, not just
unit tests: a dimension-mismatched insert vector produced a typed
`ToolDeclaredFailure`/`VectorDimensionMismatch`, not a crash; an empty
`capabilities` set on a search request produced a `CapabilityDenied`
rejection from the generic Hub admission path, and the Hub continued
working correctly on the next call afterward. That last case is where a
real audit-log-corruption bug was found and fixed during dogfooding --
exercising the adapter through the actual CLI end to end, not only unit
tests, is what caught it; it is recorded here as a fixed regression, not an
open issue.

## 13. Real dogfooding record

Run against the built `smc` binary, in order, all successful: create an
index (`dim=8`, `bit_width=4`); insert 4 one-hot vectors with external ids
101-104; describe the index (confirmed `len=4`); search for the vector
matching id 102 (correctly ranked #1, score approximately 1.003 -- not
exactly 1.0, because 2-4 bit quantization makes search scores approximate
rather than exact cosine similarity, which is expected quantization behavior
and not a bug); filtered search restricted to ids `{101, 103}` (correctly
excluded 102 and 104); remove id 102; search again (confirmed 102 no longer
appears); audit lookup by request id (full record retrieved). The
adversarial cases from Section 12 were run in the same pass.

## 14. Security and privacy boundaries

- **Index identity.** `IndexName` accepts only lowercase ASCII alphanumeric
  characters, `_`, and `-`, 1 to 64 bytes. `.`, `/`, and `\` are rejected
  outright by the allowed-charset check itself -- not path-normalized and
  then checked -- so path traversal is impossible by construction. The
  derived path is always `<scoped_root>/<name>.tvim`; `scoped_root` is fixed
  at adapter construction and never caller-supplied.
- **Index count ceiling.** `MAX_INDEX_COUNT = 256` persisted indexes per
  scoped directory (Section 5), a best-effort admission ceiling, not a
  source of truth.
- **No network access, no arbitrary filesystem access, no process spawn, no
  environment variable reads** anywhere in this adapter's code.
- **Capability requirements** are exact per operation and enforced by the
  generic Hub admission path before this adapter's handler code runs (table
  in Section 2).

## 15. Non-authority statement

Search results (`SearchHit { external_id, score, rank }`) are candidates and
evidence only. A hit is never a Semantic-truth claim, a verified-relevance
claim, a causal-compatibility claim, or a permission to act. Nothing in this
adapter elevates a turbovec search result to an authoritative judgment about
the underlying data; callers remain responsible for what they do with the
returned candidates.
