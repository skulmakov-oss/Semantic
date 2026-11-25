//! macOS: OS-owned path aliases (`/tmp`, `/var` → `/private/*`) must not make
//! ordinary paths unusable, while user-created symlinks anywhere below them
//! stay rejected by package-path admission.
#![cfg(target_os = "macos")]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static COUNTER: AtomicUsize = AtomicUsize::new(0);

const HELLO: &str = "fn main() { let x: i32 = 40; assert(x + 2 == 42); return; }\n";

fn smc(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_smc"))
        .args(args)
        .output()
        .expect("run smc")
}

/// A fresh directory spelled through the given alias root (e.g. `/tmp`).
fn fresh(base: &Path, label: &str) -> PathBuf {
    let dir = base.join(format!(
        "smc_os_alias_{label}_{}_{}",
        std::process::id(),
        COUNTER.fetch_add(1, Ordering::SeqCst)
    ));
    std::fs::create_dir_all(&dir).expect("mkdir");
    dir
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8_lossy(&output.stderr).into_owned()
}

#[test]
fn run_and_compile_under_tmp_alias_succeed() {
    let dir = fresh(Path::new("/tmp"), "tmp");
    let src = dir.join("hello.sm");
    std::fs::write(&src, HELLO).unwrap();
    let run = smc(&["run", src.to_str().unwrap()]);
    assert!(run.status.success(), "{}", stderr(&run));
    let out = dir.join("hello.smc");
    let compile = smc(&[
        "compile",
        src.to_str().unwrap(),
        "-o",
        out.to_str().unwrap(),
    ]);
    assert!(compile.status.success(), "{}", stderr(&compile));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn run_under_var_alias_tmpdir_succeeds() {
    let tmp = std::env::temp_dir();
    let dir = fresh(&tmp, "tmpdir");
    let src = dir.join("hello.sm");
    std::fs::write(&src, HELLO).unwrap();
    let run = smc(&["run", src.to_str().unwrap()]);
    assert!(
        run.status.success(),
        "{} (TMPDIR {})",
        stderr(&run),
        tmp.display()
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn physical_private_spelling_keeps_working() {
    let dir = fresh(Path::new("/private/tmp"), "physical");
    let src = dir.join("hello.sm");
    std::fs::write(&src, HELLO).unwrap();
    let run = smc(&["run", src.to_str().unwrap()]);
    assert!(run.status.success(), "{}", stderr(&run));
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn user_symlink_below_alias_is_still_rejected() {
    use std::os::unix::fs::symlink;
    let dir = fresh(Path::new("/tmp"), "inner_link");
    std::fs::create_dir_all(dir.join("real")).unwrap();
    std::fs::write(dir.join("real/hello.sm"), HELLO).unwrap();
    symlink(dir.join("real"), dir.join("link")).unwrap();
    let through = dir.join("link/hello.sm");
    let run = smc(&["run", through.to_str().unwrap()]);
    assert!(!run.status.success(), "a user symlink must be rejected");
    let err = stderr(&run);
    assert!(err.contains("contains symlink or reparse point"), "{err}");
    assert!(err.contains("/link'"), "{err}");
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn user_symlink_escaping_from_alias_is_still_rejected() {
    use std::os::unix::fs::symlink;
    let outside = fresh(&std::env::temp_dir(), "escape_target");
    std::fs::write(outside.join("hello.sm"), HELLO).unwrap();
    let dir = fresh(Path::new("/tmp"), "escape");
    symlink(&outside, dir.join("escape")).unwrap();
    let through = dir.join("escape/hello.sm");
    let run = smc(&["run", through.to_str().unwrap()]);
    assert!(
        !run.status.success(),
        "an escaping symlink must be rejected"
    );
    assert!(
        stderr(&run).contains("contains symlink or reparse point"),
        "{}",
        stderr(&run)
    );
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&outside);
}

#[test]
fn symlinked_file_below_alias_is_still_rejected() {
    use std::os::unix::fs::symlink;
    let dir = fresh(Path::new("/tmp"), "leaf_link");
    std::fs::write(dir.join("real.sm"), HELLO).unwrap();
    symlink(dir.join("real.sm"), dir.join("hello.sm")).unwrap();
    let run = smc(&["run", dir.join("hello.sm").to_str().unwrap()]);
    assert!(
        !run.status.success(),
        "a symlinked source file must be rejected"
    );
    assert!(
        stderr(&run).contains("contains symlink or reparse point"),
        "{}",
        stderr(&run)
    );
    let _ = std::fs::remove_dir_all(&dir);
}
