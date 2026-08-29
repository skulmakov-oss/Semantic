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
    NormalStringContinuation,
    RawString(usize),
    ByteNormalString,
    ByteNormalStringContinuation,
    RawByteString(usize),
}

impl LexState {
    fn is_in_string_literal(self) -> bool {
        matches!(
            self,
            LexState::NormalString
                | LexState::NormalStringContinuation
                | LexState::RawString(_)
                | LexState::ByteNormalString
                | LexState::ByteNormalStringContinuation
                | LexState::RawByteString(_)
        )
    }

    fn is_escaped_continuation(self) -> bool {
        matches!(
            self,
            LexState::NormalStringContinuation | LexState::ByteNormalStringContinuation
        )
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum VisibleSegment {
    Code(String),
    Literal(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StructuralEventKind {
    TopLevelOpenBrace,
    TopLevelCloseBrace,
    TopLevelSemicolon,
    TopLevelComma,
    BaselineOpenBrace,
    MethodBodyClose,
    BaselineSemicolon,
    BaselineComma,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct StructuralEvent {
    seg_idx: usize,
    char_idx: usize,
    kind: StructuralEventKind,
}

#[derive(Debug, Clone)]
struct ScannedLine {
    segments: Vec<VisibleSegment>,
    code_tokens: String,
    events: Vec<StructuralEvent>,
    has_top_level_semicolon: bool,
    has_top_level_comma: bool,
    has_top_level_open_paren: bool,
    has_top_level_open_brace: bool,
    has_top_level_close_brace: bool,
    first_top_level_open_brace_seg: Option<(usize, usize)>,
    ends_in_string_literal: bool,
}

impl ScannedLine {
    fn render_visible(&self) -> String {
        self.render_visible_range(None, None)
    }

    fn text_up_to_function_body_open_brace(&self) -> String {
        self.render_visible_range(None, self.first_top_level_open_brace_seg)
    }

    fn render_visible_without_enum_close_brace(&self) -> String {
        let mut cloned = self.clone();
        if let Some(VisibleSegment::Code(last_code)) = cloned.segments.last_mut() {
            if let Some(pos) = last_code.rfind('}') {
                last_code.truncate(pos);
            }
        }
        cloned.render_visible()
    }

    fn render_visible_range(
        &self,
        start: Option<(usize, usize)>,
        end: Option<(usize, usize)>,
    ) -> String {
        if let (Some(s), Some(e)) = (start, end) {
            if s > e {
                return String::new();
            }
        }
        let mut out = String::new();
        for (seg_idx, seg) in self.segments.iter().enumerate() {
            if let Some((s_seg, _)) = start {
                if seg_idx < s_seg {
                    continue;
                }
            }
            if let Some((e_seg, _)) = end {
                if seg_idx > e_seg {
                    break;
                }
            }

            match seg {
                VisibleSegment::Code(s) => {
                    let s_char = if start.is_some_and(|(s_seg, _)| s_seg == seg_idx) {
                        start.unwrap().1
                    } else {
                        0
                    };
                    let e_char = if end.is_some_and(|(e_seg, _)| e_seg == seg_idx) {
                        end.unwrap().1
                    } else {
                        s.len().saturating_sub(1)
                    };
                    if s_char <= e_char && s_char < s.len() {
                        let chunk = &s[s_char..=e_char.min(s.len() - 1)];
                        let norm = normalize_ws(chunk);
                        if !norm.is_empty() {
                            if !out.is_empty() && !out.ends_with(' ') {
                                let first = norm.chars().next().unwrap();
                                let last = out.chars().last().unwrap();
                                let skip_space = is_attached_punctuation_prefix(first)
                                    || is_attached_opening_delimiter(last)
                                    || (last == '#' && first == '[')
                                    || (out.ends_with("pub") && first == '(');
                                if !skip_space {
                                    out.push(' ');
                                }
                            }
                            out.push_str(&norm);
                        }
                    }
                }
                VisibleSegment::Literal(s) => {
                    let s_char = if start.is_some_and(|(s_seg, _)| s_seg == seg_idx) {
                        start.unwrap().1
                    } else {
                        0
                    };
                    let e_char = if end.is_some_and(|(e_seg, _)| e_seg == seg_idx) {
                        end.unwrap().1
                    } else {
                        s.len().saturating_sub(1)
                    };
                    if s_char <= e_char && s_char < s.len() {
                        let chunk = &s[s_char..=e_char.min(s.len() - 1)];
                        if !out.is_empty() && !out.ends_with(' ') {
                            let last = out.chars().last().unwrap();
                            if !is_attached_opening_delimiter(last) {
                                out.push(' ');
                            }
                        }
                        out.push_str(chunk);
                    }
                }
            }
        }
        out
    }

    fn code_tokens_range(
        &self,
        start: Option<(usize, usize)>,
        end: Option<(usize, usize)>,
    ) -> String {
        if let (Some(s), Some(e)) = (start, end) {
            if s > e {
                return String::new();
            }
        }
        let mut out = String::new();
        for (seg_idx, seg) in self.segments.iter().enumerate() {
            if let Some((s_seg, _)) = start {
                if seg_idx < s_seg {
                    continue;
                }
            }
            if let Some((e_seg, _)) = end {
                if seg_idx > e_seg {
                    break;
                }
            }

            match seg {
                VisibleSegment::Code(s) => {
                    let s_char = if start.is_some_and(|(s_seg, _)| s_seg == seg_idx) {
                        start.unwrap().1
                    } else {
                        0
                    };
                    let e_char = if end.is_some_and(|(e_seg, _)| e_seg == seg_idx) {
                        end.unwrap().1
                    } else {
                        s.len().saturating_sub(1)
                    };
                    if s_char <= e_char && s_char < s.len() {
                        let chunk = &s[s_char..=e_char.min(s.len() - 1)];
                        if !out.is_empty() && !out.ends_with(' ') {
                            out.push(' ');
                        }
                        out.push_str(chunk);
                    }
                }
                VisibleSegment::Literal(_) => {
                    if !out.is_empty() && !out.ends_with(' ') {
                        out.push(' ');
                    }
                    out.push_str("\"\"");
                }
            }
        }
        out
    }
}

fn next_pos(segments: &[VisibleSegment], pos: (usize, usize)) -> Option<(usize, usize)> {
    let (seg_idx, char_idx) = pos;
    if seg_idx >= segments.len() {
        return None;
    }
    let seg_len = match &segments[seg_idx] {
        VisibleSegment::Code(s) => s.len(),
        VisibleSegment::Literal(s) => s.len(),
    };
    if char_idx + 1 < seg_len {
        Some((seg_idx, char_idx + 1))
    } else if seg_idx + 1 < segments.len() {
        Some((seg_idx + 1, 0))
    } else {
        None
    }
}

fn prev_pos(segments: &[VisibleSegment], pos: (usize, usize)) -> Option<(usize, usize)> {
    let (seg_idx, char_idx) = pos;
    if char_idx > 0 {
        Some((seg_idx, char_idx - 1))
    } else if seg_idx > 0 {
        let prev_seg = seg_idx - 1;
        let prev_len = match &segments[prev_seg] {
            VisibleSegment::Code(s) => s.len(),
            VisibleSegment::Literal(s) => s.len(),
        };
        if prev_len > 0 {
            Some((prev_seg, prev_len - 1))
        } else {
            None
        }
    } else {
        None
    }
}

fn is_attached_punctuation_prefix(c: char) -> bool {
    matches!(c, ';' | ',' | ':' | ')' | ']' | '}' | '>')
}

fn is_attached_opening_delimiter(c: char) -> bool {
    matches!(c, '(' | '[' | '{' | '<' | '*' | '&' | '!')
}

fn is_likely_comparison_less_than(code_tokens: &str) -> bool {
    let t = code_tokens.trim_end();
    if t.is_empty() {
        return false;
    }
    if t.ends_with(|c: char| c == '\'' || c == '"' || c == ')' || c == ']' || c == '}') {
        return true;
    }
    if t.ends_with("true") || t.ends_with("false") {
        return true;
    }
    if is_public_const_or_static(code_tokens) {
        if let Some((_, after_eq)) = t.rsplit_once('=') {
            let trimmed_after = after_eq.trim();
            if !trimmed_after.ends_with("::") && !trimmed_after.contains('<') {
                return true;
            }
        }
    }
    let mut last_token_start = None;
    for (idx, c) in t.char_indices().rev() {
        if c.is_alphanumeric() || c == '_' {
            last_token_start = Some(idx);
        } else {
            break;
        }
    }
    if let Some(start) = last_token_start {
        let token = &t[start..];
        if token.starts_with(|c: char| c.is_ascii_digit()) {
            return true;
        }
    }
    false
}

fn is_macro_bang(code_tokens: &str) -> bool {
    let s = code_tokens.trim_end();
    if s.is_empty() {
        return false;
    }
    let mut last_valid_byte = None;
    for (byte_idx, c) in s.char_indices().rev() {
        if c.is_alphanumeric() || c == '_' {
            last_valid_byte = Some(byte_idx);
        } else {
            break;
        }
    }
    let Some(start_byte) = last_valid_byte else {
        return false;
    };
    let ident = &s[start_byte..];
    if ident.starts_with(|c: char| c.is_numeric()) {
        return false;
    }
    true
}

#[derive(Debug, Clone)]
struct CodeLexer {
    state: LexState,
    angle_depth: usize,
    paren_depth: usize,
    bracket_depth: usize,
    brace_depth: usize,
    pending_macro_bang: bool,
    macro_brace_stack: Vec<usize>,
}

impl CodeLexer {
    fn new() -> Self {
        Self {
            state: LexState::Normal,
            angle_depth: 0,
            paren_depth: 0,
            bracket_depth: 0,
            brace_depth: 0,
            pending_macro_bang: false,
            macro_brace_stack: Vec::new(),
        }
    }

    fn reset_top_level_depths(&mut self) {
        self.angle_depth = 0;
        self.paren_depth = 0;
        self.bracket_depth = 0;
        self.brace_depth = 0;
        self.pending_macro_bang = false;
        self.macro_brace_stack.clear();
    }

    fn scan_line(&mut self, line: &str) -> ScannedLine {
        let mut segments = Vec::new();
        let mut cur_code = String::new();
        let mut code_tokens = String::with_capacity(line.len());
        let mut events = Vec::new();
        let mut has_top_level_semicolon = false;
        let mut has_top_level_comma = false;
        let mut has_top_level_open_paren = false;
        let mut has_top_level_open_brace = false;
        let mut has_top_level_close_brace = false;
        let mut first_top_level_open_brace_seg = None;

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

                    // 9. Composite operators: <<, <=, >=, ->, =>
                    if chars[i] == '!' && i + 1 < chars.len() && chars[i + 1] == '=' {
                        cur_code.push_str("!=");
                        code_tokens.push_str("!=");
                        self.pending_macro_bang = false;
                        i += 2;
                        continue;
                    }
                    if chars[i] == '<' && i + 1 < chars.len() && chars[i + 1] == '<' {
                        cur_code.push_str("<<");
                        code_tokens.push_str("<<");
                        self.pending_macro_bang = false;
                        i += 2;
                        continue;
                    }
                    if chars[i] == '<' && i + 1 < chars.len() && chars[i + 1] == '=' {
                        cur_code.push_str("<=");
                        code_tokens.push_str("<=");
                        self.pending_macro_bang = false;
                        i += 2;
                        continue;
                    }
                    if chars[i] == '>' && i + 1 < chars.len() && chars[i + 1] == '=' {
                        cur_code.push_str(">=");
                        code_tokens.push_str(">=");
                        self.pending_macro_bang = false;
                        i += 2;
                        continue;
                    }
                    if chars[i] == '-' && i + 1 < chars.len() && chars[i + 1] == '>' {
                        cur_code.push_str("->");
                        code_tokens.push_str("->");
                        self.pending_macro_bang = false;
                        i += 2;
                        continue;
                    }
                    if chars[i] == '=' && i + 1 < chars.len() && chars[i + 1] == '>' {
                        cur_code.push_str("=>");
                        code_tokens.push_str("=>");
                        self.pending_macro_bang = false;
                        i += 2;
                        continue;
                    }

                    // 10. Structural tokens & depth tracking
                    match chars[i] {
                        '!' => {
                            self.pending_macro_bang = is_macro_bang(&code_tokens);
                            cur_code.push('!');
                            code_tokens.push('!');
                        }
                        '<' => {
                            self.pending_macro_bang = false;
                            if self.paren_depth == 0
                                && self.bracket_depth == 0
                                && !is_likely_comparison_less_than(&code_tokens)
                            {
                                self.angle_depth += 1;
                            }
                            cur_code.push('<');
                            code_tokens.push('<');
                        }
                        '>' => {
                            self.pending_macro_bang = false;
                            if self.paren_depth == 0
                                && self.bracket_depth == 0
                                && self.angle_depth > 0
                            {
                                self.angle_depth -= 1;
                            }
                            cur_code.push('>');
                            code_tokens.push('>');
                        }
                        '(' => {
                            self.pending_macro_bang = false;
                            if self.paren_depth == 0
                                && self.angle_depth == 0
                                && self.bracket_depth == 0
                                && self.brace_depth == 0
                            {
                                has_top_level_open_paren = true;
                            }
                            self.paren_depth += 1;
                            cur_code.push('(');
                            code_tokens.push('(');
                        }
                        ')' => {
                            self.pending_macro_bang = false;
                            if self.paren_depth > 0 {
                                self.paren_depth -= 1;
                            }
                            cur_code.push(')');
                            code_tokens.push(')');
                        }
                        '[' => {
                            self.pending_macro_bang = false;
                            self.bracket_depth += 1;
                            cur_code.push('[');
                            code_tokens.push('[');
                        }
                        ']' => {
                            self.pending_macro_bang = false;
                            if self.bracket_depth > 0 {
                                self.bracket_depth -= 1;
                            }
                            cur_code.push(']');
                            code_tokens.push(']');
                        }
                        '{' => {
                            let is_macro_brace = self.pending_macro_bang;
                            self.pending_macro_bang = false;
                            if is_macro_brace {
                                self.macro_brace_stack.push(self.brace_depth);
                            } else {
                                let is_top_level = self.paren_depth == 0
                                    && self.angle_depth == 0
                                    && self.bracket_depth == 0
                                    && self.brace_depth == 0;
                                if is_top_level {
                                    if !has_top_level_open_brace {
                                        has_top_level_open_brace = true;
                                        first_top_level_open_brace_seg =
                                            Some((segments.len(), cur_code.len()));
                                    }
                                    events.push(StructuralEvent {
                                        seg_idx: segments.len(),
                                        char_idx: cur_code.len(),
                                        kind: StructuralEventKind::TopLevelOpenBrace,
                                    });
                                } else if self.paren_depth == 0
                                    && self.angle_depth == 0
                                    && self.bracket_depth == 0
                                    && self.brace_depth == 1
                                {
                                    events.push(StructuralEvent {
                                        seg_idx: segments.len(),
                                        char_idx: cur_code.len(),
                                        kind: StructuralEventKind::BaselineOpenBrace,
                                    });
                                }
                            }
                            self.brace_depth += 1;
                            cur_code.push('{');
                            code_tokens.push('{');
                        }
                        '}' => {
                            self.pending_macro_bang = false;
                            let is_macro_close = self
                                .macro_brace_stack
                                .last()
                                .is_some_and(|&d| d == self.brace_depth.saturating_sub(1));
                            if is_macro_close {
                                self.macro_brace_stack.pop();
                            } else if self.paren_depth == 0
                                && self.angle_depth == 0
                                && self.bracket_depth == 0
                            {
                                if self.brace_depth == 1 {
                                    has_top_level_close_brace = true;
                                    events.push(StructuralEvent {
                                        seg_idx: segments.len(),
                                        char_idx: cur_code.len(),
                                        kind: StructuralEventKind::TopLevelCloseBrace,
                                    });
                                } else if self.brace_depth == 2 {
                                    events.push(StructuralEvent {
                                        seg_idx: segments.len(),
                                        char_idx: cur_code.len(),
                                        kind: StructuralEventKind::MethodBodyClose,
                                    });
                                }
                            }
                            if self.brace_depth > 0 {
                                self.brace_depth -= 1;
                            }
                            cur_code.push('}');
                            code_tokens.push('}');
                        }
                        ';' => {
                            self.pending_macro_bang = false;
                            if self.paren_depth == 0
                                && self.bracket_depth == 0
                                && self.brace_depth == 0
                            {
                                has_top_level_semicolon = true;
                                self.angle_depth = 0;
                                events.push(StructuralEvent {
                                    seg_idx: segments.len(),
                                    char_idx: cur_code.len(),
                                    kind: StructuralEventKind::TopLevelSemicolon,
                                });
                            } else if self.paren_depth == 0
                                && self.bracket_depth == 0
                                && self.brace_depth == 1
                                && self.macro_brace_stack.is_empty()
                            {
                                events.push(StructuralEvent {
                                    seg_idx: segments.len(),
                                    char_idx: cur_code.len(),
                                    kind: StructuralEventKind::BaselineSemicolon,
                                });
                            }
                            cur_code.push(';');
                            code_tokens.push(';');
                        }
                        ',' => {
                            self.pending_macro_bang = false;
                            if self.paren_depth == 0
                                && self.angle_depth == 0
                                && self.bracket_depth == 0
                                && self.brace_depth == 0
                            {
                                has_top_level_comma = true;
                                events.push(StructuralEvent {
                                    seg_idx: segments.len(),
                                    char_idx: cur_code.len(),
                                    kind: StructuralEventKind::TopLevelComma,
                                });
                            } else if self.paren_depth == 0
                                && self.angle_depth == 0
                                && self.bracket_depth == 0
                                && self.brace_depth == 1
                                && self.macro_brace_stack.is_empty()
                            {
                                events.push(StructuralEvent {
                                    seg_idx: segments.len(),
                                    char_idx: cur_code.len(),
                                    kind: StructuralEventKind::BaselineComma,
                                });
                            }
                            cur_code.push(',');
                            code_tokens.push(',');
                        }
                        ' ' | '\t' | '\r' | '\n' => {
                            cur_code.push(chars[i]);
                            code_tokens.push(chars[i]);
                        }
                        c => {
                            self.pending_macro_bang = false;
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
                LexState::NormalStringContinuation | LexState::ByteNormalStringContinuation => {
                    let is_byte = matches!(self.state, LexState::ByteNormalStringContinuation);
                    while i < chars.len() && (chars[i] == ' ' || chars[i] == '\t') {
                        i += 1;
                    }
                    self.state = if is_byte {
                        LexState::ByteNormalString
                    } else {
                        LexState::NormalString
                    };
                }
                LexState::NormalString | LexState::ByteNormalString => {
                    let is_byte = matches!(self.state, LexState::ByteNormalString);
                    let mut lit_str = String::new();
                    while i < chars.len() {
                        if chars[i] == '\\' {
                            if i + 1 < chars.len() {
                                lit_str.push(chars[i]);
                                lit_str.push(chars[i + 1]);
                                i += 2;
                            } else {
                                self.state = if is_byte {
                                    LexState::ByteNormalStringContinuation
                                } else {
                                    LexState::NormalStringContinuation
                                };
                                i += 1;
                                break;
                            }
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
                    if !lit_str.is_empty() {
                        if let Some(VisibleSegment::Literal(prev)) = segments.last_mut() {
                            prev.push_str(&lit_str);
                        } else {
                            segments.push(VisibleSegment::Literal(lit_str));
                        }
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
            events,
            has_top_level_semicolon,
            has_top_level_comma,
            has_top_level_open_paren,
            has_top_level_open_brace,
            has_top_level_close_brace,
            first_top_level_open_brace_seg,
            ends_in_string_literal: self.state.is_in_string_literal(),
        }
    }
}

fn parse_public_item(code_tokens: &str) -> Option<(&str, &str)> {
    let t = code_tokens.trim_start();
    if !t.starts_with("pub") {
        return None;
    }
    let after_pub = t[3..].trim_start();
    if after_pub.starts_with('(') {
        let mut depth = 0;
        let mut close_idx = None;
        for (i, c) in after_pub.char_indices() {
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
        let rest = after_pub[close_pos + 1..].trim_start();
        Some((t, rest))
    } else if t.starts_with("pub ") || t == "pub" {
        Some((t, after_pub))
    } else {
        None
    }
}

fn is_keyword_prefix<'a>(s: &'a str, kw: &str) -> Option<&'a str> {
    if let Some(rest) = s.strip_prefix(kw) {
        if rest.is_empty()
            || rest.starts_with(char::is_whitespace)
            || rest.starts_with('<')
            || rest.starts_with('(')
            || rest.starts_with('{')
            || rest.starts_with(':')
            || rest.starts_with(';')
        {
            return Some(rest.trim_start());
        }
    }
    None
}

fn is_outer_attribute_start(code_tokens: &str) -> bool {
    let t = code_tokens.trim_start();
    if let Some(rest) = t.strip_prefix('#') {
        let trimmed = rest.trim_start();
        trimmed.starts_with('[') || trimmed.is_empty()
    } else {
        false
    }
}

fn is_public_code(code_tokens: &str) -> bool {
    parse_public_item(code_tokens).is_some()
}

fn is_public_fn(code_tokens: &str) -> bool {
    if let Some((_, rest)) = parse_public_item(code_tokens) {
        let mut cur = rest.trim_start();
        loop {
            if let Some(after) = is_keyword_prefix(cur, "const") {
                cur = after;
                continue;
            }
            if let Some(after) = is_keyword_prefix(cur, "async") {
                cur = after;
                continue;
            }
            if let Some(after) = is_keyword_prefix(cur, "unsafe") {
                cur = after;
                continue;
            }
            if let Some(after) = is_keyword_prefix(cur, "extern") {
                let mut rem = after;
                if rem.starts_with("\"\"") {
                    rem = rem[2..].trim_start();
                } else if rem.starts_with('"') {
                    if let Some(close) = rem[1..].find('"') {
                        rem = rem[close + 2..].trim_start();
                    }
                }
                cur = rem;
                continue;
            }
            break;
        }
        is_keyword_prefix(cur, "fn").is_some()
    } else {
        false
    }
}

fn is_public_enum(code_tokens: &str) -> bool {
    if let Some((_, rest)) = parse_public_item(code_tokens) {
        is_keyword_prefix(rest, "enum").is_some()
    } else {
        false
    }
}

fn is_public_struct(code_tokens: &str) -> bool {
    parse_public_item(code_tokens)
        .is_some_and(|(_, rest)| is_keyword_prefix(rest, "struct").is_some())
}

fn has_public_tuple_struct_body_open(code_tokens: &str) -> bool {
    let Some((_, rest)) = parse_public_item(code_tokens) else {
        return false;
    };
    if is_keyword_prefix(rest, "struct").is_none() {
        return false;
    }
    let before_where = rest
        .split_once(" where ")
        .map_or(rest, |(before, _)| before);
    CodeLexer::new()
        .scan_line(before_where)
        .has_top_level_open_paren
}

fn is_public_union(code_tokens: &str) -> bool {
    parse_public_item(code_tokens)
        .is_some_and(|(_, rest)| is_keyword_prefix(rest, "union").is_some())
}

fn is_public_trait(code_tokens: &str) -> bool {
    parse_public_item(code_tokens).is_some_and(|(_, rest)| {
        let mut cur = rest.trim_start();
        if let Some(after) = is_keyword_prefix(cur, "unsafe") {
            cur = after;
        }
        is_keyword_prefix(cur, "trait").is_some()
    })
}

fn is_public_use(code_tokens: &str) -> bool {
    if let Some((_, rest)) = parse_public_item(code_tokens) {
        is_keyword_prefix(rest, "use").is_some()
    } else {
        false
    }
}

fn is_public_const_or_static(code_tokens: &str) -> bool {
    if let Some((_, rest)) = parse_public_item(code_tokens) {
        (is_keyword_prefix(rest, "const").is_some() || is_keyword_prefix(rest, "static").is_some())
            && !is_public_fn(code_tokens)
    } else {
        false
    }
}

fn is_public_qualifiers_only(code_tokens: &str) -> bool {
    let Some((_, rest)) = parse_public_item(code_tokens) else {
        return false;
    };
    let mut cur = rest.trim_start();
    loop {
        if let Some(after) = is_keyword_prefix(cur, "const") {
            cur = after;
            continue;
        }
        if let Some(after) = is_keyword_prefix(cur, "async") {
            cur = after;
            continue;
        }
        if let Some(after) = is_keyword_prefix(cur, "unsafe") {
            cur = after;
            continue;
        }
        if let Some(after) = is_keyword_prefix(cur, "extern") {
            let mut rem = after;
            if rem.starts_with("\"\"") {
                rem = rem[2..].trim_start();
            } else if rem.starts_with('"') {
                if let Some(close) = rem[1..].find('"') {
                    rem = rem[close + 2..].trim_start();
                } else {
                    rem = "";
                }
            }
            cur = rem;
            continue;
        }
        break;
    }
    cur.trim().is_empty()
}

fn normalized_public_surface(path: &str) -> String {
    let src = fs::read_to_string(path).unwrap_or_else(|err| panic!("read {path}: {err}"));
    normalized_public_surface_str(path, &src)
}

fn capture_attribute(
    src_lines: &[&str],
    idx: &mut usize,
    lexer: &mut CodeLexer,
    mut scanned: ScannedLine,
) -> String {
    let mut attr_text = scanned.render_visible();
    let mut is_done = lexer.bracket_depth == 0
        && scanned.segments.iter().any(|s| match s {
            VisibleSegment::Code(c) => c.contains(']'),
            _ => false,
        });
    while !is_done && *idx + 1 < src_lines.len() {
        *idx += 1;
        let next_line = src_lines[*idx];
        if next_line.trim().is_empty() && lexer.state == LexState::Normal {
            continue;
        }
        let continues_literal = lexer.state.is_in_string_literal();
        scanned = lexer.scan_line(next_line);
        let rendered = scanned.render_visible();
        if continues_literal {
            attr_text.push('\n');
        } else if !attr_text.ends_with(' ') && !rendered.starts_with(' ') {
            attr_text.push(' ');
        }
        attr_text.push_str(&rendered);
        is_done = lexer.bracket_depth == 0 && lexer.state == LexState::Normal;
    }
    attr_text
}

fn normalized_public_surface_str(path: &str, src: &str) -> String {
    let src_lines: Vec<&str> = src.lines().collect();
    let mut lines = Vec::new();
    let mut pending_attrs = Vec::new();
    let mut idx = 0usize;
    let mut file_lexer = CodeLexer::new();

    while idx < src_lines.len() {
        let raw_line = src_lines[idx];
        if raw_line.trim().is_empty() && file_lexer.state == LexState::Normal {
            idx += 1;
            continue;
        }

        let mut item_lexer = file_lexer.clone();
        let scanned = item_lexer.scan_line(raw_line);

        if is_outer_attribute_start(&scanned.code_tokens) && file_lexer.state == LexState::Normal {
            let attr_text = capture_attribute(&src_lines, &mut idx, &mut item_lexer, scanned);
            pending_attrs.push(attr_text);
            if item_lexer.state == LexState::Normal {
                item_lexer.reset_top_level_depths();
            }
            file_lexer = item_lexer;
            idx += 1;
            continue;
        }

        if is_public_code(&scanned.code_tokens) {
            lines.append(&mut pending_attrs);

            let mut current_scanned = scanned;
            let mut prefix_text = current_scanned.render_visible();
            let mut combined_code_tokens = current_scanned.code_tokens.clone();

            while is_public_qualifiers_only(&combined_code_tokens) && idx + 1 < src_lines.len() {
                idx += 1;
                let next_line = src_lines[idx];
                if next_line.trim().is_empty() && item_lexer.state == LexState::Normal {
                    continue;
                }
                let continues_literal = item_lexer.state.is_in_string_literal();
                let sc = item_lexer.scan_line(next_line);
                let rendered = sc.render_visible();
                if continues_literal {
                    prefix_text.push('\n');
                    prefix_text.push_str(&rendered);
                } else {
                    if rendered.trim().is_empty() && !sc.ends_in_string_literal {
                        continue;
                    }
                    if !prefix_text.ends_with(' ') && !rendered.starts_with(' ') {
                        prefix_text.push(' ');
                    }
                    prefix_text.push_str(&rendered);
                }
                if !combined_code_tokens.ends_with(' ') && !sc.code_tokens.starts_with(' ') {
                    combined_code_tokens.push(' ');
                }
                combined_code_tokens.push_str(&sc.code_tokens);
                current_scanned = sc;
            }

            if is_public_fn(&combined_code_tokens) {
                let mut signature = if current_scanned.has_top_level_open_brace {
                    let up_to_brace = current_scanned.text_up_to_function_body_open_brace();
                    let rendered_last = current_scanned.render_visible();
                    if prefix_text.ends_with(&rendered_last) {
                        let prefix_head = &prefix_text[..prefix_text.len() - rendered_last.len()];
                        format!("{prefix_head}{up_to_brace}")
                    } else {
                        up_to_brace
                    }
                } else {
                    prefix_text
                };

                while !current_scanned.has_top_level_open_brace
                    && !current_scanned.has_top_level_semicolon
                    && idx + 1 < src_lines.len()
                {
                    idx += 1;
                    let continuation = src_lines[idx];
                    if continuation.trim().is_empty() && item_lexer.state == LexState::Normal {
                        continue;
                    }
                    let was_escaped = item_lexer.state.is_escaped_continuation();
                    let continues_literal = item_lexer.state.is_in_string_literal();
                    current_scanned = item_lexer.scan_line(continuation);
                    let rendered = current_scanned.text_up_to_function_body_open_brace();
                    if was_escaped {
                        signature.push_str(&rendered);
                        continue;
                    }
                    if continues_literal {
                        signature.push('\n');
                        signature.push_str(&rendered);
                        continue;
                    }
                    if rendered.trim().is_empty() {
                        continue;
                    }
                    if !signature.ends_with(' ') && !rendered.starts_with(' ') {
                        signature.push(' ');
                    }
                    signature.push_str(&rendered);
                }
                lines.push(signature);
                if item_lexer.state == LexState::Normal {
                    item_lexer.reset_top_level_depths();
                }
                file_lexer = item_lexer;
                idx += 1;
                continue;
            }

            if is_public_enum(&combined_code_tokens) {
                let mut enum_decl = prefix_text;
                let mut body_open = current_scanned.has_top_level_open_brace;
                let mut body_closed = body_open && current_scanned.has_top_level_close_brace;

                while !body_open && idx + 1 < src_lines.len() {
                    idx += 1;
                    let continuation = src_lines[idx];
                    if continuation.trim().is_empty() && item_lexer.state == LexState::Normal {
                        continue;
                    }
                    let was_escaped = item_lexer.state.is_escaped_continuation();
                    let continues_literal = item_lexer.state.is_in_string_literal();
                    let sc = item_lexer.scan_line(continuation);
                    let rendered = sc.render_visible();
                    if was_escaped {
                        enum_decl.push_str(&rendered);
                        body_open = sc.has_top_level_open_brace;
                        body_closed = body_open && sc.has_top_level_close_brace;
                        continue;
                    }
                    if continues_literal {
                        enum_decl.push('\n');
                        enum_decl.push_str(&rendered);
                        body_open = sc.has_top_level_open_brace;
                        body_closed = body_open && sc.has_top_level_close_brace;
                        continue;
                    }
                    if rendered.trim().is_empty() {
                        continue;
                    }
                    if !enum_decl.ends_with(' ') && !rendered.starts_with(' ') {
                        enum_decl.push(' ');
                    }
                    enum_decl.push_str(&rendered);
                    body_open = sc.has_top_level_open_brace;
                    body_closed = body_open && sc.has_top_level_close_brace;
                }

                if body_closed {
                    // Single-line enum: pub enum State { N, F, T, S }
                    lines.push(enum_decl);
                    if item_lexer.state == LexState::Normal {
                        item_lexer.reset_top_level_depths();
                    }
                    file_lexer = item_lexer;
                    idx += 1;
                    continue;
                }

                lines.push(enum_decl);

                while body_open && idx + 1 < src_lines.len() {
                    idx += 1;
                    let item_line = src_lines[idx];
                    if item_line.trim().is_empty() && item_lexer.state == LexState::Normal {
                        continue;
                    }
                    let sc = item_lexer.scan_line(item_line);

                    let rendered = sc.render_visible();
                    if rendered.trim().is_empty() && !sc.ends_in_string_literal {
                        continue;
                    }
                    if is_outer_attribute_start(&sc.code_tokens) {
                        let attr_text =
                            capture_attribute(&src_lines, &mut idx, &mut item_lexer, sc);
                        lines.push(attr_text);
                        continue;
                    }

                    if sc.has_top_level_close_brace {
                        let without_close_brace = sc.render_visible_without_enum_close_brace();
                        if !without_close_brace.trim().is_empty() {
                            lines.push(without_close_brace);
                        }
                        break;
                    } else {
                        lines.push(rendered);
                    }
                }

                if item_lexer.state == LexState::Normal {
                    item_lexer.reset_top_level_depths();
                }
                file_lexer = item_lexer;
                idx += 1;
                continue;
            }

            if is_public_const_or_static(&combined_code_tokens) {
                let mut item = prefix_text;
                let mut prev_ended_in_string = current_scanned.ends_in_string_literal;
                let mut has_terminator = current_scanned.has_top_level_semicolon;

                while !has_terminator && idx + 1 < src_lines.len() {
                    idx += 1;
                    let next_line = src_lines[idx];
                    if next_line.trim().is_empty() && item_lexer.state == LexState::Normal {
                        continue;
                    }
                    let was_escaped = item_lexer.state.is_escaped_continuation();
                    let sc = item_lexer.scan_line(next_line);
                    has_terminator = sc.has_top_level_semicolon;
                    let rendered = sc.render_visible();
                    if rendered.trim().is_empty() && !sc.ends_in_string_literal {
                        prev_ended_in_string = sc.ends_in_string_literal;
                        continue;
                    }
                    if was_escaped {
                        item.push_str(&rendered);
                    } else if prev_ended_in_string {
                        item.push('\n');
                        item.push_str(&rendered);
                    } else {
                        if !item.ends_with(' ') && !rendered.starts_with(' ') {
                            item.push(' ');
                        }
                        item.push_str(&rendered);
                    }
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

            if is_public_use(&combined_code_tokens) {
                let mut item = prefix_text;
                let mut is_done = current_scanned.has_top_level_semicolon;

                while !is_done && idx + 1 < src_lines.len() {
                    idx += 1;
                    let next_line = src_lines[idx];
                    if next_line.trim().is_empty() && item_lexer.state == LexState::Normal {
                        continue;
                    }
                    let was_escaped = item_lexer.state.is_escaped_continuation();
                    let continues_literal = item_lexer.state.is_in_string_literal();
                    let sc = item_lexer.scan_line(next_line);
                    is_done = sc.has_top_level_semicolon;
                    let rendered = sc.render_visible();
                    if was_escaped {
                        item.push_str(&rendered);
                        continue;
                    }
                    if continues_literal {
                        item.push('\n');
                        item.push_str(&rendered);
                        continue;
                    }
                    if rendered.trim().is_empty() && !sc.ends_in_string_literal {
                        continue;
                    }
                    if !item.ends_with(' ') && !rendered.starts_with(' ') {
                        item.push(' ');
                    }
                    item.push_str(&rendered);
                }

                lines.push(item);
                if item_lexer.state == LexState::Normal {
                    item_lexer.reset_top_level_depths();
                }
                file_lexer = item_lexer;
                idx += 1;
                continue;
            }

            if is_public_struct(&combined_code_tokens) || is_public_union(&combined_code_tokens) {
                let is_unit_or_tuple = current_scanned.has_top_level_semicolon
                    || has_public_tuple_struct_body_open(&combined_code_tokens);

                if is_unit_or_tuple && !current_scanned.has_top_level_open_brace {
                    let mut item = prefix_text;
                    let mut is_done = current_scanned.has_top_level_semicolon
                        || has_public_tuple_struct_body_open(&combined_code_tokens);
                    while !is_done && idx + 1 < src_lines.len() {
                        idx += 1;
                        let continuation = src_lines[idx];
                        if continuation.trim().is_empty() && item_lexer.state == LexState::Normal {
                            continue;
                        }
                        let was_escaped = item_lexer.state.is_escaped_continuation();
                        let continues_literal = item_lexer.state.is_in_string_literal();
                        let sc = item_lexer.scan_line(continuation);
                        let opens_tuple = sc.has_top_level_open_paren
                            && sc.code_tokens.trim_start().starts_with('(');
                        if sc.has_top_level_semicolon || opens_tuple {
                            is_done = true;
                        }
                        let rendered = sc.render_visible();
                        if was_escaped {
                            item.push_str(&rendered);
                            continue;
                        }
                        if continues_literal {
                            item.push('\n');
                            item.push_str(&rendered);
                            continue;
                        }
                        if rendered.trim().is_empty() && !sc.ends_in_string_literal {
                            continue;
                        }
                        if !item.ends_with(' ') && !rendered.starts_with(' ') {
                            item.push(' ');
                        }
                        item.push_str(&rendered);
                    }
                    lines.push(item);
                    if item_lexer.state == LexState::Normal {
                        item_lexer.reset_top_level_depths();
                    }
                    file_lexer = item_lexer;
                    idx += 1;
                    continue;
                }

                // Braced struct / union
                let mut struct_header = if current_scanned.has_top_level_open_brace {
                    let up_to_brace = current_scanned.text_up_to_function_body_open_brace();
                    let rendered_last = current_scanned.render_visible();
                    if prefix_text.ends_with(&rendered_last) {
                        let prefix_head = &prefix_text[..prefix_text.len() - rendered_last.len()];
                        format!("{prefix_head}{up_to_brace}")
                    } else {
                        up_to_brace
                    }
                } else {
                    prefix_text
                };

                let mut body_open = current_scanned.has_top_level_open_brace;

                while !body_open && idx + 1 < src_lines.len() {
                    idx += 1;
                    let continuation = src_lines[idx];
                    if continuation.trim().is_empty() && item_lexer.state == LexState::Normal {
                        continue;
                    }
                    let was_escaped = item_lexer.state.is_escaped_continuation();
                    let continues_literal = item_lexer.state.is_in_string_literal();
                    let sc = item_lexer.scan_line(continuation);
                    let rendered = sc.render_visible();
                    body_open = sc.has_top_level_open_brace;
                    if was_escaped {
                        struct_header.push_str(&rendered);
                        continue;
                    }
                    if continues_literal {
                        struct_header.push('\n');
                        struct_header.push_str(&rendered);
                        continue;
                    }
                    if rendered.trim().is_empty() {
                        continue;
                    }
                    if !struct_header.ends_with(' ') && !rendered.starts_with(' ') {
                        struct_header.push(' ');
                    }
                    struct_header.push_str(&rendered);
                }

                lines.push(struct_header);

                let mut pending_field_attrs = Vec::new();
                let mut cur_field_text = String::new();
                let mut cur_field_code = String::new();
                let mut struct_body_closed = false;

                // Process remainder of opening line (if any)
                if let Some(open_brace_pos) = current_scanned.first_top_level_open_brace_seg {
                    if let Some(start_pos) = next_pos(&current_scanned.segments, open_brace_pos) {
                        let mut cur_pos = Some(start_pos);
                        for event in &current_scanned.events {
                            if event.kind == StructuralEventKind::TopLevelOpenBrace {
                                continue;
                            }
                            if event.seg_idx < start_pos.0
                                || (event.seg_idx == start_pos.0 && event.char_idx < start_pos.1)
                            {
                                continue;
                            }

                            if event.kind == StructuralEventKind::TopLevelCloseBrace {
                                let (chunk_text, chunk_code) = if let Some(limit) = prev_pos(
                                    &current_scanned.segments,
                                    (event.seg_idx, event.char_idx),
                                ) {
                                    (
                                        current_scanned.render_visible_range(cur_pos, Some(limit)),
                                        current_scanned.code_tokens_range(cur_pos, Some(limit)),
                                    )
                                } else {
                                    (String::new(), String::new())
                                };
                                if !chunk_text.trim().is_empty() {
                                    if !cur_field_text.is_empty()
                                        && !cur_field_text.ends_with(' ')
                                        && !chunk_text.starts_with(' ')
                                    {
                                        cur_field_text.push(' ');
                                        cur_field_code.push(' ');
                                    }
                                    cur_field_text.push_str(&chunk_text);
                                    cur_field_code.push_str(&chunk_code);
                                }
                                let trimmed_text = normalize_ws(&cur_field_text);
                                if !trimmed_text.is_empty() && is_public_code(&cur_field_code) {
                                    lines.append(&mut pending_field_attrs);
                                    lines.push(trimmed_text);
                                } else {
                                    pending_field_attrs.clear();
                                }
                                cur_field_text.clear();
                                cur_field_code.clear();
                                struct_body_closed = true;
                                cur_pos = next_pos(
                                    &current_scanned.segments,
                                    (event.seg_idx, event.char_idx),
                                );
                                break;
                            }

                            let chunk_text = current_scanned.render_visible_range(
                                cur_pos,
                                Some((event.seg_idx, event.char_idx)),
                            );
                            let chunk_code = current_scanned
                                .code_tokens_range(cur_pos, Some((event.seg_idx, event.char_idx)));
                            if !chunk_text.trim().is_empty() {
                                if !cur_field_text.is_empty()
                                    && !cur_field_text.ends_with(' ')
                                    && !chunk_text.starts_with(' ')
                                {
                                    cur_field_text.push(' ');
                                    cur_field_code.push(' ');
                                }
                                cur_field_text.push_str(&chunk_text);
                                cur_field_code.push_str(&chunk_code);
                            }

                            if event.kind == StructuralEventKind::BaselineComma {
                                let trimmed_text = normalize_ws(&cur_field_text);
                                if !trimmed_text.is_empty() && is_public_code(&cur_field_code) {
                                    lines.append(&mut pending_field_attrs);
                                    lines.push(trimmed_text);
                                } else {
                                    pending_field_attrs.clear();
                                }
                                cur_field_text.clear();
                                cur_field_code.clear();
                                cur_pos = next_pos(
                                    &current_scanned.segments,
                                    (event.seg_idx, event.char_idx),
                                );
                            }
                        }
                        if !struct_body_closed && cur_pos.is_some() {
                            let remainder_text =
                                current_scanned.render_visible_range(cur_pos, None);
                            let remainder_code = current_scanned.code_tokens_range(cur_pos, None);
                            if !remainder_text.trim().is_empty() {
                                if !cur_field_text.is_empty()
                                    && !cur_field_text.ends_with(' ')
                                    && !remainder_text.starts_with(' ')
                                {
                                    cur_field_text.push(' ');
                                    cur_field_code.push(' ');
                                }
                                cur_field_text.push_str(&remainder_text);
                                cur_field_code.push_str(&remainder_code);
                            }
                        }
                    }
                }

                while !struct_body_closed && idx + 1 < src_lines.len() {
                    idx += 1;
                    let field_line = src_lines[idx];
                    if field_line.trim().is_empty() && item_lexer.state == LexState::Normal {
                        continue;
                    }
                    if cur_field_text.trim().is_empty() && item_lexer.state == LexState::Normal {
                        let mut check_lexer = item_lexer.clone();
                        let check_sc = check_lexer.scan_line(field_line);
                        if is_outer_attribute_start(&check_sc.code_tokens) {
                            let attr_text =
                                capture_attribute(&src_lines, &mut idx, &mut item_lexer, check_sc);
                            pending_field_attrs.push(attr_text);
                            continue;
                        }
                    }

                    let was_escaped = item_lexer.state.is_escaped_continuation();
                    let continues_literal = item_lexer.state.is_in_string_literal();
                    let sc = item_lexer.scan_line(field_line);

                    let mut cur_pos = Some((0, 0));
                    for event in &sc.events {
                        if event.kind == StructuralEventKind::TopLevelCloseBrace {
                            let (chunk_text, chunk_code) = if let Some(limit) =
                                prev_pos(&sc.segments, (event.seg_idx, event.char_idx))
                            {
                                (
                                    sc.render_visible_range(cur_pos, Some(limit)),
                                    sc.code_tokens_range(cur_pos, Some(limit)),
                                )
                            } else {
                                (String::new(), String::new())
                            };
                            if was_escaped {
                                cur_field_text.push_str(&chunk_text);
                                cur_field_code.push_str(&chunk_code);
                            } else if continues_literal {
                                cur_field_text.push('\n');
                                cur_field_text.push_str(&chunk_text);
                                cur_field_code.push_str(&chunk_code);
                            } else if !chunk_text.trim().is_empty() {
                                if !cur_field_text.is_empty()
                                    && !cur_field_text.ends_with(' ')
                                    && !chunk_text.starts_with(' ')
                                {
                                    cur_field_text.push(' ');
                                    cur_field_code.push(' ');
                                }
                                cur_field_text.push_str(&chunk_text);
                                cur_field_code.push_str(&chunk_code);
                            }
                            let trimmed_text = normalize_ws(&cur_field_text);
                            if !trimmed_text.is_empty() && is_public_code(&cur_field_code) {
                                lines.append(&mut pending_field_attrs);
                                lines.push(trimmed_text);
                            } else {
                                pending_field_attrs.clear();
                            }
                            cur_field_text.clear();
                            cur_field_code.clear();
                            struct_body_closed = true;
                            cur_pos = next_pos(&sc.segments, (event.seg_idx, event.char_idx));
                            break;
                        }

                        let chunk_text =
                            sc.render_visible_range(cur_pos, Some((event.seg_idx, event.char_idx)));
                        let chunk_code =
                            sc.code_tokens_range(cur_pos, Some((event.seg_idx, event.char_idx)));
                        if was_escaped {
                            cur_field_text.push_str(&chunk_text);
                            cur_field_code.push_str(&chunk_code);
                        } else if continues_literal {
                            cur_field_text.push('\n');
                            cur_field_text.push_str(&chunk_text);
                            cur_field_code.push_str(&chunk_code);
                        } else if !chunk_text.trim().is_empty() {
                            if !cur_field_text.is_empty()
                                && !cur_field_text.ends_with(' ')
                                && !chunk_text.starts_with(' ')
                            {
                                cur_field_text.push(' ');
                                cur_field_code.push(' ');
                            }
                            cur_field_text.push_str(&chunk_text);
                            cur_field_code.push_str(&chunk_code);
                        }

                        if event.kind == StructuralEventKind::BaselineComma {
                            let trimmed_text = normalize_ws(&cur_field_text);
                            if !trimmed_text.is_empty() && is_public_code(&cur_field_code) {
                                lines.append(&mut pending_field_attrs);
                                lines.push(trimmed_text);
                            } else {
                                pending_field_attrs.clear();
                            }
                            cur_field_text.clear();
                            cur_field_code.clear();
                            cur_pos = next_pos(&sc.segments, (event.seg_idx, event.char_idx));
                        }
                    }

                    if !struct_body_closed && cur_pos.is_some() {
                        let remainder_text = sc.render_visible_range(cur_pos, None);
                        let remainder_code = sc.code_tokens_range(cur_pos, None);
                        if was_escaped {
                            cur_field_text.push_str(&remainder_text);
                            cur_field_code.push_str(&remainder_code);
                        } else if continues_literal {
                            cur_field_text.push('\n');
                            cur_field_text.push_str(&remainder_text);
                            cur_field_code.push_str(&remainder_code);
                        } else if !remainder_text.trim().is_empty() {
                            if !cur_field_text.is_empty()
                                && !cur_field_text.ends_with(' ')
                                && !remainder_text.starts_with(' ')
                            {
                                cur_field_text.push(' ');
                                cur_field_code.push(' ');
                            }
                            cur_field_text.push_str(&remainder_text);
                            cur_field_code.push_str(&remainder_code);
                        }
                    }
                }

                if item_lexer.state == LexState::Normal {
                    item_lexer.reset_top_level_depths();
                }
                file_lexer = item_lexer;
                idx += 1;
                continue;
            }

            if is_public_trait(&combined_code_tokens) {
                let mut trait_header = if current_scanned.has_top_level_open_brace {
                    let up_to_brace = current_scanned.text_up_to_function_body_open_brace();
                    let rendered_last = current_scanned.render_visible();
                    if prefix_text.ends_with(&rendered_last) {
                        let prefix_head = &prefix_text[..prefix_text.len() - rendered_last.len()];
                        format!("{prefix_head}{up_to_brace}")
                    } else {
                        up_to_brace
                    }
                } else {
                    prefix_text
                };

                let mut body_open = current_scanned.has_top_level_open_brace;

                while !body_open && idx + 1 < src_lines.len() {
                    idx += 1;
                    let continuation = src_lines[idx];
                    if continuation.trim().is_empty() && item_lexer.state == LexState::Normal {
                        continue;
                    }
                    let was_escaped = item_lexer.state.is_escaped_continuation();
                    let continues_literal = item_lexer.state.is_in_string_literal();
                    let sc = item_lexer.scan_line(continuation);
                    let rendered = sc.render_visible();
                    body_open = sc.has_top_level_open_brace;
                    if was_escaped {
                        trait_header.push_str(&rendered);
                        continue;
                    }
                    if continues_literal {
                        trait_header.push('\n');
                        trait_header.push_str(&rendered);
                        continue;
                    }
                    if rendered.trim().is_empty() {
                        continue;
                    }
                    if !trait_header.ends_with(' ') && !rendered.starts_with(' ') {
                        trait_header.push(' ');
                    }
                    trait_header.push_str(&rendered);
                }

                lines.push(trait_header);

                let mut pending_trait_attrs = Vec::new();
                let mut cur_member_text = String::new();
                let mut in_default_method_body = false;
                let mut trait_body_closed = false;

                // Process remainder of opening line (if any)
                if let Some(open_brace_pos) = current_scanned.first_top_level_open_brace_seg {
                    if let Some(start_pos) = next_pos(&current_scanned.segments, open_brace_pos) {
                        let mut cur_pos = Some(start_pos);
                        for event in &current_scanned.events {
                            if event.kind == StructuralEventKind::TopLevelOpenBrace {
                                continue;
                            }
                            if event.seg_idx < start_pos.0
                                || (event.seg_idx == start_pos.0 && event.char_idx < start_pos.1)
                            {
                                continue;
                            }

                            if in_default_method_body {
                                if event.kind == StructuralEventKind::MethodBodyClose {
                                    in_default_method_body = false;
                                    cur_pos = next_pos(
                                        &current_scanned.segments,
                                        (event.seg_idx, event.char_idx),
                                    );
                                }
                                continue;
                            }

                            if event.kind == StructuralEventKind::TopLevelCloseBrace {
                                let chunk_text = if let Some(limit) = prev_pos(
                                    &current_scanned.segments,
                                    (event.seg_idx, event.char_idx),
                                ) {
                                    current_scanned.render_visible_range(cur_pos, Some(limit))
                                } else {
                                    String::new()
                                };
                                if !chunk_text.trim().is_empty() {
                                    if !cur_member_text.is_empty()
                                        && !cur_member_text.ends_with(' ')
                                        && !chunk_text.starts_with(' ')
                                    {
                                        cur_member_text.push(' ');
                                    }
                                    cur_member_text.push_str(&chunk_text);
                                }
                                let trimmed_text = normalize_ws(&cur_member_text);
                                if !trimmed_text.is_empty() {
                                    lines.append(&mut pending_trait_attrs);
                                    lines.push(trimmed_text);
                                } else {
                                    pending_trait_attrs.clear();
                                }
                                cur_member_text.clear();
                                trait_body_closed = true;
                                cur_pos = next_pos(
                                    &current_scanned.segments,
                                    (event.seg_idx, event.char_idx),
                                );
                                break;
                            }

                            let chunk_text = current_scanned.render_visible_range(
                                cur_pos,
                                Some((event.seg_idx, event.char_idx)),
                            );
                            if !chunk_text.trim().is_empty() {
                                if !cur_member_text.is_empty()
                                    && !cur_member_text.ends_with(' ')
                                    && !chunk_text.starts_with(' ')
                                {
                                    cur_member_text.push(' ');
                                }
                                cur_member_text.push_str(&chunk_text);
                            }

                            if event.kind == StructuralEventKind::BaselineSemicolon {
                                let trimmed_text = normalize_ws(&cur_member_text);
                                if !trimmed_text.is_empty() {
                                    lines.append(&mut pending_trait_attrs);
                                    lines.push(trimmed_text);
                                } else {
                                    pending_trait_attrs.clear();
                                }
                                cur_member_text.clear();
                                cur_pos = next_pos(
                                    &current_scanned.segments,
                                    (event.seg_idx, event.char_idx),
                                );
                            } else if event.kind == StructuralEventKind::BaselineOpenBrace {
                                let trimmed_text = normalize_ws(&cur_member_text);
                                if !trimmed_text.is_empty() {
                                    lines.append(&mut pending_trait_attrs);
                                    lines.push(trimmed_text);
                                } else {
                                    pending_trait_attrs.clear();
                                }
                                cur_member_text.clear();
                                in_default_method_body = true;
                                cur_pos = next_pos(
                                    &current_scanned.segments,
                                    (event.seg_idx, event.char_idx),
                                );
                            }
                        }
                        if !trait_body_closed && !in_default_method_body && cur_pos.is_some() {
                            let remainder_text =
                                current_scanned.render_visible_range(cur_pos, None);
                            if !remainder_text.trim().is_empty() {
                                if !cur_member_text.is_empty()
                                    && !cur_member_text.ends_with(' ')
                                    && !remainder_text.starts_with(' ')
                                {
                                    cur_member_text.push(' ');
                                }
                                cur_member_text.push_str(&remainder_text);
                            }
                        }
                    }
                }

                while !trait_body_closed && idx + 1 < src_lines.len() {
                    idx += 1;
                    let line_text = src_lines[idx];
                    if line_text.trim().is_empty() && item_lexer.state == LexState::Normal {
                        continue;
                    }

                    if in_default_method_body {
                        let sc = item_lexer.scan_line(line_text);
                        let mut cur_pos = None;
                        for event in &sc.events {
                            if event.kind == StructuralEventKind::MethodBodyClose {
                                in_default_method_body = false;
                                cur_pos = next_pos(&sc.segments, (event.seg_idx, event.char_idx));
                                break;
                            }
                        }
                        if in_default_method_body {
                            if item_lexer.brace_depth == 1
                                && item_lexer.macro_brace_stack.is_empty()
                            {
                                in_default_method_body = false;
                            } else if item_lexer.brace_depth == 0 {
                                break;
                            }
                            continue;
                        }
                        if cur_pos.is_none() {
                            continue;
                        }
                        for event in &sc.events {
                            let Some(pos) = cur_pos else { break };
                            if event.seg_idx < pos.0
                                || (event.seg_idx == pos.0 && event.char_idx < pos.1)
                            {
                                continue;
                            }

                            if event.kind == StructuralEventKind::TopLevelCloseBrace {
                                let chunk_text = if let Some(limit) =
                                    prev_pos(&sc.segments, (event.seg_idx, event.char_idx))
                                {
                                    sc.render_visible_range(cur_pos, Some(limit))
                                } else {
                                    String::new()
                                };
                                if !chunk_text.trim().is_empty() {
                                    if !cur_member_text.is_empty()
                                        && !cur_member_text.ends_with(' ')
                                        && !chunk_text.starts_with(' ')
                                    {
                                        cur_member_text.push(' ');
                                    }
                                    cur_member_text.push_str(&chunk_text);
                                }
                                let trimmed_text = normalize_ws(&cur_member_text);
                                if !trimmed_text.is_empty() {
                                    lines.append(&mut pending_trait_attrs);
                                    lines.push(trimmed_text);
                                } else {
                                    pending_trait_attrs.clear();
                                }
                                cur_member_text.clear();
                                trait_body_closed = true;
                                cur_pos = next_pos(&sc.segments, (event.seg_idx, event.char_idx));
                                break;
                            }

                            let chunk_text = sc.render_visible_range(
                                cur_pos,
                                Some((event.seg_idx, event.char_idx)),
                            );
                            if !chunk_text.trim().is_empty() {
                                if !cur_member_text.is_empty()
                                    && !cur_member_text.ends_with(' ')
                                    && !chunk_text.starts_with(' ')
                                {
                                    cur_member_text.push(' ');
                                }
                                cur_member_text.push_str(&chunk_text);
                            }

                            if event.kind == StructuralEventKind::BaselineSemicolon {
                                let trimmed_text = normalize_ws(&cur_member_text);
                                if !trimmed_text.is_empty() {
                                    lines.append(&mut pending_trait_attrs);
                                    lines.push(trimmed_text);
                                } else {
                                    pending_trait_attrs.clear();
                                }
                                cur_member_text.clear();
                                cur_pos = next_pos(&sc.segments, (event.seg_idx, event.char_idx));
                            } else if event.kind == StructuralEventKind::BaselineOpenBrace {
                                let trimmed_text = normalize_ws(&cur_member_text);
                                if !trimmed_text.is_empty() {
                                    lines.append(&mut pending_trait_attrs);
                                    lines.push(trimmed_text);
                                } else {
                                    pending_trait_attrs.clear();
                                }
                                cur_member_text.clear();
                                in_default_method_body = true;
                                cur_pos = next_pos(&sc.segments, (event.seg_idx, event.char_idx));
                            }
                        }
                        if !trait_body_closed && !in_default_method_body && cur_pos.is_some() {
                            let remainder_text = sc.render_visible_range(cur_pos, None);
                            if !remainder_text.trim().is_empty() {
                                if !cur_member_text.is_empty()
                                    && !cur_member_text.ends_with(' ')
                                    && !remainder_text.starts_with(' ')
                                {
                                    cur_member_text.push(' ');
                                }
                                cur_member_text.push_str(&remainder_text);
                            }
                        }
                        continue;
                    }

                    if cur_member_text.trim().is_empty() && item_lexer.state == LexState::Normal {
                        let mut check_lexer = item_lexer.clone();
                        let check_sc = check_lexer.scan_line(line_text);
                        if is_outer_attribute_start(&check_sc.code_tokens) {
                            let attr_text =
                                capture_attribute(&src_lines, &mut idx, &mut item_lexer, check_sc);
                            pending_trait_attrs.push(attr_text);
                            continue;
                        }
                    }

                    let was_escaped = item_lexer.state.is_escaped_continuation();
                    let continues_literal = item_lexer.state.is_in_string_literal();
                    let sc = item_lexer.scan_line(line_text);

                    let mut cur_pos = Some((0, 0));
                    for event in &sc.events {
                        if event.kind == StructuralEventKind::TopLevelCloseBrace {
                            let chunk_text = if let Some(limit) =
                                prev_pos(&sc.segments, (event.seg_idx, event.char_idx))
                            {
                                sc.render_visible_range(cur_pos, Some(limit))
                            } else {
                                String::new()
                            };
                            if was_escaped {
                                cur_member_text.push_str(&chunk_text);
                            } else if continues_literal {
                                cur_member_text.push('\n');
                                cur_member_text.push_str(&chunk_text);
                            } else if !chunk_text.trim().is_empty() {
                                if !cur_member_text.is_empty()
                                    && !cur_member_text.ends_with(' ')
                                    && !chunk_text.starts_with(' ')
                                {
                                    cur_member_text.push(' ');
                                }
                                cur_member_text.push_str(&chunk_text);
                            }
                            let trimmed_text = normalize_ws(&cur_member_text);
                            if !trimmed_text.is_empty() {
                                lines.append(&mut pending_trait_attrs);
                                lines.push(trimmed_text);
                            } else {
                                pending_trait_attrs.clear();
                            }
                            cur_member_text.clear();
                            trait_body_closed = true;
                            cur_pos = next_pos(&sc.segments, (event.seg_idx, event.char_idx));
                            break;
                        }

                        let chunk_text =
                            sc.render_visible_range(cur_pos, Some((event.seg_idx, event.char_idx)));
                        if was_escaped {
                            cur_member_text.push_str(&chunk_text);
                        } else if continues_literal {
                            cur_member_text.push('\n');
                            cur_member_text.push_str(&chunk_text);
                        } else if !chunk_text.trim().is_empty() {
                            if !cur_member_text.is_empty()
                                && !cur_member_text.ends_with(' ')
                                && !chunk_text.starts_with(' ')
                            {
                                cur_member_text.push(' ');
                            }
                            cur_member_text.push_str(&chunk_text);
                        }

                        if event.kind == StructuralEventKind::BaselineSemicolon {
                            let trimmed_text = normalize_ws(&cur_member_text);
                            if !trimmed_text.is_empty() {
                                lines.append(&mut pending_trait_attrs);
                                lines.push(trimmed_text);
                            } else {
                                pending_trait_attrs.clear();
                            }
                            cur_member_text.clear();
                            cur_pos = next_pos(&sc.segments, (event.seg_idx, event.char_idx));
                        } else if event.kind == StructuralEventKind::BaselineOpenBrace {
                            let trimmed_text = normalize_ws(&cur_member_text);
                            if !trimmed_text.is_empty() {
                                lines.append(&mut pending_trait_attrs);
                                lines.push(trimmed_text);
                            } else {
                                pending_trait_attrs.clear();
                            }
                            cur_member_text.clear();
                            in_default_method_body = true;
                            cur_pos = next_pos(&sc.segments, (event.seg_idx, event.char_idx));
                        }
                    }

                    if !trait_body_closed && !in_default_method_body && cur_pos.is_some() {
                        let remainder_text = sc.render_visible_range(cur_pos, None);
                        if was_escaped {
                            cur_member_text.push_str(&remainder_text);
                        } else if continues_literal {
                            cur_member_text.push('\n');
                            cur_member_text.push_str(&remainder_text);
                        } else if !remainder_text.trim().is_empty() {
                            if !cur_member_text.is_empty()
                                && !cur_member_text.ends_with(' ')
                                && !remainder_text.starts_with(' ')
                            {
                                cur_member_text.push(' ');
                            }
                            cur_member_text.push_str(&remainder_text);
                        }
                    }
                }

                if item_lexer.state == LexState::Normal {
                    item_lexer.reset_top_level_depths();
                }
                file_lexer = item_lexer;
                idx += 1;
                continue;
            }

            // Other public items (type alias, mod, etc.)
            let is_item_done = |sc: &ScannedLine| {
                sc.has_top_level_open_brace || sc.has_top_level_semicolon || sc.has_top_level_comma
            };
            let mut is_complete = is_item_done(&current_scanned);

            let mut item = if current_scanned.has_top_level_open_brace {
                let up_to_brace = current_scanned.text_up_to_function_body_open_brace();
                let rendered_last = current_scanned.render_visible();
                if prefix_text.ends_with(&rendered_last) {
                    let prefix_head = &prefix_text[..prefix_text.len() - rendered_last.len()];
                    format!("{prefix_head}{up_to_brace}")
                } else {
                    up_to_brace
                }
            } else {
                prefix_text
            };

            while !is_complete && idx + 1 < src_lines.len() {
                idx += 1;
                let continuation = src_lines[idx];
                if continuation.trim().is_empty() && item_lexer.state == LexState::Normal {
                    continue;
                }
                let was_escaped = item_lexer.state.is_escaped_continuation();
                let continues_literal = item_lexer.state.is_in_string_literal();
                let sc = item_lexer.scan_line(continuation);
                if is_item_done(&sc) {
                    is_complete = true;
                }
                let rendered = if sc.has_top_level_open_brace {
                    sc.text_up_to_function_body_open_brace()
                } else {
                    sc.render_visible()
                };
                if was_escaped {
                    item.push_str(&rendered);
                    continue;
                }
                if continues_literal {
                    item.push('\n');
                    item.push_str(&rendered);
                    continue;
                }
                if rendered.trim().is_empty() && !sc.ends_in_string_literal {
                    continue;
                }
                if !item.ends_with(' ') && !rendered.starts_with(' ') {
                    item.push(' ');
                }
                item.push_str(&rendered);
            }

            lines.push(item);
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
fn public_api_guard_does_not_treat_array_length_semicolon_as_function_terminator() {
    let returns_u32 = r#"
pub fn f(
    x: [u8; 3]
) -> u32
{
    private_impl_a()
}
"#;
    let returns_u64 = returns_u32.replace("-> u32", "-> u64");
    let different_private_body = returns_u32.replace("private_impl_a", "private_impl_b");

    let surface_u32 = normalized_public_surface_str("test.rs", returns_u32);
    let surface_u64 = normalized_public_surface_str("test.rs", &returns_u64);
    let surface_different_body = normalized_public_surface_str("test.rs", &different_private_body);

    assert_ne!(
        surface_u32, surface_u64,
        "return-type drift after an array-length semicolon must alter the public surface"
    );
    assert_eq!(
        surface_u32, surface_different_body,
        "private body drift must remain excluded from the public surface"
    );
}

#[test]
fn public_api_guard_does_not_treat_array_length_semicolon_as_const_terminator() {
    let value_3 = r#"
pub const X: [u8; 3] =
[
    1,
    2,
    3,
];
"#;
    let value_4 = value_3.replace("    3,", "    4,");

    assert_ne!(
        normalized_public_surface_str("test.rs", value_3),
        normalized_public_surface_str("test.rs", &value_4),
        "initializer drift after an array-type semicolon must alter the public surface"
    );
}

#[test]
fn public_api_guard_ignores_const_generic_braces_before_enum_body() {
    let variant_b = r#"
pub enum E<const N: usize = { 1 }>
{
    A,
    B,
}
"#;
    let renamed_variant = variant_b.replace("    A,", "    RenamedA,");
    let reformatted_const = variant_b.replace("{ 1 }", "{   1   }");

    assert_ne!(
        normalized_public_surface_str("test.rs", variant_b),
        normalized_public_surface_str("test.rs", &renamed_variant),
        "an enum body after a const-generic block must remain contract-bearing"
    );
    assert_eq!(
        normalized_public_surface_str("test.rs", variant_b),
        normalized_public_surface_str("test.rs", &reformatted_const),
        "formatting-only whitespace inside const-generic code must remain normalized"
    );
}

#[test]
fn public_api_guard_ignores_nested_const_blocks_in_generic_public_items() {
    let u32_alias = r#"
pub type X = SomeType<
    { 1 },
    u32,
>;
"#;
    let u64_alias = u32_alias.replace("    u32,", "    u64,");

    assert_ne!(
        normalized_public_surface_str("test.rs", u32_alias),
        normalized_public_surface_str("test.rs", &u64_alias),
        "a nested const block must not hide a later type-alias component"
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

    // S. supported_headers ordered family mismatch: MUST DETECT
    let canonical_headers = sm_format::semcode_format::supported_headers();
    let mut reordered = canonical_headers.to_vec();
    reordered.swap(0, 1);
    assert_ne!(
        canonical_headers,
        &reordered[..],
        "S. reordering supported_headers must be detected"
    );
    let mut truncated = canonical_headers.to_vec();
    truncated.pop();
    assert_ne!(
        canonical_headers,
        &truncated[..],
        "S. removing header from supported_headers must be detected"
    );

    // T. const-generic comparison operator '>': MUST DETECT signature change
    let old_t = "pub fn build() -> Foo<{ if 1 > 0 { 32 } else { 64 } }> {\n    private_a()\n}";
    let new_t = "pub fn build() -> Foo<{ if 1 > 0 { 33 } else { 64 } }> {\n    private_a()\n}";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_t),
        normalized_public_surface_str("t.rs", new_t),
        "T. const-generic comparison operator '>' signature change must be detected"
    );
    let same_body_t =
        "pub fn build() -> Foo<{ if 1 > 0 { 32 } else { 64 } }> {\n    private_b()\n}";
    assert_eq!(
        normalized_public_surface_str("t.rs", old_t),
        normalized_public_surface_str("t.rs", same_body_t),
        "T. private body change must not change surface"
    );

    // U. const-generic shift operator '<<': MUST preserve correct function boundary
    let old_u = "pub fn build() -> Foo<{ 1 << 2 }> {\n    private_a()\n}\npub const NEXT: u32 = 1;";
    let new_u = "pub fn build() -> Foo<{ 1 << 3 }> {\n    private_a()\n}\npub const NEXT: u32 = 1;";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_u),
        normalized_public_surface_str("t.rs", new_u),
        "U. const-generic shift operator '<<' signature change must be detected"
    );
    let same_body_u =
        "pub fn build() -> Foo<{ 1 << 2 }> {\n    private_b()\n}\npub const NEXT: u32 = 1;";
    assert_eq!(
        normalized_public_surface_str("t.rs", old_u),
        normalized_public_surface_str("t.rs", same_body_u),
        "U. private body change must not change surface"
    );

    // V. combined function qualifiers: MUST classify as function
    let old_v = "pub const unsafe fn qualified() -> Foo<{ 32 }> {\n    private_a()\n}";
    let new_v = "pub const unsafe fn qualified() -> Foo<{ 64 }> {\n    private_a()\n}";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_v),
        normalized_public_surface_str("t.rs", new_v),
        "V. combined qualifiers signature change must be detected"
    );
    let same_body_v = "pub const unsafe fn qualified() -> Foo<{ 32 }> {\n    private_b()\n}";
    assert_eq!(
        normalized_public_surface_str("t.rs", old_v),
        normalized_public_surface_str("t.rs", same_body_v),
        "V. combined qualifiers private body change must not change surface"
    );

    // W. multiline cfg_attr predicate change: MUST DETECT
    let old_w =
        "#[cfg_attr(\n    feature = \"x\",\n    deprecated(note = \"old\")\n)]\npub fn api() {}";
    let new_w =
        "#[cfg_attr(\n    feature = \"y\",\n    deprecated(note = \"old\")\n)]\npub fn api() {}";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_w),
        normalized_public_surface_str("t.rs", new_w),
        "W. multiline cfg_attr predicate change must be detected"
    );

    // X. multiline attribute literal-value change: MUST DETECT
    let old_x =
        "#[cfg_attr(\n    feature = \"x\",\n    deprecated(note = \"old\")\n)]\npub fn api() {}";
    let new_x =
        "#[cfg_attr(\n    feature = \"x\",\n    deprecated(note = \"new\")\n)]\npub fn api() {}";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_x),
        normalized_public_surface_str("t.rs", new_x),
        "X. multiline attribute literal value change must be detected"
    );

    // Y. Normal multiline string trailing whitespace inside literal: MUST DETECT
    let old_y = format!("pub const S: &str = \"a{}{}\nb\";", ' ', ' ');
    let new_y = format!("pub const S: &str = \"a{}\nb\";", ' ');
    assert_ne!(
        normalized_public_surface_str("t.rs", &old_y),
        normalized_public_surface_str("t.rs", &new_y),
        "Y. normal multiline string trailing whitespace change must be detected"
    );

    // Z. Raw multiline string trailing whitespace inside literal: MUST DETECT
    let old_z = format!("pub const R: &str = r#\"a{}{}\nb\"#;", ' ', ' ');
    let new_z = format!("pub const R: &str = r#\"a{}\nb\"#;", ' ');
    assert_ne!(
        normalized_public_surface_str("t.rs", &old_z),
        normalized_public_surface_str("t.rs", &new_z),
        "Z. raw multiline string trailing whitespace change must be detected"
    );

    // AA. Literal whitespace inside struct const-generic default: MUST DETECT
    let old_aa = "pub struct S<const MSG: &'static str = \"a  b\">;";
    let new_aa = "pub struct S<const MSG: &'static str = \"a b\">;";
    assert_ne!(
        normalized_public_surface_str("t.rs", old_aa),
        normalized_public_surface_str("t.rs", new_aa),
        "AA. literal whitespace inside struct generic default must be detected"
    );
}

#[test]
fn supported_headers_match_canonical_contract() {
    use sm_format::semcode_format::*;

    let canonical_family: &[SemcodeHeaderSpec] = &[
        HEADER_V0, HEADER_V1, HEADER_V2, HEADER_V3, HEADER_V4, HEADER_V5, HEADER_V6, HEADER_V7,
        HEADER_V8, HEADER_V9, HEADER_V10, HEADER_V11, HEADER_V12, HEADER_V13, HEADER_V14,
        HEADER_V15, HEADER_V16, HEADER_V17, HEADER_V18, HEADER_V19,
    ];

    let actual = supported_headers();
    assert_eq!(
        actual, canonical_family,
        "supported_headers() must return exactly the canonical supported header family in canonical order"
    );

    for spec in actual {
        assert_eq!(
            header_spec_from_magic(&spec.magic),
            Some(*spec),
            "header_spec_from_magic must resolve canonical header for magic {:?}",
            spec.magic
        );
    }
}

#[test]
fn public_api_guard_handles_generic_angle_tokens_and_operators() {
    let fn_cmp_32 = r#"
pub fn build() -> Foo<{ if 1 > 0 { 32 } else { 64 } }> {
    private_a()
}
"#;
    let fn_cmp_33 = r#"
pub fn build() -> Foo<{ if 1 > 0 { 33 } else { 64 } }> {
    private_a()
}
"#;
    let fn_cmp_diff_body = r#"
pub fn build() -> Foo<{ if 1 > 0 { 32 } else { 64 } }> {
    private_b()
}
"#;
    let surf_cmp_32 = normalized_public_surface_str("test.rs", fn_cmp_32);
    let surf_cmp_33 = normalized_public_surface_str("test.rs", fn_cmp_33);
    let surf_cmp_diff_body = normalized_public_surface_str("test.rs", fn_cmp_diff_body);

    assert_ne!(
        surf_cmp_32, surf_cmp_33,
        "changing const-generic expression with '>' operator must alter surface"
    );
    assert_eq!(
        surf_cmp_32, surf_cmp_diff_body,
        "changing private body in function with const-generic '>' operator must not change surface"
    );
    assert!(
        !surf_cmp_32.contains("private_a"),
        "private body must not be captured: {surf_cmp_32}"
    );

    let fn_shift_1 = r#"
pub fn build() -> Foo<{ 1 << 2 }> {
    private_a()
}
pub const NEXT: u32 = 1;
"#;
    let fn_shift_2 = r#"
pub fn build() -> Foo<{ 1 << 3 }> {
    private_a()
}
pub const NEXT: u32 = 1;
"#;
    let fn_shift_diff_body = r#"
pub fn build() -> Foo<{ 1 << 2 }> {
    private_b()
}
pub const NEXT: u32 = 1;
"#;
    let surf_shift_1 = normalized_public_surface_str("test.rs", fn_shift_1);
    let surf_shift_2 = normalized_public_surface_str("test.rs", fn_shift_2);
    let surf_shift_diff_body = normalized_public_surface_str("test.rs", fn_shift_diff_body);

    assert_ne!(
        surf_shift_1, surf_shift_2,
        "changing const-generic expression with '<<' shift operator must alter surface"
    );
    assert_eq!(
        surf_shift_1, surf_shift_diff_body,
        "changing private body in function with '<<' shift operator must not change surface"
    );
    assert!(
        surf_shift_1.contains("pub const NEXT: u32 = 1;"),
        "following declaration must remain separate: {surf_shift_1}"
    );

    let fn_less = r#"
pub fn build() -> Foo<{ if 1 < 2 { 32 } else { 64 } }> {
    private_a()
}
"#;
    let surf_less = normalized_public_surface_str("test.rs", fn_less);
    assert!(
        surf_less.contains("pub fn build() -> Foo<{ if 1 < 2 { 32 } else { 64 } }> {"),
        "signature with '<' inside const block must be captured: {surf_less}"
    );
    assert!(
        !surf_less.contains("private_a"),
        "private body must not be captured: {surf_less}"
    );
}

#[test]
fn public_api_guard_treats_const_initializer_comparisons_as_top_level() {
    let private_a =
        "pub const LESS: bool = 1 < 2;\nfn hidden() -> u32 { 1 }\npub const NEXT: u32 = 7;";
    let private_b =
        "pub const LESS: bool = 1 < 2;\nfn hidden() -> u64 { 2 }\npub const NEXT: u32 = 7;";
    let changed =
        "pub const LESS: bool = 1 > 2;\nfn hidden() -> u32 { 1 }\npub const NEXT: u32 = 7;";

    let surface_a = normalized_public_surface_str("test.rs", private_a);
    let surface_b = normalized_public_surface_str("test.rs", private_b);
    assert_eq!(
        surface_a, surface_b,
        "private declarations after a const comparison must not alter the public surface"
    );
    assert_ne!(
        surface_a,
        normalized_public_surface_str("test.rs", changed),
        "changing the public const comparison must alter the public surface"
    );
    assert!(
        surface_a.contains("pub const NEXT: u32 = 7;"),
        "the public declaration after a const comparison must remain inventoried: {surface_a}"
    );
    assert!(
        !surface_a.contains("hidden"),
        "private declarations must not be captured: {surface_a}"
    );
}

#[test]
fn public_api_guard_handles_combined_function_qualifiers() {
    let fn_qual_32 = r#"
pub const unsafe fn qualified() -> Foo<{
    32
}> {
    private_a()
}
"#;
    let fn_qual_64 = r#"
pub const unsafe fn qualified() -> Foo<{
    64
}> {
    private_a()
}
"#;
    let fn_qual_diff_body = r#"
pub const unsafe fn qualified() -> Foo<{
    32
}> {
    private_b()
}
"#;
    let surf_qual_32 = normalized_public_surface_str("test.rs", fn_qual_32);
    let surf_qual_64 = normalized_public_surface_str("test.rs", fn_qual_64);
    let surf_qual_diff_body = normalized_public_surface_str("test.rs", fn_qual_diff_body);

    assert_ne!(
        surf_qual_32, surf_qual_64,
        "changing 32 -> 64 in combined qualifiers function must alter surface"
    );
    assert_eq!(
        surf_qual_32, surf_qual_diff_body,
        "changing only private_a -> private_b in combined qualifiers function must not alter surface"
    );
    assert!(
        !surf_qual_32.contains("private_a"),
        "private body must not be captured: {surf_qual_32}"
    );

    let fn_ffi = r#"
pub unsafe extern "C" fn ffi_entry() {
    private_impl()
}
"#;
    let surf_ffi = normalized_public_surface_str("test.rs", fn_ffi);
    assert!(
        surf_ffi.contains("pub unsafe extern \"C\" fn ffi_entry() {"),
        "pub unsafe extern \"C\" fn must be classified as function: {surf_ffi}"
    );
    assert!(
        !surf_ffi.contains("private_impl"),
        "private body must not be captured: {surf_ffi}"
    );
}

#[test]
fn public_api_guard_handles_multiline_outer_attributes() {
    let src_x_old = r#"
#[cfg_attr(
    feature = "x",
    deprecated(note = "old")
)]
pub fn api() {}
"#;
    let src_y_old = r#"
#[cfg_attr(
    feature = "y",
    deprecated(note = "old")
)]
pub fn api() {}
"#;
    let src_x_new = r#"
#[cfg_attr(
    feature = "x",
    deprecated(note = "new")
)]
pub fn api() {}
"#;
    let surf_x_old = normalized_public_surface_str("test.rs", src_x_old);
    let surf_y_old = normalized_public_surface_str("test.rs", src_y_old);
    let surf_x_new = normalized_public_surface_str("test.rs", src_x_new);

    assert_ne!(
        surf_x_old, surf_y_old,
        "changing attribute predicate feature x -> y must change surface"
    );
    assert_ne!(
        surf_x_old, surf_x_new,
        "changing attribute literal note old -> new must change surface"
    );

    let src_x_reformatted = r#"
