use std::fmt::Write as _;
use std::hint::black_box;
use std::time::Instant;

use semantic_core_backend::{
    detect_backend_caps, join_reg32, join_tile128, select_backend, BackendCaps, BackendKind,
};
use semantic_core_capsule::CoreCapsule;
use semantic_core_exec::{CoreConfig, CoreFunction, CoreProgram, CoreValue, Fx, Instr, RegId};
use semantic_core_quad::{
    QuadMask32, QuadState, QuadTile128, QuadTileBank, QuadroBank, QuadroReg32,
};
use semantic_core_runtime::{FunctionId, SymbolId};

pub fn run_benchmark(name: &str) -> Result<String, String> {
    match name {
        "quad-reg" => Ok(bench_quad_reg()),
        "quad-v1" => Ok(bench_quad_v1()),
        "tile" => Ok(bench_tile()),
        "exec" => Ok(bench_exec()),
        "all" => Ok(format!(
            "{}\n{}\n{}",
            bench_quad_reg(),
            bench_tile(),
            bench_exec()
        )),
        "caps" => Ok(format_caps_report(BackendKind::Auto, detect_backend_caps())),
        _ => Err(format!("unknown benchmark '{name}'")),
    }
}

pub fn format_caps_report(requested: BackendKind, caps: BackendCaps) -> String {
    let selected = select_backend(requested, caps);
    let arch = std::env::consts::ARCH;
    let yes = |value: bool| if value { "yes" } else { "no" };
    format!(
        "CPU backend report:\n  arch: {arch}\n  popcnt: {}\n  bmi1: {}\n  bmi2: {}\n  avx2: {}\n  avx512: {}\n  neon: {}\n  sve: {}\n  selected backend: {}",
        yes(caps.has_popcnt),
        yes(caps.has_bmi1),
        yes(caps.has_bmi2),
        yes(caps.has_avx2),
        yes(caps.has_avx512),
        yes(caps.has_neon),
        yes(caps.has_sve),
        match selected {
            BackendKind::Scalar => "scalar",
            BackendKind::Auto => "auto",
        }
    )
}

fn bench_quad_reg() -> String {
    let iterations = 100_000u64;
    let regs_per_iter = 64u64;
    let quadits_per_reg = QuadroReg32::LANES as u64;
    let start = Instant::now();
    let mut dst = vec![reg_filled(QuadState::T); regs_per_iter as usize];
    let src = vec![reg_filled(QuadState::F); regs_per_iter as usize];
    for _ in 0..iterations {
        join_reg32(BackendKind::Auto, &mut dst, &src);
    }
    format_bench_line(
        "quad-reg",
        iterations,
        start.elapsed().as_nanos() as u64,
        &[
            ("regs/s", iterations * regs_per_iter),
            ("quadits/s", iterations * regs_per_iter * quadits_per_reg),
        ],
    )
}

fn bench_tile() -> String {
    let iterations = 50_000u64;
    let tiles_per_iter = 32u64;
    let quadits_per_tile = QuadTile128::LANES as u64;
    let start = Instant::now();
    let mut dst = vec![tile_filled(QuadState::T); tiles_per_iter as usize];
    let src = vec![tile_filled(QuadState::F); tiles_per_iter as usize];
    for _ in 0..iterations {
        join_tile128(BackendKind::Auto, &mut dst, &src);
    }
    format_bench_line(
        "tile",
        iterations,
        start.elapsed().as_nanos() as u64,
        &[
            ("tiles/s", iterations * tiles_per_iter),
            ("quadits/s", iterations * tiles_per_iter * quadits_per_tile),
        ],
    )
}

fn bench_exec() -> String {
    let iterations = 10_000u64;
    let start = Instant::now();
    let capsule = CoreCapsule::new(CoreConfig::default());
    let program = sample_program();
    let mut last = CoreValue::Unit;
    let mut last_fuel_used = 0u64;
    for _ in 0..iterations {
        let result = capsule
            .run(&program)
            .expect("sample program should execute");
        last = result.return_value;
        last_fuel_used = result.fuel_used;
    }
    let mut text = format_bench_line(
        "exec",
        iterations,
        start.elapsed().as_nanos() as u64,
        &[
            ("instructions/s", iterations * last_fuel_used),
            ("fuel/s", iterations * last_fuel_used),
        ],
    );
    let _ = write!(text, "\nlast: {:?}", last);
    text
}

