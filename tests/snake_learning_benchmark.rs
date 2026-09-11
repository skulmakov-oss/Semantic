use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn repo_path(rel: &str) -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(rel)
        .to_string_lossy()
        .replace('\\', "/")
}

fn cli_ok(args: Vec<String>, context: &str) {
    smc_cli::run(args).unwrap_or_else(|err| panic!("{context} failed: {err}"));
}

use std::sync::atomic::{AtomicUsize, Ordering};

static DIR_COUNTER: AtomicUsize = AtomicUsize::new(0);

fn mk_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "{}_{}_{}_{}",
        prefix,
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos(),
        DIR_COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

/// #1902 (FA-08-012) split of the previously-bundled `check`+`run`+
/// `compile`+`verify` test (`ssf08_1902_snake_learning_envelope_decision.md`,
/// Model E): `check`/`compile`/`verify` never construct an `ExecutionConfig`
/// or execute the VM, so they carry no dependency on any runtime quota
/// envelope. Proves language acceptance and bytecode verification only -
/// see `snake_learning_completes_under_explicit_high_budget_envelope` for
/// the separate full-completion guarantee.
fn check_compile_verify(rel: &str) {
    let input = repo_path(rel);
    cli_ok(
        vec!["check".to_string(), input.clone()],
        &format!("smc check for {input}"),
    );

    let dir = mk_temp_dir("smc_snake_learning_benchmark");
    let out = dir.join("out.smc");
    let out_arg = out.to_string_lossy().replace('\\', "/");
    cli_ok(
        vec![
            "compile".to_string(),
            input.clone(),
            "-o".to_string(),
            out_arg.clone(),
        ],
        &format!("smc compile for {input}"),
    );
    cli_ok(
        vec!["verify".to_string(), out_arg],
        &format!("smc verify for {input}"),
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn snake_learning_passes_check_compile_verify() {
    check_compile_verify("examples/benchmarks/snake_learning.sm");
}

/// #1902 (FA-08-012) / `ssf08_1902_snake_learning_envelope_decision.md`
/// (Model E + C): `smc run`'s default `VerifiedLocal` envelope
/// (`max_steps = 100_000`, `max_calls = 16_384`) is far short of this
/// benchmark's real, measured, deterministic cost (753,864 Steps quota
/// charges / 42,477 Calls quota charges - the decision document's §13).
/// This is a residual workload/envelope mismatch, not an enforcement
/// defect (#1759's Steps/Calls enforcement is correct), so the fix is to
/// give the full-completion guarantee its own dedicated, explicitly-scoped
/// test rather than widen the published default for every program.
///
/// This does not change, and must not be read as changing, `smc run`'s
/// default behavior: `smc run examples/benchmarks/snake_learning.sm`
/// remains fail-closed under the ordinary CLI envelope, exactly as
/// before.
#[test]
fn snake_learning_completes_under_explicit_high_budget_envelope() {
    use sm_emit::compile_program_to_semcode;
    use sm_runtime_core::{ExecutionConfig, ExecutionContext, RuntimeQuotas};
    use sm_verify::verify_semcode_token_with_quotas;
    use sm_vm::run_verified_entry_semcode_with_config;

    let src = std::fs::read_to_string(repo_path("examples/benchmarks/snake_learning.sm"))
        .expect("read snake_learning.sm");
    let bytes = compile_program_to_semcode(&src).expect("compile snake_learning.sm");

    // Explicit, test-local custom envelope (decision §22/§27): only
    // max_steps/max_calls move, with wide measured headroom (~49.7%/
    // ~52.8% over the exact 753,864/42,477 measured cost); every other
    // field stays at VerifiedLocal's published default. One `quotas`
    // value is the sole authority, threaded unchanged into both
    // admission and execution (#1762 single-authority discipline).
    let quotas = RuntimeQuotas {
        max_steps: 1_500_000,
        max_calls: 90_000,
        ..RuntimeQuotas::verified_local()
    };

    let token = verify_semcode_token_with_quotas(&bytes, quotas)
        .expect("snake_learning SemCode must verify under the explicit test envelope");
    let entry = token
        .require_entry("main")
        .expect("snake_learning must expose main");

    // Canonical verified execution path (verify_semcode_token_with_quotas
    // -> require_entry -> run_verified_entry_semcode_with_config) - never
    // the raw/bypass path. Success (`Ok(())`) is the completion proof:
    // the embedded Semantic assertions (`total_score == 8`,
    // `total_steps == 1417`, non-empty Q-table) are load-bearing inside
    // `main` itself, and a failed `assert` would surface as
    // `RuntimeError::Trap(RuntimeTrap::AssertionFailed)`, not `Ok`. This
    // API returns `Result<(), RuntimeError>`, not `main`'s locals, so
    // there is no separate value to assert here.
    run_verified_entry_semcode_with_config(
        &entry,
        ExecutionConfig::new(ExecutionContext::VerifiedLocal, quotas),
    )
    .expect("snake_learning must complete under its explicit test envelope");
}
