//! AI discovery surface: find --key, tag list/show, schema keys, overview, context filters.
use ods_test_support::{TempWorkspace, temp_workspace};
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn ods_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

fn init_ws() -> (TempWorkspace, String) {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap().to_string();
    let out = Command::new(ods_bin())
        .args(["init", &root])
        .output()
        .unwrap();
    assert!(out.status.success(), "init: {:?}", out);
    (dir, root)
}

fn write_doc(dir: &TempWorkspace, name: &str, body: &str) {
    fs::write(dir.join(name), body).unwrap();
}

#[test]
fn find_by_key_status_custom_and_tag_match() {
    let (dir, root) = init_ws();
    write_doc(
        &dir,
        "a.md",
        "---\nprofile: note\nstatus: draft\nowner: alice\nteam: infra\ntags:\n  - auth\n  - billing\n---\n\n# A\n",
    );
    write_doc(
        &dir,
        "b.md",
        "---\nprofile: feature\nstatus: stable\nowner: bob\nteam: frontend\ntags:\n  - auth\n---\n\n# B\n\n## Goal\n\n## Scope\n\n## Requirements\n\n## Acceptance Criteria\n\n## Risks\n",
    );
    write_doc(
        &dir,
        "c.md",
        "---\nprofile: note\nstatus: draft\ntags:\n  - billing\n---\n\n# C\n",
    );
    assert!(
        Command::new(ods_bin())
            .args(["lint", &root])
            .output()
            .unwrap()
            .status
            .success()
    );

    let out = Command::new(ods_bin())
        .args(["find", &root, "--key", "status=draft", "--format", "json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"keys\""), "{stdout}");
    assert!(stdout.contains("a") || stdout.contains("c"), "{stdout}");
    assert!(!stdout.contains("\"b\""), "{stdout}");

    let out = Command::new(ods_bin())
        .args([
            "find",
            &root,
            "--key",
            "status=draft,stable",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("\"count\":3") || stdout.contains("\"count\": 3"),
        "{stdout}"
    );

    let out = Command::new(ods_bin())
        .args(["find", &root, "--key", "team=infra", "--format", "json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("a"), "{stdout}");

    let out = Command::new(ods_bin())
        .args([
            "find",
            &root,
            "--tag",
            "auth",
            "--tag",
            "billing",
            "--tag-match",
            "all",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("a"), "{stdout}");

    // Zero matches still success with empty ids.
    let out = Command::new(ods_bin())
        .args(["find", &root, "--status", "archived", "--format", "json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("\"count\":0") || stdout.contains("\"count\": 0"),
        "{stdout}"
    );

    // Usage when no criteria.
    let out = Command::new(ods_bin())
        .args(["find", &root])
        .output()
        .unwrap();
    assert!(!out.status.success(), "expected usage");
}

#[test]
fn tag_list_show_and_tags_regression() {
    let (dir, root) = init_ws();
    write_doc(
        &dir,
        "a.md",
        "---\nprofile: note\nstatus: draft\ntags:\n  - alpha\n---\n\n# A\n",
    );
    assert!(
        Command::new(ods_bin())
            .args(["lint", &root])
            .output()
            .unwrap()
            .status
            .success()
    );

    let out = Command::new(ods_bin())
        .args(["tag", "list", &root, "--format", "json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("alpha"), "{stdout}");

    let out = Command::new(ods_bin())
        .args(["tag", "show", &root, "alpha", "--format", "json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("\"count\":1") || stdout.contains("\"count\": 1"),
        "{stdout}"
    );

    // Regression: ods tags still works.
    let out = Command::new(ods_bin())
        .args(["tags", &root])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("alpha"), "{stdout}");

    let out = Command::new(ods_bin())
        .args(["tag", "show", &root])
        .output()
        .unwrap();
    assert!(!out.status.success(), "missing tag name should fail");

    // Rename dry-run + write.
    let out = Command::new(ods_bin())
        .args(["tag", "rename", &root, "alpha", "beta", "--format", "text"])
        .output()
        .unwrap();
    assert!(out.status.success(), "rename dry-run: {:?}", out);
    let out = Command::new(ods_bin())
        .args([
            "tag", "rename", &root, "alpha", "beta", "--write", "--format", "json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "rename write: {:?}", out);
    let body = fs::read_to_string(dir.join("a.md")).unwrap();
    assert!(body.contains("beta"), "{body}");
}

#[test]
fn schema_keys_and_schema_regression() {
    let out = Command::new(ods_bin())
        .args(["schema", "keys"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("profile") || stdout.contains("status"),
        "{stdout}"
    );

    let out = Command::new(ods_bin())
        .args(["schema", "keys", "--format", "json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("\"keys\""), "{stdout}");
    assert!(stdout.contains("\"queryable\""), "{stdout}");

    // Regression: bare schema still emits JSON Schema properties.
    let out = Command::new(ods_bin()).args(["schema"]).output().unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("properties") || stdout.contains("$schema"),
        "{stdout}"
    );
}

#[test]
fn overview_and_summary_and_stats_regression() {
    let (dir, root) = init_ws();
    write_doc(
        &dir,
        "a.md",
        "---\nprofile: note\nstatus: draft\nteam: infra\ntags:\n  - t1\n---\n\n# A\n",
    );
    assert!(
        Command::new(ods_bin())
            .args(["lint", &root])
            .output()
            .unwrap()
            .status
            .success()
    );

    let out = Command::new(ods_bin())
        .args(["overview", &root, "--format", "text"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("Total Documents"), "{stdout}");
    assert!(stdout.contains("Profiles"), "{stdout}");

    let out = Command::new(ods_bin())
        .args(["summary", &root, "--format", "json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("total_documents"), "{stdout}");
    assert!(stdout.contains("custom_keys"), "{stdout}");

    // Regression: stats still reports health.
    let out = Command::new(ods_bin())
        .args(["stats", &root])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("Health") || stdout.contains("health"),
        "{stdout}"
    );
}

#[test]
fn context_filter_unique_multi_and_id_regression() {
    let (dir, root) = init_ws();
    write_doc(
        &dir,
        "only.md",
        "---\nprofile: note\nstatus: draft\nid: only-doc\ntags:\n  - unique-tag\n---\n\n# Only\n",
    );
    write_doc(
        &dir,
        "m1.md",
        "---\nprofile: note\nstatus: stable\ntags:\n  - multi\n---\n\n# M1\n",
    );
    write_doc(
        &dir,
        "m2.md",
        "---\nprofile: note\nstatus: stable\ntags:\n  - multi\n---\n\n# M2\n",
    );
    assert!(
        Command::new(ods_bin())
            .args(["lint", &root])
            .output()
            .unwrap()
            .status
            .success()
    );

    // Unique tag filter resolves.
    let out = Command::new(ods_bin())
        .args(["context", &root, "--tag", "unique-tag"])
        .output()
        .unwrap();
    assert!(out.status.success(), "unique filter: {:?}", out);

    // Multi-match fails.
    let out = Command::new(ods_bin())
        .args(["context", &root, "--tag", "multi"])
        .output()
        .unwrap();
    assert!(!out.status.success(), "multi should fail");
    let err = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout)
    );
    assert!(
        err.contains("matched") || err.contains("unique") || err.contains("filter"),
        "{err}"
    );

    // Classic id path still works.
    let out = Command::new(ods_bin())
        .args(["context", &root, "only-doc"])
        .output()
        .unwrap();
    assert!(out.status.success(), "id path: {:?}", out);

    // Multi-key filter (unique match).
    write_doc(
        &dir,
        "okf.md",
        "---\nprofile: note\nstatus: draft\ntype: Dataset\nresource: bq://proj/table\nload:\n  - fixture.json\n---\n\n# OKF\n",
    );
    fs::write(dir.join("fixture.json"), "{\"ok\":true}\n").unwrap();
    let out = Command::new(ods_bin())
        .args([
            "context",
            &root,
            "--key",
            "status=draft",
            "--key",
            "type=Dataset",
            "--key-match",
            "and",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "multi-key context: {:?}", out);

    let out = Command::new(ods_bin())
        .args([
            "find",
            &root,
            "--key",
            "type=Dataset",
            "--key",
            "status=draft",
            "--format",
            "json",
        ])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("okf") || stdout.contains("Dataset"),
        "{stdout}"
    );

    let out = Command::new(ods_bin())
        .args(["find", &root, "--key", "load", "--format", "json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);

    // No id and no filter → usage.
    let out = Command::new(ods_bin())
        .args(["context", &root])
        .output()
        .unwrap();
    assert!(!out.status.success());
}
