#![cfg(feature = "vm-profile")]

use sm_emit::{compile_program_to_semcode, Opcode};
use sm_runtime_core::{ExecutionConfig, ExecutionContext};
use sm_verify::verify_semcode_token;
use sm_vm::run_verified_entry_semcode_with_profile;

#[test]
fn vm_opcode_profile_smoke_runs_verified_entry() {
    let src = r#"
        fn main() {
            let a: quad = T;
            let b: quad = S;
            let c = a && b;
            if c == T { return; } else { return; }
        }
    "#;

    let bytes = compile_program_to_semcode(src).expect("compile");
    let token = verify_semcode_token(&bytes).expect("verify");
    let entry_token = token.require_entry("main").expect("entry");
    let config = ExecutionConfig::for_context(ExecutionContext::VerifiedLocal);
    let profile =
        run_verified_entry_semcode_with_profile(&entry_token, config).expect("profile run");

    assert!(profile.total_instructions() > 0);
    assert!(profile.count(Opcode::LoadQ) > 0);
    assert!(profile.count(Opcode::QAnd) > 0);
    assert!(profile.count(Opcode::CmpEq) > 0);
    println!("{}", profile.summary_top_n(12));
}
