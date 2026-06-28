use ton618_core::quadro::{
    iter_mask_indices, QuadState, QuadroError, QuadroReg, StateDelta, LSB_MASK,
};

#[cfg(feature = "alloc")]
use ton618_core::quadro::{DeltaSoA, QuadroBank};

fn lane_mask(index: usize) -> u64 {
    1u64 << (index * 2)
}

fn all_states() -> [QuadState; 4] {
    [QuadState::N, QuadState::F, QuadState::T, QuadState::S]
}

fn reg_with_lane(state: QuadState) -> QuadroReg {
    let mut reg = QuadroReg::new();
    reg.try_set(0, state.bits()).unwrap();
    reg
}

fn repeated_pattern_reg(offset: usize) -> QuadroReg {
    let states = all_states();
    let mut reg = QuadroReg::new();
    for index in 0..32 {
        let state = states[(index + offset) % states.len()];
        reg.try_set(index, state.bits()).unwrap();
    }
    reg
}

fn sparse_reference_reg(mask: u64, state: QuadState) -> QuadroReg {
    let mut reg = QuadroReg::new();
    for index in 0..32 {
        let lane = lane_mask(index);
        let lane_state = if mask & lane != 0 {
            state
        } else {
            QuadState::N
        };
        reg.try_set(index, lane_state.bits()).unwrap();
    }
    reg
}

fn masked_write_reference(original: QuadroReg, mask: u64, state: QuadState) -> QuadroReg {
    let mut reg = original;
    for index in 0..32 {
        let lane = lane_mask(index);
        if mask & lane != 0 {
            reg.try_set(index, state.bits()).unwrap();
        }
    }
    reg
}

fn mask_for_state(reg: QuadroReg, predicate: impl Fn(QuadState) -> bool) -> u64 {
    let mut mask = 0;
    for index in 0..32 {
        if predicate(reg.try_get(index).unwrap()) {
            mask |= lane_mask(index);
        }
    }
    mask
}

fn reference_delta(prev: QuadroReg, next: QuadroReg) -> StateDelta {
    let mut delta = StateDelta::default();

    for index in 0..32 {
        let lane = lane_mask(index);
        let prev_state = prev.try_get(index).unwrap();
        let next_state = next.try_get(index).unwrap();

        if next_state == QuadState::T && prev_state != QuadState::T {
            delta.entered_true |= lane;
        }
        if prev_state == QuadState::T && next_state != QuadState::T {
            delta.left_true |= lane;
        }
        if next_state == QuadState::F && prev_state != QuadState::F {
            delta.entered_false |= lane;
        }
        if prev_state == QuadState::F && next_state != QuadState::F {
            delta.left_false |= lane;
        }
        if next_state == QuadState::S && prev_state != QuadState::S {
            delta.entered_super |= lane;
        }
        if prev_state == QuadState::S && next_state != QuadState::S {
            delta.left_super |= lane;
        }
    }

    delta
}

fn assert_lsb_aligned(mask: u64) {
    assert_eq!(mask & !LSB_MASK, 0);
}

fn lcg(seed: &mut u64) -> u64 {
    *seed = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    *seed
}

fn random_reg(seed: &mut u64) -> QuadroReg {
    let mut reg = QuadroReg::new();
    for index in 0..32 {
        let state = (lcg(seed) & 0b11) as u8;
        reg.try_set(index, state).unwrap();
    }
    reg
}

fn reference_bank_delta(left: &[QuadroReg], right: &[QuadroReg]) -> Vec<StateDelta> {
    left.iter()
        .copied()
        .zip(right.iter().copied())
        .map(|(lhs, rhs)| reference_delta(lhs, rhs))
        .collect()
}

fn expected_bank_merge(left: &[QuadroReg], right: &[QuadroReg]) -> Vec<QuadroReg> {
    left.iter()
        .copied()
        .zip(right.iter().copied())
        .map(|(lhs, rhs)| lhs.merge(rhs))
        .collect()
}

fn expected_bank_intersect(left: &[QuadroReg], right: &[QuadroReg]) -> Vec<QuadroReg> {
    left.iter()
        .copied()
        .zip(right.iter().copied())
        .map(|(lhs, rhs)| lhs.intersect(rhs))
        .collect()
}

fn expected_bank_inverse(regs: &[QuadroReg]) -> Vec<QuadroReg> {
    regs.iter().copied().map(QuadroReg::inverse).collect()
}

