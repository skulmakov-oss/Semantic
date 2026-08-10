use sm_emit::compile_program_to_semcode;
use sm_runtime_core::RuntimeTrap;
use sm_vm::{run_verified_semcode, RuntimeError};

fn program(body: &str) -> String {
    format!("fn main() {{\n    {body}\n    return;\n}}\n")
}

#[test]
fn assert_true_is_accepted_and_runs() {
    let bytes = compile_program_to_semcode(&program("assert(true);")).expect("compile");
    run_verified_semcode(&bytes).expect("assert(true) must not trap");
}

#[test]
fn assert_false_reaches_deterministic_assertion_trap() {
    let bytes = compile_program_to_semcode(&program("assert(false);")).expect("compile");
    let error = run_verified_semcode(&bytes).expect_err("assert(false) must trap");
    assert!(matches!(
        error,
        RuntimeError::Trap(RuntimeTrap::AssertionFailed)
    ));
}

#[test]
fn assert_with_zero_arguments_is_rejected() {
    let error =
        compile_program_to_semcode(&program("assert();")).expect_err("assert() must reject");
    assert!(
        error.message.contains("assert builtin expects 1 arg"),
        "unexpected error: {}",
        error.message
    );
}

#[test]
fn assert_with_two_positional_arguments_is_rejected() {
    let error = compile_program_to_semcode(&program("assert(true, false);"))
        .expect_err("assert(true, false) must reject");
    assert!(
        error.message.contains("assert builtin expects 1 arg"),
        "unexpected error: {}",
        error.message
    );
}

#[test]
fn assert_with_a_named_argument_is_rejected() {
    // The named-argument call syntax in this language is `name = value` (see
    // sm-front's parse_call_args), not `name: value`. `assert(condition = true)` must
    // be rejected exactly like any other malformed assert call, not silently accepted
    // because it still happens to carry exactly one argument value.
    let error = compile_program_to_semcode(&program("assert(condition = true);"))
        .expect_err("assert(condition = true) must reject");
    assert!(
        error
            .message
            .contains("assert builtin takes exactly one positional argument"),
        "unexpected error: {}",
        error.message
    );
}

#[test]
fn assert_with_a_non_bool_argument_is_rejected() {
    let error =
        compile_program_to_semcode(&program("assert(1);")).expect_err("assert(1) must reject");
    assert!(
        error
            .message
            .contains("assert builtin requires bool condition"),
        "unexpected error: {}",
        error.message
    );
}
