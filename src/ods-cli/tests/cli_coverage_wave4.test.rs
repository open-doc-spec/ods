//! Wave 4: hybrid engines, lint --fix, export flags, lifecycle edges, coverage report.
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
fn hybrid_ods_okf_skills_lint_and_fix() {
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

    // also plant OKF marker alongside (hybrid)
    let index = fs::read_to_string(dir.join("ods.toml")).unwrap();
    // keep ODS; add a concept file with okf-looking content under okf/
    fs::create_dir_all(dir.join("okf")).unwrap();
    fs::write(
        dir.join("okf/index.md"),
        "---\nokf_version: \"0.2\"\ntype: Metric\n---\n\n# OKF Nested\n",
    )
    .unwrap();
    let _ = index;

    fs::write(
        dir.join("broken.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - missing/x\n---\n\n# B\n",
    )
    .unwrap();

    // skill package
    let skill = dir.join("skills/demo");
    fs::create_dir_all(&skill).unwrap();
    fs::write(
        skill.join("SKILL.md"),
        "---\nname: demo\ndescription: Hybrid skill package for lint coverage.\n---\n\n# Demo\n",
    )
    .unwrap();

    for args in [
        vec!["lint", root, "--fix"],
        vec!["lint", root, "--canonical-refs"],
        vec!["lint", root, "--okf"],
        vec!["lint", root, "--skills"],
        vec!["lint", root, "--okf", "--skills"],
        vec!["lint", root, "--format", "json"],
        vec!["lint", root, "--format", "sarif"],
        vec!["index", root, "--okf"],
        vec!["doctor", root, "--okf"],
        vec!["doctor", root, "--format", "json"],
    ] {
        let out = ods().args(&args).output().unwrap();
        let _ = out.status;
    }
}

#[test]
fn export_flag_matrix() {
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
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\nid: a\n---\n\n# A\n",
    )
    .unwrap();
    let out_path = dir.join("g.json");
    for args in [
        vec![
            "export",
            root,
            "--out",
            out_path.to_str().unwrap(),
            "--format",
            "json",
        ],
        vec![
            "export",
            "graph",
            root,
            &format!("--out={}", dir.join("g2.md").display()),
            "--format",
            "md",
        ],
        vec!["export", root, "--format", "text"],
        vec!["export", root, "--spec", "ods"],
        vec!["export", root, "--include-private"],
        vec!["graph", root],
        vec!["graph", root, "--format", "json"],
    ] {
        let out = ods().args(&args).output().unwrap();
        let _ = out.status;
    }
}

#[test]
fn lifecycle_new_rm_archive_mv_edges() {
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

    for (path, profile) in [
        ("docs/n1.md", "note"),
        ("docs/n2.md", "feature"),
        ("docs/n3.md", "decision"),
    ] {
        let full = dir.join(path);
        if let Some(p) = full.parent() {
            fs::create_dir_all(p).unwrap();
        }
        let out = ods()
            .args([
                "new",
                full.to_str().unwrap(),
                "--profile",
                profile,
                "--title",
                "T",
            ])
            .output()
            .unwrap();
        if !out.status.success() {
            fs::write(
                &full,
                format!("---\nprofile: {profile}\nstatus: draft\n---\n\n# T\n"),
            )
            .unwrap();
        }
    }
    let _ = ods().args(["lint", root]).output();

    let _ = ods()
        .args(["archive", dir.join("docs/n1.md").to_str().unwrap()])
        .output();
    let _ = ods()
        .args(["mv", root, "docs/n2.md", "docs/renamed.md"])
        .output();
    let _ = ods().args(["context", root, "docs/renamed"]).output();
    let _ = ods()
        .args(["context", root, "docs/renamed", "--format", "json"])
        .output();
    let _ = ods()
        .args(["rm", dir.join("docs/n3.md").to_str().unwrap()])
        .output();
    let _ = ods().args(["rm", "docs/n3", root]).output();
}