#[test]
fn merge_truth_table_is_bitwise_join() {
    for lhs in all_states() {
        for rhs in all_states() {
            let merged = reg_with_lane(lhs).merge(reg_with_lane(rhs));
            let expected = QuadState::try_from(lhs.bits() | rhs.bits()).unwrap();
            assert_eq!(merged.try_get(0).unwrap(), expected, "{lhs:?} | {rhs:?}");
        }
    }
}

#[test]
fn merge_and_intersect_match_lanewise_reference_on_packed_patterns() {
    let lhs = repeated_pattern_reg(0);
    let rhs = repeated_pattern_reg(1);

    let merged = lhs.merge(rhs);
    let intersected = lhs.intersect(rhs);

    for index in 0..32 {
        let lhs_state = lhs.try_get(index).unwrap();
        let rhs_state = rhs.try_get(index).unwrap();
        assert_eq!(
            merged.try_get(index).unwrap(),
            QuadState::try_from(lhs_state.bits() | rhs_state.bits()).unwrap()
        );
        assert_eq!(
            intersected.try_get(index).unwrap(),
            QuadState::try_from(lhs_state.bits() & rhs_state.bits()).unwrap()
        );
    }
}

#[test]
fn intersect_truth_table_is_bitwise_meet() {
    for lhs in all_states() {
        for rhs in all_states() {
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
fn inverse_preserves_packed_pattern_lane_by_lane() {
    let reg = repeated_pattern_reg(0);
    let inverted = reg.inverse();

    for index in 0..32 {
        let expected = reg.try_get(index).unwrap().inverse();
        assert_eq!(inverted.try_get(index).unwrap(), expected);
    }
}

#[test]
fn mask_extraction_matrix_is_lsb_aligned_and_disjoint() {
    let reg = repeated_pattern_reg(0);
    let masks = reg.masks_all();
    let expected_null = mask_for_state(reg, |state| state == QuadState::N);
    let expected_false = mask_for_state(reg, |state| state == QuadState::F);
    let expected_true = mask_for_state(reg, |state| state == QuadState::T);
    let expected_super = mask_for_state(reg, |state| state == QuadState::S);

    assert_eq!(masks.null, expected_null);
    assert_eq!(masks.strict_false, expected_false);
    assert_eq!(masks.strict_true, expected_true);
    assert_eq!(masks.super_, expected_super);
    assert_eq!(
        masks.non_null,
        expected_false | expected_true | expected_super
    );

    assert_lsb_aligned(masks.null);
    assert_lsb_aligned(masks.strict_false);
    assert_lsb_aligned(masks.strict_true);
    assert_lsb_aligned(masks.super_);
    assert_lsb_aligned(masks.non_null);

    assert_eq!(
        masks.null | masks.strict_false | masks.strict_true | masks.super_,
        LSB_MASK
    );
    assert_eq!(masks.null & masks.strict_false, 0);
    assert_eq!(masks.null & masks.strict_true, 0);
    assert_eq!(masks.null & masks.super_, 0);
    assert_eq!(masks.strict_false & masks.strict_true, 0);
    assert_eq!(masks.strict_false & masks.super_, 0);
    assert_eq!(masks.strict_true & masks.super_, 0);
    assert_eq!(reg.mask_super(), expected_super);
    assert_eq!(
        reg.mask_non_null(),
        expected_false | expected_true | expected_super
    );
    assert_eq!(reg.popcount_quadits(), 24);
}

#[test]
fn try_get_and_try_set_validate_bounds_and_state() {
    let mut reg = QuadroReg::new();

    assert_eq!(reg.try_get(0).unwrap(), QuadState::N);
    assert_eq!(reg.try_get(31).unwrap(), QuadState::N);
    assert!(matches!(
        reg.try_get(32),
        Err(QuadroError::IndexOutOfRange { index: 32 })
    ));

    reg.try_set(0, QuadState::F.bits()).unwrap();
    reg.try_set(1, QuadState::T.bits()).unwrap();
    reg.try_set(31, QuadState::S.bits()).unwrap();

    assert_eq!(reg.try_get(0).unwrap(), QuadState::F);
    assert_eq!(reg.try_get(1).unwrap(), QuadState::T);
    assert_eq!(reg.try_get(31).unwrap(), QuadState::S);

    assert_eq!(reg.raw() & 0b11, QuadState::F.bits() as u64);
    assert_eq!((reg.raw() >> 2) & 0b11, QuadState::T.bits() as u64);
    assert_eq!((reg.raw() >> 62) & 0b11, QuadState::S.bits() as u64);

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
fn try_set_by_mask_applies_selected_lanes_only() {
    let original = repeated_pattern_reg(0);
    let mask = lane_mask(0) | lane_mask(3) | lane_mask(7) | lane_mask(12) | lane_mask(31);

    for state in all_states() {
        let mut reg = original;
        reg.try_set_by_mask(mask, state.bits()).unwrap();

        let expected = masked_write_reference(original, mask, state);
        for index in 0..32 {
            let lane = lane_mask(index);
            let expected_state = if mask & lane != 0 {
                state
            } else {
                original.try_get(index).unwrap()
            };
            assert_eq!(reg.try_get(index).unwrap(), expected_state, "index {index}");
        }
        assert_eq!(reg, expected);
    }
}

#[test]
fn clear_by_mask_and_force_super_follow_selected_lanes_only() {
    let original = repeated_pattern_reg(1);
    let mask = lane_mask(1) | lane_mask(4) | lane_mask(9) | lane_mask(17) | lane_mask(31);

    let mut cleared = original;
    cleared.clear_by_mask(mask).unwrap();

    let mut forced = original;
    forced.force_super(mask).unwrap();

    let expected_cleared = masked_write_reference(original, mask, QuadState::N);
    let expected_forced = masked_write_reference(original, mask, QuadState::S);

    assert_eq!(cleared, expected_cleared);
    assert_eq!(forced, expected_forced);

    for index in 0..32 {
        let lane = lane_mask(index);
        let original_state = original.try_get(index).unwrap();
        let expected_cleared_state = if mask & lane != 0 {
            QuadState::N
        } else {
            original_state
        };
        let expected_forced_state = if mask & lane != 0 {
            QuadState::S
        } else {
            original_state
        };
        assert_eq!(cleared.try_get(index).unwrap(), expected_cleared_state);
        assert_eq!(forced.try_get(index).unwrap(), expected_forced_state);
    }
}

#[test]
fn iter_mask_indices_reads_lsb_aligned_lanes() {
    let cases = [
        (0u64, vec![]),
        (lane_mask(0), vec![0]),
        (lane_mask(31), vec![31]),
        (
            lane_mask(0) | lane_mask(3) | lane_mask(7) | lane_mask(12) | lane_mask(31),
            vec![0, 3, 7, 12, 31],
        ),
        (LSB_MASK, (0..32).collect::<Vec<_>>()),
    ];

    for (mask, expected) in cases {
        let lanes: Vec<_> = iter_mask_indices(mask).unwrap().collect();
        assert_eq!(lanes, expected);

        let reg = sparse_reference_reg(mask, QuadState::F);
        assert_eq!(reg.popcount_quadits(), expected.len());
        assert_eq!(reg.mask_non_null(), mask);
    }

    assert!(iter_mask_indices(1u64 << 1).is_err());
}

#[test]
fn delta_reports_enter_and_leave_transitions() {
    let cases = [
        (QuadState::N, QuadState::N, StateDelta::default()),
        (
            QuadState::N,
            QuadState::F,
            StateDelta {
                entered_false: lane_mask(0),
                ..StateDelta::default()
            },
        ),
        (
            QuadState::N,
            QuadState::T,
            StateDelta {
                entered_true: lane_mask(0),
                ..StateDelta::default()
            },
        ),
        (
            QuadState::N,
            QuadState::S,
            StateDelta {
                entered_super: lane_mask(0),
                ..StateDelta::default()
            },
        ),
        (
            QuadState::F,
            QuadState::N,
            StateDelta {
                left_false: lane_mask(0),
                ..StateDelta::default()
            },
        ),
        (QuadState::F, QuadState::F, StateDelta::default()),
        (
            QuadState::F,
            QuadState::T,
            StateDelta {
                entered_true: lane_mask(0),
                left_false: lane_mask(0),
                ..StateDelta::default()
            },
        ),
        (
            QuadState::F,
            QuadState::S,
            StateDelta {
                entered_super: lane_mask(0),
                left_false: lane_mask(0),
                ..StateDelta::default()
            },
        ),
        (
            QuadState::T,
            QuadState::N,
            StateDelta {
                left_true: lane_mask(0),
                ..StateDelta::default()
            },
        ),
        (
            QuadState::T,
            QuadState::F,
            StateDelta {
                entered_false: lane_mask(0),
                left_true: lane_mask(0),
                ..StateDelta::default()
            },
        ),
        (QuadState::T, QuadState::T, StateDelta::default()),
        (
            QuadState::T,
            QuadState::S,
            StateDelta {
                entered_super: lane_mask(0),
                left_true: lane_mask(0),
                ..StateDelta::default()
            },
        ),
        (
            QuadState::S,
            QuadState::N,
            StateDelta {
                left_super: lane_mask(0),
                ..StateDelta::default()
            },
        ),
        (
            QuadState::S,
            QuadState::F,
            StateDelta {
                entered_false: lane_mask(0),
                left_super: lane_mask(0),
                ..StateDelta::default()
            },
        ),
        (
            QuadState::S,
            QuadState::T,
            StateDelta {
                entered_true: lane_mask(0),
                left_super: lane_mask(0),
                ..StateDelta::default()
            },
        ),
        (QuadState::S, QuadState::S, StateDelta::default()),
    ];

    for (prev, curr, expected) in cases {
        let mut old = QuadroReg::new();
        let mut next = QuadroReg::new();
        old.try_set(0, prev.bits()).unwrap();
        next.try_set(0, curr.bits()).unwrap();

        let delta = old.calc_delta(next);
        assert_eq!(delta, expected, "{prev:?} -> {curr:?}");
    }
}

#[test]
fn packed_delta_equivalence_matches_try_get_reference() {
    let mut seed = 0x5a17_2026_0628_0001;

    for _ in 0..16 {
        let prev = random_reg(&mut seed);
        let next = random_reg(&mut seed);
        let delta = prev.calc_delta(next);
        let expected = reference_delta(prev, next);

        assert_eq!(delta, expected);
        assert_lsb_aligned(delta.entered_true);
        assert_lsb_aligned(delta.left_true);
        assert_lsb_aligned(delta.entered_false);
        assert_lsb_aligned(delta.left_false);
        assert_lsb_aligned(delta.entered_super);
        assert_lsb_aligned(delta.left_super);
        assert_eq!(
            delta.entered_true
                & delta.left_true
                & delta.entered_false
                & delta.left_false
                & delta.entered_super
                & delta.left_super,
            0
        );
    }
}

#[cfg(feature = "alloc")]
#[test]
fn bank_operations_match_scalar_paths() {
    let counts = [1usize, 2, 3, 4, 17, 64];

    for &count in &counts {
        let mut seed = 0xdec0_0000_0000_0000 ^ count as u64;
        let left_regs: Vec<_> = (0..count).map(|_| random_reg(&mut seed)).collect();
        let right_regs: Vec<_> = (0..count).map(|_| random_reg(&mut seed)).collect();

        let mut left = QuadroBank::from_regs(left_regs.clone());
        let right = QuadroBank::from_regs(right_regs.clone());
        let expected_merge = expected_bank_merge(&left_regs, &right_regs);
        left.merge_inplace(&right).unwrap();
        assert_eq!(
            left.as_slice(),
            expected_merge.as_slice(),
            "merge count {count}"
        );

        let mut left = QuadroBank::from_regs(left_regs.clone());
        let expected_intersect = expected_bank_intersect(&left_regs, &right_regs);
        left.intersect_inplace(&right).unwrap();
        assert_eq!(
            left.as_slice(),
            expected_intersect.as_slice(),
            "intersect count {count}"
        );

        let mut left = QuadroBank::from_regs(left_regs.clone());
        let expected_inverse = expected_bank_inverse(&left_regs);
        left.inverse_inplace();
        assert_eq!(
            left.as_slice(),
            expected_inverse.as_slice(),
            "inverse count {count}"
        );

        let soa = QuadroBank::from_regs(left_regs.clone())
            .calc_deltas_soa(&right)
            .unwrap();
        let expected_deltas = reference_bank_delta(&left_regs, &right_regs);
        let expected_soa = {
            let mut soa = DeltaSoA::new();
            for delta in expected_deltas.iter().copied() {
                soa.push(delta);
            }
            soa
        };

        assert_eq!(soa, expected_soa, "soa count {count}");
        assert_eq!(soa.to_aos(), expected_deltas, "to_aos count {count}");
    }
}
