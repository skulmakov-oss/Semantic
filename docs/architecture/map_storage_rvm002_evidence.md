# SEM-RVM-002: actual VM prototype evidence

Status: isolated experimental evidence, not production API qualification.
No persistent Map implementation has been committed to production source.
No merge, dependency, verifier, wire-format or quota-default change occurred.

## Provenance

- Historical main reference: `94c5c50d16c3f2efc7691456755461bedae83b18`.
- Primary compatible Vec reference: PR #2042,
  `8db588bea6395839eb543aee453c2b2657e856d5`.
- Persistent experiment starts from that same #2042 commit and replaces the
  Map payload/opcode implementation only in a disposable Linux clone.
- Ubuntu 24.04 / WSL2, x86-64, Rust 1.97.1
  (`8bab26f4f`, 2026-07-14), release opt-level 3, LTO, codegen-units 1.
- Same compiler and admitted SemCode bytes; SHA256 identities are recorded in
  `tests/fixtures/rvm002/artifact_identities.json`. Matching artifacts were
  compared byte-for-byte. Binary SHA256 identities are recorded separately.
- VerifiedLocal configuration with explicit max_steps=2,000,000; other quotas
  unchanged. Empty capability manifest, tripwire host for pure loops.
- A GlobalAlloc wrapper over System counts allocation requests, requested bytes,
  current live bytes and peak. Default wrapper realloc behavior is included.
  These are instrumented timing/request counts, not production allocator speed
  or RSS. Instrumentation is benchmark-only.
- Existing Windows workspace compilation ran concurrently. CPU contention and
  three-sample ranges limit precise speedup claims. Largest primary candidate
  sample count is one, disclosed below. Raw prior samples remain available.

The runtime charges one Step per successfully decoded opcode before dispatch;
thus successful-loop Steps are dynamic guest instruction counts. Calls=0 and
peak Frames=1 for these loops. Compile/verify precede the timed execution; wall
measurements include complete VM execution and its per-execution planning.

## Real Semantic construction

The workload is compiled Semantic source, not a Rust insertion loop:

```text
fn main() {
    let m: Map(i32,i32) = map_empty();
    let i: i32 = 0;
    while i < N {
        m = map_set(m,i,i);
        i = i + 1;
    }
    assert(map_get(m,N-1,-1) == N-1);
    return;
}
```

`N` is replaced with each concrete integer by the recorded generator. Both
builds compile/admit the same program and produce identical SemCode bytes.
All completed runs return successfully with the assertion satisfied.

Primary interleaved comparison, #2042 Vec vs persistent VM:

| n | Vec median ms | Persistent median ms | Samples per completed model | Equal Steps |
|---:|---:|---:|---|---:|
| 16 | 0.159 | 0.147 | 3 / 3 | 243 |
| 128 | 2.040 | 1.061 | 3 / 3 | 1811 |
| 1024 | 138.855 | 14.808 | 3 / 3 | 14355 |
| 4096 | 1907.235 | 51.140 | 3 / 3 | 57363 |
| 8192 | 6796.697 | 72.500 | 3 / 3 | 114707 |
| 16384 | 30089.401 | 171.196 | 3 / 3 | 229395 |
| 32768 | INCOMPLETE (90 s bound) | 332.019 | 0 / 1 | candidate 458771 |

Raw ranges and counters: `compatible_scale.csv`. Four-size descriptive log-log
slope (1024–16384) is 1.925 for Vec and 0.852 for persistent. The latter is
affected by small-case noise/CPU load and is not a constant-time or strict
complexity claim. Source inspection gives O(n log n) node work for unique-key
construction. Timing and allocation scaling are consistent with elimination
of the former quadratic copying/search bottleneck in the actual VM prototype.

At 16384, Vec requests 65,136,156,685 bytes; persistent requests 33,434,237 bytes.
Peak extra live requested bytes are 7,671,861 vs 4,989,333. Allocation counts
are 458,844 vs 786,492: the tree performs more small node allocations while
eliminating full-payload/container copy volume. At 32768 persistent requests
70,789,637 bytes across 1,638,458 allocations; peak extra live bytes 9,970,189.
Do not report requested cumulative bytes as simultaneously resident memory.

The first historical-main comparison is preserved in
`historical_main_scale.csv`. It combines copy elision and persistent storage
and must not be presented as isolating the storage change. Historical main at
32768 also timed out at 90 seconds. No timeout was extended or retried on the
same reference. Three earlier persistent 32768 samples were 934–949 ms under
concurrent baseline/build load; this discrepancy is retained as timing noise,
not discarded to improve the result.

## Replacement and snapshots

Replacement source builds a map, then runs another Semantic loop replacing
every key with `i+1` and asserts the final value. Historical-main/reference
and persistent outcomes/counters match for every completed matched size.

| n | Main-reference median ms | Persistent median ms | Equal Steps |
|---:|---:|---:|---:|
| 1024 | 311.161 | 23.881 | 30746 |
| 4096 | 6409.784 | 128.195 | 122906 |
| 8192 | 23130.624 | 196.787 | 245786 |
| 16384 | NOT RUN (bounded baseline budget) | 415.019 | candidate 491546 |
| 32768 | NOT RUN (bounded baseline budget) | 793.420 | candidate 983066 |