#[cfg_attr(  feature = "x",   /* comment */  deprecated(note = "old")  )]
pub fn api() {}
"#;
    let surf_x_reformatted = normalized_public_surface_str("test.rs", src_x_reformatted);
    assert_eq!(
        surf_x_old, surf_x_reformatted,
        "formatting/comments-only changes to attribute must not change surface: {surf_x_old} vs {surf_x_reformatted}"
    );
}

#[test]
fn public_api_guard_preserves_newlines_inside_multiline_attribute_literals() {
    let multiline = "#[deprecated(note = \"a\nb\")]\npub fn api() {}";
    let single_line = "#[deprecated(note = \"a b\")]\npub fn api() {}";

    assert_ne!(
        normalized_public_surface_str("test.rs", multiline),
        normalized_public_surface_str("test.rs", single_line),
        "a literal newline in public attribute metadata must not normalize to a space"
    );
}

#[test]
fn public_api_guard_normalizes_multiline_enum_variant_attributes() {
    let single_line = r#"
pub enum E {
    #[cfg_attr( feature = "x", deprecated(note = "old") )]
    A,
}
"#;
    let multiline = r#"
pub enum E {
    #[cfg_attr(
        feature = "x",
        deprecated(note = "old")
    )]
    A,
}
"#;
    let changed = r#"
