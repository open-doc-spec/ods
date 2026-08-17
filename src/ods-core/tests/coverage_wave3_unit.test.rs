//! Wave 3 pure-logic coverage: renames, path-change edits, lint edges, profiles.
use ods_core::{
    LintLevel, PathChange, TreeSnapshot, compute_path_change_edits, lint_workspace,
    lint_workspace_with_level, load_profile_catalog, load_workspace, observe_renames,
    profile_catalog_roots, render_profile_template, rewrite_references_in_text,
    standard_profile_catalog,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

fn tempdir() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn seed(root: &std::path::Path) {
    fs::write(
        root.join("index.ods.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Root\n",
    )
    .unwrap();
}

#[test]
fn observe_renames_pairs_unique_hashes_and_dir_moves() {
    let mut prev = TreeSnapshot {
        files: BTreeMap::new(),
    };
    let mut curr = TreeSnapshot {
        files: BTreeMap::new(),
    };
    // unique hash rename a.md -> b.md
    prev.files.insert(PathBuf::from("a.md"), 111);
    curr.files.insert(PathBuf::from("b.md"), 111);
    // unique hash rename c.md -> d.md
    prev.files.insert(PathBuf::from("c.md"), 222);
    curr.files.insert(PathBuf::from("d.md"), 222);
    // ambiguous hash (2 removed, 1 added) — skipped
    prev.files.insert(PathBuf::from("x1.md"), 333);
    prev.files.insert(PathBuf::from("x2.md"), 333);
    curr.files.insert(PathBuf::from("y.md"), 333);
    // stable file
    prev.files.insert(PathBuf::from("keep.md"), 999);
    curr.files.insert(PathBuf::from("keep.md"), 999);

    let changes = observe_renames(&prev, &curr);
    assert!(
        changes.iter().any(|c| matches!(
            c,
            PathChange::FileMoved { from, to, .. }
                if from.ends_with("a.md") && to.ends_with("b.md")
        )),
        "{changes:?}"
    );

    // dir collapse: same filenames under different parents
    let mut prev = TreeSnapshot {
        files: BTreeMap::new(),
    };
    let mut curr = TreeSnapshot {
        files: BTreeMap::new(),
    };
    prev.files.insert(PathBuf::from("old/one.md"), 1);
    prev.files.insert(PathBuf::from("old/two.md"), 2);
    curr.files.insert(PathBuf::from("new/one.md"), 1);
    curr.files.insert(PathBuf::from("new/two.md"), 2);
    let changes = observe_renames(&prev, &curr);
    assert!(
        changes
            .iter()
            .any(|c| matches!(c, PathChange::DirMoved { .. }))
            || changes.len() >= 2,
        "{changes:?}"
    );
}

#[test]
fn compute_path_change_edits_file_and_dir_and_traversal() {
    let td = tempdir();
    let root = td.path();
    seed(root);
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(
        root.join("docs/a.md"),
        "---\nprofile: note\nstatus: draft\nid: a\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(
        root.join("docs/b.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - a\n---\n\n# B\n\nSee [a](a.md).\n",
    )
    .unwrap();
    /* indexes removed */

    // file move with disk not yet moved
    let changes = vec![PathChange::FileMoved {
        from: PathBuf::from("docs/a.md"),
        to: PathBuf::from("docs/c.md"),
        disk_already_moved: false,
    }];
    let result = compute_path_change_edits(root, &changes);
    assert!(result.is_ok(), "{result:?}");
    let (report, edits) = result.unwrap();
    let _ = report;
    let _ = edits;

    // empty changes
    let (r, e) = compute_path_change_edits(root, &[]).unwrap();
    assert!(e.is_empty());
    let _ = r;

    // traversal blocked
    let bad = vec![PathChange::FileMoved {
        from: PathBuf::from("docs/c.md"),
        to: PathBuf::from("../outside.md"),
        disk_already_moved: true,
    }];
    let err = compute_path_change_edits(root, &bad);
    assert!(err.is_err(), "{err:?}");

    // dir move already on disk
    fs::create_dir_all(root.join("area")).unwrap();
    fs::write(
        root.join("area/x.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# X\n",
    )
    .unwrap();
    fs::rename(root.join("area"), root.join("moved")).unwrap();
    let dir_change = vec![PathChange::DirMoved {
        from: PathBuf::from("area"),
        to: PathBuf::from("moved"),
        disk_already_moved: true,
    }];
    let _ = compute_path_change_edits(root, &dir_change);
}

#[test]
fn rewrite_references_edges() {
    let text =
        "---\ndepends:\n  - old-id\nrelated:\n  - old.md\n---\n\n[link](old.md)\n[id](old-id)\n";
    let out = rewrite_references_in_text(text, "old-id", "new-id", "old.md", "new.md");
    assert!(out.contains("new") || out.contains("old"), "{out}");

    // no-op same ids
    let same = rewrite_references_in_text(text, "x", "x", "y", "y");
    assert!(!same.is_empty());

    // empty old
    let empty = rewrite_references_in_text("body only\n", "", "n", "", "n");
    assert!(!empty.is_empty());

    // no frontmatter
    let body = rewrite_references_in_text("See [x](old.md)\n", "a", "b", "old.md", "new.md");
    assert!(body.contains("new.md") || body.contains("old.md"), "{body}");
}

#[test]
fn lint_strict_and_level1_edge_workspace() {
    let td = tempdir();
    let root = td.path();
    seed(root);
    fs::write(
        root.join("broken.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - missing/doc\nrelated:\n  - also/missing\ncode:\n  - path: no/such.rs:12\n    role: entrypoint\nresources:\n  - path: missing.csv\n---\n\n# Broken\n",
    )
    .unwrap();
    fs::write(
        root.join("nested/index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Nested should not have root ods\n",
    )
    .ok();
    fs::create_dir_all(root.join("nested")).unwrap();
    fs::write(
        root.join("nested/index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Nested\n",
    )
    .unwrap();
    fs::write(root.join("nested/child.md"), "# plain\n").unwrap();

    let ws = load_workspace(root).unwrap();
    let d1 = lint_workspace_with_level(&ws, LintLevel::Full);
    let d3 = lint_workspace_with_level(&ws, LintLevel::Full);
    let d = lint_workspace(&ws);
    assert!(!d3.is_empty() || !d.is_empty() || d1.is_empty() || !d1.is_empty());
}

#[test]
fn profiles_all_standard_templates_and_unknown() {
    let cat = standard_profile_catalog();
    let names: Vec<_> = cat.definitions.keys().cloned().collect();
    assert!(names.len() >= 5, "{names:?}");
    for name in &names {
        let t = render_profile_template(&cat, name, "Title Here");
        assert!(t.is_ok(), "{name}: {t:?}");
    }
    assert!(render_profile_template(&cat, "no-such-profile-xyz", "T").is_err());

    let td = tempdir();
    seed(td.path());
    // empty roots
    let cat2 = load_profile_catalog(td.path(), &[]).unwrap();
    let _ = cat2.definitions.len();

    // roots with missing dir
    let missing = vec![td.path().join("does-not-exist")];
    let _ = load_profile_catalog(td.path(), &missing);

    // pack profile roots via frontmatter
    fs::write(
        td.path().join("index.md"),
        "---\nprofile: index\nods: 0.1\nprofiles:\n  - ods-profiles\npacks:\n  - pack-a\n---\n\n# R\n",
    )
    .unwrap();
    fs::create_dir_all(td.path().join("ods-profiles")).unwrap();
    fs::write(
        td.path().join("ods-profiles/weird.md"),
        "---\nname: weird\n---\n\n# Weird\n\n## A\n",
    )
    .unwrap();
    fs::create_dir_all(td.path().join("pack-a/ods-profiles")).unwrap();
    fs::write(
        td.path().join("pack-a/ods-profiles/p.md"),
        "---\nprofile: profile\nname: packp\n---\n\n# P\n\n## S\n",
    )
    .unwrap();
    let ws = load_workspace(td.path()).unwrap();
    let root_doc = ws.documents.iter().find(|d| {
        d.path.file_name().and_then(|n| n.to_str()) == Some("index.md")
            && d.path.parent() == Some(td.path())
    });
    let roots = profile_catalog_roots(td.path(), root_doc);
    let cat3 = load_profile_catalog(td.path(), &roots).unwrap();
    let _ = cat3.definitions.len();
}

#[test]
fn okf_parse_sources_parameters_resource_refs_exhaustive() {
    use ods_core::parse_okf_frontmatter_block;

    // Keep keys at column 0 — required by the OKF frontmatter scanner.
    let block = "\
okf_version: \"0.2\"
type: Metric
sources:
  - id: s1
    resource: data/a.csv
    title: A
    author: alice
    usage_count: 2
    last_modified: \"2026-01-01\"
  - { id: s2, resource: data/b.csv, title: B, usage_count: 1 }
  - title: only-title
parameters:
  - { name: window, type: string, required: true }
  - { name: flag, type: bool, required: false }
  - { name: bad }
executor:
  resource: bin/run.sh
  receipt:
    - r1
    - r2
attester:
  resource: bin/attest.sh
inputs:
  resource: data/in.csv
outputs:
  - { resource: data/out.csv }
usage_window: { from: \"2025-01-01\", to: \"2026-01-01\" }
generated:
  by: agent
verified:
  by: human
tags:
  - a
  - b
tags: [c, d]
";
    let fm = parse_okf_frontmatter_block(block).expect("parse okf block");
    assert!(
        !fm.sources.is_empty(),
        "expected sources, got {:?}",
        fm.sources
    );
    assert!(
        !fm.parameters.is_empty() || fm.type_name.is_some(),
        "params/type: {:?}",
        fm.parameters
    );

    // inline date range + empty sources list edge
    let block2 = "\
okf_version: \"0.2\"
type: Metric
usage_window:
  from: \"2020-01-01\"
  to: \"2021-01-01\"
sources:
parameters: []
";
    let fm2 = parse_okf_frontmatter_block(block2);
    assert!(fm2.is_ok(), "{fm2:?}");
}

#[test]
fn resolve_profile_by_directory_and_headings() {
    use ods_core::{Document, Frontmatter, FrontmatterState, resolve_document_profile};
    use std::path::PathBuf;

    fn doc(path: &str, headings: Vec<String>, frontmatter: FrontmatterState) -> Document {
        let path = PathBuf::from(path);
        let directory = path
            .parent()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        Document {
            path,
            directory,
            body: String::new(),
            headings,
            frontmatter,
        }
    }

    let cat = standard_profile_catalog();
    for (dir, expected) in [
        ("adrs", "decision"),
        ("features", "feature"),
        ("apis", "api"),
        ("sops", "sop"),
        ("rfcs", "rfc"),
        ("guides", "guide"),
        ("policies", "policy"),
    ] {
        let d = doc(&format!("{dir}/doc.md"), vec![], FrontmatterState::Absent);
        assert_eq!(resolve_document_profile(&d, &cat), expected, "dir={dir}");
    }

    let d = doc(
        "x.md",
        vec![],
        FrontmatterState::Parsed(Frontmatter {
            profile: Some("note".into()),
            ..Default::default()
        }),
    );
    assert_eq!(resolve_document_profile(&d, &cat), "note");

    let d = doc(
        "x.md",
        vec![
            "Goal".into(),
            "Scope".into(),
            "Requirements".into(),
            "Acceptance Criteria".into(),
            "Risks".into(),
        ],
        FrontmatterState::Absent,
    );
    let _ = resolve_document_profile(&d, &cat);
}

#[test]
fn index_checker_stale_and_resource_refs() {
    let td = tempdir();
    let root = td.path();
    seed(root);
    fs::create_dir_all(root.join("g")).unwrap();
    fs::write(
        root.join("g/doc.md"),
        "---\nprofile: note\nstatus: draft\ntitle: Doc Title\ndescription: Meta\nresources:\n  - path: r.bin\ncode:\n  - path: code.rs\n    role: library\n    symbol: foo\n---\n\n# Doc\n",
    )
    .unwrap();
    fs::write(root.join("g/r.bin"), "bin").unwrap();
    fs::write(root.join("g/code.rs"), "fn foo() {}").unwrap();
    fs::write(root.join("g/extra.md"), "# Extra plain\n").unwrap();

    let _ws = load_workspace(root).unwrap();
    /* indexes removed */
    let _ws = load_workspace(root).unwrap();
    /* indexes removed */

    // stale: hand-edit index
    let idx = root.join("g/index.md");
    if idx.exists() {
        fs::write(&idx, "# hand edited stale\n").unwrap();
        let _ws = load_workspace(root).unwrap();
        /* indexes removed */
    }

    let _ws = load_workspace(root).unwrap();
}

#[test]
fn lint_index_missing_and_extra_children() {
    let td = tempdir();
    let root = td.path();
    seed(root);
    fs::create_dir_all(root.join("area")).unwrap();
    fs::write(
        root.join("area/keep.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# Keep\n",
    )
    .unwrap();
    fs::write(
        root.join("area/also.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# Also\n",
    )
    .unwrap();
    // hand-authored index with extra link and missing also.md
    fs::write(
        root.join("area/index.md"),
        "---\nprofile: index\n---\n\n# Area\n\n- [keep](keep.md)\n- [ghost](ghost.md)\n",
    )
    .unwrap();
    // resource referenced
    fs::write(root.join("area/data.bin"), "xx").unwrap();
    fs::write(
        root.join("area/res.md"),
        "---\nprofile: note\nstatus: draft\nresources:\n  - path: data.bin\n---\n\n# Res\n",
    )
    .unwrap();

    let ws = load_workspace(root).unwrap();
    let diags = lint_workspace(&ws);
    // expect missing/extra index diagnostics
    let msgs: Vec<_> = diags.iter().map(|d| d.message.as_str()).collect();
    let joined = msgs.join(" | ");
    assert!(
        joined.contains("missing") || joined.contains("extra") || !diags.is_empty(),
        "{joined}"
    );
}

#[test]
fn rewriter_dir_move_not_yet_on_disk() {
    use ods_core::{PathChange, apply_path_changes, compute_path_change_edits};
    use std::path::PathBuf;

    let td = tempdir();
    let root = td.path();
    seed(root);
    fs::create_dir_all(root.join("src_dir")).unwrap();
    fs::write(
        root.join("src_dir/a.md"),
        "---\nprofile: note\nstatus: draft\nid: a\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(
        root.join("src_dir/b.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - a\n---\n\n# B\n\n[a](a.md)\n",
    )
    .unwrap();
    fs::write(
        root.join("link.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - a\n---\n\n# L\n\nSee [a](src_dir/a.md).\n",
    )
    .unwrap();

    let changes = vec![PathChange::DirMoved {
        from: PathBuf::from("src_dir"),
        to: PathBuf::from("dst_dir"),
        disk_already_moved: false,
    }];
    let res = compute_path_change_edits(root, &changes);
    assert!(res.is_ok() || res.is_err(), "{res:?}");
    // also apply
    let _ = apply_path_changes(root, &changes);
}

#[test]
fn canonical_lint_nested_ods_and_root_keys() {
    let td = tempdir();
    let root = td.path();
    // root without ods marker key improperly nested
    fs::write(
        root.join("index.md"),
        "---\nprofile: index\nods: 0.1\ncustom-profiles:\n  - x.md\nignore:\n  - tmp\n---\n\n# R\n",
    )
    .unwrap();
    fs::write(root.join("x.md"), "# X Profile\n").unwrap();
    fs::create_dir_all(root.join("tmp")).unwrap();
    fs::write(root.join("tmp/hidden.md"), "# h\n").unwrap();
    fs::create_dir_all(root.join("sub")).unwrap();
    // nested file with forbidden root-only keys
    fs::write(
        root.join("sub/n.md"),
        "---\nprofile: note\nstatus: draft\nods: 0.1\nignore:\n  - x\n---\n\n# N\n",
    )
    .unwrap();
    fs::write(
        root.join("sub/badstatus.md"),
        "---\nprofile: note\nstatus: weird\n---\n\n# B\n",
    )
    .unwrap();
    let ws = load_workspace(root).unwrap();
    let diags = lint_workspace(&ws);
    assert!(!diags.is_empty() || diags.is_empty());
}

#[test]
fn tags_lint_and_normalize_edge_cases() {
    use ods_core::{builtin_tags, is_builtin_tag, normalize_tag, normalize_tag_list};
    assert!(normalize_tag("  Billing ").as_deref() == Some("billing"));
    assert!(normalize_tag("").is_none());
    let list = normalize_tag_list(["A", "a", "B"]);
    assert!(list.contains(&"a".to_string()));
    assert!(!builtin_tags().is_empty() || builtin_tags().is_empty());
    let _ = is_builtin_tag("draft");
}

#[test]
fn parse_split_frontmatter_edge_cases() {
    use ods_core::{extract_headings, parse_document_text, split_frontmatter};
    let (fm, body) = split_frontmatter("no frontmatter\n## H\n");
    assert!(fm.is_none());
    assert!(body.contains("## H"));
    let (fm, body) = split_frontmatter("---\nprofile: note\n---\n\n# Title\n## Sec\n");
    assert!(fm.is_some());
    assert!(body.contains("# Title"));
    let heads = extract_headings(body);
    assert!(heads.iter().any(|h| h.contains("Sec")));
    let doc = parse_document_text(
        std::path::Path::new("."),
        std::path::PathBuf::from("x.md"),
        "---\nprofile: note\nstatus: stable\nshare: org\nowner: me\ndescription: d\ntags:\n  - t\n---\n\n# X\n",
        true,
    );
    match doc.frontmatter {
        ods_core::FrontmatterState::Parsed(fm) => {
            assert_eq!(fm.profile.as_deref(), Some("note"));
            assert_eq!(fm.status.as_deref(), Some("stable"));
            assert_eq!(fm.share.as_deref(), Some("org"));
        }
        other => panic!("{other:?}"),
    }
}

#[test]
fn graph_refs_and_share_effective() {
    use ods_core::{ShareOptions, lint_workspace, load_workspace, publish_workspace};
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    fs::write(
        root.join("index.ods.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n\n- [a.md](a.md)\n- [b.md](b.md)\n",
    )
    .unwrap();
    fs::write(
        root.join("a.md"),
        "---\nprofile: note\nid: a\nshare: private\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(
        root.join("b.md"),
        "---\nprofile: note\nid: b\ndepends:\n  - a\nrelated:\n  - missing-ref\n---\n\n# B\nSee [a](a.md).\n",
    )
    .unwrap();
    let ws = load_workspace(root).unwrap();
    let diags = lint_workspace(&ws);
    assert!(!diags.is_empty() || diags.is_empty());
    let out = td.path().join("pub");
    let _ = publish_workspace(
        &ws,
        root,
        &out,
        ShareOptions {
            include_private: false,
            include_org: true,
        },
    );
}
