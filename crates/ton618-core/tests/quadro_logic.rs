use ton618_core::quadro::{iter_mask_indices, QuadState, QuadroError, QuadroReg, LSB_MASK};

#[cfg(feature = "alloc")]
use ton618_core::quadro::{DeltaSoA, QuadroBank};

fn lane_mask(index: usize) -> u64 {
    1u64 << (index * 2)
}

fn reg_with_lane(state: QuadState) -> QuadroReg {
    let mut reg = QuadroReg::new();
    reg.try_set(0, state.bits()).unwrap();
    reg
}

fn set_lane(reg: &mut QuadroReg, index: usize, state: QuadState) {
    reg.try_set(index, state.bits()).unwrap();
}

#[test]
fn merge_truth_table_is_bitwise_join() {
    let states = [QuadState::N, QuadState::F, QuadState::T, QuadState::S];

    for &lhs in &states {
        for &rhs in &states {
            let merged = reg_with_lane(lhs).merge(reg_with_lane(rhs));
            let expected = QuadState::try_from(lhs.bits() | rhs.bits()).unwrap();
            assert_eq!(merged.try_get(0).unwrap(), expected, "{lhs:?} | {rhs:?}");
        }
    }
}

#[test]
fn intersect_truth_table_is_bitwise_meet() {
    let states = [QuadState::N, QuadState::F, QuadState::T, QuadState::S];

    for &lhs in &states {
        for &rhs in &states {
            let intersected = reg_with_lane(lhs).intersect(reg_with_lane(rhs));
            let expected = QuadState::try_from(lhs.bits() & rhs.bits()).unwrap();
            assert_eq!(
                intersected.try_get(0).unwrap(),
                expected,
                "{lhs:?} & {rhs:?}"
            );
        }
    }
}

#[test]
fn inverse_truth_table_matches_plane_swap() {
    let cases = [
        (QuadState::N, QuadState::N),
        (QuadState::F, QuadState::T),
        (QuadState::T, QuadState::F),
        (QuadState::S, QuadState::S),
    ];

    for (input, expected) in cases {
        let reg = reg_with_lane(input).inverse();
        assert_eq!(reg.try_get(0).unwrap(), expected);
    }
}

#[test]
fn mask_extraction_tracks_lane_states() {
    let mut reg = QuadroReg::new();
    set_lane(&mut reg, 1, QuadState::F);
    set_lane(&mut reg, 2, QuadState::T);
    set_lane(&mut reg, 3, QuadState::S);

    let masks = reg.masks_all();
    let all_lanes = LSB_MASK;
    assert_eq!(
        masks.null,
        all_lanes & !(lane_mask(1) | lane_mask(2) | lane_mask(3))
    );
    assert_eq!(masks.strict_false, lane_mask(1));
    assert_eq!(masks.strict_true, lane_mask(2));
    assert_eq!(masks.super_, lane_mask(3));
    assert_eq!(masks.non_null, lane_mask(1) | lane_mask(2) | lane_mask(3));
    assert_eq!(reg.mask_super(), lane_mask(3));
    assert_eq!(
        reg.mask_non_null(),
        lane_mask(1) | lane_mask(2) | lane_mask(3)
    );
    assert_eq!(reg.popcount_quadits(), 3);
}

#[test]
fn try_get_and_try_set_validate_bounds_and_state() {
    let mut reg = QuadroReg::new();

    assert_eq!(reg.try_get(0).unwrap(), QuadState::N);
    assert!(matches!(
        reg.try_get(32),
        Err(QuadroError::IndexOutOfRange { index: 32 })
    ));

    reg.try_set(0, QuadState::F.bits()).unwrap();
    assert_eq!(reg.try_get(0).unwrap(), QuadState::F);

    assert!(matches!(
        reg.try_set(32, QuadState::F.bits()),
        Err(QuadroError::IndexOutOfRange { index: 32 })
    ));
    assert!(matches!(
        reg.try_set(0, 0b100),
        Err(QuadroError::InvalidState { state: 0b100 })
    ));
}

#[test]
fn try_set_by_mask_rejects_alignment_and_state() {
    let mut reg = QuadroReg::new();

    assert!(matches!(
        reg.try_set_by_mask(1u64 << 1, QuadState::F.bits()),
        Err(QuadroError::MisalignedMask { mask })
        if mask == 1u64 << 1
    ));
    assert!(matches!(
        reg.try_set_by_mask(lane_mask(0), 0b100),
        Err(QuadroError::InvalidState { state: 0b100 })
    ));
}

