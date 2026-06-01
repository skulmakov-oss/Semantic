use std::fs;
use std::path::{Path, PathBuf};

use sm_front::{build_fn_table, parse_program, Expr, Stmt};
use sm_ir::lower_function_to_ir;

fn collect_rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(root) else { return };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_rs_files(&p, out);
        } else if p.extension().and_then(|s| s.to_str()) == Some("rs") {
            out.push(p);
        }
    }
}

fn find_lines_with(root: &Path, needle: &str) -> Vec<(String, String)> {
    let mut files = Vec::new();
    collect_rs_files(root, &mut files);
    let mut hits = Vec::new();
    for file in files {
        let rel = file.to_string_lossy().replace('\\', "/");
        let txt = fs::read_to_string(&file).expect("read rs");
        for line in txt.lines() {
            let trimmed = line.trim();
            if trimmed.contains(needle) {
                hits.push((rel.clone(), trimmed.to_string()));
            }
        }
    }
    hits
}

#[test]
fn raw_expr_storage_is_confined_to_ast_arena() {
    let expr_vec_hits = find_lines_with(Path::new("crates/sm-front/src"), "Vec<Expr>");
    assert_eq!(
        expr_vec_hits,
        vec![(
            "crates/sm-front/src/types.rs".to_string(),
            "exprs: Vec<Expr>,".to_string()
        )],
        "raw Expr collections must remain owned only by AstArena"
    );

    let expr_box_hits = find_lines_with(Path::new("crates/sm-front/src"), "Box<Expr>");
    assert!(
        expr_box_hits.is_empty(),
        "frontend must not reintroduce boxed raw Expr ownership outside AstArena: {:?}",
        expr_box_hits
    );

    let expr_field_hits = find_lines_with(Path::new("crates/sm-front/src"), ": Expr,");
    assert!(
        expr_field_hits.is_empty(),
        "frontend fields must reference expressions by ExprId, not own Expr directly: {:?}",
        expr_field_hits
    );
}

#[test]
fn raw_stmt_storage_is_confined_to_ast_arena() {
    let stmt_vec_hits = find_lines_with(Path::new("crates/sm-front/src"), "Vec<Stmt>");
    assert_eq!(
        stmt_vec_hits,
        vec![(
            "crates/sm-front/src/types.rs".to_string(),
            "stmts: Vec<Stmt>,".to_string()
        )],
        "raw Stmt collections must remain owned only by AstArena"
    );

    let stmt_box_hits = find_lines_with(Path::new("crates/sm-front/src"), "Box<Stmt>");
    assert!(
        stmt_box_hits.is_empty(),
        "frontend must not reintroduce boxed raw Stmt ownership outside AstArena: {:?}",
        stmt_box_hits
    );

    let stmt_field_hits = find_lines_with(Path::new("crates/sm-front/src"), ": Stmt,");
    assert!(
        stmt_field_hits.is_empty(),
        "frontend fields must reference statements by StmtId, not own Stmt directly: {:?}",
        stmt_field_hits
    );
}

#[test]
fn crystalfold_stage_order_stays_cleanup_then_fold() {
    let passes = fs::read_to_string("crates/sm-ir/src/passes/mod.rs").expect("read passes mod");
    let cleanup_idx = passes
        .find("let cleanup = cleanup::StructuralCleanupPass;")
        .expect("cleanup stage exists");
    let fold_idx = passes
        .find("let fold = crystalfold::CrystalFoldPass::default();")
        .expect("crystalfold stage exists");
    assert!(
        cleanup_idx < fold_idx,
        "StructuralCleanup must remain ahead of CrystalFold in the default pass pipeline"
    );

    let opts = fs::read_to_string("docs/opts.md").expect("read opts doc");
    assert!(
        opts.contains("frontend -> semantics (warnings) -> lowering -> StructuralCleanup -> CrystalFold -> emit"),
        "docs/opts.md must keep the frozen CrystalFold pipeline contract"
    );
}

#[test]
fn structural_cleanup_pass_remains_move_only_over_instruction_streams() {
    let cleanup = fs::read_to_string("crates/sm-ir/src/passes/cleanup.rs").expect("read cleanup");
    assert!(
        !cleanup.contains("instrs[i].clone()"),
        "StructuralCleanup must not fall back to per-instruction cloning"
    );
}

#[test]
fn parsed_program_keeps_arena_nodes_addressable_across_lowering() {
    let src = r#"
        fn keep(flag: bool) -> bool {
            let seen = flag;
            return seen;
        }
    "#;
    let program = parse_program(src).expect("parse program");
    let keep = program
        .functions
        .iter()
        .find(|func| program.arena.symbol_name(func.name) == "keep")
        .expect("keep fn");

    assert_eq!(keep.body.len(), 2, "keep body should remain stmt-id based");
    let body_ids = keep.body.clone();
    let expr_count = program.arena.expr_count();
    let stmt_count = program.arena.stmt_count();

    let seen_value = match program.arena.stmt(body_ids[0]) {
        Stmt::Let { value, .. } => *value,
        other => panic!("expected first stmt to be let, got {:?}", other),
    };
    assert!(
        matches!(program.arena.expr(seen_value), Expr::Var(_)),
        "stmt ids must keep resolving into arena-owned expr nodes"
    );
    assert!(
        matches!(program.arena.stmt(body_ids[1]), Stmt::Return(Some(_))),
        "return stmt must remain addressable through arena after parse"
    );

    let fn_table = build_fn_table(&program).expect("fn table");
    let ir = lower_function_to_ir(keep, &program.arena, &fn_table).expect("lower keep fn");
    assert!(
        !ir.instrs.is_empty(),
        "lowering should consume shared arena references without taking ownership"
    );

    assert_eq!(
        program.arena.expr_count(),
        expr_count,
        "lowering must not mutate arena expr ownership"
    );
    assert_eq!(
        program.arena.stmt_count(),
        stmt_count,
        "lowering must not mutate arena stmt ownership"
    );
    assert!(
        matches!(program.arena.stmt(body_ids[0]), Stmt::Let { value, .. } if *value == seen_value),
        "stmt ids must stay stable after lowering runs against the shared arena"
    );
}

#[test]
fn parser_and_lowering_keep_shared_arena_handoff_contract() {
    let parser = fs::read_to_string("crates/sm-front/src/parser.rs").expect("read parser");
    assert!(
        parser.contains("arena: ::core::mem::take(&mut self.arena),"),
        "parse_program must move the parser arena into Program exactly once"
    );
    assert!(
        !parser.contains("arena: self.arena.clone()"),
        "parser must not clone arena state when producing Program"
    );

    let lowering =
        fs::read_to_string("crates/sm-ir/src/legacy_lowering.rs").expect("read lowering");
    for signature in [
        "pub fn lower_expr_to_ir(",
        "fn lower_function_to_ir_with_tables(",
        "pub fn lower_function_to_ir(",
    ] {
        assert!(
            lowering.contains(signature),
            "lowering entrypoint must remain present: missing `{}`",
            signature
        );
    }
    let shared_arena_count = lowering.matches("arena: &AstArena,").count();
    assert!(
        shared_arena_count >= 3,
        "lowering entrypoints must keep shared arena references; expected at least 3 explicit shared arena parameters, got {}",
        shared_arena_count
    );
    assert!(
        lowering.contains("&program.arena,"),
        "program-wide lowering must continue threading a shared Program arena through every function"
    );
}
