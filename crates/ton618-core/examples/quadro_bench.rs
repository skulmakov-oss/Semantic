use std::hint::black_box;
use std::time::Instant;
use std::vec::Vec;

use ton618_core::quadro::{DeltaSoA, QuadState, QuadroBank, QuadroReg, StateDelta};

const LANES_PER_REG: usize = 32;
const QREG_ITERS: u64 = 5_000_000;
const BANK_ITERS: u64 = 200_000;
const BASELINE_ITERS: u64 = 200_000;
const NREG: usize = 256;
const BASELINE_LEN: usize = NREG * LANES_PER_REG;

#[derive(Clone, Copy, Debug)]
struct BenchRow {
    name: &'static str,
    iters: u64,
    elapsed_ms: f64,
    ns_per_iter: f64,
    regs_per_sec: f64,
    quadits_per_sec: f64,
}

fn main() {
    sanity_checks();

    let rows = [
        bench_row("qreg_merge", QREG_ITERS, 1, || {
            let left = reg_from_seed(0x1111_2026_0629_0001);
            let right = reg_from_seed(0x2222_2026_0629_0002);
            checksum_reg(black_box(left).merge(black_box(right)))
        }),
        bench_row("qreg_intersect", QREG_ITERS, 1, || {
            let left = reg_from_seed(0x3333_2026_0629_0003);
            let right = reg_from_seed(0x4444_2026_0629_0004);
            checksum_reg(black_box(left).intersect(black_box(right)))
        }),
        bench_row("qreg_inverse", QREG_ITERS, 1, || {
            checksum_reg(black_box(reg_from_seed(0x5555_2026_0629_0005)).inverse())
        }),
        bench_row("qreg_masks_all", QREG_ITERS, 1, || {
            checksum_masks(black_box(reg_from_seed(0x6666_2026_0629_0006)).masks_all())
        }),
        bench_row("qreg_calc_delta", QREG_ITERS, 1, || {
            let prev = reg_from_seed(0x7777_2026_0629_0007);
            let next = reg_from_seed(0x8888_2026_0629_0008);
            checksum_delta(black_box(prev).calc_delta(black_box(next)))
        }),
        bench_row("qbank_merge_inplace", BANK_ITERS, NREG, || {
            bank_merge_checksum()
        }),
        bench_row("qbank_intersect_inplace", BANK_ITERS, NREG, || {
            bank_intersect_checksum()
        }),
        bench_row("qbank_inverse_inplace", BANK_ITERS, NREG, || {
            bank_inverse_checksum()
        }),
        bench_row("qbank_calc_deltas_soa", BANK_ITERS, NREG, || {
            bank_delta_checksum()
        }),
        bench_row(
            "baseline_vec_u8_delta",
            BASELINE_ITERS,
            BASELINE_LEN / LANES_PER_REG,
            baseline_delta_checksum,
        ),
    ];

    print_report(&rows);
}

fn bench_row(
    name: &'static str,
    iters: u64,
    regs_per_iter: usize,
    mut run: impl FnMut() -> u64,
) -> BenchRow {
    let start = Instant::now();
    let mut acc = 0u64;
    for _ in 0..iters {
        acc ^= black_box(run());
    }
    black_box(acc);

    let elapsed = start.elapsed();
    let elapsed_secs = elapsed.as_secs_f64();
    let regs_total = regs_per_iter as f64 * iters as f64;
    let regs_per_sec = regs_total / elapsed_secs;
    BenchRow {
        name,
        iters,
        elapsed_ms: elapsed_secs * 1_000.0,
        ns_per_iter: elapsed.as_nanos() as f64 / iters as f64,
        regs_per_sec,
        quadits_per_sec: regs_per_sec * LANES_PER_REG as f64,
    }
}

