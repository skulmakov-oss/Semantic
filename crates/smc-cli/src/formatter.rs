use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormatterMode {
    Write,
    Check,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormatterSummary {
    pub files_scanned: usize,
    pub files_changed: usize,
    pub changed_paths: Vec<PathBuf>,
}

pub fn format_path(path: &Path, mode: FormatterMode) -> Result<FormatterSummary, String> {
    let mut files = Vec::new();
    collect_semantic_files(path, &mut files)?;

    if files.is_empty() {
        return Err(format!(
            "no Semantic source files found under '{}'",
            path.display()
        ));
    }

    // SSF-09 #1580: every file is formatted (or refused) before anything is
    // written, so a refusal never leaves a tree half-formatted.
    let mut changed = Vec::new();
    let mut refused = Vec::new();
    for file in &files {
        let original = fs::read_to_string(file)
            .map_err(|e| format!("failed to read '{}': {}", file.display(), e))?;
        match format_source_checked(&original) {
            Ok(formatted) if formatted != original => changed.push((file.to_path_buf(), formatted)),
            Ok(_) => {}
            Err(refusal) => refused.push(format!("  {}: {}", file.display(), refusal)),
        }
    }
    if !refused.is_empty() {
        return Err(format!(
            "refusing to format {} file(s); the canonical formatter only applies changes proven not to alter the token stream:\n{}",
            refused.len(),
            refused.join("\n")
        ));
    }

    if mode == FormatterMode::Write {
        for (file, formatted) in &changed {
            fs::write(file, formatted.as_bytes())
                .map_err(|e| format!("failed to write '{}': {}", file.display(), e))?;
        }
    }
    let changed_paths: Vec<PathBuf> = changed.into_iter().map(|(file, _)| file).collect();

    Ok(FormatterSummary {
        files_scanned: files.len(),
        files_changed: changed_paths.len(),
        changed_paths,
    })
}

/// Why the canonical formatter declined to change a source.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormatRefusal {
    /// The input does not lex, so semantic preservation cannot be proven.
    SourceDoesNotLex,
    /// The whitespace normalization would change the token stream (for
    /// example a carriage return inside a string literal).
    TokenStreamWouldChange,
}

impl std::fmt::Display for FormatRefusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FormatRefusal::SourceDoesNotLex => {
                f.write_str("source does not lex; formatting would not be provably safe")
            }
            FormatRefusal::TokenStreamWouldChange => {
                f.write_str("formatting would change the token stream")
            }
        }
    }
}

/// SSF-09 #1580: the canonical formatter with its semantic-safety proof.
///
/// The formatter only normalizes line endings to `\n`, strips trailing
/// spaces/tabs, and ends a non-empty file with exactly one newline. A
/// changed result is returned only when the canonical lexer produces an
/// identical `(kind, text)` token sequence for the input and the output -
/// the same lexer every parser consumes - so formatting can never change
/// what the program means. Otherwise the input is refused, never
/// partially rewritten. The result is idempotent.
pub fn format_source_checked(input: &str) -> Result<String, FormatRefusal> {
    let formatted = normalize_whitespace(input);
    if formatted == input {
        return Ok(formatted);
    }
    let before = sm_front::lex(input).map_err(|_| FormatRefusal::SourceDoesNotLex)?;
    let after = sm_front::lex(&formatted).map_err(|_| FormatRefusal::TokenStreamWouldChange)?;
    if significant_tokens(&before) == significant_tokens(&after) {
        Ok(formatted)
    } else {
        Err(FormatRefusal::TokenStreamWouldChange)
    }
}

/// The `(kind, text)` token sequence with the end-of-file run of blank-line
/// newline tokens collapsed to one: dropping trailing blank lines is the only
/// newline-count change the formatter makes, and blank lines at end of file
/// carry no meaning for either grammar. Interior newlines are compared
/// exactly.
fn significant_tokens(tokens: &[sm_front::Token]) -> Vec<(sm_front::TokenKind, &str)> {
    use sm_front::TokenKind;
    let mut out: Vec<(TokenKind, &str)> =
        tokens.iter().map(|t| (t.kind, t.text.as_str())).collect();
    let dedents = out
        .iter()
        .rev()
        .take_while(|(kind, _)| *kind == TokenKind::Dedent)
        .count();
    let tail = out.split_off(out.len() - dedents);
    while out.len() >= 2
        && out[out.len() - 1].0 == TokenKind::Newline
        && out[out.len() - 2].0 == TokenKind::Newline
    {
        out.pop();
    }
    out.extend(tail);
    out
}

