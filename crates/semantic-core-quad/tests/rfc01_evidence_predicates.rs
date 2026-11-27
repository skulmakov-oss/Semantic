//! RFC-01 F1 slice B: explicit evidence/determinacy predicates.
//! The oracle derives every expectation from a hand-written state table and
//! per-lane reads; it never calls the predicates under test to build expectations.

use semantic_core_quad::{QuadMask32, QuadState, QuadTile128, QuadroReg32};

const STATES: [QuadState; 4] = [QuadState::N, QuadState::F, QuadState::T, QuadState::S];

/// (state, truth plane, falsity plane, has_evidence, is_determinate)
const TABLE: [(QuadState, bool, bool, bool, bool); 4] = [
    (QuadState::N, false, false, false, false),
    (QuadState::F, false, true, true, true),
    (QuadState::T, true, false, true, true),
    (QuadState::S, true, true, true, false),
];

fn oracle(s: QuadState) -> (bool, bool) {
    let row = TABLE.iter().find(|r| r.0 == s).expect("state in table");
    (row.3, row.4)
}

fn xorshift(seed: &mut u64) -> u64 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    *seed
}

#[test]
fn scalar_predicates_match_table_and_legacy() {
    for (s, tp, fp, ev, det) in TABLE {
        assert_eq!(s.true_plane(), tp, "{s:?} truth plane");
        assert_eq!(s.false_plane(), fp, "{s:?} falsity plane");
        assert_eq!(s.has_evidence(), ev, "{s:?} has_evidence");
        assert_eq!(s.is_determinate(), det, "{s:?} is_determinate");
        assert_eq!(s.is_known(), det, "{s:?} legacy is_known == determinate");
        assert_eq!(s.is_null(), !ev, "{s:?} is_null == !has_evidence");
        assert_eq!(
            s.is_conflict(),
            tp && fp,
            "{s:?} is_conflict == truth & falsity"
        );
    }
}

fn reg_cases() -> Vec<u64> {
    let mut v = vec![
        0,
        u64::MAX,
        0x5555_5555_5555_5555,
        0xAAAA_AAAA_AAAA_AAAA,
        1,
        2,
        3,
        3 << 62,
        2 << 62,
        1 << 62,
    ];
    for s in STATES {
        let mut r = QuadroReg32::new();
        for lane in 0..QuadroReg32::LANES {
            r.set_unchecked(lane, s);
        }
        v.push(r.raw());
    }
    let mut seed = 0x9E37_79B9_7F4A_7C15u64;
    for _ in 0..4096 {
        v.push(xorshift(&mut seed));
    }
    v
}

#[test]
fn reg32_masks_match_per_lane_oracle_and_exact_masks() {
    for raw in reg_cases() {
        let r = QuadroReg32::from_raw(raw);
        let he = r.mask_has_evidence();
        let de = r.mask_determinate();
        let (mut want_he, mut want_de) = (0u64, 0u64);
        for lane in 0..QuadroReg32::LANES {
            let (ev, det) = oracle(r.get_unchecked(lane));
            want_he |= (ev as u64) << lane;
            want_de |= (det as u64) << lane;
        }
        assert_eq!(he.raw(), want_he, "has_evidence {raw:#018x}");
        assert_eq!(de.raw(), want_de, "determinate {raw:#018x}");
        assert_eq!(he.raw() & !QuadMask32::VALID_MASK, 0);
        assert_eq!(de.raw() & !QuadMask32::VALID_MASK, 0);
        let m = r.masks();
        assert_eq!(he, m.f.or(m.t).or(m.s));
        assert_eq!(de, m.f.or(m.t));
        let (n, s) = (r.mask_null(), r.mask_conflict());
        assert_eq!(n.raw() | de.raw() | s.raw(), QuadMask32::VALID_MASK);
        assert_eq!(n.raw() & de.raw(), 0);
        assert_eq!(n.raw() & s.raw(), 0);
        assert_eq!(de.raw() & s.raw(), 0);
        assert_eq!(he, de.or(s));
        assert_eq!(he.not_valid(), n);
        assert_eq!(r.mask_known(), de, "legacy mask_known == determinate");
        assert_eq!(m.known(), de, "legacy QuadMasks32::known == determinate");
    }
}

#[test]
fn tile128_masks_match_per_lane_oracle() {
    let mut seed = 0x0123_4567_89AB_CDEFu64;
    let mut tiles = vec![
        QuadTile128::new(),
        QuadTile128::from_planes(u128::MAX, 0),
        QuadTile128::from_planes(0, u128::MAX),
        QuadTile128::from_planes(u128::MAX, u128::MAX),
    ];
    for _ in 0..2048 {
        let t = ((xorshift(&mut seed) as u128) << 64) | xorshift(&mut seed) as u128;
        let f = ((xorshift(&mut seed) as u128) << 64) | xorshift(&mut seed) as u128;
        tiles.push(QuadTile128::from_planes(t, f));
    }
    for tile in tiles {
        let (mut want_he, mut want_de) = (0u128, 0u128);
        for lane in 0..QuadTile128::LANES {
            let (ev, det) = oracle(tile.get_unchecked(lane));
            want_he |= (ev as u128) << lane;
            want_de |= (det as u128) << lane;
        }
        assert_eq!(tile.has_evidence_mask().raw(), want_he);
        assert_eq!(tile.determinate_mask().raw(), want_de);
        assert_eq!(
            tile.determinate_mask(),
            tile.known_mask(),
            "legacy known_mask == determinate"
        );
        assert_eq!(tile.has_evidence_mask().not_valid(), tile.null_mask());
        assert_eq!(
            tile.has_evidence_mask(),
            tile.determinate_mask().or(tile.conflict_mask())
        );
        assert_eq!(
            tile.null_mask().raw() | tile.determinate_mask().raw() | tile.conflict_mask().raw(),
            u128::MAX
        );
        for (i, reg) in tile.to_regs().iter().enumerate() {
            let he = (tile.has_evidence_mask().raw() >> (32 * i)) as u64 & QuadMask32::VALID_MASK;
            assert_eq!(reg.mask_has_evidence().raw(), he);
            let de = (tile.determinate_mask().raw() >> (32 * i)) as u64 & QuadMask32::VALID_MASK;
            assert_eq!(reg.mask_determinate().raw(), de);
        }
    }
}
