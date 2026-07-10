#![cfg(feature = "wgpu-backend")]

use prom_ui_backend_native::{join_u128, split_u128, GpuQuadTile128};
use semantic_core_quad::{QuadTile128, QuadroReg32};

#[test]
fn gpu_quad_tile128_layout_is_frozen() {
    assert_eq!(core::mem::size_of::<GpuQuadTile128>(), 32);
    assert_eq!(core::mem::align_of::<GpuQuadTile128>(), 16);
    assert_eq!(core::mem::offset_of!(GpuQuadTile128, t), 0);
    assert_eq!(core::mem::offset_of!(GpuQuadTile128, f), 16);
}

#[test]
fn gpu_quad_tile128_known_word_order() {
    let value = 0xDDEE_FF00_99AA_BBCC_5566_7788_1122_3344u128;
    let expected = [0x1122_3344, 0x5566_7788, 0x99AA_BBCC, 0xDDEE_FF00];

    assert_eq!(split_u128(value), expected);
    assert_eq!(join_u128(expected), value);
}

#[test]
fn gpu_quad_tile128_split_join_roundtrips() {
    let values = [
        0u128,
        u128::MAX,
        1u128,
        1u128 << 127,
        0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210u128,
        0xAAAA_AAAA_AAAA_AAAA_5555_5555_5555_5555u128,
    ];
    for value in values {
        assert_eq!(join_u128(split_u128(value)), value);
    }

    let words = [
        [0, 0, 0, 0],
        [u32::MAX, u32::MAX, u32::MAX, u32::MAX],
        [0x0123_4567, 0x89AB_CDEF, 0xFEDC_BA98, 0x7654_3210],
        [0xAAAA_AAAA, 0x5555_5555, 0x1357_9BDF, 0x2468_ACE0],
    ];
    for words in words {
        assert_eq!(split_u128(join_u128(words)), words);
    }
}

#[test]
fn gpu_quad_tile128_core_plane_roundtrip() {
    let truth = 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210u128;
    let falsity = 0xAAAA_AAAA_AAAA_AAAA_5555_5555_5555_5555u128;
    let original = QuadTile128::from_planes(truth, falsity);
    let transport = GpuQuadTile128::from(original);
    let roundtrip = QuadTile128::from(transport);

    assert_eq!(original, roundtrip);
    assert_eq!(roundtrip.true_plane(), truth);
    assert_eq!(roundtrip.false_plane(), falsity);
}

#[test]
fn gpu_quad_tile128_fields_match_core_planes() {
    let truth = 0x0123_4567_89AB_CDEF_FEDC_BA98_7654_3210u128;
    let falsity = 0x1357_9BDF_2468_ACE0_0F0F_F0F0_55AA_AA55u128;
    let core = QuadTile128::from_planes(truth, falsity);
    let gpu = GpuQuadTile128::from(core);

    assert_eq!(gpu.t, split_u128(core.true_plane()));
    assert_eq!(gpu.f, split_u128(core.false_plane()));
}

#[test]
fn gpu_quad_tile128_from_regs_roundtrip_preserves_lanes() {
    let regs = [
        QuadroReg32::from_raw(0x0123_4567_89AB_CDEF),
        QuadroReg32::from_raw(0xFEDC_BA98_7654_3210),
        QuadroReg32::from_raw(0x5555_AAAA_3333_CCCC),
        QuadroReg32::from_raw(0xBADC_0FFE_DEAD_BEEF),
    ];
    let original = QuadTile128::from_regs(regs);
    let transport = GpuQuadTile128::from(original);
    let roundtrip = QuadTile128::from(transport);

    assert_eq!(roundtrip.to_regs(), regs);
}

#[test]
fn gpu_quad_tile128_default_is_zero_transport() {
    let transport = GpuQuadTile128::default();

    assert_eq!(transport.t, [0; 4]);
    assert_eq!(transport.f, [0; 4]);
    assert_eq!(QuadTile128::from(transport), QuadTile128::new());
}
