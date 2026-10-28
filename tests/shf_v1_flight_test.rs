use sm_emit::{compile_program_to_semcode_with_options, CompileProfile, OptLevel};
use sm_verify::verify_semcode_token;
use sm_vm::{run_verified_function_semcode_with_args, Value};

const SCANNER: &str = include_str!("fixtures/shf_v1/lexer.sm");

#[test]
fn scanner_reports_byte_offsets_deterministically() {
    for opt in [OptLevel::O0, OptLevel::O1] {
        let compile = || {
            compile_program_to_semcode_with_options(SCANNER, CompileProfile::RustLike, opt)
                .expect("compile")
        };
        let bytes = compile();
        assert_eq!(bytes, compile());
        let verified = verify_semcode_token(&bytes).expect("verify");
        for (name, expected) in [
            ("identifier", 10207),
            ("equals", 30809),
            ("number", 21012),
            ("semicolon", 41213),
            ("eof", 1313),
            ("invalid_utf8_byte", 50405),
            ("invalid_symbol", 50001),
        ] {
            let entry = verified.require_entry(name).expect("entry");
            for _ in 0..2 {
                assert_eq!(
                    run_verified_function_semcode_with_args(&entry, vec![]).expect("run"),
                    Value::U32(expected)
                );
            }
        }
    }
}
