use semantic_core_quad::{
    ExactStateDelta32, PlaneDelta32, QuadLaneMask32, QuadMask32, QuadPhysicalMask32, QuadState,
    QuadTile128, QuadTileBank, QuadroBank, QuadroReg32,
};

const RAW_SAMPLES: [u64; 7] = [
    0x0000_0000_0000_0000,
    0xFFFF_FFFF_FFFF_FFFF,
    0x5555_5555_5555_5555,
    0xAAAA_AAAA_AAAA_AAAA,
    0x0123_4567_89AB_CDEF,
    0xE4E4_E4E4_E4E4_E4E4,
    0xBADC_0FFE_DEAD_BEEF,
];

fn reg(raw: u64) -> QuadroReg32 {
    QuadroReg32::from_raw(raw)
}

fn repeated(state: QuadState) -> QuadroReg32 {
    let mut value = QuadroReg32::new();
    for lane in 0..QuadroReg32::LANES {
        value.set_unchecked(lane, state);
    }
    value
}

fn tile_binary_oracle(
    lhs: [QuadroReg32; 4],
    rhs: [QuadroReg32; 4],
    operation: fn(QuadroReg32, QuadroReg32) -> QuadroReg32,
) -> QuadTile128 {
    QuadTile128::from_regs([
        operation(lhs[0], rhs[0]),
        operation(lhs[1], rhs[1]),
        operation(lhs[2], rhs[2]),
        operation(lhs[3], rhs[3]),
    ])
}

#[test]
fn qualification_state_encoding_is_frozen() {
    assert_eq!(QuadState::N.bits(), 0b00);
    assert_eq!(QuadState::F.bits(), 0b01);
    assert_eq!(QuadState::T.bits(), 0b10);
    assert_eq!(QuadState::S.bits(), 0b11);

    assert_eq!(QuadState::from_bits(0), Some(QuadState::N));
    assert_eq!(QuadState::from_bits(1), Some(QuadState::F));
    assert_eq!(QuadState::from_bits(2), Some(QuadState::T));
    assert_eq!(QuadState::from_bits(3), Some(QuadState::S));
    assert_eq!(QuadState::from_bits(4), None);
}

#[test]
fn qualification_register_scalar_swar_and_default_maps_agree() {
    for lhs_raw in RAW_SAMPLES {
        let lhs = reg(lhs_raw);
        assert_eq!(lhs.map_not_scalar(), lhs.map_not_swar());
        assert_eq!(lhs.map_not_swar(), lhs.map_not());

        for rhs_raw in RAW_SAMPLES {
            let rhs = reg(rhs_raw);
            assert_eq!(lhs.map_xor_scalar(rhs), lhs.map_xor_swar(rhs));
            assert_eq!(lhs.map_xor_swar(rhs), lhs.map_xor(rhs));
            assert_eq!(lhs.map_and_scalar(rhs), lhs.map_and_swar(rhs));
            assert_eq!(lhs.map_and_swar(rhs), lhs.map_and(rhs));
            assert_eq!(lhs.map_or_scalar(rhs), lhs.map_or_swar(rhs));
            assert_eq!(lhs.map_or_swar(rhs), lhs.map_or(rhs));
            assert_eq!(lhs.map_implies_scalar(rhs), lhs.map_implies_swar(rhs));
            assert_eq!(lhs.map_implies_swar(rhs), lhs.map_implies(rhs));
            assert_eq!(lhs.map_nand_scalar(rhs), lhs.map_nand_swar(rhs));
            assert_eq!(lhs.map_nand_swar(rhs), lhs.map_nand(rhs));
            assert_eq!(lhs.map_nor_scalar(rhs), lhs.map_nor_swar(rhs));
            assert_eq!(lhs.map_nor_swar(rhs), lhs.map_nor(rhs));
        }
    }
}

