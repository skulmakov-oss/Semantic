// #1891 forensic/repro checkpoint (2026-09-06), updated at Checkpoint W2F
// (2026-09-06) once the fix landed. Originally reproduced the live
// `next_write_path` false-negative: `sm-vm`'s write-conflict check
// (`crates/sm-vm/src/semcode_vm.rs`, `Opcode::StoreVar` handler) walked
// `FunctionBytecode.write_paths` with a single sequential cursor
// (`Frame.next_write_path`), advancing it only when the *current* StoreVar's
// own target symbol equalled `write_paths[cursor].root`. That assumed every
// statically-emitted Write event corresponded, in order, to a StoreVar that
// actually targeted that event's own root symbol at runtime. Both scenarios
// below broke that assumption and produced a live false negative: a write
// that genuinely overlapped an active Borrow was silently let through
// because the cursor never reached (or was stuck before) the entry that
// would have caught it.
//
// Checkpoint W2F replaced the cursor entirely with an exact-PC lookup
// (`FunctionBytecode.write_execution_sites`, grouped by the same
// verifier-authenticated anchor Checkpoints W2D/W2E establish and check on
// every instruction dispatch, not just the "next expected" one) - so both
// scenarios below now correctly reject. The two `defect_*` tests have been
// renamed to `fixed_*` and their assertions flipped from `Ok(())` to
// `Err(BorrowWriteConflict)`; the two `control_*` tests are unchanged - they
// already conflicted correctly before W2F and still do. This file keeps its
// original name and source counterexamples so it remains the canonical
// #1891 regression suite: 4/4 green, all four now encoding the correct
// runtime semantics rather than three correct + one documented-defective.

use sm_runtime_core::RuntimeTrap;
use sm_vm::RuntimeError;

fn run_source(source: &str) -> Result<(), RuntimeError> {
    let bytes = sm_ir::compile_program_to_semcode(source).expect("compile");
    let token = sm_verify::verify_semcode_token(&bytes).expect("admit");
    let entry = token.require_entry("main").expect("entry");
    sm_vm::run_verified_entry_semcode(&entry)
}

// --- Scenario 1: record-update Write, formerly poisoned next_write_path --
//
// `base with { a: .., b: .. }` (a record-update expression) is lowered by
// `append_record_update_write_events_from_expr`
// (crates/sm-ir/src/legacy_lowering.rs) into one Write event PER UPDATED
// FIELD, rooted at the update's *base* expression (`base`) - not at
// whatever new binding receives the update's result (`patched`). No
// StoreVar anywhere ever targets `base` as part of evaluating this
// expression (`base` is only read; `patched` is a fresh introduction). The
// old cursor's gating condition - "the *current* StoreVar's target symbol
// equals the pending write path's root" - could therefore only ever fire
// for these entries if `base` happened to be reassigned again later, for an
// entirely unrelated reason; it never was here, so the check never ran at
// all for this Write pair. Checkpoint W2F's exact-PC lookup has no such
// gate: each field-override Write event's real `MakeRecordSite` PC is
// checked the moment that MakeRecord instruction executes, regardless of
// whether `base` itself is ever reassigned. Confirmed by direct decode
// inspection (not assumed): both `write_paths` entries below genuinely
// exist, rooted at the exact same symbol and field the Borrow is active on.

#[test]
fn control_plain_record_field_write_after_borrow_conflicts() {
    // No record-update involved: a plain, direct reassignment of the
    // borrowed record after the borrow is active must reject, and does.
    let result = run_source(
        r#"
        record R { a: i32, b: i32 }
        fn main() {
            let mut base: R = R { a: 1, b: 2 };
            let R { a: ref x, b: _ } = base;
            base = R { a: 9, b: 8 };
            let _ = x;
            return;
        }
    "#,
    );
    assert!(
        matches!(
            result,
            Err(RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict))
        ),
        "control must conflict (no record-update involved): {result:?}"
    );
}

