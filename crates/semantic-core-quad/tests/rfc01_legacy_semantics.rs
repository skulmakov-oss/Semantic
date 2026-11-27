//! RFC-01 F1 slice A: pins the frozen legacy meanings documented in
//! docs/architecture/core_quad_known_semantics.md (existing APIs only).

use semantic_core_quad::{QuadState, QuadTile128, QuadroReg32, StateDelta128, StateDelta32};

#[test]
fn legacy_known_is_exact_f_or_t() {
    let want = [
        (QuadState::N, false),
        (QuadState::F, true),
        (QuadState::T, true),
        (QuadState::S, false),
    ];
    for (s, k) in want {
        assert_eq!(s.is_known(), k, "{s:?}");
        let mut r = QuadroReg32::new();
        r.set_unchecked(7, s);
        assert_eq!(r.mask_known().raw() >> 7 & 1 == 1, k, "{s:?} mask_known");
        assert_eq!(
            r.masks().known().raw() >> 7 & 1 == 1,
            k,
            "{s:?} masks().known"
        );
        let mut t = QuadTile128::new();
        t.set_unchecked(100, s);
        assert_eq!(t.known_mask().raw() >> 100 & 1 == 1, k, "{s:?} known_mask");
    }
}

#[test]
fn legacy_raw_planes_include_s() {
    let mut r = QuadroReg32::new();
    r.set_unchecked(0, QuadState::S);
    assert_eq!(r.mask_true().raw(), 1);
    assert_eq!(r.mask_false().raw(), 1);
    assert_eq!(r.masks().t.raw(), 0);
    assert_eq!(r.masks().f.raw(), 0);
}

fn d32(a: QuadState, b: QuadState) -> StateDelta32 {
    let (mut p, mut c) = (QuadroReg32::new(), QuadroReg32::new());
    p.set_unchecked(3, a);
    c.set_unchecked(3, b);
    StateDelta32::from_regs(p, c)
}

fn d128(a: QuadState, b: QuadState) -> StateDelta128 {
    let (mut p, mut c) = (QuadTile128::new(), QuadTile128::new());
    p.set_unchecked(90, a);
    c.set_unchecked(90, b);
    StateDelta128::from_tiles(p, c)
}

#[test]
fn legacy_delta_key_transitions() {
    use QuadState::*;
    // (from, to, entered_true, entered_false, left_true, left_false, became_known, became_unknown)
    let rows = [
        (N, S, true, true, false, false, false, false),
        (S, N, false, false, true, true, false, false),
        (T, S, false, true, false, false, false, true),
        (F, S, true, false, false, false, false, true),
        (S, T, false, false, false, true, true, false),
        (N, T, true, false, false, false, true, false),
    ];
    for (a, b, et, ef, lt, lf, bk, bu) in rows {
        let d = d32(a, b);
        let bit = |m: semantic_core_quad::QuadMask32| m.raw() >> 3 & 1 == 1;
        assert_eq!(
            (
                bit(d.entered_true),
                bit(d.entered_false),
                bit(d.left_true),
                bit(d.left_false),
                bit(d.became_known),
                bit(d.became_unknown)
            ),
            (et, ef, lt, lf, bk, bu),
            "32 {a:?}->{b:?}"
        );
        let e = d128(a, b);
        let bit2 = |m: semantic_core_quad::QuadMask128| m.raw() >> 90 & 1 == 1;
        assert_eq!(
            (
                bit2(e.entered_true),
                bit2(e.entered_false),
                bit2(e.left_true),
                bit2(e.left_false),
                bit2(e.became_known),
                bit2(e.became_unknown)
            ),
            (et, ef, lt, lf, bk, bu),
            "128 {a:?}->{b:?}"
        );
    }
}
