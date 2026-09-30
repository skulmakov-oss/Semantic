//! Behavioral regressions for the local admission-guard and release-asset
//! PowerShell helpers (Issue #1969, REM-022).
//!
//! Each test dot-sources a helper file into a fresh `pwsh` process and drives
//! it against throwaway fixtures: fake `smc` binaries that record their
//! invocations and fail on demand, temporary git repositories, and temporary
//! release-asset directories. The assertions are on observable outcomes
//! (exit status, which commands ran, which files survive), not on script text.
//!
//! `pwsh` is preinstalled on the hosted CI runners. When it is absent locally
//! the tests are skipped with a message; under `CI` its absence is a failure.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::atomic::{AtomicUsize, Ordering};

static NEXT_ID: AtomicUsize = AtomicUsize::new(0);

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn pwsh_available() -> bool {
    let available = Command::new("pwsh")
        .args(["-NoLogo", "-NoProfile", "-Command", "exit 0"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false);
    if !available {
        assert!(
            std::env::var_os("CI").is_none(),
            "pwsh is required for admission guard script regressions under CI"
        );
        eprintln!("skipping admission guard script regression: pwsh not found");
    }
    available
}

struct TempDir(PathBuf);

impl TempDir {
    fn new(label: &str) -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::SeqCst);
        let path = std::env::temp_dir().join(format!(
            "semantic_admission_guard_{label}_{}_{id}",
            std::process::id()
        ));
        if path.exists() {
            fs::remove_dir_all(&path).expect("remove stale temp dir");
        }
        fs::create_dir_all(&path).expect("create temp dir");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Runs `body` as a script file after dot-sourcing `scripts/<helper>`; `vars`
/// become `$env:` values. Script-file mode makes an uncaught throw end the
/// process with a non-zero exit status.
fn run_helper(helper: &str, body: &str, cwd: &Path, vars: &[(&str, &Path)]) -> Output {
    let helper_path = repo_root().join("scripts").join(helper);
    let script_dir = TempDir::new("script");
    let script_path = script_dir.path().join("case.ps1");
    fs::write(
        &script_path,
        format!(
            "$ErrorActionPreference = 'Stop'\nSet-StrictMode -Version Latest\n$PSStyle.OutputRendering = 'PlainText'\n. $env:SEMANTIC_HELPER\n{body}\n"
        ),
    )
    .expect("write pwsh case script");
    let mut command = Command::new("pwsh");
    command
        .args(["-NoLogo", "-NoProfile", "-NonInteractive", "-File"])
        .arg(&script_path)
        .current_dir(cwd)
        .env("SEMANTIC_HELPER", &helper_path)
        .env("NO_COLOR", "1")
        .env("TERM", "dumb");
    for (key, value) in vars {
        command.env(key, value);
    }
    command.output().expect("run pwsh")
}

fn text(output: &Output) -> String {
    format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    )
}

fn git(cwd: &Path, args: &[&str]) {
    let output = Command::new("git")
        .args([
            "-c",
            "user.name=Semantic Test",
            "-c",
            "user.email=semantic-test@example.invalid",
            "-c",
            "init.defaultBranch=main",
            "-c",
            "commit.gpgsign=false",
        ])
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run git");
    assert!(
        output.status.success(),
        "git {args:?} failed: {}",
        text(&output)
    );
}

fn git_stdout(cwd: &Path, args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("run git");
    assert!(output.status.success(), "git {args:?} failed");
    String::from_utf8_lossy(&output.stdout).trim().to_string()
}

/// A fake `smc` that appends each subcommand to `calls.log`, writes the
/// `compile -o` artifact, and exits 7 for the subcommand named in
/// `FAKE_SMC_FAIL`.
fn write_fake_smc(dir: &Path) -> PathBuf {
    let fake = dir.join("fake_smc.ps1");
    fs::write(
        &fake,
        r#"param([Parameter(ValueFromRemainingArguments = $true)] [string[]] $Rest)
$subcommand = $Rest[0]
Add-Content -LiteralPath $env:FAKE_SMC_LOG -Value $subcommand
if ($env:FAKE_SMC_FAIL -eq $subcommand) { exit 7 }
if ($subcommand -eq 'compile') { Set-Content -LiteralPath $Rest[3] -Value 'semcode' }
exit 0
"#,
    )
    .expect("write fake smc");
    fake
}

