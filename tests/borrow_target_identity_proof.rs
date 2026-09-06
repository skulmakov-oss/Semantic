// #1726 pre-implementation proof: a lowered-local key identifies a binding,
// but the introduction and subsequent assignments share that StoreVar key.
use sm_ir::{
    compile_program_to_ir_with_options, emit_ir_to_semcode, CompileProfile, IrInstr, OptLevel,
    OwnershipPathEventKind,
};

#[test]
fn frozen_borrow_targets_do_not_uniquely_identify_store_sites() {
    let producers = [
        (
            "bind_tuple_items",
            "fn main() { let pair: (i32, i32) = (1, 2); let (ref left, _) = pair; left = 9; assert(left == 9); return; }",
        ),
        (
            "bind_record_items",
            "record R { value: i32, flag: quad, } fn main() { let r: R = R { value: 1, flag: T }; let R { value: ref left, flag: _ } = r; left = 9; assert(left == 9); return; }",
        ),
        (
            "bind_let_else_tuple_items",
            "fn main() { let pair: (i32, quad) = (1, T); let (ref left, T) = pair else return; left = 9; assert(left == 9); return; }",
        ),
        (
            "bind_let_else_record_items",
            "record R { value: i32, flag: quad, } fn main() { let r: R = R { value: 1, flag: T }; let R { value: ref left, flag: T } = r else return; left = 9; assert(left == 9); return; }",
        ),
    ];
    for (producer, source) in producers {
        for opt in [OptLevel::O0, OptLevel::O1] {
            let ir = compile_program_to_ir_with_options(source, CompileProfile::RustLike, opt)
                .expect("admitted source");
            let main = ir.iter().find(|f| f.name == "main").expect("main");
            assert_eq!(
                main.ownership_events
                    .iter()
                    .filter(|e| e.kind == OwnershipPathEventKind::Borrow)
                    .count(),
                1
            );
            // Exact key for this fixture; never used to infer production metadata.
            let target = "__sm_local_1_left";
            let stores = main
                .instrs
                .iter()
                .enumerate()
                .filter_map(|(index, i)| match i {
                    IrInstr::StoreVar { name, src, .. } if name == target => Some((index, *src)),
                    _ => None,
                })
                .collect::<Vec<_>>();
            assert_eq!(stores.len(), 2, "{producer} {opt:?}: {stores:?}");
            let bytes = emit_ir_to_semcode(&ir, false).expect("emit");
            let (_, decoded) =
                sm_ir::semcode_decode::decode_semcode_envelope(&bytes).expect("decode");
            let main = decoded.iter().find(|f| f.name == "main").expect("main");
            assert_eq!(
                main.strings.iter().filter(|s| s.as_str() == target).count(),
                1
            );
            let token = sm_verify::verify_semcode_token(&bytes).expect("admit");
            let entry = token.require_entry("main").expect("entry");
            sm_vm::run_verified_entry_semcode(&entry).expect("execute both stores");
            eprintln!("{producer} {opt:?}: target={target}, StoreVar (IR index, source register)={stores:?}; one string-table identity; verified execution passed");
        }
    }
}

#[test]
fn deferred_record_else_return_reverses_borrow_and_store_order() {
    let source = r#"
        record R { value: i32, flag: quad, }
        fn probe(flag: quad) -> i32 {
            let pair: (i32, i32) = (9, 8);
            let r: R = R { value: 1, flag: flag };
            let R { value: ref outer, flag: T } = r else return {
                let (ref inner, _) = pair;
                inner
            };
            return outer;
        }
        fn main() {
            assert(probe(T) == 1);
            assert(probe(F) == 9);
            return;
        }
    "#;
    for opt in [OptLevel::O0, OptLevel::O1] {
        let ir = compile_program_to_ir_with_options(source, CompileProfile::RustLike, opt)
            .expect("admitted deferred record source");
        let probe = ir.iter().find(|f| f.name == "probe").expect("probe");
        let roots = probe
            .ownership_events
            .iter()
            .filter(|e| e.kind == OwnershipPathEventKind::Borrow)
            .map(|e| e.path.root.as_str())
            .collect::<Vec<_>>();
        assert_eq!(roots, ["__sm_local_2_r", "__sm_local_1_pair"]);
        let targets = probe
            .instrs
            .iter()
            .filter_map(|i| match i {
                IrInstr::StoreVar { name, .. }
                    if name.ends_with("_inner") || name.ends_with("_outer") =>
                {
                    Some(name.as_str())
                }
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(targets, ["__sm_local_3_inner", "__sm_local_4_outer"]);
        let bytes = emit_ir_to_semcode(&ir, false).expect("emit");
        let token = sm_verify::verify_semcode_token(&bytes).expect("admit");
        sm_vm::run_verified_entry_semcode(&token.require_entry("main").expect("entry"))
            .expect("both record branches execute correctly on baseline");
        eprintln!(
            "{opt:?}: Borrow roots={roots:?}; StoreVar targets={targets:?}; both branches passed"
        );
    }
}
