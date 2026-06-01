use sm_emit::compile_program_to_semcode;
use sm_ir::semcode_format::{
    read_u16_le, read_u32_le, read_u8, read_utf8, MAGIC11, MAGIC12, OWNERSHIP_EVENT_KIND_BORROW,
    OWNERSHIP_EVENT_KIND_WRITE, OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL,
    OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX, OWNERSHIP_SECTION_TAG,
};
use sm_runtime_core::RuntimeTrap;
use sm_verify::verify_semcode;
use sm_vm::{run_verified_semcode, RuntimeError};

#[derive(Clone, Copy)]
enum OwnershipPathComponentSpec {
    TupleIndex(u16),
    FieldSymbol(u32),
}

#[derive(Clone, Copy)]
struct OwnershipEventSpec<'a> {
    kind: u8,
    root: &'a str,
    components: &'a [OwnershipPathComponentSpec],
}

struct FunctionLayout {
    strings: Vec<String>,
    ownership_start: Option<usize>,
    instr_start: usize,
}

const DETERMINISTIC_RUNS: usize = 8;

#[test]
fn runtime_ownership_sibling_write_passes_on_verified_path() {
    let bytes = compile_program_to_semcode(tuple_assignment_source()).expect("compile");
    assert_eq!(&bytes[..8], &MAGIC11);

    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[OwnershipPathComponentSpec::TupleIndex(0)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[OwnershipPathComponentSpec::TupleIndex(1)],
            },
        ],
    );

    verify_semcode(&rewritten).expect("verify");
    run_verified_semcode(&rewritten).expect("sibling tuple write should pass");
}

#[test]
fn runtime_ownership_rejects_same_path_write_deterministically() {
    let bytes = compile_program_to_semcode(tuple_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[OwnershipPathComponentSpec::TupleIndex(0)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[OwnershipPathComponentSpec::TupleIndex(0)],
            },
        ],
    );

    assert_write_overlap_rejects_deterministically(&rewritten, "pair");
}

#[test]
fn runtime_ownership_rejects_parent_child_overlap_deterministically() {
    let bytes = compile_program_to_semcode(tuple_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[OwnershipPathComponentSpec::TupleIndex(0)],
            },
        ],
    );

    assert_write_overlap_rejects_deterministically(&rewritten, "pair");
}

#[test]
fn runtime_ownership_rejects_child_parent_overlap_deterministically() {
    let bytes = compile_program_to_semcode(tuple_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[OwnershipPathComponentSpec::TupleIndex(0)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[],
            },
        ],
    );

    assert_write_overlap_rejects_deterministically(&rewritten, "pair");
}

#[test]
fn runtime_ownership_inner_frame_borrow_does_not_leak_after_exit() {
    let bytes = compile_program_to_semcode(multi_frame_source()).expect("compile");
    assert_eq!(&bytes[..8], &MAGIC11);
    assert!(function_has_ownership_section(&bytes, "helper"));
    assert!(function_has_ownership_section(&bytes, "main"));

    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[OwnershipEventSpec {
            kind: OWNERSHIP_EVENT_KIND_WRITE,
            root: "pair",
            components: &[],
        }],
    );

    verify_semcode(&rewritten).expect("verify");
    run_verified_semcode(&rewritten).expect("inner-frame borrow must not leak after return");
}

#[test]
fn runtime_ownership_record_sibling_field_write_passes_on_verified_path() {
    let bytes = compile_program_to_semcode(record_assignment_source()).expect("compile");
    assert_eq!(&bytes[..8], &MAGIC12);
    assert!(function_has_ownership_section(&bytes, "main"));
    let (camera_field, quality_field) = record_field_component_ids(&bytes, "main");

    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "ctx",
                components: &[OwnershipPathComponentSpec::FieldSymbol(camera_field)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "ctx",
                components: &[OwnershipPathComponentSpec::FieldSymbol(quality_field)],
            },
        ],
    );

    verify_semcode(&rewritten).expect("verify");
    run_verified_semcode(&rewritten).expect("sibling record field write should pass");
}

