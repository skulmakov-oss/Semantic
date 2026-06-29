use core::fmt;

use ton618_core::quadro::{QuadState, QuadroReg, StateDelta, QUADIT_COUNT, QUADIT_WIDTH};

#[cfg(feature = "alloc")]
use ton618_core::quadro::QuadroBank;

const LANES: usize = QUADIT_COUNT;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShadowOperation {
    ConflictMask,
    KnownMask,
    StateDelta,
    BatchMerge,
    BatchIntersect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeltaField {
    EnteredTrue,
    LeftTrue,
    EnteredFalse,
    LeftFalse,
    EnteredSuper,
    LeftSuper,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct BaselineDelta {
    entered_true: u64,
    left_true: u64,
    entered_false: u64,
    left_false: u64,
    entered_super: u64,
    left_super: u64,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct ShadowContext {
    sweep_seed: Option<u64>,
    sweep_iteration: Option<usize>,
    lhs_raw: Option<u64>,
    rhs_raw: Option<u64>,
    old_raw: Option<u64>,
    new_raw: Option<u64>,
    expected_mask: Option<u64>,
    actual_mask: Option<u64>,
    baseline_detail: Option<u64>,
    pulsar_detail: Option<u64>,
    delta_field: Option<DeltaField>,
    old_state: Option<QuadState>,
    new_state: Option<QuadState>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ShadowMismatchReport {
    operation: ShadowOperation,
    case_id: Option<u64>,
    register_index: usize,
    quadit_index: Option<u8>,
    sweep_seed: Option<u64>,
    sweep_iteration: Option<usize>,
    baseline_raw: u64,
    pulsar_raw: u64,
    baseline_detail: Option<u64>,
    pulsar_detail: Option<u64>,
    lhs_raw: Option<u64>,
    rhs_raw: Option<u64>,
    old_raw: Option<u64>,
    new_raw: Option<u64>,
    expected_mask: Option<u64>,
    actual_mask: Option<u64>,
    delta_field: Option<DeltaField>,
    old_state: Option<QuadState>,
    new_state: Option<QuadState>,
}

impl fmt::Display for ShadowMismatchReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} case={:?} reg={} lane={:?} baseline=0x{:016x} pulsar=0x{:016x}",
            self.operation,
            self.case_id,
            self.register_index,
            self.quadit_index,
            self.baseline_raw,
            self.pulsar_raw,
        )?;
        if let Some(seed) = self.sweep_seed {
            write!(f, " seed=0x{seed:016x}")?;
        }
        if let Some(iteration) = self.sweep_iteration {
            write!(f, " iter={iteration}")?;
        }

        if let Some(field) = self.delta_field {
            write!(f, " delta_field={field:?}")?;
        }
        if let Some(old_state) = self.old_state {
            write!(f, " old_state={old_state:?}")?;
        }
        if let Some(new_state) = self.new_state {
            write!(f, " new_state={new_state:?}")?;
        }
        if let Some(lhs_raw) = self.lhs_raw {
            write!(f, " lhs=0x{lhs_raw:016x}")?;
        }
        if let Some(rhs_raw) = self.rhs_raw {
            write!(f, " rhs=0x{rhs_raw:016x}")?;
        }
        if let Some(old_raw) = self.old_raw {
            write!(f, " old=0x{old_raw:016x}")?;
        }
        if let Some(new_raw) = self.new_raw {
            write!(f, " new=0x{new_raw:016x}")?;
        }
        if let Some(expected_mask) = self.expected_mask {
            write!(f, " expected_mask=0x{expected_mask:016x}")?;
        }
        if let Some(actual_mask) = self.actual_mask {
            write!(f, " actual_mask=0x{actual_mask:016x}")?;
        }
        if let Some(baseline_detail) = self.baseline_detail {
            write!(f, " baseline_detail=0x{baseline_detail:016x}")?;
        }
        if let Some(pulsar_detail) = self.pulsar_detail {
            write!(f, " pulsar_detail=0x{pulsar_detail:016x}")?;
        }

        Ok(())
    }
}

fn lane_mask(index: usize) -> u64 {
    1u64 << (index * 2)
}

fn decode_quad_state(bits: u8) -> QuadState {
    match bits & 0b11 {
        0 => QuadState::N,
        1 => QuadState::F,
        2 => QuadState::T,
        3 => QuadState::S,
        _ => unreachable!(),
    }
}

fn encode_quad_state(state: QuadState) -> u8 {
    state.bits()
}

fn set_lane(reg: &mut QuadroReg, index: usize, state: QuadState) {
    reg.try_set(index, encode_quad_state(state)).unwrap();
}

