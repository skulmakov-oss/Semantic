use std::fs;

const PROM_REF_WRAPPERS: &[&str] = &[
    "CapabilityRef",
    "ActorRef",
    "SessionRef",
    "ClientRef",
    "RevisionRef",
    "EpochRef",
];

const TARGETS: &[(&str, &str)] = &[
    (
        "crates/sm-ir/src/lib.rs",
        "tests/golden_snapshots/public_api/sm_ir_lib.txt",
    ),
    (
        "crates/sm-profile/src/lib.rs",
        "tests/golden_snapshots/public_api/sm_profile_lib.txt",
    ),
    (
        "crates/sm-runtime-core/src/lib.rs",
        "tests/golden_snapshots/public_api/sm_runtime_core_lib.txt",
    ),
    (
        "crates/sm-verify/src/lib.rs",
        "tests/golden_snapshots/public_api/sm_verify_lib.txt",
    ),
    (
        "crates/sm-vm/src/lib.rs",
        "tests/golden_snapshots/public_api/sm_vm_lib.txt",
    ),
    (
        "crates/sm-vm/src/semcode_vm.rs",
        "tests/golden_snapshots/public_api/sm_vm_semcode_vm.txt",
    ),
    (
        "crates/prom-abi/src/lib.rs",
        "tests/golden_snapshots/public_api/prom_abi_lib.txt",
    ),
    (
        "crates/prom-cap/src/lib.rs",
        "tests/golden_snapshots/public_api/prom_cap_lib.txt",
    ),
    (
        "crates/prom-runtime/src/lib.rs",
        "tests/golden_snapshots/public_api/prom_runtime_lib.txt",
    ),
    (
        "crates/smc-cli/src/lib.rs",
        "tests/golden_snapshots/public_api/smc_cli_lib.txt",
    ),
    (
        "crates/prom-refs/src/lib.rs",
        "tests/golden_snapshots/public_api/prom_refs_lib.txt",
    ),
    (
        "crates/prom-ui/src/shell_bridge.rs",
        "tests/golden_snapshots/public_api/prom_ui_shell_bridge.txt",
    ),
    (
        "crates/prom-ui-runtime/src/shell_player.rs",
        "tests/golden_snapshots/public_api/prom_ui_runtime_shell_player.txt",
    ),
    (
        "crates/prom-ui-runtime/src/reference_admission.rs",
        "tests/golden_snapshots/public_api/prom_ui_runtime_reference_admission.txt",
    ),
    (
        "crates/prom-ui-runtime/src/reference_contour.rs",
        "tests/golden_snapshots/public_api/prom_ui_runtime_reference_contour.txt",
    ),
];