#[test]
fn qualification_truth_policy_invariants() {
    for a in QuadState::ALL {
        let a_reg = repeated(a);
        assert_eq!(a_reg.map_not().map_not(), a_reg, "NOT is not involutive");
        assert_eq!(a_reg.map_xor(a_reg), repeated(QuadState::N));

        for b in QuadState::ALL {
            let b_reg = repeated(b);
            assert_eq!(
                a_reg.map_nand(b_reg),
                a_reg.map_and(b_reg).map_not(),
                "NAND derivation mismatch"
            );
            assert_eq!(
                a_reg.map_nor(b_reg),
                a_reg.map_or(b_reg).map_not(),
                "NOR derivation mismatch"
            );
            assert_eq!(
                a_reg.map_implies(b_reg),
                repeated(a.inverse().join(b)),
                "IMPLIES policy mismatch"
            );

            assert_eq!(a_reg.map_xor(b_reg), b_reg.map_xor(a_reg));
            assert_eq!(a_reg.map_and(b_reg), b_reg.map_and(a_reg));
            assert_eq!(a_reg.map_or(b_reg), b_reg.map_or(a_reg));
            assert_eq!(a_reg.map_nand(b_reg), b_reg.map_nand(a_reg));
            assert_eq!(a_reg.map_nor(b_reg), b_reg.map_nor(a_reg));
        }
    }

    assert_ne!(
        repeated(QuadState::F).map_implies(repeated(QuadState::N)),
        repeated(QuadState::N).map_implies(repeated(QuadState::F))
    );
    assert_eq!(
        repeated(QuadState::F).map_implies(repeated(QuadState::N)),
        repeated(QuadState::T)
    );
    assert_eq!(
        repeated(QuadState::N).map_implies(repeated(QuadState::F)),
        repeated(QuadState::F)
    );
}

#[test]
fn qualification_mask_bridge_roundtrip_and_rejection() {
    let dense_masks = [
        0x0000_0000,
        0x0000_0001,
        0x8000_0000,
        0x8000_0001,
        0xFFFF_FFFF,
    ];
    for dense in dense_masks {
        let mask = QuadMask32::try_new(dense).expect("dense mask must be valid");
        let lane_mask: QuadLaneMask32 = mask;
        let physical = lane_mask.to_physical();
        assert_eq!(physical.bits() & 0xAAAA_AAAA_AAAA_AAAA, 0);
        assert_eq!(physical.try_to_lane().unwrap().raw(), dense);
    }

    for odd_bit in [1u64 << 1, 1u64 << 3, 1u64 << 63] {
        assert!(QuadPhysicalMask32::try_from_bits(odd_bit).is_err());
    }

    let highest_lane = QuadPhysicalMask32::try_from_bits(1u64 << 62).unwrap();
    assert_eq!(highest_lane.try_to_lane().unwrap().raw(), 1u64 << 31);
}

#[test]
fn qualification_exact_and_plane_delta_matrix() {
    let lane = 9usize;
    let lane_bit = 1u64 << lane;
    for previous_state in QuadState::ALL {
        for current_state in QuadState::ALL {
            let mut previous = QuadroReg32::new();
            let mut current = QuadroReg32::new();
            previous.set_unchecked(lane, previous_state);
            current.set_unchecked(lane, current_state);

            let exact: ExactStateDelta32 = QuadroReg32::exact_state_delta(previous, current);
            let exact_fields = [
                (exact.entered_neither, QuadState::N, true),
                (exact.left_neither, QuadState::N, false),
                (exact.entered_strict_false, QuadState::F, true),
                (exact.left_strict_false, QuadState::F, false),
                (exact.entered_strict_true, QuadState::T, true),
                (exact.left_strict_true, QuadState::T, false),
                (exact.entered_super, QuadState::S, true),
                (exact.left_super, QuadState::S, false),
            ];
            for (mask, target, entered) in exact_fields {
                let expected = if (entered && current_state == target && previous_state != target)
                    || (!entered && previous_state == target && current_state != target)
                {
                    lane_bit
                } else {
                    0
                };
                assert_eq!(mask.raw(), expected, "exact delta mask mismatch");
                assert_eq!(mask.raw() & !lane_bit, 0);
            }

            let plane: PlaneDelta32 = QuadroReg32::plane_delta(previous, current);
            let previous_truth = previous_state.bits() & 0b10 != 0;
            let current_truth = current_state.bits() & 0b10 != 0;
            let previous_falsity = previous_state.bits() & 0b01 != 0;
            let current_falsity = current_state.bits() & 0b01 != 0;
            let plane_fields = [
                (plane.entered_truth, !previous_truth && current_truth),
                (plane.left_truth, previous_truth && !current_truth),
                (plane.entered_falsity, !previous_falsity && current_falsity),
                (plane.left_falsity, previous_falsity && !current_falsity),
            ];
            for (mask, should_be_set) in plane_fields {
                assert_eq!(mask.raw(), if should_be_set { lane_bit } else { 0 });
                assert_eq!(mask.raw() & !lane_bit, 0);
            }
        }
    }
}