fn reg_from_lane_fn(mut f: impl FnMut(usize) -> QuadState) -> QuadroReg {
    let mut reg = QuadroReg::new();
    for index in 0..LANES {
        set_lane(&mut reg, index, f(index));
    }
    reg
}

fn all_states() -> [QuadState; 4] {
    [QuadState::N, QuadState::F, QuadState::T, QuadState::S]
}

fn pattern_all(state: QuadState) -> QuadroReg {
    reg_from_lane_fn(|_| state)
}

fn pattern_all_n() -> QuadroReg {
    pattern_all(QuadState::N)
}

fn pattern_all_f() -> QuadroReg {
    pattern_all(QuadState::F)
}

fn pattern_all_t() -> QuadroReg {
    pattern_all(QuadState::T)
}

fn pattern_all_s() -> QuadroReg {
    pattern_all(QuadState::S)
}

fn pattern_repeating_nfts() -> QuadroReg {
    let states = all_states();
    reg_from_lane_fn(|index| states[index % states.len()])
}

fn pattern_repeating_tf() -> QuadroReg {
    reg_from_lane_fn(|index| {
        if index % 2 == 0 {
            QuadState::T
        } else {
            QuadState::F
        }
    })
}

fn pattern_repeating_ft() -> QuadroReg {
    reg_from_lane_fn(|index| {
        if index % 2 == 0 {
            QuadState::F
        } else {
            QuadState::T
        }
    })
}

fn pattern_sparse_s() -> QuadroReg {
    reg_from_lane_fn(|index| {
        if matches!(index, 2 | 9 | 18 | 31) {
            QuadState::S
        } else {
            QuadState::N
        }
    })
}

fn pattern_sparse_n() -> QuadroReg {
    reg_from_lane_fn(|index| {
        if matches!(index, 1 | 7 | 19 | 30) {
            QuadState::N
        } else {
            QuadState::F
        }
    })
}

fn pattern_first_lane_conflict() -> QuadroReg {
    reg_from_lane_fn(|index| {
        if index == 0 {
            QuadState::S
        } else {
            QuadState::N
        }
    })
}

fn pattern_last_lane_conflict() -> QuadroReg {
    reg_from_lane_fn(|index| {
        if index == LANES - 1 {
            QuadState::S
        } else {
            QuadState::N
        }
    })
}

fn pattern_alternating_known_unknown() -> QuadroReg {
    reg_from_lane_fn(|index| {
        if index % 2 == 0 {
            QuadState::T
        } else {
            QuadState::N
        }
    })
}

fn pattern_mixed_hand_authored() -> QuadroReg {
    reg_from_lane_fn(|index| match index % 8 {
        0 => QuadState::N,
        1 => QuadState::F,
        2 => QuadState::T,
        3 => QuadState::S,
        4 => QuadState::F,
        5 => QuadState::T,
        6 => QuadState::N,
        _ => QuadState::S,
    })
}

fn pattern_mixed_transition() -> QuadroReg {
    reg_from_lane_fn(|index| match index % 8 {
        0 => QuadState::T,
        1 => QuadState::S,
        2 => QuadState::F,
        3 => QuadState::N,
        4 => QuadState::S,
        5 => QuadState::N,
        6 => QuadState::T,
        _ => QuadState::F,
    })
}

type Transition = (usize, QuadState, QuadState);

fn pack_transition_case(transitions: &[Transition]) -> (u64, u64) {
    let mut old_raw = 0;
    let mut new_raw = 0;

    for &(lane, old_state, new_state) in transitions {
        let shift = lane * QUADIT_WIDTH;
        let old_bits = old_state.bits() as u64;
        let new_bits = new_state.bits() as u64;
        old_raw |= old_bits << shift;
        new_raw |= new_bits << shift;
    }

    (old_raw, new_raw)
}

fn state_at(raw: u64, lane: usize) -> QuadState {
    let shift = lane * QUADIT_WIDTH;
    let bits = ((raw >> shift) & 0b11) as u8;
    decode_quad_state(bits)
}

fn all_transition_matrix_case() -> (u64, u64) {
    pack_transition_case(&[
        (0, QuadState::N, QuadState::N),
        (1, QuadState::N, QuadState::F),
        (2, QuadState::N, QuadState::T),
        (3, QuadState::N, QuadState::S),
        (4, QuadState::F, QuadState::N),
        (5, QuadState::F, QuadState::F),
        (6, QuadState::F, QuadState::T),
        (7, QuadState::F, QuadState::S),
        (8, QuadState::T, QuadState::N),
        (9, QuadState::T, QuadState::F),
        (10, QuadState::T, QuadState::T),
        (11, QuadState::T, QuadState::S),
        (12, QuadState::S, QuadState::N),
        (13, QuadState::S, QuadState::F),
        (14, QuadState::S, QuadState::T),
        (15, QuadState::S, QuadState::S),
    ])
}

