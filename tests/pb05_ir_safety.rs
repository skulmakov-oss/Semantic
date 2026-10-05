//! PB-05 regressions that cross crate boundaries: optimized IR must keep the
//! runtime rejection of the raw IR (#1723, #1729), and user code cannot claim
//! names reserved by the builtin namespace (#1721).

use sm_ir::{
    compile_program_to_semcode, emit_ir_to_semcode, passes::run_default_opt_passes, IrFunction,
    IrInstr,
};
use sm_vm::run_semcode;

fn main_fn(instrs: Vec<IrInstr>) -> Vec<IrFunction> {
    vec![IrFunction {
        name: "main".to_string(),
        instrs,
        ownership_events: Vec::new(),
        params: Vec::new(),
    }]
}

/// Runs the raw (O0) and optimized (O1) program; both must be rejected.
fn assert_o0_o1_both_trap(instrs: Vec<IrInstr>) {
    let raw = main_fn(instrs);
    let mut optimized = raw.clone();
    run_default_opt_passes(&mut optimized).expect("opt");
    let o0 = run_semcode(&emit_ir_to_semcode(&raw, false).expect("emit O0"));
    let o1 = run_semcode(&emit_ir_to_semcode(&optimized, false).expect("emit O1"));
    assert!(o0.is_err(), "raw program must trap");
    assert!(
        o1.is_err(),
        "optimizer erased a runtime rejection: {:?}",
        optimized[0].instrs
    );
}

#[test]
fn one_sided_bool_annihilator_keeps_type_trap() {
    assert_o0_o1_both_trap(vec![
        IrInstr::LoadBool { dst: 1, val: false },
        IrInstr::LoadI32 { dst: 2, val: 1 },
        IrInstr::BoolAnd {
            dst: 3,
            lhs: 1,
            rhs: 2,
        },
        IrInstr::Ret { src: None },
    ]);
}

#[test]
fn cleanup_keeps_trapping_loadvar() {
    assert_o0_o1_both_trap(vec![
        IrInstr::LoadVar {
            dst: 1,
            name: "missing".into(),
        },
        IrInstr::LoadVar {
            dst: 1,
            name: "missing".into(),
        },
        IrInstr::Ret { src: None },
    ]);
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
