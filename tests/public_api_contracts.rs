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
    NormalString,
    RawString(usize),
    ByteNormalString,
    RawByteString(usize),
}

impl LexState {
    fn is_in_string_literal(self) -> bool {
        matches!(
            self,
            LexState::NormalString
                | LexState::RawString(_)
                | LexState::ByteNormalString
                | LexState::RawByteString(_)
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum VisibleSegment {
    Code(String),
    Literal(String),
}

#[derive(Debug, Clone)]
struct ScannedLine {
    segments: Vec<VisibleSegment>,
    code_tokens: String,
    depth_delta: i32,
    has_structural_semicolon: bool,
    has_structural_open_brace: bool,
    has_structural_close_brace: bool,
    open_brace_count: usize,
    close_brace_count: usize,
    has_function_body_open_brace: bool,
    first_fn_body_open_brace_seg: Option<(usize, usize)>,
    ends_in_literal_or_comment: bool,
    ends_in_string_literal: bool,
}

impl ScannedLine {
    fn render_visible(&self) -> String {
        let mut out = String::new();
        for seg in &self.segments {
            match seg {
                VisibleSegment::Code(s) => {
                    let norm = normalize_ws(s);
                    if !norm.is_empty() {
                        if !out.is_empty() && !out.ends_with(' ') {
                            let first = norm.chars().next().unwrap();
                            if first != ';'
                                && first != ','
                                && first != ':'
                                && first != ')'
                                && first != ']'
                                && first != '}'
                                && first != '>'
                            {
                                out.push(' ');
                            }
                        }
                        out.push_str(&norm);
                    }
                }
                VisibleSegment::Literal(s) => {
                    if !out.is_empty() && !out.ends_with(' ') {
                        let last = out.chars().last().unwrap();
                        if last != '('
                            && last != '['
                            && last != '{'
                            && last != '<'
                            && last != '*'
                            && last != '&'
                            && last != '!'
                        {
                            out.push(' ');
                        }
                    }
                    out.push_str(s);
                }
            }
        }
        out
    }

    fn text_up_to_function_body_open_brace(&self) -> String {
        if let Some((target_seg, char_idx)) = self.first_fn_body_open_brace_seg {
            let mut out = String::new();
            for (idx, seg) in self.segments.iter().enumerate() {
                if idx < target_seg {
                    match seg {
                        VisibleSegment::Code(s) => {
                            let norm = normalize_ws(s);
                            if !norm.is_empty() {
                                if !out.is_empty() && !out.ends_with(' ') {
                                    let first = norm.chars().next().unwrap();
                                    if first != ';'
                                        && first != ','
                                        && first != ':'
                                        && first != ')'
                                        && first != ']'
                                        && first != '}'
                                        && first != '>'
                                    {
                                        out.push(' ');
                                    }
                                }
                                out.push_str(&norm);
                            }
                        }
                        VisibleSegment::Literal(s) => {
                            if !out.is_empty() && !out.ends_with(' ') {
                                let last = out.chars().last().unwrap();
                                if last != '('
                                    && last != '['
                                    && last != '{'
                                    && last != '<'
                                    && last != '*'
                                    && last != '&'
                                    && last != '!'
                                {
                                    out.push(' ');
                                }
                            }
                            out.push_str(s);
                        }
                    }
                } else if idx == target_seg {
                    match seg {
                        VisibleSegment::Code(s) => {
                            let prefix = &s[..=char_idx];
                            let norm = normalize_ws(prefix);
                            if !norm.is_empty() {
                                if !out.is_empty() && !out.ends_with(' ') {
                                    let first = norm.chars().next().unwrap();
                                    if first != ';'
                                        && first != ','
                                        && first != ':'
                                        && first != ')'
                                        && first != ']'
                                        && first != '}'
                                        && first != '>'
                                    {
                                        out.push(' ');
                                    }
                                }
                                out.push_str(&norm);
                            }
                        }
                        VisibleSegment::Literal(s) => {
                            if !out.is_empty() && !out.ends_with(' ') {
                                let last = out.chars().last().unwrap();
                                if last != '('
                                    && last != '['
                                    && last != '{'
                                    && last != '<'
                                    && last != '*'
                                    && last != '&'
                                    && last != '!'
                                {
                                    out.push(' ');
                                }
                            }
                            out.push_str(&s[..=char_idx]);
                        }
                    }
                    break;
                }
            }
            out
        } else {
            self.render_visible()
        }
    }
}

#[derive(Debug, Clone)]
struct CodeLexer {
    state: LexState,
    angle_depth: usize,
    paren_depth: usize,
    bracket_depth: usize,
}

impl CodeLexer {
    fn new() -> Self {
        Self {
            state: LexState::Normal,
            angle_depth: 0,
            paren_depth: 0,
            bracket_depth: 0,
        }
    }

    fn reset_top_level_depths(&mut self) {
        self.angle_depth = 0;
        self.paren_depth = 0;
        self.bracket_depth = 0;
    }

    fn scan_line(&mut self, line: &str) -> ScannedLine {
        let mut segments = Vec::new();
        let mut cur_code = String::new();
        let mut code_tokens = String::with_capacity(line.len());
        let mut depth_delta = 0i32;
        let mut has_structural_semicolon = false;
        let mut has_structural_open_brace = false;
        let mut has_structural_close_brace = false;
        let mut open_brace_count = 0usize;
        let mut close_brace_count = 0usize;
        let mut has_function_body_open_brace = false;
        let mut first_fn_body_open_brace_seg = None;

        let chars: Vec<char> = line.chars().collect();
        let mut i = 0;

        while i < chars.len() {
            match self.state {
                LexState::Normal => {
                    // 1. Line comment //
                    if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
                        code_tokens.push(' ');
                        break;
                    }

                    // 2. Block comment start /*
                    if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
                        if !cur_code.is_empty() {
                            segments.push(VisibleSegment::Code(std::mem::take(&mut cur_code)));
                        }
                        self.state = LexState::BlockComment(1);
                        code_tokens.push(' ');
                        i += 2;
                        continue;
                    }

                    // 3. Raw byte string br#"..."# or br"..."
                    if chars[i] == 'b' && i + 1 < chars.len() && chars[i + 1] == 'r' {
                        let mut j = i + 2;
                        let mut hashes = 0;
                        while j < chars.len() && chars[j] == '#' {
                            hashes += 1;
                            j += 1;
                        }
                        if j < chars.len() && chars[j] == '"' {
                            if !cur_code.is_empty() {
                                segments.push(VisibleSegment::Code(std::mem::take(&mut cur_code)));
                            }
                            self.state = LexState::RawByteString(hashes);
                            segments.push(VisibleSegment::Literal(chars[i..=j].iter().collect()));
                            code_tokens.push_str("\"\"");
                            i = j + 1;
                            continue;
                        }
                    }

                    // 4. Raw string r#"..."# or r"..."
                    if chars[i] == 'r' {
                        let mut j = i + 1;
                        let mut hashes = 0;
                        while j < chars.len() && chars[j] == '#' {
                            hashes += 1;
                            j += 1;
                        }
                        if j < chars.len() && chars[j] == '"' {
                            if !cur_code.is_empty() {
                                segments.push(VisibleSegment::Code(std::mem::take(&mut cur_code)));
                            }
                            self.state = LexState::RawString(hashes);
                            segments.push(VisibleSegment::Literal(chars[i..=j].iter().collect()));
                            code_tokens.push_str("\"\"");
                            i = j + 1;
                            continue;
                        }
                    }

                    // 5. Byte string b"..."
                    if chars[i] == 'b' && i + 1 < chars.len() && chars[i + 1] == '"' {
                        if !cur_code.is_empty() {
                            segments.push(VisibleSegment::Code(std::mem::take(&mut cur_code)));
                        }
                        self.state = LexState::ByteNormalString;
                        segments.push(VisibleSegment::Literal(chars[i..=i + 1].iter().collect()));
                        code_tokens.push_str("\"\"");
                        i += 2;
                        continue;
                    }

                    // 6. Normal string "..."
                    if chars[i] == '"' {
                        if !cur_code.is_empty() {
                            segments.push(VisibleSegment::Code(std::mem::take(&mut cur_code)));
                        }
                        self.state = LexState::NormalString;
                        segments.push(VisibleSegment::Literal("\"".to_string()));
                        code_tokens.push_str("\"\"");
                        i += 1;
                        continue;
                    }

                    // 7. Byte char b'...'
                    if chars[i] == 'b' && i + 1 < chars.len() && chars[i + 1] == '\'' {
                        let is_simple_byte_char =
                            i + 3 < chars.len() && chars[i + 2] != '\\' && chars[i + 3] == '\'';
                        let is_esc_byte_char = i + 2 < chars.len() && chars[i + 2] == '\\';
                        if is_simple_byte_char {
                            if !cur_code.is_empty() {
                                segments.push(VisibleSegment::Code(std::mem::take(&mut cur_code)));
                            }
                            segments
                                .push(VisibleSegment::Literal(chars[i..=i + 3].iter().collect()));
                            code_tokens.push_str("b''");
                            i += 4;
                            continue;
                        } else if is_esc_byte_char {
                            let mut j = i + 3;
                            let mut found = false;
                            while j < chars.len() && j <= i + 6 {
                                if chars[j] == '\'' && chars[j - 1] != '\\'
                                    || (j > i + 3
                                        && chars[j] == '\''
                                        && chars[j - 2] == '\\'
                                        && chars[j - 1] == '\\')
                                {
                                    found = true;
                                    break;
                                }
                                j += 1;
                            }
                            if found {
                                if !cur_code.is_empty() {
                                    segments
                                        .push(VisibleSegment::Code(std::mem::take(&mut cur_code)));
                                }
                                segments
                                    .push(VisibleSegment::Literal(chars[i..=j].iter().collect()));
                                code_tokens.push_str("b''");
                                i = j + 1;
                                continue;
                            }
                        }
                    }

                    // 8. Char literal '...'
                    if chars[i] == '\'' {
                        let is_simple_char =
                            i + 2 < chars.len() && chars[i + 1] != '\\' && chars[i + 2] == '\'';
                        let is_esc_char = i + 1 < chars.len() && chars[i + 1] == '\\';
                        if is_simple_char {
                            if !cur_code.is_empty() {
                                segments.push(VisibleSegment::Code(std::mem::take(&mut cur_code)));
                            }
                            segments
                                .push(VisibleSegment::Literal(chars[i..=i + 2].iter().collect()));
                            code_tokens.push_str("''");
                            i += 3;
                            continue;
                        } else if is_esc_char {
                            let mut j = i + 2;
                            let mut found = false;
                            while j < chars.len() && j <= i + 10 {
                                if (chars[j] == '\'' && chars[j - 1] != '\\')
                                    || (j > i + 2
                                        && chars[j] == '\''
                                        && chars[j - 2] == '\\'
                                        && chars[j - 1] == '\\')
                                {
                                    found = true;
                                    break;
                                }
                                j += 1;
                            }
                            if found {
                                if !cur_code.is_empty() {
                                    segments
                                        .push(VisibleSegment::Code(std::mem::take(&mut cur_code)));
                                }
                                segments
                                    .push(VisibleSegment::Literal(chars[i..=j].iter().collect()));
                                code_tokens.push_str("''");
                                i = j + 1;
                                continue;
                            }
                        }
                        cur_code.push('\'');
                        code_tokens.push('\'');
                        i += 1;
                        continue;
                    }

                    // 9. Structural tokens & depth tracking
                    match chars[i] {
                        '<' => {
                            self.angle_depth += 1;
                            cur_code.push('<');
                            code_tokens.push('<');
                        }
                        '>' => {
                            if self.angle_depth > 0 {
                                self.angle_depth -= 1;
                            }
                            cur_code.push('>');
                            code_tokens.push('>');
                        }
                        '(' => {
                            self.paren_depth += 1;
                            depth_delta += 1;
                            cur_code.push('(');
                            code_tokens.push('(');
                        }
                        ')' => {
                            if self.paren_depth > 0 {
                                self.paren_depth -= 1;
                            }
                            depth_delta -= 1;
                            cur_code.push(')');
                            code_tokens.push(')');
                        }
                        '[' => {
                            self.bracket_depth += 1;
                            depth_delta += 1;
                            cur_code.push('[');
                            code_tokens.push('[');
                        }
                        ']' => {
                            if self.bracket_depth > 0 {
                                self.bracket_depth -= 1;
                            }
                            depth_delta -= 1;
                            cur_code.push(']');
                            code_tokens.push(']');
                        }
                        '{' => {
                            depth_delta += 1;
                            open_brace_count += 1;
                            has_structural_open_brace = true;
                            let is_fn_body = self.paren_depth == 0
                                && self.angle_depth == 0
                                && self.bracket_depth == 0;
                            if is_fn_body && !has_function_body_open_brace {
                                has_function_body_open_brace = true;
                                let seg_idx = segments.len();
                                let char_idx = cur_code.len();
                                first_fn_body_open_brace_seg = Some((seg_idx, char_idx));
                            }
                            cur_code.push('{');
                            code_tokens.push('{');
                        }
                        '}' => {
                            depth_delta -= 1;
                            close_brace_count += 1;
                            has_structural_close_brace = true;
                            cur_code.push('}');
                            code_tokens.push('}');
                        }
                        ';' => {
                            has_structural_semicolon = true;
                            cur_code.push(';');
                            code_tokens.push(';');
                        }
                        c => {
                            cur_code.push(c);
                            code_tokens.push(c);
                        }
                    }
                    i += 1;
                }
                LexState::BlockComment(depth) => {
                    if chars[i] == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
                        self.state = LexState::BlockComment(depth + 1);
                        i += 2;
                    } else if chars[i] == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                        if depth <= 1 {
                            self.state = LexState::Normal;
                            code_tokens.push(' ');
                        } else {
                            self.state = LexState::BlockComment(depth - 1);
                        }
                        i += 2;
                    } else {
                        i += 1;
                    }
                }
                LexState::NormalString | LexState::ByteNormalString => {
                    let mut lit_str = String::new();
                    while i < chars.len() {
                        if chars[i] == '\\' && i + 1 < chars.len() {
                            lit_str.push(chars[i]);
                            lit_str.push(chars[i + 1]);
                            i += 2;
                        } else if chars[i] == '"' {
                            lit_str.push('"');
                            self.state = LexState::Normal;
                            i += 1;
                            break;
                        } else {
                            lit_str.push(chars[i]);
                            i += 1;
                        }
                    }
                    if let Some(VisibleSegment::Literal(prev)) = segments.last_mut() {
                        prev.push_str(&lit_str);
                    } else {
                        segments.push(VisibleSegment::Literal(lit_str));
                    }
                }
                LexState::RawString(hashes) | LexState::RawByteString(hashes) => {
                    let mut lit_str = String::new();
                    while i < chars.len() {
                        if chars[i] == '"' {
                            let mut j = i + 1;
                            let mut match_hashes = 0;
                            while j < chars.len() && match_hashes < hashes && chars[j] == '#' {
                                match_hashes += 1;
                                j += 1;
                            }
                            if match_hashes == hashes {
                                lit_str.push('"');
                                for _ in 0..hashes {
                                    lit_str.push('#');
                                }
                                self.state = LexState::Normal;
                                i = j;
                                break;
                            }
                        }
                        lit_str.push(chars[i]);
                        i += 1;
                    }
                    if let Some(VisibleSegment::Literal(prev)) = segments.last_mut() {
                        prev.push_str(&lit_str);
                    } else {
                        segments.push(VisibleSegment::Literal(lit_str));
                    }
                }
            }
        }

