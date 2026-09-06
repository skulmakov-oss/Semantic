// #1888 (FA-04-025) reconciliation, SSF-08 closure audit follow-up
// (2026-09-06). #1888's original finding: `lower_for_range_stmt`,
// `lower_for_each_stmt`, and `Stmt::Guard`'s `else_return` payload never
// called `append_record_update_write_events_from_expr` (the pre-#1891
// prescan that was, at the time, the *only* mechanism producing a
// RecordUpdate's `Write(Field)` ownership events), so a RecordUpdate
// reachable only through one of those three roots could lower to a real,
// executable `MakeRecord` with no Write ownership event describing it at
// all - a genuine fail-open gap in the ownership model at filing time.
//
// #1891 Checkpoint W2A (`crates/sm-ir/src/legacy_lowering.rs`,
// `lower_expr_with_expected`'s own `Expr::RecordUpdate` arm) relocated
// Write-event generation for RecordUpdate out of the separate prescan
// entirely: the events are now minted inline, in the same lowering step
// that emits the real `MakeRecord`, by the ONE canonical expression-lowering
// function every admitted expression - regardless of which statement-level
// caller reaches it - must pass through to become executable IR at all
// (`lower_expr` is a thin wrapper around `lower_expr_with_expected`;
// confirmed by direct reading, not by trusting the production comment
// alone). `append_record_update_write_events_from_expr`'s own
// `Expr::RecordUpdate` arm no longer emits anything for the RecordUpdate it
// visits - it only continues recursing into `base` and field values for
// *other* ownership-producing constructs that might be nested inside them.
//
// This file proves, empirically, that the ORIGINAL #1888 defect no longer
// reproduces through any of its three named roots - not because the
// prescan was patched to also visit these roots (it was not), but because
// the mechanism that made the prescan necessary for RecordUpdate has been
// structurally eliminated. A temporary mutation-proof test (cloning a real
// `IrFunction`, clearing `ownership_events`, and confirming this file's own
// `assert_record_update_write_site_correlated` panics via `catch_unwind` -
// zero production code touched) confirmed that check genuinely
// discriminates presence from absence rather than passing vacuously; it was
// deleted after confirming the discrepancy, per this session's established
// discipline for temporary mutation proofs.

use sm_ir::{IrFunction, IrInstr, OwnershipPathEventKind};
use sm_runtime_core::RuntimeTrap;
use sm_vm::RuntimeError;

fn ir_main(src: &str) -> IrFunction {
    let ir = sm_ir::compile_program_to_ir(src).expect("compile to IR");
    ir.into_iter()
        .find(|f| f.name == "main")
        .expect("main function present in IR")
}

/// Asserts the IR-level Producer C proof required by item 5: exactly one
/// `MakeRecord` carrying a `write_site`, exactly that many matching
/// `Write` ownership events sharing the identical `WriteSiteId`, correlated
/// by identity - never merely counted globally.
fn assert_record_update_write_site_correlated(f: &IrFunction) {
    let make_record_sites: Vec<_> = f
        .instrs
        .iter()
        .filter_map(|instr| match instr {
            IrInstr::MakeRecord {
                write_site: Some(w),
                ..
            } => Some(*w),
            _ => None,
        })
        .collect();
    assert_eq!(
        make_record_sites.len(),
        1,
        "expected exactly one MakeRecord carrying a real write_site (the RecordUpdate's own \
         commit point): {:?}",
        f.instrs
    );
    let site = make_record_sites[0];

    let write_events: Vec<_> = f
        .ownership_events
        .iter()
        .filter(|e| e.kind == OwnershipPathEventKind::Write)
        .collect();
    assert!(
        !write_events.is_empty(),
        "expected at least one Write ownership event - this is exactly the #1888 failure mode: \
         a real MakeRecord with no describing Write event at all"
    );
    for event in &write_events {
        assert_eq!(
            event.write_site,
            Some(site),
            "every Write event must carry the SAME WriteSiteId as the MakeRecord it describes - \
             not merely 'some write_site exists somewhere'"
        );
    }
}

fn run_source(source: &str) -> Result<(), RuntimeError> {
    let bytes = sm_ir::compile_program_to_semcode(source).expect("compile");
    let token = sm_verify::verify_semcode_token(&bytes).expect("admit");
    let entry = token.require_entry("main").expect("entry");
    sm_vm::run_verified_entry_semcode(&entry)
}

// --- A: RecordUpdate reachable only through a for-range bound ------------

fn for_range_record_update_source() -> &'static str {
    r#"
        record R { a: i32, b: i32 }
        fn main() {
            let base: R = R { a: 1, b: 2 };
            for i in 0..(base with { a: 9 }).a {
                let _ = i;
            }
            return;
        }
    "#
}