#[test]
fn try_set_by_mask_applies_selected_lanes() {
    let mut reg = QuadroReg::new();
    let mask = lane_mask(0) | lane_mask(2) | lane_mask(5);
    reg.try_set_by_mask(mask, QuadState::S.bits()).unwrap();

    assert_eq!(reg.try_get(0).unwrap(), QuadState::S);
    assert_eq!(reg.try_get(2).unwrap(), QuadState::S);
    assert_eq!(reg.try_get(5).unwrap(), QuadState::S);
    assert_eq!(reg.try_get(1).unwrap(), QuadState::N);
}

#[test]
fn delta_reports_enter_and_leave_transitions() {
    let mut old = QuadroReg::new();
    let mut next = QuadroReg::new();

    set_lane(&mut old, 0, QuadState::N);
    set_lane(&mut next, 0, QuadState::T);
    set_lane(&mut old, 1, QuadState::T);
    set_lane(&mut next, 1, QuadState::N);
    set_lane(&mut old, 2, QuadState::N);
    set_lane(&mut next, 2, QuadState::F);
    set_lane(&mut old, 3, QuadState::F);
    set_lane(&mut next, 3, QuadState::N);
    set_lane(&mut old, 4, QuadState::N);
    set_lane(&mut next, 4, QuadState::S);
    set_lane(&mut old, 5, QuadState::S);
    set_lane(&mut next, 5, QuadState::N);

    let delta = old.calc_delta(next);
    assert_eq!(delta.entered_true, lane_mask(0));
    assert_eq!(delta.left_true, lane_mask(1));
    assert_eq!(delta.entered_false, lane_mask(2));
    assert_eq!(delta.left_false, lane_mask(3));
    assert_eq!(delta.entered_super, lane_mask(4));
    assert_eq!(delta.left_super, lane_mask(5));
}

#[test]
fn iter_mask_indices_reads_lsb_aligned_lanes() {
    let mask = lane_mask(0) | lane_mask(3) | lane_mask(7);
    let lanes: Vec<_> = iter_mask_indices(mask).unwrap().collect();
    assert_eq!(lanes, vec![0, 3, 7]);

    assert!(iter_mask_indices(1u64 << 1).is_err());
}

#[cfg(feature = "alloc")]
#[test]
fn bank_operations_match_scalar_paths() {
    let mut left = QuadroBank::from_regs(vec![
        reg_with_lane(QuadState::N),
        reg_with_lane(QuadState::F),
        reg_with_lane(QuadState::T),
        reg_with_lane(QuadState::S),
    ]);
    let right = QuadroBank::from_regs(vec![
        reg_with_lane(QuadState::S),
        reg_with_lane(QuadState::T),
        reg_with_lane(QuadState::F),
        reg_with_lane(QuadState::N),
    ]);

    let mut expected_merge = left.as_slice().to_vec();
    for (slot, rhs) in expected_merge.iter_mut().zip(right.as_slice().iter()) {
        *slot = slot.merge(*rhs);
    }
    left.merge_inplace(&right).unwrap();
    assert_eq!(left.as_slice(), expected_merge.as_slice());

    let mut left = QuadroBank::from_regs(vec![
        reg_with_lane(QuadState::N),
        reg_with_lane(QuadState::F),
        reg_with_lane(QuadState::T),
        reg_with_lane(QuadState::S),
    ]);
    let mut expected_intersect = left.as_slice().to_vec();
    for (slot, rhs) in expected_intersect.iter_mut().zip(right.as_slice().iter()) {
        *slot = slot.intersect(*rhs);
    }
    left.intersect_inplace(&right).unwrap();
    assert_eq!(left.as_slice(), expected_intersect.as_slice());

    let mut left = QuadroBank::from_regs(vec![
        reg_with_lane(QuadState::N),
        reg_with_lane(QuadState::F),
        reg_with_lane(QuadState::T),
        reg_with_lane(QuadState::S),
    ]);
    let expected_inverse: Vec<_> = left
        .as_slice()
        .iter()
        .copied()
        .map(|reg| reg.inverse())
        .collect();
    left.inverse_inplace();
    assert_eq!(left.as_slice(), expected_inverse.as_slice());
}

#[cfg(feature = "alloc")]
#[test]
fn bank_delta_soa_matches_scalar_deltas() {
    let left = QuadroBank::from_regs(vec![
        reg_with_lane(QuadState::N),
        reg_with_lane(QuadState::F),
        reg_with_lane(QuadState::T),
    ]);
    let right = QuadroBank::from_regs(vec![
        reg_with_lane(QuadState::T),
        reg_with_lane(QuadState::N),
        reg_with_lane(QuadState::S),
    ]);

    let soa = left.calc_deltas_soa(&right).unwrap();
    assert_eq!(soa.len(), 3);

    let mut scalar = DeltaSoA::new();
    for (lhs, rhs) in left
        .as_slice()
        .iter()
        .copied()
        .zip(right.as_slice().iter().copied())
    {
        scalar.push(lhs.calc_delta(rhs));
    }

    assert_eq!(soa, scalar);
}
