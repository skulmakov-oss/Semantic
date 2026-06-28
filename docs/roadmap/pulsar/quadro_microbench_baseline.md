# PULSAR-Q4 — Quadro Microbench Baseline

## Status

LOCAL DIAGNOSTIC BASELINE / NOT A PUBLIC PERFORMANCE CLAIM

## Scope

This note records the first local microbenchmark baseline for the Pulsar / Quadro packed-state engine in `ton618-core`.

Workloads:

- `qreg_merge`
- `qreg_intersect`
- `qreg_inverse`
- `qreg_masks_all`
- `qreg_calc_delta`
- `qbank_merge_inplace`
- `qbank_intersect_inplace`
- `qbank_inverse_inplace`
- `qbank_calc_deltas_soa`
- `baseline_vec_u8_delta`

## Machine / Environment

- date/time: `2026-06-29T01:20:33.5687380+05:00`
- OS: Windows
- CPU: `Intel(R) Core(TM) i5-9300H CPU @ 2.40GHz`
- cores: `4`
- logical processors: `8`
- max clock: `2400 MHz`
- Rust: `rustc 1.93.1 (01f6ddf75 2026-02-11)`
- Cargo: `cargo 1.93.1 (083ac5135 2025-12-15)`
- target triple: `x86_64-pc-windows-msvc`
- branch: `pulsar/q4-quadro-bench-baseline`
- HEAD: `443785a685570c06e1382005e4e6e0625b9563e9 bench(pulsar): add Quadro microbench harness (#1196)`
- command: `cargo run -p ton618-core --example quadro_bench --release`
- mode: `release`
- features: default features via `cargo run`
- RUSTFLAGS: not set

## Commands

```powershell
Get-Date -Format o
rustc --version
rustc -vV
cargo --version
git show -s --format="%H %s" HEAD
$PSVersionTable.PSVersion.ToString()
Get-CimInstance Win32_Processor | Select-Object Name,NumberOfCores,NumberOfLogicalProcessors,MaxClockSpeed
cargo run -p ton618-core --example quadro_bench --release
cargo run -p ton618-core --example quadro_bench --release
cargo run -p ton618-core --example quadro_bench --release
```

## Raw Runs

Run 1:

```text
Pulsar Quadro microbench
build: release
note: local numbers only; not a public performance claim

name                                iters    elapsed_ms    ns/iter         regs/s      quadits/s
qreg_merge                        5000000        12.147       2.43   411620880.70 13171868182.53
qreg_intersect                    5000000        12.228       2.45   408907644.12 13085044611.82
qreg_inverse                      5000000        28.316       5.66   176580483.62  5650575475.80
qreg_masks_all                    5000000        49.848       9.97   100304323.32  3209738346.14
qreg_calc_delta                   5000000        60.918      12.18    82077816.34  2626490122.76
qbank_merge_inplace                200000     11389.889   56949.45     4495214.97   143846879.07
qbank_intersect_inplace            200000      8867.778   44338.89     5773712.49   184758799.61
qbank_inverse_inplace              200000      4898.895   24494.48    10451336.26   334442760.37
qbank_calc_deltas_soa              200000      8216.594   41082.97     6231292.81   199401369.92
baseline_vec_u8_delta              200000      9376.459   46882.30     5460483.48   174735471.30
```

Run 2:

```text
Pulsar Quadro microbench
build: release
note: local numbers only; not a public performance claim

name                                iters    elapsed_ms    ns/iter         regs/s      quadits/s
qreg_merge                        5000000        12.148       2.43   411603938.23 13171326023.25
qreg_intersect                    5000000        12.511       2.50   399638726.59 12788439250.92
qreg_inverse                      5000000        51.294      10.26    97477287.79  3119273209.34
qreg_masks_all                    5000000        33.700       6.74   148366631.75  4747732216.03
qreg_calc_delta                   5000000        47.401       9.48   105483451.76  3375470456.19
qbank_merge_inplace                200000     11459.157   57295.78     4468042.54   142977361.13
qbank_intersect_inplace            200000      8709.956   43549.78     5878330.57   188106578.13
qbank_inverse_inplace              200000      4976.728   24883.64    10287883.10   329212259.20
qbank_calc_deltas_soa              200000      8220.465   41102.33     6228357.83   199307450.40
baseline_vec_u8_delta              200000      9290.484   46452.42     5511015.18   176352485.74
```

Run 3:

```text
Pulsar Quadro microbench
build: release
note: local numbers only; not a public performance claim

name                                iters    elapsed_ms    ns/iter         regs/s      quadits/s
qreg_merge                        5000000        11.919       2.38   419487721.59 13423607091.02
qreg_intersect                    5000000        12.104       2.42   413103647.71 13219316726.57
qreg_inverse                      5000000        11.786       2.36   424246538.15 13575889220.74
qreg_masks_all                    5000000        32.549       6.51   153614550.37  4915665611.85
qreg_calc_delta                   5000000        62.711      12.54    79730955.86  2551390587.60
qbank_merge_inplace                200000     11435.365   57176.83     4477338.35   143274827.06
qbank_intersect_inplace            200000      8707.957   43539.78     5879680.27   188149768.55
qbank_inverse_inplace              200000      4895.791   24478.95    10457963.62   334654835.93
qbank_calc_deltas_soa              200000      8255.830   41279.15     6201678.31   198453705.99
baseline_vec_u8_delta              200000      9459.973   47299.86     5412277.83   173192890.64
```

## Observed Range

Approximate range across the three local runs:

| Workload | Approx range | Note |
|---|---:|---|
| `qreg_merge` | `2.38-2.43 ns/iter` | stable |
| `qreg_intersect` | `2.42-2.50 ns/iter` | stable |
| `qreg_inverse` | `2.36-10.26 ns/iter` | noisy across runs |
| `qreg_masks_all` | `6.51-9.97 ns/iter` | noisy across runs |
| `qreg_calc_delta` | `9.48-12.54 ns/iter` | moderately noisy |
| `qbank_merge_inplace` | `56949.45-57295.78 ns/iter` | stable |
| `qbank_intersect_inplace` | `43539.78-44338.89 ns/iter` | stable |
| `qbank_inverse_inplace` | `24478.95-24883.64 ns/iter` | stable |
| `qbank_calc_deltas_soa` | `41082.97-41279.15 ns/iter` | stable |
| `baseline_vec_u8_delta` | `46452.42-47299.86 ns/iter` | stable |

## Interpretation

The per-reg `qreg_*` workloads are short enough that they show visible run-to-run noise on this machine. The `qbank_*` loops and the baseline Vec<u8> delta loop are much more stable and are the better reference band for future tuning.

The benchmark is useful as a first local measurement baseline and as a regression check for later slices.

## Non-Claims

This baseline does not claim production performance.
This baseline does not claim release readiness.
This baseline does not compare against optimized external systems.
This baseline does not widen the active Core Trust Freeze contour.

## Next Actions

- keep this baseline as the reference point for future Pulsar hot-path work;
- review `QuadroReg::masks_all`, `QuadroReg::calc_delta`, and the `QuadroBank` batch loops before any optimization;
- keep SIMD, prefetch, and unsafe expansion out of this slice;
- use the next Pulsar slice for scalar cleanup first.
