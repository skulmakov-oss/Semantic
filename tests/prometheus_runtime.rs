use semantic_language::frontend::{emit_ir_to_semcode, IrFunction, IrInstr};
use semantic_language::prom_abi::{AbiValue, RecordingHostAbi};
use semantic_language::prom_cap::{CapabilityKind, CapabilityManifest};
use semantic_language::prom_gates::{DeterministicGateMock, GateDescriptor, GateId, GateRegistry};
use semantic_language::prom_runtime::{ExecutionSession, GateExecutionSession};
use semantic_language::runtime_core::{ExecutionConfig, ExecutionContext};
use semantic_language::semcode_vm::RuntimeError;

fn runtime_program() -> Vec<IrFunction> {
    vec![IrFunction {
        name: "main".to_string(),
        instrs: vec![
            IrInstr::GateRead {
                dst: 0,
                device_id: 7,
                port: 3,
            },
            IrInstr::GateWrite {
                device_id: 7,
                port: 4,
                src: 0,
            },
            IrInstr::Ret { src: None },
        ],
        ownership_events: Vec::new(),
    }]
}

fn state_query_program() -> Vec<IrFunction> {
    vec![IrFunction {
        name: "main".to_string(),
        instrs: vec![
            IrInstr::StateQuery {
                dst: 0,
                key: "decision.mode".to_string(),
            },
            IrInstr::LoadI32 { dst: 1, val: 123 },
            IrInstr::CmpEq {
                dst: 2,
                lhs: 0,
                rhs: 1,
            },
            IrInstr::Assert { cond: 2 },
            IrInstr::Ret { src: None },
        ],
        ownership_events: Vec::new(),
    }]
}

fn state_update_program() -> Vec<IrFunction> {
    vec![IrFunction {
        name: "main".to_string(),
        instrs: vec![
            IrInstr::LoadBool { dst: 0, val: true },
            IrInstr::StateUpdate {
                key: "decision.mode".to_string(),
                src: 0,
            },
            IrInstr::Ret { src: None },
        ],
        ownership_events: Vec::new(),
    }]
}

fn event_post_program() -> Vec<IrFunction> {
    vec![IrFunction {
        name: "main".to_string(),
        instrs: vec![
            IrInstr::EventPost {
                signal: "alert.raised".to_string(),
            },
            IrInstr::Ret { src: None },
        ],
        ownership_events: Vec::new(),
    }]
}

fn clock_read_program() -> Vec<IrFunction> {
    vec![IrFunction {
        name: "main".to_string(),
        instrs: vec![
            IrInstr::ClockRead { dst: 0 },
            IrInstr::LoadU32 { dst: 1, val: 42 },
            IrInstr::CmpEq {
                dst: 2,
                lhs: 0,
                rhs: 1,
            },
            IrInstr::Assert { cond: 2 },
            IrInstr::Ret { src: None },
        ],
        ownership_events: Vec::new(),
    }]
}

#[test]
fn gate_execution_session_runs_verified_program_with_bound_registry() {
    let bytes = emit_ir_to_semcode(&runtime_program(), false).expect("emit");

    let mut registry = GateRegistry::new();
    registry
        .register(GateDescriptor::read_only(7, 3, "sensor.alpha"))
        .expect("register read gate");
    registry
        .register(GateDescriptor::read_write(7, 4, "actuator.beta"))
        .expect("register write gate");

    let manifest = CapabilityManifest::gate_surface();
    let metadata = manifest.metadata();
    let mut binding = DeterministicGateMock::new();
    binding.seed_read(GateId::new(7, 3), AbiValue::I32(99));

    let mut session =
        GateExecutionSession::kernel_bound(&registry, &mut binding, &manifest, metadata.clone());
    assert_eq!(session.descriptor().context, ExecutionContext::KernelBound);
    assert!(session.descriptor().gate_registry_bound);
    assert_eq!(session.descriptor().capability_manifest, metadata);

    // Intentional byte-shim compatibility coverage: this test protects the public run_verified_semcode* API surface.
    session
        .run_verified_semcode(&bytes)
        .expect("run verified via runtime session");

    drop(session);
    assert_eq!(binding.writes(), &[(GateId::new(7, 4), AbiValue::I32(99))]);
}

#[test]
fn execution_session_runs_state_query_with_generic_host_path() {
    let bytes = emit_ir_to_semcode(&state_query_program(), false).expect("emit");

    let mut manifest = CapabilityManifest::new();
    manifest.allow(CapabilityKind::StateQuery);
    let metadata = manifest.metadata();
    let mut host = RecordingHostAbi::with_state_query_value(AbiValue::I32(123));

    let mut session = ExecutionSession::kernel_bound(&mut host, &manifest, metadata.clone());
    assert_eq!(session.descriptor().context, ExecutionContext::KernelBound);
    assert!(!session.descriptor().gate_registry_bound);
    assert_eq!(session.descriptor().capability_manifest, metadata);

    session
        .run_verified_semcode(&bytes)
        .expect("run verified via generic runtime session");

    drop(session);
    assert_eq!(host.state_queries, vec!["decision.mode".to_string()]);
}