These replacement results use the historical main baseline, explicitly not
the isolated #2042 control. Raw data: `replacement_scale.csv`. No full-size
replacement speedup ratio is claimed where the baseline was not run.

Actual VM snapshot scenario: save the constructed map in `old`, replace every
key in `m`, assert `old` retains the original last value and `m` has the new
last value. At 4096: main-reference median 5195.772 ms, persistent 99.313 ms;
both Steps 122917, Calls 0, Frames 1. Peak extra bytes 4,269,083 vs 2,505,859.
This retains one complete prior VM version; it does not prove arbitrary-history
memory bounds. Sources and raw rows are retained as `map_snapshot.rs`,
`snapshot_reference.csv`, `snapshot_prototype.csv`.

Separate structure-only retained-history probe: 100 independent updates of
one base, simplified I32 values, same counting allocator. At 4096, Vec base
extra live bytes 163840, retained versions add 32770400; persistent base
1081344, retained versions add 182400. This shows the base-memory tradeoff and
sharing benefit. It is not a full-Value or VM retained-history measurement.
Raw data/source: `storage_memory.csv`, `storage_memory.rs`.

## Actual compiler-style text/nested-value workload

The independently authored SemSeed O0 artifact has SHA256
`edad5ca66ea2cb668211498f2cccffa87bbe3a09f1c74e649e4005fed616d9e2`.
Its checker/interpreter uses text-key maps and record values. It is a limited
compiler frontend/interpreter workload, not a complete self-hosted compiler.
Both Linux runs use identical artifact/input, ArgsRead and StdoutWrite only.
The experimental benchmark strengthens its output assertion to exact
`ok:int:14`, without modifying PR #2042's production/diagnostic source.

Nine samples each: historical-main median 8.921 ms (5.958–11.152), persistent
7.875 ms (5.684–11.064). Both Steps 11785, Calls 344, peak Frames 8, effects 2.
Allocations 31463 vs 29787; requested bytes 4835242 vs 4705002. Timing ranges
overlap: wall-time improvement is INCONCLUSIVE. Counts/output are reproducible
within the samples. Raw rows: `compiler_reference.csv`, `compiler_prototype.csv`.

## Independent review and remaining qualification

Fresh independent static reviews found no actionable #2042 production defect
and no new architectural contract blocker. They identified these explicit
prototype/production requirements:

- Group duplicate imports once; the frozen VM prototype still has quadratic
  ordinal-list copying for pathological duplicate imports (P2 if adopted as-is).
- Reindex shared entries during compaction; to_pairs currently clones nested
  payloads (cost/maintenance risk, not a new current production failure).
- Implement logical Debug, not the prototype's physical-tree derived Debug.
- Validate AVL height/balance/BST ordering and occurrence/order consistency.
- Exercise exhaustion with a bounded artificial ordinal limit.
- Complete the production VM differential matrix; the model checker alone
  cannot qualify register errors, OWN0, aggregates or verifier authority.

The last two data-structure test requirements were subsequently exercised in
the standalone qualification version: 10000 Vec-differential updates, five key
families, 20 preserved versions, nested NaN, 65536-byte UTF8 key, index invariant
checks, and 300 updates under synthetic limit32 with duplicates/snapshots.
Both commands exit 0. The standalone checker uses a simplified Value model.
`map_value_vm.rs` is the exact frozen implementation used by VM measurements;
`map_value.rs` adds qualification-only invariant/limit hooks. Neither is wired
into the production workspace. A complete current production Map differential
suite remains NOT RUN pending the approved public contract and implementation.

## Reproduction

All files are experiment data under `tests/fixtures/rvm002`; no production
module imports them. Use disposable clones only. `prepare_vm_prototype.py`
records the exact three Map opcode/payload transformations and copies the frozen
tree. Pass the path of a dedicated pristine clone at the exact #2042 HEAD:
`python3 prepare_vm_prototype.py <disposable-clone>`. It validates Git identity
and tracked cleanliness before writing; do not run it against an ordinary
working directory. The baseline must be
checked out at the exact reference SHA, with the same map_scale example copied
into its crate examples directory. Build both with
`cargo +1.97.1 build -p sm-vm --example map_scale --release`.

Run `map_scale N 1 build <artifact-directory>` on each binary in alternating
order, three times per size, with a separate 90-second bound per invocation.
Large bounded failures remain INCOMPLETE. For standalone structure checks:

```text
rustc +1.97.1 --edition=2021 --cfg rvm002_qualification storage_check.rs
./storage_check
rustc +1.97.1 --edition=2021 --cfg rvm002_qualification --cfg rvm002_small_ordinals storage_check.rs
./storage_check
rustc +1.97.1 --edition=2021 -O storage_memory.rs
./storage_memory
```

Production implementation and its public API snapshots still require the exact
OWNER CONTRACT GO described in the companion contract proposal. No source
breaking implementation commit or production dependency has been made.
