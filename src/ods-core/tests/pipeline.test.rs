//! Coverage for functional pipeline: discover / parse / apply.
use ods_core::{
    apply_document_removes, apply_document_upserts, discover_markdown_paths, load_workspace,
    parse_document_text, parse_paths_parallel, rebuild_indexes,
};
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn apply_upserts_and_removes_rebuild_indexes() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(
        root.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Root\n",
    )
    .unwrap();
    let mut ws = load_workspace(root).unwrap();
    assert_eq!(ws.documents.len(), 1);

    let a_path = root.join("a.md");
    fs::write(&a_path, "---\nprofile: note\nstatus: draft\n---\n\n# A\n").unwrap();
    let doc_a = parse_document_text(
        root,
        a_path.clone(),
        &fs::read_to_string(&a_path).unwrap(),
        true,
    );
    apply_document_upserts(&mut ws, vec![doc_a]);
    assert!(ws.document_by_path(&a_path).is_some());
    assert!(ws.by_path.contains_key(&a_path));

    // Update same path
    let doc_a2 = parse_document_text(
        root,
        a_path.clone(),
        "---\nprofile: note\nstatus: stable\n---\n\n# A2\n",
        true,
    );
    apply_document_upserts(&mut ws, vec![doc_a2]);
    assert_eq!(ws.documents.iter().filter(|d| d.path == a_path).count(), 1);

    // Empty upsert is no-op
    apply_document_upserts(&mut ws, vec![]);

    let removed = apply_document_removes(&mut ws, &[a_path.as_path()]);
    assert_eq!(removed, 1);
    assert!(ws.document_by_path(&a_path).is_none());

    assert_eq!(apply_document_removes(&mut ws, &[]), 0);
    rebuild_indexes(&mut ws);
}

#[test]
fn parse_paths_parallel_and_discover() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::write(
        root.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("sub")).unwrap();
    fs::write(
        root.join("sub/note.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# N\n",
    )
    .unwrap();
    fs::write(root.join("skip.txt"), "not md").unwrap();

    let paths = discover_markdown_paths(root, &[], &[], &[]).unwrap();
    assert!(paths.iter().any(|p| p.ends_with("note.md")));
    assert!(paths.iter().any(|p| p.ends_with("index.md")));

    let docs = parse_paths_parallel(root, &paths, false).unwrap();
    assert_eq!(docs.len(), paths.len());
    // Graph mode: bodies are not retained (index child-list lint removed).
    let index_doc = docs.iter().find(|d| d.path.ends_with("index.md")).unwrap();
    assert!(index_doc.body.is_empty());

    // ODS_JOBS & ODC_JOBS path (edition 2024: env mutation is unsafe)
    unsafe {
        std::env::set_var("ODS_JOBS", "1");
    }
    let docs2 = parse_paths_parallel(root, &paths, true).unwrap();
    unsafe {
        std::env::remove_var("ODS_JOBS");
        std::env::set_var("ODC_JOBS", "1");
    }
    let docs3 = parse_paths_parallel(root, &paths, true).unwrap();
    unsafe {
        std::env::remove_var("ODC_JOBS");
    }
    assert_eq!(docs2.len(), paths.len());
    assert_eq!(docs3.len(), paths.len());
}

#[test]
fn discover_respects_workspace_ignore() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("hidden")).unwrap();
    fs::write(root.join("visible.md"), "# V\n").unwrap();
    fs::write(root.join("hidden/x.md"), "# H\n").unwrap();
    let paths = discover_markdown_paths(root, &[], &[], &["hidden".into()]).unwrap();
    assert!(paths.iter().any(|p| p.ends_with("visible.md")));
    assert!(!paths.iter().any(|p| p.to_string_lossy().contains("hidden")));
}

#[test]
fn discover_empty_missing_dir_ok() {
    let missing = PathBuf::from("/tmp/ods-does-not-exist-coverage-xyz");
    let _ = fs::remove_dir_all(&missing);
    let paths = discover_markdown_paths(&missing, &[], &[], &[]).unwrap();
    assert!(paths.is_empty());
}

#[test]
fn discover_gitignore_exhaustive_patterns() {
    let dir = tempdir().unwrap();
    let root = dir.path();
    fs::create_dir_all(root.join("sub/dir")).unwrap();
    fs::create_dir_all(root.join("nested/sub/dir")).unwrap();
    fs::write(root.join("sub/a.md"), "# A\n").unwrap();
    fs::write(root.join("sub/dir/b.md"), "# B\n").unwrap();
    fs::write(root.join("nested/sub/dir/c.md"), "# C\n").unwrap();
    fs::write(root.join("keep.md"), "# K\n").unwrap();

    let gitignore = vec![
        "sub/a.md".into(),       // relative == pattern
        "sub/dir".into(),        // relative.starts_with
        "nested/sub/dir".into(), // relative.contains
        "keep.md".into(),        // name == pattern
    ];
    let paths = discover_markdown_paths(root, &[], &gitignore, &[]).unwrap();
    assert!(paths.is_empty());
}