#[test]
fn qualification_tile_maps_match_four_register_oracle() {
    let lhs = [
        reg(RAW_SAMPLES[0]),
        reg(RAW_SAMPLES[1]),
        reg(RAW_SAMPLES[2]),
        reg(RAW_SAMPLES[3]),
    ];
    let rhs = [
        reg(RAW_SAMPLES[4]),
        reg(RAW_SAMPLES[5]),
        reg(RAW_SAMPLES[6]),
        reg(RAW_SAMPLES[0]),
    ];
    let lhs_tile = QuadTile128::from_regs(lhs);
    let rhs_tile = QuadTile128::from_regs(rhs);
    assert_eq!(lhs_tile.to_regs(), lhs);
    assert_eq!(rhs_tile.to_regs(), rhs);
    assert_eq!(
        lhs_tile.map_not(),
        QuadTile128::from_regs(lhs.map(QuadroReg32::map_not))
    );
    assert_eq!(
        lhs_tile.map_xor(rhs_tile),
        tile_binary_oracle(lhs, rhs, QuadroReg32::map_xor)
    );
    assert_eq!(
        lhs_tile.map_and(rhs_tile),
        tile_binary_oracle(lhs, rhs, QuadroReg32::map_and)
    );
    assert_eq!(
        lhs_tile.map_or(rhs_tile),
        tile_binary_oracle(lhs, rhs, QuadroReg32::map_or)
    );
    assert_eq!(
        lhs_tile.map_implies(rhs_tile),
        tile_binary_oracle(lhs, rhs, QuadroReg32::map_implies)
    );
    assert_eq!(
        lhs_tile.map_nand(rhs_tile),
        tile_binary_oracle(lhs, rhs, QuadroReg32::map_nand)
    );
    assert_eq!(
        lhs_tile.map_nor(rhs_tile),
        tile_binary_oracle(lhs, rhs, QuadroReg32::map_nor)
    );
}

fn assert_reg_binary_map(
    operation: &str,
    lhs: [QuadroReg32; 4],
    rhs: [QuadroReg32; 4],
    apply: fn(&mut QuadroBank<4>, &QuadroBank<4>),
    oracle: fn(QuadroReg32, QuadroReg32) -> QuadroReg32,
) {
    let rhs_bank = QuadroBank::from_array(rhs);
    let rhs_before = *rhs_bank.as_array();
    let mut actual = QuadroBank::from_array(lhs);
    apply(&mut actual, &rhs_bank);
    for index in 0..4 {
        assert_eq!(
            actual.get(index).unwrap(),
            oracle(lhs[index], rhs[index]),
            "{operation} register bank element {index}"
        );
    }
    assert_eq!(
        *rhs_bank.as_array(),
        rhs_before,
        "{operation} changed rhs bank"
    );
}

#[test]
fn qualification_reg_bank_maps_match_elementwise_oracle() {
    let lhs = [
        reg(RAW_SAMPLES[0]),
        reg(RAW_SAMPLES[1]),
        reg(RAW_SAMPLES[2]),
        reg(RAW_SAMPLES[3]),
    ];
    let rhs = [
        reg(RAW_SAMPLES[4]),
        reg(RAW_SAMPLES[5]),
        reg(RAW_SAMPLES[6]),
        reg(RAW_SAMPLES[0]),
    ];
    let mut unary = QuadroBank::from_array(lhs);
    unary.map_not_inplace();
    for (index, value) in lhs.iter().enumerate() {
        assert_eq!(unary.get(index).unwrap(), value.map_not());
    }
    assert_reg_binary_map(
        "xor",
        lhs,
        rhs,
        QuadroBank::map_xor_inplace,
        QuadroReg32::map_xor,
    );
    assert_reg_binary_map(
        "and",
        lhs,
        rhs,
        QuadroBank::map_and_inplace,
        QuadroReg32::map_and,
    );
    assert_reg_binary_map(
        "or",
        lhs,
        rhs,
        QuadroBank::map_or_inplace,
        QuadroReg32::map_or,
    );
    assert_reg_binary_map(
        "implies",
        lhs,
        rhs,
        QuadroBank::map_implies_inplace,
        QuadroReg32::map_implies,
    );
    assert_reg_binary_map(
        "nand",
        lhs,
        rhs,
        QuadroBank::map_nand_inplace,
        QuadroReg32::map_nand,
    );
    assert_reg_binary_map(
        "nor",
        lhs,
        rhs,
        QuadroBank::map_nor_inplace,
        QuadroReg32::map_nor,
    );
}

