// #1891 W1.5 (2026-09-06): design/proof evidence, not a production change.
//
// W1 initially classified `Expr::RecordUpdate`'s emitted `Write(base.field)`
// ownership events as an independent semantic defect (no physical mutation
// of `base` exists anywhere in the VM) and removed them experimentally. That
// removal was reverted: `docs/spec/runtime_ownership.md` (the frozen v0
// spec) turns out to already normatively name "direct record-field
// `Borrow`/`Write` transport" and "`Write(Field)` payloads" in its own
// Transport and Verifier Admission contracts (added deliberately by commit
// 01f06421 "emit: enable record field write ownership transport", which
// touched that frozen doc on purpose) - and record-update is the *only*
// language construct that can ever produce a `Write` event carrying a
// `Field` component (plain assignment only ever writes a whole binding, no
// field projection). So "direct record-field Write transport" in the frozen
// spec can only ever mean record-update's output. Multiple already-merged,
// "COMPLETE"-status repository contracts (the PCC4 record-field-ownership
// matrix, its 7hell-gated positive golden, capability-promotion tests, the
// #1709 regression suite, a CTF trust-freeze hash pin) depend on this
// producer's existence. The corrected classification: `RecordUpdate`'s
// `Write` is not physical-mutation tracking (it never claimed to be) - it is
// an intentional, already-normatively-adopted ownership-level effect, whose
// exact execution site was the open question this file answers.
//
// These 8 tests prove, empirically and across every shape item 2 of the
// W1.5 mandate names (single/multiple/all-overridden fields, nested update,
// branch, loop, call argument, reassignment to an existing binding), that
// every admitted `RecordUpdate` expression emits exactly one `MakeRecord`
// instruction, and that instruction is a 1:1 stand-in for that expression:
// distinct source occurrences (including a nested update, or one per
// branch) get distinct `MakeRecord`s; a single occurrence's N overridden
// fields all correspond to that same one `MakeRecord`, regardless of N; a
// `MakeRecord` inside a loop body is one static instruction, visited N times
// at runtime, never N static instructions. This holds structurally, not by
// coincidence: `Expr::RecordUpdate`'s own `lower_expr` arm pushes exactly
// one `MakeRecord` unconditionally at its end, and that arm is reached
// exactly once per AST node of this kind through the ordinary recursive
// lowering dispatch - the same reasoning that made `StoreVar` the correct
// Borrow/plain-assignment site in #1726 applies here with `MakeRecord` in
// its place. (First run of these probes initially showed unexpected counts
// one-per-test too high; root cause verified directly, not assumed:
// `MakeRecord` is also emitted by plain `RecordLiteral` construction, so
// every `R { .. }` literal in a test's own setup contributes its own
// MakeRecord alongside the update's - expected counts below already net
// that out.)
//
// This is the proof the W1.5 mandate asked for before any implementation:
// `WriteSiteId -> exact MakeRecord` is a sound correspondence for producer
// C, exactly as `WriteSiteId -> exact StoreVar` is for producers A/B. No
// production code was changed to establish this.

use sm_ir::{compile_program_to_ir_with_options, CompileProfile, IrInstr, OptLevel};

fn make_record_count(src: &str) -> usize {
    let ir = compile_program_to_ir_with_options(src, CompileProfile::RustLike, OptLevel::O0)
        .expect("compile");
    let main = ir.iter().find(|f| f.name == "main").expect("main");
    main.instrs
        .iter()
        .filter(|i| matches!(i, IrInstr::MakeRecord { .. }))
        .count()
}

fn write_event_count(src: &str) -> usize {
    let ir = compile_program_to_ir_with_options(src, CompileProfile::RustLike, OptLevel::O0)
        .expect("compile");
    let main = ir.iter().find(|f| f.name == "main").expect("main");
    main.ownership_events
        .iter()
        .filter(|e| e.kind == sm_ir::OwnershipPathEventKind::Write)
        .count()
}

#[test]
fn one_overridden_field() {
    let src = r#"
        record R { a: i32, b: i32 }
        fn main() {
            let base: R = R { a: 1, b: 2 };
            let fresh: R = base with { a: 9 };
            let _ = fresh;
            return;
        }
    "#;
    assert_eq!(
        make_record_count(src),
        2,
        "one MakeRecord for base's own literal, one for the update"
    );
    assert_eq!(write_event_count(src), 1);
}

