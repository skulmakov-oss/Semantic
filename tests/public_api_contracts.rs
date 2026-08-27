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
        "crates/sm-emit/src/lib.rs",
        "tests/golden_snapshots/public_api/sm_emit_lib.txt",
    ),
    (
        "crates/sm-emit/src/hello_real_semcode.rs",
        "tests/golden_snapshots/public_api/sm_emit_hello_real_semcode.txt",
    ),
    (
        "crates/sm-emit/src/hello_observation_bytes.rs",
        "tests/golden_snapshots/public_api/sm_emit_hello_observation_bytes.txt",
    ),
    (
        "crates/sm-format/src/lib.rs",
        "tests/golden_snapshots/public_api/sm_format_lib.txt",
    ),
    (
        "crates/sm-format/src/local_format.rs",
        "tests/golden_snapshots/public_api/sm_format_local_format.txt",
    ),
    (
        "crates/sm-format/src/semcode_decode.rs",
        "tests/golden_snapshots/public_api/sm_format_semcode_decode.txt",
    ),
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
        "crates/sm-verify/src/hello_pending_admission.rs",
        "tests/golden_snapshots/public_api/sm_verify_hello_pending_admission.txt",
    ),
    (
        "crates/sm-verify/src/hello_real_semcode_admission.rs",
        "tests/golden_snapshots/public_api/sm_verify_hello_real_semcode_admission.txt",
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
    (
        "crates/semantic-hub/src/lib.rs",
        "tests/golden_snapshots/public_api/semantic_hub_lib.txt",
    ),
    (
        "crates/semantic-hub-turbovec/src/lib.rs",
        "tests/golden_snapshots/public_api/semantic_hub_turbovec_lib.txt",
    ),
];