fn assert_tile_binary_map(
    operation: &str,
    lhs: [QuadTile128; 4],
    rhs: [QuadTile128; 4],
    apply: fn(&mut QuadTileBank<4>, &QuadTileBank<4>),
    oracle: fn(QuadTile128, QuadTile128) -> QuadTile128,
) {
    let rhs_bank = QuadTileBank::from_array(rhs);
    let rhs_before = *rhs_bank.as_array();
    let mut actual = QuadTileBank::from_array(lhs);
    apply(&mut actual, &rhs_bank);
    for index in 0..4 {
        assert_eq!(
            actual.get(index).unwrap(),
            oracle(lhs[index], rhs[index]),
            "{operation} tile bank element {index}"
        );
    }
    assert_eq!(
        *rhs_bank.as_array(),
        rhs_before,
        "{operation} changed rhs bank"
    );
}

#[test]
fn qualification_tile_bank_maps_match_elementwise_oracle() {
    let lhs_regs = [
        reg(RAW_SAMPLES[0]),
        reg(RAW_SAMPLES[1]),
        reg(RAW_SAMPLES[2]),
        reg(RAW_SAMPLES[3]),
    ];
    let rhs_regs = [
        reg(RAW_SAMPLES[4]),
        reg(RAW_SAMPLES[5]),
        reg(RAW_SAMPLES[6]),
        reg(RAW_SAMPLES[0]),
    ];
    let lhs = [
        QuadTile128::from_regs([lhs_regs[0]; 4]),
        QuadTile128::from_regs([lhs_regs[1]; 4]),
        QuadTile128::from_regs([lhs_regs[2]; 4]),
        QuadTile128::from_regs([lhs_regs[3]; 4]),
    ];
    let rhs = [
        QuadTile128::from_regs([rhs_regs[0]; 4]),
        QuadTile128::from_regs([rhs_regs[1]; 4]),
        QuadTile128::from_regs([rhs_regs[2]; 4]),
        QuadTile128::from_regs([rhs_regs[3]; 4]),
    ];
    let mut unary = QuadTileBank::from_array(lhs);
    unary.map_not_inplace();
    for (index, value) in lhs.iter().enumerate() {
        assert_eq!(unary.get(index).unwrap(), value.map_not());
    }
    assert_tile_binary_map(
        "xor",
        lhs,
        rhs,
        QuadTileBank::map_xor_inplace,
        QuadTile128::map_xor,
    );
    assert_tile_binary_map(
        "and",
        lhs,
        rhs,
        QuadTileBank::map_and_inplace,
        QuadTile128::map_and,
    );
    assert_tile_binary_map(
        "or",
        lhs,
        rhs,
        QuadTileBank::map_or_inplace,
        QuadTile128::map_or,
    );
    assert_tile_binary_map(
        "implies",
        lhs,
        rhs,
        QuadTileBank::map_implies_inplace,
        QuadTile128::map_implies,
    );
    assert_tile_binary_map(
        "nand",
        lhs,
        rhs,
        QuadTileBank::map_nand_inplace,
        QuadTile128::map_nand,
    );
    assert_tile_binary_map(
        "nor",
        lhs,
        rhs,
        QuadTileBank::map_nor_inplace,
        QuadTile128::map_nor,
    );
}

#[test]
fn qualification_truth_and_lattice_families_remain_distinct() {
    assert_eq!(
        repeated(QuadState::F).map_and(repeated(QuadState::T)),
        repeated(QuadState::F)
    );
    assert_eq!(
        repeated(QuadState::F).meet(repeated(QuadState::T)),
        repeated(QuadState::N)
    );
    assert_eq!(
        repeated(QuadState::F).map_or(repeated(QuadState::T)),
        repeated(QuadState::T)
    );
    assert_eq!(
        repeated(QuadState::F).join(repeated(QuadState::T)),
        repeated(QuadState::S)
    );

    for state in QuadState::ALL {
        assert_eq!(repeated(state).map_not(), repeated(state.inverse()));
        assert_eq!(repeated(state).inverse(), repeated(state.inverse()));
    }
}
