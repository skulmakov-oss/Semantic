use std::path::PathBuf;

use semantic_core_capsule::{CoreCapsule, CoreConfig, CoreEnginePolicy};
use semantic_core_exec::{
    CoreFunction, CoreProgram, CoreResultDigest, CoreStatus, CoreValue, Fx, Instr, RegId,
};
use semantic_core_quad::QuadState;
use semantic_core_runtime::{CoreTrap, FunctionId, SymbolId};

const CORE_PROGRAM_FILE_FORMAT_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
struct CoreProgramFile {
    format_version: u32,
    config: Option<CoreConfig>,
    program: CoreProgram,
}

fn golden_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("golden")
        .join(format!("{name}.core.json"))
}

fn load_program_file(path: PathBuf) -> CoreProgramFile {
    let text = std::fs::read_to_string(path).expect("golden file read");
    let file: CoreProgramFile = serde_json::from_str(&text).expect("golden json parse");
    assert_eq!(
        file.format_version, CORE_PROGRAM_FILE_FORMAT_VERSION,
        "unsupported golden format version"
    );
    file
}

#[test]
fn all_golden_programs_pass() {
    let expectations = [
        (
            "quad_join",
            CoreStatus::Returned,
            CoreValue::Quad(QuadState::S),
        ),
        (
            "quad_meet",
            CoreStatus::Returned,
            CoreValue::Quad(QuadState::T),
        ),
        ("quad_known", CoreStatus::Returned, CoreValue::Bool(true)),
        ("bool_branch", CoreStatus::Returned, CoreValue::I32(2)),
        ("i32_arithmetic", CoreStatus::Returned, CoreValue::I32(42)),
        ("u32_arithmetic", CoreStatus::Returned, CoreValue::U32(20)),
        (
            "fx_arithmetic",
            CoreStatus::Returned,
            CoreValue::Fx(Fx::from_raw(163840)),
        ),
        ("call_return", CoreStatus::Returned, CoreValue::I32(42)),
        (
            "fuel_trap",
            CoreStatus::Trapped(CoreTrap::FuelExceeded),
            CoreValue::Unit,
        ),
    ];

    for (name, status, value) in expectations {
        let file = load_program_file(golden_path(name));
        let capsule = CoreCapsule::new(file.config.unwrap_or_default());
        let result = capsule.run(&file.program).expect("golden executes");
        assert_eq!(result.status, status, "unexpected status for {name}");
        assert_eq!(result.return_value, value, "unexpected value for {name}");
    }
}

#[test]
fn same_program_same_digest() {
    let file = load_program_file(golden_path("quad_join"));
    let capsule = CoreCapsule::new(CoreConfig::default());
    let left = capsule.run(&file.program).unwrap();
    let right = capsule.run(&file.program).unwrap();
    assert_eq!(
        CoreResultDigest::from_result(&left),
        CoreResultDigest::from_result(&right)
    );
}

#[test]
fn different_result_different_digest() {
    let capsule = CoreCapsule::new(CoreConfig::default());
    let join = load_program_file(golden_path("quad_join"));
    let meet = load_program_file(golden_path("quad_meet"));
    let left = capsule.run(&join.program).unwrap();
    let right = capsule.run(&meet.program).unwrap();
    assert_ne!(
        CoreResultDigest::from_result(&left),
        CoreResultDigest::from_result(&right)
    );
}

#[test]
fn scalar_auto_same_digest() {
    let file = load_program_file(golden_path("i32_arithmetic"));
    let auto = CoreCapsule::new(CoreConfig::default());
    let scalar = CoreCapsule::new(
        CoreConfig::default().with_engine_policy(CoreEnginePolicy::DeterministicReference),
    );
    let auto_result = auto.run(&file.program).unwrap();
    let scalar_result = scalar.run(&file.program).unwrap();
    assert_eq!(
        CoreResultDigest::from_result(&auto_result),
        CoreResultDigest::from_result(&scalar_result)
    );
}

#[test]
fn differential_qjoin_seeded() {
    differential_binary(
        1000,
        |a, b| a.join(b),
        |dst, lhs, rhs| Instr::QJoin { dst, lhs, rhs },
    );
}

#[test]
fn differential_qmeet_seeded() {
    differential_binary(
        1000,
        |a, b| a.meet(b),
        |dst, lhs, rhs| Instr::QMeet { dst, lhs, rhs },
    );
}

#[test]
fn differential_qnot_seeded() {
    let capsule = CoreCapsule::new(CoreConfig::default());
    let mut seed = 0x00C0_FFEE_u64;
    for _ in 0..1000 {
        let value = random_quad(&mut seed);
        let program = CoreProgram {
            functions: vec![CoreFunction {
                name_id: SymbolId(0),
                regs: 2,
                instrs: vec![
                    Instr::LoadQuad {
                        dst: RegId(0),
                        value,
                    },
                    Instr::QNot {
                        dst: RegId(1),
                        src: RegId(0),
                    },
                    Instr::Ret { src: RegId(1) },
                ],
            }],
            entry: FunctionId(0),
        };
        let result = capsule.run(&program).unwrap();
        assert_eq!(result.return_value, CoreValue::Quad(value.inverse()));
    }
}

#[test]
fn differential_qimpl_seeded() {
    differential_binary(
        1000,
        |a, b| a.inverse().join(b),
        |dst, lhs, rhs| Instr::QImpl { dst, lhs, rhs },
    );
}

fn differential_binary(
    cases: usize,
    direct: impl Fn(QuadState, QuadState) -> QuadState,
    make_instr: impl Fn(RegId, RegId, RegId) -> Instr,
) {
    let capsule = CoreCapsule::new(CoreConfig::default());
    let mut seed = 0x51A1_5173_u64;
    for _ in 0..cases {
        let lhs = random_quad(&mut seed);
        let rhs = random_quad(&mut seed);
        let program = CoreProgram {
            functions: vec![CoreFunction {
                name_id: SymbolId(0),
                regs: 3,
                instrs: vec![
                    Instr::LoadQuad {
                        dst: RegId(0),
                        value: lhs,
                    },
                    Instr::LoadQuad {
                        dst: RegId(1),
                        value: rhs,
                    },
                    make_instr(RegId(2), RegId(0), RegId(1)),
                    Instr::Ret { src: RegId(2) },
                ],
            }],
            entry: FunctionId(0),
        };
        let result = capsule.run(&program).unwrap();
        assert_eq!(result.return_value, CoreValue::Quad(direct(lhs, rhs)));
    }
}

fn random_quad(seed: &mut u64) -> QuadState {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 7;
    *seed ^= *seed << 17;
    QuadState::from_bits((*seed as u8) & 0b11).unwrap()
}