const QUAD_V1_ITERATIONS: u64 = 100_000;
const REG_BANK_LEN: usize = 64;
const TILE_BANK_LEN: usize = 32;
const QUAD_V1_RAW_SAMPLES: [u64; 7] = [
    0x0000_0000_0000_0000,
    0xFFFF_FFFF_FFFF_FFFF,
    0x5555_5555_5555_5555,
    0xAAAA_AAAA_AAAA_AAAA,
    0x0123_4567_89AB_CDEF,
    0xE4E4_E4E4_E4E4_E4E4,
    0xBADC_0FFE_DEAD_BEEF,
];

struct BenchSample {
    name: &'static str,
    ops: u64,
    nanos: u64,
    checksum: u64,
}

fn bench_quad_v1() -> String {
    bench_quad_v1_with_iterations(QUAD_V1_ITERATIONS)
}

fn bench_quad_v1_with_iterations(iterations: u64) -> String {
    let iterations = iterations.max(1);
    let reg_not_scalar = time_reg_unary(
        "quad-v1/reg-not-scalar",
        iterations,
        QuadroReg32::map_not_scalar,
    );
    let reg_not_swar = time_reg_unary(
        "quad-v1/reg-not-swar",
        iterations,
        QuadroReg32::map_not_swar,
    );
    let reg_not_default =
        time_reg_unary("quad-v1/reg-not-default", iterations, QuadroReg32::map_not);
    let reg_and_scalar = time_reg_binary(
        "quad-v1/reg-and-scalar",
        iterations,
        QuadroReg32::map_and_scalar,
    );
    let reg_and_swar = time_reg_binary(
        "quad-v1/reg-and-swar",
        iterations,
        QuadroReg32::map_and_swar,
    );
    let reg_and_default =
        time_reg_binary("quad-v1/reg-and-default", iterations, QuadroReg32::map_and);
    let tile_xor = time_tile_xor(iterations);
    let reg_bank_xor = time_reg_bank_xor(iterations);
    let tile_bank_xor = time_tile_bank_xor(iterations);
    let mask_roundtrip = time_mask_roundtrip(iterations);
    let (mask_iteration, visited_lanes) = time_mask_iteration(iterations);

    let build_mode = if cfg!(debug_assertions) {
        "debug"
    } else {
        "release"
    };
    let mut text = format!(
        "quad-v1 benchmark\narchitecture: {}\nbuild mode: {build_mode}\niteration count: {iterations}\npolicy: observational relative timings; no performance threshold",
        std::env::consts::ARCH
    );
    for sample in [
        &reg_not_scalar,
        &reg_not_swar,
        &reg_not_default,
        &reg_and_scalar,
        &reg_and_swar,
        &reg_and_default,
    ] {
        let _ = write!(text, "\n{}", format_quad_v1_sample(sample, &[]));
    }
    let _ = write!(
        text,
        "\n{}",
        format_quad_v1_sample(
            &tile_xor,
            &[
                ("tiles/s", tile_xor.ops),
                ("quadits/s", tile_xor.ops * QuadTile128::LANES as u64),
            ],
        )
    );
    let _ = write!(
        text,
        "\n{}",
        format_quad_v1_sample(
            &reg_bank_xor,
            &[
                ("registers/s", reg_bank_xor.ops * REG_BANK_LEN as u64),
                (
                    "quadits/s",
                    reg_bank_xor.ops * REG_BANK_LEN as u64 * QuadroReg32::LANES as u64,
                ),
            ],
        )
    );
    let _ = write!(
        text,
        "\n{}",
        format_quad_v1_sample(
            &tile_bank_xor,
            &[
                ("tiles/s", tile_bank_xor.ops * TILE_BANK_LEN as u64),
                (
                    "quadits/s",
                    tile_bank_xor.ops * TILE_BANK_LEN as u64 * QuadTile128::LANES as u64,
                ),
            ],
        )
    );
    let _ = write!(
        text,
        "\n{}",
        format_quad_v1_sample(&mask_roundtrip, &[("conversions/s", mask_roundtrip.ops)],)
    );
    let _ = write!(
        text,
        "\n{}",
        format_quad_v1_sample(&mask_iteration, &[("visited-lanes/s", visited_lanes)],)
    );
    let _ = write!(
        text,
        "\nquad-v1/relative-reg-not: swar_vs_scalar={:.4} default_vs_scalar={:.4}",
        relative_ratio(&reg_not_scalar, &reg_not_swar),
        relative_ratio(&reg_not_scalar, &reg_not_default),
    );
    let _ = write!(
        text,
        "\nquad-v1/relative-reg-and: swar_vs_scalar={:.4} default_vs_scalar={:.4}",
        relative_ratio(&reg_and_scalar, &reg_and_swar),
        relative_ratio(&reg_and_scalar, &reg_and_default),
    );
    text
}

