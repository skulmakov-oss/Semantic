# SHF-P3 production evidence: shared sequence storage and verified consumption

Generated from the raw qualification data of branch `perf/vm-p3-cow-consume`.

**Reference.** `pristine` is the unmodified `main` (`0373bec5`) VM.

**Candidates.**
- `impl` is this branch, built without any research feature.
- `p3` is the P3-R1 research prototype, included for parity.

**Inputs.** Identical verified SemCode artifacts, byte-identical across platforms.

**Quotas.** Trusted-compiler envelope (`max_steps` 200,000,000); Calls 16,384 and Frames 256, both unchanged.

**Timing.** Warm sample, release build, 7 interleaved rounds with alternating order. Cells over 60 s or 300 s get 3 or 1 rounds; a cell over the 900 s budget is INCOMPLETE.

**Platforms.** Windows 11 x64 (native, MSVC) and Ubuntu 24.04.5 LTS (**WSL2**, GNU), on the same Intel i5-9300H. Do not compare absolute times across platforms.

**Measured commit.** The qualification ran on commit `4497a3a` (E1 + E3/E4). The branch head `811ca77` adds only the empty-plan fast path in `ConsumptionPlan::site()`. Its A/B (`ab_fastpath.json`, Windows and Ubuntu) showed identical outcomes and counters and no measurable change on optimized workloads.

**Windows A, `append_100000`.** The two non-PASS rows are reference timeouts: the pristine VM exceeded the 900 s budget. `impl` and `p3` returned `ok`, Steps 1,500,027, Calls 0, peak Frames 1, identical to the pristine result recorded for the same artifact in P3-R1. On Ubuntu the reference completed and the row is PASS.

## Windows 11 (native)

### Correctness (reference = pristine)

| Phase | Passed / total | Notes |
|---|---:|---|
| A: 224 fixtures × {impl, p3} | 446 / 448 | append_100000 [impl]: reference ["TIMEOUT", null, null, null]; append_100000 [p3]: reference ["TIMEOUT", null, null, null] |
| B: failure sweeps (every Steps / Calls / Frames limit) | 1819 / 1819 | |
| C: alternating programs in one process | 36 / 36 | |
| D: byte mutations (verifier-rejected and admitted) | 120 / 120 | |
| E: 20 repeats in one process | 8 / 8 | |

### Performance (median warm ms; [min-max]; peak memory MB)

