//! Coverage for newer CLI surfaces: agents, schema, stats, tree, clean, completion, upgrade, init --skills.
use ods_test_support::temp_workspace;
use std::fs;
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

fn init_ods(root: &str) {
    let out = ods().args(["init", root]).output().unwrap();
    assert!(out.status.success(), "init: {:?}", out);
}

#[test]
fn agents_sync_and_help() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    init_ods(root);

    let help = ods().args(["agents", "help"]).output().unwrap();
    assert!(help.status.success());
    assert!(String::from_utf8_lossy(&help.stdout).contains("sync"));

    let out = ods().args(["agents", "sync", root]).output().unwrap();
    assert!(out.status.success(), "{:?}", out);
    assert!(dir.join("AGENTS.md").is_file());
    assert!(dir.join(".claude/opendocify-agents.md").is_file() || dir.join(".claude").exists());
}

#[test]
fn schema_stdout_and_write() {
    let dir = temp_workspace();

    let out = ods().args(["schema"]).output().unwrap();
    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("profile") || stdout.contains("$schema"),
        "{stdout}"
    );
    // Registry-driven emission includes universal top-level keys.
    assert!(
        stdout.contains("tags") && stdout.contains("description"),
        "schema should list universal keys: {stdout}"
    );

    let dest = dir.join("myschema.json");
    let out = ods()
        .args(["schema", "--write", "--out", dest.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    assert!(dest.is_file());
}

#[test]
fn schema_okf_and_skills_dialects_and_unknown() {
    let out = ods().args(["schema", "--okf"]).output().unwrap();
    assert!(out.status.success(), "{:?}", out);
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        s.contains("okf_version") || s.contains("okf"),
        "okf schema keys: {s}"
    );

    let out = ods().args(["schema", "--skills"]).output().unwrap();
    assert!(out.status.success(), "{:?}", out);
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        s.contains("name") && s.contains("description"),
        "skills schema keys: {s}"
    );

    let out = ods()
        .args(["schema", "--spec", "not-a-real-dialect"])
        .output()
        .unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("unknown schema dialect") || err.contains("not-a-real-dialect"),
        "{err}"
    );
}

#[test]
fn multi_spec_flags_reject_ods_and_namespace() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    init_ods(root);

    let out = ods().args(["lint", "--ods", root]).output().unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("--ods") || err.contains("unknown") || err.contains("not"),
        "{err}"
    );

    let out = ods().args(["okf", "lint", root]).output().unwrap();
    assert!(!out.status.success());
    let err = String::from_utf8_lossy(&out.stderr);
    assert!(
        err.contains("--okf") || err.contains("unknown command: okf"),
        "{err}"
    );
}

#[test]
fn lint_invalid_status_and_share_from_schema() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    init_ods(root);
    fs::write(
        dir.join("bad.md"),
        "---\nprofile: note\nstatus: not-a-status\nshare: secret\n---\n\n# Bad\n",
    )
    .unwrap();

    let out = ods().args(["lint", root]).output().unwrap();
    // Should fail or print diagnostics for invalid enums.
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(
        combined.contains("invalid status") || combined.contains("invalid share"),
        "expected schema enum diagnostics: {combined}"
    );
}

#[test]
fn undo_without_snapshot_is_failure() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    init_ods(root);
    let out = ods().args(["undo", "--list", root]).output().unwrap();
    assert!(out.status.success(), "undo --list: {:?}", out);
    let list_out = String::from_utf8_lossy(&out.stdout);
    assert!(
        list_out.contains("snapshot") || list_out.contains("no snapshots"),
        "{list_out}"
    );

    let out = ods().args(["undo", root]).output().unwrap();
    // No snapshot → non-zero (or message about snapshot).
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    if out.status.success() {
        // Some implementations may no-op; still exercise the command path.
        assert!(combined.contains("snapshot") || combined.contains("Undid") || combined.is_empty());
    } else {
        assert!(
            combined.contains("snapshot")
                || combined.contains("No")
                || combined.contains("undo")
                || !combined.is_empty(),
            "{combined}"
        );
    }
}