fn time_reg_unary(
    name: &'static str,
    iterations: u64,
    operation: fn(QuadroReg32) -> QuadroReg32,
) -> BenchSample {
    let start = Instant::now();
    let mut checksum = 0u64;
    for index in 0..iterations {
        let input = QuadroReg32::from_raw(QUAD_V1_RAW_SAMPLES[index as usize % 7]);
        let result = operation(black_box(input));
        checksum = mix_checksum(checksum, black_box(result.raw()));
    }
    BenchSample {
        name,
        ops: iterations,
        nanos: start.elapsed().as_nanos() as u64,
        checksum: black_box(checksum),
    }
}

fn time_reg_binary(
    name: &'static str,
    iterations: u64,
    operation: fn(QuadroReg32, QuadroReg32) -> QuadroReg32,
) -> BenchSample {
    let start = Instant::now();
    let mut checksum = 0u64;
    for index in 0..iterations {
        let lhs = QuadroReg32::from_raw(QUAD_V1_RAW_SAMPLES[index as usize % 7]);
        let rhs = QuadroReg32::from_raw(QUAD_V1_RAW_SAMPLES[(index as usize + 3) % 7]);
        let result = operation(black_box(lhs), black_box(rhs));
        checksum = mix_checksum(checksum, black_box(result.raw()));
    }
    BenchSample {
        name,
        ops: iterations,
        nanos: start.elapsed().as_nanos() as u64,
        checksum: black_box(checksum),
    }
}

fn time_tile_xor(iterations: u64) -> BenchSample {
    let lhs = QuadTile128::from_regs([
        QuadroReg32::from_raw(QUAD_V1_RAW_SAMPLES[0]),
        QuadroReg32::from_raw(QUAD_V1_RAW_SAMPLES[1]),
        QuadroReg32::from_raw(QUAD_V1_RAW_SAMPLES[2]),
        QuadroReg32::from_raw(QUAD_V1_RAW_SAMPLES[3]),
    ]);
    let rhs = QuadTile128::from_regs([
        QuadroReg32::from_raw(QUAD_V1_RAW_SAMPLES[4]),
        QuadroReg32::from_raw(QUAD_V1_RAW_SAMPLES[5]),
        QuadroReg32::from_raw(QUAD_V1_RAW_SAMPLES[6]),
        QuadroReg32::from_raw(QUAD_V1_RAW_SAMPLES[0]),
    ]);
    let start = Instant::now();
    let mut checksum = 0u64;
    for _ in 0..iterations {
        let result = black_box(lhs).map_xor(black_box(rhs));
        checksum = mix_checksum(checksum, black_box(tile_checksum(result)));
    }
    BenchSample {
        name: "quad-v1/tile-xor-default",
        ops: iterations,
        nanos: start.elapsed().as_nanos() as u64,
        checksum: black_box(checksum),
    }
}

