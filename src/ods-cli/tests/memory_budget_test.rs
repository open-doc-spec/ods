use ods_test_support::ChildGuard;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::time::Duration;

fn ods_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

fn rss_limit_kb() -> u64 {
    if cfg!(debug_assertions) || std::env::var("ODS_MEM_TEST_RELAXED").is_ok() {
        32_768
    } else {
        10_240
    }
}

fn parse_rss_kb(stderr: &str) -> Option<u64> {
    stderr
        .lines()
        .find_map(|line| line.split("rss_kb=").nth(1))
        .and_then(|rest| rest.split_whitespace().next())
        .and_then(|value| value.parse().ok())
}

fn run_ods(subcmd_and_args: &[&str]) -> (std::process::ExitStatus, String, String) {
    let out = Command::new(ods_bin())
        .args(subcmd_and_args)
        .env("ODS_AUTO_UPDATE", "0")
        .env("ODS_LOW_MEMORY", "1")
        .env("ODS_MEM_REPORT", "1")
        .output()
        .unwrap();
    (
        out.status,
        String::from_utf8_lossy(&out.stdout).into_owned(),
        String::from_utf8_lossy(&out.stderr).into_owned(),
    )
}

fn rss_sampling_available() -> bool {
    let out = Command::new(ods_bin())
        .arg("--version")
        .env("ODS_MEM_REPORT", "1")
        .output()
        .unwrap();
    let stderr = String::from_utf8_lossy(&out.stderr);
    parse_rss_kb(&stderr).is_some()
}

fn assert_command_under_budget(label: &str, args: &[&str]) {
    if !rss_sampling_available() {
        eprintln!("skip {label}: RSS sampling unavailable on this host");
        return;
    }
    let (status, _stdout, stderr) = run_ods(args);
    assert!(
        status.success(),
        "{label} failed: status={status} stderr={stderr}"
    );
    let rss_kb = parse_rss_kb(&stderr).unwrap_or_else(|| panic!("{label}: no rss_kb in {stderr}"));
    assert!(
        rss_kb > 0,
        "{label}: rss_kb should be positive, got {rss_kb}"
    );
    let limit = rss_limit_kb();
    assert!(
        rss_kb < limit,
        "{label}: rss_kb={rss_kb} exceeded budget {limit} KB"
    );
}

fn init_workspace(dir: &tempfile::TempDir) {
    let init = Command::new(ods_bin())
        .args(["init", dir.path().to_str().unwrap()])
        .env("ODS_AUTO_UPDATE", "0")
        .output()
        .unwrap();
    assert!(init.status.success(), "{init:?}");
    for i in 1..=20 {
        std::fs::write(
            dir.path().join(format!("doc{i}.md")),
            format!(
                "---\nprofile: note\nstatus: draft\ndescription: seed {i}\n---\n\n# Doc {i}\n\n## Section\n\n{}\n",
                "body line\n".repeat(200)
            ),
        )
        .unwrap();
    }
}

#[test]
fn lint_find_overview_stay_within_rss_budget() {
    let dir = tempfile::tempdir().unwrap();
    init_workspace(&dir);
    let root = dir.path().to_str().unwrap();

    assert_command_under_budget("lint", &["lint", root]);
    assert_command_under_budget("find", &["find", "note", "--root", root]);
    assert_command_under_budget("overview", &["overview", root]);
    assert_command_under_budget("export", &["export", root, "--format", "json"]);
    assert_command_under_budget("fmt", &["fmt", root]);
    assert_command_under_budget("tag list", &["tag", "list", root]);
}

#[test]
fn context_read_summary_stay_within_rss_budget() {
    let dir = tempfile::tempdir().unwrap();
    init_workspace(&dir);
    let root = dir.path().to_str().unwrap();

    assert_command_under_budget("read summary", &["read", root, "doc1.md", "--summary"]);
    assert_command_under_budget(
        "context",
        &["context", root, "doc1.md", "--max-tokens", "500"],
    );
}

#[test]
fn poll_serve_prints_memory_report() {
    let dir = tempfile::tempdir().unwrap();
    let init = Command::new(ods_bin())
        .args(["init", dir.path().to_str().unwrap()])
        .env("ODS_AUTO_UPDATE", "0")
        .output()
        .unwrap();
    assert!(init.status.success(), "{init:?}");
    let mut guard = ChildGuard::new(
        Command::new(ods_bin())
            .args([
                "serve",
                "--mode",
                "poll",
                "--memory-report",
                "--poll-secs",
                "60",
                "--root",
                dir.path().to_str().unwrap(),
            ])
            .env("ODS_AUTO_UPDATE", "0")
            .env("ODS_LOW_MEMORY", "1")
            .stderr(Stdio::piped())
            .spawn()
            .unwrap(),
    );
    std::thread::sleep(Duration::from_secs(2));
    let mut stderr_pipe = guard
        .child_mut()
        .expect("child")
        .stderr
        .take()
        .expect("stderr");
    let _ = guard.terminate();
    let mut stderr = String::new();
    stderr_pipe.read_to_string(&mut stderr).unwrap();
    assert!(stderr.contains("mode=poll"), "{stderr}");
    assert!(stderr.contains("rss_kb="), "{stderr}");

    let rss_kb = parse_rss_kb(&stderr);
    if rss_kb.is_none() {
        eprintln!("skip poll_serve_prints_memory_report: RSS sampling unavailable");
        return;
    }
    let rss_kb = rss_kb.unwrap();
    assert!(
        rss_kb > 0,
        "rss_kb should be a real positive sample: {rss_kb}"
    );
    assert!(
        rss_kb < rss_limit_kb(),
        "ods serve RSS ({rss_kb} KB) exceeded budget — investigate before raising service.max_rss_mb"
    );
}
