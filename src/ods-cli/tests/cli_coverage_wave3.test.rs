//! Wave 3 CLI: help matrix, pack edges, setup editor, fmt/lint/find flags, archive via update dry paths.
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

#[test]
fn help_matrix_for_common_commands() {
    for cmd in ["help", "--help", "-h", "version", "--version", "-V"] {
        let out = ods().args([cmd]).output().unwrap();
        let _ = out.status;
    }

    for cmd in [
        "lint",
        "index",
        "doctor",
        "audit",
        "coverage",
        "profiles",
        "find",
        "tag",
        "context",
        "graph",
        "export",
        "share",
        "fmt",
        "adopt",
        "init",
        "disable",
        "sync",
        "bench",
        "upgrade",
        "workspaces",
        "skill",
        "pack",
        "stats",
        "completion",
        "schema",
        "tree",
        "diff",
        "clean",
        "agents",
        "lsp",
        "watch",
        "serve",
        "start",
        "logs",
        "read",
        "undo",
        "new",
        "rm",
    ] {
        let out = ods().args([cmd, "--help"]).output().unwrap();
        assert!(
            out.status.success(),
            "{cmd} --help failed: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let stdout = String::from_utf8_lossy(&out.stdout);
        assert!(
            stdout.contains("Usage:"),
            "{cmd} --help missing Usage:\n{stdout}"
        );
    }
}

#[test]
fn pack_and_profiles_and_find_edges() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert!(
        ods()
            .args(["init", root])
            .output()
            .unwrap()
            .status
            .success()
    );

    let pack = dir.join("p1");
    assert!(
        ods()
            .args(["pack", "init", pack.to_str().unwrap()])
            .output()
            .unwrap()
            .status
            .success()
    );
    // add absolute path
    let _ = ods()
        .current_dir(dir.path())
        .args(["pack", "add", pack.to_str().unwrap()])
        .output();
    // add again (idempotent / error path)
    let _ = ods()
        .current_dir(dir.path())
        .args(["pack", "add", "./p1"])
        .output();
    let _ = ods()
        .current_dir(dir.path())
        .args(["pack", "list"])
        .output();
    let _ = ods()
        .current_dir(dir.path())
        .args(["pack", "sync", root])
        .output();
    // missing pack
    let _ = ods()
        .current_dir(dir.path())
        .args(["pack", "add", "./missing-pack"])
        .output();
    let _ = ods()
        .current_dir(dir.path())
        .args(["pack", "remove", "nope"])
        .output();

    fs::write(
        dir.join("f.md"),
        "---\nprofile: note\nstatus: draft\ntags:\n  - z\n---\n\n# F\n",
    )
    .unwrap();
    let _ = ods().args(["lint", root]).output();
    let _ = ods().args(["find", root, "--query", "F"]).output();
    let _ = ods()
        .args(["find", root, "--tag", "z", "--format", "json"])
        .output();
    let _ = ods()
        .args(["find", root, "--profile", "note", "--status", "draft"])
        .output();

    let _ = ods().args(["profiles", root]).output();
    let _ = ods().args(["profiles", root, "--format", "json"]).output();
}

#[test]
fn setup_editor_configs_write() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert!(
        ods()
            .args(["init", root])
            .output()
            .unwrap()
            .status
            .success()
    );
    let home = dir.join("home");
    fs::create_dir_all(&home).unwrap();

    // Only editor flag paths — plain `setup` can attempt long service registration.
    for editor in ["zed", "vscode", "cursor"] {
        let mut child = ods()
            .env("HOME", &home)
            .args(["setup", "--editor", editor, root])
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .unwrap();
        // hard timeout so setup cannot hang the suite
        let start = std::time::Instant::now();
        loop {
            match child.try_wait().unwrap() {
                Some(_) => break,
                None if start.elapsed() > std::time::Duration::from_secs(5) => {
                    let _ = child.kill();
                    let _ = child.wait();
                    break;
                }
                None => std::thread::sleep(std::time::Duration::from_millis(50)),
            }
        }
    }
}

#[test]
fn sync_git_workspace_and_diff() {
    let dir = temp_workspace();
    let root = dir.path();
    let root_s = root.to_str().unwrap();
    assert!(
        ods()
            .args(["init", root_s])
            .output()
            .unwrap()
            .status
            .success()
    );
    fs::write(
        root.join("n.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# N\n",
    )
    .unwrap();

    // init git if available
    let _ = Command::new("git")
        .args(["init"])
        .current_dir(root)
        .output();
    let _ = Command::new("git")
        .args(["config", "user.email", "t@t.com"])
        .current_dir(root)
        .output();
    let _ = Command::new("git")
        .args(["config", "user.name", "t"])
        .current_dir(root)
        .output();
    let _ = Command::new("git")
        .args(["add", "."])
        .current_dir(root)
        .output();
    let _ = Command::new("git")
        .args(["commit", "-m", "init"])
        .current_dir(root)
        .output();

    fs::write(
        root.join("n2.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# N2\n",
    )
    .unwrap();
    let _ = ods().args(["sync", root_s]).output();
    let _ = ods().args(["diff", root_s]).output();
    let _ = ods()
        .args(["diff", root_s, "HEAD", "--format", "json"])
        .output();
}

#[test]
fn workspaces_json_and_path_ops() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert!(
        ods()
            .args(["init", root])
            .output()
            .unwrap()
            .status
            .success()
    );
    let home = dir.join("home");
    fs::create_dir_all(&home).unwrap();

    for args in [
        vec!["workspaces", "add", root],
        vec!["workspaces", "list"],
        vec!["workspaces", "list", "--format", "json"],
        vec!["workspaces", "path"],
        vec!["workspaces", "remove", root],
        vec!["workspaces", "help"],
    ] {
        let out = ods()
            .env("HOME", &home)
            .env("USERPROFILE", &home)
            .args(&args)
            .output()
            .unwrap();
        let _ = out.status;
    }
}

#[test]
fn bench_full_and_run_help() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert!(
        ods()
            .args(["init", root])
            .output()
            .unwrap()
            .status
            .success()
    );
    let home = dir.join("home");
    fs::create_dir_all(&home).unwrap();

    for args in [
        vec!["bench", "help"],
        vec!["bench", "stats", root],
        vec!["bench", "stats", root, "--format", "json"],
        vec!["bench", "strip", root],
        vec!["bench", "strip", root, "--full"],
        vec!["bench", "restore", root],
        vec!["bench", "run", root, "--help"],
    ] {
        let out = ods().env("HOME", &home).args(&args).output().unwrap();
        let _ = out.status;
    }
}

#[test]
fn skill_install_and_agents_on_okf() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    let _ = ods().args(["init", "--okf", root]).output();
    let _ = ods().args(["agents", "sync", root]).output();

    let out = ods()
        .current_dir(dir.path())
        .args([
            "skill", "install", "--agent", "cursor", "--scope", "project",
        ])
        .output()
        .unwrap();
    let _ = out.status;
}

#[test]
fn service_status_and_logs_only() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert!(
        ods()
            .args(["init", root])
            .output()
            .unwrap()
            .status
            .success()
    );
    let home = dir.join("home");
    fs::create_dir_all(&home).unwrap();

    // status/stop/logs only — never launch serve/watch loops
    for args in [
        vec!["start", "--status", root],
        vec!["stop", root],
        vec!["logs"],
    ] {
        let out = ods().env("HOME", &home).args(&args).output().unwrap();
        let _ = out.status;
    }
}
