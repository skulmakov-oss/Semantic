use semantic_language::frontend::compile_program_to_semcode;

fn compile(source: &str) -> Vec<u8> {
    compile_program_to_semcode(source).expect("compile")
}

fn inferred_variant(src: &str) -> String {
    src.replace("    let energy: f64 = energy3(a, b, c);", "    let energy = energy3(a, b, c);")
        .replace(
            "    let guard_stage: quad = policy_trace_guard(tamper_state, quality_state, operator_override);",
            "    let guard_stage = policy_trace_guard(tamper_state, quality_state, operator_override);",
        )
        .replace(
            "        let fusion_stage: quad = fusion_consensus_state(camera_state, badge_state);",
            "        let fusion_stage = fusion_consensus_state(camera_state, badge_state);",
        )
        .replace(
            "        let result_stage: quad = policy_trace_quality(fusion_stage, quality_state);",
            "        let result_stage = policy_trace_quality(fusion_stage, quality_state);",
        )
        .replace("    let status: quad = suite_result();", "    let status = suite_result();")
}

#[test]
fn quad_surface_inferred_locals_preserve_lowering_on_real_example() {
    let annotated = include_str!("../examples/semantic_policy_overdrive_trace.sm");
    let inferred = inferred_variant(annotated);

    let annotated_bytes = compile(annotated);
    let inferred_bytes = compile(&inferred);
    assert_eq!(
        annotated_bytes, inferred_bytes,
        "quad local inference should not change lowering profile for the semantic_policy_overdrive_trace example"
    );
}
