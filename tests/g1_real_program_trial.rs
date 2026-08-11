#[path = "support/executable_bundle_support.rs"]
mod executable_bundle_support;

fn cli_ok(command: &str, rel: &str) {
    let path = executable_bundle_support::repo_path(rel);
    smc_cli::run(vec![command.to_string(), path.clone()])
        .unwrap_or_else(|err| panic!("smc {command} failed for {path}: {err}"));
}

fn cli_err(command: &str, rel: &str) -> String {
    let path = executable_bundle_support::repo_path(rel);
    smc_cli::run(vec![command.to_string(), path.clone()])
        .expect_err(&format!("smc {command} unexpectedly passed for {path}"))
}

#[test]
fn g1_cli_batch_core_checks_and_runs() {
    let rel = "examples/qualification/g1_real_program_trial/cli_batch_core/src/main.sm";
    cli_ok("check", rel);
    cli_ok("run", rel);
}

#[test]
fn g1_rule_state_decision_checks_and_runs() {
    let rel = "examples/qualification/g1_real_program_trial/rule_state_decision/src/main.sm";
    cli_ok("check", rel);
    cli_ok("run", rel);
}

#[test]
fn g1_data_audit_record_iterable_checks_and_runs() {
    let rel = "examples/qualification/g1_real_program_trial/data_audit_record_iterable/src/main.sm";
    cli_ok("check", rel);
    cli_ok("run", rel);
}

#[test]
fn g1_module_helpers_program_checks_and_runs() {
    let rel =
        "examples/qualification/executable_module_entry/wave2_local_helper_import/src/main.sm";
    cli_ok("check", rel);
    cli_ok("run", rel);
}

#[test]
fn g1_selected_import_module_program_checks_and_runs() {
    let rel = "examples/qualification/executable_module_entry/positive_selected_import/src/main.sm";
    cli_ok("check", rel);
    cli_ok("run", rel);
}

#[test]
fn g1_top_level_alias_module_program_remains_out_of_scope() {
    let rel = "examples/qualification/executable_module_entry/negative_alias_import/src/main.sm";
    let check_err = cli_err("check", rel);
    assert!(
        check_err.contains(
            "top-level executable Import admits direct local-path and package-qualified helper modules plus selected local imports"
        )
    );
    let run_err = cli_err("run", rel);
    assert!(
        run_err.contains(
            "top-level executable Import admits direct local-path and package-qualified helper modules plus selected local imports"
        )
    );
}
