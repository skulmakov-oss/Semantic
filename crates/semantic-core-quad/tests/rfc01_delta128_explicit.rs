//! RFC-01 F1 slice D: PlaneDelta128 / ExactStateDelta128, checked per lane against a
//! hand-written table and against the existing 32-lane PlaneDelta32 / ExactStateDelta32.

use semantic_core_quad::{
    ExactStateDelta128, PlaneDelta128, QuadMask32, QuadState, QuadTile128, QuadroReg32,
    StateDelta128,
};

const STATES: [QuadState; 4] = [QuadState::N, QuadState::F, QuadState::T, QuadState::S];

fn tp(s: QuadState) -> bool {
    matches!(s, QuadState::T | QuadState::S)
}
fn fp(s: QuadState) -> bool {
    matches!(s, QuadState::F | QuadState::S)
}
fn background(lane: usize) -> QuadState {
    STATES[(lane * 5 + 1) % 4]
}

/// plane (4) then exact (8) expectations
fn expected(a: QuadState, b: QuadState) -> [bool; 12] {
    let into = |q: QuadState| a != q && b == q;
    let out = |q: QuadState| a == q && b != q;
    [
        !tp(a) && tp(b),
        tp(a) && !tp(b),
        !fp(a) && fp(b),
        fp(a) && !fp(b),
        into(QuadState::N),
        out(QuadState::N),
        into(QuadState::F),
        out(QuadState::F),
        into(QuadState::T),
        out(QuadState::T),
        into(QuadState::S),
        out(QuadState::S),
    ]
}

fn fields(p: PlaneDelta128, x: ExactStateDelta128) -> [u128; 12] {
    [
        p.entered_truth.raw(),
        p.left_truth.raw(),
        p.entered_falsity.raw(),
        p.left_falsity.raw(),
        x.entered_neither.raw(),
        x.left_neither.raw(),
        x.entered_strict_false.raw(),
        x.left_strict_false.raw(),
        x.entered_strict_true.raw(),
        x.left_strict_true.raw(),
        x.entered_super.raw(),
        x.left_super.raw(),
    ]
}

#[test]
fn all_16_transitions_on_every_128_lane() {
    let mut cases = 0;
    for lane in 0..QuadTile128::LANES {
        for a in STATES {
            for b in STATES {
                let (mut p, mut c) = (QuadTile128::new(), QuadTile128::new());
                for j in 0..QuadTile128::LANES {
                    p.set_unchecked(j, background(j));
                    c.set_unchecked(j, background(j));
                }
                p.set_unchecked(lane, a);
                c.set_unchecked(lane, b);
                let got = fields(
                    QuadTile128::plane_delta(p, c),
                    QuadTile128::exact_state_delta(p, c),
                );
                for (k, (m, want)) in got.iter().zip(expected(a, b)).enumerate() {
                    assert_eq!(
                        *m,
                        (want as u128) << lane,
                        "lane {lane} {a:?}->{b:?} field {k}"
                    );
                }
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 2048);
}

fn xorshift(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

fn wide(seed: &mut u64) -> u128 {
    ((xorshift(seed) as u128) << 64) | xorshift(seed) as u128
}

#[test]
fn matches_32_lane_types_and_legacy_plane_subset() {
    let mut seed = 0x1357_9BDF_2468_ACE0u64;
    let mut tiles = vec![
        (
            QuadTile128::new(),
            QuadTile128::from_planes(u128::MAX, u128::MAX),
        ),
        (
            QuadTile128::from_planes(u128::MAX, 0),
            QuadTile128::from_planes(0, u128::MAX),
        ),
    ];
    for _ in 0..2000 {
        tiles.push((
            QuadTile128::from_planes(wide(&mut seed), wide(&mut seed)),
            QuadTile128::from_planes(wide(&mut seed), wide(&mut seed)),
        ));
    }
    for (p, c) in tiles {
        let pd = QuadTile128::plane_delta(p, c);
        let xd = QuadTile128::exact_state_delta(p, c);
        // per-lane oracle
        let mut want = [0u128; 12];
        for lane in 0..QuadTile128::LANES {
            for (k, bit) in expected(p.get_unchecked(lane), c.get_unchecked(lane))
                .iter()
                .enumerate()
            {
                want[k] |= (*bit as u128) << lane;
            }
        }
        assert_eq!(fields(pd, xd), want);
        // legacy StateDelta128 plane subset is the same event
        let sd = StateDelta128::from_tiles(p, c);
        assert_eq!(sd.entered_true, pd.entered_truth);
        assert_eq!(sd.left_true, pd.left_truth);
        assert_eq!(sd.entered_false, pd.entered_falsity);
        assert_eq!(sd.left_false, pd.left_falsity);
        assert_eq!(sd.entered_super, xd.entered_super);
        assert_eq!(sd.left_super, xd.left_super);
        // every lane is in exactly one exact state before and after: entries and exits balance
        let entered = xd.entered_neither.raw()
            | xd.entered_strict_false.raw()
            | xd.entered_strict_true.raw()
            | xd.entered_super.raw();
        let left = xd.left_neither.raw()
            | xd.left_strict_false.raw()
            | xd.left_strict_true.raw()
            | xd.left_super.raw();
        assert_eq!(entered, left);
        assert_eq!(entered, sd.changed.raw());
        // agrees with the existing 32-lane types register by register
        for (i, (pr, cr)) in p.to_regs().iter().zip(c.to_regs().iter()).enumerate() {
            let chunk = |m: u128| (m >> (32 * i)) as u64 & QuadMask32::VALID_MASK;
            let p32 = QuadroReg32::plane_delta(*pr, *cr);
            let x32 = QuadroReg32::exact_state_delta(*pr, *cr);
            assert_eq!(p32.entered_truth.raw(), chunk(pd.entered_truth.raw()));
            assert_eq!(p32.left_truth.raw(), chunk(pd.left_truth.raw()));
            assert_eq!(p32.entered_falsity.raw(), chunk(pd.entered_falsity.raw()));
            assert_eq!(p32.left_falsity.raw(), chunk(pd.left_falsity.raw()));
            assert_eq!(x32.entered_neither.raw(), chunk(xd.entered_neither.raw()));
            assert_eq!(x32.left_neither.raw(), chunk(xd.left_neither.raw()));
            assert_eq!(
                x32.entered_strict_false.raw(),
                chunk(xd.entered_strict_false.raw())
            );
            assert_eq!(
                x32.left_strict_false.raw(),
                chunk(xd.left_strict_false.raw())
            );
            assert_eq!(
                x32.entered_strict_true.raw(),
                chunk(xd.entered_strict_true.raw())
            );
            assert_eq!(x32.left_strict_true.raw(), chunk(xd.left_strict_true.raw()));
            assert_eq!(x32.entered_super.raw(), chunk(xd.entered_super.raw()));
            assert_eq!(x32.left_super.raw(), chunk(xd.left_super.raw()));
        }
    }
    assert_eq!(PlaneDelta128::default(), PlaneDelta128::default());
    assert_eq!(ExactStateDelta128::default().entered_super.raw(), 0);
}