fn normalize_ws(line: &str) -> String {
    line.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LexState {
    Normal,
    BlockComment(usize),
    RawString(usize),
    NormalString,
}

struct CodeSanitizer {
    state: LexState,
}

impl CodeSanitizer {
    fn new() -> Self {
        Self {
            state: LexState::Normal,
        }
    }

    fn clean_line(&mut self, line: &str) -> String {
        let mut out = String::with_capacity(line.len());
        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            match self.state {
                LexState::Normal => {
                    if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
                        break;
                    } else if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
                        self.state = LexState::BlockComment(1);
                        i += 2;
                    } else if chars[i] == '"' {
                        self.state = LexState::NormalString;
                        out.push('"');
                        i += 1;
                    } else if (chars[i] == 'r' || chars[i] == 'b' || chars[i] == 'c')
                        && i + 1 < chars.len()
                        && (chars[i + 1] == '"'
                            || chars[i + 1] == '#'
                            || (chars[i] == 'b' && (chars[i + 1] == 'r' || chars[i + 1] == '"')))
                    {
                        let mut j = i;
                        if chars[j] == 'b' || chars[j] == 'c' {
                            j += 1;
                        }
                        if j < chars.len() && chars[j] == 'r' {
                            let r_start = i;
                            j += 1;
                            let mut hashes = 0;
                            while j < chars.len() && chars[j] == '#' {
                                hashes += 1;
                                j += 1;
                            }
                            if j < chars.len() && chars[j] == '"' {
                                self.state = LexState::RawString(hashes);
                                out.extend(&chars[r_start..=j]);
                                i = j + 1;
                                continue;
                            }
                        } else if j < chars.len() && chars[j] == '"' {
                            self.state = LexState::NormalString;
                            out.extend(&chars[i..=j]);
                            i = j + 1;
                            continue;
                        }
                        out.push(chars[i]);
                        i += 1;
                    } else if chars[i] == '\'' {
                        if i + 1 < chars.len()
                            && chars[i + 1] == '\\'
                            && i + 3 < chars.len()
                            && chars[i + 3] == '\''
                        {
                            out.extend(&chars[i..=i + 3]);
                            i += 4;
                        } else if i + 2 < chars.len() && chars[i + 2] == '\'' {
                            out.extend(&chars[i..=i + 2]);
                            i += 3;
                        } else {
                            out.push(chars[i]);
                            i += 1;
                        }
                    } else {
                        out.push(chars[i]);
                        i += 1;
                    }
                }
                LexState::BlockComment(depth) => {
                    if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
                        self.state = LexState::BlockComment(depth + 1);
                        i += 2;
                    } else if chars[i] == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                        if depth <= 1 {
                            self.state = LexState::Normal;
                        } else {
                            self.state = LexState::BlockComment(depth - 1);
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                LexState::NormalString => {
                    out.push(chars[i]);
                    if chars[i] == '\\' && i + 1 < chars.len() {
                        out.push(chars[i + 1]);
                        i += 2;
                    } else if chars[i] == '"' {
                        self.state = LexState::Normal;
                        i += 1;
                    } else {
                        i += 1;
                    }
                }
                LexState::RawString(hashes) => {
                    out.push(chars[i]);
                    if chars[i] == '"' {
                        let mut j = i + 1;
                        let mut match_hashes = 0;
                        while j < chars.len() && match_hashes < hashes && chars[j] == '#' {
                            match_hashes += 1;
                            j += 1;
                        }
                        if match_hashes == hashes {
                            out.extend(&chars[(i + 1)..j]);
                            self.state = LexState::Normal;
                            i = j;
                            continue;
                        }
                    }
                    i += 1;
                }
            }
        }

        out
    }

    fn depth_delta_of_code(code: &str) -> i32 {
        let mut d = 0i32;
        let mut in_str = false;
        let mut in_raw: Option<usize> = None;
        let chars: Vec<char> = code.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if in_str {
                if chars[i] == '\\' {
                    i += 2;
                } else if chars[i] == '"' {
                    in_str = false;
                    i += 1;
                } else {
                    i += 1;
                }
            } else if let Some(hashes) = in_raw {
                if chars[i] == '"' {
                    let mut j = i + 1;
                    let mut match_hashes = 0;
                    while j < chars.len() && match_hashes < hashes && chars[j] == '#' {
                        match_hashes += 1;
                        j += 1;
                    }
                    if match_hashes == hashes {
                        in_raw = None;
                        i = j;
                        continue;
                    }
                }
                i += 1;
            } else if chars[i] == '"' {
                in_str = true;
                i += 1;
            } else if (chars[i] == 'r' || chars[i] == 'b' || chars[i] == 'c') && i + 1 < chars.len()
            {
                let mut j = i;
                if chars[j] == 'b' || chars[j] == 'c' {
                    j += 1;
                }
                if j < chars.len() && chars[j] == 'r' {
                    j += 1;
                    let mut hashes = 0;
                    while j < chars.len() && chars[j] == '#' {
                        hashes += 1;
                        j += 1;
                    }
                    if j < chars.len() && chars[j] == '"' {
                        in_raw = Some(hashes);
                        i = j + 1;
                        continue;
                    }
                }
                match chars[i] {
                    '{' | '[' | '(' => d += 1,
                    '}' | ']' | ')' => d -= 1,
                    _ => {}
                }
                i += 1;
            } else {
                match chars[i] {
                    '{' | '[' | '(' => d += 1,
                    '}' | ']' | ')' => d -= 1,
                    _ => {}
                }
                i += 1;
            }
        }
        d
    }
}

fn is_public_item(line: &str) -> bool {
    line.starts_with("pub ") || line.starts_with("pub(")
}

fn is_public_fn(line: &str) -> bool {
    let is_pub = is_public_item(line);
    if !is_pub {
        return false;
    }
    line.starts_with("pub fn ")
        || line.starts_with("pub const fn ")
        || line.starts_with("pub async fn ")
        || line.starts_with("pub unsafe fn ")
        || line.starts_with("pub extern ")
        || line.starts_with("pub unsafe extern ")
        || line.starts_with("pub(crate) fn ")
        || line.starts_with("pub(crate) const fn ")
        || line.starts_with("pub(crate) async fn ")
        || line.starts_with("pub(crate) unsafe fn ")
        || line.contains(" fn ")
}