fn smoke(fail_on: Option<&str>) -> (Output, Vec<String>, bool) {
    let temp = TempDir::new("smoke");
    let fake = write_fake_smc(temp.path());
    let log = temp.path().join("calls.log");
    let fixture = temp.path().join("project_root");
    fs::create_dir_all(&fixture).unwrap();
    let smoke_dir = temp.path().join("smoke_out");
    let body = format!(
        "{}Invoke-SmcProjectRootSmoke -SmcBinary $env:FAKE_SMC -FixtureRoot $env:FIXTURE -TempDirectory $env:SMOKE_DIR -ArtifactName 'out.smc'\nWrite-Host 'SMOKE PASSED'",
        match fail_on {
            Some(step) => format!("$env:FAKE_SMC_FAIL = '{step}'\n"),
            None => String::new(),
        }
    );
    let output = run_helper(
        "admission_guard_lib.ps1",
        &body,
        temp.path(),
        &[
            ("FAKE_SMC", &fake),
            ("FAKE_SMC_LOG", &log),
            ("FIXTURE", &fixture),
            ("SMOKE_DIR", &smoke_dir),
        ],
    );
    let calls = fs::read_to_string(&log)
        .unwrap_or_default()
        .lines()
        .map(str::to_string)
        .collect();
    (output, calls, smoke_dir.exists())
}

#[test]
fn project_root_smoke_runs_all_five_commands_when_each_succeeds() {
    if !pwsh_available() {
        return;
    }
    let (output, calls, smoke_dir_left) = smoke(None);
    assert!(output.status.success(), "{}", text(&output));
    assert!(text(&output).contains("SMOKE PASSED"));
    assert_eq!(calls, ["check", "run", "compile", "verify", "run-smc"]);
    assert!(!smoke_dir_left, "smoke temp directory must be cleaned up");
}

// FND-184: a failing `smc check` must fail the step even though every later
// command would succeed and leave $LASTEXITCODE at 0.
#[test]
fn project_root_smoke_fails_on_check_even_when_later_commands_succeed() {
    if !pwsh_available() {
        return;
    }
    let (output, calls, smoke_dir_left) = smoke(Some("check"));
    assert!(!output.status.success(), "{}", text(&output));
    assert!(text(&output).contains("exit code 7: smc check"));
    assert!(!text(&output).contains("SMOKE PASSED"));
    assert_eq!(calls, ["check"], "the sequence must stop at the failure");
    assert!(!smoke_dir_left, "smoke temp directory must be cleaned up");
}

// FND-187: the same guarantee for `smc run` in the package-baseline smoke.
#[test]
fn project_root_smoke_fails_on_run_even_when_later_commands_succeed() {
    if !pwsh_available() {
        return;
    }
    let (output, calls, _) = smoke(Some("run"));
    assert!(!output.status.success(), "{}", text(&output));
    assert!(text(&output).contains("exit code 7: smc run"));
    assert_eq!(calls, ["check", "run"]);
}

#[test]
fn project_root_smoke_fails_on_verify() {
    if !pwsh_available() {
        return;
    }
    let (output, calls, _) = smoke(Some("verify"));
    assert!(!output.status.success(), "{}", text(&output));
    assert_eq!(calls, ["check", "run", "compile", "verify"]);
}

#[test]
fn local_ci_step_fails_when_its_native_command_fails() {
    if !pwsh_available() {
        return;
    }
    let temp = TempDir::new("step");
    let fake = write_fake_smc(temp.path());
    let log = temp.path().join("calls.log");
    let output = run_helper(
        "admission_guard_lib.ps1",
        "$env:FAKE_SMC_FAIL = 'verify'\nInvoke-LocalCiStep 'child' { & $env:FAKE_SMC verify x }\nWrite-Host 'STEP PASSED'",
        temp.path(),
        &[("FAKE_SMC", &fake), ("FAKE_SMC_LOG", &log)],
    );
    assert!(!output.status.success(), "{}", text(&output));
    assert!(text(&output).contains("local_ci step failed: child"));
    assert!(!text(&output).contains("STEP PASSED"));
}

fn init_repo_with_commit(dir: &Path) {
    git(dir, &["init", "-q"]);
    fs::write(dir.join("tracked.txt"), "one\n").unwrap();
    git(dir, &["add", "tracked.txt"]);
    git(dir, &["commit", "-q", "-m", "initial"]);
}