| Workload | pristine | impl | p3 (research) | pristine / impl | impl / p3 | peak mem pristine → impl |
|---|---:|---:|---:|---:|---:|---|
| lexer_720B | 60.29 [56.4-69.0] | 26.85 [24.4-28.6] | 27.23 [24.9-29.4] | ×2.2 | ×0.99 | 7.8 → 6.2 |
| lexer_2880B | 401.84 [361.0-667.3] | 59.42 [58.0-70.8] | 62.92 [57.8-125.7] | ×6.8 | ×0.94 | 12.7 → 6.8 |
| lexer_11520B | 7737.56 [6272.9-10343.8] | 259.25 [249.9-447.0] | 267.23 [258.9-453.0] | ×29.8 | ×0.97 | 31.5 → 10.2 |
| lexer_23040B | 31209.49 [29350.7-40958.5] | 947.29 [566.7-1058.8] | 960.40 [548.4-1053.5] | ×32.9 | ×0.99 | 54.9 → 13.5 |
| lexer_46080B | 155642.94 [151435.7-155705.4] | 1368.55 [1282.0-2297.5] | 1415.67 [1271.9-2327.4] | ×113.7 | ×0.97 | 102.7 → 20.9 |
| b_parse_8_orig | 89.44 [80.0-109.0] | 11.46 [10.2-14.0] | 10.82 [10.2-11.9] | ×7.8 | ×1.06 | 8.7 → 6.4 |
| b_parse_8_fastlex | 145.98 [81.0-149.4] | 16.76 [9.8-19.8] | 14.82 [10.4-17.8] | ×8.7 | ×1.13 | 8.9 → 6.3 |
| b_parse_16_orig | 558.29 [517.3-563.8] | 35.83 [20.6-37.3] | 34.56 [21.5-37.7] | ×15.6 | ×1.04 | 11.5 → 6.4 |
| b_parse_16_fastlex | 325.25 [315.2-390.0] | 17.57 [16.8-23.1] | 17.99 [16.4-21.6] | ×18.5 | ×0.98 | 11.7 → 6.3 |
| b_parse_32_orig | 2120.34 [1248.7-2277.5] | 65.52 [38.8-70.4] | 66.01 [38.0-87.1] | ×32.4 | ×0.99 | 16.9 → 6.3 |
| b_parse_32_fastlex | 2021.23 [1258.9-2315.0] | 38.90 [32.0-63.2] | 38.78 [33.5-88.3] | ×52.0 | ×1.00 | 17.4 → 6.3 |
| b_eval_loop_10_orig | 17.57 [17.2-21.7] | 9.59 [9.0-11.7] | 9.82 [9.4-11.1] | ×1.8 | ×0.98 | 6.9 → 6.4 |
| b_eval_loop_20_orig | 40.08 [38.1-48.5] | 25.59 [21.8-26.6] | 24.32 [22.2-29.6] | ×1.6 | ×1.05 | 6.9 → 6.5 |
| b_eval_loop_40_orig | 36.49 [34.4-42.8] | 22.91 [21.1-25.6] | 23.92 [21.9-25.6] | ×1.6 | ×0.96 | 7.0 → 6.4 |
| b_eval_loop_80_orig | 109.52 [94.4-119.6] | 66.45 [65.3-72.2] | 73.30 [69.6-88.3] | ×1.6 | ×0.91 | 6.9 → 6.5 |
| append_100 | 0.61 [0.6-0.6] | 0.28 [0.2-0.4] | 0.31 [0.3-0.3] | ×2.2 | ×0.92 | 5.9 → 5.8 |
| append_1000 | 30.40 [25.7-35.6] | 2.31 [2.3-2.5] | 2.61 [2.6-3.0] | ×13.2 | ×0.88 | 6.6 → 5.8 |
| append_10000 | 3561.83 [3195.2-5064.5] | 41.82 [22.8-46.9] | 47.78 [28.1-59.1] | ×85.2 | ×0.88 | 12.6 → 7.2 |
| append_50000 | 221799.30 [215886.8-240069.3] | 216.15 [117.8-245.3] | 238.32 [152.9-266.4] | ×1026.1 | ×0.91 | 35.5 → 11.5 |
| append_100000 | INCOMPLETE (budget) | 402.88 [237.3-479.4] | 423.22 [264.7-486.9] | - | ×0.95 | - |
| seq_i32 | 116.02 [112.2-124.0] | 3.96 [3.9-4.8] | 4.83 [4.4-5.3] | ×29.3 | ×0.82 | 7.0 → 5.7 |
| seq_record | 2208.32 [1353.1-2661.1] | 10.62 [6.1-13.2] | 11.71 [6.9-13.8] | ×207.9 | ×0.91 | 10.7 → 6.1 |
| seq_text_large | 53.28 [51.5-65.7] | 1.44 [1.4-1.6] | 1.61 [1.6-1.8] | ×37.0 | ×0.89 | 9.8 → 5.9 |
| seq_mixed_depth | 1283.25 [973.0-1455.3] | 12.77 [8.2-13.1] | 13.34 [8.3-14.4] | ×100.5 | ×0.96 | 19.4 → 10.6 |
| nested_loops | 31.78 [30.8-40.8] | 3.98 [3.9-4.9] | 5.32 [4.4-5.7] | ×8.0 | ×0.75 | 6.9 → 5.8 |
| function_accumulate | 12.21 [11.7-13.0] | 1.82 [1.8-1.9] | 2.11 [2.1-2.3] | ×6.7 | ×0.86 | 6.4 → 5.7 |
| map_set_loop | 47.43 [44.3-52.8] | 51.91 [47.2-54.9] | 61.05 [43.3-65.7] | ×0.9 | ×0.85 | 5.9 → 5.9 |
| record_with_loop | 6.75 [6.0-7.8] | 6.64 [5.2-8.0] | 6.15 [5.6-9.1] | ×1.0 | ×1.08 | 5.7 → 5.7 |

