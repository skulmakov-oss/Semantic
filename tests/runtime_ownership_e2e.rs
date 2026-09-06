#![allow(clippy::op_ref, clippy::needless_lifetimes)]
use sm_emit::compile_program_to_semcode;
use sm_ir::semcode_format::{
    header_spec_from_magic, read_u16_le, read_u32_le, read_u8, read_utf8,
    ACTIVATION_MODE_FRAME_ENTRY, ACTIVATION_MODE_STORE_VAR_SITE, MAGIC20,
    OWNERSHIP_EVENT_KIND_BORROW, OWNERSHIP_EVENT_KIND_WRITE, OWNERSHIP_PATH_COMPONENT_ADT_PAYLOAD,
    OWNERSHIP_PATH_COMPONENT_FIELD_SYMBOL, OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX,
    OWNERSHIP_PATH_COMPONENT_TUPLE_INDEX, OWNERSHIP_SECTION_TAG,
    SEMCODE_OWNERSHIP_ANCHOR_MIN_REVISION, WRITE_EXECUTION_MODE_STORE_VAR_SITE,
};

// #1726 Checkpoint D2a: a Tuple/Record Borrow event now always carries a
// resolved ActivationSiteId, promoting the artifact to SEMCOD20/rev21 (a
// Borrow event gains an activation-mode byte, +4 anchor bytes for
// StoreVarSite, before its own root). Programs with no such Borrow event
// (plain-local, ADT-only) stay below that revision and keep the legacy
// layout. Every hand-rolled OWN0 reader/writer in this file must therefore
// know the artifact's actual header revision - it is never safe to assume
// either shape.
fn header_rev_of(bytes: &[u8]) -> u16 {
    let mut magic = [0u8; 8];
    magic.copy_from_slice(&bytes[0..8]);
    header_spec_from_magic(&magic)
        .expect("known header magic")
        .rev
}

/// Reads a Borrow event's activation-mode prefix or a Write event's
/// execution-mode prefix, whichever the event's own `kind` and the header
/// revision require (#1891 Checkpoint W2D: Write gained the identical wire
/// position as Borrow's existing activation-mode prefix, at the same
/// revision gate), positioning `cursor` at the start of the event's `root`
/// field either way.
fn skip_borrow_activation_prefix(code: &[u8], cursor: &mut usize, kind: u8, header_rev: u16) {
    if header_rev < SEMCODE_OWNERSHIP_ANCHOR_MIN_REVISION {
        return;
    }
    if kind == OWNERSHIP_EVENT_KIND_BORROW {
        let mode = read_u8(code, cursor).expect("activation mode");
        if mode == ACTIVATION_MODE_STORE_VAR_SITE {
            let _ = read_u32_le(code, cursor).expect("executable anchor");
        }
    } else if kind == OWNERSHIP_EVENT_KIND_WRITE {
        let _mode = read_u8(code, cursor).expect("write execution mode");
        let _ = read_u32_le(code, cursor).expect("write executable anchor");
    }
}
use sm_runtime_core::RuntimeTrap;
use sm_vm::RuntimeError;

fn run_token_first_main(semcode: &[u8]) -> Result<(), RuntimeError> {
    let token = sm_verify::verify_semcode_token(semcode).expect("token admission");
    let entry_token = token.require_entry("main").expect("entry resolution");
    sm_vm::run_verified_entry_semcode(&entry_token)
}

#[derive(Clone, Copy)]
enum OwnershipPathComponentSpec {
    TupleIndex(u16),
    FieldSymbol(u32),
    SequenceIndexStatic(u32),
    AdtPayload(u32, u16),
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
    // #1773 (FA-09-005): end of the OWN0 section, i.e. the start of the
    // (now mandatory) SIG0 section that follows it - lets a rewrite that
    // only wants to replace OWN0 preserve the real SIG0 + instruction
    // stream bytes verbatim, rather than discarding them.
    own0_end: usize,
}

