//! PB-02 (#1617 Phase B) end-to-end frontend admission regressions that
//! cross the sm-front -> sm-ir boundary. Unit regressions live next to their
//! owners in `crates/sm-front/src/{lib,parser,typecheck}.rs`.

use sm_ir::{compile_program_to_ir, IrInstr};

const QTRUTH: [(&str, &str); 4] = [
    ("qtruth_and", "qtruth_and(F, T)"),
    ("qtruth_or", "qtruth_or(F, T)"),
    ("qtruth_impl", "qtruth_impl(F, T)"),
    ("qtruth_not", "qtruth_not(F)"),
];

fn main_ir(src: &str) -> Vec<IrInstr> {
    compile_program_to_ir(src)
        .expect("compiles")
        .into_iter()
        .find(|f| f.name == "main")
        .expect("main")
        .instrs
}

// #1654: language-owned qtruth calls still lower to dedicated QTruth IR...
#[test]
fn qtruth_builtin_calls_lower_to_dedicated_ir() {
    for (name, call) in QTRUTH {
        let src = format!("fn main() {{\n    let x: quad = {call};\n    return;\n}}\n");
        let ir = main_ir(&src);
        let lowered = ir.iter().any(|i| {
            matches!(
                (name, i),
                ("qtruth_and", IrInstr::QTruthAnd { .. })
                    | ("qtruth_or", IrInstr::QTruthOr { .. })
                    | ("qtruth_impl", IrInstr::QTruthImpl { .. })
                    | ("qtruth_not", IrInstr::QTruthNot { .. })
            )
        });
        assert!(
            lowered,
            "{name} must lower to its QTruth instruction: {ir:?}"
        );
        assert!(
            !ir.iter()
                .any(|i| matches!(i, IrInstr::Call { name: n, .. } if n == name)),
            "{name} must not lower to an ordinary call"
        );
    }
}

// ...and a same-name user function can no longer be admitted and then
// silently replaced by the intrinsic.
#[test]
fn user_qtruth_function_is_rejected_before_lowering() {
    for (name, call) in QTRUTH {
        let params = if name == "qtruth_not" {
            "a: quad"
        } else {
            "a: quad, b: quad"
        };
        let src = format!(
            "fn {name}({params}) -> quad {{\n    return T;\n}}\nfn main() {{\n    let x: quad = {call};\n    return;\n}}\n"
        );
        let err = compile_program_to_ir(&src).expect_err(name);
        let msg = format!("{err:?}");
        assert!(
            msg.contains(name) && msg.contains("reserved"),
            "{name}: {msg}"
        );
    }
}
