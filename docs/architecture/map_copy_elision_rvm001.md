# SEM-RVM-001: compatible Map copy elision

Status: implementation candidate; OWNER merge authorization remains separate.
Base: `94c5c50d16c3f2efc7691456755461bedae83b18`.

## Scope and contract

`Value::Map(Vec<(MapKey, Value)>)` remains unchanged. MAP_GET and
MAP_CONTAINS borrow the register value instead of cloning every pair.
MAP_SET borrows the input and clones surviving pairs once, removes every
matching key and appends the replacement. The input, including nested
owned aggregates, remains independent. Destination/source register aliasing
is safe: the result is owned before the destination is written.

The register helper preserves invalid/uninitialized register errors. Operand
validation order remains map, key, value/default, destination. A present GET
still ignores the default register. No verifier, capability, wire-format,
Steps, Calls, Frames, dependency, or public API policy changes are included.

Language Map equality/iteration is not added. Public Rust `PartialEq` and
pair order are preserved, including externally constructed duplicate keys.

## Reproduction

Run `cargo +1.97.1 run --release -p sm-vm --example map_cost`. The example
uses canonical verified tokens and the unchanged VerifiedLocal envelope.
It emits CSV for 16/128/1024-entry maps with 256-byte text values and
compiler-generated Map construction loops at 16/128/512 entries.
GET/CONTAINS/SET batches contain 100 operations; clone batches contain 100
public `Value` clones. Source construction measures complete VM execution.

The optional first argument names an existing SemSeed artifact; the optional
second argument names an existing output directory for generated SemCode.
SemSeed is the independently authored Semantic lexer/parser/checker/interpreter
from SEM-BOOT-003, not a replacement host-language implementation.
The supplied O0 artifact SHA-256 is
`edad5ca66ea2cb668211498f2cccffa87bbe3a09f1c74e649e4005fed616d9e2`.
Its input is `let x:int = 10; x = x - 3; x = x * 2; return x;`.
Both builds return `ok:int:14`, Steps 11785, Calls 344, peak Frames 8,
EffectCalls 2, with the default quotas and verifier admission.
The artifact is external to this focused PR; its source is not copied here.

Environment: Windows x86-64, Intel Core i5-9300H, Rust 1.97.1
(`8bab26f4f`), matched release builds, same example source.
Three alternating baseline/candidate process pairs, nine samples per case
per process, give 27 samples per build/case. Generated input artifacts were
SHA-256 identical between builds. Raw measurements are in
`tests/fixtures/rvm001/paired_vm.csv`.

## Measured results

All times below are instrumented wall time, milliseconds per batch/run.
Counting allocator overhead is included. Allocations include the input
argument clone and VM preparation. Peak is extra simultaneously live
requested allocation bytes, not RSS or allocator-resident memory. The
allocator uses GlobalAlloc's default reallocation implementation; these
measurements must not be presented as an uninstrumented production speedup.

| Case | Baseline median (range) ms | Candidate median (range) ms | Allocations before → after | Allocated bytes before → after |
|---|---:|---:|---:|---:|
| GET, 1024 entries, 100 reads | 42.006 (29.127–73.536) | 1.683 (1.078–3.221) | 103859 → 1359 | 37278144 → 414144 |
| CONTAINS, 1024 entries, 100 reads | 38.417 (28.711–79.003) | 1.679 (1.323–5.100) | 103759 → 1259 | 37252344 → 388344 |
| Insert, 1024 entries, 100 independent updates | 119.669 (78.280–209.502) | 76.213 (48.051–126.733) | 207159 → 104659 | 106023744 → 69159744 |
| Replace, 1024 entries, 100 independent updates | 126.900 (73.159–226.499) | 47.077 (33.860–119.313) | 206959 → 104459 | 84698944 → 47834944 |
| Source construction, 512 entries | 39.477 (28.632–78.475) | 34.730 (25.172–56.420) | 12379 → 11867 | 77154300 → 63496188 |
| SemSeed actual compiler pipeline | 15.830 (13.095–47.807) | 16.467 (12.957–55.907) | 31463 → 31411 | 4835242 → 4831360 |

GET/CONTAINS demonstrate a substantial large-map improvement in this
instrumented workload. SET allocation reductions are deterministic; noisy
overlapping timing ranges do not support a general SET speedup claim.
Source-construction and SemSeed wall-time improvement/regression are
INCONCLUSIVE. Public clone allocation count and bytes are unchanged.

## Remaining costs and next decision

Lookup remains O(n) key comparisons; GET clones only the selected value.
SET remains O(n), with deep copies of surviving owned values and key strings.
Sequential construction remains quadratic. Registers, locals, arguments,
closures and nested Map/Tuple/Record/ADT values still cross owned-value
clone boundaries; Sequence retains existing Arc COW behavior.

No allocation/heap quota exists in the current quota taxonomy. One VM Step
can still perform significant copying. This PR neither introduces allocation
accounting nor changes Calls or the trusted-compiler envelope.

A separate storage proposal evaluates persistent trees and deterministic
HAMT. This compatible PR is not fulfillment of the full storage redesign
and does not authorize its public API migration.