## Ubuntu 24.04.5 LTS (WSL2)

### Correctness (reference = pristine)

| Phase | Passed / total | Notes |
|---|---:|---|
| A: 224 fixtures × {impl, p3} | 448 / 448 | - |
| B: failure sweeps (every Steps / Calls / Frames limit) | 1819 / 1819 | |
| C: alternating programs in one process | 36 / 36 | |
| D: byte mutations (verifier-rejected and admitted) | 120 / 120 | |
| E: 20 repeats in one process | 8 / 8 | |

### Performance (median warm ms; [min-max]; peak memory MB)

| Workload | pristine | impl | p3 (research) | pristine / impl | impl / p3 | peak mem pristine → impl |
|---|---:|---:|---:|---:|---:|---|
| lexer_720B | 40.54 [38.3-43.7] | 16.22 [15.8-17.6] | 17.06 [15.8-17.4] | ×2.5 | ×0.95 | 19.7 → 19.7 |
| lexer_2880B | 476.83 [448.6-527.8] | 69.21 [66.9-74.2] | 71.33 [70.1-77.1] | ×6.9 | ×0.97 | 19.7 → 19.7 |
| lexer_11520B | 6671.09 [5779.0-8430.9] | 181.34 [165.4-302.7] | 179.79 [166.5-300.4] | ×36.8 | ×1.01 | 26.8 → 19.7 |
| lexer_23040B | 28702.44 [24938.9-33656.2] | 415.91 [369.3-640.2] | 511.80 [391.4-705.9] | ×69.0 | ×0.81 | 52.1 → 19.7 |
| lexer_46080B | 124392.23 [115285.9-132017.6] | 942.58 [888.5-1553.5] | 1393.03 [870.6-1574.0] | ×132.0 | ×0.68 | 100.5 → 20.0 |
| b_parse_8_orig | 96.15 [91.0-116.8] | 12.51 [10.7-16.1] | 13.36 [11.2-17.9] | ×7.7 | ×0.94 | 19.7 → 19.7 |
| b_parse_8_fastlex | 100.86 [89.6-107.1] | 11.25 [9.2-14.2] | 11.34 [9.7-12.0] | ×9.0 | ×0.99 | 19.7 → 19.7 |
| b_parse_16_orig | 238.06 [201.5-391.4] | 13.39 [11.9-23.7] | 13.59 [12.1-32.4] | ×17.8 | ×0.99 | 19.9 → 19.9 |
| b_parse_16_fastlex | 242.19 [217.5-369.9] | 12.98 [11.6-20.1] | 18.16 [13.8-23.0] | ×18.7 | ×0.72 | 19.9 → 19.9 |
| b_parse_32_orig | 811.03 [785.2-1265.3] | 24.90 [24.0-39.7] | 28.95 [24.3-43.3] | ×32.6 | ×0.86 | 19.9 → 19.9 |
| b_parse_32_fastlex | 822.44 [770.1-1294.4] | 20.84 [19.8-27.0] | 21.39 [20.7-23.6] | ×39.5 | ×0.97 | 19.9 → 19.9 |
| b_eval_loop_10_orig | 14.48 [11.7-16.4] | 7.82 [6.2-8.4] | 7.64 [6.7-9.7] | ×1.9 | ×1.02 | 19.9 → 19.9 |
| b_eval_loop_20_orig | 16.84 [15.8-25.0] | 9.37 [8.9-11.5] | 10.99 [9.8-12.2] | ×1.8 | ×0.85 | 19.9 → 19.9 |
| b_eval_loop_40_orig | 29.12 [25.7-37.1] | 17.68 [14.6-21.0] | 18.64 [15.2-22.3] | ×1.6 | ×0.95 | 19.9 → 19.9 |
| b_eval_loop_80_orig | 54.36 [39.8-73.9] | 30.55 [26.0-48.7] | 27.91 [26.9-50.0] | ×1.8 | ×1.09 | 19.9 → 19.9 |
| append_100 | 0.29 [0.3-0.4] | 0.16 [0.2-0.2] | 0.19 [0.2-0.2] | ×1.8 | ×0.85 | 19.0 → 19.0 |
| append_1000 | 21.75 [21.4-32.2] | 1.45 [1.4-1.5] | 1.66 [1.6-1.8] | ×15.0 | ×0.88 | 19.0 → 19.0 |
| append_10000 | 1995.53 [1899.1-2354.7] | 16.81 [15.4-26.9] | 17.28 [16.6-33.4] | ×118.7 | ×0.97 | 19.4 → 19.4 |
| append_50000 | 84427.83 [81508.3-86162.9] | 83.28 [79.5-97.0] | 95.83 [94.7-147.3] | ×1013.8 | ×0.87 | 36.9 → 19.4 |
| append_100000 | 348415.39 [348415.4-348415.4] | 260.88 [249.5-284.8] | 297.55 [289.7-308.1] | ×1335.5 | ×0.88 | 71.4 → 19.4 |
| seq_i32 | 140.26 [131.5-151.3] | 4.56 [4.4-5.4] | 5.11 [4.9-6.5] | ×30.7 | ×0.89 | 19.4 → 19.4 |
| seq_record | 959.04 [922.2-1263.2] | 3.84 [3.7-4.6] | 4.46 [4.0-5.1] | ×250.0 | ×0.86 | 19.7 → 19.7 |
| seq_text_large | 39.06 [36.8-43.2] | 0.83 [0.7-1.5] | 0.85 [0.8-1.0] | ×47.1 | ×0.98 | 19.4 → 19.4 |
| seq_mixed_depth | 634.32 [584.3-695.4] | 3.92 [3.8-4.8] | 4.60 [4.0-5.6] | ×161.9 | ×0.85 | 19.7 → 19.7 |
| nested_loops | 24.05 [18.1-39.4] | 1.61 [1.5-2.9] | 1.98 [1.7-4.0] | ×15.0 | ×0.81 | 19.7 → 19.7 |
| function_accumulate | 9.08 [8.4-10.3] | 1.18 [1.2-1.6] | 1.33 [1.3-2.0] | ×7.7 | ×0.89 | 19.7 → 19.7 |
| map_set_loop | 8.40 [7.8-8.8] | 9.55 [8.6-10.1] | 8.90 [8.3-9.4] | ×0.9 | ×1.07 | 19.7 → 19.7 |
| record_with_loop | 2.08 [2.0-2.4] | 1.94 [1.9-2.7] | 2.10 [2.0-2.7] | ×1.1 | ×0.93 | 19.7 → 19.7 |