const DETERMINISTIC_RUNS: usize = 8;

#[test]
fn runtime_ownership_sibling_write_passes_on_verified_path() {
    let bytes = compile_program_to_semcode(tuple_assignment_source()).expect("compile");
    // #1891 Checkpoint W2D: the tuple-destructuring assignment's Write
    // events now always carry a resolved WriteSiteId (Checkpoint W2C),
    // promoting this artifact to SEMCOD20/rev21 - was SEMCOD19/rev20 before
    // this checkpoint.
    assert_eq!(&bytes[..8], &MAGIC20);

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

    run_token_first_main(&rewritten).expect("sibling tuple write should pass");
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
fn runtime_ownership_sequence_same_index_conflict_rejects() {
    let bytes = compile_program_to_semcode(sequence_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "seq",
                components: &[OwnershipPathComponentSpec::SequenceIndexStatic(0)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "seq",
                components: &[OwnershipPathComponentSpec::SequenceIndexStatic(0)],
            },
        ],
    );

    assert_write_overlap_rejects_deterministically(&rewritten, "seq");
}

#[test]
fn runtime_ownership_sequence_sibling_index_write_passes() {
    let bytes = compile_program_to_semcode(sequence_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "seq",
                components: &[OwnershipPathComponentSpec::SequenceIndexStatic(0)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "seq",
                components: &[OwnershipPathComponentSpec::SequenceIndexStatic(1)],
            },
        ],
    );

    run_token_first_main(&rewritten).expect("sibling sequence write should pass");
}

#[test]
fn runtime_ownership_sequence_parent_child_conflict_rejects() {
    let bytes = compile_program_to_semcode(sequence_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "seq",
                components: &[],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "seq",
                components: &[OwnershipPathComponentSpec::SequenceIndexStatic(0)],
            },
        ],
    );

    assert_write_overlap_rejects_deterministically(&rewritten, "seq");
}

#[test]
fn runtime_ownership_sequence_child_parent_conflict_rejects() {
    let bytes = compile_program_to_semcode(sequence_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "seq",
                components: &[OwnershipPathComponentSpec::SequenceIndexStatic(0)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "seq",
                components: &[],
            },
        ],
    );

    assert_write_overlap_rejects_deterministically(&rewritten, "seq");
}

#[test]
fn runtime_ownership_sequence_dynamic_borrow_conflicts_with_static_index_zero_write() {
    let bytes = compile_program_to_semcode(sequence_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "seq",
                components: &[],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "seq",
                components: &[OwnershipPathComponentSpec::SequenceIndexStatic(0)],
            },
        ],
    );

    assert_write_overlap_rejects_deterministically(&rewritten, "seq");
}

#[test]
fn runtime_ownership_sequence_dynamic_borrow_conflicts_with_static_sibling_write() {
    let bytes = compile_program_to_semcode(sequence_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "seq",
                components: &[],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "seq",
                components: &[OwnershipPathComponentSpec::SequenceIndexStatic(1)],
            },
        ],
    );

    assert_write_overlap_rejects_deterministically(&rewritten, "seq");
}

#[test]
fn runtime_ownership_sequence_dynamic_borrow_conflicts_with_parent_write() {
    let bytes = compile_program_to_semcode(sequence_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "seq",
                components: &[],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "seq",
                components: &[],
            },
        ],
    );

    assert_write_overlap_rejects_deterministically(&rewritten, "seq");
}

#[test]
fn runtime_ownership_sequence_parent_borrow_conflicts_with_dynamic_write() {
    let bytes = compile_program_to_semcode(sequence_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "seq",
                components: &[],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "seq",
                components: &[],
            },
        ],
    );

    assert_write_overlap_rejects_deterministically(&rewritten, "seq");
}