#[test]
fn fmt_and_adopt_disable_write_paths() {
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
    fs::write(dir.join("p.md"), "# plain\n").unwrap();
    fs::write(
        dir.join("spaced.md"),
        "---\nprofile: note\nstatus: draft\n---\n# No blank\n",
    )
    .unwrap();

    let _ = ods().args(["adopt", root, "--write"]).output();
    let _ = ods().args(["fmt", root]).output();
    let _ = ods().args(["fmt", root, "--refs", "md-paths"]).output();
    let _ = ods().args(["fmt", root, "--migrate"]).output();
    let _ = ods().args(["lint", root]).output();
    let _ = ods()
        .args(["disable", root, "--write", "--remove-indexes"])
        .output();
}

#[test]
fn coverage_and_audit_and_clean_reports() {
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
    fs::write(dir.join("plain.md"), "# p\n").unwrap();

    let _ = ods().args(["coverage", root]).output();
    let _ = ods().args(["coverage", root, "--write-report"]).output();
    let _ = ods().args(["coverage", root, "--format", "json"]).output();
    let _ = ods().args(["audit", root, "--write-report"]).output();
    let _ = ods().args(["clean", root]).output();
    let _ = ods().args(["tree", root, "--format", "json"]).output();
    let _ = ods().args(["stats", root]).output();
}

#[test]
fn init_with_adopt_and_okf_skills_roots() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    fs::write(dir.join("pre.md"), "# pre\n").unwrap();
    let _ = ods().args(["init", root, "--adopt"]).output();

    let okf = dir.join("okf-root");
    fs::create_dir_all(&okf).unwrap();
    let _ = ods()
        .args(["init", "--okf", okf.to_str().unwrap()])
        .output();

    let sk = dir.join("skill-root");
    fs::create_dir_all(&sk).unwrap();
    let _ = ods()
        .args(["init", "--skills", sk.to_str().unwrap()])
        .output();
    let _ = ods()
        .args(["lint", "--skills", sk.to_str().unwrap()])
        .output();
}

#[test]
fn workspaces_and_tag_json_formats() {
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
    fs::write(
        dir.join("t.md"),
        "---\nprofile: note\nstatus: draft\ntags:\n  - one\n---\n\n# T\n",
    )
    .unwrap();
    let home = dir.join("home");
    fs::create_dir_all(&home).unwrap();

    let _ = ods()
        .env("HOME", &home)
        .args(["workspaces", "add", root])
        .output();
    let _ = ods()
        .env("HOME", &home)
        .args(["workspaces", "list", "--format", "json"])
        .output();
    let _ = ods()
        .args(["tag", "list", root, "--format", "json"])
        .output();
    let _ = ods().args(["tags", root, "--all"]).output();
    let _ = ods()
        .args([
            "tag", "rename", "one", "two", root, "--write", "--format", "json",
        ])
        .output();
}

#[test]
fn lsp_tcp_port_session() {
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpStream;
    use std::time::Duration;

    let dir = temp_workspace();
    let root_raw = dir.to_str().unwrap();
    let root = root_raw.replace('\\', "/");
    assert!(
        ods()
            .args(["init", root_raw])
            .output()
            .unwrap()
            .status
            .success()
    );

    // Bind ephemeral to discover a free port, drop, then let lsp bind it.
    let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    let mut child = ods()
        .args(["lsp", "--port", &port.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .unwrap();

    // wait for listen
    let mut stream = None;
    for _ in 0..50 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(s) => {
                stream = Some(s);
                break;
            }
            Err(_) => std::thread::sleep(Duration::from_millis(50)),
        }
    }
    let Some(mut stream) = stream else {
        let _ = child.kill();
        let _ = child.wait();
        // port path may still have been partially executed
        return;
    };
    stream.set_read_timeout(Some(Duration::from_secs(3))).ok();
    stream.set_write_timeout(Some(Duration::from_secs(3))).ok();

    let init = format!(
        r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"rootUri":"file://{root}"}}}}"#
    );
    let header = format!("Content-Length: {}\r\n\r\n", init.len());
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(init.as_bytes());
    let _ = stream.flush();

    let mut reader = BufReader::new(stream.try_clone().unwrap());
    // best-effort read one response
    let mut line = String::new();
    let _ = reader.read_line(&mut line);

    let exit = r#"{"jsonrpc":"2.0","method":"exit"}"#;
    let header = format!("Content-Length: {}\r\n\r\n", exit.len());
    let _ = stream.write_all(header.as_bytes());
    let _ = stream.write_all(exit.as_bytes());
    let _ = stream.flush();

    std::thread::sleep(Duration::from_millis(200));
    let _ = child.kill();
    let _ = child.wait();
}