fn time_reg_bank_xor(iterations: u64) -> BenchSample {
    let lhs_regs = std::array::from_fn(|index| {
        QuadroReg32::from_raw(QUAD_V1_RAW_SAMPLES[index % QUAD_V1_RAW_SAMPLES.len()])
    });
    let rhs_regs = std::array::from_fn(|index| {
        QuadroReg32::from_raw(QUAD_V1_RAW_SAMPLES[(index + 3) % QUAD_V1_RAW_SAMPLES.len()])
    });
    let mut lhs = QuadroBank::<REG_BANK_LEN>::from_array(lhs_regs);
    let rhs = QuadroBank::<REG_BANK_LEN>::from_array(rhs_regs);
    let start = Instant::now();
    let mut checksum = 0u64;
    for index in 0..iterations {
        black_box(&mut lhs).map_xor_inplace(black_box(&rhs));
        checksum = mix_checksum(
            checksum,
            black_box(lhs.as_array()[index as usize % REG_BANK_LEN].raw()),
        );
    }
    BenchSample {
        name: "quad-v1/reg-bank-xor-inplace",
        ops: iterations,
        nanos: start.elapsed().as_nanos() as u64,
        checksum: black_box(checksum),
    }
}

fn time_tile_bank_xor(iterations: u64) -> BenchSample {
    let lhs_tiles = std::array::from_fn(tile_from_offset);
    let rhs_tiles = std::array::from_fn(|index| tile_from_offset(index + 3));
    let mut lhs = QuadTileBank::<TILE_BANK_LEN>::from_array(lhs_tiles);
    let rhs = QuadTileBank::<TILE_BANK_LEN>::from_array(rhs_tiles);
    let start = Instant::now();
    let mut checksum = 0u64;
    for index in 0..iterations {
        black_box(&mut lhs).map_xor_inplace(black_box(&rhs));
        checksum = mix_checksum(
            checksum,
            black_box(tile_checksum(
                lhs.as_array()[index as usize % TILE_BANK_LEN],
            )),
        );
    }
    BenchSample {
        name: "quad-v1/tile-bank-xor-inplace",
        ops: iterations,
        nanos: start.elapsed().as_nanos() as u64,
        checksum: black_box(checksum),
    }
}

fn time_mask_roundtrip(iterations: u64) -> BenchSample {
    let masks = [
        QuadMask32::try_new(0x0000_0000).expect("valid mask"),
        QuadMask32::try_new(0x0000_0001).expect("valid mask"),
        QuadMask32::try_new(0x8000_0000).expect("valid mask"),
        QuadMask32::try_new(0x8000_0001).expect("valid mask"),
        QuadMask32::try_new(0xFFFF_FFFF).expect("valid mask"),
    ];
    let start = Instant::now();
    let mut checksum = 0u64;
    for index in 0..iterations {
        let mask = black_box(masks[index as usize % masks.len()]);
        let roundtrip = mask
            .to_physical()
            .try_to_lane()
            .expect("physical mask from a lane mask must be valid");
        checksum = mix_checksum(checksum, black_box(roundtrip.raw()));
    }
    BenchSample {
        name: "quad-v1/mask-roundtrip",
        ops: iterations,
        nanos: start.elapsed().as_nanos() as u64,
        checksum: black_box(checksum),
    }
}

fn time_mask_iteration(iterations: u64) -> (BenchSample, u64) {
    let masks = [
        QuadMask32::try_new(0x0000_0000).expect("valid mask"),
        QuadMask32::try_new(0x0000_0001).expect("valid mask"),
        QuadMask32::try_new(0x8000_0000).expect("valid mask"),
        QuadMask32::try_new(0x5555_AAAA).expect("valid mask"),
        QuadMask32::try_new(0xFFFF_FFFF).expect("valid mask"),
    ];
    let start = Instant::now();
    let mut checksum = 0u64;
    let mut visited_lanes = 0u64;
    for index in 0..iterations {
        let mask = black_box(masks[index as usize % masks.len()]);
        checksum = mix_checksum(checksum, mask.raw());
        for lane in mask.iter() {
            checksum = mix_checksum(checksum, black_box(lane as u64));
            visited_lanes += 1;
        }
    }
    (
        BenchSample {
            name: "quad-v1/mask-iteration",
            ops: iterations,
            nanos: start.elapsed().as_nanos() as u64,
            checksum: black_box(checksum),
        },
        visited_lanes,
    )
}

