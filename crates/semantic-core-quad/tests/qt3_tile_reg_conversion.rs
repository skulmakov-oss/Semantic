//! QT-3a contract tests for `QuadTile128::from_regs` / `QuadTile128::to_regs`.
//!
//! Normative contract (unchanged by QT-3a): tile lane `32 * k + i` corresponds to lane `i` of
//! `regs[k]`; a register lane `i` stores falsity at raw bit `2i` and truth at raw bit `2i + 1`
//! (N = 00, F = 01, T = 10, S = 11); the tile stores lane `l` at bit `l` of its truth and falsity
//! planes. Two independent references are used:
//! * `oracle_*`: computes raw bits directly from the definition above (no crate accessors);
//! * `legacy_*`: the previous per-lane implementation, kept verbatim as a behavioural reference.
//!
//! Round trips alone are not sufficient (two wrong conversions can cancel), so every direction is
//! compared against the references and against explicit raw values.

use semantic_core_quad::{QuadState, QuadTile128, QuadroReg32, LSB_MASK_32, MSB_MASK_32};

fn oracle_to_regs(t: u128, f: u128) -> [u64; 4] {
    let mut raw = [0u64; 4];
    for lane in 0..128 {
        let (k, i) = (lane / 32, lane % 32);
        raw[k] |= (((f >> lane) & 1) as u64) << (2 * i);
        raw[k] |= (((t >> lane) & 1) as u64) << (2 * i + 1);
    }
    raw
}

fn oracle_from_regs(raw: [u64; 4]) -> (u128, u128) {
    let (mut t, mut f) = (0u128, 0u128);
    for lane in 0..128 {
        let (k, i) = (lane / 32, lane % 32);
        f |= (((raw[k] >> (2 * i)) & 1) as u128) << lane;
        t |= (((raw[k] >> (2 * i + 1)) & 1) as u128) << lane;
    }
    (t, f)
}

/// The pre-QT-3a implementation, verbatim apart from being a free function.
fn legacy_from_regs(regs: [QuadroReg32; 4]) -> QuadTile128 {
    let mut out = QuadTile128::new();
    let mut lane = 0usize;
    while lane < QuadTile128::LANES {
        out.set_unchecked(lane, regs[lane / 32].get_unchecked(lane % 32));
        lane += 1;
    }
    out
}

fn legacy_to_regs(tile: QuadTile128) -> [QuadroReg32; 4] {
    let mut regs = [QuadroReg32::new(); 4];
    let mut lane = 0usize;
    while lane < QuadTile128::LANES {
        regs[lane / 32].set_unchecked(lane % 32, tile.get_unchecked(lane));
        lane += 1;
    }
    regs
}

fn raws(regs: [QuadroReg32; 4]) -> [u64; 4] {
    regs.map(QuadroReg32::raw)
}

/// Checks one tile in both directions against both references.
fn check_tile(t: u128, f: u128) {
    let tile = QuadTile128::from_planes(t, f);
    let regs = tile.to_regs();
    assert_eq!(
        raws(regs),
        oracle_to_regs(t, f),
        "to_regs vs oracle t={t:#x} f={f:#x}"
    );
    assert_eq!(regs, legacy_to_regs(tile), "to_regs vs legacy");
    let back = QuadTile128::from_regs(regs);
    assert_eq!(
        (back.true_plane(), back.false_plane()),
        (t, f),
        "from_regs(to_regs)"
    );
}

fn check_regs(raw: [u64; 4]) {
    let regs = raw.map(QuadroReg32::from_raw);
    let tile = QuadTile128::from_regs(regs);
    assert_eq!(
        (tile.true_plane(), tile.false_plane()),
        oracle_from_regs(raw),
        "from_regs vs oracle {raw:x?}"
    );
    assert_eq!(tile, legacy_from_regs(regs), "from_regs vs legacy");
    assert_eq!(raws(tile.to_regs()), raw, "to_regs(from_regs)");
}

