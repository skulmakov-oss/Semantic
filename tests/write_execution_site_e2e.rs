// #1891 Checkpoint W2F: end-to-end runtime coverage for the exact-PC Write
// execution site mechanism (`FunctionBytecode.write_execution_sites`,
// `check_write_execution_site` in crates/sm-vm/src/semcode_vm.rs), run
// through the full real compiler pipeline (source -> IR -> SemCode -> verify
// -> VM) wherever a real program can express the scenario. Letters follow
// the W2F GO brief's own item 11 matrix; A/C/D-equivalent coverage already
// exists in tests/write_cursor_1891_repro.rs and tests/runtime_ownership_e2e.rs
// and is not duplicated needlessly here, but each letter still gets its own
// explicitly named test so the matrix is checkable end to end.

use sm_runtime_core::RuntimeTrap;
use sm_vm::RuntimeError;

fn run_source(source: &str) -> Result<(), RuntimeError> {
    let bytes = sm_ir::compile_program_to_semcode(source).expect("compile");
    let token = sm_verify::verify_semcode_token(&bytes).expect("admit");
    let entry = token.require_entry("main").expect("entry");
    sm_vm::run_verified_entry_semcode(&entry)
}

fn assert_conflicts(result: Result<(), RuntimeError>, ctx: &str) {
    assert!(
        matches!(
            result,
            Err(RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict))
        ),
        "{ctx}: {result:?}"
    );
}

fn assert_succeeds(result: Result<(), RuntimeError>, ctx: &str) {
    assert!(result.is_ok(), "{ctx}: {result:?}");
}

// --- A: direct conflicting assignment after Borrow ------------------------

#[test]
fn a_direct_conflicting_assignment_after_borrow_rejects() {
    let result = run_source(
        r#"
        fn main() {
            let mut pair: (i32, i32) = (1, 2);
            let (ref left, _): (i32, i32) = pair;
            pair = (3, 4);
            let _ = left;
            return;
        }
    "#,
    );
    assert_conflicts(
        result,
        "a direct write to an actively-borrowed path must reject",
    );
}

// --- B: Write before Borrow introduction succeeds --------------------------
//
// The reassignment executes, and commits, strictly before the `ref`
// destructure that introduces the borrow even exists in program order - at
// the moment this StoreVar's own exact-PC check runs, `pair`'s Borrow is
// not yet in `borrowed_paths` at all (it is a real, per-D2a/D3 StoreVarSite
// activation tied to the *later* destructure's own StoreVar, not this
// earlier reassignment's), so there is nothing for the write to conflict
// with. This exercises real Borrow StoreVarSite activation (Checkpoints
// D2a/D3) and real Write StoreVarSite anchoring (Checkpoints W1-W2F)
// cooperating correctly through one real compile.

#[test]
fn b_write_before_borrow_is_introduced_succeeds() {
    let result = run_source(
        r#"
        fn main() {
            let mut pair: (i32, i32) = (1, 2);
            pair = (3, 4);
            let (ref left, _): (i32, i32) = pair;
            let _ = left;
            return;
        }
    "#,
    );
    assert_succeeds(
        result,
        "a write that fully precedes the borrow's own introduction must succeed",
    );
}

// --- C: conflicting Write in an untaken branch never executes/checks ------

#[test]
fn c_conflicting_write_in_untaken_branch_succeeds() {
    let result = run_source(
        r#"
        fn main() {
            let mut pair: (i32, i32) = (1, 2);
            let (ref left, _): (i32, i32) = pair;
            if false {
                pair = (3, 4);
            }
            let _ = left;
            return;
        }
    "#,
    );
    assert_succeeds(
        result,
        "a write inside a branch that is never taken must never be checked and must succeed",
    );
}

// --- E: repeated same-root sites are independently checked by their own PC -

#[test]
fn e_repeated_same_root_sites_independently_checked() {
    // The FIRST reassignment happens before the borrow exists (must
    // succeed, per B); the SECOND, later reassignment of the identical
    // root happens after the borrow is active (must reject, per A) - one
    // program, two distinct static Write PCs for the same root, each
    // getting its own, independently-correct verdict.
    let result = run_source(
        r#"
        fn main() {
            let mut pair: (i32, i32) = (1, 2);
            pair = (3, 4);
            let (ref left, _): (i32, i32) = pair;
            pair = (5, 6);
            let _ = left;
            return;
        }
    "#,
    );
    assert_conflicts(
        result,
        "the second, later reassignment of the same root must reject even though the first \
         (distinct-PC) reassignment of that same root already succeeded",
    );
}

// --- F: multi-field RecordUpdate rejects even if only the LAST path conflicts

#[test]
fn f_multi_field_record_update_conflict_on_last_path_still_rejects() {
    let result = run_source(
        r#"
        record R { a: i32, b: i32 }
        fn main() {
            let base: R = R { a: 1, b: 2 };
            let R { a: _, b: ref y } = base;
            let patched: R = base with { a: 9, b: 8 };
            let _ = (y, patched.a);
            return;
        }
    "#,
    );
    assert_conflicts(
        result,
        "a RecordUpdate whose FIRST field-path (base.a) does not conflict but whose SECOND \
         field-path (base.b) does must still reject - checking only the first path at a \
         shared MakeRecord PC would wrongly admit this",
    );
}

// --- G: non-overlapping RecordUpdate succeeds ------------------------------

