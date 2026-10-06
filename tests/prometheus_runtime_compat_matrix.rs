use semantic_language::prom_abi::AbiValue;
use semantic_language::prom_cap::{CapabilityKind, CapabilityManifest};
use semantic_language::prom_gates::DeterministicGateMock;
use semantic_language::prom_gates::{GateDescriptor, GateId, GateRegistry};
use semantic_language::prom_runtime::GateExecutionSession;
use semantic_language::runtime_core::ExecutionContext;

fn read_text(path: &str) -> String {
    let raw = std::fs::read_to_string(path).unwrap_or_else(|e| panic!("read '{}': {}", path, e));
    normalize_newlines(&raw)
}

fn write_text(path: &str, text: &str) {
    std::fs::write(path, text).unwrap_or_else(|e| panic!("write '{}': {}", path, e))
}

fn normalize_newlines(s: &str) -> String {
    s.replace("\r\n", "\n")
}

fn update_mode() -> bool {
    std::env::var("SM_UPDATE_SNAPSHOTS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

fn assert_snapshot(path: &str, got: &str) {
    if update_mode() {
        write_text(path, got);
        return;
    }
    let expected = normalize_newlines(&read_text(path));
    let got = normalize_newlines(got);
    assert_eq!(expected, got, "snapshot mismatch at {}", path);
}

fn render_compat_matrix() -> String {
    let manifest = CapabilityManifest::gate_surface();
    let metadata = manifest.metadata();

    let mut registry = GateRegistry::new();
    registry
        .register(GateDescriptor::read_only(7, 3, "sensor.alpha"))
        .expect("register read");
    registry
        .register(GateDescriptor::read_write(7, 4, "actuator.beta"))
        .expect("register write");

    let duplicate_message = registry
        .register(GateDescriptor::read_write(7, 4, "actuator.beta.shadow"))
        .expect_err("duplicate registration must fail")
        .message;
    let unknown_read_message = registry
        .validate_read(GateId::new(9, 9))
        .expect_err("unknown gate read must fail")
        .message;
    let readonly_write_message = registry
        .validate_write(GateId::new(7, 3))
        .expect_err("readonly gate write must fail")
        .message;

    let mut binding = DeterministicGateMock::new();
    binding.seed_read(GateId::new(7, 3), AbiValue::I32(11));
    let session = GateExecutionSession::kernel_bound(&registry, &mut binding, &manifest);

    let mut out = String::new();
    out.push_str(&format!(
        "manifest_schema={}\nmanifest_version={:?}\n",
        metadata.schema, metadata.version
    ));
    out.push_str(&format!(
        "capabilities={:?}\n",
        [
            (
                CapabilityKind::GateRead,
                manifest.allows(CapabilityKind::GateRead)
            ),
            (
                CapabilityKind::GateWrite,
                manifest.allows(CapabilityKind::GateWrite)
            ),
            (
                CapabilityKind::PulseEmit,
                manifest.allows(CapabilityKind::PulseEmit)
            ),
        ]
    ));
    out.push_str(&format!(
        "session_context={:?}\nsession_gate_registry_bound={}\nsession_manifest={}@{:?}\n",
        session.descriptor().context,
        session.descriptor().gate_registry_bound,
        session.descriptor().capability_manifest.schema,
        session.descriptor().capability_manifest.version
    ));
    out.push_str(&format!(
        "compat_matrix_duplicate_registration={}\ncompat_matrix_unknown_read={}\ncompat_matrix_readonly_write={}\n",
        duplicate_message, unknown_read_message, readonly_write_message
    ));
    out.push_str(&format!(
        "context_baseline={:?}\n",
        ExecutionContext::KernelBound
    ));
    out
}

#[test]
fn golden_prometheus_runtime_compat_matrix_snapshot() {
    let got = render_compat_matrix();
    assert_snapshot("tests/golden_snapshots/runtime/compat_matrix.txt", &got);
}