fn observation_case() -> (u64, u64) {
    pack_transition_case(&[
        (0, QuadState::N, QuadState::T),
        (1, QuadState::N, QuadState::F),
        (2, QuadState::N, QuadState::S),
        (3, QuadState::N, QuadState::N),
        (4, QuadState::N, QuadState::T),
        (5, QuadState::N, QuadState::F),
        (6, QuadState::N, QuadState::S),
        (7, QuadState::N, QuadState::N),
    ])
}

fn resolution_case() -> (u64, u64) {
    pack_transition_case(&[
        (0, QuadState::S, QuadState::T),
        (1, QuadState::S, QuadState::F),
        (2, QuadState::S, QuadState::N),
        (3, QuadState::S, QuadState::T),
        (4, QuadState::S, QuadState::F),
        (5, QuadState::S, QuadState::N),
        (6, QuadState::S, QuadState::T),
        (7, QuadState::S, QuadState::F),
    ])
}

fn conflict_formation_case() -> (u64, u64) {
    pack_transition_case(&[
        (0, QuadState::T, QuadState::S),
        (1, QuadState::F, QuadState::S),
        (2, QuadState::T, QuadState::F),
        (3, QuadState::F, QuadState::T),
        (4, QuadState::T, QuadState::S),
        (5, QuadState::F, QuadState::S),
        (6, QuadState::T, QuadState::F),
        (7, QuadState::F, QuadState::T),
    ])
}

fn mixed_transition_case() -> (u64, u64) {
    pack_transition_case(&[
        (0, QuadState::N, QuadState::S),
        (1, QuadState::F, QuadState::N),
        (2, QuadState::T, QuadState::F),
        (3, QuadState::S, QuadState::T),
        (4, QuadState::N, QuadState::F),
        (5, QuadState::T, QuadState::S),
        (6, QuadState::S, QuadState::N),
        (7, QuadState::F, QuadState::T),
        (8, QuadState::N, QuadState::N),
        (9, QuadState::S, QuadState::S),
        (10, QuadState::T, QuadState::N),
        (11, QuadState::F, QuadState::F),
    ])
}

#[derive(Debug, Clone, Copy)]
struct ShadowFuzzer {
    state: u64,
}

impl ShadowFuzzer {
    fn new(seed: u64) -> Self {
        Self {
            state: if seed == 0 {
                0x1BAD_B002_7A11_C0DE
            } else {
                seed
            },
        }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.state = x;
        x
    }
}

const SHADOW_SWEEP_SEEDS: &[u64] = &[
    0,
    1,
    2,
    3,
    7,
    13,
    31,
    42,
    127,
    255,
    1024,
    0x618,
    u64::MAX,
    0x5555_5555_5555_5555,
    0xAAAA_AAAA_AAAA_AAAA,
    0xDEAD_BEEF_CAFE_BABE,
    0x0123_4567_89AB_CDEF,
    0xFEDC_BA98_7654_3210,
];

const SHADOW_SWEEP_ITERS_PER_SEED: usize = 128;

fn shadow_sweep_case_id(seed_index: usize, iteration_index: usize) -> u64 {
    (seed_index as u64)
        .saturating_mul(SHADOW_SWEEP_ITERS_PER_SEED as u64)
        .saturating_add(iteration_index as u64)
}

fn baseline_conflict_mask_raw(raw: u64) -> u64 {
    let mut mask = 0;
    for index in 0..LANES {
        if state_at(raw, index) == QuadState::S {
            mask |= lane_mask(index);
        }
    }
    mask
}

fn baseline_known_mask_raw(raw: u64) -> u64 {
    let mut mask = 0;
    for index in 0..LANES {
        if state_at(raw, index) != QuadState::N {
            mask |= lane_mask(index);
        }
    }
    mask
}

fn baseline_merge_raw(lhs_raw: u64, rhs_raw: u64) -> u64 {
    let mut raw = 0;
    for index in 0..LANES {
        let lhs_state = state_at(lhs_raw, index);
        let rhs_state = state_at(rhs_raw, index);
        let merged = decode_quad_state(lhs_state.bits() | rhs_state.bits());
        raw |= (merged.bits() as u64) << (index * QUADIT_WIDTH);
    }
    raw
}

