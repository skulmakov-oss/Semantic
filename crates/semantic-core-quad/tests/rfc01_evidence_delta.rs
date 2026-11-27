//! RFC-01 F1 slice C: EvidenceDelta32/128, with legacy StateDelta32/128 regression.
//! Expectations come from a hand-written state table applied per lane.

use semantic_core_quad::{
    EvidenceDelta128, EvidenceDelta32, QuadMask32, QuadState, QuadTile128, QuadroReg32,
    StateDelta128, StateDelta32,
};

const STATES: [QuadState; 4] = [QuadState::N, QuadState::F, QuadState::T, QuadState::S];

fn tp(s: QuadState) -> bool {
    matches!(s, QuadState::T | QuadState::S)
}
fn fp(s: QuadState) -> bool {
    matches!(s, QuadState::F | QuadState::S)
}
fn ev(s: QuadState) -> bool {
    s != QuadState::N
}
fn det(s: QuadState) -> bool {
    matches!(s, QuadState::F | QuadState::T)
}
fn background(lane: usize) -> QuadState {
    STATES[(lane * 7 + 3) % 4]
}

/// Expected per-lane events: evidence (4) then legacy StateDelta (11).
fn expected(a: QuadState, b: QuadState) -> [bool; 15] {
    [
        !ev(a) && ev(b),
        ev(a) && !ev(b),
        !det(a) && det(b),
        det(a) && !det(b),
        !tp(a) && tp(b),
        tp(a) && !tp(b),
        !fp(a) && fp(b),
        fp(a) && !fp(b),
        a != QuadState::S && b == QuadState::S,
        a == QuadState::S && b != QuadState::S,
        a != b,
        !det(a) && det(b),
        det(a) && !det(b),
        a != QuadState::S && b == QuadState::S,
        a == QuadState::S && b != QuadState::S,
    ]
}

fn fields32(e: EvidenceDelta32, s: StateDelta32) -> [QuadMask32; 15] {
    [
        e.gained_evidence,
        e.lost_evidence,
        e.became_determinate,
        e.lost_determinacy,
        s.entered_true,
        s.left_true,
        s.entered_false,
        s.left_false,
        s.entered_super,
        s.left_super,
        s.changed,
        s.became_known,
        s.became_unknown,
        s.became_conflicted,
        s.resolved_conflict,
    ]
}

fn fields128(e: EvidenceDelta128, s: StateDelta128) -> [u128; 15] {
    [
        e.gained_evidence.raw(),
        e.lost_evidence.raw(),
        e.became_determinate.raw(),
        e.lost_determinacy.raw(),
        s.entered_true.raw(),
        s.left_true.raw(),
        s.entered_false.raw(),
        s.left_false.raw(),
        s.entered_super.raw(),
        s.left_super.raw(),
        s.changed.raw(),
        s.became_known.raw(),
        s.became_unknown.raw(),
        s.became_conflicted.raw(),
        s.resolved_conflict.raw(),
    ]
}

#[test]
fn all_16_transitions_on_every_32_lane() {
    let mut cases = 0;
    for lane in 0..QuadroReg32::LANES {
        for a in STATES {
            for b in STATES {
                let (mut p, mut c) = (QuadroReg32::new(), QuadroReg32::new());
                for j in 0..QuadroReg32::LANES {
                    p.set_unchecked(j, background(j));
                    c.set_unchecked(j, background(j));
                }
                p.set_unchecked(lane, a);
                c.set_unchecked(lane, b);
                let got = fields32(
                    EvidenceDelta32::from_regs(p, c),
                    StateDelta32::from_regs(p, c),
                );
                for (k, (m, want)) in got.iter().zip(expected(a, b)).enumerate() {
                    assert_eq!(
                        m.raw(),
                        (want as u64) << lane,
                        "lane {lane} {a:?}->{b:?} field {k}"
                    );
                    assert_eq!(m.raw() & !QuadMask32::VALID_MASK, 0);
                }
                cases += 1;
            }
        }
    }
    assert_eq!(cases, 512);
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
                let got = fields128(
                    EvidenceDelta128::from_tiles(p, c),
                    StateDelta128::from_tiles(p, c),
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
fn mixed_multi_lane_transitions_match_per_lane_oracle() {
    let mut seed = 0xDEAD_BEEF_CAFE_F00Du64;
    for _ in 0..2000 {
        let p = QuadroReg32::from_raw(xorshift(&mut seed));
        let c = QuadroReg32::from_raw(xorshift(&mut seed));
        let got = fields32(
            EvidenceDelta32::from_regs(p, c),
            StateDelta32::from_regs(p, c),
        );
        let mut want = [0u64; 15];
        for lane in 0..QuadroReg32::LANES {
            for (k, bit) in expected(p.get_unchecked(lane), c.get_unchecked(lane))
                .iter()
                .enumerate()
            {
                want[k] |= (*bit as u64) << lane;
            }
        }
        for k in 0..15 {
            assert_eq!(got[k].raw(), want[k], "field {k}");
        }
        let pt = QuadTile128::from_planes(wide(&mut seed), wide(&mut seed));
        let ct = QuadTile128::from_planes(wide(&mut seed), wide(&mut seed));
        let e128 = EvidenceDelta128::from_tiles(pt, ct);
        for (i, (pr, cr)) in pt.to_regs().iter().zip(ct.to_regs().iter()).enumerate() {
            let e32 = EvidenceDelta32::from_regs(*pr, *cr);
            let chunk = |m: u128| (m >> (32 * i)) as u64 & QuadMask32::VALID_MASK;
            assert_eq!(e32.gained_evidence.raw(), chunk(e128.gained_evidence.raw()));
            assert_eq!(e32.lost_evidence.raw(), chunk(e128.lost_evidence.raw()));
            assert_eq!(
                e32.became_determinate.raw(),
                chunk(e128.became_determinate.raw())
            );
            assert_eq!(
                e32.lost_determinacy.raw(),
                chunk(e128.lost_determinacy.raw())
            );
        }
    }
}

#[test]
fn evidence_events_are_presence_not_plane_bits() {
    // T -> S adds falsity evidence but is not a change of evidence presence.
    let (mut p, mut c) = (QuadroReg32::new(), QuadroReg32::new());
    p.set_unchecked(0, QuadState::T);
    c.set_unchecked(0, QuadState::S);
    let e = EvidenceDelta32::from_regs(p, c);
    assert!(e.gained_evidence.is_empty() && e.lost_evidence.is_empty());
    assert_eq!(e.lost_determinacy.raw(), 1);
    assert_eq!(StateDelta32::from_regs(p, c).entered_false.raw(), 1);
    assert_eq!(EvidenceDelta32::default().gained_evidence.raw(), 0);
}