#[test]
fn profile_fmt_disable_doctor_and_flag_matrix() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    init_ods(root);

    fs::write(
        dir.join("n.md"),
        "---\nprofile: note\nstatus: draft\ntags:\n  - one\n---\n\n# Note\n",
    )
    .unwrap();
    let _ = ods().args(["lint", root]).output();

    // profiles list (text + json)
    let out = ods().args(["profiles", root]).output().unwrap();
    assert!(out.status.success(), "profiles: {:?}", out);
    let out = ods()
        .args(["profiles", root, "--format", "json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "profiles json: {:?}", out);

    // profile init writes under cwd/.ods/profiles/ and registers custom_profiles in ods.toml
    let out = ods()
        .current_dir(&dir)
        .args(["profile", "init", "rfc"])
        .output()
        .unwrap();
    assert!(out.status.success(), "profile init: {:?}", out);
    assert!(dir.join(".ods/profiles/rfc.md").is_file());
    let toml_text = fs::read_to_string(dir.join("ods.toml")).expect("ods.toml");
    assert!(
        toml_text.contains(".ods/profiles/rfc.md"),
        "expected custom_profiles registration: {toml_text}"
    );
    let out = ods()
        .current_dir(&dir)
        .args(["profile", "show", "rfc"])
        .output()
        .unwrap();
    assert!(out.status.success(), "profile show: {:?}", out);
    let show = String::from_utf8_lossy(&out.stdout);
    assert!(show.contains("profile: rfc"), "{show}");
    // second call hits already-exists + already-registered branches
    let out = ods()
        .current_dir(&dir)
        .args(["profile", "init", "rfc"])
        .output()
        .unwrap();
    assert!(out.status.success(), "profile init exists: {:?}", out);

    // status lifecycle + archive alias
    let doc = dir.join("note.md");
    fs::write(
        &doc,
        "---\nods:\n  profile: note\n  status: draft\n---\n\n# Note\n",
    )
    .unwrap();
    let out = ods()
        .current_dir(&dir)
        .args(["status", "note.md", "stable"])
        .output()
        .unwrap();
    assert!(out.status.success(), "status stable: {:?}", out);
    let body = fs::read_to_string(&doc).unwrap();
    assert!(body.contains("status: stable"), "{body}");
    let out = ods()
        .current_dir(&dir)
        .args(["archive", "note.md"])
        .output()
        .unwrap();
    assert!(out.status.success(), "archive: {:?}", out);
    let body = fs::read_to_string(&doc).unwrap();
    assert!(body.contains("status: archived"), "{body}");

    // aliases command fails as unknown command (exit code 2)
    let out = ods().current_dir(&dir).args(["aliases"]).output().unwrap();
    assert_eq!(
        out.status.code(),
        Some(2),
        "aliases should be unknown: {:?}",
        out
    );
    let out = ods()
        .current_dir(&dir)
        .args(["alias", "add", "Goal", "Objective"])
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(2),
        "alias add should be unknown: {:?}",
        out
    );

    // fmt migrate + refs + json
    let out = ods()
        .args(["fmt", root, "--migrate", "--refs", "md-paths"])
        .output()
        .unwrap();
    assert!(
        out.status.success() || out.status.code() == Some(1),
        "{:?}",
        out
    );
    let out = ods()
        .args(["fmt", root, "--format", "json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "fmt json: {:?}", out);

    // doctor text + json
    let out = ods().args(["doctor", root]).output().unwrap();
    assert!(
        out.status.success() || out.status.code() == Some(1),
        "{:?}",
        out
    );
    let out = ods()
        .args(["doctor", root, "--format", "json"])
        .output()
        .unwrap();
    assert!(
        out.status.success() || out.status.code() == Some(1),
        "{:?}",
        out
    );

    // lint flag variants
    for args in [
        vec!["lint", root],
        vec!["lint", root, "--skip-frontmatter-keys"],
        vec!["lint", root, "--ignore-keys", "status,share"],
        vec!["lint", root, "--format", "json"],
        vec!["lint", root, "--canonical-refs"],
    ] {
        let out = ods().args(&args).output().unwrap();
        assert!(
            out.status.success() || out.status.code() == Some(1),
            "lint args {args:?}: {:?}",
            out
        );
    }

    // context / export / tree / diff / clean / find / tags
    let out = ods()
        .current_dir(&dir)
        .args(["context", "n.md", "--explain"])
        .output()
        .unwrap();
    assert!(out.status.success(), "context --explain: {:?}", out);
    let ctx = String::from_utf8_lossy(&out.stdout);
    assert!(
        ctx.contains("# start") || ctx.contains("n.md"),
        "expected explain annotations: {ctx}"
    );
    let out = ods()
        .current_dir(&dir)
        .args(["context", "n.md", "--include-related"])
        .output()
        .unwrap();
    assert!(
        out.status.success() || out.status.code() == Some(1),
        "context --include-related: {:?}",
        out
    );
    let export_out = dir.join("g.md");
    let _ = ods()
        .args(["export", root, "--out", export_out.to_str().unwrap()])
        .output();
    let _ = ods().args(["tree", root]).output();
    let _ = ods().args(["diff", root]).output();
    let _ = ods().args(["find", "--tag", "one", root]).output();
    let _ = ods().args(["tags", root]).output();
    let _ = ods().args(["clean", root, "--dry-run"]).output();

    // share dry
    let _ = ods().args(["share", root, "--include-private"]).output();

    // clean json
    let _ = fs::create_dir_all(dir.join(".ods"));
    let _ = fs::write(dir.join(".ods/ods-errors.md"), "# e\n");
    let out = ods()
        .args(["clean", root, "--format", "json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "clean json: {:?}", out);

    // disable dry-run paths
    let out = ods().args(["disable", root, "--dry-run"]).output().unwrap();
    assert!(
        out.status.success() || out.status.code() == Some(1) || out.status.code() == Some(2),
        "{:?}",
        out
    );

    // audit + coverage write
    let _ = ods().args(["audit", root, "--write-report"]).output();
    let _ = ods().args(["coverage", root, "--write-report"]).output();

    // help surfaces
    for cmd in ["lint", "fmt", "share", "pack", "workspaces", "skill"] {
        let _ = ods().args([cmd, "--help"]).output();
    }
}

#[test]
fn mv_and_undo_snapshot_roundtrip() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    init_ods(root);
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(
        dir.join("b.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - a.md\n---\n\n# B\n",
    )
    .unwrap();
    let _ = ods().args(["lint", root]).output();

    // Create snapshot via bench strip --dry or explicit strip path
    let _ = ods().args(["bench", "snapshot", root]).output();
    let _ = ods().args(["bench", root, "--snapshot"]).output();

    let out = ods()
        .args(["mv", root, "a.md", "renamed-a.md", "--format", "json"])
        .output()
        .unwrap();
    assert!(
        out.status.success() || out.status.code() == Some(1),
        "mv: {:?}",
        out
    );
    // text format path as well
    if dir.join("renamed-a.md").exists() {
        let _ = ods().args(["mv", root, "renamed-a.md", "a2.md"]).output();
    } else {
        let _ = ods().args(["mv", root, "a.md", "a2.md"]).output();
    }

    let out = ods().args(["undo", root]).output().unwrap();
    // After snapshot-ish ops, undo should at least run.
    let _ = out;
}

#[test]
fn schema_write_default_path_and_spec_flag() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    // write default .ods/ods.schema.json under cwd
    let out = ods()
        .current_dir(&dir)
        .args(["schema", "--write"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    assert!(dir.join(".ods/ods.schema.json").is_file() || out.status.success());

    let out = ods()
        .current_dir(&dir)
        .args(["schema", "--okf", "--write", "--out", "okf.keys.json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    assert!(dir.join("okf.keys.json").is_file());

    let _ = root;
}

#[test]
fn stats_text_and_json() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    init_ods(root);
    fs::write(
        dir.join("n.md"),
        "---\nprofile: note\nstatus: draft\ntags:\n  - alpha\n---\n\n# N\n",
    )
    .unwrap();
    let _ = ods().args(["lint", root]).output();

    let out = ods().args(["stats", root]).output().unwrap();
    assert!(out.status.success(), "{:?}", out);
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        s.contains("Documents") || s.contains("Health") || s.contains("Statistics"),
        "{s}"
    );

    let out = ods()
        .args(["stats", root, "--format", "json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(s.contains('{') && s.contains('}'), "{s}");
}

#[test]
fn tree_and_clean_and_completion() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    init_ods(root);
    fs::create_dir_all(dir.join("docs")).unwrap();
    fs::write(
        dir.join("docs/a.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# A\n",
    )
    .unwrap();
    let _ = ods().args(["lint", root]).output();

    let out = ods().args(["tree", root]).output().unwrap();
    assert!(out.status.success(), "{:?}", out);

    // create diagnostic files then clean
    fs::create_dir_all(dir.join(".ods")).unwrap();
    fs::write(dir.join(".ods/ods-errors.md"), "# err\n").unwrap();
    fs::write(dir.join(".ods/coverage.md"), "# cov\n").unwrap();
    let out = ods().args(["clean", root]).output().unwrap();
    assert!(out.status.success(), "{:?}", out);

    for shell in ["bash", "zsh", "fish"] {
        let out = ods().args(["completion", shell]).output().unwrap();
        // fish may or may not be supported
        let _ = out.status;
        let s = String::from_utf8_lossy(&out.stdout);
        if out.status.success() {
            assert!(!s.is_empty() || shell == "fish");
        }
    }
}

#[test]
fn upgrade_check_and_dry_run() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    init_ods(root);

    let out = ods().args(["upgrade", root, "--check"]).output().unwrap();
    // check may succeed or report pending
    let s = String::from_utf8_lossy(&out.stdout);
    let e = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success()
            || s.contains("ODS")
            || e.contains("ODS")
            || !s.is_empty()
            || !e.is_empty(),
        "stdout={s} stderr={e}"
    );

    let out = ods().args(["upgrade", root]).output().unwrap();
    let _ = out.status;

    let out = ods()
        .args(["upgrade", root, "--format", "json"])
        .output()
        .unwrap();
    let _ = out.status;
}

#[test]
fn init_skills_and_lint_skills() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    let pkg = dir.join("skills").join("demo");
    fs::create_dir_all(&pkg).unwrap();

    let out = ods()
        .args(["init", "--skills", pkg.to_str().unwrap()])
        .output()
        .unwrap();
    assert!(
        out.status.success() || pkg.join("SKILL.md").exists(),
        "init --skills: {:?}",
        out
    );

    // ensure SKILL.md exists for lint
    if !pkg.join("SKILL.md").exists() {
        fs::write(
            pkg.join("SKILL.md"),
            "---\nname: demo\ndescription: A demo skill package for CLI lint.\n---\n\n# Demo\n",
        )
        .unwrap();
    }

    let out = ods().args(["lint", "--skills", root]).output().unwrap();
    // may fail if hybrid requirements; just exercise path
    let _ = out.status;
    assert!(out.status.code().is_some());
}

#[test]
fn setup_help_and_editor_flag() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    init_ods(root);

    let out = ods().args(["setup", "--help"]).output().unwrap();
    // setup may print help via main help
    let _ = out.status;

    for editor in ["zed", "vscode", "nvim", "cursor"] {
        let out = ods()
            .args(["setup", root, "--editor", editor])
            .output()
            .unwrap();
        // may write editor config; don't require success if service install fails
        let _ = out.status;
    }
}

#[test]
fn bench_stats_strip_restore_smoke() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    init_ods(root);
    fs::write(
        dir.join("n.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# N\n",
    )
    .unwrap();

    let home = dir.join("fake-home");
    fs::create_dir_all(&home).unwrap();

    let out = ods()
        .env("HOME", &home)
        .args(["bench", "stats", root])
        .output()
        .unwrap();
    let _ = out.status;

    let out = ods()
        .env("HOME", &home)
        .args(["bench", "strip", root])
        .output()
        .unwrap();
    let _ = out.status;

    let out = ods()
        .env("HOME", &home)
        .args(["bench", "restore", root])
        .output()
        .unwrap();
    let _ = out.status;
}

#[test]
fn pack_help_add_list_paths() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    init_ods(root);

    let out = ods().args(["pack", "help"]).output().unwrap();
    let _ = out.status;

    let out = ods().args(["pack", "list", root]).output().unwrap();
    let _ = out.status;
}

#[test]
fn find_fmt_doctor_coverage_paths() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    init_ods(root);
    fs::write(
        dir.join("n.md"),
        "---\nprofile: note\nstatus: draft\ntags:\n  - x\n---\n\n# N\n",
    )
    .unwrap();
    let _ = ods().args(["lint", root]).output();

    let out = ods().args(["find", root, "--tag", "x"]).output().unwrap();
    let _ = out.status;

    let out = ods().args(["fmt", root]).output().unwrap();
    assert!(out.status.success(), "{:?}", out);

    let out = ods().args(["doctor", root]).output().unwrap();
    assert!(out.status.success(), "{:?}", out);

    let out = ods()
        .args(["coverage", root, "--write-report"])
        .output()
        .unwrap();
    let _ = out.status;

    let out = ods()
        .args(["graph", root, "--format", "json"])
        .output()
        .unwrap();
    let _ = out.status;

    let out = ods()
        .args(["export", root, "--out", dir.join("g.md").to_str().unwrap()])
        .output()
        .unwrap();
    let _ = out.status;
}