// FND-168: tracked modifications without whitespace errors pass
// `git diff --check`; the tracked-change guard must still fail on them.
#[test]
fn tracked_worktree_guard_rejects_unstaged_and_staged_changes() {
    if !pwsh_available() {
        return;
    }
    let temp = TempDir::new("dirty");
    let repo = temp.path();
    init_repo_with_commit(repo);
    let check = "Assert-TrackedWorktreeUnchanged\nWrite-Host 'TREE CLEAN'";

    fs::write(repo.join("untracked.txt"), "scratch\n").unwrap();
    let clean = run_helper("admission_guard_lib.ps1", check, repo, &[]);
    assert!(clean.status.success(), "{}", text(&clean));
    assert!(text(&clean).contains("TREE CLEAN"));

    fs::write(repo.join("tracked.txt"), "two\n").unwrap();
    let diff_check = Command::new("git")
        .args(["diff", "--check"])
        .current_dir(repo)
        .status()
        .unwrap();
    assert!(
        diff_check.success(),
        "precondition: git diff --check passes"
    );
    let unstaged = run_helper("admission_guard_lib.ps1", check, repo, &[]);
    assert!(!unstaged.status.success(), "{}", text(&unstaged));
    assert!(text(&unstaged).contains("tracked files differ from HEAD"));
    assert!(text(&unstaged).contains("tracked.txt"));

    git(repo, &["add", "tracked.txt"]);
    let staged = run_helper("admission_guard_lib.ps1", check, repo, &[]);
    assert!(!staged.status.success(), "{}", text(&staged));
    assert!(text(&staged).contains("tracked files differ from HEAD"));
}

// FND-190: `-BaseRef origin/release` must refresh `origin/release`, not
// `origin/main`, before the merge preflight uses it.
#[test]
fn merge_preflight_fetches_the_selected_remote_base_ref() {
    if !pwsh_available() {
        return;
    }
    let temp = TempDir::new("baseref");
    let upstream = temp.path().join("upstream");
    let clone = temp.path().join("clone");
    fs::create_dir_all(&upstream).unwrap();
    init_repo_with_commit(&upstream);
    git(temp.path(), &["clone", "-q", "upstream", "clone"]);

    git(&upstream, &["checkout", "-q", "-b", "release"]);
    fs::write(upstream.join("release.txt"), "release\n").unwrap();
    git(&upstream, &["add", "release.txt"]);
    git(&upstream, &["commit", "-q", "-m", "release work"]);
    let release_head = git_stdout(&upstream, &["rev-parse", "HEAD"]);

    let before = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", "origin/release"])
        .current_dir(&clone)
        .status()
        .unwrap();
    assert!(
        !before.success(),
        "precondition: origin/release not fetched"
    );

    let output = run_helper(
        "admission_guard_lib.ps1",
        "$sha = Update-MergePreflightBaseRef -BaseRef 'origin/release'\nWrite-Host \"BASE=$sha\"",
        &clone,
        &[],
    );
    assert!(output.status.success(), "{}", text(&output));
    assert!(
        text(&output).contains(&format!("BASE={release_head}")),
        "{}",
        text(&output)
    );
}

#[test]
fn merge_preflight_rejects_an_unresolvable_base_ref() {
    if !pwsh_available() {
        return;
    }
    let temp = TempDir::new("badref");
    init_repo_with_commit(temp.path());
    let output = run_helper(
        "admission_guard_lib.ps1",
        "Update-MergePreflightBaseRef -BaseRef 'no-such-base-ref' | Out-Null\nWrite-Host 'RESOLVED'",
        temp.path(),
        &[],
    );
    assert!(!output.status.success(), "{}", text(&output));
    assert!(text(&output).contains("does not resolve to a commit"));
    assert!(!text(&output).contains("RESOLVED"));
}