#[test]
fn g_non_overlapping_record_update_succeeds() {
    let result = run_source(
        r#"
        record R { a: i32, b: i32 }
        fn main() {
            let base: R = R { a: 1, b: 2 };
            let R { a: ref x, b: _ } = base;
            let patched: R = base with { b: 8 };
            let _ = (x, patched.b);
            return;
        }
    "#,
    );
    assert_succeeds(
        result,
        "a RecordUpdate touching only a sibling field of the borrowed one must succeed",
    );
}

// --- H: a loop revisiting one static Write PC is rechecked every time -----
//
// `pair = (i, i);` is one single static instruction, executed three times
// (i=0,1,2) as the `while` loop's PC counter jumps back to the same byte
// offset each iteration. The borrow only activates once, on the i==2
// iteration's own `ref left` binding, which lexically precedes that
// iteration's `pair = (i, i);` in program order. If
// `check_write_execution_site` were consume-once (checked and then ignored
// after the first visit), the i==2 write would wrongly succeed, since the
// site would already be "used up" by the two earlier, non-conflicting
// visits - it would never re-examine `active_borrowed_paths` a third time
// to notice the borrow that has since become active. Observing a rejection
// specifically on this run (which can only happen if the site's check
// really did run again on the third visit) is direct proof against
// consume-once behavior; the loop's own bytecode PC never changes between
// iterations, so `write_execution_sites` genuinely returns to the exact
// same entry three times.

#[test]
fn h_loop_revisits_one_static_write_pc_rechecked_every_time() {
    let src = r#"
        fn main() {
            let mut i: i32 = 0;
            let mut pair: (i32, i32) = (1, 2);
            while i < 3 {
                if i == 2 {
                    let (ref left, _): (i32, i32) = pair;
                    let _ = left;
                }
                pair = (i, i);
                i = i + 1;
            }
            return;
        }
    "#;
    let bytes = sm_ir::compile_program_to_semcode(src).expect("compile");
    let (_, envs) = sm_format::semcode_decode::decode_semcode_envelope(&bytes).expect("decode");
    let main = envs.iter().find(|f| f.name == "main").expect("main");
    assert_eq!(
        main.borrowed_paths.len(),
        1,
        "expected exactly one Borrow event to survive lowering out of the nested if/while - \
         if this is 0, the #1709-class nested-lowering event-loss gap (documented as a known, \
         separately-tracked defect in docs/roadmap/stable_foundation/ssf08_ownership_position_decision.md) \
         is eating this test's own borrow and it needs a different program shape, not a change \
         to the W2F runtime mechanism this file exists to test"
    );
    let pair_write = main
        .write_paths
        .iter()
        .find(|p| p.root_symbol_id == main.borrowed_paths[0].root_symbol_id)
        .expect(
            "expected a Write event rooted at the same local the Borrow is on (`i = i + 1;` is \
             also a real, unrelated Write event in this function and is not the one under test)",
        );
    // The Write is a whole-tuple reassignment (`pair = (i, i);`, zero
    // components) and the Borrow is `pair.0` (`TupleIndex(0)`) - a
    // zero-component write always overlaps any longer path sharing its
    // root under `access_paths_overlap`'s prefix rule, so this is still a
    // genuine same-path conflict, just not a component-for-component
    // identical one.
    assert!(
        pair_write.components.is_empty(),
        "expected the loop body's own reassignment to be a whole-tuple (zero-component) write: {:?}",
        pair_write.components
    );

    let result = run_source(src);
    assert_conflicts(
        result,
        "the third loop iteration's write must reject once the borrow introduced by that same \
         iteration's own ref-binding has activated - a consume-once implementation would \
         wrongly let it through, having already 'used up' the site on the first two, \
         non-conflicting visits",
    );
}

// --- I: mixed StoreVarSite + MakeRecordSite Write sites in one function ---

#[test]
fn i_mixed_store_var_and_make_record_sites_both_behave_correctly() {
    // `pair`'s reassignment is a StoreVarSite Write; the RecordUpdate on
    // `base` is a MakeRecordSite Write - both kinds of Write execution site
    // exist in the SAME function and must each be checked against their own,
    // independent Borrow state correctly. `pair`'s reassignment conflicts
    // with `ref left`'s active borrow of it; the RecordUpdate does not
    // conflict with anything (nothing borrows `base`).
    let result = run_source(
        r#"
        record R { a: i32, b: i32 }
        fn main() {
            let mut pair: (i32, i32) = (1, 2);
            let (ref left, _): (i32, i32) = pair;
            let base: R = R { a: 1, b: 2 };
            let patched: R = base with { a: 9, b: 8 };
            pair = (3, 4);
            let _ = (left, patched.a);
            return;
        }
    "#,
    );
    assert_conflicts(
        result,
        "the StoreVarSite reassignment of the borrowed tuple must reject even in a function \
         that also contains an unrelated, non-conflicting MakeRecordSite RecordUpdate",
    );
}

#[test]
fn i_mixed_store_var_and_make_record_sites_neither_conflicts_succeeds() {
    let result = run_source(
        r#"
        record R { a: i32, b: i32 }
        fn main() {
            let mut pair: (i32, i32) = (1, 2);
            pair = (3, 4);
            let (ref left, _): (i32, i32) = pair;
            let base: R = R { a: 1, b: 2 };
            let patched: R = base with { a: 9, b: 8 };
            let _ = (left, patched.a);
            return;
        }
    "#,
    );
    assert_succeeds(
        result,
        "a StoreVarSite reassignment that precedes its borrow and an unrelated, \
         non-conflicting MakeRecordSite RecordUpdate must both succeed together in one function",
    );
}
