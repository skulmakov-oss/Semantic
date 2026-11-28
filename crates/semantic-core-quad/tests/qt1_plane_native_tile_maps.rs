//! QT-1: the plane-native `QuadTile128` truth maps must equal, bit for bit, both the
//! Quad Logic Frame lookup tables (per lane) and the four-register decomposition
//! that served as the implementation before QT-1.

use semantic_core_quad::{logic_frame, QuadState, QuadTile128, QuadroReg32};

type TileOp = fn(QuadTile128, QuadTile128) -> QuadTile128;
type RegOp = fn(QuadroReg32, QuadroReg32) -> QuadroReg32;
type LaneOp = fn(QuadState, QuadState) -> QuadState;

const BINARY: [(&str, TileOp, RegOp, LaneOp); 6] = [
    (
        "and",
        QuadTile128::map_and,
        QuadroReg32::map_and,
        logic_frame::and,
    ),
    (
        "or",
        QuadTile128::map_or,
        QuadroReg32::map_or,
        logic_frame::or,
    ),
    (
        "xor",
        QuadTile128::map_xor,
        QuadroReg32::map_xor,
        logic_frame::xor,
    ),
    (
        "implies",
        QuadTile128::map_implies,
        QuadroReg32::map_implies,
        logic_frame::implies,
    ),
    (
        "nand",
        QuadTile128::map_nand,
        QuadroReg32::map_nand,
        logic_frame::nand,
    ),
    (
        "nor",
        QuadTile128::map_nor,
        QuadroReg32::map_nor,
        logic_frame::nor,
    ),
];

fn decomposed(a: QuadTile128, b: QuadTile128, op: RegOp) -> QuadTile128 {
    let (x, y) = (a.to_regs(), b.to_regs());
    QuadTile128::from_regs([
        op(x[0], y[0]),
        op(x[1], y[1]),
        op(x[2], y[2]),
        op(x[3], y[3]),
    ])
}

fn tile(states: impl Fn(usize) -> QuadState) -> QuadTile128 {
    let mut t = QuadTile128::new();
    for lane in 0..QuadTile128::LANES {
        t.set_unchecked(lane, states(lane));
    }
    t
}

/// Deterministic SplitMix64 stream for plane bits.
fn next(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *seed;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn cases() -> Vec<(QuadTile128, QuadTile128)> {
    let mut out = Vec::new();
    // Every ordered pair of states reaches every one of the 128 lanes.
    for shift in 0..16 {
        let a = tile(|lane| QuadState::ALL[((lane + shift) % 16) / 4]);
        let b = tile(|lane| QuadState::ALL[(lane + shift) % 4]);
        out.push((a, b));
    }
    let mut seed = 0x0051_7128;
    for _ in 0..4096 {
        let mut plane = || ((next(&mut seed) as u128) << 64) | next(&mut seed) as u128;
        out.push((
            QuadTile128::from_planes(plane(), plane()),
            QuadTile128::from_planes(plane(), plane()),
        ));
    }
    out
}

#[test]
fn qt1_binary_maps_match_lanes_and_decomposition() {
    for (a, b) in cases() {
        for (name, tile_op, reg_op, lane_op) in BINARY {
            let got = tile_op(a, b);
            assert_eq!(got, decomposed(a, b, reg_op), "{name}: decomposition");
            for lane in 0..QuadTile128::LANES {
                let want = lane_op(a.get_unchecked(lane), b.get_unchecked(lane));
                assert_eq!(got.get_unchecked(lane), want, "{name}: lane {lane}");
            }
        }
    }
}

#[test]
fn qt1_not_matches_lanes_and_decomposition() {
    for (a, _) in cases() {
        let got = a.map_not();
        let r = a.to_regs();
        let reference = QuadTile128::from_regs([
            r[0].map_not(),
            r[1].map_not(),
            r[2].map_not(),
            r[3].map_not(),
        ]);
        assert_eq!(got, reference);
        for lane in 0..QuadTile128::LANES {
            assert_eq!(
                got.get_unchecked(lane),
                logic_frame::not(a.get_unchecked(lane))
            );
        }
    }
}
