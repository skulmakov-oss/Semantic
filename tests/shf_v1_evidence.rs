use sm_emit::{compile_program_to_semcode_with_options, CompileProfile, OptLevel};
use sm_verify::verify_semcode_token;
use sm_vm::{run_verified_function_semcode_with_args, Value};

#[test]
fn evidence_join_and_quarantine() {
    let source = include_str!("fixtures/shf_v1/evidence.sm");
    for opt in [OptLevel::O0, OptLevel::O1] {
        let compile = || {
            compile_program_to_semcode_with_options(source, CompileProfile::RustLike, opt)
                .expect("compile")
        };
        let bytes = compile();
        assert_eq!(bytes, compile());
        let verified = verify_semcode_token(&bytes).expect("verify");
        let run = |name: &str, args: Vec<Value>| {
            run_verified_function_semcode_with_args(
                &verified.require_entry(name).expect("entry"),
                args,
            )
            .expect("run")
        };
        for a in 0..4u32 {
            for b in 0..4u32 {
                for _ in 0..2 {
                    assert_eq!(
                        run("evidence_join", vec![Value::U32(a), Value::U32(b)]),
                        Value::U32(a | b)
                    );
                }
            }
        }
        for (a, b) in [(4, 0), (0, 4), (u32::MAX, 2), (2, u32::MAX), (4, u32::MAX)] {
            for _ in 0..2 {
                assert_eq!(
                    run("evidence_join", vec![Value::U32(a), Value::U32(b)]),
                    Value::U32(4),
                    "invalid input ({a}, {b})"
                );
            }
        }
        for (which, expected) in [(0, 3), (1, 1), (2, 1), (3, 1), (4, 1), (5, 0)] {
            for _ in 0..2 {
                assert_eq!(
                    run("scenario", vec![Value::U32(which)]),
                    Value::U32(expected)
                );
            }
        }
        assert_eq!(run("quarantine_only", vec![]), Value::U32(0));
        assert_eq!(run("admitted_after_quarantine", vec![]), Value::U32(1));
        // state, membership of sources 10/20 in each plane and quarantine,
        // then exact one-entry plane counts and one/two-entry quarantine counts.
        let golden: [[u32; 11]; 8] = [
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], // N has no provenance
            [1, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0], // F
            [2, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0], // T
            [3, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0], // S carries both planes
            [3, 1, 1, 0, 0, 0, 0, 1, 1, 0, 0], // same-source conflict, repeated
            [3, 1, 0, 0, 1, 0, 0, 1, 1, 0, 0], // reversed source order
            [2, 1, 0, 0, 0, 0, 1, 1, 0, 1, 0], // rejected F is isolated
            [2, 1, 0, 0, 0, 1, 1, 1, 0, 0, 1], // rejection does not erase T
        ];
        for (scenario, fields) in golden.iter().enumerate() {
            for (field, expected) in fields.iter().enumerate() {
                for _ in 0..2 {
                    assert_eq!(
                        run(
                            "probe",
                            vec![Value::U32(scenario as u32), Value::U32(field as u32)]
                        ),
                        Value::U32(*expected),
                        "scenario {scenario}, field {field}"
                    );
                }
            }
        }
    }
}