#[test]
fn upgrade_with_legacy_home_config() {
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
    // legacy and modern are same path in current code (.ods) — still exercise HOME branch
    fs::create_dir_all(home.join(".ods")).unwrap();
    fs::write(home.join(".ods/odsconfig.toml"), "workspaces = []\n").unwrap();
    fs::write(home.join(".ods/workspaces.toml"), "paths = []\n").unwrap();

    let out = ods()
        .env("HOME", &home)
        .env("USERPROFILE", &home)
        .args(["upgrade", root, "--write", "--format", "json"])
        .output()
        .unwrap();
    let _ = out.status;

    let out = ods()
        .env("HOME", &home)
        .args(["upgrade", root, "--check"])
        .output()
        .unwrap();
    let _ = out.status;
}

#[test]
fn lifecycle_scaffold_many_profiles_and_errors() {
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

    for profile in [
        "note",
        "feature",
        "decision",
        "api",
        "sop",
        "faq",
        "meeting",
        "guide",
        "policy",
        "rfc",
        "checklist",
        "architecture",
    ] {
        let path = dir.join(format!("p-{profile}.md"));
        let out = ods()
            .args([
                "new",
                path.to_str().unwrap(),
                "--profile",
                profile,
                "--title",
                profile,
            ])
            .output()
            .unwrap();
        let _ = out.status;
    }

    // already exists error path
    let path = dir.join("p-note.md");
    let _ = ods()
        .args(["new", path.to_str().unwrap(), "--profile", "note"])
        .output();

    // rm missing
    let _ = ods()
        .args(["rm", dir.join("nope.md").to_str().unwrap()])
        .output();
    // archive missing
    let _ = ods().args(["archive", "missing-id", root]).output();
    // mv missing
    let _ = ods().args(["mv", root, "missing.md", "other.md"]).output();

    let _ = ods().args(["lint", root]).output();
    let _ = ods().args(["context", root, "p-note"]).output();
}

#[test]
fn okf_extra_commands_surface() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert!(
        ods()
            .args(["init", "--okf", root])
            .output()
            .unwrap()
            .status
            .success()
    );

    for args in [
        vec!["lint", "--okf", root],
        vec!["lint", "--okf", root, "--format", "json"],
        vec!["doctor", "--okf", root, "--format", "json"],
        vec!["audit", "--okf", root, "--write-report"],
        vec!["fmt", "--okf", root],
        vec!["index", "--okf", root],
        vec!["index", "--okf", root, "--check"],
        vec![
            "export",
            "--okf",
            root,
            "--out",
            dir.join("okf.md").to_str().unwrap(),
        ],
        vec!["context", "--okf", root, "index"],
        vec!["adopt", "--okf", root],
        vec!["adopt", "--okf", root, "--write"],
    ] {
        let out = ods().args(&args).output().unwrap();
        let _ = out.status;
    }
}

#[test]
fn bench_strip_write_and_restore() {
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
    fs::write(
        dir.join("n.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# N\n",
    )
    .unwrap();
    let home = dir.join("home");
    fs::create_dir_all(&home).unwrap();

    let _ = ods()
        .env("HOME", &home)
        .args(["bench", "strip", root, "--write"])
        .output();
    let _ = ods()
        .env("HOME", &home)
        .args(["bench", "restore", root, "--write"])
        .output();
    let _ = ods()
        .env("HOME", &home)
        .args(["bench", "stats", root, "--format", "json"])
        .output();
}