pub enum E {
    #[cfg_attr(
        feature = "x",
        deprecated(note = "new")
    )]
    A,
}
"#;

    assert_eq!(
        normalized_public_surface_str("test.rs", single_line),
        normalized_public_surface_str("test.rs", multiline),
        "formatting-only changes to an enum variant attribute must not alter the public surface"
    );
    assert_ne!(
        normalized_public_surface_str("test.rs", multiline),
        normalized_public_surface_str("test.rs", changed),
        "contract changes inside a multiline enum variant attribute must alter the public surface"
    );
}

#[test]
fn public_api_guard_ignores_private_multiline_tuple_struct_fields() {
    let private_u32 = "pub struct S(\n    u32,\n);\npub const NEXT: u32 = 1;";
    let private_u64 = "pub struct S(\n    u64,\n);\npub const NEXT: u32 = 1;";

    let u32_surface = normalized_public_surface_str("test.rs", private_u32);
    let u64_surface = normalized_public_surface_str("test.rs", private_u64);
    assert_eq!(
        u32_surface, u64_surface,
        "changing a private tuple field type must not alter the public surface"
    );
    assert!(
        u32_surface.contains("pub const NEXT: u32 = 1;"),
        "the declaration after a tuple struct must remain inventoried: {u32_surface}"
    );
}