#[test]
fn execution_session_denies_state_query_without_manifest_capability() {
    let bytes = emit_ir_to_semcode(&state_query_program(), false).expect("emit");

    let manifest = CapabilityManifest::new();
    let metadata = manifest.metadata();
    let mut host = RecordingHostAbi::with_state_query_value(AbiValue::I32(123));
    let mut session = ExecutionSession::kernel_bound(&mut host, &manifest, metadata);

    let err = session
        .run_verified_semcode(&bytes)
        .expect_err("state query must require capability");

    match err {
        RuntimeError::CapabilityDenied(denied) => {
            assert_eq!(denied.capability, CapabilityKind::StateQuery);
        }
        other => panic!("expected CapabilityDenied, got {other:?}"),
    }

    drop(session);
    assert!(host.state_queries.is_empty());
}

#[test]
fn execution_session_runs_state_update_with_generic_host_path() {
    let bytes = emit_ir_to_semcode(&state_update_program(), false).expect("emit");

    let mut manifest = CapabilityManifest::new();
    manifest.allow(CapabilityKind::StateUpdate);
    let metadata = manifest.metadata();
    let mut host = RecordingHostAbi::default();

    let mut session = ExecutionSession::kernel_bound(&mut host, &manifest, metadata.clone());
    assert_eq!(session.descriptor().context, ExecutionContext::KernelBound);
    assert!(!session.descriptor().gate_registry_bound);
    assert_eq!(session.descriptor().capability_manifest, metadata);

    session
        .run_verified_semcode(&bytes)
        .expect("run verified via generic runtime session");

    drop(session);
    assert_eq!(
        host.state_updates,
        vec![("decision.mode".to_string(), AbiValue::Bool(true))]
    );
}

#[test]
fn execution_session_denies_state_update_without_manifest_capability() {
    let bytes = emit_ir_to_semcode(&state_update_program(), false).expect("emit");

    let manifest = CapabilityManifest::new();
    let metadata = manifest.metadata();
    let mut host = RecordingHostAbi::default();
    let mut session = ExecutionSession::kernel_bound(&mut host, &manifest, metadata);

    let err = session
        .run_verified_semcode(&bytes)
        .expect_err("state update must require capability");

    match err {
        RuntimeError::CapabilityDenied(denied) => {
            assert_eq!(denied.capability, CapabilityKind::StateUpdate);
        }
        other => panic!("expected CapabilityDenied, got {other:?}"),
    }

    drop(session);
    assert!(host.state_updates.is_empty());
}

#[test]
fn execution_session_runs_event_post_with_generic_host_path() {
    let bytes = emit_ir_to_semcode(&event_post_program(), false).expect("emit");

    let mut manifest = CapabilityManifest::new();
    manifest.allow(CapabilityKind::EventPost);
    let metadata = manifest.metadata();
    let mut host = RecordingHostAbi::default();

    let mut session = ExecutionSession::kernel_bound(&mut host, &manifest, metadata.clone());
    assert_eq!(session.descriptor().context, ExecutionContext::KernelBound);
    assert!(!session.descriptor().gate_registry_bound);
    assert_eq!(session.descriptor().capability_manifest, metadata);

    session
        .run_verified_semcode(&bytes)
        .expect("run verified via generic runtime session");

    drop(session);
    assert_eq!(host.event_posts, vec!["alert.raised".to_string()]);
}

#[test]
fn execution_session_denies_event_post_without_manifest_capability() {
    let bytes = emit_ir_to_semcode(&event_post_program(), false).expect("emit");

    let manifest = CapabilityManifest::new();
    let metadata = manifest.metadata();
    let mut host = RecordingHostAbi::default();
    let mut session = ExecutionSession::kernel_bound(&mut host, &manifest, metadata);

    let err = session
        .run_verified_semcode(&bytes)
        .expect_err("event post must require capability");

    match err {
        RuntimeError::CapabilityDenied(denied) => {
            assert_eq!(denied.capability, CapabilityKind::EventPost);
        }
        other => panic!("expected CapabilityDenied, got {other:?}"),
    }

    drop(session);
    assert!(host.event_posts.is_empty());
}

