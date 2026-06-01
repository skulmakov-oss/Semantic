use std::path::PathBuf;

fn repo_path(rel: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel)
}

fn read(rel: &str) -> String {
    std::fs::read_to_string(repo_path(rel))
        .unwrap_or_else(|err| panic!("failed to read {rel}: {err}"))
}

#[test]
fn application_completeness_closeout_artifacts_exist() {
    let required = [
        "reports/application_completeness_benchmark_verdict.md",
        "docs/roadmap/application_completeness_pr_ledger.md",
        "docs/roadmap/snake_trace_adapter_contract.md",
        "examples/benchmarks/README.md",
        "examples/benchmarks/snake_core.sm",
        "examples/benchmarks/snake_learning.sm",
        "examples/benchmarks/snake_trace_sample.txt",
        "tests/snake_core_benchmark.rs",
        "tests/snake_learning_benchmark.rs",
    ];

    for rel in required {
        assert!(
            repo_path(rel).exists(),
            "missing required closeout artifact: {rel}"
        );
    }
}

#[test]
fn application_completeness_verdict_contains_required_evidence() {
    let report = read("reports/application_completeness_benchmark_verdict.md");

    let required_markers = [
        "Application-Completeness Benchmark Verdict",
        "Status: closed evidence report",
        "Semantic now supports benchmark-class application logic on the admitted deterministic surface.",
        "snake_core: score=0 steps=200",
        "snake_learning: total_score=8 total_steps=1417 episodes=10",
        "PR-F1",
        "PR-F2",
        "PR-F3",
        "PR-F4",
        "check / run / compile / verify",
        "Semantic emits deterministic text traces.",
        "Renderer is not the Semantic runtime.",
        "Application-completeness benchmark pack: CLOSED",
    ];

    for marker in required_markers {
        assert!(
            report.contains(marker),
            "closeout verdict missing required evidence marker: {marker}"
        );
    }
}

#[test]
fn application_completeness_verdict_preserves_non_claims() {
    let report = read("reports/application_completeness_benchmark_verdict.md");

    let required_non_claims = [
        "This verdict does not mean Semantic is public release.",
        "This verdict does not widen the published stable contour.",
        "public release readiness",
        "general-purpose language completeness",
        "browser ownership",
        "GUI ownership",
        "file I/O support",
        "argv support",
        "async runtime support",
        "full renderer support",
        "production optimization",
    ];

    for marker in required_non_claims {
        assert!(
            report.contains(marker),
            "closeout verdict missing required non-claim marker: {marker}"
        );
    }
}

#[test]
fn application_completeness_ledger_marks_f4_closed() {
    let ledger = read("docs/roadmap/application_completeness_pr_ledger.md");

    let required_markers = [
        "`PR-F1` [landed]",
        "`PR-F2` [landed]",
        "`PR-F3` [landed]",
        "`PR-F4` [landed]",
        "Status: closed by PR-F4.",
        "application-completeness benchmark pack marked closed",
    ];

    for marker in required_markers {
        assert!(
            ledger.contains(marker),
            "application-completeness ledger missing closeout marker: {marker}"
        );
    }

    assert!(
        !ledger.contains("`PR-F4` [required]"),
        "ledger still marks PR-F4 as required after closeout"
    );
}

#[test]
fn snake_trace_adapter_boundary_remains_external() {
    let contract = read("docs/roadmap/snake_trace_adapter_contract.md");

    let required_markers = [
        "Semantic trace is data.",
        "Renderer is an adapter.",
        "Renderer is not the Semantic runtime.",
        "Semantic program -> stdout text trace -> external adapter",
        "The adapter must not mutate Semantic runtime state.",
        "PR-F3 must not introduce:",
        "file I/O",
        "argv",
        "DOM API",
        "Canvas API",
        "WebGL API",
        "new SemCode opcode",
        "new capability",
    ];

    for marker in required_markers {
        assert!(
            contract.contains(marker),
            "trace adapter contract missing boundary marker: {marker}"
        );
    }
}