/// The canonical formatter as a total function: the formatted text when
/// [`format_source_checked`] accepts it, otherwise `input` unchanged.
pub fn format_source_text(input: &str) -> String {
    format_source_checked(input).unwrap_or_else(|_| input.to_string())
}

fn normalize_whitespace(input: &str) -> String {
    let normalized = input.replace("\r\n", "\n").replace('\r', "\n");
    let mut lines: Vec<String> = normalized
        .split('\n')
        .map(|line| trim_trailing_whitespace(line).to_string())
        .collect();

    while lines.last().is_some_and(|line| line.is_empty()) {
        lines.pop();
    }

    if lines.is_empty() {
        String::new()
    } else {
        format!("{}\n", lines.join("\n"))
    }
}

fn collect_semantic_files(path: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if path.is_file() {
        if is_semantic_source(path) {
            out.push(path.to_path_buf());
        }
        return Ok(());
    }

    let entries = fs::read_dir(path)
        .map_err(|e| format!("failed to read directory '{}': {}", path.display(), e))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("failed to read directory entry: {}", e))?;
        let entry_path = entry.path();
        if entry_path.is_dir() {
            if should_skip_dir(&entry_path) {
                continue;
            }
            collect_semantic_files(&entry_path, out)?;
        } else if is_semantic_source(&entry_path) {
            out.push(entry_path);
        }
    }

    out.sort();
    Ok(())
}

fn is_semantic_source(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .map(|value| value.eq_ignore_ascii_case("sm"))
        .unwrap_or(false)
}

fn should_skip_dir(path: &Path) -> bool {
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or_default();
    matches!(
        name,
        ".git" | "target" | "node_modules" | "dist" | ".semantic-cache"
    )
}

