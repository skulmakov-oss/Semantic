//! SSF-09 C1A P1A0-R1P-D4: record equality classifies each declared field
//! type through `canonicalize_declared_type` (a named enum is `Adt`, never the
//! parser's raw `Record` spelling), recursively through containers and nested
//! records. Equality verdicts are unchanged: an enum-bearing record is still
//! rejected, now by the record-equality policy instead of a false
//! "unknown record type" cause; genuine records stay records at every depth;
//! direct enum equality stays unsupported.

use sm_emit::compile_program_to_semcode;
use sm_front::{parse_program, type_check_program};
use sm_ir::{compile_program_to_ir, CompilePipelineError};
use sm_runtime_core::RuntimeTrap;
use sm_verify::verify_semcode;
use sm_vm::{run_verified_semcode, RuntimeError};

const E: &str = "enum E {\n    A(i32),\n    B,\n}\n";
const INNER: &str = "record Inner {\n    v: i32,\n}\n";

const RECORD_POLICY: &str =
    "record equality is allowed only when every field type already supports stable equality";
const FAMILY_POLICY: &str =
    "equality is allowed only when the value family already supports stable equality";
const FALSE_CAUSE: &str = "record equality subset references unknown record type";

fn with_main(decls: &str, body: &str) -> String {
    format!("{decls}fn main() {{\n{body}\n    return;\n}}\n")
}

/// `decls` + `record R { <field> }`, comparing one `R` value with itself.
fn record_eq(decls: &str, field: &str, init: &str, op: &str) -> String {
    with_main(
        &format!("{decls}record R {{\n    {field},\n}}\n"),
        &format!("    let x: R = R {{ {init} }};\n    let b: bool = x {op} x;"),
    )
}

/// The program parses and is rejected by the frontend, with the same message
/// from typecheck and from the compile pipeline; returns the message.
fn rejected(src: &str) -> String {
    let program = parse_program(src).expect("negative witnesses must parse");
    let tc = type_check_program(&program).expect_err("typecheck must reject");
    match compile_program_to_ir(src).expect_err("the pipeline must reject") {
        CompilePipelineError::Frontend(err) => assert_eq!(err.message, tc.message),
        other => panic!("expected a frontend error, got {other:?}"),
    }
    tc.message
}

/// Rejected by the owning record-equality policy, never by the false
/// unknown-record cause.
fn record_policy(src: &str) {
    let message = rejected(src);
    assert!(!message.contains(FALSE_CAUSE), "{message}");
    assert_eq!(message, RECORD_POLICY);
}

/// Parse, typecheck, lower, emit, verify and run; the program's own `assert`s
/// check the equality result.
fn runs(src: &str) {
    let program = parse_program(src).expect("parse");
    if let Err(err) = type_check_program(&program) {
        panic!("valid program must typecheck, got {}", err.message);
    }
    compile_program_to_ir(src).expect("valid program must lower");
    let bytes = compile_program_to_semcode(src).expect("valid program must emit");
    verify_semcode(&bytes).expect("emitted program must verify");
    run_verified_semcode(&bytes).expect("verified execution");
}

fn traps_on_assert(src: &str) {
    let bytes = compile_program_to_semcode(src).expect("valid program must emit");
    let err = run_verified_semcode(&bytes).expect_err("the assertion must fail");
    assert!(
        matches!(err, RuntimeError::Trap(RuntimeTrap::AssertionFailed)),
        "{err:?}"
    );
}

// A. Direct EQ-RAW

#[test]
fn d4_a1_enum_field_record_eq_uses_record_policy() {
    record_policy(&record_eq(E, "e: E", "e: E::B", "=="));
}

#[test]
fn d4_a2_enum_field_record_ne_uses_record_policy() {
    record_policy(&record_eq(E, "e: E", "e: E::B", "!="));
}

#[test]
fn d4_a3_payloadless_enum_field_record_eq_uses_record_policy() {
    let k = "enum K {\n    P,\n    Q,\n}\n";
    record_policy(&record_eq(k, "k: K", "k: K::P", "=="));
}

// B. Field-order independence

#[test]
fn d4_b1_enum_and_map_fields_reject_the_same_in_either_order() {
    let enum_first = with_main(
        &format!("{E}record R {{\n    e: E,\n    m: Map(i32, i32),\n}}\n"),
        "    let x: R = R { e: E::B, m: map_empty() };\n    let b: bool = x == x;",
    );
    let map_first = with_main(
        &format!("{E}record R {{\n    m: Map(i32, i32),\n    e: E,\n}}\n"),
        "    let x: R = R { m: map_empty(), e: E::B };\n    let b: bool = x == x;",
    );
    record_policy(&enum_first);
    record_policy(&map_first);
}

#[test]
fn d4_b2_enum_and_scalar_fields_reject_the_same_in_either_order() {
    let enum_first = with_main(
        &format!("{E}record R {{\n    e: E,\n    n: i32,\n}}\n"),
        "    let x: R = R { e: E::B, n: 1 };\n    let b: bool = x == x;",
    );
    let scalar_first = with_main(
        &format!("{E}record R {{\n    n: i32,\n    e: E,\n}}\n"),
        "    let x: R = R { n: 1, e: E::B };\n    let b: bool = x == x;",
    );
    record_policy(&enum_first);
    record_policy(&scalar_first);
}

// C. Recursive canonicalization

#[test]
fn d4_c1_sequence_of_enum_field() {
    record_policy(&record_eq(E, "s: Sequence(E)", "s: [E::B]", "=="));
}

#[test]
fn d4_c2_option_of_enum_field() {
    record_policy(&record_eq(E, "o: Option(E)", "o: Option::Some(E::B)", "=="));
}

