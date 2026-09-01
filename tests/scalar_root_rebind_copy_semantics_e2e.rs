// SSF-08 (#1579): empirical proof that a plain (non-destructuring) root
// rebind — `let y = x;` with no tuple/record/ADT pattern involved — is
// copy-by-value end to end, for representative scalar families and for a
// non-Copy aggregate. This backs the "Scalar / direct root" row of
// `docs/roadmap/stable_foundation/ssf08_ownership_position_decision.md`:
// only pattern-based destructuring capture is tracked as move/borrow; a
// plain root rebind produces a zero-component `AccessPath`, which
// `docs/spec/runtime_ownership.md` never promised to track.

use sm_emit::compile_program_to_semcode;
use sm_verify::verify_semcode_token;
use sm_vm::run_verified_entry_semcode;

fn run_source(src: &str) {
    let bytes = compile_program_to_semcode(src).expect("compile");
    let token = verify_semcode_token(&bytes).expect("token admission");
    let entry_token = token.require_entry("main").expect("entry resolution");
    run_verified_entry_semcode(&entry_token).expect("vm run");
}

#[test]
fn plain_rebind_i32_remains_usable_copy_semantics() {
    run_source(
        r#"
        fn main() {
            let a: i32 = 7;
            let b = a;
            assert(a == 7);
            assert(b == 7);
            return;
        }
        "#,
    );
}

#[test]
fn plain_rebind_bool_remains_usable_copy_semantics() {
    run_source(
        r#"
        fn main() {
            let c: bool = (1 == 1);
            let d = c;
            assert(c == d);
            return;
        }
        "#,
    );
}

#[test]
fn plain_rebind_quad_remains_usable_copy_semantics() {
    run_source(
        r#"
        fn main() {
            let e: quad = T;
            let f = e;
            assert(e == T);
            assert(f == T);
            return;
        }
        "#,
    );
}

#[test]
fn plain_rebind_f64_remains_usable_copy_semantics() {
    run_source(
        r#"
        fn main() {
            let g: f64 = 2.5;
            let h = g;
            assert(g == 2.5);
            assert(h == 2.5);
            return;
        }
        "#,
    );
}

#[test]
fn plain_rebind_text_remains_usable_copy_semantics() {
    run_source(
        r#"
        fn main() {
            let i: text = "hi";
            let j = i;
            assert(i == "hi");
            assert(j == "hi");
            return;
        }
        "#,
    );
}

/// A plain root rebind of a non-Copy aggregate (a record) also remains
/// copy-by-value today — the boundary is not "Copy types copy, non-Copy
/// types move" (no such classification exists in the frontend); it is
/// "a plain root rebind never tracks move/borrow for any type, only
/// pattern-based destructuring capture does."
#[test]
fn plain_rebind_non_copy_record_remains_usable_copy_semantics() {
    run_source(
        r#"
        record Point { x: i32, y: i32 }

        fn main() {
            let p: Point = Point { x: 1, y: 2 };
            let q = p;
            assert(p.x == 1);
            assert(q.x == 1);
            return;
        }
        "#,
    );
}
