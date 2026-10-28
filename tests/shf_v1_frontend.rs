use sm_emit::{compile_program_to_semcode_with_options, CompileProfile, OptLevel};
use sm_runtime_core::{ExecutionConfig, ExecutionContext, QuotaExceeded, QuotaKind};
use sm_verify::verify_semcode_token;
use sm_vm::{
    run_verified_function_semcode_with_args, run_verified_function_semcode_with_args_and_config,
    RuntimeError, Value,
};

#[test]
fn frontend_pipeline() {
    let source = format!(
        "{}\n{}",
        include_str!("fixtures/shf_v1/lexer.sm"),
        include_str!("fixtures/shf_v1/frontend.sm")
    );
    for opt in [OptLevel::O0, OptLevel::O1] {
        let compile = || {
            compile_program_to_semcode_with_options(&source, CompileProfile::RustLike, opt)
                .expect("compile")
        };
        let bytes = compile();
        assert_eq!(bytes, compile());
        let verified = verify_semcode_token(&bytes).expect("verify");
        let run = |name: &str, input: &str, extra: Vec<Value>| {
            let entry = verified.require_entry(name).expect("entry");
            let mut args = vec![Value::Text(input.into())];
            args.extend(extra);
            run_verified_function_semcode_with_args(&entry, args).expect("run")
        };
        for (input, expected) in [
            ("", 0),
            ("// hi\nlet a = 1 + 2 * 3; let b = (a - 4) / 2;", 0),
            ("let x = missing;", 4),
            ("let x = x;", 4),
            ("let x = 1; let x = 2;", 3),
            ("let let = 1;", 3),
            ("let x = 1 + ;", 2),
            ("let x = (1;", 2),
            ("let x = @;", 1),
            ("let x = \u{e9};", 1),
            ("let x = 1", 2),
        ] {
            for _ in 0..2 {
                assert_eq!(
                    run("frontend_error", input, vec![]),
                    Value::U32(expected),
                    "{input:?} {opt:?}"
                );
            }
        }
        let input = "let x = 1 + 2 * 3;";
        assert_eq!(run("frontend_nodes", input, vec![]), Value::I32(6));
        for (idx, kind) in [(0, 2), (1, 2), (2, 2), (3, 8), (4, 6), (5, 12)] {
            assert_eq!(
                run("frontend_node_kind", input, vec![Value::I32(idx)]),
                Value::U32(kind)
            );
        }
        for (idx, field, expected) in [(3, 0, 1), (3, 1, 2), (4, 0, 0), (4, 1, 3), (5, 0, 4)] {
            assert_eq!(
                run(
                    "frontend_field",
                    input,
                    vec![Value::I32(idx), Value::U32(field)]
                ),
                Value::I32(expected)
            );
        }
        assert_eq!(
            run("frontend_diagnostic_start", "let x = \u{e9};", vec![]),
            Value::U32(8)
        );
        assert_eq!(
            run("frontend_diagnostic_end", "let x = \u{e9};", vec![]),
            Value::U32(9)
        );
        for (index, field, expected) in [
            (0, 0, 1),
            (0, 1, 5),
            (0, 2, 8),
            (1, 0, 1),
            (1, 1, 9),
            (1, 2, 10),
        ] {
            assert_eq!(
                run(
                    "frontend_token_field",
                    "//x\n let x=1;",
                    vec![Value::I32(index), Value::U32(field)]
                ),
                Value::U32(expected)
            );
        }
        assert_eq!(
            run(
                "frontend_node_span",
                input,
                vec![Value::I32(4), Value::Bool(false)]
            ),
            Value::U32(8)
        );
        assert_eq!(
            run(
                "frontend_node_span",
                input,
                vec![Value::I32(4), Value::Bool(true)]
            ),
            Value::U32(17)
        );
        let left = "let a=9-2-1;";
        assert_eq!(
            run("frontend_field", left, vec![Value::I32(4), Value::U32(0)]),
            Value::I32(2)
        );
        let medium = "let a=1; let b=a+2; let c=b*3; let d=(c+a)*2; let e=d-4; let f=e/2; let g=f+1; let h=g*b;";
        assert_eq!(run("frontend_error", medium, vec![]), Value::U32(0));
        for input in [
            "let",
            "let x",
            "let x=",
            "let x=(",
            "let x=1+",
            "let x=1; let",
        ] {
            assert_eq!(
                run("frontend_error", input, vec![]),
                Value::U32(2),
                "{input}"
            );
            assert_eq!(
                run("frontend_diagnostic_start", input, vec![]),
                Value::U32(input.len() as u32)
            );
            assert_eq!(
                run("frontend_diagnostic_end", input, vec![]),
                Value::U32(input.len() as u32)
            );
        }
        let config = ExecutionConfig::for_context(ExecutionContext::VerifiedLocal);
        let with_budget = |entry: &str, input: &str| {
            run_verified_function_semcode_with_args_and_config(
                &verified.require_entry(entry).expect("entry"),
                vec![Value::Text(input.into())],
                config,
            )
        };
        let nodes = format!("let x={}1;", "1+".repeat(253));
        let tokens = format!("let x={}1;", "1+".repeat(255));
        let nested = format!("let x={}1{};", "(".repeat(64), ")".repeat(64));
        let boundary = " ".repeat(4096);
        let manageable_nodes = format!("let x={}1;", "1+".repeat(29));
        assert_eq!(
            with_budget("frontend_nodes", &manageable_nodes).expect("arena"),
            Value::I32(60)
        );
        assert_eq!(
            with_budget("frontend_error", &nested).expect("depth limit"),
            Value::U32(6)
        );
        for (label, name, input) in [
            ("508-node arena", "frontend_nodes", nodes.as_str()),
            ("token limit", "frontend_error", tokens.as_str()),
            ("4096-byte boundary", "frontend_error", boundary.as_str()),
        ] {
            for _ in 0..2 {
                let error = with_budget(name, input).expect_err(label);
                assert!(
                    matches!(
                        error,
                        RuntimeError::QuotaExceeded(QuotaExceeded {
                            kind: QuotaKind::Steps,
                            limit: 100_000,
                            used: 100_001,
                        })
                    ),
                    "{label} {opt:?}: {error:?}"
                );
            }
        }
        let long = " ".repeat(4097);
        assert_eq!(run("frontend_error", &long, vec![]), Value::U32(6));
        let early_invalid = format!("@{}", " ".repeat(4096));
        for _ in 0..2 {
            assert_eq!(run("frontend_error", &early_invalid, vec![]), Value::U32(6));
            assert_eq!(
                run("frontend_diagnostic_start", &early_invalid, vec![]),
                Value::U32(4097)
            );
            assert_eq!(
                run("frontend_diagnostic_end", &early_invalid, vec![]),
                Value::U32(4097)
            );
        }
    }
}