        if !cur_code.is_empty() {
            segments.push(VisibleSegment::Code(cur_code));
        }

        ScannedLine {
            segments,
            code_tokens,
            depth_delta,
            has_structural_semicolon,
            has_structural_open_brace,
            has_structural_close_brace,
            open_brace_count,
            close_brace_count,
            has_function_body_open_brace,
            first_fn_body_open_brace_seg,
            ends_in_literal_or_comment: self.state != LexState::Normal,
            ends_in_string_literal: self.state.is_in_string_literal(),
        }
    }
}

fn parse_public_item(code_tokens: &str) -> Option<(&str, &str)> {
    let t = code_tokens.trim_start();
    if !t.starts_with("pub") {
        return None;
    }
    let rest = if t.starts_with("pub(") {
        let mut depth = 0;
        let mut close_idx = None;
        for (i, c) in t.char_indices() {
            if c == '(' {
                depth += 1;
            } else if c == ')' {
                depth -= 1;
                if depth == 0 {
                    close_idx = Some(i);
                    break;
                }
            }
        }
        let close_pos = close_idx?;
        t[close_pos + 1..].trim_start()
    } else if t.starts_with("pub ") || t == "pub" {
        t[3..].trim_start()
    } else {
        return None;
    };
    Some((t, rest))
}