#[test]
fn multiple_overridden_fields() {
    let src = r#"
        record R { a: i32, b: i32, c: i32 }
        fn main() {
            let base: R = R { a: 1, b: 2, c: 3 };
            let fresh: R = base with { a: 9, c: 7 };
            let _ = fresh;
            return;
        }
    "#;
    assert_eq!(
        make_record_count(src),
        2,
        "one MakeRecord for base's own literal, one for the update regardless of field count"
    );
    assert_eq!(write_event_count(src), 2);
}

#[test]
fn every_field_overridden() {
    let src = r#"
        record R { a: i32, b: i32 }
        fn main() {
            let base: R = R { a: 1, b: 2 };
            let fresh: R = base with { a: 9, b: 8 };
            let _ = fresh;
            return;
        }
    "#;
    assert_eq!(make_record_count(src), 2, "1 literal + 1 update");
    assert_eq!(write_event_count(src), 2);
}

#[test]
fn nested_record_update_gets_two_distinct_make_records() {
    let src = r#"
        record R { a: i32, b: i32 }
        fn main() {
            let base: R = R { a: 1, b: 2 };
            let inner: R = R { a: 5, b: 6 };
            let fresh: R = base with { a: (inner with { b: 9 }).a };
            let _ = fresh;
            return;
        }
    "#;
    assert_eq!(
        make_record_count(src),
        4,
        "2 literals (base, inner) + 2 updates - the outer and the nested inner \
         update must each get their own MakeRecord"
    );
    assert_eq!(
        write_event_count(src),
        2,
        "one write for the outer update's field, one for the inner's"
    );
}

#[test]
fn record_update_inside_branch() {
    let src = r#"
        record R { a: i32, b: i32 }
        fn main() {
            let base: R = R { a: 1, b: 2 };
            let cond: bool = true;
            let fresh: R = if cond {
                base with { a: 9 }
            } else {
                base with { b: 8 }
            };
            let _ = fresh;
            return;
        }
    "#;
    assert_eq!(
        make_record_count(src),
        3,
        "1 literal + 2 updates - each branch's own record-update gets its own \
         static MakeRecord"
    );
    assert_eq!(
        write_event_count(src),
        2,
        "one write event per branch's own update"
    );
}

#[test]
fn record_update_inside_loop() {
    let src = r#"
        record R { a: i32, b: i32 }
        fn main() {
            let base: R = R { a: 1, b: 2 };
            let mut i: i32 = 0;
            let mut last: R = base;
            while i < 3 {
                last = base with { a: i };
                i = i + 1;
            }
            let _ = last;
            return;
        }
    "#;
    assert_eq!(
        make_record_count(src),
        2,
        "1 literal + 1 static update - one static MakeRecord regardless of how \
         many times the loop body runs"
    );
    assert_eq!(
        write_event_count(src),
        3,
        "one static write event for the record-update field (regardless of loop \
         count), one for `last`'s own whole-value reassignment (producer B), \
         one for `i`'s own reassignment (also producer B) - three distinct \
         static Write events total, still each visited N times at runtime, \
         never N events"
    );
}

#[test]
fn record_update_as_call_argument() {
    let src = r#"
        record R { a: i32, b: i32 }
        fn sink(x: R) -> i32 = x.a;
        fn main() {
            let base: R = R { a: 1, b: 2 };
            let out: i32 = sink(base with { a: 9 });
            let _ = out;
            return;
        }
    "#;
    assert_eq!(make_record_count(src), 2, "1 literal + 1 update");
    assert_eq!(write_event_count(src), 1);
}

#[test]
fn record_update_assigned_to_existing_binding() {
    let src = r#"
        record R { a: i32, b: i32 }
        fn main() {
            let base: R = R { a: 1, b: 2 };
            let mut fresh: R = base;
            fresh = base with { a: 9 };
            let _ = fresh;
            return;
        }
    "#;
    assert_eq!(make_record_count(src), 2, "1 literal + 1 update");
    // The reassignment of `fresh` itself is producer B's own Write (whole-value,
    // no Field component) - plus the record-update's own Field-component write.
    assert_eq!(write_event_count(src), 2);
}