fn baseline_intersect_raw(lhs_raw: u64, rhs_raw: u64) -> u64 {
    let mut raw = 0;
    for index in 0..LANES {
        let lhs_state = state_at(lhs_raw, index);
        let rhs_state = state_at(rhs_raw, index);
        let intersected = decode_quad_state(lhs_state.bits() & rhs_state.bits());
        raw |= (intersected.bits() as u64) << (index * QUADIT_WIDTH);
    }
    raw
}

fn baseline_delta(old_raw: u64, new_raw: u64) -> BaselineDelta {
    let mut delta = BaselineDelta::default();
    for index in 0..LANES {
        let lane = lane_mask(index);
        let old_state = state_at(old_raw, index);
        let new_state = state_at(new_raw, index);

        if old_state != QuadState::T && new_state == QuadState::T {
            delta.entered_true |= lane;
        }
        if old_state == QuadState::T && new_state != QuadState::T {
            delta.left_true |= lane;
        }
        if old_state != QuadState::F && new_state == QuadState::F {
            delta.entered_false |= lane;
        }
        if old_state == QuadState::F && new_state != QuadState::F {
            delta.left_false |= lane;
        }
        if old_state != QuadState::S && new_state == QuadState::S {
            delta.entered_super |= lane;
        }
        if old_state == QuadState::S && new_state != QuadState::S {
            delta.left_super |= lane;
        }
    }
    delta
}

fn first_differing_quadit_index(left_raw: u64, right_raw: u64) -> Option<u8> {
    let diff = left_raw ^ right_raw;
    if diff == 0 {
        return None;
    }

    for index in 0..LANES {
        if ((diff >> (index * 2)) & 0b11) != 0 {
            return Some(index as u8);
        }
    }

    None
}

fn first_differing_lane_between_masks(expected_mask: u64, actual_mask: u64) -> Option<u8> {
    first_differing_quadit_index(expected_mask, actual_mask)
}

fn first_differing_delta_field(
    expected: BaselineDelta,
    actual: StateDelta,
    old_raw: u64,
    new_raw: u64,
) -> Option<(DeltaField, u64, u64, u8, QuadState, QuadState)> {
    let pairs = [
        (
            DeltaField::EnteredTrue,
            expected.entered_true,
            actual.entered_true,
        ),
        (DeltaField::LeftTrue, expected.left_true, actual.left_true),
        (
            DeltaField::EnteredFalse,
            expected.entered_false,
            actual.entered_false,
        ),
        (
            DeltaField::LeftFalse,
            expected.left_false,
            actual.left_false,
        ),
        (
            DeltaField::EnteredSuper,
            expected.entered_super,
            actual.entered_super,
        ),
        (
            DeltaField::LeftSuper,
            expected.left_super,
            actual.left_super,
        ),
    ];

    for (field, expected_mask, actual_mask) in pairs {
        if expected_mask != actual_mask {
            let lane = first_differing_lane_between_masks(expected_mask, actual_mask).unwrap_or(0);
            return Some((
                field,
                expected_mask,
                actual_mask,
                lane,
                state_at(old_raw, lane as usize),
                state_at(new_raw, lane as usize),
            ));
        }
    }

    None
}

fn shadow_compare_raw(
    operation: ShadowOperation,
    case_id: Option<u64>,
    register_index: usize,
    baseline_raw: u64,
    pulsar_raw: u64,
    context: ShadowContext,
) -> Result<(), Box<ShadowMismatchReport>> {
    if baseline_raw == pulsar_raw {
        return Ok(());
    }

    let ShadowContext {
        sweep_seed,
        sweep_iteration,
        lhs_raw,
        rhs_raw,
        old_raw,
        new_raw,
        expected_mask,
        actual_mask,
        baseline_detail,
        pulsar_detail,
        delta_field,
        old_state,
        new_state,
    } = context;

    Err(Box::new(ShadowMismatchReport {
        operation,
        case_id,
        register_index,
        quadit_index: first_differing_quadit_index(baseline_raw, pulsar_raw),
        baseline_raw,
        pulsar_raw,
        baseline_detail: baseline_detail.or(Some(baseline_raw)),
        pulsar_detail: pulsar_detail.or(Some(pulsar_raw)),
        sweep_seed,
        sweep_iteration,
        lhs_raw,
        rhs_raw,
        old_raw,
        new_raw,
        expected_mask,
        actual_mask,
        delta_field,
        old_state,
        new_state,
    }))
}