#[test]
fn runtime_ownership_inner_frame_borrow_does_not_leak_after_exit() {
    let bytes = compile_program_to_semcode(multi_frame_source()).expect("compile");
    assert_eq!(&bytes[..8], &MAGIC20);
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

    run_token_first_main(&rewritten).expect("inner-frame borrow must not leak after return");
}

#[test]
fn runtime_ownership_record_sibling_field_write_passes_on_verified_path() {
    let bytes = compile_program_to_semcode(record_assignment_source()).expect("compile");
    assert_eq!(&bytes[..8], &MAGIC20);
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

    run_token_first_main(&rewritten).expect("sibling record field write should pass");
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
    assert_eq!(&bytes[..8], &MAGIC20);
    assert!(function_has_ownership_section(&bytes, "helper"));
    assert!(function_has_ownership_section(&bytes, "main"));

    run_token_first_main(&bytes).expect("inner-frame record borrow must not leak after return");
}

#[test]
fn runtime_ownership_unsupported_paths_do_not_silently_claim_support() {
    let src = schema_source();
    let bytes = compile_program_to_semcode(src).expect("compile");
    // #1773 (FA-09-005): OWN0 is now unconditionally present (mandatory
    // once the header reaches SEMCODE_SIGNATURE_MIN_REVISION), so the real
    // invariant this program must uphold is "no ownership events recorded"
    // - see `any_function_has_nonempty_ownership_section`'s doc comment.
    assert!(!any_function_has_nonempty_ownership_section(&bytes));
    run_token_first_main(&bytes).expect("run");

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
    assert_eq!(&bytes[..8], &MAGIC20);
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
    assert_eq!(&bytes[..8], &MAGIC20);
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

fn sequence_assignment_source() -> &'static str {
    r#"
        fn main() {
            let seq: (i32, bool) = (1, true);
            let other: i32 = 0;
            (seq, other) = ((2, false), 1);
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
    for _ in 0..runs {
        run_token_first_main(bytes).expect("verified run must stay successful");
    }
}

fn observe_borrow_write_conflict_surface(bytes: &[u8]) -> String {
    let err = run_token_first_main(bytes).expect_err("runtime overlap must reject");
    let rendered = format!("{err}");
    assert!(matches!(
        err,
        RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict)
    ));
    assert_eq!(rendered, "write path overlaps active borrow");
    rendered
}

