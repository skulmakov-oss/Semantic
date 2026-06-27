use semantic_language::frontend;
use std::fs;

#[test]
fn boundary_core_ir_emit_are_accessible() {
    let src = "fn main() { return; }";

    let p = frontend::core::parse_program(src).expect("parse");
    let fns = frontend::core::build_fn_table(&p).expect("fn table");
    let _ = frontend::ir::lower_function_to_ir(&p.functions[0], &p.arena, &fns).expect("lower");

    let bytes = frontend::emit::compile_program_to_semcode(src).expect("emit");
    assert!(bytes.len() >= 8);
}

#[test]
fn sm_emit_reexports_semcode_contract_from_sm_ir() {
    let src = fs::read_to_string("crates/sm-emit/src/lib.rs").expect("read sm-emit lib");

    assert!(
        src.contains("pub use sm_format::semcode_format"),
        "sm-emit must re-export the canonical SemCode contract from sm-format"
    );
    assert!(
        !src.contains("mod semcode_format;"),
        "sm-emit must not carry a second local semcode_format owner"
    );
}