fn normalize_ws(line: &str) -> String {
    line.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn is_public_item(line: &str) -> bool {
    line.starts_with("pub ") || line.starts_with("pub(")
}

fn is_public_fn(line: &str) -> bool {
    is_public_item(line) && (line.starts_with("pub fn") || line.contains(" fn "))
}

fn normalized_public_surface(path: &str) -> String {
    let src = fs::read_to_string(path).unwrap_or_else(|err| panic!("read {path}: {err}"));
    let src_lines: Vec<&str> = src.lines().collect();
    let mut lines = Vec::new();
    let mut pending_attrs = Vec::new();
    let mut idx = 0usize;

    while idx < src_lines.len() {
        let line = src_lines[idx].trim();
        if line.starts_with("#[") {
            pending_attrs.push(normalize_ws(line));
            idx += 1;
            continue;
        }
        if is_public_item(line) {
            lines.append(&mut pending_attrs);
            if is_public_fn(line) {
                let mut signature = normalize_ws(line);
                while !signature.ends_with('{') && !signature.ends_with(';') {
                    idx += 1;
                    if idx >= src_lines.len() {
                        break;
                    }
                    let continuation = src_lines[idx].trim();
                    if continuation.is_empty() {
                        continue;
                    }
                    signature.push(' ');
                    signature.push_str(&normalize_ws(continuation));
                }
                lines.push(signature);
                idx += 1;
                continue;
            }
            lines.push(normalize_ws(line));
            idx += 1;
            continue;
        }
        pending_attrs.clear();
        idx += 1;
    }

    format!(
        "source: {}\n{}\n",
        path.replace('\\', "/"),
        lines.join("\n")
    )
}

fn normalize_snapshot_text(text: &str) -> String {
    text.replace("\r\n", "\n").trim_end().to_string()
}

#[test]
fn public_api_inventory_matches_checked_in_contract_snapshots() {
    for (source, snapshot) in TARGETS {
        let actual = normalized_public_surface(source).trim_end().to_string();
        let expected =
            fs::read_to_string(snapshot).unwrap_or_else(|err| panic!("read {snapshot}: {err}"));
        assert_eq!(
            actual,
            normalize_snapshot_text(&expected),
            "public API inventory drifted for {source}; update snapshot only for intentional contract changes"
        );
    }
}

#[test]
fn prom_refs_forbidden_impl_surface_is_absent() {
    let path = "crates/prom-refs/src/lib.rs";
    let src = std::fs::read_to_string(path).unwrap();

    // 1. Remove comments
    let mut no_comments = String::with_capacity(src.len());
    let mut in_block_comment = false;
    let mut in_line_comment = false;
    let mut chars = src.chars().peekable();
    while let Some(c) = chars.next() {
        if in_block_comment {
            if c == '*' && chars.peek() == Some(&'/') {
                chars.next();
                in_block_comment = false;
            }
        } else if in_line_comment {
            if c == '\n' {
                in_line_comment = false;
                no_comments.push('\n');
            }
        } else if c == '/' && chars.peek() == Some(&'*') {
            chars.next();
            in_block_comment = true;
        } else if c == '/' && chars.peek() == Some(&'/') {
            chars.next();
            in_line_comment = true;
        } else {
            no_comments.push(c);
        }
    }

    // 2. Normalize whitespace
    let mut normalized = no_comments
        .replace("\n", " ")
        .replace("\r", " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");

    // 3. Canonicalize known standard trait prefixes
    let prefixes = [
        "::core::convert::",
        "core::convert::",
        "::std::convert::",
        "std::convert::",
        "::core::ops::",
        "core::ops::",
        "::std::ops::",
        "std::ops::",
        "::core::borrow::",
        "core::borrow::",
        "::std::borrow::",
        "std::borrow::",
        "::core::default::",
        "core::default::",
        "::std::default::",
        "std::default::",
    ];
    for p in prefixes {
        normalized = normalized.replace(p, "");
    }

    let approved_types = [
        "ReferenceToken",
        "CapabilityRef",
        "ActorRef",
        "SessionRef",
        "ClientRef",
        "RevisionRef",
        "EpochRef",
    ];

    let wrappers = PROM_REF_WRAPPERS;

    for t in approved_types {
        let patterns = [
            format!("impl Default for {}", t),
            format!("impl From<u64> for {}", t),
            format!("impl Into<u64> for {}", t),
            format!("impl TryFrom<u64> for {}", t),
            format!("impl TryInto<u64> for {}", t),
            format!("impl From<{}> for u64", t),
            format!("impl Into<{}> for u64", t),
            format!("impl TryFrom<{}> for u64", t),
            format!("impl TryInto<{}> for u64", t),
            format!("impl Deref for {}", t),
            format!("impl AsRef<u64> for {}", t),
            format!("impl Borrow<u64> for {}", t),
        ];

        for pattern in patterns {
            assert!(
                !normalized.contains(&pattern),
                "Forbidden impl pattern found: {}",
                pattern
            );
        }
    }

    // Cross-domain conversions
    let mut generated_pairs = 0;
    let mut checked_patterns = 0;
    for src_wrapper in wrappers {
        for dest_wrapper in wrappers {
            if src_wrapper != dest_wrapper {
                generated_pairs += 1;
                let patterns = [
                    format!("impl From<{}> for {}", src_wrapper, dest_wrapper),
                    format!("impl Into<{}> for {}", dest_wrapper, src_wrapper),
                    format!("impl TryFrom<{}> for {}", src_wrapper, dest_wrapper),
                    format!("impl TryInto<{}> for {}", dest_wrapper, src_wrapper),
                ];
                for pattern in patterns {
                    checked_patterns += 1;
                    assert!(
                        !normalized.contains(&pattern),
                        "Forbidden cross-domain conversion pattern found: {}",
                        pattern
                    );
                }
            }
        }
    }
    assert_eq!(generated_pairs, 30, "Should generate 30 cross-domain pairs");
    assert_eq!(checked_patterns, 120, "Should check 120 patterns");

    // Forbidden inherent methods
    let forbidden_methods = [
        "pub fn raw",
        "pub const fn raw",
        "pub fn from_raw",
        "pub const fn from_raw",
        "pub fn resolve",
        "pub const fn resolve",
        "pub fn validate",
        "pub const fn validate",
        "pub fn verify",
        "pub const fn verify",
        "pub fn grant",
        "pub const fn grant",
        "pub fn admit",
        "pub const fn admit",
        "pub fn register",
        "pub const fn register",
        "pub fn serialize",
        "pub const fn serialize",
        "pub fn deserialize",
        "pub const fn deserialize",
    ];
    for method in forbidden_methods {
        assert!(
            !normalized.contains(method),
            "Forbidden public method declaration: {}",
            method
        );
    }

    // Note: this scan protects explicit source-level `impl` contracts only.
    // It does not constitute full semantic or compiler-level negative proof.
    // Alias-based or macro-generated implementations remain outside the textual guard’s proof boundary.
}

fn extract_define_ref_wrapper_names(source: &str) -> Result<Vec<String>, String> {
    let mut names = Vec::new();
    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;

    let mut no_comments_or_strings = String::new();

    while i < chars.len() {
        if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '/' {
            // line comment
            i += 2;
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
            }
        } else if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
            // block comment
            i += 2;
            let mut depth = 1;
            while i < chars.len() && depth > 0 {
                if i + 1 < chars.len() && chars[i] == '/' && chars[i + 1] == '*' {
                    depth += 1;
                    i += 2;
                } else if i + 1 < chars.len() && chars[i] == '*' && chars[i + 1] == '/' {
                    depth -= 1;
                    i += 2;
                } else {
                    i += 1;
                }
            }
        } else if chars[i] == '"' {
            // string literal
            i += 1;
            while i < chars.len() && chars[i] != '"' {
                if chars[i] == '\\' {
                    i += 2;
                } else {
                    i += 1;
                }
            }
            if i < chars.len() {
                i += 1;
            }
        } else if chars[i] == 'r'
            && i + 1 < chars.len()
            && (chars[i + 1] == '"' || chars[i + 1] == '#')
        {
            // raw string literal
            i += 1;
            let mut hash_count = 0;
            while i < chars.len() && chars[i] == '#' {
                hash_count += 1;
                i += 1;
            }
            if i < chars.len() && chars[i] == '"' {
                i += 1;
                loop {
                    if i >= chars.len() {
                        break;
                    }
                    if chars[i] == '"' {
                        let mut closing_hashes = 0;
                        let mut j = i + 1;
                        while j < chars.len() && chars[j] == '#' && closing_hashes < hash_count {
                            closing_hashes += 1;
                            j += 1;
                        }
                        if closing_hashes == hash_count {
                            i = j;
                            break;
                        }
                    }
                    i += 1;
                }
            }
        } else if chars[i] == '\'' {
            // character literal or lifetime
            no_comments_or_strings.push(chars[i]);
            i += 1;
            while i < chars.len() {
                let c = chars[i];
                if c == '\\' {
                    no_comments_or_strings.push(chars[i]);
                    if i + 1 < chars.len() {
                        no_comments_or_strings.push(chars[i + 1]);
                        i += 2;
                    } else {
                        i += 1;
                    }
                } else if c == '\'' {
                    no_comments_or_strings.push(chars[i]);
                    i += 1;
                    break;
                } else if !c.is_alphanumeric() && c != '_' {
                    // if it's a lifetime, there's no closing quote, so break on non-identifier
                    break;
                } else {
                    no_comments_or_strings.push(chars[i]);
                    i += 1;
                }
            }
        } else {
            no_comments_or_strings.push(chars[i]);
            i += 1;
        }
    }

    let clean_chars: Vec<char> = no_comments_or_strings.chars().collect();
    let mut i = 0;
    while i < clean_chars.len() {
        if clean_chars[i].is_alphabetic() || clean_chars[i] == '_' {
            let mut ident = String::new();
            while i < clean_chars.len()
                && (clean_chars[i].is_alphanumeric() || clean_chars[i] == '_')
            {
                ident.push(clean_chars[i]);
                i += 1;
            }
            if ident == "define_ref_wrapper" {
                let mut temp_i = i;
                while temp_i < clean_chars.len() && clean_chars[temp_i].is_whitespace() {
                    temp_i += 1;
                }
                if temp_i < clean_chars.len() && clean_chars[temp_i] == '!' {
                    temp_i += 1;
                    while temp_i < clean_chars.len() && clean_chars[temp_i].is_whitespace() {
                        temp_i += 1;
                    }
                    if temp_i < clean_chars.len()
                        && (clean_chars[temp_i] == '('
                            || clean_chars[temp_i] == '['
                            || clean_chars[temp_i] == '{')
                    {
                        temp_i += 1;
                        while temp_i < clean_chars.len() && clean_chars[temp_i].is_whitespace() {
                            temp_i += 1;
                        }

                        let mut arg = String::new();
                        while temp_i < clean_chars.len()
                            && (clean_chars[temp_i].is_alphanumeric() || clean_chars[temp_i] == '_')
                        {
                            arg.push(clean_chars[temp_i]);
                            temp_i += 1;
                        }

                        let is_valid_ident = !arg.is_empty()
                            && (arg.chars().next().unwrap().is_alphabetic()
                                || arg.starts_with('_'));
                        if !is_valid_ident {
                            return Err(format!(
                                "Invalid identifier in define_ref_wrapper! invocation: '{}'",
                                arg
                            ));
                        }

                        names.push(arg);
                        i = temp_i;
                    }
                }
            }
        } else {
            i += 1;
        }
    }

    Ok(names)
}