#[test]
fn fixed_record_update_after_borrow_now_rejects() {
    // `base.a` is borrowed via `ref x`, then a record-update reads `base`
    // to build `patched`, declaring (via its own emitted Write events) a
    // conceptual write to `base.a` and `base.b` - the same field the
    // borrow is active on. This must conflict under the same rule the
    // control above proves, and now does: Checkpoint W2F's exact-PC lookup
    // checks this MakeRecord's real anchor regardless of whether `base` is
    // ever itself reassigned.
    let src = r#"
        record R { a: i32, b: i32 }
        fn main() {
            let mut base: R = R { a: 1, b: 2 };
            let R { a: ref x, b: _ } = base;
            let patched: R = base with { a: 9, b: 8 };
            let _ = (x, patched.a);
            return;
        }
    "#;

    let bytes = sm_ir::compile_program_to_semcode(src).expect("compile");
    let (_, envs) = sm_format::semcode_decode::decode_semcode_envelope(&bytes).expect("decode");
    let main = envs.iter().find(|f| f.name == "main").expect("main");
    assert_eq!(
        main.borrowed_paths.len(),
        1,
        "expected exactly one Borrow event"
    );
    assert_eq!(
        main.write_paths.len(),
        2,
        "expected one Write event per updated field"
    );
    assert_eq!(
        main.borrowed_paths[0].root_symbol_id, main.write_paths[0].root_symbol_id,
        "the Borrow and the record-update's first Write event must share a root - \
         confirming this is a genuine same-field conflict, not a false alarm"
    );
    assert_eq!(
        main.borrowed_paths[0].components, main.write_paths[0].components,
        "the Borrow and the record-update's first Write event must name the exact \
         same field"
    );

    let result = run_source(src);
    assert!(
        matches!(
            result,
            Err(RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict))
        ),
        "#1891 regression: this record-update Write must reject exactly like the \
         control does - {result:?}"
    );
}

// --- Scenario 2: untaken-branch static Write, formerly poisoned next_write_path --
//
// Two `write_paths` entries exist in this program - one per source-level
// reassignment, in program order - regardless of whether either branch is
// ever taken at runtime. If the FIRST one (by static/emission order) sat
// inside a branch that was never taken, the old `next_write_path` cursor
// never advanced past it (nothing ever executed a StoreVar targeting that
// entry's own root), permanently blocking every later, genuinely-executed
// write whose own root differed from that stuck entry's root. Checkpoint
// W2F's exact-PC lookup has no cursor to get stuck: each Write event's own
// real StoreVar anchor is checked independently, keyed by PC, so an untaken
// branch's site is simply never visited and never consumes anything - it
// cannot block the later, genuinely-executed write's own independent check.

#[test]
fn control_taken_branch_write_before_conflicting_write_still_conflicts() {
    // Same shape as the defect below, but the first branch is TAKEN - the
    // cursor advances past it normally, and the second (real) write is
    // checked and correctly rejected.
    let result = run_source(
        r#"
        fn main() {
            let mut pair: (i32, i32) = (1, 2);
            let mut other: (i32, i32) = (5, 6);
            let (ref left, _): (i32, i32) = pair;
            if true {
                other = (7, 7);
            }
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
        "control must conflict (the earlier write's branch is taken, cursor advances \
         normally): {result:?}"
    );
}

#[test]
fn fixed_untaken_branch_write_no_longer_poisons_later_conflicting_write() {
    // Identical to the control above except `if true` -> `if false`: the
    // `other = (7, 7)` write never executes. The later `pair = (3, 4)`
    // write genuinely overlaps `left`'s active borrow of `pair.0` and must
    // reject under the same rule the control proves, and now does:
    // Checkpoint W2F never visits the untaken branch's own StoreVar PC, so
    // its Write site is simply never checked - it cannot poison anything.
    let src = r#"
        fn main() {
            let mut pair: (i32, i32) = (1, 2);
            let mut other: (i32, i32) = (5, 6);
            let (ref left, _): (i32, i32) = pair;
            if false {
                other = (7, 7);
            }
            pair = (3, 4);
            let _ = left;
            return;
        }
    "#;

    let bytes = sm_ir::compile_program_to_semcode(src).expect("compile");
    let (_, envs) = sm_format::semcode_decode::decode_semcode_envelope(&bytes).expect("decode");
    let main = envs.iter().find(|f| f.name == "main").expect("main");
    assert_eq!(
        main.write_paths.len(),
        2,
        "expected one Write event per reassignment"
    );
    assert_ne!(
        main.write_paths[0].root_symbol_id, main.write_paths[1].root_symbol_id,
        "the two write events must target different roots for this to isolate the \
         cursor desync, not merely a coincidentally-identical entry"
    );
    assert_eq!(
        main.borrowed_paths[0].root_symbol_id, main.write_paths[1].root_symbol_id,
        "the SECOND write event (the one that actually executes) must be the one \
         sharing a root with the active Borrow"
    );

    let result = run_source(src);
    assert!(
        matches!(
            result,
            Err(RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict))
        ),
        "#1891 regression: this later, genuinely-executed Write must reject exactly \
         like the control does - {result:?}"
    );
}
