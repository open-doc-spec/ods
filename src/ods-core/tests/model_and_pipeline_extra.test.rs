//! model ods pin + pipeline discover/gitignore coverage.
use ods_core::{
    CodeRole, apply_document_upserts, discover_markdown_paths, load_options_graph, load_workspace,
    load_workspace_with_options, parse_document_text, parse_paths_parallel,
};
use std::fs;
use tempfile::tempdir;

#[test]
fn discover_gitignore_and_excluded_roots() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(root.join(".gitignore"), "ignored_dir/\n*.tmp.md\n").unwrap();
    fs::create_dir_all(root.join("ignored_dir")).unwrap();
    fs::create_dir_all(root.join("keep")).unwrap();
    fs::create_dir_all(root.join("excluded_root")).unwrap();
    fs::write(root.join("keep/a.md"), "# A\n").unwrap();
    fs::write(root.join("ignored_dir/b.md"), "# B\n").unwrap();
    fs::write(root.join("x.tmp.md"), "# T\n").unwrap();
    fs::write(root.join("excluded_root/c.md"), "# C\n").unwrap();

    let gitignore = vec!["ignored_dir".into(), "*.tmp.md".into()];
    // discover does its own gitignore matching via patterns list
    let paths =
        discover_markdown_paths(root, &[root.join("excluded_root")], &gitignore, &[]).unwrap();
    assert!(paths.iter().any(|p| p.ends_with("a.md")));
    assert!(
        !paths
            .iter()
            .any(|p| p.to_string_lossy().contains("ignored_dir"))
    );
    assert!(
        !paths
            .iter()
            .any(|p| p.to_string_lossy().contains("excluded_root"))
    );
}

#[test]
fn load_graph_options_and_parallel() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(
        root.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
    )
    .unwrap();
    fs::write(
        root.join("n.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# N\n\n## Overview\n",
    )
    .unwrap();
    let ws = load_workspace_with_options(root, load_options_graph()).unwrap();
    assert!(ws.documents.len() >= 2);
    let note = ws
        .documents
        .iter()
        .find(|d| d.path.ends_with("n.md"))
        .unwrap();
    assert!(note.body.is_empty()); // graph load drops note bodies
    let index = ws
        .documents
        .iter()
        .find(|d| d.path.ends_with("index.md"))
        .unwrap();
    assert!(index.body.is_empty()); // graph load drops all bodies

    let paths: Vec<_> = ws.documents.iter().map(|d| d.path.clone()).collect();
    let docs = parse_paths_parallel(root, &paths, true).unwrap();
    assert_eq!(docs.len(), paths.len());
    let _ = load_workspace(root).unwrap();
}

#[test]
fn code_role_parse_and_as_str_all_variants() {
    for (s, role) in [
        ("entrypoint", CodeRole::Entrypoint),
        ("implementation", CodeRole::Implementation),
        ("test", CodeRole::Test),
        ("schema", CodeRole::Schema),
        ("migration", CodeRole::Migration),
        ("config", CodeRole::Config),
        ("infrastructure", CodeRole::Infrastructure),
        ("pipeline", CodeRole::Pipeline),
    ] {
        assert_eq!(CodeRole::parse(s), Some(role));
        assert_eq!(role.as_str(), s);
    }
    assert_eq!(CodeRole::parse("nope"), None);
    assert_eq!(CodeRole::parse(" ENTRYPOINT "), Some(CodeRole::Entrypoint));
}

#[test]
fn apply_upsert_when_by_path_stale() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(
        root.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
    )
    .unwrap();
    let mut ws = load_workspace(root).unwrap();
    let p = root.join("ghost.md");
    let doc = parse_document_text(
        root,
        p.clone(),
        "---\nprofile: note\nstatus: draft\n---\n\n# G\n",
        true,
    );
    // Insert without going through by_path (stale map): push then apply upsert hits position branch
    ws.documents.push(doc.clone());
    ws.by_path.clear();
    apply_document_upserts(&mut ws, vec![doc]);
    assert!(ws.documents.iter().filter(|d| d.path == p).count() >= 1);
}

#[test]
fn parse_paths_parallel_jobs_and_error() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    let file = root.join("test.md");
    fs::write(&file, "# Test\n").unwrap();

    // Test ODS_JOBS = 0 and 2
    unsafe {
        std::env::set_var("ODS_JOBS", "0");
    }
    assert_eq!(
        parse_paths_parallel(root, std::slice::from_ref(&file), true)
            .unwrap()
            .len(),
        1
    );
    unsafe {
        std::env::set_var("ODS_JOBS", "2");
    }
    assert_eq!(
        parse_paths_parallel(root, std::slice::from_ref(&file), true)
            .unwrap()
            .len(),
        1
    );

    // Test invalid ODS_JOBS
    unsafe {
        std::env::set_var("ODS_JOBS", "invalid");
    }
    let docs = parse_paths_parallel(root, std::slice::from_ref(&file), true).unwrap();
    assert_eq!(docs.len(), 1);

    unsafe {
        std::env::remove_var("ODS_JOBS");
    }

    // Error path: non-existent file
    let bad_file = root.join("nonexistent.md");
    assert!(parse_paths_parallel(root, &[bad_file], true).is_err());
}
