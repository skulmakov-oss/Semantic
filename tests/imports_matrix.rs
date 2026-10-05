use sm_sema::{check_file_with_provider, ModuleProvider};
use std::collections::BTreeSet;
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

#[test]
fn imports_policy_matrix() {
    let root = Path::new("tests/fixtures/imports");
    let provider = FsProvider;
    let mut seen = BTreeSet::<String>::new();

    for case in fixture_dirs(root) {
        let expect = fs::read_to_string(case.join("EXPECT"))
            .expect("EXPECT")
            .trim()
            .to_string();
        let entry = case.join("main.sm").canonicalize().expect("main.sm");
        let res = check_file_with_provider(&entry, &provider);

        if expect == "OK" {
            seen.insert("OK".to_string());
            if let Err(e) = res {
                panic!("case '{}' expected OK, got: {}", case.display(), e);
            }
            continue;
        }

        let code = expect.strip_prefix("ERR ").expect("ERR format");
        seen.insert(code.to_string());
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
                let expect_substr = case.join("EXPECT_SUBSTR");
                if expect_substr.exists() {
                    let content = fs::read_to_string(&expect_substr).expect("EXPECT_SUBSTR");
                    for needle in content.lines().map(|l| l.trim()).filter(|l| !l.is_empty()) {
                        assert!(
                            text.contains(needle),
                            "case '{}': expected diagnostic to contain '{}', got:\n{}",
                            case.display(),
                            needle,
                            text
                        );
                    }
                }
            }
        }
    }

    for required in ["OK", "E0241", "E0242", "E0243", "E0244", "E0245"] {
        assert!(
            seen.contains(required),
            "imports matrix missing coverage for {}",
            required
        );
    }
}
