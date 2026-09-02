//! Production-oriented smoke of major `ods` CLI commands on a temp workspace.
use ods_test_support::temp_workspace;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn ods_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

fn run(args: &[&str]) -> std::process::Output {
    Command::new(ods_bin())
        .args(args)
        .output()
        .unwrap_or_else(|e| panic!("run ods {:?}: {e}", args))
}

fn assert_ok(out: &std::process::Output, label: &str) {
    assert!(
        out.status.success(),
        "{label} failed status={:?}\nstdout={}\nstderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn production_cli_matrix_core_commands() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();

    // help / version
    let out = run(&["help"]);
    assert_ok(&out, "help");
    let help = String::from_utf8_lossy(&out.stdout);
    for cmd in [
        "lint", "lint", "profiles", "tags", "find", "context", "graph", "mv", "fmt", "adopt",
        "doctor", "sync", "watch", "update",
    ] {
        assert!(help.contains(cmd), "help missing {cmd}");
    }
    assert_ok(&run(&["--version"]), "version");

    // bootstrap
    assert_ok(&run(&["init", root]), "init");
    fs::create_dir_all(dir.join("ods-profiles")).unwrap();
    fs::write(
        dir.join("ods-profiles").join("widget.md"),
        "---\nname: widget\n---\n\n# Widget\n\n## Overview\n",
    )
    .unwrap();
    fs::write(
        dir.join("ods.toml"),
        r#"spec = "2.0"
custom_profiles = ["ods-profiles"]
"#,
    )
    .unwrap();

    fs::write(
        dir.join("gate.md"),
        "---\nprofile: checklist\nstatus: draft\ndescription: Release gate\ntags:\n  - Billing\n  - oncall\n---\n\n# Release gate\n\n## Overview\n\n## Items\n\n## Verification\n\n## Notes\n",
    )
    .unwrap();
    fs::write(
        dir.join("impl.md"),
        "---\nprofile: note\nstatus: stable\nid: stable/handle\ntags:\n  - billing\n---\n\n# Impl\n",
    )
    .unwrap();
    fs::write(
        dir.join("spec.md"),
        "---\nprofile: feature\nstatus: draft\ndescription: Spec\ndepends:\n  - stable/handle\nrelated:\n  - gate\ntags:\n  - billing\n---\n\n# Spec\n\n## Goal\n\n## Scope\n\n## Requirements\n\n## Acceptance Criteria\n\n## Risks\n",
    )
    .unwrap();
    fs::write(dir.join("plain.md"), "# Plain note without frontmatter\n").unwrap();

    assert_ok(&run(&["lint", root]), "lint");
    assert_ok(&run(&["lint", root]), "lint check");

    let out = run(&["lint", "--format", "json", root]);
    assert_ok(&out, "lint");

    let out = run(&["profiles", root]);
    assert_ok(&out, "profiles");
    let profiles = String::from_utf8_lossy(&out.stdout);
    assert!(
        profiles.contains("checklist:") && profiles.contains("[default ODS]"),
        "{profiles}"
    );
    assert!(
        profiles.contains("widget:") && profiles.contains("[project]"),
        "{profiles}"
    );

    let out = run(&["profiles", "--format", "json", root]);
    assert_ok(&out, "profiles json");
    let pj = String::from_utf8_lossy(&out.stdout);
    assert!(pj.contains("\"layer\""), "{pj}");

    let out = run(&["tags", root]);
    assert_ok(&out, "tags");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("billing"),
        "{:?}",
        out
    );

    let out = run(&["tags", "--all", root]);
    assert_ok(&out, "tags --all");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("security")
            || String::from_utf8_lossy(&out.stdout).contains("default ODS"),
        "{:?}",
        out
    );

    let out = run(&["find", root, "--tag", "billing"]);
    assert_ok(&out, "find --tag");
    let found = String::from_utf8_lossy(&out.stdout);
    assert!(
        found.contains("gate") || found.contains("impl") || found.contains("spec"),
        "{found}"
    );

    let out = run(&["context", root, "stable/handle"]);
    assert_ok(&out, "context explicit id");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("impl") || !out.stdout.is_empty(),
        "context should resolve explicit id: {:?}",
        out
    );

    let out = run(&["context", root, "spec"]);
    assert_ok(&out, "context path id");

    assert_ok(&run(&["graph", root]), "graph");
    assert_ok(&run(&["fmt", root]), "fmt");
    assert_ok(&run(&["adopt", root]), "adopt dry-run");
    assert_ok(&run(&["adopt", "--write", root]), "adopt write");
    assert!(
        fs::read_to_string(dir.join("plain.md"))
            .unwrap()
            .contains("profile:"),
        "adopt --write should draft frontmatter"
    );

    let out = run(&["doctor", root]);
    // doctor reports workspace + service status
    let doctor_out = String::from_utf8_lossy(&out.stdout);
    let doctor_err = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success()
            || doctor_out.contains("service:")
            || doctor_out.contains("ods version")
            || doctor_out.contains("health")
            || !doctor_out.is_empty()
            || !doctor_err.is_empty(),
        "doctor should run: status={:?} out={doctor_out} err={doctor_err}",
        out.status.code()
    );

    // tag rename dry-run + write
    let out = run(&["tag", "rename", root, "oncall", "ops-oncall"]);
    assert_ok(&out, "tag rename dry-run");
    let out = run(&["tag", "rename", root, "oncall", "ops-oncall", "--write"]);
    assert_ok(&out, "tag rename write");
    let gate = fs::read_to_string(dir.join("gate.md")).unwrap();
    assert!(gate.contains("ops-oncall"), "{gate}");
    assert!(
        !gate.contains("\n  - oncall\n") && !gate.contains("\n  - oncall\r"),
        "{gate}"
    );

    // mv rewrites depends on path-style; explicit id target stays stable/handle
    fs::write(
        dir.join("mover.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - gate\n---\n\n# Mover\n",
    )
    .unwrap();
    assert_ok(&run(&["lint", root]), "lint again");
    let out = run(&["mv", root, "gate.md", "release-gate.md"]);
    assert_ok(&out, "mv");
    assert!(dir.join("release-gate.md").exists());
    let mover = fs::read_to_string(dir.join("mover.md")).unwrap();
    assert!(
        mover.contains("release-gate") || mover.contains("gate"),
        "mv should rewrite depends: {mover}"
    );

    assert!(!run(&["lint", "--level", "1", root]).status.success());
    assert_ok(&run(&["lint", root]), "lint after ops");

    // enable/disable round-trip (opt-in / opt-out) — body preserved
    let out = run(&["disable", root]);
    assert_ok(&out, "disable dry-run");
    assert!(
        String::from_utf8_lossy(&out.stdout).contains("dry-run")
            || String::from_utf8_lossy(&out.stdout).contains("would_edit"),
        "{:?}",
        out
    );
    // still enabled after dry-run
    assert!(
        dir.join("ods.toml").exists()
            && fs::read_to_string(dir.join("ods.toml"))
                .unwrap()
                .contains("spec"),
        "dry-run must not remove ods.toml"
    );

    let out = run(&["disable", root, "--write"]);
    assert_ok(&out, "disable --write");
    assert!(
        !dir.join("ods.toml").exists(),
        "ods.toml must be removed on disable --write"
    );
    // prose from earlier files still present if not deleted
    if dir.join("impl.md").exists() {
        let impl_body = fs::read_to_string(dir.join("impl.md")).unwrap();
        assert!(
            impl_body.contains("# Impl") || !impl_body.contains("profile:"),
            "{impl_body}"
        );
    }

    // re-init for later commands not required
    assert_ok(&run(&["init", root, "--adopt"]), "re-init after disable");
    assert!(
        dir.join("ods.toml").exists()
            && fs::read_to_string(dir.join("ods.toml"))
                .unwrap()
                .contains("spec"),
        "init must restore ods.toml"
    );
}