#[test]
fn runtime_ownership_record_same_field_conflict_rejects() {
    let bytes = compile_program_to_semcode(record_assignment_source()).expect("compile");
    let (camera_field, _) = record_field_component_ids(&bytes, "main");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "ctx",
                components: &[OwnershipPathComponentSpec::FieldSymbol(camera_field)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "ctx",
                components: &[OwnershipPathComponentSpec::FieldSymbol(camera_field)],
            },
        ],
    );

    assert_write_overlap_rejects_deterministically(&rewritten, "ctx");
}

#[test]
fn runtime_ownership_record_parent_child_conflict_rejects() {
    let bytes = compile_program_to_semcode(record_assignment_source()).expect("compile");
    let (camera_field, _) = record_field_component_ids(&bytes, "main");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "ctx",
                components: &[],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "ctx",
                components: &[OwnershipPathComponentSpec::FieldSymbol(camera_field)],
            },
        ],
    );

    assert_write_overlap_rejects_deterministically(&rewritten, "ctx");
}

#[test]
fn runtime_ownership_record_child_parent_conflict_rejects() {
    let bytes = compile_program_to_semcode(record_assignment_source()).expect("compile");
    let (camera_field, _) = record_field_component_ids(&bytes, "main");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "ctx",
                components: &[OwnershipPathComponentSpec::FieldSymbol(camera_field)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "ctx",
                components: &[],
            },
        ],
    );

    assert_write_overlap_rejects_deterministically(&rewritten, "ctx");
}

#[test]
fn runtime_ownership_conflict_surface_is_stable_across_tuple_and_record_cases() {
    let tuple_bytes = compile_program_to_semcode(tuple_assignment_source()).expect("compile");
    let tuple_same_path = rewrite_function_ownership_events(
        &tuple_bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[OwnershipPathComponentSpec::TupleIndex(0)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[OwnershipPathComponentSpec::TupleIndex(0)],
            },
        ],
    );
    let tuple_parent_child = rewrite_function_ownership_events(
        &tuple_bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[OwnershipPathComponentSpec::TupleIndex(0)],
            },
        ],
    );
    let tuple_child_parent = rewrite_function_ownership_events(
        &tuple_bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[OwnershipPathComponentSpec::TupleIndex(0)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[],
            },
        ],
    );

    let record_bytes = compile_program_to_semcode(record_assignment_source()).expect("compile");
    let (camera_field, _) = record_field_component_ids(&record_bytes, "main");
    let record_same_field = rewrite_function_ownership_events(
        &record_bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "ctx",
                components: &[OwnershipPathComponentSpec::FieldSymbol(camera_field)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "ctx",
                components: &[OwnershipPathComponentSpec::FieldSymbol(camera_field)],
            },
        ],
    );
    let record_parent_child = rewrite_function_ownership_events(
        &record_bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "ctx",
                components: &[],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "ctx",
                components: &[OwnershipPathComponentSpec::FieldSymbol(camera_field)],
            },
        ],
    );
    let record_child_parent = rewrite_function_ownership_events(
        &record_bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "ctx",
                components: &[OwnershipPathComponentSpec::FieldSymbol(camera_field)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "ctx",
                components: &[],
            },
        ],
    );

    let observed = [
        observe_borrow_write_conflict_surface(&tuple_same_path),
        observe_borrow_write_conflict_surface(&tuple_parent_child),
        observe_borrow_write_conflict_surface(&tuple_child_parent),
        observe_borrow_write_conflict_surface(&record_same_field),
        observe_borrow_write_conflict_surface(&record_parent_child),
        observe_borrow_write_conflict_surface(&record_child_parent),
    ];

    for rendered in &observed[1..] {
        assert_eq!(rendered, &observed[0]);
    }
}

#[test]
fn runtime_ownership_record_inner_frame_borrow_does_not_leak_after_exit() {
    let bytes = compile_program_to_semcode(record_multi_frame_source()).expect("compile");
    assert_eq!(&bytes[..8], &MAGIC12);
    assert!(function_has_ownership_section(&bytes, "helper"));
    assert!(function_has_ownership_section(&bytes, "main"));

    verify_semcode(&bytes).expect("verify");
    run_verified_semcode(&bytes).expect("inner-frame record borrow must not leak after return");
}

