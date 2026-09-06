use sm_emit::compile_program_to_semcode;
use sm_runtime_core::RuntimeTrap;
use sm_vm::RuntimeError;

fn run_source(source: &str) -> Result<(), RuntimeError> {
    let bytes = compile_program_to_semcode(source).expect("compile");
    let token = sm_verify::verify_semcode_token(&bytes).expect("admit");
    let entry = token.require_entry("main").expect("entry");
    sm_vm::run_verified_entry_semcode(&entry)
}

#[test]
fn write_before_borrow_succeeds() {
    run_source(
        r#"
        fn main() {
            let mut pair: (i32, i32) = (1, 2);
            pair = (3, 4);
            let (ref left, _) = pair;
            assert(left == 3);
            return;
        }
    "#,
    )
    .expect("write precedes activation");
}

#[test]
fn branch_execution_controls_borrow_activation() {
    for taken in [false, true] {
        let source = format!(
            r#"
            fn main() {{
                let mut pair: (i32, i32) = (1, 2);
                if {taken} {{ let (ref left, _) = pair; }}
                pair = (3, 4);
                return;
            }}
        "#
        );
        let result = run_source(&source);
        if taken {
            assert!(matches!(
                result,
                Err(RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict))
            ));
        } else {
            result.expect("untaken borrow must remain pending");
        }
    }
}

// #1726 Checkpoint D3 replay (test-only fix, stale-test reconciliation):
// this test used to assert that O1 must preserve the dead introduction's
// StoreVar target alongside its ownership event - the opposite of the
// already-accepted Checkpoint C contract for this exact source
// (`ssf08_1726_checkpoint_c_unreachable_borrow_introduction_removed_coherently`,
// crates/sm-ir/src/legacy_lowering.rs), which coherently removes an
// unreachable annotated StoreVar AND its paired Borrow event together, not
// preserving one while dropping the other (which would leave an orphaned
// activation site on whichever side survived). Renamed and rewritten to
// prove the accepted invariant instead of the stale pre-Checkpoint-C
// expectation. The original counterexample source is unchanged, so this
// remains a real regression against the same program.
#[test]
fn optimizer_removes_unreachable_borrow_and_anchor_coherently() {
    use sm_ir::{
        compile_program_to_ir_with_options, emit_ir_to_semcode, CompileProfile, IrInstr, OptLevel,
    };
    let source = r#"
        fn main() {
            let pair: (i32, i32) = (1, 2);
            return;
            let (ref left, _): (i32, i32) = pair;
        }
    "#;
    let o0 = compile_program_to_ir_with_options(source, CompileProfile::RustLike, OptLevel::O0)
        .expect("O0 IR");
    let o1 = compile_program_to_ir_with_options(source, CompileProfile::RustLike, OptLevel::O1)
        .expect("O1 IR");

    // O0 (no cleanup pass runs): the dead introduction and its paired
    // Borrow event may still exist before cleanup.
    assert_eq!(o0[0].ownership_events.len(), 1);
    let dead_target = o0[0]
        .instrs
        .iter()
        .find_map(|i| match i {
            IrInstr::StoreVar {
                name,
                activation_site: Some(_),
                ..
            } if name.ends_with("_left") => Some(name),
            _ => None,
        })
        .expect("O0 annotated StoreVar(left)");

    // O1 (Checkpoint C's coherent removal): the annotated dead StoreVar is
    // gone, and its paired Borrow event is gone with it - never one without
    // the other, on either side.
    assert!(
        o1[0].ownership_events.is_empty(),
        "Checkpoint C must remove the unreachable Borrow's paired event along \
         with its StoreVar, not preserve it: {:?}",
        o1[0].ownership_events
    );
    assert!(
        !o1[0]
            .instrs
            .iter()
            .any(|i| matches!(i, IrInstr::StoreVar { name, .. } if name == dead_target)),
        "the unreachable annotated StoreVar itself must be gone from O1"
    );
    assert!(
        o1[0].instrs.iter().all(|i| !matches!(
            i,
            IrInstr::StoreVar {
                activation_site: Some(_),
                ..
            }
        )),
        "no orphan activation site may remain in O1: {:?}",
        o1[0].instrs
    );

    // Both optimization levels must still compile, verify, and run
    // correctly end to end - the dead code is never executed either way.
    for (level, ir) in [("O0", &o0), ("O1", &o1)] {
        let bytes = emit_ir_to_semcode(ir, false).expect("baseline emission");
        let token = sm_verify::verify_semcode_token(&bytes).expect("baseline admission");
        let entry = token.require_entry("main").expect("entry");
        sm_vm::run_verified_entry_semcode(&entry).expect("baseline execution");
        eprintln!(
            "{level}: events={:?}, stores={:?}",
            ir[0].ownership_events,
            ir[0]
                .instrs
                .iter()
                .filter_map(|i| match i {
                    IrInstr::StoreVar { name, .. } => Some(name),
                    _ => None,
                })
                .collect::<Vec<_>>()
        );
    }
}

// #1726 Checkpoint D3, item 12.B: once the introducing StoreVar has
// executed, the Borrow is active - a later overlapping write must reject.
#[test]
fn post_activation_write_conflicts() {
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
    assert!(
        matches!(
            result,
            Err(RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict))
        ),
        "a write after the borrow's own introduction has executed must conflict: {result:?}"
    );
}