#[test]
fn execution_session_runs_clock_read_with_generic_host_path() {
    let bytes = emit_ir_to_semcode(&clock_read_program(), false).expect("emit");

    let mut manifest = CapabilityManifest::new();
    manifest.allow(CapabilityKind::ClockRead);
    let metadata = manifest.metadata();
    let mut host = RecordingHostAbi::with_clock_read_value(42);

    let mut session = ExecutionSession::kernel_bound(&mut host, &manifest, metadata.clone());
    assert_eq!(session.descriptor().context, ExecutionContext::KernelBound);
    assert!(!session.descriptor().gate_registry_bound);
    assert_eq!(session.descriptor().capability_manifest, metadata);

    session
        .run_verified_semcode(&bytes)
        .expect("run verified via generic runtime session");

    drop(session);
    assert_eq!(host.clock_reads, 1);
}

#[test]
fn execution_session_denies_clock_read_without_manifest_capability() {
    let bytes = emit_ir_to_semcode(&clock_read_program(), false).expect("emit");

    let manifest = CapabilityManifest::new();
    let metadata = manifest.metadata();
    let mut host = RecordingHostAbi::with_clock_read_value(42);
    let mut session = ExecutionSession::kernel_bound(&mut host, &manifest, metadata);

    let err = session
        .run_verified_semcode(&bytes)
        .expect_err("clock read must require capability");

    match err {
        RuntimeError::CapabilityDenied(denied) => {
            assert_eq!(denied.capability, CapabilityKind::ClockRead);
        }
        other => panic!("expected CapabilityDenied, got {other:?}"),
    }

    drop(session);
    assert_eq!(host.clock_reads, 0);
}

// --- #1822 (umbrella #1617) regression matrix -------------------------
//
// `ExecutionSession`/`GateExecutionSession::run_verified_semcode_entry`
// used to admit via the hardcoded-default `verify_semcode_token(bytes)`
// (VerifiedLocal's 4096-register budget) regardless of `self.config`,
// even when the session was explicitly constructed with `kernel_bound()`
// (8192). Built directly via `emit_ir_to_semcode` -- a single `LoadI32`
// into `r5000` (between the two budgets) followed by a `Ret` reading it
// back -- rather than the older raw-byte-patching idiom, for the same
// reason established elsewhere in this campaign: full, unambiguous
// control over the exact register referenced.

fn r5000_program() -> Vec<IrFunction> {
    vec![IrFunction {
        name: "main".to_string(),
        instrs: vec![
            IrInstr::LoadI32 { dst: 5000, val: 1 },
            IrInstr::Ret { src: Some(5000) },
        ],
        ownership_events: Vec::new(),
    }]
}

#[test]
fn execution_session_kernel_bound_admits_r5000() {
    let bytes = emit_ir_to_semcode(&r5000_program(), false).expect("emit");
    let manifest = CapabilityManifest::new();
    let metadata = manifest.metadata();
    let mut host = RecordingHostAbi::default();
    let mut session = ExecutionSession::kernel_bound(&mut host, &manifest, metadata);
    session
        .run_verified_semcode(&bytes)
        .expect("KernelBound session must admit and run an r5000 artifact");
}

#[test]
fn execution_session_verified_local_rejects_r5000() {
    let bytes = emit_ir_to_semcode(&r5000_program(), false).expect("emit");
    let manifest = CapabilityManifest::new();
    let metadata = manifest.metadata();
    let mut host = RecordingHostAbi::default();
    let mut session = ExecutionSession::new(
        &mut host,
        &manifest,
        ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
        metadata,
    );
    let err = session
        .run_verified_semcode(&bytes)
        .expect_err("VerifiedLocal session must still reject an r5000 artifact");
    assert!(
        matches!(err, RuntimeError::VerifierRejected(_)),
        "expected VerifierRejected, got {err:?}"
    );
}

#[test]
fn gate_execution_session_kernel_bound_admits_r5000() {
    let bytes = emit_ir_to_semcode(&r5000_program(), false).expect("emit");
    let registry = GateRegistry::new();
    let manifest = CapabilityManifest::new();
    let metadata = manifest.metadata();
    let mut binding = DeterministicGateMock::new();
    let mut session =
        GateExecutionSession::kernel_bound(&registry, &mut binding, &manifest, metadata);
    session
        .run_verified_semcode(&bytes)
        .expect("KernelBound gate session must admit and run an r5000 artifact");
}

#[test]
fn gate_execution_session_verified_local_rejects_r5000() {
    let bytes = emit_ir_to_semcode(&r5000_program(), false).expect("emit");
    let registry = GateRegistry::new();
    let manifest = CapabilityManifest::new();
    let metadata = manifest.metadata();
    let mut binding = DeterministicGateMock::new();
    let mut session = GateExecutionSession::new(
        &registry,
        &mut binding,
        &manifest,
        ExecutionConfig::for_context(ExecutionContext::VerifiedLocal),
        metadata,
    );
    let err = session
        .run_verified_semcode(&bytes)
        .expect_err("VerifiedLocal gate session must still reject an r5000 artifact");
    assert!(
        matches!(err, RuntimeError::VerifierRejected(_)),
        "expected VerifierRejected, got {err:?}"
    );
}
