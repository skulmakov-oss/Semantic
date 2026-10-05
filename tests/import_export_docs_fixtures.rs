use sm_sema::{check_file_with_provider, ModuleProvider};
use std::fs;
use std::path::{Component, Path, PathBuf};

struct FsProvider;

impl ModuleProvider for FsProvider {
    fn read_module(&self, module_id: &str) -> Result<Vec<u8>, String> {
        fs::read(module_id).map_err(|e| e.to_string())
    }

    fn resolve_import(&self, importer_module_id: &str, spec: &str) -> Result<String, String> {
        Ok(resolve_fixture_import(importer_module_id, spec))
    }
}

fn resolve_fixture_import(importer_module_id: &str, spec: &str) -> String {
    let importer = Path::new(importer_module_id);
    let base = importer.parent().unwrap_or_else(|| Path::new("."));
    let mut spec_path = PathBuf::from(spec);
    if spec_path.extension().is_none() {
        spec_path.set_extension("exo");
    }
    let joined = if spec_path.is_absolute() {
        spec_path
    } else {
        base.join(spec_path)
    };
    // PB-04 (#1690, #1691): the provider owns canonical module ids. Lexical
    // cleanup keeps an unresolved leading `..`, the id is lossless UTF-8, and
    // `\` is folded only on Windows, where it is a path separator.
    let mut parts: Vec<Component<'_>> = Vec::new();
    for c in joined.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir if matches!(parts.last(), Some(Component::Normal(_))) => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    let normalized: PathBuf = parts.iter().collect();
    let text = normalized.to_str().expect("fixture paths are UTF-8");
    if cfg!(windows) {
        text.replace('\\', "/")
    } else {
        text.to_string()
    }
}

/// Every directory entry must be readable (#1702): a discovery error fails the
/// test instead of silently shrinking the corpus.
fn fixture_dirs(root: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    for entry in fs::read_dir(root).expect("read fixture root") {
        let p = entry.expect("read fixture entry").path();
        if p.is_dir() {
            out.push(p);
        }
    }
    out.sort();
    out
}

/// PB-04 (#1702): the documented import/export fixture corpus cited by
/// `docs/spec/modules.md`. The executed set must equal this inventory exactly.
const EXPECTED_FIXTURES: [&str; 10] = [
    "fail_alias_import_vs_import_E0241",
    "fail_alias_local_vs_import_E0241",
    "fail_alias_select_as_conflict_E0241",
    "fail_bad_select_alias_E0245",
    "fail_collision_E0242",
    "fail_kind_mismatch_E0245",
    "fail_missing_select_E0244",
    "fail_symbol_cycle_E0243",
    "fail_symbol_cycle_chain_E0243",
    "pass_select_alias",
];

/// The fixture directories under `root`, failing closed when the discovered
/// set differs from [`EXPECTED_FIXTURES`] in either direction.
fn fixture_inventory(root: &Path) -> Result<Vec<PathBuf>, String> {
    let dirs = fixture_dirs(root);
    let found: Vec<String> = dirs
        .iter()
        .map(|d| {
            d.file_name()
                .expect("dir name")
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    let expected: Vec<String> = EXPECTED_FIXTURES.iter().map(|s| s.to_string()).collect();
    if found != expected {
        return Err(format!(
            "fixture inventory drifted: expected {expected:?}, found {found:?}"
        ));
    }
    Ok(dirs)
}

#[test]
fn fixture_inventory_fails_closed_on_missing_or_unexpected_case() {
    let base = std::env::temp_dir().join(format!("pb04_fixture_inventory_{}", std::process::id()));
    let _ = fs::remove_dir_all(&base);
    for name in EXPECTED_FIXTURES.iter().skip(1) {
        fs::create_dir_all(base.join(name)).expect("mkdir");
    }
    assert!(
        fixture_inventory(&base).is_err(),
        "a missing fixture must fail"
    );
    fs::create_dir_all(base.join(EXPECTED_FIXTURES[0])).expect("mkdir");
    assert!(fixture_inventory(&base).is_ok());
    fs::create_dir_all(base.join("zz_unexpected")).expect("mkdir");
    assert!(
        fixture_inventory(&base).is_err(),
        "an unexpected fixture must fail"
    );
    let _ = fs::remove_dir_all(&base);
}

#[test]
fn imports_docs_fixtures() {
    let root = Path::new("tests/fixtures/imports");
    let provider = FsProvider;
    let cases = fixture_inventory(root).unwrap_or_else(|e| panic!("{e}"));
    for case in cases {
        let expect = fs::read_to_string(case.join("EXPECT"))
            .expect("EXPECT")
            .trim()
            .to_string();
        let entry = case.join("main.sm").canonicalize().expect("main.sm");
        let res = check_file_with_provider(&entry, &provider);
        if expect == "OK" {
            if let Err(e) = res {
                panic!("case '{}' expected OK, got: {}", case.display(), e);
            }
            continue;
        }
        let code = expect.strip_prefix("ERR ").expect("ERR format");
        match res {
            Ok(_) => panic!("case '{}' expected {}, got OK", case.display(), code),
            Err(e) => {
                let text = e.to_string();
                assert!(
                    text.contains(&format!("Error [{}]", code)),
                    "case '{}': expected code {}, got:\n{}",
                    case.display(),
                    code,
                    text
                );
            }
        }
    }
}