#[test]
fn runtime_ownership_unsupported_paths_do_not_silently_claim_support() {
    for src in [adt_source(), schema_source()] {
        let bytes = compile_program_to_semcode(src).expect("compile");
        assert_ne!(&bytes[..8], &MAGIC11);
        assert_ne!(&bytes[..8], &MAGIC12);
        assert!(!any_function_has_ownership_section(&bytes));
        verify_semcode(&bytes).expect("verify");
        run_verified_semcode(&bytes).expect("run");
    }

    let _ = compile_program_to_semcode(indirect_record_projection_source())
        .expect_err("indirect record-field projection must not silently claim support");
}

#[test]
fn runtime_ownership_sibling_write_is_stable_across_runs() {
    let bytes = compile_program_to_semcode(tuple_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[OwnershipPathComponentSpec::TupleIndex(0)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[OwnershipPathComponentSpec::TupleIndex(1)],
            },
        ],
    );

    assert_repeated_verified_success(&rewritten, DETERMINISTIC_RUNS);
}

#[test]
fn runtime_ownership_same_path_rejects_identically_across_runs() {
    let bytes = compile_program_to_semcode(tuple_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[OwnershipPathComponentSpec::TupleIndex(0)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[OwnershipPathComponentSpec::TupleIndex(0)],
            },
        ],
    );

    assert_repeated_write_overlap_rejects(&rewritten, "pair", DETERMINISTIC_RUNS);
}

#[test]
fn runtime_ownership_parent_child_rejects_identically_across_runs() {
    let bytes = compile_program_to_semcode(tuple_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[OwnershipPathComponentSpec::TupleIndex(0)],
            },
        ],
    );

    assert_repeated_write_overlap_rejects(&rewritten, "pair", DETERMINISTIC_RUNS);
}

#[test]
fn runtime_ownership_child_parent_rejects_identically_across_runs() {
    let bytes = compile_program_to_semcode(tuple_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "pair",
                components: &[OwnershipPathComponentSpec::TupleIndex(0)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "pair",
                components: &[],
            },
        ],
    );

    assert_repeated_write_overlap_rejects(&rewritten, "pair", DETERMINISTIC_RUNS);
}

#[test]
fn runtime_ownership_multi_frame_cleanup_is_stable_across_runs() {
    let bytes = compile_program_to_semcode(multi_frame_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[OwnershipEventSpec {
            kind: OWNERSHIP_EVENT_KIND_WRITE,
            root: "pair",
            components: &[],
        }],
    );

    assert_repeated_verified_success(&rewritten, DETERMINISTIC_RUNS);
}

#[test]
fn runtime_ownership_record_sibling_write_is_stable_across_runs() {
    let bytes = compile_program_to_semcode(record_assignment_source()).expect("compile");
    assert_eq!(&bytes[..8], &MAGIC12);
    let (camera_field, quality_field) = record_field_component_ids(&bytes, "main");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "ctx",
                components: &[OwnershipPathComponentSpec::FieldSymbol(camera_field)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "ctx",
                components: &[OwnershipPathComponentSpec::FieldSymbol(quality_field)],
            },
        ],
    );

    assert_repeated_verified_success(&rewritten, DETERMINISTIC_RUNS);
}

#[test]
fn runtime_ownership_record_same_field_rejects_identically_across_runs() {
    let bytes = compile_program_to_semcode(record_assignment_source()).expect("compile");
    let (camera_field, _) = record_field_component_ids(&bytes, "main");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "ctx",
                components: &[OwnershipPathComponentSpec::FieldSymbol(camera_field)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "ctx",
                components: &[OwnershipPathComponentSpec::FieldSymbol(camera_field)],
            },
        ],
    );

    assert_repeated_write_overlap_rejects(&rewritten, "ctx", DETERMINISTIC_RUNS);
}

#[test]
fn runtime_ownership_record_parent_child_rejects_identically_across_runs() {
    let bytes = compile_program_to_semcode(record_assignment_source()).expect("compile");
    let (camera_field, _) = record_field_component_ids(&bytes, "main");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "ctx",
                components: &[],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "ctx",
                components: &[OwnershipPathComponentSpec::FieldSymbol(camera_field)],
            },
        ],
    );

    assert_repeated_write_overlap_rejects(&rewritten, "ctx", DETERMINISTIC_RUNS);
}