fn is_public_enum(line: &str) -> bool {
    let is_pub = is_public_item(line);
    if !is_pub {
        return false;
    }
    line.starts_with("pub enum ") || line.starts_with("pub(crate) enum ") || line.contains(" enum ")
}

fn is_public_const_or_static(line: &str) -> bool {
    let is_pub = is_public_item(line);
    if !is_pub {
        return false;
    }
    (line.starts_with("pub const ")
        || line.starts_with("pub static ")
        || line.starts_with("pub(crate) const ")
        || line.starts_with("pub(crate) static "))
        && !line.starts_with("pub const fn ")
        && !line.starts_with("pub(crate) const fn ")
        && !line.contains(" const fn ")
}

fn normalized_public_surface(path: &str) -> String {
    let src = fs::read_to_string(path).unwrap_or_else(|err| panic!("read {path}: {err}"));
    normalized_public_surface_str(path, &src)
}

fn normalized_public_surface_str(path: &str, src: &str) -> String {
    let src_lines: Vec<&str> = src.lines().collect();
    let mut lines = Vec::new();
    let mut pending_attrs = Vec::new();
    let mut idx = 0usize;

    while idx < src_lines.len() {
        let raw_line = src_lines[idx].trim();
        if raw_line.is_empty() {
            idx += 1;
            continue;
        }
        if raw_line.starts_with("#[") {
            pending_attrs.push(normalize_ws(raw_line));
            idx += 1;
            continue;
        }
        if is_public_item(raw_line) {
            lines.append(&mut pending_attrs);
            if is_public_fn(raw_line) {
                let mut sanitizer = CodeSanitizer::new();
                let clean_first = sanitizer.clean_line(raw_line);
                let mut signature = normalize_ws(&clean_first);
                let mut s = clean_first.clone();
                while !s.ends_with('{') && !s.ends_with(';') && idx + 1 < src_lines.len() {
                    idx += 1;
                    let continuation = src_lines[idx].trim();
                    let clean = sanitizer.clean_line(continuation);
                    if continuation.is_empty() || clean.trim().is_empty() {
                        continue;
                    }
                    signature.push(' ');
                    signature.push_str(&normalize_ws(&clean));
                    s = clean;
                }
                lines.push(signature);
                idx += 1;
                continue;
            }

            if is_public_enum(raw_line) {
                let mut sanitizer = CodeSanitizer::new();
                let clean_first = sanitizer.clean_line(raw_line);
                let mut enum_decl = normalize_ws(&clean_first);
                let mut s = clean_first.clone();
                while !s.contains('{') && idx + 1 < src_lines.len() {
                    idx += 1;
                    let continuation = src_lines[idx].trim();
                    let clean = sanitizer.clean_line(continuation);
                    if continuation.is_empty() || clean.trim().is_empty() {
                        continue;
                    }
                    enum_decl.push(' ');
                    enum_decl.push_str(&normalize_ws(&clean));
                    s = clean;
                }
                lines.push(enum_decl);

                let mut depth = 1i32;
                while depth > 0 && idx + 1 < src_lines.len() {
                    idx += 1;
                    let item_line = src_lines[idx].trim();
                    if item_line.is_empty() {
                        continue;
                    }
                    let clean = sanitizer.clean_line(item_line);
                    if clean.trim().is_empty() && sanitizer.state == LexState::Normal {
                        continue;
                    }
                    if item_line.starts_with("#[") && sanitizer.state == LexState::Normal {
                        lines.push(normalize_ws(item_line));
                        continue;
                    }
                    depth += CodeSanitizer::depth_delta_of_code(&clean);
                    if depth == 0 {
                        let clean_trimmed = clean.trim_end_matches('}').trim();
                        if !clean_trimmed.is_empty() {
                            lines.push(normalize_ws(clean_trimmed));
                        }
                        break;
                    } else {
                        lines.push(normalize_ws(&clean));
                    }
                }
                idx += 1;
                continue;
            }

            if is_public_const_or_static(raw_line) {
                let mut sanitizer = CodeSanitizer::new();
                let clean_first = sanitizer.clean_line(raw_line);
                let mut item = normalize_ws(&clean_first);
                let mut depth = CodeSanitizer::depth_delta_of_code(&clean_first);
                let mut has_semi_at_zero = depth <= 0 && clean_first.contains(';');

                while !has_semi_at_zero && idx + 1 < src_lines.len() {
                    idx += 1;
                    let next_line = src_lines[idx].trim();
                    if next_line.is_empty() {
                        continue;
                    }
                    let clean = sanitizer.clean_line(next_line);
                    depth += CodeSanitizer::depth_delta_of_code(&clean);
                    if depth <= 0 && clean.contains(';') {
                        has_semi_at_zero = true;
                    }
                    if clean.trim().is_empty() {
                        continue;
                    }
                    item.push(' ');
                    item.push_str(&normalize_ws(&clean));
                }
                lines.push(normalize_ws(&item));
                idx += 1;
                continue;
            }

            let mut sanitizer = CodeSanitizer::new();
            let clean_first = sanitizer.clean_line(raw_line);
            let mut item = normalize_ws(&clean_first);
            let mut depth = CodeSanitizer::depth_delta_of_code(&clean_first);

            let is_complete = |it: &str, d: i32| {
                it.ends_with('{')
                    || (d <= 0 && (it.ends_with(';') || it.ends_with(',') || it.ends_with('}')))
            };

            while !is_complete(&item, depth) && idx + 1 < src_lines.len() {
                idx += 1;
                let continuation = src_lines[idx].trim();
                if continuation.is_empty() {
                    continue;
                }
                let clean = sanitizer.clean_line(continuation);
                depth += CodeSanitizer::depth_delta_of_code(&clean);
                if clean.trim().is_empty() {
                    continue;
                }
                item.push(' ');
                item.push_str(&normalize_ws(&clean));
            }
            lines.push(normalize_ws(&item));
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

fn update_mode() -> bool {
    std::env::var("SM_UPDATE_PUBLIC_API_SNAPSHOTS")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

#[test]
fn public_api_inventory_matches_checked_in_contract_snapshots() {
    for (source, snapshot) in TARGETS {
        let actual = normalized_public_surface(source).trim_end().to_string();
        if update_mode() {
            fs::write(snapshot, format!("{actual}\n"))
                .unwrap_or_else(|err| panic!("write {snapshot}: {err}"));
            continue;
        }
        let expected =
            fs::read_to_string(snapshot).unwrap_or_else(|err| panic!("read {snapshot}: {err}"));
        assert_eq!(
            actual,
            normalize_snapshot_text(&expected),
            "public API inventory drifted for {source}; update snapshot only for intentional contract changes"
        );
    }
}

fn verification_code_contract_name(code: sm_verify::VerificationCode) -> &'static str {
    use sm_verify::VerificationCode;

    match code {
        VerificationCode::BadHeader => "BadHeader",
        VerificationCode::UnsupportedVersion => "UnsupportedVersion",
        VerificationCode::TruncatedFunction => "TruncatedFunction",
        VerificationCode::InvalidFunctionName => "InvalidFunctionName",
        VerificationCode::DuplicateFunction => "DuplicateFunction",
        VerificationCode::InvalidStringTable => "InvalidStringTable",
        VerificationCode::InvalidDebugSection => "InvalidDebugSection",
        VerificationCode::InvalidOwnershipSection => "InvalidOwnershipSection",
        VerificationCode::UnknownOpcode => "UnknownOpcode",
        VerificationCode::OperandOutOfBounds => "OperandOutOfBounds",
        VerificationCode::InvalidJumpTarget => "InvalidJumpTarget",
        VerificationCode::InvalidStringReference => "InvalidStringReference",
        VerificationCode::InvalidRegisterReference => "InvalidRegisterReference",
        VerificationCode::UnknownCallTarget => "UnknownCallTarget",
        VerificationCode::ResourceLimitExceeded => "ResourceLimitExceeded",
        VerificationCode::CapabilityViolation => "CapabilityViolation",
        VerificationCode::AmbiguousInstructionFraming => "AmbiguousInstructionFraming",
        VerificationCode::OpcodeRequiresNewerHeader => "OpcodeRequiresNewerHeader",
        VerificationCode::ReachableFunctionFallthrough => "ReachableFunctionFallthrough",
        VerificationCode::InvalidSignatureSection => "InvalidSignatureSection",
        VerificationCode::CallArgumentCountMismatch => "CallArgumentCountMismatch",
        VerificationCode::UndefinedRegisterRead => "UndefinedRegisterRead",
        VerificationCode::AnalysisStateLimitExceeded => "AnalysisStateLimitExceeded",
        VerificationCode::AnalysisWorkLimitExceeded => "AnalysisWorkLimitExceeded",
    }
}

#[test]
fn verification_code_variants_match_public_contract() {
    use sm_verify::VerificationCode;

    let variants = [
        VerificationCode::BadHeader,
        VerificationCode::UnsupportedVersion,
        VerificationCode::TruncatedFunction,
        VerificationCode::InvalidFunctionName,
        VerificationCode::DuplicateFunction,
        VerificationCode::InvalidStringTable,
        VerificationCode::InvalidDebugSection,
        VerificationCode::InvalidOwnershipSection,
        VerificationCode::UnknownOpcode,
        VerificationCode::OperandOutOfBounds,
        VerificationCode::InvalidJumpTarget,
        VerificationCode::InvalidStringReference,
        VerificationCode::InvalidRegisterReference,
        VerificationCode::UnknownCallTarget,
        VerificationCode::ResourceLimitExceeded,
        VerificationCode::CapabilityViolation,
        VerificationCode::AmbiguousInstructionFraming,
        VerificationCode::OpcodeRequiresNewerHeader,
        VerificationCode::ReachableFunctionFallthrough,
        VerificationCode::InvalidSignatureSection,
        VerificationCode::CallArgumentCountMismatch,
        VerificationCode::UndefinedRegisterRead,
        VerificationCode::AnalysisStateLimitExceeded,
        VerificationCode::AnalysisWorkLimitExceeded,
    ];

    for variant in variants {
        assert_eq!(
            verification_code_contract_name(variant),
            format!("{variant:?}")
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

#[test]
fn public_api_guard_captures_enum_variants_and_detects_drift() {
    let base_enum = r#"
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Demo {
            Alpha,
            Beta,
        }
    "#;
    let surface_base = normalized_public_surface_str("demo.rs", base_enum);
    assert!(
        surface_base.contains("Alpha"),
        "surface must contain variant Alpha"
    );
    assert!(
        surface_base.contains("Beta"),
        "surface must contain variant Beta"
    );

    // 1. Variant rename (Beta -> Gamma)
    let renamed = r#"
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Demo {
            Alpha,
            Gamma,
        }
    "#;
    assert_ne!(
        surface_base,
        normalized_public_surface_str("demo.rs", renamed),
        "variant rename must alter public API surface"
    );

    // 2. Variant addition
    let added = r#"
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Demo {
            Alpha,
            Beta,
            Gamma,
        }
    "#;
    assert_ne!(
        surface_base,
        normalized_public_surface_str("demo.rs", added),
        "variant addition must alter public API surface"
    );

    // 3. Variant removal
    let removed = r#"
        #[derive(Debug, Clone, Copy, PartialEq, Eq)]
        pub enum Demo {
            Alpha,
        }
    "#;
    assert_ne!(
        surface_base,
        normalized_public_surface_str("demo.rs", removed),
        "variant removal must alter public API surface"
    );

    // 4. Discriminant change
    let disc_1 = r#"
        pub enum Disc {
            A = 0x01,
            B = 0x02,
        }
    "#;
    let disc_2 = r#"
        pub enum Disc {
            A = 0x01,
            B = 0x03,
        }
    "#;
    let surface_disc_1 = normalized_public_surface_str("disc.rs", disc_1);
    let surface_disc_2 = normalized_public_surface_str("disc.rs", disc_2);
    assert!(
        surface_disc_1.contains("B = 0x02"),
        "discriminant must be captured"
    );
    assert_ne!(
        surface_disc_1, surface_disc_2,
        "discriminant change must alter public API surface"
    );

    // 5. Tuple variant field change
    let tuple_1 = r#"
        pub enum Tup {
            Val(u32),
        }
    "#;
    let tuple_2 = r#"
        pub enum Tup {
            Val(u64),
        }
    "#;
    let surface_tup_1 = normalized_public_surface_str("tup.rs", tuple_1);
    let surface_tup_2 = normalized_public_surface_str("tup.rs", tuple_2);
    assert!(
        surface_tup_1.contains("Val(u32)"),
        "tuple variant field must be captured"
    );
    assert_ne!(
        surface_tup_1, surface_tup_2,
        "tuple variant field shape change must alter public API surface"
    );

    // 6. Struct variant field change
    let struct_1 = r#"
        pub enum StructVar {
            Payload {
                tag: u16,
                count: u32,
            },
        }
    "#;
    let struct_2 = r#"
        pub enum StructVar {
            Payload {
                tag: u16,
                count: u64,
            },
        }
    "#;
    let surface_struct_1 = normalized_public_surface_str("struct_var.rs", struct_1);
    let surface_struct_2 = normalized_public_surface_str("struct_var.rs", struct_2);
    assert!(
        surface_struct_1.contains("count: u32"),
        "struct variant field must be captured"
    );
    assert_ne!(
        surface_struct_1, surface_struct_2,
        "struct variant field shape change must alter public API surface"
    );
}

#[test]
fn public_api_guard_captures_multiline_const_and_detects_value_drift() {
    let const_1 = r#"
        pub const HEADER_DEMO: Spec = Spec {
            epoch: 0,
            rev: 1,
            capabilities: CAP_A | CAP_B,
        };
    "#;
    let const_2 = r#"
        pub const HEADER_DEMO: Spec = Spec {
            epoch: 0,
            rev: 2,
            capabilities: CAP_A | CAP_B,
        };
    "#;
    let surface_const_1 = normalized_public_surface_str("const.rs", const_1);
    let surface_const_2 = normalized_public_surface_str("const.rs", const_2);
    assert!(
        surface_const_1.contains("rev: 1"),
        "const struct literal field value must be captured"
    );
    assert_ne!(
        surface_const_1, surface_const_2,
        "const struct literal field value drift must alter public API surface"
    );

    let arr_1 = r#"
        pub const BYTES: [u8; 3] = [
            1,
            2,
            3,
        ];
    "#;
    let arr_2 = r#"
        pub const BYTES: [u8; 3] = [
            1,
            9,
            3,
        ];
    "#;
    let surface_arr_1 = normalized_public_surface_str("arr.rs", arr_1);
    let surface_arr_2 = normalized_public_surface_str("arr.rs", arr_2);
    assert!(
        surface_arr_1.contains("2"),
        "const array element must be captured"
    );
    assert_ne!(
        surface_arr_1, surface_arr_2,
        "const array element drift must alter public API surface"
    );
}

#[test]
fn public_api_guard_does_not_overcapture_function_implementation_bodies() {
    let fn_impl_a = r#"
        pub fn compute_hash(seed: u64) -> u64 {
            let mut state = seed.wrapping_add(0x9e3779b97f4a7c15);
            state ^= state >> 30;
            state
        }
    "#;
    let fn_impl_b = r#"
        pub fn compute_hash(seed: u64) -> u64 {
            // completely different private implementation
            let state = seed.rotate_left(13);
            internal_helper(state);
            state
        }
    "#;
    let surface_a = normalized_public_surface_str("fn_demo.rs", fn_impl_a);
    let surface_b = normalized_public_surface_str("fn_demo.rs", fn_impl_b);

    assert_eq!(
        surface_a, surface_b,
        "changing private function body must not alter public API inventory"
    );
    assert!(
        !surface_a.contains("wrapping_add"),
        "function body internals must not be in public API inventory"
    );
    assert!(
        !surface_b.contains("rotate_left"),
        "function body internals must not be in public API inventory"
    );
}

#[test]
fn public_api_guard_handles_complex_lexical_literals_and_comments() {
    // A. Raw string const with braces and delimiters inside
    let src_a = r##"
        pub const X: &str = r#"{ not a Rust block }"#;
        pub const NEXT_A: u32 = 10;
    "##;
    let surface_a = normalized_public_surface_str("case_a.rs", src_a);
    assert!(
        surface_a.contains(r##"pub const X: &str = r#"{ not a Rust block }"#;"##),
        "raw string literal const must be captured: {surface_a}"
    );
    assert!(
        surface_a.contains("pub const NEXT_A: u32 = 10;"),
        "subsequent public const must be recognized: {surface_a}"
    );

    // B. Raw string with unmatched-looking delimiter
    let src_b = r##"
        pub const X: &str = r#"}"#;
        pub const NEXT_B: u32 = 20;
    "##;
    let surface_b = normalized_public_surface_str("case_b.rs", src_b);
    assert!(
        surface_b.contains(r##"pub const X: &str = r#"}"#;"##),
        "raw string with unmatched brace must be captured: {surface_b}"
    );
    assert!(
        surface_b.contains("pub const NEXT_B: u32 = 20;"),
        "subsequent public const must be recognized: {surface_b}"
    );

    // C. Higher-hash raw string
    let src_c = r###"
        pub const X: &str = r##"{ \" }"##;
        pub const NEXT_C: u32 = 30;
    "###;
    let surface_c = normalized_public_surface_str("case_c.rs", src_c);
    assert!(
        surface_c.contains(r###"pub const X: &str = r##"{ \" }"##;"###),
        "higher-hash raw string must be captured: {surface_c}"
    );
    assert!(
        surface_c.contains("pub const NEXT_C: u32 = 30;"),
        "subsequent public const must be recognized: {surface_c}"
    );

    // D. Raw byte string
    let src_d = r##"
        pub const X: &[u8] = br#"{ }"#;
        pub const NEXT_D: u32 = 40;
    "##;
    let surface_d = normalized_public_surface_str("case_d.rs", src_d);
    assert!(
        surface_d.contains(r##"pub const X: &[u8] = br#"{ }"#;"##),
        "raw byte string must be captured: {surface_d}"
    );
    assert!(
        surface_d.contains("pub const NEXT_D: u32 = 40;"),
        "subsequent public const must be recognized: {surface_d}"
    );

    // E. Block comment inside a public const
    let src_e = r#"
        pub const X: Spec = Spec {
            /* fake } delimiter */
            value: 1,
        };
        pub const NEXT_E: u32 = 50;
    "#;
    let surface_e = normalized_public_surface_str("case_e.rs", src_e);
    assert!(
        surface_e.contains("pub const X: Spec = Spec { value: 1, };"),
        "const with inline block comment must be captured: {surface_e}"
    );
    assert!(
        surface_e.contains("pub const NEXT_E: u32 = 50;"),
        "subsequent public const must be recognized: {surface_e}"
    );

    // F. Multi-line block comment
    let src_f = r#"
        pub const X: Spec = Spec {
            /*
               fake }
               fake {
            */
            value: 1,
        };
        pub const NEXT_F: u32 = 60;
    "#;
    let surface_f = normalized_public_surface_str("case_f.rs", src_f);
    assert!(
        surface_f.contains("pub const X: Spec = Spec { value: 1, };"),
        "const with multi-line block comment must be captured: {surface_f}"
    );
    assert!(
        surface_f.contains("pub const NEXT_F: u32 = 60;"),
        "subsequent public const must be recognized: {surface_f}"
    );

    // G. Public enum with raw strings and block comments
    let src_g = r#"
        pub enum LexEnum {
            Alpha,
            /*
               fake }
            */
            Beta,
        }
        pub const NEXT_G: u32 = 70;
    "#;
    let surface_g = normalized_public_surface_str("case_g.rs", src_g);
    assert!(
        surface_g.contains("Alpha"),
        "enum variant Alpha must be captured: {surface_g}"
    );
    assert!(
        surface_g.contains("Beta"),
        "enum variant Beta must be captured: {surface_g}"
    );
    assert!(
        surface_g.contains("pub const NEXT_G: u32 = 70;"),
        "subsequent public const after enum must be recognized: {surface_g}"
    );
}
