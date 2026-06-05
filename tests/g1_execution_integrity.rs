use std::{fs, path::PathBuf};

#[path = "support/executable_bundle_support.rs"]
mod executable_bundle_support;

use semantic_language::{
    frontend::{compile_program_to_ir, compile_program_to_semcode},
    semantics::check_source,
    semcode_format::header_spec_from_magic,
    semcode_verify::{verify_semcode, VerificationCode},
    semcode_vm::{disasm_semcode, run_semcode},
};

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn source_text(rel: &str) -> String {
    let path = repo_path(rel);
    fs::read_to_string(&path).unwrap_or_else(|err| panic!("read failed for {path}: {err}"))
}

fn label_for(rel: &str) -> &str {
    rel.split('/').rev().nth(2).expect("fixture folder name")
}

fn csv(values: &[String]) -> String {
    values.join(",")
}

fn normalize_selected_import_name(name: &str) -> String {
    if let Some(rest) = name.strip_prefix("execsel_") {
        if let Some((hash, suffix)) = rest.split_once('_') {
            if hash.len() == 16 && hash.chars().all(|ch| ch.is_ascii_hexdigit()) {
                return format!("execsel_<stable>_{suffix}");
            }
        }
    }
    name.to_string()
}

fn disasm_function_names(disasm: &str) -> Vec<String> {
    disasm
        .lines()
        .filter_map(|line| {
            line.strip_prefix("fn ")
                .and_then(|rest| rest.split(": code=").next())
                .map(normalize_selected_import_name)
        })
        .collect()
}

fn execution_summary(rel: &str) -> String {
    let src = executable_bundle_support::bundle_source(rel);
    let sema = check_source(&src).expect("semantic check");
    let ir = compile_program_to_ir(&src).expect("compile ir");
    let bytes = compile_program_to_semcode(&src).expect("compile semcode");
    let token = sm_verify::verify_semcode_token(&bytes).expect("verify");
    let verified = token.program().clone();
    let disasm = disasm_semcode(&bytes).expect("disasm");
    let entry_token = token.require_entry("main").expect("require main");
    sm_vm::run_verified_entry_semcode(&entry_token).expect("verified run");

    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[..8]);
    let header = header_spec_from_magic(&magic).expect("known header");

    let mut ir_names: Vec<String> = ir
        .iter()
        .map(|func| normalize_selected_import_name(&func.name))
        .collect();
    let mut verified_names: Vec<String> = verified
        .functions
        .iter()
        .map(|func| normalize_selected_import_name(&func.name))
        .collect();
    let mut disasm_names = disasm_function_names(&disasm);

    ir_names.sort();
    verified_names.sort();
    disasm_names.sort();

    assert_eq!(
        ir_names, verified_names,
        "IR and verifier surface drifted for {rel}"
    );
    assert_eq!(
        verified_names, disasm_names,
        "verifier and disasm surface drifted for {rel}"
    );

    format!(
        "program={}\nsema:warnings={} laws={}\nir:names={}\nsemcode:magic={} rev={}\nverify:names={}\ndisasm:names={}\nrun=ok\n",
        label_for(rel),
        sema.warnings.len(),
        sema.scheduled_laws.len(),
        csv(&ir_names),
        String::from_utf8_lossy(&magic),
        header.rev,
        csv(&verified_names),
        csv(&disasm_names),
    )
}

fn first_function_code_offset(bytes: &[u8]) -> usize {
    let name_len = u16::from_le_bytes([bytes[8], bytes[9]]) as usize;
    8 + 2 + name_len + 4
}

