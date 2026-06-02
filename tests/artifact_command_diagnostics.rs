use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Clone, Copy, Debug)]
enum ArtifactCommand {
    Verify,
    RunSmc,
    Disasm,
}

impl ArtifactCommand {
    fn as_str(self) -> &'static str {
        match self {
            ArtifactCommand::Verify => "verify",
            ArtifactCommand::RunSmc => "run-smc",
            ArtifactCommand::Disasm => "disasm",
        }
    }
}

#[derive(Clone, Copy, Debug)]
enum MalformedArtifactCase {
    MissingFile,
    EmptyFile,
    ShortHeader,
    RandomBytes,
}

impl MalformedArtifactCase {
    fn label(self) -> &'static str {
        match self {
            MalformedArtifactCase::MissingFile => "missing",
            MalformedArtifactCase::EmptyFile => "empty",
            MalformedArtifactCase::ShortHeader => "short",
            MalformedArtifactCase::RandomBytes => "random",
        }
    }

    fn write(self, dir: &Path) -> PathBuf {
        let path = dir.join(format!("{}.smc", self.label()));
        match self {
            MalformedArtifactCase::MissingFile => {}
            MalformedArtifactCase::EmptyFile => {
                std::fs::write(&path, []).expect("write empty artifact");
            }
            MalformedArtifactCase::ShortHeader => {
                std::fs::write(&path, b"SEMC").expect("write short artifact");
            }
            MalformedArtifactCase::RandomBytes => {
                std::fs::write(&path, b"not a semcode artifact").expect("write random artifact");
            }
        }
        path
    }
}

fn mk_temp_dir(prefix: &str) -> PathBuf {
    let dir = std::env::temp_dir()
        .join("semantic-tests")
        .join("artifact-command-diagnostics")
        .join(format!(
            "{}_{}_{}",
            prefix,
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("clock")
                .as_nanos()
        ));
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

fn write_valid_project_root(dir: &Path) {
    std::fs::create_dir_all(dir.join("src")).expect("mkdir src");
    std::fs::write(
        dir.join("semantic.toml"),
        r#"
[package]
name = "artifact_command_diagnostics"

[project]
entry = "src/main.sm"
"#,
    )
    .expect("write semantic.toml");
    std::fs::write(
        dir.join("src/main.sm"),
        r#"
fn main() {
    return;
}
"#,
    )
    .expect("write source");
}

fn run_smc(command: ArtifactCommand, input: &Path, cwd: &Path) -> Output {
    let exe = env!("CARGO_BIN_EXE_smc");
    Command::new(exe)
        .arg(command.as_str())
        .arg(input)
        .current_dir(cwd)
        .output()
        .expect("run smc")
}

fn collect_files(root: &Path) -> BTreeMap<String, Vec<u8>> {
    fn recurse(root: &Path, current: &Path, files: &mut BTreeMap<String, Vec<u8>>) {
        for entry in std::fs::read_dir(current).expect("read dir") {
            let entry = entry.expect("dir entry");
            let path = entry.path();
            if path.is_dir() {
                recurse(root, &path, files);
            } else if path.is_file() {
                let rel = path
                    .strip_prefix(root)
                    .expect("strip prefix")
                    .to_string_lossy()
                    .replace('\\', "/");
                files.insert(rel, std::fs::read(&path).expect("read file"));
            }
        }
    }

    let mut files = BTreeMap::new();
    recurse(root, root, &mut files);
    files
}

fn assert_malformed_input_rejected(command: ArtifactCommand, case: MalformedArtifactCase) {
    let dir = mk_temp_dir(&format!("{}_{}", command.as_str(), case.label()));
    write_valid_project_root(&dir);
    let input = case.write(&dir);
    let before = collect_files(&dir);

    let output = run_smc(command, &input, &dir);
    let after = collect_files(&dir);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "{command:?} unexpectedly passed for {case:?}"
    );
    assert!(
        stdout.is_empty(),
        "{command:?} produced stdout for {case:?}: {stdout}"
    );
    assert!(
        !stderr.is_empty(),
        "{command:?} produced empty stderr for {case:?}"
    );
    assert!(
        !stderr.contains("panicked at"),
        "{command:?} panicked for {case:?}:\n{stderr}"
    );
    assert_eq!(before, after, "{command:?} mutated files for {case:?}");

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn verify_rejects_malformed_semcode_inputs() {
    for case in [
        MalformedArtifactCase::MissingFile,
        MalformedArtifactCase::EmptyFile,
        MalformedArtifactCase::ShortHeader,
        MalformedArtifactCase::RandomBytes,
    ] {
        assert_malformed_input_rejected(ArtifactCommand::Verify, case);
    }
}

#[test]
fn run_smc_rejects_malformed_semcode_inputs() {
    for case in [
        MalformedArtifactCase::MissingFile,
        MalformedArtifactCase::EmptyFile,
        MalformedArtifactCase::ShortHeader,
        MalformedArtifactCase::RandomBytes,
    ] {
        assert_malformed_input_rejected(ArtifactCommand::RunSmc, case);
    }
}

#[test]
fn disasm_rejects_malformed_semcode_inputs() {
    for case in [
        MalformedArtifactCase::MissingFile,
        MalformedArtifactCase::EmptyFile,
        MalformedArtifactCase::ShortHeader,
        MalformedArtifactCase::RandomBytes,
    ] {
        assert_malformed_input_rejected(ArtifactCommand::Disasm, case);
    }
}