fn assert_repeated_write_overlap_rejects(bytes: &[u8], _symbol_name: &str, runs: usize) {
    let mut observed = Vec::with_capacity(runs);
    for _ in 0..runs {
        let err = run_token_first_main(bytes).expect_err("runtime overlap must reject");
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

// #1773 (FA-09-005): OWN0 is now unconditionally present in every compiled
// function's envelope (mandatory once the header reaches
// SEMCODE_SIGNATURE_MIN_REVISION), so `any_function_has_ownership_section`
// no longer distinguishes "does this program need ownership tracking" -
// it's now vacuously true for any program. This checks the thing that
// still matters: does the (always-present) section actually record any
// borrow/write events, or is it structurally empty.
fn any_function_has_nonempty_ownership_section(bytes: &[u8]) -> bool {
    let header_rev = header_rev_of(bytes);
    let mut cursor = 8usize;
    while cursor < bytes.len() {
        let (name, code, next) = next_function(bytes, cursor);
        let _ = name;
        let layout = parse_function_layout(code, header_rev);
        if let Some(ownership_start) = layout.ownership_start {
            let mut event_cursor = ownership_start + OWNERSHIP_SECTION_TAG.len();
            let count = read_u16_le(code, &mut event_cursor).expect("ownership count");
            if count > 0 {
                return true;
            }
        }
        cursor = next;
    }
    false
}

fn function_has_ownership_section(bytes: &[u8], target: &str) -> bool {
    let header_rev = header_rev_of(bytes);
    let (_, code, _) = find_function(bytes, target);
    parse_function_layout(code, header_rev)
        .ownership_start
        .is_some()
}

fn rewrite_function_ownership_events(
    bytes: &[u8],
    target: &str,
    events: &[OwnershipEventSpec<'_>],
) -> Vec<u8> {
    let header_rev = header_rev_of(bytes);
    let mut out = Vec::with_capacity(bytes.len());
    out.extend_from_slice(&bytes[..8]);

    let mut cursor = 8usize;
    let mut rewrote = false;
    while cursor < bytes.len() {
        let (name, code, next) = next_function(bytes, cursor);
        let rewritten = if name == target {
            rewrote = true;
            rewrite_function_code(code, events, header_rev)
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

fn rewrite_function_code(
    code: &[u8],
    events: &[OwnershipEventSpec<'_>],
    header_rev: u16,
) -> Vec<u8> {
    let layout = parse_function_layout(code, header_rev);
    let ownership_start = layout.ownership_start.expect("OWN0 section");
    let mut out = Vec::with_capacity(code.len());
    out.extend_from_slice(&code[..ownership_start]);
    out.extend_from_slice(&ownership_section_bytes(&layout, events, header_rev));
    // #1773 (FA-09-005): preserve the real SIG0 section verbatim - only
    // OWN0 is being replaced here, not the range through `instr_start`,
    // which would silently drop SIG0 along with it.
    out.extend_from_slice(&code[layout.own0_end..]);
    out
}

// #1724 (FA-04-018): `layout.strings` is the compiled function's own
// local StoreVar/LoadVar/Call operand string table. Before #1724, a
// lexical binding's StoreVar/LoadVar key was identical to its source
// spelling, so this harness could look a root up by raw name directly.
// #1724 intentionally removes that representation coincidence - every
// lexical binding now gets a scope-aware, deterministic, mangled key of
// the form `__sm_local_<id>_<source_name>` (see `LoweredLocalEnv` in
// `crates/sm-ir/src/legacy_lowering.rs`), so shadowed bindings with the
// same spelling no longer collide on one runtime-local identity.
//
// This resolver finds the *single* lowered key whose mangled suffix
// matches `source_name` exactly. It fails closed on both "not found" and
// "ambiguous" (more than one lowered key for that spelling, e.g. a
// shadowed variable): no raw-spelling fallback, no first-match
// heuristic. Either of those would silently reintroduce, at this
// test-harness boundary, the exact identity-collapse #1724's production
// fix exists to repair - and specifically for #1724's own shadowing
// regressions, a "pick any match" resolver would hide the very bug this
// suite exists to catch instead of surfacing it as an ambiguous lookup.
fn resolve_unique_lowered_local_key(layout: &FunctionLayout, source_name: &str) -> String {
    let matches: Vec<&String> = layout
        .strings
        .iter()
        .filter(|candidate| is_lowered_local_key_for(candidate, source_name))
        .collect();
    match matches.as_slice() {
        [] => panic!(
            "no lowered runtime-local key found for source root '{source_name}' - \
             looked for '__sm_local_<id>_{source_name}' in the function's string \
             table {:?}",
            layout.strings
        ),
        [single] => (*single).clone(),
        multiple => panic!(
            "ambiguous lowered runtime-local key for source root '{source_name}': \
             {multiple:?} all match - the test must disambiguate which binding it means"
        ),
    }
}

/// Exact-shape check for `__sm_local_<digits>_<source_name>`: strips the
/// reserved prefix, consumes a non-empty run of ASCII digits as the
/// counter, requires exactly one `_` separator, then compares the
/// remainder to `source_name` verbatim (not merely as a suffix - a bare
/// suffix check could be fooled by a source name that itself contains a
/// leading-underscore-adjacent substring of another binding's spelling).
fn is_lowered_local_key_for(candidate: &str, source_name: &str) -> bool {
    let Some(rest) = candidate.strip_prefix("__sm_local_") else {
        return false;
    };
    let digits_end = rest
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(rest.len());
    if digits_end == 0 {
        return false;
    }
    let Some(after_digits) = rest.get(digits_end..) else {
        return false;
    };
    let Some(spelling) = after_digits.strip_prefix('_') else {
        return false;
    };
    spelling == source_name
}

fn ownership_section_bytes(
    layout: &FunctionLayout,
    events: &[OwnershipEventSpec<'_>],
    header_rev: u16,
) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&OWNERSHIP_SECTION_TAG);
    out.extend_from_slice(&(events.len() as u16).to_le_bytes());
    for event in events {
        let lowered_key = resolve_unique_lowered_local_key(layout, event.root);
        let root = layout
            .strings
            .iter()
            .position(|name| *name == lowered_key)
            .expect("resolved lowered key must be present in the string table it was resolved from")
            as u32;
        // #1726 Checkpoint D2a / #1891 Checkpoint W2D: these are synthetic
        // events, not resolved from a real compile, so there is no real
        // ActivationSiteId/WriteSiteId/anchor to encode. `FrameEntry` and
        // `StoreVarSite(0)` are correct either way: sm-vm does not consult
        // either wire mode tag's anchor yet (Borrow: Checkpoint D3, Write:
        // Checkpoint W2E+), so this only needs to satisfy the rev21
        // structural grammar, not express any particular activation/
        // execution semantics. This prefix must land AFTER `kind` and
        // BEFORE `root` on the wire, so it is passed into
        // `append_ownership_event` rather than pushed directly onto `out`
        // here - `out`'s very next byte is still `kind`, written first by
        // that function.
        let mut mode_prefix = Vec::new();
        if header_rev >= SEMCODE_OWNERSHIP_ANCHOR_MIN_REVISION {
            if event.kind == OWNERSHIP_EVENT_KIND_BORROW {
                mode_prefix.push(ACTIVATION_MODE_FRAME_ENTRY);
            } else if event.kind == OWNERSHIP_EVENT_KIND_WRITE {
                mode_prefix.push(WRITE_EXECUTION_MODE_STORE_VAR_SITE);
                mode_prefix.extend_from_slice(&0u32.to_le_bytes());
            }
        }
        append_ownership_event(&mut out, event.kind, &mode_prefix, root, event.components);
    }
    out
}

// #1724 (FA-04-018): proves `resolve_unique_lowered_local_key` cannot be
// fooled by shadowing - the exact scenario #1724's production fix exists
// to repair. Two distinct lexical bindings spelled "x" now lower to two
// distinct keys (`__sm_local_<id>_x`) in the same function's string
// table; asking for "x" without further disambiguation must fail closed
// rather than silently picking either one.
#[test]
#[should_panic(expected = "ambiguous")]
fn resolve_unique_lowered_local_key_fails_closed_on_shadowed_spelling() {
    let src = r#"
        fn main() {
            let x: i32 = 1;
            if true {
                let x: i32 = 2;
                let y: i32 = x;
                let _ = y;
            }
            let z: i32 = x;
            let _ = z;
            return;
        }
    "#;
    let semcode = compile_program_to_semcode(src).expect("compile");
    let (_, code, _) = find_function(&semcode, "main");
    let layout = parse_function_layout(code, header_rev_of(&semcode));
    let _ = resolve_unique_lowered_local_key(&layout, "x");
}

#[test]
fn resolve_unique_lowered_local_key_resolves_single_binding() {
    let src = r#"
        fn main() {
            let total: i32 = 1;
            let _ = total;
            return;
        }
    "#;
    let semcode = compile_program_to_semcode(src).expect("compile");
    let (_, code, _) = find_function(&semcode, "main");
    let layout = parse_function_layout(code, header_rev_of(&semcode));
    let key = resolve_unique_lowered_local_key(&layout, "total");
    assert!(is_lowered_local_key_for(&key, "total"));
    assert!(layout.strings.contains(&key));
}

fn append_ownership_event(
    out: &mut Vec<u8>,
    kind: u8,
    mode_prefix: &[u8],
    root: u32,
    components: &[OwnershipPathComponentSpec],
) {
    out.push(kind);
    out.extend_from_slice(mode_prefix);
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
            OwnershipPathComponentSpec::SequenceIndexStatic(index) => {
                out.push(OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX);
                out.extend_from_slice(&index.to_le_bytes());
            }
            OwnershipPathComponentSpec::AdtPayload(variant, index) => {
                out.push(OWNERSHIP_PATH_COMPONENT_ADT_PAYLOAD);
                out.extend_from_slice(&variant.to_le_bytes());
                out.extend_from_slice(&index.to_le_bytes());
            }
        }
    }
}

fn record_field_component_ids(bytes: &[u8], target: &str) -> (u32, u32) {
    let header_rev = header_rev_of(bytes);
    let (_, code, _) = find_function(bytes, target);
    let layout = parse_function_layout(code, header_rev);
    let mut cursor = layout.ownership_start.expect("OWN0 section");
    cursor += OWNERSHIP_SECTION_TAG.len();
    let count = read_u16_le(code, &mut cursor).expect("ownership count") as usize;

    let mut borrow_field = None;
    let mut write_field = None;
    for _ in 0..count {
        let kind = read_u8(code, &mut cursor).expect("ownership kind");
        skip_borrow_activation_prefix(code, &mut cursor, kind, header_rev);
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
                OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX => {
                    let _ = read_u32_le(code, &mut cursor).expect("sequence component");
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

fn parse_function_layout(code: &[u8], header_rev: u16) -> FunctionLayout {
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
            let kind = read_u8(code, &mut cursor).expect("ownership kind");
            skip_borrow_activation_prefix(code, &mut cursor, kind, header_rev);
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
                    OWNERSHIP_PATH_COMPONENT_SEQUENCE_INDEX => {
                        let _ = read_u32_le(code, &mut cursor).expect("ownership component value");
                    }
                    OWNERSHIP_PATH_COMPONENT_ADT_PAYLOAD => {
                        let _ = read_u32_le(code, &mut cursor).expect("ownership variant symbol");
                        let _ = read_u16_le(code, &mut cursor).expect("ownership adt index");
                    }
                    _ => panic!("unexpected ownership component kind 0x{kind:02x}"),
                }
            }
        }
    }

    FunctionLayout {
        strings,
        ownership_start,
        own0_end: cursor,
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

fn option_assignment_source() -> &'static str {
    r#"
fn main() {
    let mut opt: Option(f64) = Option::Some(42.0);
    opt = Option::None;
    opt = Option::Some(0.0);
    return;
}
"#
}

#[test]
fn runtime_ownership_option_rejects_same_path_write_deterministically() {
    let bytes = compile_program_to_semcode(option_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "opt",
                components: &[OwnershipPathComponentSpec::AdtPayload(42, 0)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "opt",
                components: &[OwnershipPathComponentSpec::AdtPayload(42, 0)],
            },
        ],
    );

    assert_write_overlap_rejects_deterministically(&rewritten, "opt");
}

#[test]
fn runtime_ownership_option_sibling_write_passes_on_verified_path() {
    let bytes = compile_program_to_semcode(option_assignment_source()).expect("compile");
    let rewritten = rewrite_function_ownership_events(
        &bytes,
        "main",
        &[
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_BORROW,
                root: "opt",
                components: &[OwnershipPathComponentSpec::AdtPayload(42, 0)],
            },
            OwnershipEventSpec {
                kind: OWNERSHIP_EVENT_KIND_WRITE,
                root: "opt",
                components: &[OwnershipPathComponentSpec::AdtPayload(43, 0)],
            },
        ],
    );

    run_token_first_main(&rewritten).expect("sibling adt payload write should pass");
}
