use ods_test_support::temp_workspace;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn ods_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

#[test]
fn help_lists_new_commands() {
    let out = Command::new(ods_bin()).arg("help").output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    for cmd in [
        "setup",
        "doctor",
        "sync",
        "watch",
        "tags",
        "find",
        "tag rename",
        "init",
        "disable",
        "update",
        "workspaces",
    ] {
        assert!(stdout.contains(cmd), "help missing {cmd}: {stdout}");
    }
}

#[test]
fn setup_help_lists_setup_behavior() {
    let out = Command::new(ods_bin())
        .args(["setup", "--help"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("ods setup") || stdout.contains("ods setup"),
        "{stdout}"
    );
    assert!(stdout.contains("doctor"), "{stdout}");
}

#[test]
fn workspaces_help_lists_subcommands() {
    let out = Command::new(ods_bin())
        .args(["workspaces", "--help"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("ods workspaces") || stdout.contains("ods workspaces"),
        "{stdout}"
    );
    assert!(stdout.contains("add"), "{stdout}");
    assert!(stdout.contains("remove"), "{stdout}");
    assert!(stdout.contains("list"), "{stdout}");
}

#[test]
fn setup_outside_workspace_prompts_to_run_init() {
    let dir = tempfile::Builder::new()
        .prefix("ods-setup-outside-")
        .tempdir()
        .unwrap();
    let out = Command::new(ods_bin())
        .env("ODS_AUTO_UPDATE", "0")
        .env("ODS_SETUP_NO_START", "1")
        .args(["setup", dir.path().to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("no ODS workspace found"), "{stdout}");
    assert!(
        stdout.contains("run 'ods init")
            || stdout.contains("run 'ods init")
            || stdout.contains("ods init"),
        "{stdout}"
    );
    assert!(!dir.path().join("ods.toml").exists());
}

#[test]
fn setup_inside_workspace_runs_doctor_without_test_service_start() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    let out = Command::new(ods_bin())
        .args(["init", root])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);

    let out = Command::new(ods_bin())
        .env("ODS_AUTO_UPDATE", "0")
        .env("ODS_SETUP_NO_START", "1")
        .args(["setup", root])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("workspace"), "{stdout}");
    assert!(stdout.contains("service"), "{stdout}");
    assert!(stdout.contains("doctor"), "{stdout}");
    assert!(
        stdout.contains("ods version") || stdout.contains("ods cli version"),
        "{stdout}"
    );
    assert!(stdout.contains("root ods spec"), "{stdout}");
    assert!(stdout.contains("root ods"), "{stdout}");
}

#[test]
fn setup_updates_stale_root_ods_version() {
    let dir = temp_workspace();
    fs::write(dir.join("ods.toml"), "spec = \"draft-1\"\n").unwrap();

    let out = Command::new(ods_bin())
        .env("ODS_AUTO_UPDATE", "0")
        .env("ODS_SETUP_NO_START", "1")
        .args(["setup", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);

    let root = fs::read_to_string(dir.join("ods.toml")).unwrap();
    assert!(root.contains("spec = \"2.0\""), "{root}");
}

#[test]
fn doctor_reports_stale_root_ods_version() {
    let dir = temp_workspace();
    fs::write(dir.join("ods.toml"), "spec = \"draft-1\"\n").unwrap();

    let out = Command::new(ods_bin())
        .args(["doctor", dir.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(!out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("root ods spec: draft-1"), "{stdout}");
    assert!(stdout.contains("2.0"), "{stdout}");
}