#[test]
fn a_for_range_bound_record_update_reaches_lowering() {
    // Item 4: prove the fixture actually reaches lowering (parser + type
    // checker accept it, IR is produced), not merely that a source string
    // parses.
    let f = ir_main(for_range_record_update_source());
    assert!(
        f.instrs
            .iter()
            .any(|i| matches!(i, IrInstr::MakeRecord { .. })),
        "expected the for-range bound's RecordUpdate to actually lower to a MakeRecord"
    );
}

#[test]
fn a_for_range_bound_record_update_produces_correlated_write_event() {
    let f = ir_main(for_range_record_update_source());
    assert_record_update_write_site_correlated(&f);
}

// --- B: RecordUpdate reachable only through a for-each iterable ----------

fn for_each_record_update_source() -> &'static str {
    r#"
        record R { items: Sequence(i32) }
        fn main() {
            let base: R = R { items: [1, 2, 3] };
            for item in (base with { items: [4, 5, 6] }).items {
                let _ = item;
            }
            return;
        }
    "#
}

#[test]
fn b_for_each_iterable_record_update_reaches_lowering() {
    let f = ir_main(for_each_record_update_source());
    assert!(
        f.instrs
            .iter()
            .any(|i| matches!(i, IrInstr::MakeRecord { .. })),
        "expected the for-each iterable's RecordUpdate to actually lower to a MakeRecord"
    );
}

#[test]
fn b_for_each_iterable_record_update_produces_correlated_write_event() {
    let f = ir_main(for_each_record_update_source());
    assert_record_update_write_site_correlated(&f);
}

// --- C: RecordUpdate reachable only through a guard else-return payload --

fn guard_else_return_record_update_source() -> &'static str {
    r#"
        record R { a: i32, b: i32 }
        fn make_or_default(base: R) -> R {
            guard base.a > 0 else return base with { a: 99 };
            return base;
        }
        fn main() {
            let base: R = R { a: -1, b: 2 };
            let result: R = make_or_default(base);
            let _ = result.a;
            return;
        }
    "#
}

#[test]
fn c_guard_else_return_record_update_reaches_lowering() {
    let ir = sm_ir::compile_program_to_ir(guard_else_return_record_update_source())
        .expect("compile to IR");
    let f = ir
        .into_iter()
        .find(|f| f.name == "make_or_default")
        .expect("make_or_default present in IR");
    assert!(
        f.instrs
            .iter()
            .any(|i| matches!(i, IrInstr::MakeRecord { .. })),
        "expected the guard else-return payload's RecordUpdate to actually lower to a MakeRecord"
    );
    // Stash for the next test via re-compile rather than sharing state -
    // deliberately re-derives rather than caching, matching this session's
    // established discipline of never trusting decode results across
    // separate assertions without re-verifying.
}

#[test]
fn c_guard_else_return_record_update_produces_correlated_write_event() {
    let ir = sm_ir::compile_program_to_ir(guard_else_return_record_update_source())
        .expect("compile to IR");
    let f = ir
        .into_iter()
        .find(|f| f.name == "make_or_default")
        .expect("make_or_default present in IR");
    assert_record_update_write_site_correlated(&f);
}

// --- Downstream chain proof: source -> IR -> exact MakeRecord PC -> rev21
// --- OWN0 -> verifier admission -> VM Write check at the real MakeRecord PC

#[test]
fn downstream_chain_for_range_record_update_conflicts_with_active_borrow() {
    // Same shape as fixture A, but with a real, active Borrow of `base.a` in
    // scope when the for-range bound's RecordUpdate executes - proving the
    // Write event this mechanism produces is not just present in metadata
    // but actually enforced at runtime, at the exact MakeRecord PC, exactly
    // like every other Checkpoint W2F Producer C site.
    let src = r#"
        record R { a: i32, b: i32 }
        fn main() {
            let base: R = R { a: 1, b: 2 };
            let R { a: ref x, b: _ } = base;
            for i in 0..(base with { a: 9 }).a {
                let _ = i;
            }
            let _ = x;
            return;
        }
    "#;
    let result = run_source(src);
    assert!(
        matches!(
            result,
            Err(RuntimeError::Trap(RuntimeTrap::BorrowWriteConflict))
        ),
        "the for-range bound's RecordUpdate writes base.a while base.a is actively borrowed - \
         this must reject exactly like any other RecordUpdate Write site: {result:?}"
    );
}

#[test]
fn downstream_chain_for_range_record_update_no_conflict_succeeds() {
    // Companion control: same shape, no active borrow at all - must succeed,
    // proving the rejection above is a real conflict detection, not an
    // unconditional trap on this fixture shape.
    let result = run_source(for_range_record_update_source());
    assert!(
        result.is_ok(),
        "with no active borrow, the for-range bound's RecordUpdate must succeed: {result:?}"
    );
}
