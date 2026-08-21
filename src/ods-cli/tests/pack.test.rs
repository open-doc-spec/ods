use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn ods_bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_ods"))
}

#[test]
fn test_pack_subcommands_end_to_end() {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();

    // 1. Initialize ODS workspace
    let status = ods_bin()
        .args(["init", root.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success());

    // 2. Run `ods pack init my-custom-pack`
    let pack_dir = root.join("my-custom-pack");
    let status = ods_bin()
        .args(["pack", "init", pack_dir.to_str().unwrap()])
        .status()
        .unwrap();
    assert!(status.success());
    assert!(pack_dir.join("ods.toml").exists());
    assert!(pack_dir.join("ods-profiles").is_dir());

    // 3. Run `ods pack add ./my-custom-pack` from workspace root
    let status = ods_bin()
        .current_dir(root)
        .args(["pack", "add", "./my-custom-pack"])
        .status()
        .unwrap();
    assert!(status.success());

    // Verify packs is written to root ods.toml
    let root_index_content = fs::read_to_string(root.join("ods.toml")).unwrap();
    assert!(root_index_content.contains("packs"));
    assert!(root_index_content.contains("my-custom-pack"));

    // 4. Run `ods pack list`
    let output = ods_bin()
        .current_dir(root)
        .args(["pack", "list"])
        .output()
        .unwrap();
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("ODS Workspace Packs"));
    assert!(stdout.contains("my-custom-pack"));

    // 5. Run `ods pack remove my-custom-pack`
    let status = ods_bin()
        .current_dir(root)
        .args(["pack", "remove", "my-custom-pack"])
        .status()
        .unwrap();
    assert!(status.success());
}
