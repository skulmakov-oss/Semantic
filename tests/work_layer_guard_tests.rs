use std::fs;
use std::process::Command;

fn smc_output(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(args)
        .output()
        .expect("run smc")
}

#[test]
fn test_work_prove_canonical_intent() {
    let path = "tests/test_work_seed.sm";
    fs::write(path, "module main;").unwrap();

    let output = smc_output(&["work", path, "prove"]);
    let _ = fs::remove_file(path);

    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure the command was dispatched properly to the underlying pipeline
    // It should not fail at the intent parsing layer
    assert!(!stderr.contains("Unknown intent"));
    assert!(!stderr.contains("Unexpected token"));
}

#[test]
fn test_work_rejects_non_canonical_intent() {
    let output = smc_output(&["work", "main.sm", "build"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Ensure the guard boundary explicitly rejects non-canonical intent
    assert!(stderr.contains("Unknown intent 'build'"));
    assert!(stderr.contains("Did you mean 'work <subject> prove'"));
}

#[test]
fn test_work_requires_minimum_arguments() {
    let output = smc_output(&["work"]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("usage: smc work <subject> <intent>"));
}