#[test]
fn runtime_ownership_record_child_parent_rejects_identically_across_runs() {
    let bytes = compile_program_to_semcode(record_assignment_source()).expect("compile");
    let (camera_field, _) = record_field_component_ids(&bytes, "main");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "ctx",
                components: &[OwnershipPathComponentSpec::FieldSymbol(camera_field)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "ctx",
                components: &[],
            },
        ],
    );

    assert_repeated_write_overlap_rejects(&rewritten, "ctx", DETERMINISTIC_RUNS);
}

#[test]
fn runtime_ownership_record_multi_frame_cleanup_is_stable_across_runs() {
    let bytes = compile_program_to_semcode(record_multi_frame_source()).expect("compile");
    assert_eq!(&bytes[..8], &MAGIC12);
    assert!(function_has_ownership_section(&bytes, "helper"));
    assert!(function_has_ownership_section(&bytes, "main"));

    assert_repeated_verified_success(&bytes, DETERMINISTIC_RUNS);
}

fn tuple_assignment_source() -> &'static str {
    r#"
        fn main() {
            let pair: (i32, bool) = (1, true);
            let other: i32 = 0;
            (pair, other) = ((2, false), 1);
            return;
        }
    "#
}

fn record_assignment_source() -> &'static str {
    r#"
        record DecisionContext {
            camera: quad,
            quality: f64,
        }

        fn main() {
            let camera: f64 = 0.0;
            let quality: f64 = 1.0;
            let ctx: f64 = 1.0;
            let probe: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
            let DecisionContext { camera: ref seen_camera, quality: _ } = probe;
            let patched: DecisionContext = probe with { quality: 1.0 };
            let _ = seen_camera;
            let _ = patched;
            ctx += 2.0;
            return;
        }
    "#
}

fn multi_frame_source() -> &'static str {
    r#"
        fn helper(pair: (i32, bool)) {
            let (ref left, _): (i32, bool) = pair;
            let _ = left;
            return;
        }

        fn main() {
            let pair: (i32, bool) = (1, true);
            let other: i32 = 0;
            helper((3, false));
            (pair, other) = ((2, false), 1);
            return;
        }
    "#
}

fn record_multi_frame_source() -> &'static str {
    r#"
        record DecisionContext {
            camera: quad,
            quality: f64,
        }

        fn helper(ctx: DecisionContext) {
            let DecisionContext { camera: ref seen_camera, quality: _ } = ctx;
            let _ = seen_camera;
            return;
        }

        fn main() {
            let ctx: DecisionContext = DecisionContext { camera: T, quality: 0.75 };
            helper(DecisionContext { camera: N, quality: 0.5 });
            let patched: DecisionContext = ctx with { quality: 1.0 };
            let _ = patched;
            return;
        }
    "#
}

fn adt_source() -> &'static str {
    r#"
        enum Maybe {
            None,
            Some(bool),
        }

        fn choose(flag: bool) -> Maybe {
            return Maybe::Some(flag);
        }

        fn main() {
            let left: Maybe = choose(true);
            let right: Maybe = Maybe::None;
            let _ = left;
            let _ = right;
            return;
        }
    "#
}

fn schema_source() -> &'static str {
    r#"
        api schema Telemetry version(1) {
            level: i32,
            active: bool,
        }

        fn main() {
            let total: i32 = 1;
            let _ = total;
            return;
        }
    "#
}

fn indirect_record_projection_source() -> &'static str {
    r#"
        record CameraState {
            active: quad,
        }

        record DecisionContext {
            camera: CameraState,
            quality: f64,
        }

        fn main() {
            let ctx: DecisionContext =
                DecisionContext { camera: CameraState { active: T }, quality: 0.75 };
            let DecisionContext { camera: CameraState { active: ref seen_active }, quality: _ } = ctx;
            let _ = seen_active;
            return;
        }
    "#
}

fn assert_write_overlap_rejects_deterministically(bytes: &[u8], symbol_name: &str) {
    assert_repeated_write_overlap_rejects(bytes, symbol_name, 2);
}

fn assert_repeated_verified_success(bytes: &[u8], runs: usize) {
    verify_semcode(bytes).expect("verify");
    for _ in 0..runs {
        run_verified_semcode(bytes).expect("verified run must stay successful");
    }
}

fn observe_borrow_write_conflict_surface(bytes: &[u8]) -> String {
    verify_semcode(bytes).expect("verify");

    let err = run_verified_semcode(bytes).expect_err("runtime overlap must reject");
    let rendered = format!("{err}");
    assert!(matches!(
        err,
        RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict)
    ));
    assert_eq!(rendered, "write path overlaps active borrow");
    rendered
}

