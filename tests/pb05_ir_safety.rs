//! PB-05 regressions that cross crate boundaries: optimized IR must keep the
//! runtime rejection of the raw IR (#1723, #1729), and user code cannot claim
//! names reserved by the builtin namespace (#1721).

use sm_front::QuadVal;
use sm_ir::{
    compile_program_to_semcode, emit_ir_to_semcode, passes::run_default_opt_passes, IrFunction,
    IrInstr,
};
use sm_vm::{run_semcode, RuntimeError};

fn main_fn(instrs: Vec<IrInstr>) -> Vec<IrFunction> {
    vec![IrFunction {
        name: "main".to_string(),
        instrs,
        ownership_events: Vec::new(),
        params: Vec::new(),
    }]
}

/// Runtime rejection class: the `RuntimeError` variant, ignoring payload text.
fn rejection_class(err: &RuntimeError) -> core::mem::Discriminant<RuntimeError> {
    core::mem::discriminant(err)
}

/// O0 and O1 must preserve the same runtime rejection class.
fn assert_same_rejection_class(o0: &Result<(), RuntimeError>, o1: &Result<(), RuntimeError>) {
    let (Err(e0), Err(e1)) = (o0, o1) else {
        panic!("both levels must reject: O0={o0:?} O1={o1:?}");
    };
    assert_eq!(
        rejection_class(e0),
        rejection_class(e1),
        "optimizer changed the rejection class: O0={e0:?} O1={e1:?}"
    );
}

/// Runs the raw (O0) and optimized (O1) program and returns the O0 rejection
/// after proving O1 rejects with the same class.
fn o0_o1_rejection(instrs: Vec<IrInstr>) -> RuntimeError {
    let raw = main_fn(instrs);
    let mut optimized = raw.clone();
    run_default_opt_passes(&mut optimized).expect("opt");
    let o0 = run_semcode(&emit_ir_to_semcode(&raw, false).expect("emit O0"));
    let o1 = run_semcode(&emit_ir_to_semcode(&optimized, false).expect("emit O1"));
    assert_same_rejection_class(&o0, &o1);
    o0.expect_err("checked above")
}

fn type_invalid(op: IrInstr, lhs: IrInstr) -> Vec<IrInstr> {
    vec![
        lhs,
        IrInstr::LoadI32 { dst: 2, val: 1 },
        op,
        IrInstr::Ret { src: None },
    ]
}

#[test]
fn one_sided_rewrites_keep_type_mismatch_class() {
    let cases = [
        type_invalid(
            IrInstr::BoolAnd {
                dst: 3,
                lhs: 1,
                rhs: 2,
            },
            IrInstr::LoadBool { dst: 1, val: false },
        ),
        type_invalid(
            IrInstr::BoolOr {
                dst: 3,
                lhs: 1,
                rhs: 2,
            },
            IrInstr::LoadBool { dst: 1, val: true },
        ),
        type_invalid(
            IrInstr::QAnd {
                dst: 3,
                lhs: 1,
                rhs: 2,
            },
            IrInstr::LoadQ {
                dst: 1,
                val: QuadVal::N,
            },
        ),
        type_invalid(
            IrInstr::AddI32 {
                dst: 3,
                lhs: 2,
                rhs: 1,
            },
            IrInstr::LoadBool { dst: 1, val: true },
        ),
    ];
    for case in cases {
        let err = o0_o1_rejection(case.clone());
        assert!(
            matches!(err, RuntimeError::TypeMismatchRuntime(_)),
            "{case:?}: {err:?}"
        );
    }
}

#[test]
fn cleanup_keeps_unknown_variable_class() {
    // `valid` is defined, so erasing the `missing` load would make O1 succeed.
    let err = o0_o1_rejection(vec![
        IrInstr::LoadI32 { dst: 2, val: 7 },
        IrInstr::StoreVar {
            name: "valid".into(),
            src: 2,
            activation_site: None,
            write_site: None,
        },
        IrInstr::LoadVar {
            dst: 1,
            name: "missing".into(),
        },
        IrInstr::LoadVar {
            dst: 1,
            name: "valid".into(),
        },
        IrInstr::Ret { src: None },
    ]);
    assert!(matches!(err, RuntimeError::UnknownVariable(_)), "{err:?}");
}

/// The comparator itself: both `Err` is not enough.
#[test]
fn rejection_comparator_distinguishes_classes() {
    let mismatch: Result<(), RuntimeError> = Err(RuntimeError::TypeMismatchRuntime("a".into()));
    let reworded: Result<(), RuntimeError> = Err(RuntimeError::TypeMismatchRuntime("b".into()));
    assert_same_rejection_class(&mismatch, &reworded);
    for other in [
        Err(RuntimeError::UnknownVariable("a".into())),
        Err(RuntimeError::StackUnderflow),
        Ok(()),
    ] {
        let caught = std::panic::catch_unwind(|| assert_same_rejection_class(&mismatch, &other));
        assert!(caught.is_err(), "comparator accepted O1={other:?}");
    }
}

/// Every bare name that `sm-ir` lowers as an intrinsic (instead of a call)
/// must be reserved by the frontend builtin namespace, so a user function of
/// that name cannot exist (#1721). The names are read from the lowering source
/// and the frontend is the only policy authority consulted.
#[test]
fn every_intrinsic_lowered_name_is_reserved_by_the_frontend() {
    let lowering = include_str!("../crates/sm-ir/src/legacy_lowering.rs");
    let mut names = std::collections::BTreeSet::new();
    // (marker, bytes of the marker that precede the name)
    const EQ: &str = "resolve_symbol_name(arena, *name)? == \"";
    for (marker, skip) in [(EQ, EQ.len()), ("\"qtruth_", 1)] {
        assert_eq!(&marker[skip - 1..skip], "\"");
        for (i, _) in lowering.match_indices(marker) {
            let rest = &lowering[i + skip..];
            let name = &rest[..rest.find('"').expect("closing quote")];
            if name != "qtruth_" {
                names.insert(name.to_string());
            }
        }
    }
    for expected in [
        "random_seed",
        "random_next_i32",
        "qtruth_and",
        "qtruth_impl",
    ] {
        assert!(
            names.contains(expected),
            "scanner lost {expected}: {names:?}"
        );
    }
    for name in &names {
        let src = format!(
            "fn {name}() {{}}
fn main() {{}}
"
        );
        let err = format!("{:?}", compile_program_to_semcode(&src).expect_err(name));
        assert!(
            err.contains("is reserved for the language builtin"),
            "{name}: {err}"
        );
    }
    // Control: the same shape with an ordinary name compiles.
    compile_program_to_semcode(
        "fn ordinary_name() {}
fn main() {}
",
    )
    .expect("control");
}
