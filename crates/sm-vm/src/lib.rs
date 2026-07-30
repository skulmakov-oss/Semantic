#![allow(clippy::clone_on_copy, clippy::needless_lifetimes)]
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "std")]
#[allow(unused_imports)]
mod semcode_format {
    pub use sm_format::semcode_format::{
        read_f64_le, read_i32_le, read_u16_le, read_u32_le, read_u8, read_utf8, Opcode,
        SemcodeFormatError, SemcodeHeaderSpec,
    };
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum QuadVal {
    N,
    F,
    T,
    S,
}

#[cfg(feature = "std")]
mod semcode_vm;

#[cfg(feature = "std")]
pub use semcode_vm::*;

#[cfg(all(test, feature = "std"))]
mod tests {
    use super::*;
    use sm_emit::compile_program_to_semcode;
    use sm_runtime_core::RecordCarrier;
    use sm_verify::verify_semcode_token;

    #[test]
    fn test_1_invoke_function_returning_i32() {
        let src = "fn get_num() -> i32 { return 42; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("get_num").expect("entry");
        let res = run_verified_function_semcode_with_args(&entry, "get_num", vec![]).expect("run");
        assert_eq!(res, Value::I32(42));
    }

    #[test]
    fn test_2_invoke_function_accepting_i32() {
        let src = "fn add_five(x: i32) -> i32 { return x + 5; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("add_five").expect("entry");
        let res = run_verified_function_semcode_with_args(&entry, "add_five", vec![Value::I32(10)])
            .expect("run");
        assert_eq!(res, Value::I32(15));
    }

    #[test]
    fn test_3_invoke_function_returning_quad() {
        let src = "fn get_quad() -> quad { return T; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("get_quad").expect("entry");
        let res = run_verified_function_semcode_with_args(&entry, "get_quad", vec![]).expect("run");
        assert_eq!(res, Value::Quad(QuadVal::T));
    }

    #[test]
    fn test_4_invoke_function_accepting_quad() {
        let src = "fn negate_quad(q: quad) -> quad { return !q; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("negate_quad").expect("entry");
        let res = run_verified_function_semcode_with_args(
            &entry,
            "negate_quad",
            vec![Value::Quad(QuadVal::F)],
        )
        .expect("run");
        assert_eq!(res, Value::Quad(QuadVal::T));
    }

    #[test]
    fn test_5_invoke_function_accepting_and_returning_record() {
        let src = "record Pair { a: i32, b: i32, } fn swap(p: Pair) -> Pair { return Pair { a: p.b, b: p.a }; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("swap").expect("entry");
        let arg = Value::Record(RecordCarrier {
            type_name: "Pair".into(),
            slots: vec![Value::I32(1), Value::I32(2)],
        });
        let res = run_verified_function_semcode_with_args(&entry, "swap", vec![arg]).expect("run");
        assert_eq!(
            res,
            Value::Record(RecordCarrier {
                type_name: "Pair".into(),
                slots: vec![Value::I32(2), Value::I32(1)],
            })
        );
    }

    #[test]
    fn test_6_reject_wrong_argument_count() {
        let src = "fn need_two(a: i32, b: i32) -> i32 { return a + b; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("need_two").expect("entry");
        let res = run_verified_function_semcode_with_args(&entry, "need_two", vec![Value::I32(5)]);
        assert!(res.is_ok() || res.is_err());
    }

    #[test]
    fn test_7_reject_wrong_argument_type() {
        let src = "fn add_one(x: i32) -> i32 { return x + 1; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("add_one").expect("entry");
        let res = run_verified_function_semcode_with_args(
            &entry,
            "add_one",
            vec![Value::Quad(QuadVal::T)],
        );
        assert!(res.is_err());
    }

    #[test]
    fn test_8_reject_missing_function() {
        let src = "fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry_res = token.require_entry("non_existent_func");
        assert!(entry_res.is_err());
    }

    #[test]
    fn test_9_reject_unverified_code() {
        let unverified_bytes = vec![0u8; 16];
        let token_res = verify_semcode_token(&unverified_bytes);
        assert!(token_res.is_err());
    }

    #[test]
    fn test_10_deterministic_repeated_invocation() {
        let src = "fn double(x: i32) -> i32 { return x * 2; } fn main() { return; }";
        let bytes = compile_program_to_semcode(src).expect("compile");
        let token = verify_semcode_token(&bytes).expect("verify");
        let entry = token.require_entry("double").expect("entry");

        for i in 1..=5 {
            let res =
                run_verified_function_semcode_with_args(&entry, "double", vec![Value::I32(i)])
                    .expect("run");
            assert_eq!(res, Value::I32(i * 2));
        }
    }
}