fn shadow_compare_delta_actual(
    case_id: Option<u64>,
    register_index: usize,
    old_raw: u64,
    new_raw: u64,
    actual: StateDelta,
    context: ShadowContext,
) -> Result<(), Box<ShadowMismatchReport>> {
    let expected = baseline_delta(old_raw, new_raw);

    if expected == BaselineDelta::from(actual) {
        return Ok(());
    }

    let (delta_field, expected_mask, actual_mask, quadit_index, old_state, new_state) =
        first_differing_delta_field(expected, actual, old_raw, new_raw).unwrap_or((
            DeltaField::EnteredTrue,
            expected.entered_true,
            actual.entered_true,
            0,
            state_at(old_raw, 0),
            state_at(new_raw, 0),
        ));

    Err(Box::new(ShadowMismatchReport {
        operation: ShadowOperation::StateDelta,
        case_id,
        register_index,
        quadit_index: Some(quadit_index),
        sweep_seed: context.sweep_seed,
        sweep_iteration: context.sweep_iteration,
        baseline_raw: expected_mask,
        pulsar_raw: actual_mask,
        baseline_detail: Some(old_raw),
        pulsar_detail: Some(new_raw),
        lhs_raw: None,
        rhs_raw: None,
        old_raw: Some(old_raw),
        new_raw: Some(new_raw),
        expected_mask: Some(expected_mask),
        actual_mask: Some(actual_mask),
        delta_field: Some(delta_field),
        old_state: Some(old_state),
        new_state: Some(new_state),
    }))
}

fn shadow_compare_delta(
    case_id: Option<u64>,
    register_index: usize,
    old_raw: u64,
    new_raw: u64,
) -> Result<(), Box<ShadowMismatchReport>> {
    let old = QuadroReg::from_raw(old_raw);
    let new = QuadroReg::from_raw(new_raw);
    let actual = old.calc_delta(new);
    shadow_compare_delta_actual(
        case_id,
        register_index,
        old_raw,
        new_raw,
        actual,
        ShadowContext::default(),
    )
}

impl From<StateDelta> for BaselineDelta {
    fn from(value: StateDelta) -> Self {
        Self {
            entered_true: value.entered_true,
            left_true: value.left_true,
            entered_false: value.entered_false,
            left_false: value.left_false,
            entered_super: value.entered_super,
            left_super: value.left_super,
        }
    }
}

fn assert_shadow_ok(result: Result<(), Box<ShadowMismatchReport>>) {
    if let Err(report) = result {
        panic!("{report}");
    }
}

fn raw_compare_context(lhs_raw: u64, rhs_raw: u64) -> ShadowContext {
    ShadowContext {
        sweep_seed: None,
        sweep_iteration: None,
        lhs_raw: Some(lhs_raw),
        rhs_raw: Some(rhs_raw),
        ..ShadowContext::default()
    }
}

fn mask_compare_context(mask: u64) -> ShadowContext {
    ShadowContext {
        sweep_seed: None,
        sweep_iteration: None,
        expected_mask: Some(mask),
        actual_mask: Some(mask),
        ..ShadowContext::default()
    }
}

#[test]
fn shadow_conflict_mask_matches_scalar_baseline() {
    let cases = [
        ("all_n", pattern_all_n()),
        ("all_f", pattern_all_f()),
        ("all_t", pattern_all_t()),
        ("all_s", pattern_all_s()),
        ("sparse_s", pattern_sparse_s()),
        ("first_lane_conflict", pattern_first_lane_conflict()),
        ("last_lane_conflict", pattern_last_lane_conflict()),
        ("mixed_hand_authored", pattern_mixed_hand_authored()),
    ];

    for (case_id, (name, reg)) in cases.iter().enumerate() {
        let expected = baseline_conflict_mask_raw(reg.raw());
        let actual = reg.mask_super();
        let result = shadow_compare_raw(
            ShadowOperation::ConflictMask,
            Some(case_id as u64),
            0,
            expected,
            actual,
            mask_compare_context(expected),
        );
        assert_shadow_ok(result);
        assert_eq!(expected, actual, "conflict mask case {name}");
    }
}

#[test]
fn shadow_known_mask_matches_scalar_baseline() {
    let cases = [
        ("all_n", pattern_all_n()),
        ("all_f", pattern_all_f()),
        ("all_t", pattern_all_t()),
        ("all_s", pattern_all_s()),
        (
            "alternating_known_unknown",
            pattern_alternating_known_unknown(),
        ),
        ("sparse_n", pattern_sparse_n()),
        ("mixed_hand_authored", pattern_mixed_hand_authored()),
    ];

    for (case_id, (name, reg)) in cases.iter().enumerate() {
        let expected = baseline_known_mask_raw(reg.raw());
        let actual = reg.mask_non_null();
        let result = shadow_compare_raw(
            ShadowOperation::KnownMask,
            Some(case_id as u64),
            0,
            expected,
            actual,
            mask_compare_context(expected),
        );
        assert_shadow_ok(result);
        assert_eq!(expected, actual, "known mask case {name}");
    }
}

