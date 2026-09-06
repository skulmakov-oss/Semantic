// #1891 forensic/repro checkpoint (2026-09-06). Reproduces, on the current
// tree (post-#1726 Checkpoints A-D3), the live `next_write_path` false-
// negative: `sm-vm`'s write-conflict check
// (`crates/sm-vm/src/semcode_vm.rs`, `Opcode::StoreVar` handler) walks
// `FunctionBytecode.write_paths` with a single sequential cursor
// (`Frame.next_write_path`), advancing it only when the *current* StoreVar's
// own target symbol equals `write_paths[cursor].root`. This assumes every
// statically-emitted Write event corresponds, in order, to a StoreVar that
// actually targets that event's own root symbol at runtime. Both scenarios
// below break that assumption and produce a live false negative: a write
// that genuinely overlaps an active Borrow is silently let through because
// the cursor never reaches (or is stuck before) the entry that would have
// caught it. This file does not fix anything - #1726's own runtime
// (Checkpoint D3, PC-exact Borrow activation) is proven correct and
// unrelated; the defect is entirely in this separate, older write-cursor
// mechanism. Each "poisoned" test's `expect_err`/`Ok` assertion documents
// the CURRENT (defective) behavior on purpose, so that fixing #1891 will
// require flipping these assertions - their failure post-fix is the signal
// the fix worked, not a regression to chase down confused.
//
// This file asserts today's actual (defective) behavior, not the desired
// one, so it stays green under `cargo test --workspace` alongside the
// #1726 qualification suite without being part of that suite's own
// checklist. It exists solely to ground #1891 in fresh, evidenced
// reproductions on the current tree before any fix is designed.

use sm_runtime_core::RuntimeTrap;
use sm_vm::RuntimeError;

fn run_source(source: &str) -> Result<(), RuntimeError> {
    let bytes = sm_ir::compile_program_to_semcode(source).expect("compile");
    let token = sm_verify::verify_semcode_token(&bytes).expect("admit");
    let entry = token.require_entry("main").expect("entry");
    sm_vm::run_verified_entry_semcode(&entry)
}

// --- Scenario 1: record-update Write poisons next_write_path -------------
//
// `base with { a: .., b: .. }` (a record-update expression) is lowered by
// `append_record_update_write_events_from_expr`
// (crates/sm-ir/src/legacy_lowering.rs) into one Write event PER UPDATED
// FIELD, rooted at the update's *base* expression (`base`) - not at
// whatever new binding receives the update's result (`patched`). No
// StoreVar anywhere ever targets `base` as part of evaluating this
// expression (`base` is only read; `patched` is a fresh introduction, so
// `frame.locals.contains_key(&symbol)` is false for its own StoreVar and
// the write-check never even looks at it). The write-check's own gating
// condition - "the *current* StoreVar's target symbol equals the pending
// write path's root" - can therefore only ever fire for these entries if
// `base` happens to be reassigned again later, for an entirely unrelated
// reason. Confirmed by direct decode inspection (not assumed): both
// `write_paths` entries below genuinely exist, rooted at the exact same
// symbol and field the Borrow is active on.

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
fn defect_record_update_after_borrow_is_silently_accepted() {
    // `base.a` is borrowed via `ref x`, then a record-update reads `base`
    // to build `patched`, declaring (via its own emitted Write events) a
    // conceptual write to `base.a` and `base.b` - the same field the
    // borrow is active on. This must conflict under the same rule the
    // control above proves. It currently does not: `base` is never itself
    // reassigned, so `next_write_path`'s cursor never advances past
    // position 0, and the check never runs at all for this Write pair.
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
        result.is_ok(),
        "DEFECT (#1891) NOT REPRODUCED - a fix may already be in place, or this \
         program no longer exercises the write-cursor desync; investigate before \
         reusing this test as a fix-verification regression: {result:?}"
    );
}

// --- Scenario 2: untaken-branch static Write poisons next_write_path -----
//
// Two `write_paths` entries exist in this program - one per source-level
// reassignment, in program order - regardless of whether either branch is
// ever taken at runtime. If the FIRST one (by static/emission order) sits
// inside a branch that is never taken, `next_write_path`'s cursor never
// advances past it (nothing ever executes a StoreVar targeting that
// entry's own root), permanently blocking every later, genuinely-executed
// write whose own root differs from that stuck entry's root: the filter
// `write_paths[cursor].root == symbol` fails for the real write too, so its
// check is skipped entirely - not merely deferred.

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
fn defect_untaken_branch_write_poisons_a_later_unrelated_conflicting_write() {
    // Identical to the control above except `if true` -> `if false`: the
    // `other = (7, 7)` write never executes. The later `pair = (3, 4)`
    // write genuinely overlaps `left`'s active borrow of `pair.0` and must
    // reject under the same rule the control proves - it currently does
    // not, because the cursor is stuck on the untaken write's own
    // (different-rooted) entry.
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
        result.is_ok(),
        "DEFECT (#1891) NOT REPRODUCED - a fix may already be in place, or this \
         program no longer exercises the write-cursor desync; investigate before \
         reusing this test as a fix-verification regression: {result:?}"
    );
}