fn is_public_code(code_tokens: &str) -> bool {
    parse_public_item(code_tokens).is_some()
}

fn is_public_fn(code_tokens: &str) -> bool {
    if let Some((_, rest)) = parse_public_item(code_tokens) {
        rest.starts_with("fn ")
            || rest.starts_with("const fn ")
            || rest.starts_with("async fn ")
            || rest.starts_with("unsafe fn ")
            || rest.starts_with("extern ")
            || rest.starts_with("unsafe extern ")
    } else {
        false
    }
}

fn is_public_enum(code_tokens: &str) -> bool {
    if let Some((_, rest)) = parse_public_item(code_tokens) {
        rest.starts_with("enum ")
    } else {
        false
    }
}

fn is_public_use(code_tokens: &str) -> bool {
    if let Some((_, rest)) = parse_public_item(code_tokens) {
        rest.starts_with("use ")
    } else {
        false
    }
}

fn is_public_const_or_static(code_tokens: &str) -> bool {
    if let Some((_, rest)) = parse_public_item(code_tokens) {
        (rest.starts_with("const ") || rest.starts_with("static ")) && !is_public_fn(code_tokens)
    } else {
        false
    }
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
    let mut file_lexer = CodeLexer::new();

    while idx < src_lines.len() {
        let raw_line = if file_lexer.state.is_in_string_literal() {
            src_lines[idx]
        } else {
            src_lines[idx].trim()
        };
        if raw_line.is_empty() && !file_lexer.state.is_in_string_literal() {
            idx += 1;
            continue;
        }

        let mut item_lexer = file_lexer.clone();
        let scanned = item_lexer.scan_line(raw_line);

        if raw_line.starts_with("#[") && file_lexer.state == LexState::Normal {
            pending_attrs.push(normalize_ws(raw_line));
            file_lexer = item_lexer;
            idx += 1;
            continue;
        }

        if is_public_code(&scanned.code_tokens) {
            lines.append(&mut pending_attrs);

            if is_public_fn(&scanned.code_tokens) {
                let mut current_scanned = scanned;
                let mut signature = current_scanned.text_up_to_function_body_open_brace();
                while !current_scanned.has_function_body_open_brace
                    && !current_scanned.has_structural_semicolon
                    && idx + 1 < src_lines.len()
                {
                    idx += 1;
                    let continuation = if item_lexer.state.is_in_string_literal() {
                        src_lines[idx]
                    } else {
                        src_lines[idx].trim()
                    };
                    current_scanned = item_lexer.scan_line(continuation);
                    let rendered = current_scanned.text_up_to_function_body_open_brace();
                    if continuation.is_empty() || rendered.trim().is_empty() {
                        continue;
                    }
                    signature.push(' ');
                    signature.push_str(&rendered);
                }
                lines.push(normalize_ws(&signature));
                if item_lexer.state == LexState::Normal {
                    item_lexer.reset_top_level_depths();
                }
                file_lexer = item_lexer;
                idx += 1;
                continue;
            }

            if is_public_enum(&scanned.code_tokens) {
                let mut enum_decl = scanned.render_visible();
                let mut enum_brace_depth =
                    scanned.open_brace_count as i32 - scanned.close_brace_count as i32;
                let mut current_scanned = scanned;

                while enum_brace_depth == 0
                    && !current_scanned.has_structural_open_brace
                    && idx + 1 < src_lines.len()
                {
                    idx += 1;
                    let continuation = if item_lexer.state.is_in_string_literal() {
                        src_lines[idx]
                    } else {
                        src_lines[idx].trim()
                    };
                    current_scanned = item_lexer.scan_line(continuation);
                    let rendered = current_scanned.render_visible();
                    if continuation.is_empty() || rendered.trim().is_empty() {
                        continue;
                    }
                    enum_decl.push(' ');
                    enum_decl.push_str(&rendered);
                    enum_brace_depth += current_scanned.open_brace_count as i32
                        - current_scanned.close_brace_count as i32;
                }

                if enum_brace_depth <= 0
                    && current_scanned.has_structural_open_brace
                    && current_scanned.has_structural_close_brace
                {
                    // Single-line enum: pub enum State { N, F, T, S }
                    lines.push(normalize_ws(&enum_decl));
                    if item_lexer.state == LexState::Normal {
                        item_lexer.reset_top_level_depths();
                    }
                    file_lexer = item_lexer;
                    idx += 1;
                    continue;
                }

                lines.push(normalize_ws(&enum_decl));

                while enum_brace_depth > 0 && idx + 1 < src_lines.len() {
                    idx += 1;
                    let item_line = if item_lexer.state.is_in_string_literal() {
                        src_lines[idx]
                    } else {
                        src_lines[idx].trim()
                    };
                    if item_line.is_empty() && !item_lexer.state.is_in_string_literal() {
                        continue;
                    }
                    let sc = item_lexer.scan_line(item_line);
                    enum_brace_depth += sc.open_brace_count as i32 - sc.close_brace_count as i32;

                    let rendered = sc.render_visible();
                    if rendered.trim().is_empty() && !sc.ends_in_literal_or_comment {
                        continue;
                    }
                    if item_line.starts_with("#[") && !sc.ends_in_literal_or_comment {
                        lines.push(normalize_ws(item_line));
                        continue;
                    }

                    if enum_brace_depth == 0 {
                        let clean_trimmed = rendered.trim_end_matches('}').trim();
                        if !clean_trimmed.is_empty() {
                            lines.push(normalize_ws(clean_trimmed));
                        }
                        break;
                    } else {
                        lines.push(normalize_ws(&rendered));
                    }
                }

                if item_lexer.state == LexState::Normal {
                    item_lexer.reset_top_level_depths();
                }
                file_lexer = item_lexer;
                idx += 1;
                continue;
            }

            if is_public_const_or_static(&scanned.code_tokens) {
                let mut item = scanned.render_visible();
                let mut depth = scanned.depth_delta;
                let mut prev_ended_in_string = scanned.ends_in_string_literal;
                let mut has_semi_at_zero = depth <= 0 && scanned.has_structural_semicolon;

                while !has_semi_at_zero && idx + 1 < src_lines.len() {
                    idx += 1;
                    let next_line = if item_lexer.state.is_in_string_literal() {
                        src_lines[idx]
                    } else {
                        src_lines[idx].trim()
                    };
                    if next_line.is_empty() && !item_lexer.state.is_in_string_literal() {
                        continue;
                    }
                    let sc = item_lexer.scan_line(next_line);
                    depth += sc.depth_delta;
                    if depth <= 0 && sc.has_structural_semicolon {
                        has_semi_at_zero = true;
                    }
                    let rendered = sc.render_visible();
                    if rendered.trim().is_empty() && !sc.ends_in_string_literal {
                        prev_ended_in_string = sc.ends_in_string_literal;
                        continue;
                    }
                    if prev_ended_in_string {
                        item.push('\n');
                    } else if !item.ends_with(' ') {
                        item.push(' ');
                    }
                    item.push_str(&rendered);
                    prev_ended_in_string = sc.ends_in_string_literal;
                }

                lines.push(item);
                if item_lexer.state == LexState::Normal {
                    item_lexer.reset_top_level_depths();
                }
                file_lexer = item_lexer;
                idx += 1;
                continue;
            }

            if is_public_use(&scanned.code_tokens) {
                let mut item = scanned.render_visible();
                let mut depth = scanned.depth_delta;
                let mut is_done = depth <= 0 && scanned.has_structural_semicolon;

                while !is_done && idx + 1 < src_lines.len() {
                    idx += 1;
                    let next_line = src_lines[idx].trim();
                    if next_line.is_empty() {
                        continue;
                    }
                    let sc = item_lexer.scan_line(next_line);
                    depth += sc.depth_delta;
                    if depth <= 0 && sc.has_structural_semicolon {
                        is_done = true;
                    }
                    let rendered = sc.render_visible();
                    if rendered.trim().is_empty() && !sc.ends_in_literal_or_comment {
                        continue;
                    }
                    item.push(' ');
                    item.push_str(&rendered);
                }

                lines.push(normalize_ws(&item));
                if item_lexer.state == LexState::Normal {
                    item_lexer.reset_top_level_depths();
                }
                file_lexer = item_lexer;
                idx += 1;
                continue;
            }

            // Other public items (struct, type alias, trait, mod)
            let mut item = scanned.render_visible();
            let mut depth = scanned.depth_delta;
            let is_item_done = |sc: &ScannedLine, d: i32| {
                sc.has_structural_open_brace
                    || (d <= 0
                        && (sc.has_structural_semicolon
                            || sc.code_tokens.trim().ends_with(',')
                            || sc.code_tokens.trim().ends_with(';')))
            };
            let mut is_complete = is_item_done(&scanned, depth);

            while !is_complete && idx + 1 < src_lines.len() {
                idx += 1;
                let continuation = src_lines[idx].trim();
                if continuation.is_empty() {
                    continue;
                }
                let sc = item_lexer.scan_line(continuation);
                depth += sc.depth_delta;
                if is_item_done(&sc, depth) {
                    is_complete = true;
                }
                let rendered = sc.render_visible();
                if rendered.trim().is_empty() && !sc.ends_in_literal_or_comment {
                    continue;
                }
                item.push(' ');
                item.push_str(&rendered);
            }

            lines.push(normalize_ws(&item));
            if item_lexer.state == LexState::Normal {
                item_lexer.reset_top_level_depths();
            }
            file_lexer = item_lexer;
            idx += 1;
            continue;
        }

        // Advance file_lexer for non-public lines
        if item_lexer.state == LexState::Normal {
            item_lexer.reset_top_level_depths();
        }
        file_lexer = item_lexer;
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

#[test]
fn public_api_guard_captures_multiline_string_with_semicolons_and_raw_strings() {
    let old_src = r#"
pub const TEXT: &str = "first;
SECOND LINE";
pub const NEXT: u32 = 1;
"#;
    let new_src = r#"
pub const TEXT: &str = "first;
CHANGED PUBLIC VALUE";
pub const NEXT: u32 = 1;
"#;
    let old_surface = normalized_public_surface_str("test.rs", old_src);
    let new_surface = normalized_public_surface_str("test.rs", new_src);
    assert_ne!(
        old_surface, new_surface,
        "changing multiline string continuation after literal semicolon must change surface"
    );
    assert!(
        old_surface.contains("pub const TEXT: &str = \"first;\nSECOND LINE\";"),
        "full multiline string const must be captured: {old_surface}"
    );
    assert!(
        old_surface.contains("pub const NEXT: u32 = 1;"),
        "NEXT must remain separate public declaration: {old_surface}"
    );

    let old_raw = r##"
pub const RAW: &str = r#"first;
{ fake structural delimiter }
SECOND LINE"#;

pub const AFTER_RAW: u32 = 2;
"##;
    let new_raw = r##"
pub const RAW: &str = r#"first;
{ fake structural delimiter }
CHANGED PUBLIC VALUE"#;

pub const AFTER_RAW: u32 = 2;
"##;
    let old_raw_surface = normalized_public_surface_str("test.rs", old_raw);
    let new_raw_surface = normalized_public_surface_str("test.rs", new_raw);
    assert_ne!(
        old_raw_surface, new_raw_surface,
        "changing multiline raw string continuation after literal semicolon and fake braces must change surface"
    );
    assert!(
        old_raw_surface.contains("pub const AFTER_RAW: u32 = 2;"),
        "AFTER_RAW must remain separate public declaration: {old_raw_surface}"
    );
}

#[test]
fn public_api_guard_handles_char_and_byte_char_delimiters_without_depth_leak() {
    let src = r#"
pub const OPEN: char = '{';
pub const CLOSE: char = '}';
pub const BYTE_OPEN: u8 = b'{';
pub const BYTE_CLOSE: u8 = b'}';
pub const ESC_QUOTE: char = '\'';
pub const ESC_SLASH: char = '\\';
pub const ESC_NEWLINE: char = '\n';
pub const BYTE_ESC_QUOTE: u8 = b'\'';
pub const NEXT: u32 = 3;
"#;
    let surface = normalized_public_surface_str("test.rs", src);
    assert!(
        surface.contains("pub const OPEN: char = '{';"),
        "OPEN captured: {surface}"
    );
    assert!(
        surface.contains("pub const CLOSE: char = '}';"),
        "CLOSE captured: {surface}"
    );
    assert!(
        surface.contains("pub const BYTE_OPEN: u8 = b'{';"),
        "BYTE_OPEN captured: {surface}"
    );
    assert!(
        surface.contains("pub const BYTE_CLOSE: u8 = b'}';"),
        "BYTE_CLOSE captured: {surface}"
    );
    assert!(
        surface.contains(r"pub const ESC_QUOTE: char = '\'';"),
        "ESC_QUOTE captured: {surface}"
    );
    assert!(
        surface.contains(r"pub const ESC_SLASH: char = '\\';"),
        "ESC_SLASH captured: {surface}"
    );
    assert!(
        surface.contains(r"pub const ESC_NEWLINE: char = '\n';"),
        "ESC_NEWLINE captured: {surface}"
    );
    assert!(
        surface.contains(r"pub const BYTE_ESC_QUOTE: u8 = b'\'';"),
        "BYTE_ESC_QUOTE captured: {surface}"
    );
    assert!(
        surface.contains("pub const NEXT: u32 = 3;"),
        "NEXT captured: {surface}"
    );

    let changed_src = r#"
pub const OPEN: char = 'x';
pub const CLOSE: char = '}';
pub const BYTE_OPEN: u8 = b'{';
pub const BYTE_CLOSE: u8 = b'}';
pub const ESC_QUOTE: char = '\'';
pub const ESC_SLASH: char = '\\';
pub const ESC_NEWLINE: char = '\n';
pub const BYTE_ESC_QUOTE: u8 = b'\'';
pub const NEXT: u32 = 3;
"#;
    let changed_surface = normalized_public_surface_str("test.rs", changed_src);
    assert_ne!(
        surface, changed_surface,
        "changing char value must change surface"
    );
}

#[test]
fn public_api_guard_handles_single_line_enum_without_swallowing_function_body() {
    let src1 = r#"
pub enum State { N, F, T, S }

pub fn value() -> u32 {
    private_a()
}
"#;
    let src2 = r#"
pub enum State { N, F, T, S }

pub fn value() -> u32 {
    completely_different_private_body()
}
"#;
    let surface1 = normalized_public_surface_str("test.rs", src1);
    let surface2 = normalized_public_surface_str("test.rs", src2);
    assert_eq!(
        surface1, surface2,
        "single-line enum followed by function must not capture private function body: {surface1} vs {surface2}"
    );
    assert!(
        surface1.contains("State")
            && surface1.contains("N")
            && surface1.contains("F")
            && surface1.contains("T")
            && surface1.contains("S"),
        "enum variants must be captured: {surface1}"
    );
    assert!(
        surface1.contains("pub fn value() -> u32"),
        "function signature must be captured: {surface1}"
    );
    assert!(
        !surface1.contains("private_a"),
        "private body must not be captured: {surface1}"
    );
    assert!(
        !surface1.contains("completely_different_private_body"),
        "private body must not be captured: {surface1}"
    );
}

#[test]
fn public_api_guard_does_not_misclassify_keywords_inside_string_literals() {
    let src = r#"
pub const ENUM_TEXT: &str = " enum ";
pub const FN_TEXT: &str = " fn ";
pub const STRUCT_TEXT: &str = " struct ";
pub const NEXT: u32 = 4;
"#;
    let surface = normalized_public_surface_str("test.rs", src);
    assert!(
        surface.contains("pub const ENUM_TEXT: &str = \" enum \";"),
        "ENUM_TEXT captured: {surface}"
    );
    assert!(
        surface.contains("pub const FN_TEXT: &str = \" fn \";"),
        "FN_TEXT captured: {surface}"
    );
    assert!(
        surface.contains("pub const STRUCT_TEXT: &str = \" struct \";"),
        "STRUCT_TEXT captured: {surface}"
    );
    assert!(
        surface.contains("pub const NEXT: u32 = 4;"),
        "NEXT captured: {surface}"
    );
}

#[test]
fn public_api_guard_mutation_and_false_pass_matrix() {
    // A. Enum variant change -> MUST change surface
    let old_a = "pub enum E { A, B }";
    let new_a = "pub enum E { A, C }";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_a),
        normalized_public_surface_str("t.rs", new_a),
        "A. enum variant change must change surface"
    );

    // B. Enum discriminant change -> MUST change surface
    let old_b = "pub enum E { A = 1, B = 2 }";
    let new_b = "pub enum E { A = 1, B = 3 }";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_b),
        normalized_public_surface_str("t.rs", new_b),
        "B. enum discriminant change must change surface"
    );

    // C. Tuple variant payload type change -> MUST change surface
    let old_c = "pub enum E { A(u32) }";
    let new_c = "pub enum E { A(u64) }";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_c),
        normalized_public_surface_str("t.rs", new_c),
        "C. tuple variant payload type change must change surface"
    );

    // D. Struct variant field type change -> MUST change surface
    let old_d = "pub enum E { A { x: u32 } }";
    let new_d = "pub enum E { A { x: u64 } }";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_d),
        normalized_public_surface_str("t.rs", new_d),
        "D. struct variant field type change must change surface"
    );

    // E. HEADER-style const field value change -> MUST change surface
    let old_e = "pub const HEADER: Header = Header { magic: [1, 2, 3], version: 1 };";
    let new_e = "pub const HEADER: Header = Header { magic: [1, 2, 4], version: 1 };";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_e),
        normalized_public_surface_str("t.rs", new_e),
        "E. HEADER-style const field value change must change surface"
    );

    // F. Multiline string continuation change -> MUST change surface
    let old_f = "pub const S: &str = \"hello\nworld\";";
    let new_f = "pub const S: &str = \"hello\nmodified\";";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_f),
        normalized_public_surface_str("t.rs", new_f),
        "F. multiline string continuation change must change surface"
    );

    // G. Multiline raw-string continuation change -> MUST change surface
    let old_g = "pub const R: &str = r#\"hello\nworld\"#;";
    let new_g = "pub const R: &str = r#\"hello\nmodified\"#;";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_g),
        normalized_public_surface_str("t.rs", new_g),
        "G. multiline raw-string continuation change must change surface"
    );

    // H. Char literal value change -> MUST change surface
    let old_h = "pub const C: char = 'a';";
    let new_h = "pub const C: char = 'b';";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_h),
        normalized_public_surface_str("t.rs", new_h),
        "H. char literal value change must change surface"
    );

    // I. Private function body only change -> MUST NOT change surface
    let old_i = "pub fn f() -> u32 { private_impl_1(); 42 }";
    let new_i = "pub fn f() -> u32 { private_impl_2(); 99 }";
    assert_eq!(
        normalized_public_surface_str("t.rs", old_i),
        normalized_public_surface_str("t.rs", new_i),
        "I. private function body change must not change surface"
    );

    // J. Comments only change -> MUST NOT change surface
    let old_j = "// comment 1\npub const X: u32 = 1; /* inline comment */";
    let new_j = "// different comment\npub const X: u32 = 1; /* another comment */";
    assert_eq!(
        normalized_public_surface_str("t.rs", old_j),
        normalized_public_surface_str("t.rs", new_j),
        "J. comments change must not change surface"
    );

    // K. Whitespace-only formatting change -> MUST NOT change surface
    let old_k = "    pub   const   X:   u32   =   1;\n";
    let new_k = "pub const X: u32 = 1;\n";
    assert_eq!(
        normalized_public_surface_str("t.rs", old_k),
        normalized_public_surface_str("t.rs", new_k),
        "K. whitespace formatting change must not change surface"
    );

    // L. String literal double-space -> single-space: MUST DETECT
    let old_l = "pub const S: &str = \"a  b\";";
    let new_l = "pub const S: &str = \"a b\";";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_l),
        normalized_public_surface_str("t.rs", new_l),
        "L. string literal double-space vs single-space change must be detected"
    );

    // M. Multiline newline -> space: MUST DETECT
    let old_m = "pub const S: &str = \"a\nb\";";
    let new_m = "pub const S: &str = \"a b\";";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_m),
        normalized_public_surface_str("t.rs", new_m),
        "M. multiline literal newline vs space change must be detected"
    );

    // N. Raw-string indentation: MUST DETECT
    let old_n = "pub const R: &str = r#\"a\n  b\"#;";
    let new_n = "pub const R: &str = r#\"a\n b\"#;";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_n),
        normalized_public_surface_str("t.rs", new_n),
        "N. raw-string indentation change must be detected"
    );

    // O. Multiline pub-use member: MUST DETECT
    let old_o = "pub use demo::{\n    Alpha,\n    Beta,\n};";
    let new_o = "pub use demo::{\n    Alpha,\n    Gamma,\n};";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_o),
        normalized_public_surface_str("t.rs", new_o),
        "O. multiline pub-use member change must be detected"
    );

    // P. Const-generic signature value: MUST DETECT
    let old_p = "pub fn build() -> Foo<{ 32 }> {\n    private_a()\n}";
    let new_p = "pub fn build() -> Foo<{ 64 }> {\n    private_a()\n}";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_p),
        normalized_public_surface_str("t.rs", new_p),
        "P. const-generic signature value change must be detected"
    );

    // Q. Block comment token separator: declaration MUST remain inventoried
    let old_q = "pub/* comment */const X: u32 = 1;";
    let new_q = "pub/* comment */const X: u32 = 2;";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_q),
        normalized_public_surface_str("t.rs", new_q),
        "Q. block comment token separator declaration must detect value changes"
    );

    // R. pub(super) multiline const field: MUST DETECT
    let old_r = "pub(super) const HEADER: Spec = Spec {\n    rev: 1,\n};";
    let new_r = "pub(super) const HEADER: Spec = Spec {\n    rev: 2,\n};";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_r),
        normalized_public_surface_str("t.rs", new_r),
        "R. pub(super) multiline const field change must be detected"
    );
}