fn tile_from_offset(offset: usize) -> QuadTile128 {
    QuadTile128::from_regs(std::array::from_fn(|index| {
        QuadroReg32::from_raw(QUAD_V1_RAW_SAMPLES[(offset + index) % QUAD_V1_RAW_SAMPLES.len()])
    }))
}

fn tile_checksum(tile: QuadTile128) -> u64 {
    tile.to_regs()
        .into_iter()
        .fold(0u64, |checksum, reg| mix_checksum(checksum, reg.raw()))
}

fn mix_checksum(checksum: u64, value: u64) -> u64 {
    checksum.rotate_left(7) ^ value.wrapping_mul(0x9E37_79B9_7F4A_7C15)
}

fn format_quad_v1_sample(sample: &BenchSample, extra_rates: &[(&str, u64)]) -> String {
    let mut text = format_bench_line(sample.name, sample.ops, sample.nanos, extra_rates);
    let _ = write!(text, " checksum={:016x}", sample.checksum);
    text
}

fn relative_ratio(baseline: &BenchSample, candidate: &BenchSample) -> f64 {
    baseline.nanos.max(1) as f64 / candidate.nanos.max(1) as f64
}

fn format_bench_line(name: &str, ops: u64, nanos: u64, extra_rates: &[(&str, u64)]) -> String {
    let nanos = nanos.max(1);
    let ops = ops.max(1);
    let mut text = format!(
        "{name}: ops={ops} ns_total={nanos} ops/s={:.2} ns/op={:.2}",
        rate_per_sec(ops, nanos),
        nanos as f64 / ops as f64,
    );
    for (label, total) in extra_rates {
        let _ = write!(text, " {label}={:.2}", rate_per_sec(*total, nanos));
    }
    text
}

fn rate_per_sec(total: u64, nanos: u64) -> f64 {
    (total as f64 * 1_000_000_000f64) / nanos.max(1) as f64
}

fn reg_filled(state: QuadState) -> QuadroReg32 {
    let mut reg = QuadroReg32::new();
    for lane in 0..QuadroReg32::LANES {
        reg.set_unchecked(lane, state);
    }
    reg
}

fn tile_filled(state: QuadState) -> QuadTile128 {
    let mut tile = QuadTile128::new();
    for lane in 0..QuadTile128::LANES {
        tile.set_unchecked(lane, state);
    }
    tile
}

fn sample_program() -> CoreProgram {
    CoreProgram {
        functions: vec![CoreFunction {
            name_id: SymbolId(0),
            regs: 4,
            instrs: vec![
                Instr::LoadFx {
                    dst: RegId(0),
                    value: Fx::from_raw(2 << 16),
                },
                Instr::LoadFx {
                    dst: RegId(1),
                    value: Fx::from_raw(3 << 16),
                },
                Instr::FxAdd {
                    dst: RegId(2),
                    lhs: RegId(0),
                    rhs: RegId(1),
                },
                Instr::Ret { src: RegId(2) },
            ],
        }],
        entry: FunctionId(0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quad_v1_benchmark_smoke_output_is_complete() {
        let output = bench_quad_v1_with_iterations(2);
        for required in [
            "quad-v1 benchmark",
            "quad-v1/reg-not-scalar",
            "quad-v1/reg-not-swar",
            "quad-v1/reg-not-default",
            "quad-v1/reg-and-scalar",
            "quad-v1/reg-and-swar",
            "quad-v1/reg-and-default",
            "quad-v1/tile-xor-default",
            "quad-v1/reg-bank-xor-inplace",
            "quad-v1/tile-bank-xor-inplace",
            "quad-v1/mask-roundtrip",
            "quad-v1/mask-iteration",
            "quad-v1/relative-reg-not",
            "quad-v1/relative-reg-and",
            "checksum=",
        ] {
            assert!(
                output.contains(required),
                "missing output label: {required}"
            );
        }
        assert!(!output.contains("NaN"));
        assert!(!output.contains("inf"));
    }

    #[test]
    fn unknown_benchmark_behavior_is_unchanged() {
        assert_eq!(
            run_benchmark("not-a-benchmark"),
            Err("unknown benchmark 'not-a-benchmark'".to_string())
        );
    }
}