#[test]
fn shadow_state_delta_matches_scalar_baseline() {
    let cases = [
        ("all_transition_matrix_case", all_transition_matrix_case()),
        ("observation_case", observation_case()),
        ("resolution_case", resolution_case()),
        ("conflict_formation_case", conflict_formation_case()),
        ("mixed_transition_case", mixed_transition_case()),
    ];

    for (case_id, (name, (old_raw, new_raw))) in cases.iter().enumerate() {
        let expected = baseline_delta(*old_raw, *new_raw);
        let result = shadow_compare_delta(Some(case_id as u64), 0, *old_raw, *new_raw);
        assert_shadow_ok(result);

        let old = QuadroReg::from_raw(*old_raw);
        let new = QuadroReg::from_raw(*new_raw);
        let actual = old.calc_delta(new);
        assert_eq!(BaselineDelta::from(actual), expected, "delta case {name}");
    }
}

#[cfg(feature = "alloc")]
#[test]
fn shadow_batch_merge_matches_scalar_baseline() {
    let cases = vec![
        (
            "all_n_vs_all_t",
            vec![
                pattern_all_n(),
                pattern_all_n(),
                pattern_all_n(),
                pattern_all_n(),
            ],
            vec![
                pattern_all_t(),
                pattern_all_t(),
                pattern_all_t(),
                pattern_all_t(),
            ],
        ),
        (
            "all_t_vs_all_f",
            vec![
                pattern_all_t(),
                pattern_all_t(),
                pattern_all_t(),
                pattern_all_t(),
            ],
            vec![
                pattern_all_f(),
                pattern_all_f(),
                pattern_all_f(),
                pattern_all_f(),
            ],
        ),
        (
            "mixed_vs_sparse_conflicts",
            vec![
                pattern_repeating_nfts(),
                pattern_sparse_s(),
                pattern_mixed_hand_authored(),
                pattern_first_lane_conflict(),
            ],
            vec![
                pattern_sparse_s(),
                pattern_all_n(),
                pattern_mixed_transition(),
                pattern_last_lane_conflict(),
            ],
        ),
        (
            "alternating_vs_inverse",
            vec![
                pattern_repeating_tf(),
                pattern_repeating_tf(),
                pattern_repeating_tf(),
                pattern_repeating_tf(),
            ],
            vec![
                pattern_repeating_ft(),
                pattern_repeating_ft(),
                pattern_repeating_ft(),
                pattern_repeating_ft(),
            ],
        ),
        (
            "mixed_vs_mixed",
            vec![
                pattern_mixed_hand_authored(),
                pattern_sparse_n(),
                pattern_all_s(),
                pattern_all_f(),
            ],
            vec![
                pattern_mixed_transition(),
                pattern_sparse_s(),
                pattern_all_n(),
                pattern_all_t(),
            ],
        ),
    ];

    for (case_id, (name, lhs_regs, rhs_regs)) in cases.into_iter().enumerate() {
        let expected_raws: Vec<_> = lhs_regs
            .iter()
            .copied()
            .zip(rhs_regs.iter().copied())
            .map(|(lhs, rhs)| baseline_merge_raw(lhs.raw(), rhs.raw()))
            .collect();

        let mut lhs_bank = QuadroBank::from_regs(lhs_regs.clone());
        let rhs_bank = QuadroBank::from_regs(rhs_regs.clone());
        lhs_bank.merge_inplace(&rhs_bank).unwrap();

        for (index, ((lhs_raw, rhs_raw), expected)) in lhs_regs
            .iter()
            .copied()
            .zip(rhs_regs.iter().copied())
            .zip(expected_raws.iter().copied())
            .enumerate()
        {
            let actual = lhs_bank.as_slice()[index];
            let result = shadow_compare_raw(
                ShadowOperation::BatchMerge,
                Some(case_id as u64),
                index,
                expected,
                actual.raw(),
                ShadowContext {
                    lhs_raw: Some(lhs_raw.raw()),
                    rhs_raw: Some(rhs_raw.raw()),
                    baseline_detail: Some(expected),
                    pulsar_detail: Some(actual.raw()),
                    ..ShadowContext::default()
                },
            );
            assert_shadow_ok(result);
        }

        for (index, expected) in expected_raws.iter().copied().enumerate() {
            assert_eq!(
                lhs_bank.as_slice()[index].raw(),
                expected,
                "batch merge case {name}"
            );
        }
    }
}