#[test]
fn public_api_guard_preserves_literal_whitespace_and_raw_string_indentation() {
    let s_double_space = "pub const S: &str = \"a  b\";";
    let s_single_space = "pub const S: &str = \"a b\";";
    assert_ne!(
        normalized_public_surface_str("test.rs", s_double_space),
        normalized_public_surface_str("test.rs", s_single_space),
        "surfaces must differ for double space vs single space inside literal"
    );

    let s_multiline = "pub const S: &str = \"a\nb\";";
    assert_ne!(
        normalized_public_surface_str("test.rs", s_multiline),
        normalized_public_surface_str("test.rs", s_single_space),
        "surfaces must differ for multiline string vs single line space"
    );

    let r_indent_2 = "pub const R: &str = r#\"a\n  b\"#;";
    let r_indent_1 = "pub const R: &str = r#\"a\n b\"#;";
    assert_ne!(
        normalized_public_surface_str("test.rs", r_indent_2),
        normalized_public_surface_str("test.rs", r_indent_1),
        "surfaces must differ for raw string indentation change"
    );
}

#[test]
fn public_api_guard_captures_multiline_pub_use_reexports() {
    let old_use = r#"
pub use demo::{
    Alpha,
    Beta,
};
pub const NEXT: u32 = 10;
"#;
    let new_use = r#"
pub use demo::{
    Alpha,
    Gamma,
};
pub const NEXT: u32 = 10;
"#;
    let old_surf = normalized_public_surface_str("test.rs", old_use);
    let new_surf = normalized_public_surface_str("test.rs", new_use);
    assert_ne!(
        old_surf, new_surf,
        "multiline pub use re-export member change must alter surface"
    );
    assert!(
        old_surf.contains("Alpha") && old_surf.contains("Beta"),
        "re-exported members Alpha and Beta must be captured: {old_surf}"
    );
    assert!(
        old_surf.contains("pub const NEXT: u32 = 10;"),
        "following declaration must be inventoried: {old_surf}"
    );
}

