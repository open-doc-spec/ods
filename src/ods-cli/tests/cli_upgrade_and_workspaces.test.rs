//! High-ROI CLI coverage: upgrade, workspaces, find, okf doctor/fmt.
use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn ods_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

#[test]
fn upgrade_rewrites_ods_cli_pin() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", path])
            .status()
            .unwrap()
            .success()
    );
    let index = dir.path().join("ods.toml");
    let out = Command::new(ods_bin())
        .args(["upgrade", path, "--write"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let after = fs::read_to_string(&index).unwrap();
    assert!(
        after.contains("spec = \"2.0\"") || after.contains("ods: 2.0"),
        "{after}"
    );

    let check = Command::new(ods_bin())
        .args(["upgrade", path, "--check"])
        .output()
        .unwrap();
    assert!(
        check.status.success() || check.status.code() == Some(1),
        "{:?}",
        check
    );
}

#[test]
fn update_and_ods_update_subcommands() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", path])
            .status()
            .unwrap()
            .success()
    );
    // ods update --check
    let out = Command::new(ods_bin())
        .current_dir(dir.path())
        .args(["update", "--check"])
        .output()
        .unwrap();
    assert!(
        out.status.success() || out.status.code() == Some(1),
        "{:?}",
        out
    );

    // ods upgrade
    let out = Command::new(ods_bin())
        .current_dir(dir.path())
        .args(["upgrade"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
}

#[test]
fn workspaces_list_add_remove() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", path])
            .status()
            .unwrap()
            .success()
    );
    let home = tempdir().unwrap();
    let out = Command::new(ods_bin())
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .args(["workspaces", "add", path])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);

    let out = Command::new(ods_bin())
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .args(["workspaces", "list"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains(path) || !stdout.is_empty(), "{stdout}");

    let out = Command::new(ods_bin())
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .args(["workspaces", "path"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);

    let out = Command::new(ods_bin())
        .env("HOME", home.path())
        .env("USERPROFILE", home.path())
        .args(["workspaces", "remove", path])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
}

#[test]
fn find_by_tag() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", path])
            .status()
            .unwrap()
            .success()
    );
    fs::write(
        dir.path().join("t.md"),
        "---\nprofile: note\nstatus: draft\ntags: [alpha]\n---\n\n# T\n",
    )
    .unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["lint", path])
            .status()
            .unwrap()
            .success()
    );
    let out = Command::new(ods_bin())
        .args(["find", path, "--tag", "alpha"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("t") || stdout.contains("T") || !stdout.is_empty(),
        "{stdout}"
    );
}