#[cfg(feature = "alloc")]
#[test]
fn shadow_batch_intersect_matches_scalar_baseline() {
    let cases = vec![
        (
            "all_s_vs_all_t",
            vec![
                pattern_all_s(),
                pattern_all_s(),
                pattern_all_s(),
                pattern_all_s(),
            ],
            vec![
                pattern_all_t(),
                pattern_all_t(),
                pattern_all_t(),
                pattern_all_t(),
            ],
        ),
        (
            "all_s_vs_all_f",
            vec![
                pattern_all_s(),
                pattern_all_s(),
                pattern_all_s(),
                pattern_all_s(),
            ],
            vec![
                pattern_all_f(),
                pattern_all_f(),
                pattern_all_f(),
                pattern_all_f(),
            ],
        ),
        (
            "all_t_vs_all_f",
            vec![
                pattern_all_t(),
                pattern_all_t(),
                pattern_all_t(),
                pattern_all_t(),
            ],
            vec![
                pattern_all_f(),
                pattern_all_f(),
                pattern_all_f(),
                pattern_all_f(),
            ],
        ),
        (
            "mixed_vs_all_s",
            vec![
                pattern_repeating_nfts(),
                pattern_sparse_s(),
                pattern_mixed_hand_authored(),
                pattern_last_lane_conflict(),
            ],
            vec![
                pattern_all_s(),
                pattern_all_s(),
                pattern_all_s(),
                pattern_all_s(),
            ],
        ),
        (
            "mixed_vs_all_n",
            vec![
                pattern_repeating_tf(),
                pattern_sparse_s(),
                pattern_mixed_hand_authored(),
                pattern_first_lane_conflict(),
            ],
            vec![
                pattern_all_n(),
                pattern_all_n(),
                pattern_all_n(),
                pattern_all_n(),
            ],
        ),
        (
            "mixed_vs_mixed",
            vec![
                pattern_mixed_hand_authored(),
                pattern_sparse_n(),
                pattern_all_s(),
                pattern_all_t(),
            ],
            vec![
                pattern_mixed_transition(),
                pattern_sparse_s(),
                pattern_all_n(),
                pattern_all_f(),
            ],
        ),
    ];

    for (case_id, (name, lhs_regs, rhs_regs)) in cases.into_iter().enumerate() {
        let expected_raws: Vec<_> = lhs_regs
            .iter()
            .copied()
            .zip(rhs_regs.iter().copied())
            .map(|(lhs, rhs)| baseline_intersect_raw(lhs.raw(), rhs.raw()))
            .collect();

        let mut lhs_bank = QuadroBank::from_regs(lhs_regs.clone());
        let rhs_bank = QuadroBank::from_regs(rhs_regs.clone());
        lhs_bank.intersect_inplace(&rhs_bank).unwrap();

        for (index, ((lhs_raw, rhs_raw), expected)) in lhs_regs
            .iter()
            .copied()
            .zip(rhs_regs.iter().copied())
            .zip(expected_raws.iter().copied())
            .enumerate()
        {
            let actual = lhs_bank.as_slice()[index];
            let result = shadow_compare_raw(
                ShadowOperation::BatchIntersect,
                Some(case_id as u64),
                index,
                expected,
                actual.raw(),
                ShadowContext {
                    lhs_raw: Some(lhs_raw.raw()),
                    rhs_raw: Some(rhs_raw.raw()),
                    baseline_detail: Some(expected),
                    pulsar_detail: Some(actual.raw()),
                    ..ShadowContext::default()
                },
            );
            assert_shadow_ok(result);
        }

        for (index, expected) in expected_raws.iter().copied().enumerate() {
            assert_eq!(
                lhs_bank.as_slice()[index].raw(),
                expected,
                "batch intersect case {name}"
            );
        }
    }
}