fn print_report(rows: &[BenchRow]) {
    println!("Pulsar Quadro microbench");
    println!("build: release");
    println!("note: local numbers only; not a public performance claim");
    println!();
    println!(
        "{:<28} {:>12} {:>13} {:>10} {:>14} {:>14}",
        "name", "iters", "elapsed_ms", "ns/iter", "regs/s", "quadits/s"
    );
    for row in rows {
        println!(
            "{:<28} {:>12} {:>13.3} {:>10.2} {:>14.2} {:>14.2}",
            row.name,
            row.iters,
            row.elapsed_ms,
            row.ns_per_iter,
            row.regs_per_sec,
            row.quadits_per_sec
        );
    }
    println!();
    println!("This benchmark is local diagnostic evidence only.");
    println!("It does not declare production performance or release readiness.");
}

fn sanity_checks() {
    let n = reg_with_state(QuadState::N);
    let f = reg_with_state(QuadState::F);
    let t = reg_with_state(QuadState::T);
    let s = reg_with_state(QuadState::S);

    assert_eq!(f.merge(t).try_get(0).unwrap(), QuadState::S);
    assert_eq!(s.intersect(t).try_get(0).unwrap(), QuadState::T);
    assert_eq!(t.inverse().try_get(0).unwrap(), QuadState::F);
    assert_eq!(s.mask_super(), lane_mask(0));
    assert_eq!(n.calc_delta(t).entered_true, lane_mask(0));

    #[cfg(feature = "alloc")]
    {
        let left_regs = small_bank_regs(0xaaaa_bbbb_cccc_dddd, 4);
        let right_regs = small_bank_regs(0x1111_2222_3333_4444, 4);
        let left = QuadroBank::from_regs(left_regs.clone());
        let right = QuadroBank::from_regs(right_regs.clone());
        let delta = left.calc_deltas_soa(&right).unwrap();
        let expected: Vec<StateDelta> = left_regs
            .iter()
            .copied()
            .zip(right_regs.iter().copied())
            .map(|(lhs, rhs)| lhs.calc_delta(rhs))
            .collect();
        assert_eq!(delta.to_aos(), expected);
    }
}

#[cfg(feature = "alloc")]
fn bank_merge_checksum() -> u64 {
    let left_regs = bank_regs(0x0101_2026_0629_0001, NREG);
    let right_regs = bank_regs(0x0202_2026_0629_0002, NREG);
    let left_template = QuadroBank::from_regs(left_regs);
    let right = QuadroBank::from_regs(right_regs);
    let mut scratch = left_template.clone();
    scratch
        .as_mut_slice()
        .copy_from_slice(left_template.as_slice());
    scratch.merge_inplace(&right).unwrap();
    bank_checksum(&scratch)
}

#[cfg(feature = "alloc")]
fn bank_intersect_checksum() -> u64 {
    let left_regs = bank_regs(0x0303_2026_0629_0003, NREG);
    let right_regs = bank_regs(0x0404_2026_0629_0004, NREG);
    let left_template = QuadroBank::from_regs(left_regs);
    let right = QuadroBank::from_regs(right_regs);
    let mut scratch = left_template.clone();
    scratch
        .as_mut_slice()
        .copy_from_slice(left_template.as_slice());
    scratch.intersect_inplace(&right).unwrap();
    bank_checksum(&scratch)
}

#[cfg(feature = "alloc")]
fn bank_inverse_checksum() -> u64 {
    let left_regs = bank_regs(0x0505_2026_0629_0005, NREG);
    let left_template = QuadroBank::from_regs(left_regs);
    let mut scratch = left_template.clone();
    scratch
        .as_mut_slice()
        .copy_from_slice(left_template.as_slice());
    scratch.inverse_inplace();
    bank_checksum(&scratch)
}

#[cfg(feature = "alloc")]
fn bank_delta_checksum() -> u64 {
    let left_regs = bank_regs(0x0606_2026_0629_0006, NREG);
    let right_regs = bank_regs(0x0707_2026_0629_0007, NREG);
    let left = QuadroBank::from_regs(left_regs);
    let right = QuadroBank::from_regs(right_regs);
    let delta = left.calc_deltas_soa(&right).unwrap();
    delta_checksum(&delta)
}

