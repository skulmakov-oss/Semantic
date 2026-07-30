//! Empirical determinism qualification for the `vector.turbovec` adapter.
//!
//! Per the Hub v0 determinism policy, a tool's determinism class must be
//! measured, not inferred from its name or API shape. This test exercises
//! the real TurboVec dependency (not a stub) through the adapter's public
//! `HubTool` contract -- a narrowly-scoped adapter-level test, per the
//! project's stated exception for qualification/benchmark baselines -- and
//! records exactly what was and was not verified.
//!
//! Measured on this machine/toolchain only. Cross-CPU-backend byte
//! identity (e.g. AVX-512 vs. scalar fallback on a different host) is
//! explicitly NOT claimed here; `docs/spec/hub/turbovec_adapter_v0.md`
//! records the narrower guarantee this test actually establishes.

use semantic_hub::runtime::{HubTool, RestrictedHubContext};
use semantic_hub::{HubCapabilitySet, HubResourceBudget};
use semantic_hub_turbovec::TurboVecAdapter;

fn temp_dir(tag: &str) -> std::path::PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "semantic-hub-turbovec-determinism-{tag}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    dir
}

fn ctx<'a>(budget: &'a HubResourceBudget, caps: &'a HubCapabilitySet) -> RestrictedHubContext<'a> {
    RestrictedHubContext {
        resource_budget: budget,
        capability_context: caps,
        deadline: None,
    }
}

/// Small deterministic pseudo-random generator (splitmix64) so the fixture
/// data is reproducible without pulling in a `rand` dependency for the test
/// itself -- only the *input* needs to be fixed; measuring turbovec's own
/// behavior is the point of the test.
fn splitmix64_vectors(n: usize, dim: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut state = seed;
    let mut next = || {
        state = state.wrapping_add(0x9E3779B97F4A7C15);
        let mut z = state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58476D1CE4E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D049BB133111EB);
        z ^ (z >> 31)
    };
    (0..n)
        .map(|_| {
            (0..dim)
                .map(|_| ((next() >> 40) as f32 / (1u64 << 24) as f32) - 0.5)
                .collect()
        })
        .collect()
}

fn build_populated_index(dir: &std::path::Path, dim: usize, n: usize) -> (Vec<u64>, Vec<Vec<f32>>) {
    let mut adapter = TurboVecAdapter::new(dir, HubResourceBudget::V0_CEILING);
    let budget = HubResourceBudget::V0_CEILING;
    let caps = HubCapabilitySet::empty();
    let op = |s: &str| semantic_hub::HubOperationId::new(s).unwrap();

    adapter
        .handle(
            &op("vector.index.create"),
            serde_json::to_vec(&serde_json::json!({"index": "qual", "dim": dim, "bit_width": 4}))
                .unwrap()
                .as_slice(),
            &ctx(&budget, &caps),
        )
        .unwrap();

    let vectors = splitmix64_vectors(n, dim, 0xC0FFEE);
    let ids: Vec<u64> = (0..n as u64).collect();
    adapter
        .handle(
            &op("vector.index.insert"),
            serde_json::to_vec(&serde_json::json!({
                "index": "qual",
                "vectors": vectors,
                "ids": ids,
            }))
            .unwrap()
            .as_slice(),
            &ctx(&budget, &caps),
        )
        .unwrap();

    (ids, vectors)
}

fn run_search(adapter: &mut TurboVecAdapter, query: &[f32], k: usize) -> Vec<u8> {
    let budget = HubResourceBudget::V0_CEILING;
    let caps = HubCapabilitySet::empty();
    let op = semantic_hub::HubOperationId::new("vector.search").unwrap();
    adapter
        .handle(
            &op,
            serde_json::to_vec(&serde_json::json!({
                "index": "qual",
                "queries": [query],
                "k": k,
            }))
            .unwrap()
            .as_slice(),
            &ctx(&budget, &caps),
        )
        .unwrap()
        .payload
}

#[test]
fn repeated_search_on_the_same_loaded_index_is_byte_identical() {
    let dir = temp_dir("same-instance");
    let (_, vectors) = build_populated_index(&dir, 32, 200);
    let mut adapter = TurboVecAdapter::new(&dir, HubResourceBudget::V0_CEILING);
    let query = vectors[7].clone();

    let first = run_search(&mut adapter, &query, 10);
    for i in 0..9 {
        let repeat = run_search(&mut adapter, &query, 10);
        assert_eq!(first, repeat, "search reply differed on repetition {i}");
    }
}

#[test]
fn search_after_reloading_the_index_from_disk_matches_the_original() {
    // Simulates the real Hub CLI usage pattern: each `smc hub invoke` is a
    // separate process, so the only thing carried between calls is the
    // persisted `.tvim` file. This is the determinism guarantee that
    // actually matters for CLI users.
    let dir = temp_dir("reload");
    let (_, vectors) = build_populated_index(&dir, 32, 200);
    let query = vectors[42].clone();

    let mut first_process = TurboVecAdapter::new(&dir, HubResourceBudget::V0_CEILING);
    let first = run_search(&mut first_process, &query, 10);
    drop(first_process);

    let mut second_process = TurboVecAdapter::new(&dir, HubResourceBudget::V0_CEILING);
    let second = run_search(&mut second_process, &query, 10);

    let mut third_process = TurboVecAdapter::new(&dir, HubResourceBudget::V0_CEILING);
    let third = run_search(&mut third_process, &query, 10);

    assert_eq!(
        first, second,
        "reply changed after one fresh reload from disk"
    );
    assert_eq!(
        second, third,
        "reply changed after a second fresh reload from disk"
    );
}

#[test]
fn exact_duplicate_vectors_produce_a_stable_tie_break_order_across_repeats() {
    // Does not assert a specific tie-break RULE (that is turbovec's own
    // internal behavior, not something this adapter defines) -- only that
    // whatever order it picks is the SAME order every time, which is the
    // property Hub callers actually depend on for reproducible evidence.
    let dir = temp_dir("ties");
    let mut adapter = TurboVecAdapter::new(&dir, HubResourceBudget::V0_CEILING);
    let budget = HubResourceBudget::V0_CEILING;
    let caps = HubCapabilitySet::empty();
    let op = |s: &str| semantic_hub::HubOperationId::new(s).unwrap();

    adapter
        .handle(
            &op("vector.index.create"),
            serde_json::to_vec(&serde_json::json!({"index": "qual", "dim": 16, "bit_width": 4}))
                .unwrap()
                .as_slice(),
            &ctx(&budget, &caps),
        )
        .unwrap();

    let identical_vector = vec![0.25f32; 16];
    let vectors = vec![identical_vector.clone(); 5];
    let ids: Vec<u64> = (100..105).collect();
    adapter
        .handle(
            &op("vector.index.insert"),
            serde_json::to_vec(&serde_json::json!({
                "index": "qual",
                "vectors": vectors,
                "ids": ids,
            }))
            .unwrap()
            .as_slice(),
            &ctx(&budget, &caps),
        )
        .unwrap();

    let first = run_search(&mut adapter, &identical_vector, 5);
    for i in 0..5 {
        let repeat = run_search(&mut adapter, &identical_vector, 5);
        assert_eq!(first, repeat, "tie-break order differed on repetition {i}");
    }
}
