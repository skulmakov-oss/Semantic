use core::fmt;

use ton618_core::quadro::{QuadState, QuadroReg};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShadowOperation {
    ConflictMask,
    KnownMask,
    StateDelta,
    BatchMerge,
    BatchIntersect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShadowMismatchReport {
    operation: ShadowOperation,
    case_id: Option<u64>,
    register_index: usize,
    quadit_index: Option<u8>,
    baseline_raw: u64,
    pulsar_raw: u64,
    baseline_detail: Option<u64>,
    pulsar_detail: Option<u64>,
}

impl fmt::Display for ShadowMismatchReport {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:?} case={:?} reg={} quadit={:?} baseline=0x{:016x} pulsar=0x{:016x}",
            self.operation,
            self.case_id,
            self.register_index,
            self.quadit_index,
            self.baseline_raw,
            self.pulsar_raw,
        )
    }
}

fn first_differing_quadit_index(baseline_raw: u64, pulsar_raw: u64) -> Option<u8> {
    let diff = baseline_raw ^ pulsar_raw;
    if diff == 0 {
        return None;
    }

    for index in 0..32 {
        if ((diff >> (index * 2)) & 0b11) != 0 {
            return Some(index as u8);
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
) -> Result<(), ShadowMismatchReport> {
    if baseline_raw == pulsar_raw {
        return Ok(());
    }

    Err(ShadowMismatchReport {
        operation,
        case_id,
        register_index,
        quadit_index: first_differing_quadit_index(baseline_raw, pulsar_raw),
        baseline_raw,
        pulsar_raw,
        baseline_detail: Some(baseline_raw),
        pulsar_detail: Some(pulsar_raw),
    })
}

fn shadow_assert_register_eq(
    operation: ShadowOperation,
    case_id: Option<u64>,
    register_index: usize,
    baseline: QuadroReg,
    pulsar: QuadroReg,
) -> Result<(), ShadowMismatchReport> {
    shadow_compare_raw(
        operation,
        case_id,
        register_index,
        baseline.raw(),
        pulsar.raw(),
    )
}

fn scalar_checked_baseline() -> QuadroReg {
    let mut reg = QuadroReg::new();
    reg.try_set(0, QuadState::F.bits()).unwrap();
    reg.try_set(2, QuadState::T.bits()).unwrap();
    reg.try_set(7, QuadState::S.bits()).unwrap();
    reg.try_set(12, QuadState::F.bits()).unwrap();
    reg.try_set(31, QuadState::T.bits()).unwrap();
    reg
}

fn lane_state_reg(index: usize, state: QuadState) -> QuadroReg {
    let mut reg = QuadroReg::new();
    reg.try_set(index, state.bits()).unwrap();
    reg
}

#[test]
fn identical_raw_registers_pass_shadow_comparison() {
    let baseline = scalar_checked_baseline();
    let pulsar = QuadroReg::from_raw(baseline.raw());

    shadow_assert_register_eq(
        ShadowOperation::ConflictMask,
        Some(0x2026_0629),
        0,
        baseline,
        pulsar,
    )
    .unwrap();
}

#[test]
fn mismatched_raw_registers_report_first_differing_quadit() {
    let baseline = lane_state_reg(2, QuadState::T);
    let pulsar = lane_state_reg(2, QuadState::F);

    let report = shadow_assert_register_eq(
        ShadowOperation::KnownMask,
        Some(0x2026_0629),
        3,
        baseline,
        pulsar,
    )
    .unwrap_err();

    assert_eq!(report.operation, ShadowOperation::KnownMask);
    assert_eq!(report.case_id, Some(0x2026_0629));
    assert_eq!(report.register_index, 3);
    assert_eq!(report.quadit_index, Some(2));
    assert_eq!(report.baseline_raw, baseline.raw());
    assert_eq!(report.pulsar_raw, pulsar.raw());
    assert_eq!(report.baseline_detail, Some(report.baseline_raw));
    assert_eq!(report.pulsar_detail, Some(report.pulsar_raw));
    assert!(!report.to_string().is_empty());
}

#[test]
fn raw_compare_helper_preserves_structured_failure_shape() {
    let baseline = QuadroReg::from_raw(0x0000_0000_0000_00f0);
    let pulsar = QuadroReg::from_raw(0x0000_0000_0000_00f4);

    let report = shadow_compare_raw(
        ShadowOperation::StateDelta,
        None,
        11,
        baseline.raw(),
        pulsar.raw(),
    )
    .unwrap_err();

    assert_eq!(report.operation, ShadowOperation::StateDelta);
    assert_eq!(report.case_id, None);
    assert_eq!(report.register_index, 11);
    assert_eq!(report.quadit_index, Some(1));
    assert_eq!(report.baseline_raw, baseline.raw());
    assert_eq!(report.pulsar_raw, pulsar.raw());
}

#[test]
fn shadow_example_checked_api_matches_packed_constructor_path() {
    let baseline = scalar_checked_baseline();
    let pulsar = QuadroReg::from_raw(baseline.raw());

    shadow_assert_register_eq(
        ShadowOperation::BatchMerge,
        Some(0x2026_0629),
        0,
        baseline,
        pulsar,
    )
    .unwrap();
}

#[test]
fn batch_intersect_label_is_available_for_future_targets() {
    let baseline = QuadroReg::from_raw(0x0000_0000_0000_000f);
    let pulsar = QuadroReg::from_raw(0x0000_0000_0000_000f);

    shadow_assert_register_eq(
        ShadowOperation::BatchIntersect,
        Some(0x2026_0629),
        2,
        baseline,
        pulsar,
    )
    .unwrap();
}
