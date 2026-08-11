use crate::package_manifest::{
    admit_package_entry_module, find_nearest_manifest, resolve_package_import_path,
};
use std::collections::{BTreeMap, HashMap, HashSet};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CacheEvent {
    Hit,
    Miss,
    Invalidate,
}

impl CacheEvent {
    fn trace_name(self) -> &'static str {
        match self {
            CacheEvent::Hit => "cache_hit",
            CacheEvent::Miss => "cache_miss",
            CacheEvent::Invalidate => "invalidate",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum CacheReason {
    Reused,
    CacheDisabled,
    NotFound,
    HeaderInvalid,
    KindMismatch,
    VersionMismatch,
    ToolchainMismatch,
    FeatureMismatch,
    CapsMismatch,
    PayloadSizeMismatch,
    ChecksumMismatch,
    FingerprintMismatch,
    GraphChanged,
    DenyPolicy,
}

impl CacheReason {
    pub(crate) fn trace_code(self) -> &'static str {
        match self {
            CacheReason::Reused => "REUSED",
            CacheReason::CacheDisabled => "CACHE_DISABLED",
            CacheReason::NotFound => "NOT_FOUND",
            CacheReason::HeaderInvalid => "HEADER_INVALID",
            CacheReason::KindMismatch => "KIND_MISMATCH",
            CacheReason::VersionMismatch => "SCHEMA_CHANGED",
            CacheReason::ToolchainMismatch => "TOOLCHAIN_CHANGED",
            CacheReason::FeatureMismatch => "FEATURES_CHANGED",
            CacheReason::CapsMismatch => "CAPS_CHANGED",
            CacheReason::PayloadSizeMismatch => "CORRUPT_PACK",
            CacheReason::ChecksumMismatch => "CORRUPT_PACK",
            CacheReason::FingerprintMismatch => "SOURCE_CHANGED",
            CacheReason::GraphChanged => "DEP_CHANGED",
            CacheReason::DenyPolicy => "DENY_POLICY",
        }
    }
}

pub(crate) fn emit_trace(
    enabled: bool,
    event: CacheEvent,
    reason: CacheReason,
    module: &Path,
    pack_kind: &str,
    key: &str,
) {
    if !enabled {
        return;
    }
    eprintln!(
        "{{\"event\":\"{}\",\"reason\":\"{}\",\"module\":\"{}\",\"pack_kind\":\"{}\",\"key\":\"{}\"}}",
        event.trace_name(),
        reason.trace_code(),
        escape_json(&module.to_string_lossy()),
        escape_json(pack_kind),
        escape_json(key),
    );
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModuleGraphNode {
    pub(crate) key: String,
    pub(crate) deps: Vec<String>,
    pub(crate) source_hash: u64,
    pub(crate) exports_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ModuleGraphSnapshot {
    nodes: Vec<ModuleGraphNode>,
}

impl ModuleGraphSnapshot {
    pub(crate) fn read_from_root(root: &Path) -> Result<Self, String> {
        let root_canonical = root
            .canonicalize()
            .map_err(|e| format!("resolve '{}': {}", root.display(), e))?;
        let root_base = root_canonical.parent().unwrap_or_else(|| Path::new("."));
        let mut visiting = HashSet::new();
        let mut graph: HashMap<PathBuf, (u64, Vec<PathBuf>)> = HashMap::new();
        collect_module_graph(&root_canonical, root_base, &mut visiting, &mut graph)?;
        let mut items: Vec<(PathBuf, (u64, Vec<PathBuf>))> = graph.into_iter().collect();
        items.sort_by(|a, b| a.0.cmp(&b.0));
        let mut nodes = Vec::with_capacity(items.len());
        for (path, (source_hash, deps)) in items {
            let mut dep_keys: Vec<String> = deps
                .iter()
                .map(|d| canonical_module_key(d, root_base))
                .collect();
            dep_keys.sort();
            let key = canonical_module_key(&path, root_base);
            // v1.1: exports hash still follows source hash; later slices can plug explicit export hashing.
            nodes.push(ModuleGraphNode {
                key,
                deps: dep_keys,
                source_hash,
                exports_hash: source_hash,
            });
        }
        Ok(Self { nodes })
    }

    pub(crate) fn module_count(&self) -> usize {
        self.nodes.len()
    }

    pub(crate) fn hash(&self, schema_version: u32) -> u64 {
        fnv1a64(&self.encode(schema_version))
    }

    pub(crate) fn write_to(&self, path: &Path, schema_version: u32) -> Result<(), String> {
        std::fs::write(path, self.encode(schema_version))
            .map_err(|e| format!("write cache graph '{}': {}", path.display(), e))
    }

    fn encode(&self, schema_version: u32) -> Vec<u8> {
        let mut blob = Vec::new();
        blob.extend_from_slice(
            format!("EXOGRAPH {} {}\n", schema_version, self.nodes.len()).as_bytes(),
        );
        for node in &self.nodes {
            blob.extend_from_slice(node.key.as_bytes());
            blob.push(0);
            blob.extend_from_slice(format!("{:016x}", node.source_hash).as_bytes());
            blob.push(0);
            blob.extend_from_slice(format!("{:016x}", node.exports_hash).as_bytes());
            blob.push(0);
            blob.extend_from_slice(node.deps.join(",").as_bytes());
            blob.push(b'\n');
        }
        blob
    }
}

pub(crate) fn read_graph_hash(path: &Path) -> Option<u64> {
    let bytes = std::fs::read(path).ok()?;
    Some(fnv1a64(&bytes))
}

pub(crate) fn module_graph_fingerprint(root: &Path, schema_version: u32) -> Result<u64, String> {
    Ok(ModuleGraphSnapshot::read_from_root(root)?.hash(schema_version))
}

pub(crate) fn module_graph_module_count(root: &Path) -> Result<usize, String> {
    Ok(ModuleGraphSnapshot::read_from_root(root)?.module_count())
}

pub(crate) fn update_cache_index(
    index_path: &Path,
    root: &Path,
    fingerprint: u64,
    graph_hash: Option<u64>,
    module_count: usize,
) -> Result<(), String> {
    let root_key = root
        .canonicalize()
        .map_err(|e| format!("resolve '{}': {}", root.display(), e))?
        .to_string_lossy()
        .replace('\\', "/");
    let mut entries = BTreeMap::<String, String>::new();
    if let Ok(text) = std::fs::read_to_string(index_path) {
        for line in text.lines() {
            if !line.starts_with("K ") {
                continue;
            }
            if let Some((k, rest)) = line[2..].split_once('\t') {
                entries.insert(k.to_string(), rest.to_string());
            }
        }
    }
    let gh = graph_hash.unwrap_or(0);
    entries.insert(
        root_key,
        format!(
            "FP={:016x}\tGH={:016x}\tMC={}",
            fingerprint, gh, module_count
        ),
    );
    let mut out = String::from("EXOIDX v2\n");
    for (key, value) in entries {
        out.push_str("K ");
        out.push_str(&key);
        out.push('\t');
        out.push_str(&value);
        out.push('\n');
    }
    std::fs::write(index_path, out)
        .map_err(|e| format!("write cache index '{}': {}", index_path.display(), e))
}

fn parse_import_specs(source: &str) -> Vec<String> {
    let mut out = Vec::new();
    for line in source.lines() {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
            continue;
        }
        if !trimmed.starts_with("Import") {
            continue;
        }
        let mut rest = trimmed["Import".len()..].trim();
        if rest.is_empty() {
            continue;
        }
        if let Some(after_pub) = rest.strip_prefix("pub ") {
            rest = after_pub.trim_start();
        }
        let spec = if let Some(stripped) = rest.strip_prefix('"') {
            if let Some(end) = stripped.find('"') {
                stripped[..end].to_string()
            } else {
                stripped.to_string()
            }
        } else {
            rest.split_whitespace().next().unwrap_or("").to_string()
        };
        if !spec.is_empty() {
            out.push(spec);
        }
    }
    out
}

fn canonical_module_key(canonical: &Path, root_base: &Path) -> String {
    if let Ok(rel) = canonical.strip_prefix(root_base) {
        let value = rel.to_string_lossy().replace('\\', "/");
        if value.is_empty() {
            ".".to_string()
        } else {
            value
        }
    } else {
        canonical.to_string_lossy().replace('\\', "/")
    }
}

fn collect_module_graph(
    path: &Path,
    root_base: &Path,
    visiting: &mut HashSet<PathBuf>,
    graph: &mut HashMap<PathBuf, (u64, Vec<PathBuf>)>,
) -> Result<(), String> {
    let canonical = path
        .canonicalize()
        .map_err(|e| format!("resolve '{}': {}", path.display(), e))?;
    admit_package_entry_module(&canonical)
        .map(|_| ())
        .map_err(|e| e.to_string())?;
    if graph.contains_key(&canonical) {
        return Ok(());
    }
    if !visiting.insert(canonical.clone()) {
        return Err(format!(
            "cyclic import while scanning '{}'",
            canonical.display()
        ));
    }
    let source = std::fs::read_to_string(&canonical)
        .map_err(|e| format!("read '{}': {}", canonical.display(), e))?;
    let mut source_hash = fnv1a64(source.as_bytes());
    // Fold the governing manifest's own (small, single-file) content into the module's
    // change fingerprint. Without this, module_graph_fingerprint -- the cheap signal
    // `smc watch` polls every ~600ms to decide whether to rebuild -- has no way to see a
    // change to a declared-but-unimported dependency (added/removed/re-pinned), so admission's
    // full declared-dependency-graph validation (see admit_package_entry_module) would need
    // to run on every idle tick just to catch that case, not only on real rebuilds. Reading
    // one small manifest file is cheap enough to do unconditionally every tick; the full
    // graph walk stays gated on an actual fingerprint change (DL-023).
    if let Some(manifest_path) = find_nearest_manifest(&canonical) {
        if let Ok(manifest_source) = std::fs::read_to_string(&manifest_path) {
            let manifest_hash = fnv1a64(manifest_source.as_bytes());
            let mut combined = Vec::with_capacity(16);
            combined.extend_from_slice(&source_hash.to_le_bytes());
            combined.extend_from_slice(&manifest_hash.to_le_bytes());
            source_hash = fnv1a64(&combined);
        }
    }
    let mut deps = Vec::new();
    for spec in parse_import_specs(&source) {
        let child = resolve_package_import_path(&canonical, &spec).map_err(|e| e.to_string())?;
        let child_canonical = child
            .canonicalize()
            .map_err(|e| format!("resolve '{}': {}", child.display(), e))?;
        deps.push(child_canonical.clone());
        collect_module_graph(&child_canonical, root_base, visiting, graph)?;
    }
    deps.sort();
    deps.dedup();
    let _ = root_base;
    graph.insert(canonical.clone(), (source_hash, deps));
    let _ = visiting.remove(&canonical);
    Ok(())
}

fn fnv1a64(bytes: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for byte in bytes {
        hash ^= *byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn escape_json(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn mk_temp_dir(prefix: &str) -> PathBuf {
        let base = std::env::temp_dir().join(format!(
            "{}_{}_{}",
            prefix,
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
        std::fs::create_dir_all(&base).expect("mkdir");
        base
    }

    #[test]
    fn module_graph_fingerprint_changes_on_dependency_edit() {
        let dir = mk_temp_dir("smc_module_graph_fp");
        let root = dir.join("root.sm");
        let child = dir.join("child.sm");
        std::fs::write(
            &root,
            r#"
Import "child.sm"
Law "R" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write root");
        std::fs::write(
            &child,
            r#"
Law "C" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write child");
        let fp1 = module_graph_fingerprint(&root, 2).expect("fp1");
        std::fs::write(
            &child,
            r#"
Law "C2" [priority 2]:
    When true -> System.recovery()
"#,
        )
        .expect("rewrite child");
        let fp2 = module_graph_fingerprint(&root, 2).expect("fp2");
        assert_ne!(fp1, fp2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn module_graph_fingerprint_tracks_local_path_package_dependencies() {
        let dir = mk_temp_dir("smc_pkg_module_graph_fp");
        let app_src = dir.join("app").join("src");
        let math_src = dir.join("math").join("src");
        std::fs::create_dir_all(&app_src).expect("mkdir app src");
        std::fs::create_dir_all(&math_src).expect("mkdir math src");
        std::fs::write(
            dir.join("app").join("Semantic.package"),
            r#"
format 1
package app
manifest_dir .
module_root src
dep math math ../math
"#,
        )
        .expect("write app manifest");
        std::fs::write(
            dir.join("math").join("Semantic.package"),
            r#"
format 1
package math
manifest_dir .
module_root src
"#,
        )
        .expect("write math manifest");
        let root = app_src.join("main.sm");
        let child = math_src.join("core.sm");
        std::fs::write(
            &root,
            r#"
Import "math::core.sm"
Law "Root" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write root");
        std::fs::write(
            &child,
            r#"
Law "Core" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write child");
        let fp1 = module_graph_fingerprint(&root, 2).expect("fp1");
        std::fs::write(
            &child,
            r#"
Law "Core2" [priority 2]:
    When true -> System.recovery()
"#,
        )
        .expect("rewrite child");
        let fp2 = module_graph_fingerprint(&root, 2).expect("fp2");
        assert_ne!(fp1, fp2);
        let _ = std::fs::remove_dir_all(&dir);
    }

    // `smc watch` polls module_graph_fingerprint every ~600ms to decide whether to
    // rebuild, and only resets admission's declared-dependency-graph cache when the
    // fingerprint actually changes (see cmd_watch in app.rs). Before this fix, the
    // fingerprint tracked source Import edges only, so adding/removing a declared-but-
    // unimported dependency in the entry's own manifest left `fp` unchanged -- meaning
    // `smc watch` would never notice or re-validate that change. collect_module_graph now
    // folds the governing manifest's own content into each module's hash, so this must
    // change `fp` even though `root.sm` never imports the added dependency (DL-023).
    #[test]
    fn module_graph_fingerprint_changes_on_unused_declared_dependency_edit() {
        let dir = mk_temp_dir("smc_module_graph_fp_unused_dep");
        let app_src = dir.join("app").join("src");
        std::fs::create_dir_all(&app_src).expect("mkdir app src");
        std::fs::write(
            dir.join("app").join("Semantic.package"),
            "format 1\npackage app\nmanifest_dir .\nmodule_root src\n",
        )
        .expect("write app manifest without a dependency");
        let root = app_src.join("main.sm");
        std::fs::write(
            &root,
            r#"
Law "Root" [priority 1]:
    When true -> System.recovery()
"#,
        )
        .expect("write root");
        let fp1 = module_graph_fingerprint(&root, 2).expect("fp1");

        // Declare an unused dependency -- root.sm still imports nothing. The dependency
        // must actually exist (DL-021 rejects a missing declared dependency outright,
        // regardless of usage, and module_graph_fingerprint now admits the entry as part
        // of computing the fingerprint).
        let math_src = dir.join("math").join("src");
        std::fs::create_dir_all(&math_src).expect("mkdir math src");
        std::fs::write(
            dir.join("math").join("Semantic.package"),
            "format 1\npackage math\nmanifest_dir .\nmodule_root src\n",
        )
        .expect("write math manifest");
        std::fs::write(math_src.join("core.sm"), "fn core() { return; }\n")
            .expect("write math source");
        std::fs::write(
            dir.join("app").join("Semantic.package"),
            "format 1\npackage app\nmanifest_dir .\nmodule_root src\ndep math math ../math\n",
        )
        .expect("rewrite app manifest with an unused dependency");
        let fp2 = module_graph_fingerprint(&root, 2).expect("fp2");
        assert_ne!(
            fp1, fp2,
            "adding a declared-but-unimported dependency must change the watch fingerprint"
        );
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn cache_index_records_graph_hash_and_module_count() {
        let dir = mk_temp_dir("smc_cache_index");
        let index = dir.join("index.bin");
        let root = dir.join("root.sm");
        std::fs::write(&root, "fn main() { return; }\n").expect("write root");
        update_cache_index(&index, &root, 0x11, Some(0x22), 3).expect("write index");
        let text = std::fs::read_to_string(&index).expect("read index");
        assert!(text.contains("EXOIDX v2"));
        assert!(text.contains("FP=0000000000000011"));
        assert!(text.contains("GH=0000000000000022"));
        assert!(text.contains("MC=3"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn trace_reason_codes_are_stable() {
        let cases = [
            (CacheReason::Reused, "REUSED"),
            (CacheReason::CacheDisabled, "CACHE_DISABLED"),
            (CacheReason::NotFound, "NOT_FOUND"),
            (CacheReason::HeaderInvalid, "HEADER_INVALID"),
            (CacheReason::KindMismatch, "KIND_MISMATCH"),
            (CacheReason::VersionMismatch, "SCHEMA_CHANGED"),
            (CacheReason::ToolchainMismatch, "TOOLCHAIN_CHANGED"),
            (CacheReason::FeatureMismatch, "FEATURES_CHANGED"),
            (CacheReason::CapsMismatch, "CAPS_CHANGED"),
            (CacheReason::PayloadSizeMismatch, "CORRUPT_PACK"),
            (CacheReason::ChecksumMismatch, "CORRUPT_PACK"),
            (CacheReason::FingerprintMismatch, "SOURCE_CHANGED"),
            (CacheReason::GraphChanged, "DEP_CHANGED"),
            (CacheReason::DenyPolicy, "DENY_POLICY"),
        ];
        for (reason, expected) in cases {
            assert_eq!(reason.trace_code(), expected);
        }
    }
}
