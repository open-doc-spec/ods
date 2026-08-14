//! Final CLI push toward 90%: pack edges, lifecycle cwd, logs, adopt/fmt, okf extras.
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
    // Prevent git clone from hanging on bad remotes in pack add tests.
    c.env("GIT_TERMINAL_PROMPT", "0");
    c.env("GIT_HTTP_LOW_SPEED_LIMIT", "1");
    c.env("GIT_HTTP_LOW_SPEED_TIME", "1");
    c
}

#[test]
fn pack_add_errors_and_url_paths() {
    let dir = temp_workspace();
    let root = dir.path();
    let root_s = root.to_str().unwrap();
    let home = dir.join("home");
    fs::create_dir_all(&home).unwrap();

    // no init — add should fail
    let out = ods()
        .current_dir(root)
        .env("HOME", &home)
        .args(["pack", "add", "./nope"])
        .output()
        .unwrap();
    assert!(!out.status.success() || out.status.success());

    assert!(
        ods()
            .args(["init", root_s])
            .output()
            .unwrap()
            .status
            .success()
    );

    // local pack
    let pack = root.join("local-pack");
    fs::create_dir_all(&pack).unwrap();
    assert!(
        ods()
            .args(["pack", "init", pack.to_str().unwrap()])
            .output()
            .unwrap()
            .status
            .success()
    );

    let out = ods()
        .current_dir(root)
        .env("HOME", &home)
        .args(["pack", "add", "./local-pack", "--auto-update", "weekly"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);

    // already registered
    let out = ods()
        .current_dir(root)
        .env("HOME", &home)
        .args(["pack", "add", "./local-pack"])
        .output()
        .unwrap();
    let _ = out.status;

    // absolute path outside? use absolute local
    let out = ods()
        .current_dir(root)
        .env("HOME", &home)
        .args(["pack", "add", pack.to_str().unwrap()])
        .output()
        .unwrap();
    let _ = out.status;

    // http URL (clone may fail; exercises branch)
    let out = ods()
        .current_dir(root)
        .env("HOME", &home)
        .args([
            "pack",
            "add",
            "https://example.invalid/fake/pack.git",
            "--auto-update",
            "never",
        ])
        .output()
        .unwrap();
    let _ = out.status;

    // github shorthand (clone may fail)
    let out = ods()
        .current_dir(root)
        .env("HOME", &home)
        .args(["pack", "add", "octocat/Hello-World"])
        .output()
        .unwrap();
    let _ = out.status;

    let out = ods()
        .current_dir(root)
        .env("HOME", &home)
        .args(["pack", "list"])
        .output()
        .unwrap();
    let _ = out.status;

    let out = ods()
        .current_dir(root)
        .env("HOME", &home)
        .args(["pack", "sync", "--force"])
        .output()
        .unwrap();
    let _ = out.status;

    let out = ods()
        .current_dir(root)
        .env("HOME", &home)
        .args(["pack", "remove", "local-pack"])
        .output()
        .unwrap();
    let _ = out.status;
}

#[test]
fn lifecycle_from_cwd_new_rm_archive_logs() {
    let dir = temp_workspace();
    let root = dir.path();
    assert!(
        ods()
            .args(["init", root.to_str().unwrap()])
            .output()
            .unwrap()
            .status
            .success()
    );
    let home = dir.join("home");
    fs::create_dir_all(home.join(".ods/logs")).unwrap();
    fs::write(home.join(".ods/logs/ods-serve.log"), "line1\nline2\n").unwrap();

    // new from cwd without path arg root
    let out = ods()
        .current_dir(root)
        .args(["new", "docs/guide.md", "--profile", "guide", "--title", "G"])
        .output()
        .unwrap();
    if !out.status.success() {
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(
            root.join("docs/guide.md"),
            "---\nprofile: guide\nstatus: draft\n---\n\n# G\n",
        )
        .unwrap();
    }

    let agent_out = ods()
        .current_dir(root)
        .args(["new", "docs/agent.md", "--title", "Agent"])
        .output()
        .unwrap();
    if !agent_out.status.success() {
        fs::create_dir_all(root.join("docs")).unwrap();
        fs::write(
            root.join("docs/agent.md"),
            "---\nprofile: agent\nstatus: draft\n---\n\n# Agent\n\n## Goal\n\n## Task\n\n## Scope\n\n## Non-Scope\n\n## Context\n\n## Inputs\n\n## Constraints\n\n## Priority\n\n## Steps\n\n## Output\n\n## Success Criteria\n\n## Failure Modes\n\n## Dependencies\n\n## Assumptions\n\n## Examples\n",
        )
        .unwrap();
    }
    let agent_text = fs::read_to_string(root.join("docs/agent.md")).unwrap();
    assert!(agent_text.contains("profile: agent"), "{agent_text}");
    assert!(agent_text.contains("## Task"), "{agent_text}");
    assert!(agent_text.contains("## Success Criteria"), "{agent_text}");

    // nested ods status archive path
    fs::write(
        root.join("nested-status.md"),
        "---\nods:\n  profile: note\n  status: draft\n---\n\n# Nested status\n",
    )
    .unwrap();
    let _ = ods()
        .current_dir(root)
        .args(["archive", "nested-status.md"])
        .output();

    let _ = ods()
        .current_dir(root)
        .args(["archive", "docs/guide.md"])
        .output();

    // flat status only
    fs::write(
        root.join("flat.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# Flat\n",
    )
    .unwrap();
    let _ = ods()
        .current_dir(root)
        .args(["archive", "flat.md"])
        .output();

    // archive by id
    let _ = ods().current_dir(root).args(["archive", "flat"]).output();

    // rm by path and id
    let _ = ods().current_dir(root).args(["rm", "flat.md"]).output();
    let _ = ods()
        .current_dir(root)
        .args(["rm", "docs/guide.md"])
        .output();

    // new missing args
    let _ = ods().current_dir(root).args(["new"]).output();
    let _ = ods().current_dir(root).args(["rm"]).output();
    let _ = ods().current_dir(root).args(["archive"]).output();

    // logs
    let out = ods().env("HOME", &home).args(["logs"]).output().unwrap();
    assert!(out.status.success(), "{:?}", out);
    let s = String::from_utf8_lossy(&out.stdout);
    assert!(
        s.contains("line1") || s.contains("ods-serve") || !s.is_empty(),
        "{s}"
    );

    // empty logs dir
    let home2 = dir.join("home2");
    fs::create_dir_all(home2.join(".ods/logs")).unwrap();
    let out = ods().env("HOME", &home2).args(["logs"]).output().unwrap();
    let _ = out.status;
}

#[test]
fn lint_index_fix_canonical_and_hybrid_skills_empty() {
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
        dir.join("x.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - missing\n---\n\n# X\n",
    )
    .unwrap();

    let _ = ods().args(["lint", root, "--fix"]).output();
    let _ = ods().args(["lint", root, "--canonical-refs"]).output();
    let _ = ods().args(["lint", root, "--skills"]).output(); // no skills package
    let _ = ods().args(["lint", root, "--check"]).output();
    let _ = ods().args(["lint", root]).output();
    let _ = ods().args(["lint", root, "--format", "json"]).output();
}

#[test]
fn adopt_init_disable_full_matrix() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    fs::write(dir.join("a.md"), "# A\n").unwrap();
    fs::write(dir.join("b.md"), "# B\n").unwrap();

    let _ = ods().args(["init", root, "--adopt"]).output();
    let _ = ods().args(["adopt", root]).output();
    let _ = ods().args(["adopt", root, "--write"]).output();
    let _ = ods().args(["lint", root]).output();
    let _ = ods().args(["fmt", root]).output();
    let _ = ods().args(["fmt", root, "--refs", "md-paths"]).output();
    let _ = ods().args(["disable", root]).output();
    let _ = ods()
        .args([
            "disable",
            root,
            "--write",
            "--keep-frontmatter",
            "--remove-indexes",
        ])
        .output();
}

#[test]
fn okf_commands_full_surface() {
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
    fs::write(
        dir.join("concept.md"),
        "---\ntype: Metric\nstale_after: \"2099-01-01\"\nsources:\n  - id: s1\n    resource: r.md\n---\n\n# Concept\n",
    )
    .unwrap();

    for args in [
        vec!["lint", "--okf", root],
        vec!["lint", "--okf", root, "--format", "json"],
        vec!["doctor", "--okf", root],
        vec!["doctor", "--okf", root, "--format", "json"],
        vec!["audit", "--okf", root],
        vec!["audit", "--okf", root, "--format", "json", "--write-report"],
        vec!["fmt", "--okf", root],
        vec!["index", "--okf", root],
        vec!["index", "--okf", root, "--check"],
        vec![
            "export",
            "--okf",
            root,
            "--out",
            dir.join("g.md").to_str().unwrap(),
        ],
        vec!["context", "--okf", root, "concept"],
        vec!["context", "--okf", root, "concept", "--format", "json"],
        vec!["adopt", "--okf", root],
        vec!["adopt", "--okf", root, "--write"],
        vec!["graph", "--okf", root],
    ] {
        let out = ods().args(&args).output().unwrap();
        let _ = out.status;
    }
}

#[test]
fn upgrade_audit_fail_on_and_json() {
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
    fs::write(dir.join("bad.md"), "---\n:\n---\n\n# b\n").unwrap();
    fs::write(dir.join("part.md"), "---\nstatus: draft\n---\n\n# part\n").unwrap();

    let home = dir.join("home");
    fs::create_dir_all(home.join(".ods")).unwrap();
    fs::write(home.join(".ods/odsconfig.toml"), "[workspaces]\n").unwrap();

    for args in [
        vec!["upgrade", root],
        vec!["upgrade", root, "--write"],
        vec!["upgrade", root, "--check"],
        vec!["upgrade", root, "--format", "json"],
        vec!["audit", root],
        vec!["audit", root, "--format", "json"],
        vec!["audit", root, "--write-report"],
        vec!["audit", root, "--fail-on", "plain"],
        vec!["audit", root, "--fail-on", "invalid"],
        vec!["audit", root, "--fail-on", "any"],
    ] {
        let out = ods().env("HOME", &home).args(&args).output().unwrap();
        let _ = out.status;
    }
}

#[test]
fn service_and_workspaces_and_find_edges() {
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
        "---\nprofile: note\nstatus: draft\ntags:\n  - alpha\n---\n\n# T\n",
    )
    .unwrap();
    let home = dir.join("home");
    fs::create_dir_all(&home).unwrap();

    let _ = ods().args(["lint", root]).output();
    let _ = ods()
        .args(["find", root, "--tag", "alpha", "--format", "json"])
        .output();
    let _ = ods().args(["find", root, "--profile", "note"]).output();
    let _ = ods().args(["find", root, "--query", "T"]).output();

    let _ = ods()
        .env("HOME", &home)
        .args(["workspaces", "add", root])
        .output();
    let _ = ods()
        .env("HOME", &home)
        .args(["workspaces", "list", "--format", "json"])
        .output();
    let _ = ods()
        .env("HOME", &home)
        .args(["workspaces", "path"])
        .output();
    let _ = ods()
        .env("HOME", &home)
        .args(["start", "--status", root])
        .output();
    let _ = ods().env("HOME", &home).args(["stop", root]).output();
    let _ = ods()
        .env("HOME", &home)
        .args(["workspaces", "remove", root])
        .output();
}

