use semantic_language::frontend::compile_program_to_semcode;

fn compile(source: &str) -> Vec<u8> {
    compile_program_to_semcode(source).expect("compile")
}

#[test]
fn quad_surface_local_inference_matches_annotated_lowering() {
    let annotated = r#"
fn main() {
    let q: quad = T;
    let value: i32 = if q == T { 1 } else { 2 };
    let branch: i32 = match q {
        N => { 0 }
        F => { 1 }
        T => { 2 }
        S => { 3 }
        _ => { 4 }
    };
    assert(value == 1);
    assert(branch == 2);
    return;
}
"#;
    let inferred = r#"
fn main() {
    let q = T;
    let value: i32 = if q == T { 1 } else { 2 };
    let branch: i32 = match q {
        N => { 0 }
        F => { 1 }
        T => { 2 }
        S => { 3 }
        _ => { 4 }
    };
    assert(value == 1);
    assert(branch == 2);
    return;
}
"#;

    let annotated_bytes = compile(annotated);
    let inferred_bytes = compile(inferred);
    assert_eq!(
        annotated_bytes, inferred_bytes,
        "quad local inference should not change lowering profile"
    );
}