fn splitmix(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *seed;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

const BOUNDARY_LANES: [usize; 9] = [0, 31, 32, 63, 64, 95, 96, 127, 1];

#[test]
fn qt3_every_state_at_every_lane_with_every_background() {
    for background in QuadState::ALL {
        for lane in 0..QuadTile128::LANES {
            for state in QuadState::ALL {
                let mut tile = QuadTile128::new();
                for l in 0..QuadTile128::LANES {
                    tile.set_unchecked(l, background);
                }
                tile.set_unchecked(lane, state);
                check_tile(tile.true_plane(), tile.false_plane());
                // explicit raw value of the register that holds the lane
                let regs = tile.to_regs();
                let bg = background.bits() as u64 * LSB_MASK_32;
                let (k, i) = (lane / 32, lane % 32);
                let expect = (bg & !(0b11 << (2 * i))) | ((state.bits() as u64) << (2 * i));
                assert_eq!(regs[k].raw(), expect, "lane {lane}");
                for (j, r) in regs.iter().enumerate() {
                    if j != k {
                        assert_eq!(r.raw(), bg, "untouched register {j}");
                    }
                }
                check_regs(raws(regs));
            }
        }
    }
}

#[test]
fn qt3_plane_order_and_state_encoding() {
    // truth-only tile -> every lane T (10) -> odd raw bits; falsity-only -> F (01) -> even bits
    assert_eq!(
        raws(QuadTile128::from_planes(u128::MAX, 0).to_regs()),
        [MSB_MASK_32; 4]
    );
    assert_eq!(
        raws(QuadTile128::from_planes(0, u128::MAX).to_regs()),
        [LSB_MASK_32; 4]
    );
    assert_eq!(
        raws(QuadTile128::from_planes(u128::MAX, u128::MAX).to_regs()),
        [u64::MAX; 4]
    );
    assert_eq!(raws(QuadTile128::new().to_regs()), [0; 4]);
    // register order: low lanes in regs[0]
    let low = QuadTile128::from_planes(0xFFFF_FFFF, 0);
    assert_eq!(raws(low.to_regs()), [MSB_MASK_32, 0, 0, 0]);
    let high = QuadTile128::from_planes(0, 0xFFFF_FFFFu128 << 96);
    assert_eq!(raws(high.to_regs()), [0, 0, 0, LSB_MASK_32]);
    for (state, bits) in [
        (QuadState::N, 0b00u64),
        (QuadState::F, 0b01),
        (QuadState::T, 0b10),
        (QuadState::S, 0b11),
    ] {
        let mut reg = QuadroReg32::new();
        reg.set_unchecked(0, state);
        assert_eq!(reg.raw(), bits);
        let tile = QuadTile128::from_regs([
            reg,
            QuadroReg32::new(),
            QuadroReg32::new(),
            QuadroReg32::new(),
        ]);
        assert_eq!(tile.get_unchecked(0), state);
        assert_eq!(
            ((tile.true_plane() & 1) as u64) << 1 | (tile.false_plane() & 1) as u64,
            bits
        );
    }
}

#[test]
fn qt3_structured_patterns_and_boundaries() {
    let alt = 0x5555_5555_5555_5555_5555_5555_5555_5555u128;
    let mut planes = vec![
        (0, 0),
        (u128::MAX, 0),
        (0, u128::MAX),
        (u128::MAX, u128::MAX),
        (alt, !alt),
        (!alt, alt),
        (alt, alt),
        (
            0x0F0F_0F0F_0F0F_0F0F_0F0F_0F0F_0F0F_0F0F,
            0x00FF_00FF_00FF_00FF_00FF_00FF_00FF_00FF,
        ),
        (
            0xFFFF_FFFF_0000_0000_FFFF_FFFF_0000_0000,
            0x0000_0000_FFFF_FFFF_0000_0000_FFFF_FFFF,
        ),
    ];
    for &lane in &BOUNDARY_LANES {
        planes.push((1u128 << lane, 0));
        planes.push((0, 1u128 << lane));
        planes.push((1u128 << lane, 1u128 << lane));
        planes.push((!(1u128 << lane), u128::MAX));
        planes.push((u128::MAX, !(1u128 << lane)));
    }
    for (t, f) in planes {
        check_tile(t, f);
    }
    let mut raw_patterns = vec![
        [0u64; 4],
        [u64::MAX; 4],
        [LSB_MASK_32; 4],
        [MSB_MASK_32; 4],
        [LSB_MASK_32, MSB_MASK_32, u64::MAX, 0],
    ];
    for bit in [0, 1, 62, 63] {
        for k in 0..4 {
            let mut r = [0u64; 4];
            r[k] = 1 << bit;
            raw_patterns.push(r);
        }
    }
    for raw in raw_patterns {
        check_regs(raw);
    }
}

#[test]
fn qt3_seeded_dense_and_sparse() {
    let mut seed = 0x51A3_0001u64;
    for round in 0..40_000 {
        let a = ((splitmix(&mut seed) as u128) << 64) | splitmix(&mut seed) as u128;
        let b = ((splitmix(&mut seed) as u128) << 64) | splitmix(&mut seed) as u128;
        let (t, f) = match round % 4 {
            0 => (a, b),                                // dense
            1 => (a & b & (a >> 3), a & !b & (b >> 5)), // sparse
            2 => (a, a),                                // correlated planes
            _ => (a | b, a & b),                        // dense truth, sparse falsity
        };
        check_tile(t, f);
        let raw = [
            splitmix(&mut seed),
            splitmix(&mut seed),
            splitmix(&mut seed),
            splitmix(&mut seed),
        ];
        check_regs(raw);
    }
}

#[test]
fn qt3_layout_and_consumers_unchanged() {
    assert_eq!(core::mem::size_of::<QuadroReg32>(), 8);
    assert_eq!(core::mem::size_of::<[QuadroReg32; 4]>(), 32);
    assert_eq!(core::mem::size_of::<QuadTile128>(), 32);
    assert_eq!(core::mem::align_of::<QuadTile128>(), 16);
    // tile truth maps keep agreeing with the four-register decomposition
    let mut seed = 0x51A3_0002u64;
    for _ in 0..2_000 {
        let p = |s: &mut u64| ((splitmix(s) as u128) << 64) | splitmix(s) as u128;
        let a = QuadTile128::from_planes(p(&mut seed), p(&mut seed));
        let b = QuadTile128::from_planes(p(&mut seed), p(&mut seed));
        let (ra, rb) = (a.to_regs(), b.to_regs());
        let by_regs = |m: fn(QuadroReg32, QuadroReg32) -> QuadroReg32| {
            QuadTile128::from_regs([
                m(ra[0], rb[0]),
                m(ra[1], rb[1]),
                m(ra[2], rb[2]),
                m(ra[3], rb[3]),
            ])
        };
        assert_eq!(a.map_and(b), by_regs(QuadroReg32::map_and));
        assert_eq!(a.map_or(b), by_regs(QuadroReg32::map_or));
        assert_eq!(a.map_xor(b), by_regs(QuadroReg32::map_xor));
        assert_eq!(
            a.map_not(),
            QuadTile128::from_regs(ra.map(QuadroReg32::map_not))
        );
    }
}