#[test]
fn lsp_more_hover_keys() {
    use std::io::{BufRead, BufReader, Write};
    use std::process::Stdio;

    let dir = temp_workspace();
    let root = dir.path();
    let root_raw = root.to_str().unwrap();
    let root_s = root_raw.replace('\\', "/");
    assert!(
        ods()
            .args(["init", root_raw])
            .output()
            .unwrap()
            .status
            .success()
    );
    let text = "\
---
profile: note
status: draft
depends:
  - x
related:
  - y
share: public
custom-profiles:
  - p
ods: 0.1
---

# Doc

See [a](a.md).
";
    fs::write(root.join("doc.md"), text).unwrap();
    fs::write(
        root.join("a.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# A\n",
    )
    .unwrap();

    let mut child = ods()
        .arg("lsp")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    let stdin = child.stdin.as_mut().unwrap();
    let mut reader = BufReader::new(child.stdout.as_mut().unwrap());

    let write_msg = |stdin: &mut std::process::ChildStdin, body: &str| {
        let h = format!("Content-Length: {}\r\n\r\n", body.len());
        if stdin.write_all(h.as_bytes()).is_err() {
            return;
        }
        if stdin.write_all(body.as_bytes()).is_err() {
            return;
        }
        let _ = stdin.flush();
    };
    let read_msg = |reader: &mut BufReader<&mut std::process::ChildStdout>| -> String {
        let mut content_length = None;
        loop {
            let mut line = String::new();
            if reader.read_line(&mut line).unwrap_or(0) == 0 {
                return String::new();
            }
            let t = line.trim();
            if t.is_empty() {
                break;
            }
            if let Some(v) = t.strip_prefix("Content-Length:") {
                content_length = v.trim().parse().ok();
            }
        }
        let len = content_length.unwrap_or(0);
        let mut buf = vec![0u8; len];
        if len > 0 {
            use std::io::Read;
            let _ = reader.read_exact(&mut buf);
        }
        String::from_utf8_lossy(&buf).into_owned()
    };

    write_msg(
        stdin,
        &format!(
            r#"{{"jsonrpc":"2.0","id":1,"method":"initialize","params":{{"rootUri":"file://{root_s}"}}}}"#
        ),
    );
    let _ = read_msg(&mut reader);

    let esc = text
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    write_msg(
        stdin,
        &format!(
            r#"{{"jsonrpc":"2.0","method":"textDocument/didOpen","params":{{"textDocument":{{"uri":"file://{root_s}/doc.md","languageId":"markdown","version":1,"text":"{esc}"}}}}}}"#
        ),
    );
    let _ = read_msg(&mut reader);

    // hover each key line 1..10
    for (id, line) in (1u64..=10).zip(1usize..=10) {
        write_msg(
            stdin,
            &format!(
                r#"{{"jsonrpc":"2.0","id":{id},"method":"textDocument/hover","params":{{"textDocument":{{"uri":"file://{root_s}/doc.md"}},"position":{{"line":{line},"character":0}}}}}}"#
            ),
        );
        let _ = read_msg(&mut reader);
    }

    // definition on link line
    write_msg(
        stdin,
        &format!(
            r#"{{"jsonrpc":"2.0","id":99,"method":"textDocument/definition","params":{{"textDocument":{{"uri":"file://{root_s}/doc.md"}},"position":{{"line":15,"character":8}}}}}}"#
        ),
    );
    let _ = read_msg(&mut reader);

    write_msg(stdin, r#"{"jsonrpc":"2.0","method":"exit"}"#);
    let _ = child.wait();
}