## Copy and COW counters (scratch-instrumented copy of this branch; deterministic, platform-independent)

| Workload | COW unique / updates | Shared fallback (copied) | Consume operations | Plan construction ms (instrumented) |
|---|---:|---:|---:|---:|
| append_100 | 99 / 100 | 1 | 300 | 0.0355 |
| append_1000 | 999 / 1000 | 1 | 3000 | 0.0321 |
| append_10000 | 9999 / 10000 | 1 | 30000 | 0.0456 |
| append_50000 | 49999 / 50000 | 1 | 150000 | 0.0357 |
| append_100000 | 99999 / 100000 | 1 | 300000 | 0.0323 |
| seq_i32 | 1999 / 2000 | 1 | 6000 | 0.0170 |
| seq_quad | 1999 / 2000 | 1 | 6000 | 0.0328 |
| seq_text_short | 1999 / 2000 | 1 | 6000 | 0.0190 |
| seq_text_large | 499 / 500 | 1 | 1500 | 0.0279 |
| seq_seq_i32 | 1999 / 2000 | 1 | 6000 | 0.0242 |
| seq_tuple | 1999 / 2000 | 1 | 6000 | 0.0334 |
| seq_record | 1999 / 2000 | 1 | 6000 | 0.0238 |
| seq_mixed_depth | 999 / 2000 | 1001 | 6000 | 0.0231 |
| map_set_loop | 0 / 0 | 0 | 0 | 0.0133 |
| record_with_loop | 0 / 0 | 0 | 0 | 0.0165 |
| lexer_720B | 239 / 241 | 2 | 720 | 0.4915 |
| lexer_2880B | 959 / 961 | 2 | 2880 | 0.2885 |
| lexer_11520B | 3839 / 3841 | 2 | 11520 | 0.3244 |
| lexer_23040B | 7679 / 7681 | 2 | 23040 | 0.4495 |
| lexer_46080B | 15359 / 15361 | 2 | 46080 | 0.4571 |
| b_parse_8_orig | 78 / 81 | 3 | 240 | 1.4795 |
| b_parse_8_fastlex | 78 / 81 | 3 | 240 | 1.0863 |
| b_parse_16_orig | 158 / 161 | 3 | 480 | 0.7548 |
| b_parse_16_fastlex | 158 / 161 | 3 | 480 | 1.4912 |
| b_parse_32_orig | 318 / 321 | 3 | 960 | 0.8288 |
| b_parse_32_fastlex | 318 / 321 | 3 | 960 | 0.8797 |
| b_eval_loop_10_orig | 31 / 67 | 36 | 135 | 1.8756 |
| b_eval_loop_10_fastlex | 31 / 67 | 36 | 135 | 2.9276 |
| b_eval_loop_20_orig | 41 / 107 | 66 | 195 | 1.6510 |
| b_eval_loop_20_fastlex | 41 / 107 | 66 | 195 | 1.6827 |
| b_eval_loop_40_orig | 61 / 187 | 126 | 315 | 1.7133 |
| b_eval_loop_40_fastlex | 61 / 187 | 126 | 315 | 1.7058 |
| b_eval_loop_80_orig | 101 / 347 | 246 | 555 | 1.6448 |
| b_eval_loop_80_fastlex | 101 / 347 | 246 | 555 | 2.3291 |