#[cfg(feature = "alloc")]
fn bank_checksum(bank: &QuadroBank) -> u64 {
    let bank = black_box(bank.as_slice());
    bank.first().map(|reg| reg.raw()).unwrap_or(0)
        ^ bank.last().map(|reg| reg.raw()).unwrap_or(0)
        ^ bank.len() as u64
}

#[cfg(feature = "alloc")]
fn delta_checksum(delta: &DeltaSoA) -> u64 {
    let delta = black_box(delta);
    let len = delta.len();
    delta.entered_true[0]
        ^ delta.left_true[0]
        ^ delta.entered_false[0]
        ^ delta.left_false[0]
        ^ delta.entered_super[0]
        ^ delta.left_super[0]
        ^ delta.entered_true[len - 1]
        ^ delta.left_true[len - 1]
        ^ delta.entered_false[len - 1]
        ^ delta.left_false[len - 1]
        ^ delta.entered_super[len - 1]
        ^ delta.left_super[len - 1]
        ^ len as u64
}

fn checksum_reg(reg: QuadroReg) -> u64 {
    black_box(reg).raw()
}

fn checksum_masks(masks: ton618_core::quadro::QuadMasks) -> u64 {
    black_box(masks.null)
        ^ black_box(masks.strict_false)
        ^ black_box(masks.strict_true)
        ^ black_box(masks.super_)
        ^ black_box(masks.non_null)
}

fn checksum_delta(delta: StateDelta) -> u64 {
    black_box(delta.entered_true)
        ^ black_box(delta.left_true)
        ^ black_box(delta.entered_false)
        ^ black_box(delta.left_false)
        ^ black_box(delta.entered_super)
        ^ black_box(delta.left_super)
}

fn reg_with_state(state: QuadState) -> QuadroReg {
    let mut reg = QuadroReg::new();
    reg.try_set(0, state.bits()).unwrap();
    reg
}

fn reg_from_seed(mut seed: u64) -> QuadroReg {
    let mut reg = QuadroReg::new();
    for lane in 0..LANES_PER_REG {
        let state = (lcg(&mut seed) & 0b11) as u8;
        reg.try_set(lane, state).unwrap();
    }
    reg
}

fn small_bank_regs(seed: u64, count: usize) -> Vec<QuadroReg> {
    bank_regs(seed, count)
}

fn bank_regs(mut seed: u64, count: usize) -> Vec<QuadroReg> {
    (0..count)
        .map(|_| {
            let mut reg = QuadroReg::new();
            for lane in 0..LANES_PER_REG {
                let state = (lcg(&mut seed) & 0b11) as u8;
                reg.try_set(lane, state).unwrap();
            }
            reg
        })
        .collect()
}

fn baseline_states(mut seed: u64, len: usize) -> Vec<u8> {
    (0..len).map(|_| (lcg(&mut seed) & 0b11) as u8).collect()
}

fn baseline_delta_checksum() -> u64 {
    let prev = baseline_states(0x0808_2026_0629_0008, BASELINE_LEN);
    let next = baseline_states(0x0909_2026_0629_0009, BASELINE_LEN);
    let mut checksum = 0u64;

    for (&lhs, &rhs) in prev.iter().zip(next.iter()) {
        if rhs == QuadState::T.bits() && lhs != QuadState::T.bits() {
            checksum ^= 1;
        }
        if lhs == QuadState::T.bits() && rhs != QuadState::T.bits() {
            checksum ^= 1 << 1;
        }
        if rhs == QuadState::F.bits() && lhs != QuadState::F.bits() {
            checksum ^= 1 << 2;
        }
        if lhs == QuadState::F.bits() && rhs != QuadState::F.bits() {
            checksum ^= 1 << 3;
        }
        if rhs == QuadState::S.bits() && lhs != QuadState::S.bits() {
            checksum ^= 1 << 4;
        }
        if lhs == QuadState::S.bits() && rhs != QuadState::S.bits() {
            checksum ^= 1 << 5;
        }
    }

    black_box(checksum)
}

fn lane_mask(index: usize) -> u64 {
    1u64 << (index * 2)
}

fn lcg(seed: &mut u64) -> u64 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *seed
}