#[test]
fn g1_execution_integrity_stage_summaries_match_current_baseline() {
    let mut observed = String::new();
    for rel in [
        "examples/qualification/g1_real_program_trial/cli_batch_core/src/main.sm",
        "examples/qualification/g1_real_program_trial/rule_state_decision/src/main.sm",
        "examples/qualification/g1_real_program_trial/data_audit_record_iterable/src/main.sm",
        "examples/qualification/executable_module_entry/wave2_local_helper_import/src/main.sm",
        "examples/qualification/executable_module_entry/positive_selected_import/src/main.sm",
    ] {
        observed.push_str(&execution_summary(rel));
        observed.push('\n');
    }

    let expected = "\
program=cli_batch_core
sema:warnings=0 laws=0
ir:names=classify_exit,main
semcode:magic=SEMCOD13 rev=14
verify:names=classify_exit,main
disasm:names=classify_exit,main
run=ok

program=rule_state_decision
sema:warnings=0 laws=0
ir:names=decide,main
semcode:magic=SEMCODE0 rev=1
verify:names=decide,main
disasm:names=decide,main
run=ok

program=data_audit_record_iterable
sema:warnings=0 laws=0
ir:names=__impl::Iterable::Samples::next,main,summarize
semcode:magic=SEMCOD12 rev=13
verify:names=__impl::Iterable::Samples::next,main,summarize
disasm:names=__impl::Iterable::Samples::next,main,summarize
run=ok

program=wave2_local_helper_import
sema:warnings=0 laws=0
ir:names=main,score
semcode:magic=SEMCODE0 rev=1
verify:names=main,score
disasm:names=main,score
run=ok

program=positive_selected_import
sema:warnings=0 laws=0
ir:names=execsel_<stable>_scale,execsel_<stable>_score,main,score
semcode:magic=SEMCODE0 rev=1
verify:names=execsel_<stable>_scale,execsel_<stable>_score,main,score
disasm:names=execsel_<stable>_scale,execsel_<stable>_score,main,score
run=ok

";
    assert_eq!(observed, expected);
}

#[test]
fn g1_execution_integrity_repeated_compiles_are_byte_stable() {
    for rel in [
        "examples/qualification/g1_real_program_trial/cli_batch_core/src/main.sm",
        "examples/qualification/g1_real_program_trial/rule_state_decision/src/main.sm",
        "examples/qualification/g1_real_program_trial/data_audit_record_iterable/src/main.sm",
        "examples/qualification/executable_module_entry/wave2_local_helper_import/src/main.sm",
        "examples/qualification/executable_module_entry/positive_selected_import/src/main.sm",
    ] {
        let src = executable_bundle_support::bundle_source(rel);

        let first = compile_program_to_semcode(&src).expect("first compile");
        let second = compile_program_to_semcode(&src).expect("second compile");
        let third = compile_program_to_semcode(&src).expect("third compile");
        assert_eq!(first, second, "second compile drifted for {rel}");
        assert_eq!(second, third, "third compile drifted for {rel}");

        let disasm_first = disasm_semcode(&first).expect("first disasm");
        let disasm_second = disasm_semcode(&second).expect("second disasm");
        assert_eq!(disasm_first, disasm_second, "disasm drifted for {rel}");

        let token = sm_verify::verify_semcode_token(&first).expect("verify");
        let entry_token = token.require_entry("main").expect("require main");
        for _ in 0..3 {
            sm_vm::run_verified_entry_semcode(&entry_token)
                .expect("verified run must stay successful");
        }
    }
}

#[test]
fn g1_execution_integrity_malformed_semcode_rejects_before_execution() {
    let src =
        source_text("examples/qualification/g1_real_program_trial/rule_state_decision/src/main.sm");
    let mut bytes = compile_program_to_semcode(&src).expect("compile");

    let code_offset = first_function_code_offset(&bytes);
    bytes[code_offset] = 0xff;

    let reject = verify_semcode(&bytes).expect_err("malformed semcode must reject");
    assert!(
        reject
            .diagnostics
            .iter()
            .any(|diag| diag.code == VerificationCode::InvalidStringTable),
        "expected InvalidStringTable in verifier rejection, got: {reject:?}"
    );

    let runtime_err =
        run_semcode(&bytes).expect_err("raw runtime path must reject malformed semcode");
    let rendered = format!("{runtime_err}");
    assert!(
        rendered.contains("bad SemCode format") || rendered.contains("string"),
        "unexpected runtime rejection: {rendered}"
    );
}