## Retained memory across repeated executions (Windows, peak working set)

| Workload | 1 run | 10 runs | 40 runs |
|---|---:|---:|---:|
| lexer_11520B | 9.50 MB | 10.30 MB | 10.34 MB |
| b_parse_32_orig | 6.19 MB | 6.58 MB | 6.65 MB |
| append_10000 | 7.22 MB | 7.24 MB | 7.23 MB |

Memory levels off after the first runs, with no growth trend from 10 to 40 runs: no retained-memory leak observed.

## Open item: `map_set_loop` (not optimized by this change)

`map_set_loop` measured as follows:
- **Qualification runs:** ×0.9 (Windows) and ×0.88 (Ubuntu) of pristine speed.
- **Repeated A/B runs on Ubuntu:** ×1.04–1.10 slower.
- **Noise:** the A/B spread was ±70%, and the sample ranges overlap.

A slowdown of up to about 8% on this Map-heavy microbenchmark is therefore **possible but not resolvable above noise**. The bisect points at the `Arc` representation commit rather than the consumption plan. `record_with_loop`, also not optimized, improves by 7–10%. The item is reported for OWNER review.

## SHF-P3-FIX1: planner startup qualification (Windows 11, native)

**Setup.**
- Release builds from identical SemCode.
- Variants: `pristine` (`0373bec5`), `p3_orig` (`08637f9`, before FIX1) and `fix1`.
- 7 interleaved rounds; median wall time per process, including verify, plan and run.
- Synthetic workloads run with `--steps 1`.

**Planner time.** The `plan ms` columns come from separate, scratch-instrumented builds of the same trees. They are noisy and are **not** additive to wall time.