#[test]
fn d4_c3_tuple_with_enum_field() {
    record_policy(&record_eq(E, "t: (i32, E)", "t: (1, E::B)", "=="));
}

#[test]
fn d4_c4_result_with_enum_field() {
    record_policy(&record_eq(
        E,
        "r: Result(E, i32)",
        "r: Result::Ok(E::B)",
        "==",
    ));
}

#[test]
fn d4_c5_nested_containers_of_enum_field() {
    record_policy(&record_eq(
        E,
        "s: Sequence(Sequence(E))",
        "s: [[E::B]]",
        "==",
    ));
    record_policy(&record_eq(
        E,
        "o: Option(Sequence(E))",
        "o: Option::None",
        "==",
    ));
}

#[test]
fn d4_c6_nested_record_with_enum_field() {
    let decls = format!("{E}record Holder {{\n    e: E,\n}}\n");
    record_policy(&record_eq(
        &decls,
        "h: Holder",
        "h: Holder { e: E::B }",
        "==",
    ));
    record_policy(&record_eq(
        &decls,
        "s: Sequence(Holder)",
        "s: [Holder { e: E::B }]",
        "==",
    ));
}

#[test]
fn d4_c7_map_and_closure_enum_fields_are_unchanged() {
    record_policy(&record_eq(E, "m: Map(i32, E)", "m: map_empty()", "=="));
    record_policy(&record_eq(E, "g: Closure(E -> i32)", "g: (v => 1)", "=="));
}

// D. Genuine-record preservation

#[test]
fn d4_d1_nested_genuine_record_equality_runs() {
    let decls = format!("{INNER}record Outer {{\n    inner: Inner,\n}}\n");
    let same = "    let x: Outer = Outer { inner: Inner { v: 1 } };\n    let y: Outer = Outer { inner: Inner { v: 1 } };\n";
    let other = "    let x: Outer = Outer { inner: Inner { v: 1 } };\n    let y: Outer = Outer { inner: Inner { v: 2 } };\n";
    runs(&with_main(&decls, &format!("{same}    assert(x == y);")));
    runs(&with_main(&decls, &format!("{other}    assert(x != y);")));
    traps_on_assert(&with_main(&decls, &format!("{other}    assert(x == y);")));
}

#[test]
fn d4_d2_three_level_genuine_record_equality_runs() {
    let decls = format!(
        "{INNER}record Mid {{\n    inner: Inner,\n}}\nrecord Outer {{\n    mid: Mid,\n}}\n"
    );
    let x = "    let x: Outer = Outer { mid: Mid { inner: Inner { v: 1 } } };\n";
    runs(&with_main(&decls, &format!("{x}    assert(x == x);")));
}

/// `record Outer { <field> }` holding genuine `Inner` values inside a
/// container; `x` and `y` differ only inside the container.
fn genuine_in_container(field: &str, same: &str, other: &str) {
    let decls = format!("{INNER}record Outer {{\n    {field},\n}}\n");
    let pair = |a: &str, b: &str| {
        format!("    let x: Outer = Outer {{ {a} }};\n    let y: Outer = Outer {{ {b} }};\n")
    };
    runs(&with_main(
        &decls,
        &format!("{}    assert(x == y);", pair(same, same)),
    ));
    runs(&with_main(
        &decls,
        &format!("{}    assert(x != y);", pair(same, other)),
    ));
}

#[test]
fn d4_d3_sequence_of_genuine_record_equality_runs() {
    genuine_in_container(
        "s: Sequence(Inner)",
        "s: [Inner { v: 1 }]",
        "s: [Inner { v: 2 }]",
    );
}

#[test]
fn d4_d4_option_of_genuine_record_equality_runs() {
    genuine_in_container(
        "o: Option(Inner)",
        "o: Option::Some(Inner { v: 1 })",
        "o: Option::Some(Inner { v: 2 })",
    );
}

#[test]
fn d4_d5_tuple_with_genuine_record_equality_runs() {
    genuine_in_container(
        "t: (i32, Inner)",
        "t: (1, Inner { v: 1 })",
        "t: (1, Inner { v: 2 })",
    );
}

#[test]
fn d4_d6_nested_containers_of_genuine_record_equality_runs() {
    genuine_in_container(
        "s: Sequence(Option(Inner))",
        "s: [Option::Some(Inner { v: 1 })]",
        "s: [Option::Some(Inner { v: 2 })]",
    );
}

#[test]
fn d4_d7_genuine_record_with_unsupported_field_stays_rejected() {
    let decls = "record Inner {\n    m: Map(i32, i32),\n}\n";
    record_policy(&record_eq(
        decls,
        "inner: Inner",
        "inner: Inner { m: map_empty() }",
        "==",
    ));
}

// E. Direct enum equality stays unsupported

#[test]
fn d4_e1_direct_enum_equality_stays_rejected() {
    for op in ["==", "!="] {
        let src = with_main(
            E,
            &format!("    let a: E = E::B;\n    let b: bool = a {op} a;"),
        );
        assert_eq!(rejected(&src), FAMILY_POLICY);
    }
}

#[test]
fn d4_e2_enum_field_read_equality_stays_rejected() {
    let src = with_main(
        &format!("{E}record R {{\n    e: E,\n}}\n"),
        "    let x: R = R { e: E::B };\n    let b: bool = x.e == x.e;",
    );
    assert_eq!(rejected(&src), FAMILY_POLICY);
}

#[test]
fn d4_e3_payloadless_enum_equality_stays_rejected() {
    let src = with_main(
        "enum K {\n    P,\n    Q,\n}\n",
        "    let a: K = K::P;\n    let b: bool = a == a;",
    );
    assert_eq!(rejected(&src), FAMILY_POLICY);
}