#[test]
fn public_api_guard_captures_public_multiline_tuple_struct_fields() {
    let public_u32 = "pub struct S(\n    pub u32,\n);";
    let public_u64 = "pub struct S(\n    pub u64,\n);";

    assert_ne!(
        normalized_public_surface_str("test.rs", public_u32),
        normalized_public_surface_str("test.rs", public_u64),
        "changing a public tuple field type must alter the public surface"
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

#[test]
fn public_api_guard_preserves_multiline_string_trailing_literal_whitespace() {
    // A. Normal multiline string trailing whitespace: 2 spaces vs 1 space before newline
    let s_two_spaces = format!("pub const S: &str = \"a{}{}\nb\";", ' ', ' ');
    let s_one_space = format!("pub const S: &str = \"a{}\nb\";", ' ');
    let surf_s_two = normalized_public_surface_str("test.rs", &s_two_spaces);
    let surf_s_one = normalized_public_surface_str("test.rs", &s_one_space);
    assert_ne!(
        surf_s_two, surf_s_one,
        "normal multiline string trailing whitespace inside literal must change surface"
    );

    // B. Raw multiline string trailing whitespace: 2 spaces vs 1 space before newline
    let r_two_spaces = format!("pub const R: &str = r#\"a{}{}\nb\"#;", ' ', ' ');
    let r_one_space = format!("pub const R: &str = r#\"a{}\nb\"#;", ' ');
    let surf_r_two = normalized_public_surface_str("test.rs", &r_two_spaces);
    let surf_r_one = normalized_public_surface_str("test.rs", &r_one_space);
    assert_ne!(
        surf_r_two, surf_r_one,
        "raw multiline string trailing whitespace inside literal must change surface"
    );
}

#[test]
fn public_api_guard_preserves_literal_whitespace_in_signatures_and_generic_items() {
    // C. Literal whitespace in public function signature
    let fn_lit_two_spaces = "pub fn build() -> Foo<\"a  b\"> {\n    private_a()\n}";
    let fn_lit_one_space = "pub fn build() -> Foo<\"a b\"> {\n    private_a()\n}";
    let fn_lit_diff_body = "pub fn build() -> Foo<\"a  b\"> {\n    private_b()\n}";
    let surf_fn_two = normalized_public_surface_str("test.rs", fn_lit_two_spaces);
    let surf_fn_one = normalized_public_surface_str("test.rs", fn_lit_one_space);
    let surf_fn_diff_body = normalized_public_surface_str("test.rs", fn_lit_diff_body);

    assert_ne!(
        surf_fn_two, surf_fn_one,
        "literal whitespace change in function signature must change surface"
    );
    assert_eq!(
        surf_fn_two, surf_fn_diff_body,
        "private body change must not change surface"
    );
    assert!(
        !surf_fn_two.contains("private_a"),
        "private body must not be captured in surface: {surf_fn_two}"
    );

    // D. Literal whitespace in struct, enum variant, and type alias
    let struct_two_spaces = "pub struct S<const MSG: &'static str = \"a  b\">;";
    let struct_one_space = "pub struct S<const MSG: &'static str = \"a b\">;";
    assert_ne!(
        normalized_public_surface_str("test.rs", struct_two_spaces),
        normalized_public_surface_str("test.rs", struct_one_space),
        "literal whitespace in struct const generic default must change surface"
    );

    let type_two_spaces = "pub type Msg = StaticMsg<\"a  b\">;";
    let type_one_space = "pub type Msg = StaticMsg<\"a b\">;";
    assert_ne!(
        normalized_public_surface_str("test.rs", type_two_spaces),
        normalized_public_surface_str("test.rs", type_one_space),
        "literal whitespace in type alias must change surface"
    );

    let enum_two_spaces = "pub enum E {\n    Variant = \"a  b\",\n}";
    let enum_one_space = "pub enum E {\n    Variant = \"a b\",\n}";
    assert_ne!(
        normalized_public_surface_str("test.rs", enum_two_spaces),
        normalized_public_surface_str("test.rs", enum_one_space),
        "literal whitespace in enum variant must change surface"
    );
}

#[test]
fn public_api_guard_preserves_literal_newlines_in_function_signatures() {
    let multiline = "pub fn f() -> type_macro!(\"a\nb\") {\n    private_a()\n}";
    let single_line = "pub fn f() -> type_macro!(\"a b\") {\n    private_a()\n}";
    let different_body = "pub fn f() -> type_macro!(\"a\nb\") {\n    private_b()\n}";

    let multiline_surface = normalized_public_surface_str("test.rs", multiline);
    assert_ne!(
        multiline_surface,
        normalized_public_surface_str("test.rs", single_line),
        "a literal newline in a public function signature must not normalize to a space"
    );
    assert_eq!(
        multiline_surface,
        normalized_public_surface_str("test.rs", different_body),
        "private function-body changes must not alter the public surface"
    );
    assert!(
        !multiline_surface.contains("private_a"),
        "private function bodies must not be captured: {multiline_surface}"
    );
}

#[test]
fn public_api_guard_normalizes_formatting_whitespace_outside_literals() {
    // E. Formatting outside literals remains normalized
    let src_spaces = "    pub   const   X:   u32   =   1;\n";
    let src_normal = "pub const X: u32 = 1;\n";
    assert_eq!(
        normalized_public_surface_str("test.rs", src_spaces),
        normalized_public_surface_str("test.rs", src_normal),
        "whitespace formatting outside literals must remain normalized"
    );

    let src_comments = "pub /* c1 */ const /* c2 */ X: u32 = 1;\n";
    assert_eq!(
        normalized_public_surface_str("test.rs", src_comments),
        normalized_public_surface_str("test.rs", src_normal),
        "comments-only differences outside literals must remain normalized"
    );
}

#[test]
fn public_api_guard_ignores_multiline_block_comments_inside_enums() {
    let src_short_comment = r#"
pub enum Mode {
    /* brief comment */
    Active,
    Inactive,
}
"#;
    let src_multiline_comment = r#"
pub enum Mode {
    /*
     * Detailed multiline
     * block comment with
     * several lines
     */
    Active,
    Inactive,
}
"#;
    let surf_short = normalized_public_surface_str("test.rs", src_short_comment);
    let surf_multiline = normalized_public_surface_str("test.rs", src_multiline_comment);
    assert_eq!(
        surf_short, surf_multiline,
        "multiline block comments inside enums must not produce spurious empty lines or alter snapshot: {surf_short} vs {surf_multiline}"
    );
}

#[test]
fn public_api_guard_handles_top_level_less_than_comparison() {
    let src_less_old = r#"
pub const LESS: bool = 1 < 2;
pub fn next_api() {
    private_a()
}
"#;
    let src_less_new = r#"
pub const LESS: bool = 1 < 3;
pub fn next_api() {
    private_a()
}
"#;
    let src_less_diff_body = r#"
pub const LESS: bool = 1 < 2;
pub fn next_api() {
    private_b()
}
"#;
    let surf_old = normalized_public_surface_str("test.rs", src_less_old);
    let surf_new = normalized_public_surface_str("test.rs", src_less_new);
    let surf_diff_body = normalized_public_surface_str("test.rs", src_less_diff_body);

    assert_ne!(
        surf_old, surf_new,
        "changing value in top-level '<' comparison const must alter surface"
    );
    assert_eq!(
        surf_old, surf_diff_body,
        "changing private body in next_api after top-level '<' const must not alter surface"
    );
    assert!(
        !surf_old.contains("private_a"),
        "private body of next_api must not be captured: {surf_old}"
    );
    assert!(
        surf_old.contains("pub const LESS: bool = 1 < 2;"),
        "const declaration must be captured intact: {surf_old}"
    );
    assert!(
        surf_old.contains("pub fn next_api() {"),
        "next function must be inventoried separately: {surf_old}"
    );
}

#[test]
fn public_api_guard_preserves_multiline_literal_newlines_in_signatures() {
    let fn_nl = "pub fn f() -> type_macro!(\"a\nb\") {\n    private_impl()\n}";
    let fn_sp = "pub fn f() -> type_macro!(\"a b\") {\n    private_impl()\n}";
    let fn_nl_diff_body = "pub fn f() -> type_macro!(\"a\nb\") {\n    different_private_impl()\n}";

    let surf_nl = normalized_public_surface_str("test.rs", fn_nl);
    let surf_sp = normalized_public_surface_str("test.rs", fn_sp);
    let surf_nl_diff_body = normalized_public_surface_str("test.rs", fn_nl_diff_body);

    assert_ne!(
        surf_nl, surf_sp,
        "multiline literal with newline in signature macro must produce different surface from space"
    );
    assert_eq!(
        surf_nl, surf_nl_diff_body,
        "private body change must not change surface"
    );
    assert!(
        !surf_nl.contains("private_impl"),
        "private body must not be captured: {surf_nl}"
    );
}

#[test]
fn public_api_guard_handles_multiline_qualifiers_on_functions() {
    let src_multiline_qual = r#"
pub unsafe extern
"C" fn f() {
    private_a()
}
"#;
    let src_diff_body = r#"
pub unsafe extern
"C" fn f() {
    private_b()
}
"#;
    let src_diff_qual = r#"
pub unsafe extern
"system" fn f() {
    private_a()
}
"#;
    let surf_base = normalized_public_surface_str("test.rs", src_multiline_qual);
    let surf_diff_body = normalized_public_surface_str("test.rs", src_diff_body);
    let surf_diff_qual = normalized_public_surface_str("test.rs", src_diff_qual);

    assert_eq!(
        surf_base, surf_diff_body,
        "private body changes in functions with multiline qualifiers must not alter surface: {surf_base} vs {surf_diff_body}"
    );
    assert_ne!(
        surf_base, surf_diff_qual,
        "qualifier changes across lines must alter public surface"
    );
    assert!(
        !surf_base.contains("private_a"),
        "private body must not be captured in multiline qualifier function: {surf_base}"
    );
    assert!(
        surf_base.contains("pub unsafe extern \"C\" fn f() {"),
        "signature must be captured properly: {surf_base}"
    );
}

#[test]
fn public_api_guard_distinguishes_braced_macros_from_function_and_item_bodies() {
    // 1. Braced macro in return type: payload change must alter surface; private body change must not
    let fn_braced_u32 = "pub fn f() -> type_macro! { u32 } {\n    private_a()\n}";
    let fn_braced_u64 = "pub fn f() -> type_macro! { u64 } {\n    private_a()\n}";
    let fn_braced_diff_body = "pub fn f() -> type_macro! { u32 } {\n    private_b()\n}";

    let surf_u32 = normalized_public_surface_str("test.rs", fn_braced_u32);
    let surf_u64 = normalized_public_surface_str("test.rs", fn_braced_u64);
    let surf_diff_body = normalized_public_surface_str("test.rs", fn_braced_diff_body);

    assert_ne!(
        surf_u32, surf_u64,
        "payload difference in braced macro must produce different public surface"
    );
    assert_eq!(
        surf_u32, surf_diff_body,
        "private body change after braced return type macro must not alter public surface"
    );
    assert!(
        !surf_u32.contains("private_a") && !surf_u32.contains("private_b"),
        "private body must not leak into surface: {surf_u32}"
    );
    assert!(
        surf_u32.contains("pub fn f() -> type_macro! { u32 } {"),
        "complete macro invocation and function header must be captured: {surf_u32}"
    );

    // 2. Path-qualified braced macro
    let fn_path_u32 = "pub fn f() -> foo::bar! { u32 } {\n    private_a()\n}";
    let fn_path_u64 = "pub fn f() -> foo::bar! { u64 } {\n    private_a()\n}";
    let surf_path_u32 = normalized_public_surface_str("test.rs", fn_path_u32);
    let surf_path_u64 = normalized_public_surface_str("test.rs", fn_path_u64);
    assert_ne!(
        surf_path_u32, surf_path_u64,
        "path-qualified braced macro payload difference must alter public surface"
    );
    assert!(
        !surf_path_u32.contains("private_a"),
        "private body must not leak: {surf_path_u32}"
    );

    // 3. Parenthesized macro form
    let fn_paren_u32 = "pub fn f() -> type_macro!(u32) {\n    private_a()\n}";
    let fn_paren_u64 = "pub fn f() -> type_macro!(u64) {\n    private_a()\n}";
    let fn_paren_diff_body = "pub fn f() -> type_macro!(u32) {\n    private_b()\n}";
    let surf_paren_u32 = normalized_public_surface_str("test.rs", fn_paren_u32);
    let surf_paren_u64 = normalized_public_surface_str("test.rs", fn_paren_u64);
    let surf_paren_diff_body = normalized_public_surface_str("test.rs", fn_paren_diff_body);
    assert_ne!(surf_paren_u32, surf_paren_u64);
    assert_eq!(surf_paren_u32, surf_paren_diff_body);
    assert!(!surf_paren_u32.contains("private_a"));

    // 4. Bracket-delimited macro form
    let fn_bracket_u32 = "pub fn f() -> type_macro![u32] {\n    private_a()\n}";
    let fn_bracket_u64 = "pub fn f() -> type_macro![u64] {\n    private_a()\n}";
    let surf_bracket_u32 = normalized_public_surface_str("test.rs", fn_bracket_u32);
    let surf_bracket_u64 = normalized_public_surface_str("test.rs", fn_bracket_u64);
    assert_ne!(surf_bracket_u32, surf_bracket_u64);
    assert!(!surf_bracket_u32.contains("private_a"));

    // 5. Existing plain function body detection
    let fn_plain = "pub fn f() -> u32 {\n    private_a()\n}";
    let surf_plain = normalized_public_surface_str("test.rs", fn_plain);
    assert!(surf_plain.contains("pub fn f() -> u32 {"));
    assert!(!surf_plain.contains("private_a"));

    // 6. Multiline qualifier + braced macro
    let fn_multi_qual_u32 =
        "pub unsafe extern\n\"C\" fn f() -> type_macro! { u32 } {\n    private_a()\n}";
    let fn_multi_qual_u64 =
        "pub unsafe extern\n\"C\" fn f() -> type_macro! { u64 } {\n    private_a()\n}";
    let fn_multi_qual_diff_qual =
        "pub unsafe extern\n\"system\" fn f() -> type_macro! { u32 } {\n    private_a()\n}";
    let fn_multi_qual_diff_body =
        "pub unsafe extern\n\"C\" fn f() -> type_macro! { u32 } {\n    private_b()\n}";

    let surf_mq_u32 = normalized_public_surface_str("test.rs", fn_multi_qual_u32);
    let surf_mq_u64 = normalized_public_surface_str("test.rs", fn_multi_qual_u64);
    let surf_mq_diff_qual = normalized_public_surface_str("test.rs", fn_multi_qual_diff_qual);
    let surf_mq_diff_body = normalized_public_surface_str("test.rs", fn_multi_qual_diff_body);

    assert_ne!(
        surf_mq_u32, surf_mq_u64,
        "payload change must alter surface"
    );
    assert_ne!(
        surf_mq_u32, surf_mq_diff_qual,
        "qualifier change must alter surface"
    );
    assert_eq!(
        surf_mq_u32, surf_mq_diff_body,
        "private body change must not alter surface"
    );
    assert!(!surf_mq_u32.contains("private_a"));

    // 7. Sibling paths: const, type alias, struct where clause
    let const_macro_u32 = "pub const X: type_macro! { u32 } = 10;";
    let const_macro_u64 = "pub const X: type_macro! { u64 } = 10;";
    let surf_c_u32 = normalized_public_surface_str("test.rs", const_macro_u32);
    let surf_c_u64 = normalized_public_surface_str("test.rs", const_macro_u64);
    assert_ne!(surf_c_u32, surf_c_u64);

    let type_macro_u32 = "pub type T = type_macro! { u32 };";
    let type_macro_u64 = "pub type T = type_macro! { u64 };";
    let surf_t_u32 = normalized_public_surface_str("test.rs", type_macro_u32);
    let surf_t_u64 = normalized_public_surface_str("test.rs", type_macro_u64);
    assert_ne!(surf_t_u32, surf_t_u64);

    let struct_where_macro =
        "pub struct Foo<T> where T: type_macro! { u32 } {\n    private_field: u32,\n}";
    let surf_s = normalized_public_surface_str("test.rs", struct_where_macro);
    assert!(surf_s.contains("pub struct Foo<T> where T: type_macro! { u32 } {"));
    assert!(!surf_s.contains("private_field"));
}

#[test]
fn public_api_guard_handles_unicode_macro_identifiers() {
    let fn_unicode_u32 = "pub fn f() -> Москва! { u32 } {\n    private_a()\n}";
    let fn_unicode_u64 = "pub fn f() -> Москва! { u64 } {\n    private_a()\n}";
    let fn_unicode_diff_body = "pub fn f() -> Москва! { u32 } {\n    private_b()\n}";

    let surf_u32 = normalized_public_surface_str("test.rs", fn_unicode_u32);
    let surf_u64 = normalized_public_surface_str("test.rs", fn_unicode_u64);
    let surf_diff_body = normalized_public_surface_str("test.rs", fn_unicode_diff_body);

    assert_ne!(
        surf_u32, surf_u64,
        "Unicode macro payload change must alter public surface: {surf_u32} vs {surf_u64}"
    );
    assert_eq!(
        surf_u32, surf_diff_body,
        "private body change after Unicode macro must not alter public surface"
    );
    assert!(
        !surf_u32.contains("private_a"),
        "private body must not leak into public surface: {surf_u32}"
    );
    assert!(
        surf_u32.contains("pub fn f() -> Москва! { u32 } {"),
        "Unicode macro and function header must be captured intact: {surf_u32}"
    );

    // Path-qualified Unicode macro
    let fn_path_u32 = "pub fn f() -> модуль::Макрос! { u32 } {\n    private_a()\n}";
    let fn_path_u64 = "pub fn f() -> модуль::Макрос! { u64 } {\n    private_a()\n}";
    let surf_path_u32 = normalized_public_surface_str("test.rs", fn_path_u32);
    let surf_path_u64 = normalized_public_surface_str("test.rs", fn_path_u64);
    assert_ne!(
        surf_path_u32, surf_path_u64,
        "path-qualified Unicode macro payload change must alter public surface"
    );
    assert!(
        !surf_path_u32.contains("private_a"),
        "private body must not leak: {surf_path_u32}"
    );
}

#[test]
fn public_api_guard_handles_trivia_separated_outer_attributes() {
    let canonical = "#[deprecated(note = \"old\")]\npub fn api() {}";
    let trivia_attr = "# /* comment */ [deprecated(note = \"old\")]\npub fn api() {}";
    let trivia_attr_new = "# /* comment */ [deprecated(note = \"new\")]\npub fn api() {}";

    let surf_canon = normalized_public_surface_str("test.rs", canonical);
    let surf_trivia = normalized_public_surface_str("test.rs", trivia_attr);
    let surf_trivia_new = normalized_public_surface_str("test.rs", trivia_attr_new);

    assert_eq!(
        surf_canon, surf_trivia,
        "trivia-separated attribute must normalize to same public contract as canonical attribute"
    );
    assert_ne!(
        surf_trivia, surf_trivia_new,
        "mutating note in trivia-separated attribute must alter public surface"
    );

    // Multiline cfg_attr with trivia
    let cfg_old = "# /*x*/ [cfg_attr(\n    feature = \"x\",\n    deprecated(note = \"old\")\n)]\npub fn api() {}";
    let cfg_new_feat = "# /*x*/ [cfg_attr(\n    feature = \"y\",\n    deprecated(note = \"old\")\n)]\npub fn api() {}";
    let cfg_new_note = "# /*x*/ [cfg_attr(\n    feature = \"x\",\n    deprecated(note = \"new\")\n)]\npub fn api() {}";

    let surf_cfg_old = normalized_public_surface_str("test.rs", cfg_old);
    let surf_cfg_new_feat = normalized_public_surface_str("test.rs", cfg_new_feat);
    let surf_cfg_new_note = normalized_public_surface_str("test.rs", cfg_new_note);

    assert_ne!(
        surf_cfg_old, surf_cfg_new_feat,
        "changing feature in cfg_attr must alter surface"
    );
    assert_ne!(
        surf_cfg_old, surf_cfg_new_note,
        "changing note in cfg_attr must alter surface"
    );

    // Enum variant attribute with trivia
    let enum_old = "pub enum E {\n    # /*x*/ [deprecated(note = \"old\")]\n    A,\n}";
    let enum_new = "pub enum E {\n    # /*x*/ [deprecated(note = \"new\")]\n    A,\n}";
    let enum_canon = "pub enum E {\n    #[deprecated(note = \"old\")]\n    A,\n}";

    let surf_e_old = normalized_public_surface_str("test.rs", enum_old);
    let surf_e_new = normalized_public_surface_str("test.rs", enum_new);
    let surf_e_canon = normalized_public_surface_str("test.rs", enum_canon);

    assert_eq!(
        surf_e_old, surf_e_canon,
        "enum variant trivia-separated attribute must match canonical"
    );
    assert_ne!(
        surf_e_old, surf_e_new,
        "mutating enum variant attribute note must alter surface"
    );
}

#[test]
fn public_api_guard_handles_trivia_separated_restricted_visibility() {
    let const_trivia_rev1 = "pub /*x*/ (crate) const HEADER: Spec = Spec {\n    rev: 1,\n};";
    let const_trivia_rev2 = "pub /*x*/ (crate) const HEADER: Spec = Spec {\n    rev: 2,\n};";
    let const_canon_rev1 = "pub(crate) const HEADER: Spec = Spec {\n    rev: 1,\n};";

    let surf_rev1 = normalized_public_surface_str("test.rs", const_trivia_rev1);
    let surf_rev2 = normalized_public_surface_str("test.rs", const_trivia_rev2);
    let surf_canon1 = normalized_public_surface_str("test.rs", const_canon_rev1);

    assert_eq!(
        surf_rev1, surf_canon1,
        "trivia-separated pub(crate) must normalize equal to canonical"
    );
    assert_ne!(
        surf_rev1, surf_rev2,
        "mutating multiline struct initializer in pub(crate) const must alter surface"
    );
    assert!(
        surf_rev1.contains("rev: 1"),
        "complete struct initializer must be captured in const: {surf_rev1}"
    );

    // pub(super) and pub(in path)
    let static_super_old = "pub /*x*/ ( super ) static GLOBAL: u32 = 100;";
    let static_super_new = "pub /*x*/ ( super ) static GLOBAL: u32 = 101;";
    let surf_super_old = normalized_public_surface_str("test.rs", static_super_old);
    let surf_super_new = normalized_public_surface_str("test.rs", static_super_new);
    assert_ne!(surf_super_old, surf_super_new);

    let static_in_path_old = "pub /*x*/ ( in crate::module ) static GLOBAL: u32 = 100;";
    let static_in_path_new = "pub /*x*/ ( in crate::module ) static GLOBAL: u32 = 101;";
    let surf_in_old = normalized_public_surface_str("test.rs", static_in_path_old);
    let surf_in_new = normalized_public_surface_str("test.rs", static_in_path_new);
    assert_ne!(surf_in_old, surf_in_new);
}

#[test]
fn public_api_guard_handles_mixed_width_unicode_macro_identifiers() {
    let fn_mixed_32 = "pub fn f() -> éx! { u32 } {\n    private_a()\n}";
    let fn_mixed_64 = "pub fn f() -> éx! { u64 } {\n    private_a()\n}";
    let fn_mixed_diff_body = "pub fn f() -> éx! { u32 } {\n    private_b()\n}";

    let surf_mixed_32 = normalized_public_surface_str("test.rs", fn_mixed_32);
    let surf_mixed_64 = normalized_public_surface_str("test.rs", fn_mixed_64);
    let surf_mixed_diff_body = normalized_public_surface_str("test.rs", fn_mixed_diff_body);

    assert_ne!(
        surf_mixed_32, surf_mixed_64,
        "payload mutation inside mixed-width Unicode macro must alter surface"
    );
    assert_eq!(
        surf_mixed_32, surf_mixed_diff_body,
        "private body change with mixed-width Unicode macro must not alter surface"
    );
    assert!(
        !surf_mixed_32.contains("private_a"),
        "private body must not be captured: {surf_mixed_32}"
    );

    // Other multi-byte / mixed-width shapes
    let fn_greek_32 = "pub fn f() -> alpha_β_gamma! { u32 } {\n    private_a()\n}";
    let fn_greek_64 = "pub fn f() -> alpha_β_gamma! { u64 } {\n    private_a()\n}";
    assert_ne!(
        normalized_public_surface_str("test.rs", fn_greek_32),
        normalized_public_surface_str("test.rs", fn_greek_64)
    );

    let fn_cjk_32 = "pub fn f() -> 名前_macro! { u32 } {\n    private_a()\n}";
    let fn_cjk_64 = "pub fn f() -> 名前_macro! { u64 } {\n    private_a()\n}";
    assert_ne!(
        normalized_public_surface_str("test.rs", fn_cjk_32),
        normalized_public_surface_str("test.rs", fn_cjk_64)
    );
}

#[test]
fn public_api_guard_handles_braced_public_items_and_member_visibility() {
    // 1. Single-line struct
    let struct_single_32 = "pub struct S { pub x: u32 }";
    let struct_single_64 = "pub struct S { pub x: u64 }";
    assert_ne!(
        normalized_public_surface_str("test.rs", struct_single_32),
        normalized_public_surface_str("test.rs", struct_single_64),
        "public struct field type change in single-line struct must alter surface"
    );

    let struct_private_32 = "pub struct S { private: u32, pub x: bool }";
    let struct_private_64 = "pub struct S { private: u64, pub x: bool }";
    assert_eq!(
        normalized_public_surface_str("test.rs", struct_private_32),
        normalized_public_surface_str("test.rs", struct_private_64),
        "private struct field type change in single-line struct must NOT alter surface"
    );

    // 2. Multiline struct
    let struct_multi_32 = "pub struct S {\n    pub x: u32,\n    private: InternalA,\n}";
    let struct_multi_64 = "pub struct S {\n    pub x: u64,\n    private: InternalA,\n}";
    let struct_multi_diff_priv = "pub struct S {\n    pub x: u32,\n    private: InternalB,\n}";
    assert_ne!(
        normalized_public_surface_str("test.rs", struct_multi_32),
        normalized_public_surface_str("test.rs", struct_multi_64),
        "public struct field type change in multiline struct must alter surface"
    );
    assert_eq!(
        normalized_public_surface_str("test.rs", struct_multi_32),
        normalized_public_surface_str("test.rs", struct_multi_diff_priv),
        "private struct field type change in multiline struct must NOT alter surface"
    );

    // 3. Single-line and multiline trait
    let trait_req_32 = "pub trait T { fn f() -> u32; }";
    let trait_req_64 = "pub trait T { fn f() -> u64; }";
    assert_ne!(
        normalized_public_surface_str("test.rs", trait_req_32),
        normalized_public_surface_str("test.rs", trait_req_64),
        "required trait method return type change must alter surface"
    );

    let trait_def_body_a = "pub trait T { fn f() -> u32 { private_a() } }";
    let trait_def_body_b = "pub trait T { fn f() -> u32 { private_b() } }";
    let trait_def_sig_64 = "pub trait T { fn f() -> u64 { private_a() } }";
    assert_eq!(
        normalized_public_surface_str("test.rs", trait_def_body_a),
        normalized_public_surface_str("test.rs", trait_def_body_b),
        "trait default method private implementation change must NOT alter surface"
    );
    assert_ne!(
        normalized_public_surface_str("test.rs", trait_def_body_a),
        normalized_public_surface_str("test.rs", trait_def_sig_64),
        "trait default method signature change must alter surface"
    );

    // 4. Trait associated items
    let trait_assoc_1 =
        "pub trait T {\n    type Item;\n    const N: usize;\n    fn req(&self) -> Self::Item;\n}";
    let trait_assoc_2 = "pub trait T {\n    type Item: Display;\n    const N: usize;\n    fn req(&self) -> Self::Item;\n}";
    let trait_assoc_3 =
        "pub trait T {\n    type Item;\n    const N: u32;\n    fn req(&self) -> Self::Item;\n}";
    assert_ne!(
        normalized_public_surface_str("test.rs", trait_assoc_1),
        normalized_public_surface_str("test.rs", trait_assoc_2),
        "trait associated type bound change must alter surface"
    );
    assert_ne!(
        normalized_public_surface_str("test.rs", trait_assoc_1),
        normalized_public_surface_str("test.rs", trait_assoc_3),
        "trait associated const type change must alter surface"
    );

    // 5. Sibling union item
    let union_pub_32 = "pub union U { pub a: u32, private: u32 }";
    let union_pub_64 = "pub union U { pub a: u64, private: u32 }";
    let union_priv_diff = "pub union U { pub a: u32, private: u64 }";
    assert_ne!(
        normalized_public_surface_str("test.rs", union_pub_32),
        normalized_public_surface_str("test.rs", union_pub_64),
        "public union field type change must alter surface"
    );
    assert_eq!(
        normalized_public_surface_str("test.rs", union_pub_32),
        normalized_public_surface_str("test.rs", union_priv_diff),
        "private union field type change must NOT alter surface"
    );
}

#[test]
fn public_api_guard_handles_escaped_newline_string_continuations() {
    let src_a = "pub const S: &str = \"a\\\n    b\";";
    let src_b = "pub const S: &str = \"a\\\n        b\";";
    let src_c = "pub const S: &str = \"a\\\n    c\";";

    assert!(src_a.contains("\\\n    "));
    assert!(src_b.contains("\\\n        "));
    assert!(src_c.contains("\\\n    "));

    let surf_a = normalized_public_surface_str("test.rs", src_a);
    let surf_b = normalized_public_surface_str("test.rs", src_b);
    let surf_c = normalized_public_surface_str("test.rs", src_c);

    assert_eq!(
        surf_a, surf_b,
        "indentation-only changes after escaped newline must produce identical public surfaces"
    );
    assert_ne!(
        surf_a, surf_c,
        "semantic payload change after escaped newline must alter public surface"
    );

    // Actual literal newline vs space must remain distinct
    let src_lit_nl = "pub const S: &str = \"a\nb\";";
    let src_lit_sp = "pub const S: &str = \"a b\";";
    assert_ne!(
        normalized_public_surface_str("test.rs", src_lit_nl),
        normalized_public_surface_str("test.rs", src_lit_sp),
        "actual literal newline must remain distinct from space"
    );

    // Raw strings must NOT normalize escaped newlines
    let src_raw_a = "pub const R: &str = r#\"a\\\n    b\"#;";
    let src_raw_b = "pub const R: &str = r#\"a\\\n        b\"#;";
    assert_ne!(
        normalized_public_surface_str("test.rs", src_raw_a),
        normalized_public_surface_str("test.rs", src_raw_b),
        "raw string indentation change must alter surface"
    );

    // Byte string continuation
    let src_byte_a = "pub const B: &[u8] = b\"a\\\n    b\";";
    let src_byte_b = "pub const B: &[u8] = b\"a\\\n        b\";";
    assert_eq!(
        normalized_public_surface_str("test.rs", src_byte_a),
        normalized_public_surface_str("test.rs", src_byte_b),
        "byte string escaped newline continuation must normalize indentation"
    );
}

#[test]
fn public_api_guard_handles_wrapped_struct_fields_p2_1() {
    // Regression A: Wrapped type
    let struct_wrapped_32 = "pub struct S {\n    pub value:\n        Vec<u32>,\n}";
    let struct_wrapped_64 = "pub struct S {\n    pub value:\n        Vec<u64>,\n}";
    let surf_w32 = normalized_public_surface_str("test.rs", struct_wrapped_32);
    let surf_w64 = normalized_public_surface_str("test.rs", struct_wrapped_64);
    assert_ne!(
        surf_w32, surf_w64,
        "wrapped struct field type change must alter surface"
    );

    // Regression B: Formatting-only reflow
    let struct_oneline = "pub struct S {\n    pub value: Vec<u32>,\n}";
    let surf_oneline = normalized_public_surface_str("test.rs", struct_oneline);
    assert_eq!(
        surf_w32, surf_oneline,
        "wrapped struct field must match one-line struct field surface"
    );

    // Regression C: Private wrapped field
    let struct_priv_w32 =
        "pub struct S {\n    private:\n        Vec<u32>,\n    pub visible: bool,\n}";
    let struct_priv_w64 =
        "pub struct S {\n    private:\n        Vec<u64>,\n    pub visible: bool,\n}";
    assert_eq!(
        normalized_public_surface_str("test.rs", struct_priv_w32),
        normalized_public_surface_str("test.rs", struct_priv_w64),
        "private wrapped struct field change must NOT alter surface"
    );

    // Regression D: Public field after private field containing comparison syntax
    let struct_cmp_32 =
        "pub struct S {\n    private: [(); (1 < 2) as usize],\n    pub visible: u32,\n}";
    let struct_cmp_64 =
        "pub struct S {\n    private: [(); (1 < 2) as usize],\n    pub visible: u64,\n}";
    let struct_cmp_diff_priv =
        "pub struct S {\n    private: [(); (2 < 3) as usize],\n    pub visible: u32,\n}";
    assert_ne!(
        normalized_public_surface_str("test.rs", struct_cmp_32),
        normalized_public_surface_str("test.rs", struct_cmp_64),
        "public field type after private comparison field must alter surface"
    );
    assert_eq!(
        normalized_public_surface_str("test.rs", struct_cmp_32),
        normalized_public_surface_str("test.rs", struct_cmp_diff_priv),
        "private comparison field change must NOT alter surface"
    );

    // Regression E: Nested generic public field
    let struct_nested_32 =
        "pub struct S {\n    pub value:\n        Result<Vec<u32>, Option<[u8; 3]>>,\n}";
    let struct_nested_64 =
        "pub struct S {\n    pub value:\n        Result<Vec<u64>, Option<[u8; 3]>>,\n}";
    assert_ne!(
        normalized_public_surface_str("test.rs", struct_nested_32),
        normalized_public_surface_str("test.rs", struct_nested_64),
        "nested generic wrapped field type change must alter surface"
    );

    // Regression F: Restricted visibility
    let struct_restr_32 = "pub struct S {\n    pub(crate) value:\n        Vec<u32>,\n}";
    let struct_restr_64 = "pub struct S {\n    pub(crate) value:\n        Vec<u64>,\n}";
    assert_ne!(
        normalized_public_surface_str("test.rs", struct_restr_32),
        normalized_public_surface_str("test.rs", struct_restr_64),
        "restricted public wrapped field type change must alter surface"
    );
}

#[test]
fn public_api_guard_handles_inline_default_trait_bodies_p2_2() {
    // Regression A: Inline default method body isolation
    let trait_inline_a = "pub trait T {\n    fn f() -> u32 { private_a() }\n}";
    let trait_inline_b = "pub trait T {\n    fn f() -> u32 { private_b() }\n}";
    assert_eq!(
        normalized_public_surface_str("test.rs", trait_inline_a),
        normalized_public_surface_str("test.rs", trait_inline_b),
        "inline default method body change in multiline trait must NOT alter surface"
    );

    // Regression B: Inline default method signature mutation
    let trait_inline_sig_64 = "pub trait T {\n    fn f() -> u64 { private_a() }\n}";
    assert_ne!(
        normalized_public_surface_str("test.rs", trait_inline_a),
        normalized_public_surface_str("test.rs", trait_inline_sig_64),
        "inline default method signature mutation must alter surface"
    );

    // Regression C: Required method
    let trait_req_32 = "pub trait T {\n    fn f() -> u32;\n}";
    let trait_req_64 = "pub trait T {\n    fn f() -> u64;\n}";
    assert_ne!(
        normalized_public_surface_str("test.rs", trait_req_32),
        normalized_public_surface_str("test.rs", trait_req_64),
        "required method signature mutation must alter surface"
    );

    // Regression D: Multiline default method
    let trait_multi_def_32 = "pub trait T {\n    fn f(\n        &self,\n        x: Vec<u32>,\n    ) -> u32\n    {\n        private_a()\n    }\n}";
    let trait_multi_def_64 = "pub trait T {\n    fn f(\n        &self,\n        x: Vec<u64>,\n    ) -> u32\n    {\n        private_a()\n    }\n}";
    let trait_multi_def_diff_body = "pub trait T {\n    fn f(\n        &self,\n        x: Vec<u32>,\n    ) -> u32\n    {\n        private_b()\n    }\n}";
    assert_ne!(
        normalized_public_surface_str("test.rs", trait_multi_def_32),
        normalized_public_surface_str("test.rs", trait_multi_def_64),
        "multiline default method signature mutation must alter surface"
    );
    assert_eq!(
        normalized_public_surface_str("test.rs", trait_multi_def_32),
        normalized_public_surface_str("test.rs", trait_multi_def_diff_body),
        "multiline default method body mutation must NOT alter surface"
    );

    // Regression E: Default body containing nested braces
    let trait_nested_body_a = "pub trait T {\n    fn f() -> u32 {\n        if private_cond() {\n            private_a()\n        } else {\n            private_b()\n        }\n    }\n}";
    let trait_nested_body_b = "pub trait T {\n    fn f() -> u32 {\n        if private_cond() {\n            private_c()\n        } else {\n            private_d()\n        }\n    }\n}";
    assert_eq!(
        normalized_public_surface_str("test.rs", trait_nested_body_a),
        normalized_public_surface_str("test.rs", trait_nested_body_b),
        "nested default method body mutation must NOT alter surface"
    );

    // Regression F: Type-position braced macro in trait method signature
    let trait_macro_32 = "pub trait T {\n    fn f() -> type_macro! { u32 } { private_a() }\n}";
    let trait_macro_64 = "pub trait T {\n    fn f() -> type_macro! { u64 } { private_a() }\n}";
    let trait_macro_diff_body =
        "pub trait T {\n    fn f() -> type_macro! { u32 } { private_b() }\n}";
    assert_ne!(
        normalized_public_surface_str("test.rs", trait_macro_32),
        normalized_public_surface_str("test.rs", trait_macro_64),
        "type-position braced macro payload change in trait method must alter surface"
    );
    assert_eq!(
        normalized_public_surface_str("test.rs", trait_macro_32),
        normalized_public_surface_str("test.rs", trait_macro_diff_body),
        "private body change with braced macro in trait method must NOT alter surface"
    );
}

#[test]
fn public_api_guard_handles_digit_suffixed_generic_identifiers_p2_3() {
    // Regression A: Digit-suffixed type
    let fn_vec2_1 = "pub fn f() -> Vec2<{1}> {\n    private_a()\n}";
    let fn_vec2_2 = "pub fn f() -> Vec2<{2}> {\n    private_a()\n}";
    let fn_vec2_diff_body = "pub fn f() -> Vec2<{1}> {\n    private_b()\n}";
    assert_ne!(
        normalized_public_surface_str("test.rs", fn_vec2_1),
        normalized_public_surface_str("test.rs", fn_vec2_2),
        "const generic arg change in Vec2<{{1}}> must alter surface"
    );
    assert_eq!(
        normalized_public_surface_str("test.rs", fn_vec2_1),
        normalized_public_surface_str("test.rs", fn_vec2_diff_body),
        "private body change with Vec2<{{1}}> must NOT alter surface"
    );

    // Regression B: Path-qualified digit-suffixed type
    let fn_math_vec2_1 = "pub fn f() -> math::Vec2<{1}> {\n    private_a()\n}";
    let fn_math_vec2_2 = "pub fn f() -> math::Vec2<{2}> {\n    private_a()\n}";
    assert_ne!(
        normalized_public_surface_str("test.rs", fn_math_vec2_1),
        normalized_public_surface_str("test.rs", fn_math_vec2_2),
        "const generic arg change in math::Vec2<{{1}}> must alter surface"
    );

    // Regression C: Nested generic
    let fn_opt_vec2_1 = "pub fn f() -> Option<Vec2<{1}>> {\n    private_a()\n}";
    let fn_opt_vec2_2 = "pub fn f() -> Option<Vec2<{2}>> {\n    private_a()\n}";
    assert_ne!(
        normalized_public_surface_str("test.rs", fn_opt_vec2_1),
        normalized_public_surface_str("test.rs", fn_opt_vec2_2),
        "nested generic arg change in Option<Vec2<{{1}}>> must alter surface"
    );

    // Regression D: Existing comparison
    let const_less_1 = "pub const LESS: bool = 1 < 2;\npub fn next() {}";
    let const_less_2 = "pub const LESS: bool = 1 < 3;\npub fn next() {}";
    assert_ne!(
        normalized_public_surface_str("test.rs", const_less_1),
        normalized_public_surface_str("test.rs", const_less_2)
    );

    // Regression E: Identifier expression comparison
    let const_ident_cmp_1 = "pub const LESS: bool = A < B;\npub fn next() {}";
    let const_ident_cmp_2 = "pub const LESS: bool = A < C;\npub fn next() {}";
    assert_ne!(
        normalized_public_surface_str("test.rs", const_ident_cmp_1),
        normalized_public_surface_str("test.rs", const_ident_cmp_2)
    );

    // Regression F: Const-generic expression containing comparison
    let fn_const_expr_cmp_1 =
        "pub fn f() -> Foo<{ if 1 < 2 { 32 } else { 64 } }> {\n    private_a()\n}";
    let fn_const_expr_cmp_2 =
        "pub fn f() -> Foo<{ if 1 < 2 { 32 } else { 128 } }> {\n    private_a()\n}";
    let fn_const_expr_cmp_diff_body =
        "pub fn f() -> Foo<{ if 1 < 2 { 32 } else { 64 } }> {\n    private_b()\n}";
    assert_ne!(
        normalized_public_surface_str("test.rs", fn_const_expr_cmp_1),
        normalized_public_surface_str("test.rs", fn_const_expr_cmp_2)
    );
    assert_eq!(
        normalized_public_surface_str("test.rs", fn_const_expr_cmp_1),
        normalized_public_surface_str("test.rs", fn_const_expr_cmp_diff_body)
    );
}
