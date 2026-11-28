//! QT-2: QuadroReg32 lane masks are computed by bit compaction. They must equal a
//! per-lane reading of the register for every state at every lane, exhaustive
//! 4-lane blocks at every offset, adversarial words and random words, and the
//! deltas built on them must keep their raw-plane and exact-state meanings.

use semantic_core_quad::{QuadState, QuadroReg32};

/// n, f, t, s (exact), raw truth, raw falsity, legacy known, has evidence, determinate.
fn per_lane(r: QuadroReg32) -> [u64; 9] {
    let mut m = [0u64; 9];
    for lane in 0..QuadroReg32::LANES {
        let state = r.get_unchecked(lane);
        let bit = 1u64 << lane;
        let slot = match state {
            QuadState::N => 0,
            QuadState::F => 1,
            QuadState::T => 2,
            QuadState::S => 3,
        };
        m[slot] |= bit;
        if state.true_plane() {
            m[4] |= bit;
        }
        if state.false_plane() {
            m[5] |= bit;
        }
        if state.is_known() {
            m[6] |= bit;
        }
        if state.has_evidence() {
            m[7] |= bit;
        }
        if state.is_determinate() {
            m[8] |= bit;
        }
    }
    m
}

fn masks(r: QuadroReg32) -> [u64; 9] {
    let m = r.masks();
    [
        m.n.raw(),
        m.f.raw(),
        m.t.raw(),
        m.s.raw(),
        r.mask_true().raw(),
        r.mask_false().raw(),
        r.mask_known().raw(),
        r.mask_has_evidence().raw(),
        r.mask_determinate().raw(),
    ]
}

fn next(seed: &mut u64) -> u64 {
    *seed = seed.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *seed;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

fn corpus() -> Vec<QuadroReg32> {
    let mut out = Vec::new();
    for lane in 0..QuadroReg32::LANES {
        for state in QuadState::ALL {
            for background in [QuadState::N, QuadState::T, QuadState::S] {
                let mut r = QuadroReg32::new();
                for l in 0..QuadroReg32::LANES {
                    r.set_unchecked(l, background);
                }
                r.set_unchecked(lane, state);
                out.push(r);
            }
        }
    }
    for offset in 0..=28 {
        for pattern in 0..256u64 {
            out.push(QuadroReg32::from_raw(pattern << (2 * offset)));
        }
    }
    for word in [
        0,
        u64::MAX,
        0x5555_5555_5555_5555,
        0xAAAA_AAAA_AAAA_AAAA,
        0x3333_3333_3333_3333,
        0xCCCC_CCCC_CCCC_CCCC,
        0x8000_0000_0000_0001,
        3 << 62,
    ] {
        out.push(QuadroReg32::from_raw(word));
    }
    let mut seed = 0x0752_0002;
    for _ in 0..20_000 {
        out.push(QuadroReg32::from_raw(next(&mut seed)));
    }
    out
}

#[test]
fn qt2_masks_match_per_lane_reading() {
    for r in corpus() {
        let got = masks(r);
        assert_eq!(got, per_lane(r), "raw {:#018x}", r.raw());
        for mask in got {
            assert_eq!(mask >> 32, 0, "only lane bits 0..=31 may be set");
        }
    }
}

#[test]
fn qt2_deltas_keep_plane_and_exact_meanings() {
    let mut seed = 0x0752_0003;
    for _ in 0..20_000 {
        let previous = QuadroReg32::from_raw(next(&mut seed));
        let current = QuadroReg32::from_raw(next(&mut seed));
        let (p, c) = (per_lane(previous), per_lane(current));
        let plane = QuadroReg32::plane_delta(previous, current);
        assert_eq!(plane.entered_truth.raw(), c[4] & !p[4]);
        assert_eq!(plane.left_truth.raw(), p[4] & !c[4]);
        assert_eq!(plane.entered_falsity.raw(), c[5] & !p[5]);
        assert_eq!(plane.left_falsity.raw(), p[5] & !c[5]);
        let exact = QuadroReg32::exact_state_delta(previous, current);
        assert_eq!(exact.entered_neither.raw(), c[0] & !p[0]);
        assert_eq!(exact.left_neither.raw(), p[0] & !c[0]);
        assert_eq!(exact.entered_strict_false.raw(), c[1] & !p[1]);
        assert_eq!(exact.left_strict_false.raw(), p[1] & !c[1]);
        assert_eq!(exact.entered_strict_true.raw(), c[2] & !p[2]);
        assert_eq!(exact.left_strict_true.raw(), p[2] & !c[2]);
        assert_eq!(exact.entered_super.raw(), c[3] & !p[3]);
        assert_eq!(exact.left_super.raw(), p[3] & !c[3]);
    }
}