#[test]
fn public_api_guard_handles_function_signature_const_generic_braces() {
    let fn_32 = r#"
pub fn build() -> Foo<{ 32 }> {
    private_a()
}
"#;
    let fn_64 = r#"
pub fn build() -> Foo<{ 64 }> {
    private_a()
}
"#;
    let fn_32_diff_body = r#"
pub fn build() -> Foo<{ 32 }> {
    private_b()
}
"#;
    let surf_32 = normalized_public_surface_str("test.rs", fn_32);
    let surf_64 = normalized_public_surface_str("test.rs", fn_64);
    let surf_32_diff_body = normalized_public_surface_str("test.rs", fn_32_diff_body);

    assert_ne!(
        surf_32, surf_64,
        "const-generic value change in return type must change surface"
    );
    assert_eq!(
        surf_32, surf_32_diff_body,
        "changing private body in function with const-generic return type must not change surface"
    );
    assert!(
        !surf_32.contains("private_a"),
        "private body must not be captured: {surf_32}"
    );
}

#[test]
fn public_api_guard_treats_comments_as_token_separators() {
    let src1 = "pub/* comment */const X: u32 = 1;";
    let src2 = "pub const X: u32 = 1;";
    let surf1 = normalized_public_surface_str("test.rs", src1);
    let surf2 = normalized_public_surface_str("test.rs", src2);
    assert_eq!(
        surf1, surf2,
        "comment separating pub and const must yield normalized declaration"
    );

    let src3 = "pub/* comment */const X: u32 = 2;";
    let surf3 = normalized_public_surface_str("test.rs", src3);
    assert_ne!(
        surf1, surf3,
        "value change with comment token separator must change surface"
    );

    let src_enum = "pub/*x*/ enum E { A, B }";
    let surf_enum = normalized_public_surface_str("test.rs", src_enum);
    assert!(
        surf_enum.contains("A") && surf_enum.contains("B"),
        "enum with comment separator must capture variants: {surf_enum}"
    );

    let src_crate = "pub(crate)/*x*/ const X: u32 = 1;";
    let surf_crate = normalized_public_surface_str("test.rs", src_crate);
    assert!(
        surf_crate.contains("pub(crate) const X: u32 = 1;"),
        "pub(crate) with comment separator must be captured: {surf_crate}"
    );
}

#[test]
fn public_api_guard_handles_all_pub_restricted_visibilities() {
    let src_super_1 = r#"
pub(super) const HEADER: Spec = Spec {
    rev: 1,
};
"#;
    let src_super_2 = r#"
pub(super) const HEADER: Spec = Spec {
    rev: 2,
};
"#;
    let surf_super_1 = normalized_public_surface_str("test.rs", src_super_1);
    let surf_super_2 = normalized_public_surface_str("test.rs", src_super_2);
    assert_ne!(
        surf_super_1, surf_super_2,
        "pub(super) multiline const change must be captured"
    );

    let src_in_path = "pub(in crate::module) static GLOBAL_DATA: u32 = 100;";
    let surf_in_path = normalized_public_surface_str("test.rs", src_in_path);
    assert!(
        surf_in_path.contains("pub(in crate::module) static GLOBAL_DATA: u32 = 100;"),
        "pub(in ...) restricted visibility static must be captured: {surf_in_path}"
    );
}