fn assert_repeated_write_overlap_rejects(bytes: &[u8], _symbol_name: &str, runs: usize) {
    verify_semcode(bytes).expect("verify");

    let mut observed = Vec::with_capacity(runs);
    for _ in 0..runs {
        let err = run_verified_semcode(bytes).expect_err("runtime overlap must reject");
        let rendered = format!("{err}");
        assert!(matches!(
            err,
            RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict)
        ));
        assert_eq!(rendered, "write path overlaps active borrow");
        observed.push(rendered);
    }

    for rendered in &observed[1..] {
        assert_eq!(rendered, &observed[0]);
    }
}

fn any_function_has_ownership_section(bytes: &[u8]) -> bool {
    let mut cursor = 8usize;
    while cursor < bytes.len() {
        let (name, code, next) = next_function(bytes, cursor);
        let _ = name;
        if parse_function_layout(code).ownership_start.is_some() {
            return true;
        }
        cursor = next;
    }
    false
}

fn function_has_ownership_section(bytes: &[u8], target: &str) -> bool {
    let (_, code, _) = find_function(bytes, target);
    parse_function_layout(code).ownership_start.is_some()
}

fn rewrite_function_ownership_events(
    bytes: &[u8],
    target: &str,
    events: &[OwnershipEventSpec<'_>],
) -> Vec<u8> {
    let mut out = Vec::with_capacity(bytes.len());
    out.extend_from_slice(&bytes[..8]);

    let mut cursor = 8usize;
    let mut rewrote = false;
    while cursor < bytes.len() {
        let (name, code, next) = next_function(bytes, cursor);
        let rewritten = if name == target {
            rewrote = true;
            rewrite_function_code(code, events)
        } else {
            code.to_vec()
        };

        out.extend_from_slice(&(name.len() as u16).to_le_bytes());
        out.extend_from_slice(name.as_bytes());
        out.extend_from_slice(&(rewritten.len() as u32).to_le_bytes());
        out.extend_from_slice(&rewritten);
        cursor = next;
    }

    assert!(rewrote, "target function '{target}' not found");
    out
}

fn rewrite_function_code(code: &[u8], events: &[OwnershipEventSpec<'_>]) -> Vec<u8> {
    let layout = parse_function_layout(code);
    let ownership_start = layout.ownership_start.expect("OWN0 section");
    let mut out = Vec::with_capacity(code.len());
    out.extend_from_slice(&code[..ownership_start]);
    out.extend_from_slice(&ownership_section_bytes(&layout.strings, events));
    out.extend_from_slice(&code[layout.instr_start..]);
    out
}

fn ownership_section_bytes(strings: &[String], events: &[OwnershipEventSpec<'_>]) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&OWNERSHIP_SECTION_TAG);
    out.extend_from_slice(&(events.len() as u16).to_le_bytes());
    for event in events {
        let root = strings
            .iter()
            .position(|name| name == event.root)
            .unwrap_or_else(|| panic!("missing root '{}'", event.root)) as u32;
        append_ownership_event(&mut out, event.kind, root, event.components);
    }
    out
}

fn append_ownership_event(
    out: &mut Vec<u8>,
    kind: u8,
    root: u32,
    components: &[OwnershipPathComponentSpec],
) {
    out.push(kind);
    out.extend_from_slice(&root.to_le_bytes());
    out.extend_from_slice(&(components.len() as u16).to_le_bytes());
    for component in components {
        match component {
            OwnershipPathComponentSpec::TupleIndex(index) => {
                out.push(OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX);
                out.extend_from_slice(&index.to_le_bytes());
            }
            OwnershipPathComponentSpec::FieldSymbol(field) => {
                out.push(OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL);
                out.extend_from_slice(&field.to_le_bytes());
            }
        }
    }
}

fn record_field_component_ids(bytes: &[u8], target: &str) -> (u32, u32) {
    let (_, code, _) = find_function(bytes, target);
    let layout = parse_function_layout(code);
    let mut cursor = layout.ownership_start.expect("OWN0 section");
    cursor += OWNERSHIP_SECTION_TAG.len();
    let count = read_u16_le(code, &mut cursor).expect("ownership count") as usize;

    let mut borrow_field = None;
    let mut write_field = None;
    for _ in 0..count {
        let kind = read_u8(code, &mut cursor).expect("ownership kind");
        let _ = read_u32_le(code, &mut cursor).expect("ownership root");
        let component_count = read_u16_le(code, &mut cursor).expect("ownership component count");
        let mut only_field = None;
        for _ in 0..component_count {
            let component_kind = read_u8(code, &mut cursor).expect("ownership component kind");
            match component_kind {
                OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX => {
                    let _ = read_u16_le(code, &mut cursor).expect("tuple component");
                }
                OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL => {
                    only_field = Some(read_u32_le(code, &mut cursor).expect("field component"));
                }
                _ => panic!("unexpected ownership component kind 0x{component_kind:02x}"),
            }
        }
        match (kind, only_field) {
            (OWNERSHIP_EVENT_KIND_BORROW, Some(field)) => borrow_field = Some(field),
            (OWNERSHIP_EVENT_KIND_WRITE, Some(field)) => write_field = Some(field),
            _ => {}
        }
    }

    (
        borrow_field.expect("record borrow field"),
        write_field.expect("record write field"),
    )
}

fn parse_function_layout(code: &[u8]) -> FunctionLayout {
    let mut cursor = 0usize;
    let string_count = read_u16_le(code, &mut cursor).expect("string count") as usize;
    let mut strings = Vec::with_capacity(string_count);
    for _ in 0..string_count {
        let len = read_u16_le(code, &mut cursor).expect("string len") as usize;
        strings.push(
            read_utf8(code, &mut cursor, len)
                .expect("string")
                .to_string(),
        );
    }

    if cursor + 4 <= code.len() && &code[cursor..cursor + 4] == b"DBG0" {
        cursor += 4;
        let count = read_u16_le(code, &mut cursor).expect("debug count") as usize;
        for _ in 0..count {
            let _ = read_u32_le(code, &mut cursor).expect("debug pc");
            let _ = read_u32_le(code, &mut cursor).expect("debug line");
            let _ = read_u16_le(code, &mut cursor).expect("debug col");
        }
    }

    let ownership_start =
        if cursor + 4 <= code.len() && &code[cursor..cursor + 4] == OWNERSHIP_SECTION_TAG {
            Some(cursor)
        } else {
            None
        };

    if ownership_start.is_some() {
        cursor += OWNERSHIP_SECTION_TAG.len();
        let count = read_u16_le(code, &mut cursor).expect("ownership count") as usize;
        for _ in 0..count {
            let _ = read_u8(code, &mut cursor).expect("ownership kind");
            let _ = read_u32_le(code, &mut cursor).expect("ownership root");
            let component_count =
                read_u16_le(code, &mut cursor).expect("ownership component count") as usize;
            for _ in 0..component_count {
                let kind = read_u8(code, &mut cursor).expect("ownership component kind");
                match kind {
                    OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX => {
                        let _ = read_u16_le(code, &mut cursor).expect("ownership component value");
                    }
                    OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL => {
                        let _ = read_u32_le(code, &mut cursor).expect("ownership component value");
                    }
                    _ => panic!("unexpected ownership component kind 0x{kind:02x}"),
                }
            }
        }
    }

    FunctionLayout {
        strings,
        ownership_start,
        instr_start: cursor,
    }
}

fn find_function<'a>(bytes: &'a [u8], target: &str) -> (String, &'a [u8], usize) {
    let mut cursor = 8usize;
    while cursor < bytes.len() {
        let (name, code, next) = next_function(bytes, cursor);
        if name == target {
            return (name, code, next);
        }
        cursor = next;
    }
    panic!("function '{target}' not found");
}

fn next_function<'a>(bytes: &'a [u8], start: usize) -> (String, &'a [u8], usize) {
    let mut cursor = start;
    let name_len = read_u16_le(bytes, &mut cursor).expect("function name len") as usize;
    let name = read_utf8(bytes, &mut cursor, name_len).expect("function name");
    let code_len = read_u32_le(bytes, &mut cursor).expect("function code len") as usize;
    let code_start = cursor;
    let code_end = code_start + code_len;
    (name, &bytes[code_start..code_end], code_end)
}