fn trim_trailing_whitespace(line: &str) -> &str {
    line.trim_end_matches([' ', '\t'])
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
        fs::create_dir_all(&base).expect("mkdir");
        base
    }

    #[test]
    fn format_source_text_normalizes_whitespace_and_final_newline() {
        let input = "fn main() {    \r\n    return;\t\r\n}\r\n\r\n";
        let got = format_source_text(input);
        assert_eq!(got, "fn main() {\n    return;\n}\n");
    }

    #[test]
    fn format_path_check_reports_changes_without_writing() {
        let dir = mk_temp_dir("smc_fmt_check");
        let file = dir.join("main.sm");
        fs::write(&file, "fn main() {  \n    return;\n}\n\n").expect("write");

        let summary = format_path(&file, FormatterMode::Check).expect("check");
        assert_eq!(summary.files_scanned, 1);
        assert_eq!(summary.files_changed, 1);
        assert_eq!(
            fs::read_to_string(&file).expect("read"),
            "fn main() {  \n    return;\n}\n\n"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn format_source_text_is_idempotent() {
        let input = "fn main() {  \r\n    return;\t\r\n}\r\n\r\n";
        let once = format_source_text(input);
        let twice = format_source_text(&once);
        assert_eq!(once, twice);
    }

    #[test]
    fn format_source_text_preserves_string_and_comment_contents() {
        let input = "fn main() {\n    // keep this comment intact\n    let s: text = \"  spaced text  \";\n    return;\n}\n";
        assert_eq!(format_source_text(input), input);
    }

    #[test]
    fn format_source_text_preserves_compact_guard_returns() {
        let input = "fn f(x: i32) -> i32 {\n    if x == 0 { return 1; }\n    return 0;\n}\n";
        assert_eq!(format_source_text(input), input);
    }

    #[test]
    fn format_source_text_does_not_collapse_multiline_control_flow() {
        let input =
            "fn f(x: i32) -> i32 {\n    if x == 0 {\n        return 1;\n    }\n    return 0;\n}\n";
        assert_eq!(format_source_text(input), input);
    }

    #[test]
    fn format_source_text_preserves_logos_significant_indentation() {
        let input = "Entity Sensor:\n    state val: quad\n    prop active: bool\n";
        assert_eq!(format_source_text(input), input);
    }

    #[test]
    fn format_path_write_is_idempotent_on_disk() {
        let dir = mk_temp_dir("smc_fmt_idempotent");
        let file = dir.join("main.sm");
        fs::write(&file, "fn main() {  \r\n    return;\n}\n\n").expect("write");

        format_path(&file, FormatterMode::Write).expect("first write");
        let after_first = fs::read_to_string(&file).expect("read after first");

        let summary = format_path(&file, FormatterMode::Write).expect("second write");
        let after_second = fs::read_to_string(&file).expect("read after second");

        assert_eq!(after_first, after_second);
        assert_eq!(
            summary.files_changed, 0,
            "second write pass should change nothing"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn formatter_refuses_carriage_return_inside_string_literal() {
        let input = "fn main() {\n    let s: text = \"a\rb\";\n    return;\n}\n";
        assert_eq!(
            format_source_checked(input),
            Err(FormatRefusal::TokenStreamWouldChange)
        );
        assert_eq!(format_source_text(input), input);
    }

    #[test]
    fn formatter_refuses_to_change_source_that_does_not_lex() {
        let input = "fn main() {  \n    let s: text = \"open   \n}\n";
        assert_eq!(
            format_source_checked(input),
            Err(FormatRefusal::SourceDoesNotLex)
        );
    }

    #[test]
    fn formatter_accepts_already_formatted_source_even_if_it_does_not_lex() {
        let input = "fn main( {\n";
        assert_eq!(format_source_checked(input), Ok(input.to_string()));
    }

    #[test]
    fn formatter_preserves_token_stream_on_every_accepted_change() {
        let inputs = [
            "fn main() {    \r\n    return;\t\r\n}\r\n\r\n",
            "Entity Sensor:  \n    state val: quad \n    prop active: bool\t\n\n",
            "fn f(x: i32) -> i32 {\n    // comment   \n    return x;\n}",
        ];
        for input in inputs {
            let formatted = format_source_checked(input).expect("accepted");
            let before = sm_front::lex(input).unwrap();
            let after = sm_front::lex(&formatted).unwrap();
            assert_eq!(
                significant_tokens(&before),
                significant_tokens(&after),
                "{input:?}"
            );
            assert_eq!(format_source_checked(&formatted), Ok(formatted.clone()));
        }
    }

    #[test]
    fn significant_tokens_keep_interior_blank_lines() {
        let a = sm_front::lex("fn main() {\n\n    return;\n}\n").unwrap();
        let b = sm_front::lex("fn main() {\n    return;\n}\n").unwrap();
        assert_ne!(significant_tokens(&a), significant_tokens(&b));
    }

    #[test]
    fn format_path_refusal_writes_nothing() {
        let dir = mk_temp_dir("smc_fmt_refusal");
        let good = dir.join("a.sm");
        let bad = dir.join("b.sm");
        fs::write(&good, "fn main() {  \n    return;\n}\n").expect("write good");
        fs::write(&bad, "fn main() {  \n    let s: text = \"a\rb\";\n}\n").expect("write bad");

        let err = format_path(&dir, FormatterMode::Write).expect_err("must refuse");
        assert!(err.contains("b.sm"), "{err}");
        assert_eq!(
            fs::read_to_string(&good).expect("read good"),
            "fn main() {  \n    return;\n}\n"
        );

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn collect_semantic_files_walks_in_deterministic_sorted_order() {
        let dir = mk_temp_dir("smc_fmt_dir_order");
        fs::write(dir.join("zeta.sm"), "fn main() { return; }\n").expect("write zeta");
        fs::write(dir.join("alpha.sm"), "fn main() { return; }\n").expect("write alpha");
        fs::write(dir.join("mid.sm"), "fn main() { return; }\n").expect("write mid");

        let mut files = Vec::new();
        collect_semantic_files(&dir, &mut files).expect("collect");
        let names: Vec<String> = files
            .iter()
            .map(|p| p.file_name().unwrap().to_string_lossy().to_string())
            .collect();

        assert_eq!(names, vec!["alpha.sm", "mid.sm", "zeta.sm"]);

        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn format_path_write_updates_recursive_tree() {
        let dir = mk_temp_dir("smc_fmt_write");
        let nested = dir.join("nested");
        fs::create_dir_all(&nested).expect("nested");
        let file = nested.join("main.sm");
        fs::write(&file, "fn main() {  \n    return;\n}\n\n").expect("write");
        fs::write(nested.join("note.txt"), "keep me").expect("write txt");

        let summary = format_path(&dir, FormatterMode::Write).expect("write");
        assert_eq!(summary.files_scanned, 1);
        assert_eq!(summary.files_changed, 1);
        assert_eq!(
            fs::read_to_string(&file).expect("read"),
            "fn main() {\n    return;\n}\n"
        );

        let _ = fs::remove_dir_all(&dir);
    }
}
