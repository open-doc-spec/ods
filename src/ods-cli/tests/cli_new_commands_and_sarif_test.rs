//! Integration tests for new commands (stats, completion, schema, tree, diff, clean) and SARIF output.
use std::fs;
use std::process::Command;
use tempfile::tempdir;

fn ods_bin() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

#[test]
fn test_cli_stats_command() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let init = Command::new(ods_bin())
        .args(["init", path])
        .output()
        .unwrap();
    assert!(init.status.success());

    let stats = Command::new(ods_bin())
        .args(["stats", path])
        .output()
        .unwrap();
    assert!(stats.status.success());
    let stdout = String::from_utf8_lossy(&stats.stdout);
    assert!(stdout.contains("Total Documents"), "{stdout}");

    let stats_json = Command::new(ods_bin())
        .args(["stats", path, "--format", "json"])
        .output()
        .unwrap();
    assert!(stats_json.status.success());
    let json_str = String::from_utf8_lossy(&stats_json.stdout);
    assert!(json_str.contains("total_documents"), "{json_str}");
}

#[test]
fn test_cli_completion_command() {
    for shell in ["bash", "zsh", "fish", "powershell"] {
        let out = Command::new(ods_bin())
            .args(["completion", shell])
            .output()
            .unwrap();
        assert!(out.status.success());
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(stdout.contains("ods"), "{stdout}");
    }
}

#[test]
fn test_cli_schema_command() {
    let dir = tempdir().unwrap();
    let schema_file = dir.path().join("ods.schema.json");

    let out = Command::new(ods_bin())
        .args(["schema", "--write", "--out", schema_file.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success());
    assert!(schema_file.exists());
    let content = fs::read_to_string(&schema_file).unwrap();
    assert!(content.contains("Open Document Spec (ODS) 2.0 Frontmatter Schema"));
}

#[test]
fn test_cli_tree_command() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let _ = Command::new(ods_bin())
        .args(["init", path])
        .output()
        .unwrap();

    let tree = Command::new(ods_bin())
        .args(["tree", path])
        .output()
        .unwrap();
    assert!(tree.status.success());
    let stdout = String::from_utf8_lossy(&tree.stdout);
    assert!(
        stdout.contains("ods.toml") || stdout.contains("index.ods.md"),
        "{stdout}"
    );
}

#[test]
fn test_cli_diff_command() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let _ = Command::new(ods_bin())
        .args(["init", path])
        .output()
        .unwrap();

    let diff = Command::new(ods_bin())
        .args(["diff", path])
        .output()
        .unwrap();
    assert!(diff.status.success() || diff.status.code() == Some(128)); // 128 if not git repo
}

#[test]
fn test_cli_clean_command() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let _ = Command::new(ods_bin())
        .args(["init", path])
        .output()
        .unwrap();
    let err_file = dir.path().join(".ods/ods-errors.md");
    fs::create_dir_all(err_file.parent().unwrap()).unwrap();
    fs::write(&err_file, "# Dummy Errors").unwrap();

    let clean = Command::new(ods_bin())
        .args(["clean", path])
        .output()
        .unwrap();
    assert!(clean.status.success());
    assert!(!err_file.exists());
}

#[test]
fn test_cli_lint_sarif_format() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();

    let _ = Command::new(ods_bin())
        .args(["init", path])
        .output()
        .unwrap();

    let sarif = Command::new(ods_bin())
        .args(["lint", path, "--format", "sarif"])
        .output()
        .unwrap();
    assert!(sarif.status.success());
    let stdout = String::from_utf8_lossy(&sarif.stdout);
    assert!(stdout.contains("sarif-schema-2.1.0.json"), "{stdout}");
}

#[test]
fn test_cli_setup_git_hooks() {
    let dir = tempdir().unwrap();
    let path = dir.path().to_str().unwrap();
    fs::write(
        dir.path().join("ods.toml"),
        "spec = \"0.1\"
",
    )
    .unwrap();

    let git_dir = dir.path().join(".git/hooks");
    fs::create_dir_all(&git_dir).unwrap();

    let setup = Command::new(ods_bin())
        .env("ODS_AUTO_UPDATE", "0")
        .env("ODS_SETUP_NO_START", "1")
        .args(["setup", path, "--git-hooks"])
        .output()
        .unwrap();
    assert!(setup.status.success(), "{:?}", setup);
    let hook_file = git_dir.join("pre-commit");
    assert!(hook_file.exists());
    let content = fs::read_to_string(&hook_file).unwrap();
    assert!(content.contains("ods lint"));
}
