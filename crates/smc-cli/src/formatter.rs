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

    let mut changed_paths = Vec::new();

    for file in &files {
        let original = fs::read_to_string(file)
            .map_err(|e| format!("failed to read '{}': {}", file.display(), e))?;
        let formatted = format_source_text(&original);
        if formatted != original {
            if mode == FormatterMode::Write {
                fs::write(file, formatted.as_bytes())
                    .map_err(|e| format!("failed to write '{}': {}", file.display(), e))?;
            }
            changed_paths.push(file.to_path_buf());
        }
    }

    Ok(FormatterSummary {
        files_scanned: files.len(),
        files_changed: changed_paths.len(),
        changed_paths,
    })
}

pub fn format_source_text(input: &str) -> String {
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