#[test]
fn prom_refs_wrapper_invocation_extractor_ignores_non_code_occurrences() {
    let source = r##"
        macro_rules! define_ref_wrapper {
            ($name:ident, $doc:expr) => { ... }
        }

        // define_ref_wrapper!(Fake1, ...);
        /* define_ref_wrapper!(Fake2, ...); */

        let a = "define_ref_wrapper!(Fake3, ...)";
        let b = r#"define_ref_wrapper!(Fake4, ...)"#;

        let _quote = '\'';
        let _plain = 'x';

        // These must remain ignored as non-code text around literal state transitions.
        let _text = "define_ref_wrapper!(FakeStringRef, \"fake\");";

        // Synthetic cases crossing or adjacent to character literal boundaries
        let _cross1 = 'd';efine_ref_wrapper!(FakeCross1, "...");
        let _cross2 = define_ref_wrapper'!';(FakeCross2, "...");

        define_ref_wrapper!(Real1, "doc");
        define_ref_wrapper! {
            Real2,
            "doc"
        }
        define_ref_wrapper![
            BracketRef,
            "real"
        ];
    "##;

    let names = extract_define_ref_wrapper_names(source).unwrap();
    assert_eq!(names, vec!["Real1", "Real2", "BracketRef"]);
}

#[test]
fn prom_refs_wrapper_invocations_match_contract() {
    let path = "crates/prom-refs/src/lib.rs";
    let src = fs::read_to_string(path).unwrap();

    let actual_wrappers = extract_define_ref_wrapper_names(&src).unwrap();

    let mut expected: Vec<String> = PROM_REF_WRAPPERS.iter().map(|s| s.to_string()).collect();
    let mut actual = actual_wrappers.clone();

    expected.sort();
    actual.sort();

    let mut actual_unique = actual.clone();
    actual_unique.dedup();

    if actual.len() != actual_unique.len() {
        println!("expected wrappers: {:?}", expected);
        println!("actual wrappers: {:?}", actual);
        println!(
            "missing wrappers: {:?}",
            expected
                .iter()
                .filter(|w| !actual.contains(w))
                .collect::<Vec<_>>()
        );
        println!(
            "unexpected wrappers: {:?}",
            actual
                .iter()
                .filter(|w| !expected.contains(w))
                .collect::<Vec<_>>()
        );
        println!("duplicate wrappers: {:?}", actual);
        panic!("duplicate wrappers found: {:?}", actual);
    }

    if actual != expected {
        println!("expected wrappers: {:?}", expected);
        println!("actual wrappers: {:?}", actual);

        let missing: Vec<_> = expected.iter().filter(|w| !actual.contains(w)).collect();
        let unexpected: Vec<_> = actual.iter().filter(|w| !expected.contains(w)).collect();

        println!("missing wrappers: {:?}", missing);
        println!("unexpected wrappers: {:?}", unexpected);
        println!("duplicate wrappers: []");

        panic!("Wrapper contract mismatch");
    }

    assert_eq!(actual.len(), 6, "actual invocation count = 6");
    assert_eq!(actual_unique.len(), 6, "actual unique wrapper count = 6");
    assert_eq!(expected.len(), 6, "expected wrapper count = 6");
}