#[test]
fn shadow_sweep_all_operations_match_scalar_baselines() {
    let mut checks = 0usize;

    for (seed_index, seed) in SHADOW_SWEEP_SEEDS.iter().copied().enumerate() {
        let mut fuzzer = ShadowFuzzer::new(seed);

        for iteration in 0..SHADOW_SWEEP_ITERS_PER_SEED {
            let lhs_raw = fuzzer.next_u64();
            let rhs_raw = fuzzer.next_u64();
            let case_id = shadow_sweep_case_id(seed_index, iteration);
            let context = ShadowContext {
                sweep_seed: Some(seed),
                sweep_iteration: Some(iteration),
                ..ShadowContext::default()
            };

            let lhs = QuadroReg::from_raw(lhs_raw);
            let rhs = QuadroReg::from_raw(rhs_raw);

            assert_shadow_ok(shadow_compare_raw(
                ShadowOperation::ConflictMask,
                Some(case_id),
                iteration,
                baseline_conflict_mask_raw(lhs_raw),
                lhs.mask_super(),
                ShadowContext {
                    expected_mask: Some(baseline_conflict_mask_raw(lhs_raw)),
                    actual_mask: Some(lhs.mask_super()),
                    ..context
                },
            ));

            assert_shadow_ok(shadow_compare_raw(
                ShadowOperation::KnownMask,
                Some(case_id),
                iteration,
                baseline_known_mask_raw(lhs_raw),
                lhs.mask_non_null(),
                ShadowContext {
                    expected_mask: Some(baseline_known_mask_raw(lhs_raw)),
                    actual_mask: Some(lhs.mask_non_null()),
                    ..context
                },
            ));

            let delta = lhs.calc_delta(rhs);
            assert_shadow_ok(shadow_compare_delta_actual(
                Some(case_id),
                iteration,
                lhs_raw,
                rhs_raw,
                delta,
                context,
            ));

            let expected_merge = baseline_merge_raw(lhs_raw, rhs_raw);
            let actual_merge = lhs.merge(rhs).raw();
            assert_shadow_ok(shadow_compare_raw(
                ShadowOperation::BatchMerge,
                Some(case_id),
                iteration,
                expected_merge,
                actual_merge,
                ShadowContext {
                    lhs_raw: Some(lhs_raw),
                    rhs_raw: Some(rhs_raw),
                    baseline_detail: Some(expected_merge),
                    pulsar_detail: Some(actual_merge),
                    ..context
                },
            ));

            let expected_intersect = baseline_intersect_raw(lhs_raw, rhs_raw);
            let actual_intersect = lhs.intersect(rhs).raw();
            assert_shadow_ok(shadow_compare_raw(
                ShadowOperation::BatchIntersect,
                Some(case_id),
                iteration,
                expected_intersect,
                actual_intersect,
                ShadowContext {
                    lhs_raw: Some(lhs_raw),
                    rhs_raw: Some(rhs_raw),
                    baseline_detail: Some(expected_intersect),
                    pulsar_detail: Some(actual_intersect),
                    ..context
                },
            ));

            checks += 5;
        }
    }

    println!(
        "shadow sweep ok: seeds={} iters={} checks={checks}",
        SHADOW_SWEEP_SEEDS.len(),
        SHADOW_SWEEP_ITERS_PER_SEED
    );
}

#[test]
fn shadow_mismatch_report_captures_first_lane_and_inputs() {
    let lhs = pattern_first_lane_conflict();
    let rhs = pattern_last_lane_conflict();

    let report = shadow_compare_raw(
        ShadowOperation::ConflictMask,
        Some(0x2026_0629),
        0,
        lhs.raw(),
        rhs.raw(),
        raw_compare_context(lhs.raw(), rhs.raw()),
    )
    .unwrap_err();

    assert_eq!(report.operation, ShadowOperation::ConflictMask);
    assert_eq!(report.case_id, Some(0x2026_0629));
    assert_eq!(report.quadit_index, Some(0));
    assert_eq!(report.lhs_raw, Some(lhs.raw()));
    assert_eq!(report.rhs_raw, Some(rhs.raw()));
}

#[test]
fn shadow_delta_mismatch_report_captures_changed_lane_and_inputs() {
    let (old_raw, new_raw) = mixed_transition_case();
    let old = QuadroReg::from_raw(old_raw);
    let expected_delta = baseline_delta(old_raw, new_raw);
    let wrong_delta = old.calc_delta(pattern_all_f());
    let (delta_field, expected_mask, actual_mask, lane, old_state, new_state) =
        first_differing_delta_field(expected_delta, wrong_delta, old_raw, new_raw).unwrap();

    let report = shadow_compare_delta_actual(
        Some(0x2026_0629),
        7,
        old_raw,
        new_raw,
        wrong_delta,
        ShadowContext::default(),
    )
    .unwrap_err();

    assert_eq!(report.operation, ShadowOperation::StateDelta);
    assert_eq!(report.case_id, Some(0x2026_0629));
    assert_eq!(report.register_index, 7);
    assert_eq!(report.quadit_index, Some(lane));
    assert_eq!(report.old_raw, Some(old_raw));
    assert_eq!(report.new_raw, Some(new_raw));
    assert_eq!(report.delta_field, Some(delta_field));
    assert_eq!(report.old_state, Some(old_state));
    assert_eq!(report.new_state, Some(new_state));
    assert_eq!(report.expected_mask, Some(expected_mask));
    assert_eq!(report.actual_mask, Some(actual_mask));
    assert_eq!(baseline_delta(old_raw, new_raw), expected_delta);
}