**Earlier sections.** The tables above were measured on the pre-FIX1 tree and are kept as historical evidence.

| Workload | Instructions | Accepted triples (orig = fix1) | pristine ms | p3_orig ms | fix1 ms [min-max] | fix1 / pristine | p3_orig / fix1 | plan ms orig → fix1 (instr.) | Outcomes identical |
|---|---:|---:|---:|---:|---:|---:|---:|---|---|
| acc_400x1 --steps 1 | 1,606 | 400 | 33.9 | 36.2 | 35.7 [31-40] | ×1.06 | ×1.01 | 3.1 → 0.8 | yes |
| acc_1300x1 --steps 1 | 5,206 | 1300 | 30.4 | 66.3 | 32.1 [31-36] | ×1.06 | ×2.06 | 29.7 → 1.9 | yes |
| acc_1300x16 --steps 1 | 83,281 | 20800 | 79.9 | 580.0 | 114.2 [112-202] | ×1.43 | ×5.08 | 781.5 → 117.2 | yes |
| acc_1300x64 --steps 1 | 333,121 | 83200 | 294.1 | 3400.0 | 383.8 [331-689] | ×1.30 | ×8.86 | 3186.2 → 166.5 | yes |
| rejected_1300x16 --steps 1 | 104,081 | 0 | 71.9 | 70.4 | 70.7 [69-73] | ×0.98 | ×1.00 | 0.0 → 0.0 | yes |
| nopush_2000x16 --steps 1 | 128,065 | 0 | 89.4 | 88.9 | 89.7 [86-92] | ×1.00 | ×0.99 | 0.0 → 0.0 | yes |
| acc_1300x64_repeat10 --steps 1 --repeat 10 | 333,121 | 832000 | 3515.0 | 32030.5 | 5339.5 [4327-5672] | ×1.52 | ×6.00 | 1994.5 → 138.3 | yes |
| small_copy_by_value  | 64 | 1 | 42.2 | 42.8 | 44.7 [28-49] | ×1.06 | ×0.96 | 0.0 → 0.1 | yes |
| lexer_46080B  | 1,187 | 5 | 153437.9 | 2516.4 | 2501.2 [1458-3924] | ×0.02 | ×1.01 | 0.2 → 0.5 | yes |
| b_parse_32_orig  | 2,292 | 10 | 1437.1 | 70.8 | 69.1 [66-113] | ×0.05 | ×1.03 | 0.6 → 1.5 | yes |
| b_eval_loop_40_orig  | 5,244 | 18 | 113.1 | 92.0 | 91.6 [89-100] | ×0.81 | ×1.01 | 1.6 → 3.3 | yes |
| map_set_loop  | 26 | 0 | 104.0 | 109.3 | 98.4 [92-106] | ×0.95 | ×1.11 | 0.0 → 0.0 | yes |

**Reading.**
- **Startup amplification removed.** The largest case went from ×11.6 of pristine to ×1.30.
- **Linear growth.** fix1's wall time over pristine grows linearly with code size: about 2 ms at 5k instructions, 34 ms at 83k and 90 ms at 333k.
- **Instrumented planner times** are noisy and only indicative.
- **Small functions are a little slower to plan.** On small, ordinary functions the indexed planner's fixed setup cost (hash indexes) makes the instrumented plan time about 2× the old planner's: under 2 ms per execution, for example 0.6 → 1.5 ms on `b_parse_32`. This is not visible in wall time.
- **Counts in the `repeat10` row** are summed over 10 executions in one process.
- **No regression elsewhere.** No-push and rejected-candidate programs show no planner overhead. Lexer, parser, evaluator and `map_set_loop` are unchanged within noise.
- **Small programs.** The difference is inside the sample spread: inconclusive.
- **Accepted sites** are identical in every row. The 224-fixture corpus plus 26 red-team artifacts (114,768 accepted triples) give identical site maps under both planners (scratch oracle run).
