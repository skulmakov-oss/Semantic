// SSF-08 Lane 2b (#1724 / FA-04-018): "lowering flattens lexical scopes
// into one string-local namespace and can leak shadowed bindings".
//
// Baseline defect, reproduced directly against `main @ cb24cda5` before
// any repair: `let x = 1; if true { let x = 2; } return x;` returned `2`
// at the VM level, because both the outer and inner `x` bindings lowered
// to the *same* runtime-local key (`resolve_symbol_name` - the raw
// source spelling), so the inner `StoreVar` silently overwrote the
// outer's frame slot.
//
// Every test here proves the strongest possible statement:
//     source lexical semantics == VM observable result
// using only plain `i32`/`bool` values and `assert(...)` inside the
// compiled program itself - deliberately staying inside ordinary VM
// local transport, not ownership metadata, so this suite does not depend
// on the still-open #1725 (OWN0 root identity) or #1726 (OWN0 event
// timing) in any way.

fn run_source(src: &str) {
    let bytes = sm_emit::compile_program_to_semcode(src).expect("compile");
    let token = sm_verify::verify_semcode_token(&bytes).expect("token admission");
    let entry_token = token.require_entry("main").expect("entry resolution");
    sm_vm::run_verified_entry_semcode(&entry_token).expect("vm run");
}

#[test]
fn if_branch_shadow_restores_outer_binding_after_scope_exit() {
    run_source(
        r#"
        fn main() {
            let x: i32 = 1;
            if true {
                let x: i32 = 2;
                assert(x == 2);
            }
            assert(x == 1);
            return;
        }
        "#,
    );
}

#[test]
fn both_if_branches_shadow_and_outer_binding_survives_either_path() {
    run_source(
        r#"
        fn shadow_in_branch(take_then: bool) -> i32 {
            let x: i32 = 10;
            if take_then {
                let x: i32 = 20;
                assert(x == 20);
            } else {
                let x: i32 = 30;
                assert(x == 30);
            }
            return x;
        }

        fn main() {
            assert(shadow_in_branch(true) == 10);
            assert(shadow_in_branch(false) == 10);
            return;
        }
        "#,
    );
}

#[test]
fn match_arm_shadow_restores_outer_binding_after_match() {
    run_source(
        r#"
        fn main() {
            let x: i32 = 7;
            match x {
                7 => {
                    let x: i32 = 99;
                    assert(x == 99);
                }
                _ => {
                    let x: i32 = -1;
                    assert(x == -1);
                }
            }
            assert(x == 7);
            return;
        }
        "#,
    );
}

#[test]
fn statement_loop_body_shadow_restores_outer_binding_after_loop() {
    run_source(
        r#"
        fn main() {
            let x: i32 = 1;
            let mut i: i32 = 0;
            loop {
                let x: i32 = 999;
                assert(x == 999);
                i = i + 1;
                if i >= 3 {
                    break;
                }
            }
            assert(x == 1);
            return;
        }
        "#,
    );
}

#[test]
fn while_body_shadow_restores_outer_binding_after_loop() {
    run_source(
        r#"
        fn main() {
            let x: i32 = 1;
            let mut i: i32 = 0;
            while i < 3 {
                let x: i32 = 42;
                assert(x == 42);
                i = i + 1;
            }
            assert(x == 1);
            return;
        }
        "#,
    );
}

#[test]
fn for_range_loop_variable_does_not_collide_with_outer_binding_of_same_spelling() {
    // The loop variable itself is named the same as an outer binding -
    // the loop variable's own repeated per-iteration `StoreVar` must keep
    // resolving to the loop's own binding, not the outer one, and the
    // outer binding must be untouched after the loop.
    run_source(
        r#"
        fn main() {
            let i: i32 = -1;
            let mut sum: i32 = 0;
            for i in 0..3 {
                sum = sum + i;
            }
            assert(sum == 3);
            assert(i == -1);
            return;
        }
        "#,
    );
}

#[test]
fn nested_value_block_shadow_does_not_fight_ownership_event_channel_threading() {
    // #1709 (Lane 2a) threaded one ownership-event sink through nested
    // value-block lowering; #1724 (Lane 2b) threads one lexical-identity
    // authority through the exact same call graph. This proves the two
    // repairs compose: the inner block's shadow of `x` is fully isolated,
    // and the block's own tail value is computed correctly from the
    // *inner* binding while the outer binding survives the block.
    run_source(
        r#"
        fn main() {
            let x: i32 = 1;
            let y: i32 = {
                let x: i32 = 2;
                x
            };
            assert(y == 2);
            assert(x == 1);
            return;
        }
        "#,
    );
}

#[test]
fn reassignment_targets_the_same_binding_not_a_fresh_one() {
    // Negative control (#1724 spec, required regression): a plain
    // reassignment to an existing mutable binding must resolve to that
    // binding's *existing* lowered key, not allocate a new one - this is
    // the guard against an overcorrection where every `StoreVar` mints a
    // fresh identity.
    run_source(
        r#"
        fn main() {
            let mut x: i32 = 1;
            x = 2;
            assert(x == 2);
            x = 3;
            assert(x == 3);
            return;
        }
        "#,
    );
}

#[test]
fn ordinary_program_without_shadowing_is_unaffected() {
    // Confirms the identity layer is transparent where no collision
    // exists at all - the common case must remain exactly as correct as
    // before #1724.
    run_source(
        r#"
        fn add_one(n: i32) -> i32 {
            let result: i32 = n + 1;
            return result;
        }

        fn main() {
            let a: i32 = 5;
            let b: i32 = add_one(a);
            assert(a == 5);
            assert(b == 6);
            return;
        }
        "#,
    );
}