// FND-192: the full preflight must run every gate the default guard runs.
#[test]
fn full_and_merge_preflight_plans_are_supersets_of_the_default_guard() {
    if !pwsh_available() {
        return;
    }
    let temp = TempDir::new("plan");
    let output = run_helper(
        "admission_guard_lib.ps1",
        "foreach ($mode in 'Default','MergePreflight','FullPreflight') { Write-Host \"$mode=$((Get-AdmissionGuardPlan -Mode $mode) -join ',')\" }",
        temp.path(),
        &[],
    );
    assert!(output.status.success(), "{}", text(&output));
    let stdout = String::from_utf8_lossy(&output.stdout);
    let plan = |mode: &str| -> Vec<String> {
        let prefix = format!("{mode}=");
        stdout
            .lines()
            .find_map(|line| line.strip_prefix(&prefix))
            .unwrap_or_else(|| panic!("missing plan for {mode}: {stdout}"))
            .split(',')
            .map(str::to_string)
            .collect()
    };
    let default = plan("Default");
    for gate in [
        "PRReady",
        "Readiness",
        "LegacyAdditional",
        "DiffCheck",
        "TrackedClean",
    ] {
        assert!(default.iter().any(|g| g == gate), "default lacks {gate}");
    }
    for mode in ["FullPreflight", "MergePreflight"] {
        let superset = plan(mode);
        for gate in &default {
            assert!(superset.contains(gate), "{mode} lacks default gate {gate}");
        }
        assert!(superset.iter().any(|g| g == "MergePreflight"));
    }
}

fn release_fixture(tag_dir: &Path, assets_dir: &Path) {
    fs::create_dir_all(assets_dir).unwrap();
    for name in ["smc.exe", "svm.exe", "bundle.zip"] {
        fs::write(assets_dir.join(name), format!("asset {name}")).unwrap();
    }
    fs::create_dir_all(tag_dir.join("logs")).unwrap();
    fs::write(tag_dir.join("logs").join("stale.stdout.txt"), "stale").unwrap();
}

fn initialize_release_output(temp: &TempDir, tag_dir: &Path, assets: &Path) -> Output {
    run_helper(
        "release_asset_output.ps1",
        "$root = Initialize-ReleaseAssetOutput -TagOutputDirectory $env:TAG_DIR -AssetsDirectory $env:ASSETS -RequiredAssets @('smc.exe','svm.exe','bundle.zip')\nWrite-Host \"ROOT=$root\"",
        temp.path(),
        &[("TAG_DIR", tag_dir), ("ASSETS", assets)],
    )
}

// FND-059: re-using assets downloaded by an earlier run (inside the per-tag
// output directory) must not delete them before they are resolved.
#[test]
fn release_asset_output_preserves_caller_assets_inside_the_tag_directory() {
    if !pwsh_available() {
        return;
    }
    let temp = TempDir::new("assets_inside");
    let tag_dir = temp.path().join("release-asset-smoke").join("v9.9.9");
    let assets = tag_dir.join("downloaded");
    release_fixture(&tag_dir, &assets);

    let output = initialize_release_output(&temp, &tag_dir, &assets);
    assert!(output.status.success(), "{}", text(&output));
    let restored = tag_dir.join("caller-assets");
    for name in ["smc.exe", "svm.exe", "bundle.zip"] {
        assert_eq!(
            fs::read_to_string(restored.join(name)).unwrap(),
            format!("asset {name}")
        );
    }
    assert!(text(&output).contains("caller-assets"), "{}", text(&output));
    assert!(
        !tag_dir.join("logs").exists(),
        "stale output must still be reset"
    );
}

#[test]
fn release_asset_output_leaves_external_caller_assets_untouched() {
    if !pwsh_available() {
        return;
    }
    let temp = TempDir::new("assets_outside");
    let tag_dir = temp.path().join("release-asset-smoke").join("v9.9.9");
    let assets = temp.path().join("external-assets");
    release_fixture(&tag_dir, &assets);

    let output = initialize_release_output(&temp, &tag_dir, &assets);
    assert!(output.status.success(), "{}", text(&output));
    assert!(
        text(&output).contains("external-assets"),
        "{}",
        text(&output)
    );
    assert!(assets.join("smc.exe").exists());
    assert!(tag_dir.is_dir() && !tag_dir.join("logs").exists());
}

#[test]
fn release_asset_output_fails_before_deleting_when_assets_directory_is_missing() {
    if !pwsh_available() {
        return;
    }
    let temp = TempDir::new("assets_missing");
    let tag_dir = temp.path().join("release-asset-smoke").join("v9.9.9");
    let assets = temp.path().join("does-not-exist");
    fs::create_dir_all(tag_dir.join("logs")).unwrap();
    fs::write(tag_dir.join("logs").join("keep.txt"), "previous run").unwrap();

    let output = initialize_release_output(&temp, &tag_dir, &assets);
    assert!(!output.status.success(), "{}", text(&output));
    assert!(tag_dir.join("logs").join("keep.txt").exists());
}
