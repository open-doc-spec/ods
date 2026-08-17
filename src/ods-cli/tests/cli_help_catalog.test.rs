//! Production help catalog: top-level map + every command documents usage.
use std::path::PathBuf;
use std::process::Command;

fn ods_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

fn ods() -> Command {
    let mut c = Command::new(ods_bin());
    c.env("ODS_AUTO_UPDATE", "0");
    c
}

fn run_ok(args: &[&str]) -> String {
    let out = ods().args(args).output().expect("run ods");
    assert!(
        out.status.success(),
        "{args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).into_owned()
}

const COMMANDS: &[&str] = &[
    "init",
    "setup",
    "lint",
    "overview",
    "summary",
    "find",
    "read",
    "context",
    "tag",
    "tags",
    "tree",
    "graph",
    "schema",
    "new",
    "adopt",
    "fmt",
    "mv",
    "rm",
    "remove",
    "status",
    "archive",
    "undo",
    "doctor",
    "stats",
    "audit",
    "coverage",
    "export",
    "share",
    "diff",
    "sync",
    "clean",
    "disable",
    "revert",
    "profile",
    "profiles",
    "start",
    "stop",
    "serve",
    "watch",
    "logs",
    "lsp",
    "completion",
    "pack",
    "skill",
    "agents",
    "workspaces",
    "bench",
    "sandbox",
    "update",
    "upgrade",
    "index",
];

#[test]
fn top_level_help_is_a_full_command_map() {
    let stdout = run_ok(&["--help"]);
    assert!(stdout.contains("Usage:"));
    assert!(stdout.contains("Getting started:"));
    assert!(stdout.contains("Global options"));
    assert!(stdout.contains("Examples:"));
    assert!(stdout.contains("ods help <command>"));
    assert!(stdout.contains("setup [path]"));
    assert!(stdout.contains("serve --root . --mode poll"));
    assert!(stdout.contains("ODS_LOW_MEMORY=1"));
    assert!(stdout.contains("tag rename"));
    assert!(!stdout.contains("ods-lsp"));
    assert!(!stdout.to_ascii_lowercase().contains("zed extension"));
    assert!(!stdout.contains("Guidance for maintainers"));
    assert!(!stdout.contains("ods alias"));
    assert!(!stdout.contains("ods aliases"));
}

#[test]
fn help_command_and_flag_are_equivalent() {
    let a = run_ok(&["help", "lint"]);
    let b = run_ok(&["lint", "--help"]);
    let c = run_ok(&["--help", "lint"]);
    assert!(a.contains("Usage:"));
    assert_eq!(a, b);
    assert_eq!(a, c);
}

#[test]
fn every_command_help_documents_usage_and_exits() {
    for cmd in COMMANDS {
        let stdout = run_ok(&[cmd, "--help"]);
        assert!(
            stdout.contains("Usage:"),
            "{cmd} --help missing Usage:\n{stdout}"
        );
        assert!(
            stdout.starts_with("ods ") || stdout.contains("ods "),
            "{cmd} --help should name the binary:\n{stdout}"
        );
    }
}

#[test]
fn unknown_help_target_is_usage_error() {
    let out = ods().args(["help", "not-a-real-cmd"]).output().unwrap();
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("unknown") || err.contains("not-a-real-cmd"),
        "{err}"
    );
    assert!(err.contains("Next:"), "{err}");
}