// #1726 Checkpoint D3, item 12.E / item 5: only the introduction's own
// StoreVar PC may activate the Borrow. Reassigning the bound name afterward
// (a different PC, same lowered-local identity) must not itself do
// anything - observed here by the Borrow remaining correctly active (from
// the introduction) across that reassignment, still rejecting a later
// write to the borrowed root.
#[test]
fn only_introduction_pc_activates_not_reassignment() {
    let result = run_source(
        r#"
        fn main() {
            let mut pair: (i32, i32) = (1, 2);
            let (ref left, _): (i32, i32) = pair;
            left = 99;
            assert(left == 99);
            pair = (3, 4);
            return;
        }
    "#,
    );
    assert!(
        matches!(result, Err(RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict))),
        "the borrow must remain active (from its own introduction) across the target's own reassignment: {result:?}"
    );
}

// #1726 Checkpoint D3, item 12.F: a static anchor skipped on an earlier loop
// iteration must still activate the first time it is actually visited.
#[test]
fn loop_activates_on_first_executed_visit_not_merely_because_the_anchor_exists() {
    let result = run_source(
        r#"
        fn main() {
            let mut pair: (i32, i32) = (1, 2);
            let mut i: i32 = 0;
            while i < 2 {
                if i == 1 {
                    let (ref left, _): (i32, i32) = pair;
                    let _ = left;
                }
                i = i + 1;
            }
            pair = (3, 4);
            return;
        }
    "#,
    );
    assert!(
        matches!(
            result,
            Err(RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict))
        ),
        "activation on a later loop iteration must still be observed: {result:?}"
    );
}

// #1726 Checkpoint D3, item 12.F: the same static anchor executing on every
// iteration must activate once and then be harmless (no error, no
// re-activation side effect) on every later visit.
#[test]
fn loop_repeated_anchor_visits_are_harmless() {
    run_source(
        r#"
        fn main() {
            let pair: (i32, i32) = (1, 2);
            let mut i: i32 = 0;
            while i < 3 {
                let (ref left, _): (i32, i32) = pair;
                let _ = left;
                i = i + 1;
            }
            return;
        }
    "#,
    )
    .expect("repeated visits to the same anchor must stay harmless");
}

// #1726 Checkpoint D3, item 12.G: an ADT/Option `FrameEntry` Borrow and a
// Tuple `StoreVarSite` Borrow coexisting in one rev21 function each keep
// their own explicit mode - FrameEntry stays eager, StoreVarSite stays lazy.
#[test]
fn mixed_frame_entry_and_store_var_site_activation_in_one_function() {
    run_source(
        r#"
        fn main() {
            let opt: Option(i32) = Option::Some(1);
            let extracted: i32 = match opt {
                Option::Some(ref value) => { value }
                Option::None => { 0 }
            };
            let mut pair: (i32, i32) = (1, 2);
            pair = (3, 4);
            let (ref left, _): (i32, i32) = pair;
            assert(left == 3);
            assert(extracted == 1);
            return;
        }
    "#,
    )
    .expect(
        "a lazy StoreVarSite borrow must still behave lazily alongside an eager FrameEntry borrow",
    );

    let result = run_source(
        r#"
        fn main() {
            let mut opt: Option(i32) = Option::Some(1);
            let extracted: i32 = match opt {
                Option::Some(ref value) => { value }
                Option::None => { 0 }
            };
            let mut pair: (i32, i32) = (1, 2);
            pair = (3, 4);
            let (ref left, _): (i32, i32) = pair;
            opt = Option::Some(9);
            assert(left == 3);
            assert(extracted == 1);
            return;
        }
    "#,
    );
    assert!(
        matches!(result, Err(RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict))),
        "the FrameEntry (ADT) borrow must still be eager/active from frame entry even alongside a lazy StoreVarSite borrow in the same function: {result:?}"
    );
}

// #1726 Checkpoint D3, item 9 / item 12.H: a Borrow event whose own producer
// (ADT/Option) never allocates an ActivationSiteId and so never needs rev21
// for its own sake keeps exactly its pre-D3 eager/frame-entry runtime
// behavior - conflicting regardless of which match arm actually ran. (As of
// #1891 Checkpoint W2D, this specific program's own reassignment separately
// promotes it to rev21 anyway, on the Write side - orthogonal to, and
// without affecting, the Borrow-side claim this test proves.)
#[test]
fn rev20_legacy_borrow_stays_eager_regardless_of_which_arm_ran() {
    let bytes = compile_program_to_semcode(
        r#"
        fn main() {
            let mut opt: Option(i32) = Option::Some(1);
            let extracted: i32 = match opt {
                Option::Some(ref value) => { value }
                Option::None => { 0 }
            };
            opt = Option::Some(9);
            let _ = extracted;
            return;
        }
    "#,
    )
    .expect("compile");
    // #1891 Checkpoint W2D: `opt = Option::Some(9);` is a plain reassignment
    // (producer B), which now always carries a resolved WriteSiteId
    // (Checkpoint W2C) - promoting this artifact to SEMCOD20/rev21 on its
    // own merits, independent of this test's actual subject (the ADT/Option
    // Borrow producer, which still never allocates an ActivationSiteId and
    // still never needs rev21 for its own sake). The behavioral claim below
    // - legacy FrameEntry Borrow stays eager regardless of which match arm
    // ran - is what this test exists to prove, and is unaffected by the
    // Write-side promotion.
    let token = sm_verify::verify_semcode_token(&bytes).expect("admit");
    let entry = token.require_entry("main").expect("entry");
    let result = sm_vm::run_verified_entry_semcode(&entry);
    assert!(
        matches!(
            result,
            Err(RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict))
        ),
        "legacy FrameEntry Borrow must remain eager/active from frame entry: {result:?}"
    );
}
