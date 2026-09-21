#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DiagLevel {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic<M> {
    pub level: DiagLevel,
    pub code: &'static str,
    pub message: M,
}

pub fn diagnostic_catalog() -> &'static [(&'static str, &'static str)] {
    &[
        (
            "E0000",
            "Generic frontend parse/type error. See caret span for exact location.",
        ),
        ("E0001", "Unexpected character in source input."),
        ("E0002", "Expected logical operator '&&'."),
        ("E0003", "Expected logical operator '||'."),
        ("E0004", "Unterminated string literal."),
        ("E0101", "Bad indentation level (INDENT/DEDENT mismatch)."),
        ("E0200", "Expected Logos declaration (System/Entity/Law)."),
        (
            "E0201",
            "Type mismatch. Example: expected QVec/Bool, found other type.",
        ),
        ("E0210", "Malformed Entity declaration header."),
        ("E0211", "Expected ':' after Entity name."),
        ("E0212", "Expected newline after Entity header."),
        ("E0213", "Expected INDENT for Entity body."),
        ("E0214", "Expected Entity field declaration."),
        ("E0215", "Entity field must start with 'state' or 'prop'."),
        ("E0216", "Expected ':' in Entity field declaration."),
        ("E0220", "Duplicate Entity declaration."),
        ("E0221", "Duplicate Law inside the same Entity scope."),
        ("E0222", "Law body is empty."),
        ("E0223", "Shadowing is forbidden inside a Law scope."),
        ("E0224", "Empty When condition."),
        ("E0225", "Empty When body/effect."),
        ("E0230", "Expected 'When' clause in Law body."),
        ("E0234", "Expected type annotation."),
        ("E0238", "Cyclic import detected."),
        ("E0239", "Import resolution/read/parse failure."),
        ("E0240", "Import re-export is not supported in v0.1."),
        ("E0241", "Duplicate import alias within one module."),
        ("E0242", "Public re-export collision."),
        ("E0243", "Symbol re-export cycle detected."),
        (
            "E0244",
            "Selected import symbol not found in the dependency exports.",
        ),
        (
            "E0245",
            "Invalid selected import: duplicate alias, wildcard/select conflict, or kind mismatch.",
        ),
        (
            "W0240",
            "Dead law branch detected: When condition is always false.",
        ),
        (
            "W0241",
            "Constant folding candidate detected for fx.* call with literals.",
        ),
        ("W0250", "Law name style warning (expected UpperCamelCase)."),
        ("W0251", "Large Law block warning (too many When clauses)."),
        (
            "W0252",
            "Unused Entity field warning (state/prop not referenced).",
        ),
        ("W0253", "Magic number warning (consider named constant)."),
    ]
}

#[cfg(feature = "alloc")]
pub fn render_single_line_caret(line: u32, col: u32, src_line: &str) -> alloc::string::String {
    let mut caret = alloc::string::String::new();
    for _ in 0..(col.saturating_sub(1) as usize) {
        caret.push(' ');
    }
    caret.push('^');
    alloc::format!("{line:>2} | {src_line}\n  | {caret}")
}

#[cfg(feature = "alloc")]
pub fn render_context_with_caret(
    source: &str,
    mark: crate::SourceMark,
    radius: usize,
) -> alloc::string::String {
    let line = mark.line.max(1) as usize;
    let col = mark.col.max(1) as usize;
    let lines: alloc::vec::Vec<&str> = source.lines().collect();
    let lo = line.saturating_sub(radius).max(1);
    let hi = (line + radius).min(lines.len().max(1));

    let mut body = alloc::string::String::new();
    for ln in lo..=hi {
        let text = lines.get(ln - 1).copied().unwrap_or("");
        body.push_str(&alloc::format!("{ln:>4} | {text}\n"));
        if ln == line {
            let mut underline = alloc::string::String::new();
            for _ in 0..col.saturating_sub(1) {
                underline.push(' ');
            }
            underline.push('^');
            body.push_str(&alloc::format!("     | {underline}\n"));
        }
    }
    body
}

#[cfg(feature = "alloc")]
pub fn format_parser_error_at_input(
    code: &str,
    msg: &str,
    line: u32,
    col: u32,
    src_line: &str,
) -> alloc::string::String {
    let snippet = render_single_line_caret(line, col, src_line);
    alloc::format!("error[{code}]: {msg}\n --> <input>:{line}:{col}\n  |\n{snippet}")
}

#[cfg(feature = "alloc")]
pub fn format_multiple_parser_errors(
    code: &str,
    messages: &[alloc::string::String],
) -> alloc::string::String {
    let mut out = alloc::format!(
        "error[{code}]: multiple parser errors ({}):",
        messages.len()
    );
    for (i, msg) in messages.iter().enumerate() {
        out.push_str(&alloc::format!("\n{}. {}", i + 1, msg));
    }
    out
}

#[cfg(feature = "alloc")]
pub fn append_help_line(body: &mut alloc::string::String, help: &str) {
    body.push_str(&alloc::format!("help: {help}\n"));
}

#[cfg(feature = "alloc")]
pub fn format_diagnostic_header(
    level: DiagLevel,
    code: &str,
    message: &str,
    line: u32,
    col: u32,
) -> alloc::string::String {
    let lvl = match level {
        DiagLevel::Error => "Error",
        DiagLevel::Warning => "Warning",
    };
    alloc::format!("{lvl} [{code}]: {message} at line {line}:{col}")
}

#[cfg(feature = "alloc")]
pub fn suggest_closest_case_insensitive<'a>(
    input: &str,
    candidates: &'a [&'a str],
    max_distance: usize,
) -> Option<&'a str> {
    let mut best: Option<(&str, usize)> = None;
    for c in candidates {
        let d = edit_distance_case_insensitive(input, c);
        if d <= max_distance {
            match best {
                Some((_, bd)) if d >= bd => {}
                _ => best = Some((c, d)),
            }
        }
    }
    best.map(|(s, _)| s)
}

#[cfg(feature = "alloc")]
pub fn edit_distance_case_insensitive(a: &str, b: &str) -> usize {
    let a = a.to_ascii_lowercase();
    let b = b.to_ascii_lowercase();
    edit_distance(&a, &b)
}

#[cfg(feature = "alloc")]
pub fn edit_distance(a: &str, b: &str) -> usize {
    let aa: alloc::vec::Vec<char> = a.chars().collect();
    let bb: alloc::vec::Vec<char> = b.chars().collect();
    let mut dp: alloc::vec::Vec<usize> = (0..=bb.len()).collect();
    for (i, ca) in aa.iter().enumerate() {
        let mut prev = dp[0];
        dp[0] = i + 1;
        for (j, cb) in bb.iter().enumerate() {
            let tmp = dp[j + 1];
            let cost = if ca == cb { 0 } else { 1 };
            dp[j + 1] = (dp[j + 1] + 1).min(dp[j] + 1).min(prev + cost);
            prev = tmp;
        }
    }
    dp[bb.len()]
}
