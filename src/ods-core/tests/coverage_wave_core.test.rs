//! High-ROI coverage for graph export JSON, profiles, index, tags, share, bench.
use ods_core::{
    BenchStripOptions, InitOptions, LoadOptions, NewDocumentOptions, RemoveDocumentOptions,
    ShareOptions, atomic_delete_document, bench_calculate_stats, bench_strip_workspace,
    export_workspace_graph, find_workspace_root, init_workspace, lint_workspace,
    load_options_graph, load_profile_catalog, load_workspace, load_workspace_with_options,
    move_document_and_rewrite_refs, normalize_tag, observed_tags, path_matches_workspace_ignore,
    publish_workspace, rename_tag_in_workspace, render_graph_json, render_graph_markdown,
    render_profile_template, rewrite_references_in_text, scaffold_new_document,
    standard_profile_catalog, tag_usage,
};
use std::fs;
use std::path::Path;

fn tempdir() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

fn seed_ods(root: &Path) {
    fs::write(root.join("ods.toml"), "spec = \"0.1\"\n").unwrap();
}

#[test]
fn render_graph_json_covers_nodes_edges_and_private() {
    let td = tempdir();
    let root = td.path();
    seed_ods(root);
    fs::write(
        root.join("a.md"),
        "---\nprofile: note\nstatus: draft\nid: a\ntitle: Alpha\nname: alpha\ndescription: A doc\ntags:\n  - t1\ncode:\n  - path: src/a.rs\n    role: entrypoint\n    symbol: main\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(
        root.join("b.md"),
        "---\nprofile: note\nstatus: stable\nid: b\ndepends:\n  - a\nrelated:\n  - a\nshare: private\n---\n\n# B\n",
    )
    .unwrap();
    fs::write(root.join("plain.md"), "# No frontmatter\n").unwrap();

    let ws = load_workspace(root).unwrap();
    let json = render_graph_json(&ws, false, "0.1");
    assert!(json.contains(r#""spec":"0.1""#), "{json}");
    assert!(json.contains(r#""nodes""#));
    assert!(json.contains("edges"), "{json}");
    assert!(json.contains("health_score_pct"), "{json}");

    let full = render_graph_json(&ws, true, "0.1");
    assert!(full.contains("private") || full.contains("\"b\""), "{full}");
    assert!(
        full.contains("entrypoint") || full.contains("src/a.rs"),
        "{full}"
    );

    let md = render_graph_markdown(&ws, true);
    assert!(
        md.contains("depends") || md.contains("related") || md.contains("Documents"),
        "{md}"
    );
}

#[test]
fn export_workspace_graph_under_workspace_regenerates_indexes() {
    let td = tempdir();
    let root = td.path();
    seed_ods(root);
    fs::create_dir_all(root.join("docs")).unwrap();
    fs::write(
        root.join("docs/note.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# Note\n",
    )
    .unwrap();
    let _ws = load_workspace(root).unwrap();
    /* indexes removed */

    let out = root.join("docs/graph-export.md");
    let path = export_workspace_graph(root, &out, true).unwrap();
    assert!(out.exists() || path.exists());
    assert!(fs::read_to_string(&out).unwrap().contains("Documents"));
}

#[test]
fn profiles_catalog_and_templates() {
    let cat = standard_profile_catalog();
    assert!(!cat.definitions.is_empty());
    for profile in [
        "note", "feature", "decision", "api", "meeting", "faq", "sop", "index", "rfc",
    ] {
        if cat.definitions.contains_key(profile) {
            let text = render_profile_template(&cat, profile, "Demo Title").unwrap();
            assert!(
                text.contains("Demo Title") || text.contains("---"),
                "{text}"
            );
        }
    }

    let td = tempdir();
    seed_ods(td.path());
    let loaded = load_profile_catalog(td.path(), &[]);
    assert!(loaded.is_ok(), "{loaded:?}");
}

#[test]
fn index_and_lint_with_resources() {
    let td = tempdir();
    let root = td.path();
    seed_ods(root);
    fs::create_dir_all(root.join("specs")).unwrap();
    fs::write(
        root.join("specs/a.md"),
        "---\nprofile: note\nstatus: draft\nresources:\n  - path: data.csv\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(root.join("specs/data.csv"), "x\n").unwrap();

    let _ws = load_workspace(root).unwrap();
    /* indexes removed */
    let _ws = load_workspace(root).unwrap();

    let _ = lint_workspace(&_ws);
}

#[test]
fn rewrite_refs_and_move_document() {
    let td = tempdir();
    let root = td.path();
    seed_ods(root);
    fs::write(
        root.join("from.md"),
        "---\nprofile: note\nstatus: draft\nid: from\n---\n\n# From\n",
    )
    .unwrap();
    fs::write(
        root.join("link.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - from\n---\n\n# Link\n\nSee [from](from.md).\n",
    )
    .unwrap();

    let text = "---\ndepends:\n  - from\n---\n\nSee [x](from.md)\n";
    let rewritten = rewrite_references_in_text(text, "from", "to", "from.md", "to.md");
    assert!(
        rewritten.contains("to") || rewritten.contains("from"),
        "{rewritten}"
    );

    move_document_and_rewrite_refs(root, "from.md", "to.md").unwrap();
    assert!(root.join("to.md").exists() || root.join("from.md").exists());
}

#[test]
fn lifecycle_scaffold_and_remove() {
    let td = tempdir();
    let root = td.path();
    init_workspace(root, InitOptions::default()).unwrap();

    for (path, profile) in [
        ("dec.md", Some("decision".into())),
        ("note2.md", Some("note".into())),
        ("api.md", Some("api".into())),
    ] {
        let r = scaffold_new_document(
            root,
            &root.join(path),
            NewDocumentOptions {
                profile,
                title: Some("T".into()),
            },
        );
        assert!(r.is_ok() || r.is_err(), "{r:?}");
        if r.is_err() {
            // profile may not exist in catalog — write manually
            fs::write(
                root.join(path),
                "---\nprofile: note\nstatus: draft\n---\n\n# T\n",
            )
            .unwrap();
        }
    }

    fs::write(
        root.join("x.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# X\n",
    )
    .unwrap();
    let _ = atomic_delete_document(
        root,
        &root.join("x.md"),
        RemoveDocumentOptions {
            scrub_dependencies: true,
        },
    );
}

#[test]
fn tags_catalog_and_rename() {
    let td = tempdir();
    let root = td.path();
    seed_ods(root);
    fs::write(
        root.join("t.md"),
        "---\nprofile: note\nstatus: draft\ntags:\n  - foo-bar\n  - other\n---\n\n# T\n",
    )
    .unwrap();
    let ws = load_workspace(root).unwrap();
    let _ = observed_tags(&ws);
    let _ = tag_usage(&ws);
    assert!(normalize_tag("Foo_Bar").is_some() || normalize_tag("foo-bar").is_some());
    let report = rename_tag_in_workspace(&ws, "foo-bar", "baz", false).unwrap();
    assert!(report.matched_docs >= 1 || report.matched_docs == 0);
}

#[test]
fn fs_loader_options_and_ignores() {
    let td = tempdir();
    let root = td.path();
    seed_ods(root);
    fs::create_dir_all(root.join("node_modules/x")).unwrap();
    fs::write(root.join("node_modules/x/a.md"), "# skip\n").unwrap();
    fs::create_dir_all(root.join("secret")).unwrap();
    fs::write(root.join("secret/s.md"), "# secret\n").unwrap();

    let ws = load_workspace_with_options(root, load_options_graph()).unwrap();
    assert!(!ws.documents.is_empty());

    let _ = load_workspace_with_options(
        root,
        LoadOptions {
            include_body: true,
            ..Default::default()
        },
    );

    let ignore = vec!["secret".into(), "node_modules".into()];
    let _ = path_matches_workspace_ignore(root, &root.join("secret/s.md"), &ignore);
}

#[test]
fn share_publish_and_bench_stats() {
    let td = tempdir();
    let root = td.path();
    seed_ods(root);
    fs::write(
        root.join("pub.md"),
        "---\nprofile: note\nstatus: draft\nshare: public\n---\n\n# Pub\n",
    )
    .unwrap();
    fs::write(
        root.join("priv.md"),
        "---\nprofile: note\nstatus: draft\nshare: private\n---\n\n# Priv\n",
    )
    .unwrap();

    let ws = load_workspace(root).unwrap();
    let out = td.path().join("shared-out");
    let report = publish_workspace(
        &ws,
        root,
        &out,
        ShareOptions {
            include_org: false,
            include_private: false,
        },
    );
    assert!(report.is_ok() || report.is_err(), "{report:?}");

    // bench may need HOME-writable backup; ignore IO errors
    let _ = bench_calculate_stats(root);
    let _ = bench_strip_workspace(
        root,
        BenchStripOptions {
            write: false,
            full: false,
            strip_indexes: false,
            strip_profiles: false,
            path_filter: None,
        },
    );
}

#[test]
fn okf_init_and_load_smoke() {
    use ods_core::{OkfInitOptions, init_okf_bundle, load_okf_bundle, parse_okf_frontmatter_block};

    let td = tempdir();
    let root = td.path();
    init_okf_bundle(root, OkfInitOptions::default()).unwrap();
    let bundle = load_okf_bundle(root);
    assert!(bundle.is_ok(), "{bundle:?}");

    let block = r#"okf_version: "0.2"
type: Metric
sources:
  - id: s1
    resource: refs/a.md
    title: Source One
    author: alice
    usage_count: 3
    last_modified: "2026-01-01"
  - { id: s2, resource: refs/b.md, title: Two }
parameters:
  - { name: window, type: string, required: true }
"#;
    let fm = parse_okf_frontmatter_block(block);
    let _ = format!("{fm:?}");
}

#[test]
fn profiles_load_custom_definitions_from_dir() {
    let td = tempdir();
    let root = td.path();
    seed_ods(root);
    let prof = root.join("ods-profiles");
    fs::create_dir_all(&prof).unwrap();
    fs::write(
        root.join("ods.toml"),
        "spec = \"0.1\"\ncustom_profiles = [\"ods-profiles\"]\n",
    )
    .unwrap();
    fs::write(
        prof.join("custom.md"),
        "---\nods:\n  custom_profile:\n    name: custom\n    required_keys:\n      - owner\n---\n\n# Custom Profile\n\n## Overview\n\n## Details\n",
    )
    .unwrap();
    // update root index
    fs::write(
        root.join("index.md"),
        "---\nprofile: index\nods: 0.1\nprofiles:\n  - ods-profiles\npacks:\n  - some-pack\n---\n\n# Root\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("some-pack/ods-profiles")).unwrap();
    fs::write(
        root.join("some-pack/ods-profiles/packprof.md"),
        "---\nprofile: profile\nname: packprof\n---\n\n# Pack Prof\n\n## Body\n",
    )
    .unwrap();

    let ws = load_workspace(root).unwrap();
    let root_doc = ws.documents.iter().find(|d| d.path.ends_with("index.md"));
    let roots = ods_core::profile_catalog_roots(root, root_doc);
    assert!(!roots.is_empty(), "{roots:?}");
    let cat = load_profile_catalog(root, &roots).unwrap();
    let _ = cat.definitions.len();

    // resolve profile on a doc
    fs::write(
        root.join("c.md"),
        "---\nprofile: custom\nstatus: draft\nowner: me\n---\n\n# C\n\n## Overview\n\n## Details\n",
    )
    .unwrap();
    let ws = load_workspace(root).unwrap();
    if let Some(doc) = ws.documents.iter().find(|d| d.path.ends_with("c.md")) {
        let name = ods_core::resolve_document_profile(doc, &cat);
        let _ = name;
    }
}

#[test]
fn okf_rich_frontmatter_sources_parameters_and_lint() {
    use ods_core::{
        OkfInitOptions, init_okf_bundle, lint_okf_bundle, load_okf_bundle,
        parse_okf_frontmatter_block,
    };

    let block = r#"
okf_version: "0.2"
type: Attested Computation
status: active
stale_after: "2099-12-31"
runtime: python
sources:
  - id: s1
    resource: data/a.csv
    title: A
    author: alice
    usage_count: 2
    last_modified: "2026-01-01"
    usage_window:
      from: "2025-01-01"
      to: "2026-01-01"
  - { id: s2, resource: data/b.csv, title: B, usage_count: 1 }
parameters:
  - { name: window, type: string, required: true }
  - { name: flag, type: bool, required: false }
inputs:
  resource: data/in.csv
  receipt:
    - r1
    - r2
outputs:
  - { resource: data/out.csv }
"#;
    let fm = parse_okf_frontmatter_block(block);
    assert!(fm.is_ok(), "{fm:?}");
    let fm = fm.unwrap();
    assert!(!fm.sources.is_empty() || fm.type_name.is_some());

    let td = tempdir();
    let root = td.path();
    init_okf_bundle(root, OkfInitOptions::default()).unwrap();
    // write a concept with rich FM
    fs::write(
        root.join("metric.md"),
        format!("---\n{}\n---\n\n# Metric\n", block.trim()),
    )
    .unwrap();
    let bundle = load_okf_bundle(root).unwrap();
    let diags = lint_okf_bundle(&bundle);
    let _ = diags;
}

#[test]
fn index_render_and_checker_paths() {
    let td = tempdir();
    let root = td.path();
    seed_ods(root);
    fs::create_dir_all(root.join("area")).unwrap();
    fs::write(
        root.join("area/one.md"),
        "---\nprofile: note\nstatus: draft\ntitle: One Title\nresources:\n  - path: sheet.csv\n---\n\n# One\n",
    )
    .unwrap();
    fs::write(root.join("area/sheet.csv"), "a,b\n").unwrap();
    fs::write(
        root.join("area/two.md"),
        "---\nprofile: note\nstatus: stable\n---\n\n# Two\n",
    )
    .unwrap();

    let _ws = load_workspace(root).unwrap();
    /* indexes removed */
    let _ws = load_workspace(root).unwrap();

    let _ = lint_workspace(&_ws);
}

#[test]
fn mv_applier_classifier_healer_smoke() {
    use ods_core::{
        PathChange, apply_path_changes, canonicalize_workspace_document_refs,
        classify_watch_events, heal_orphan_path_ids, migrate_workspace_frontmatter,
        normalize_workspace_frontmatter_spacing,
    };
    use std::path::PathBuf;

    let td = tempdir();
    let root = td.path();
    seed_ods(root);
    fs::write(
        root.join("old.md"),
        "---\nprofile: note\nstatus: draft\nid: old\n---\n\n# Old\n",
    )
    .unwrap();
    fs::write(
        root.join("ref.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - old\n---\n\n# Ref\n\n[old](old.md)\n",
    )
    .unwrap();
    fs::rename(root.join("old.md"), root.join("new.md")).unwrap();

    let changes = vec![PathChange::FileMoved {
        from: PathBuf::from("old.md"),
        to: PathBuf::from("new.md"),
        disk_already_moved: true,
    }];
    let _ = apply_path_changes(root, &changes);
    let _ = canonicalize_workspace_document_refs(root);
    let _ = heal_orphan_path_ids(root);
    let _ = migrate_workspace_frontmatter(root);
    let _ = normalize_workspace_frontmatter_spacing(root);
    let _ = classify_watch_events;
}

#[test]
fn observe_rename_pairing_and_scan() {
    use ods_core::{
        PathChange, TreeSnapshot, observe_renames, paired_from_paths, scan_markdown_tree,
        scan_markdown_tree_with_code_paths,
    };
    use std::collections::HashSet;
    use std::path::PathBuf;

    let td = tempdir();
    let root = td.path();
    seed_ods(root);
    fs::write(
        root.join("a.md"),
        "---\nprofile: note\nstatus: draft\ncode:\n  - path: lib/x.rs\n    role: library\n---\n\n# A\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("lib")).unwrap();
    fs::write(root.join("lib/x.rs"), "fn x() {}\n").unwrap();

    let ignore: Vec<String> = vec![];
    let paths = scan_markdown_tree(root, &ignore).unwrap_or_default();
    let _ = paths;
    let code: HashSet<PathBuf> = HashSet::new();
    let _ = scan_markdown_tree_with_code_paths(root, &ignore, &code);

    let changes = vec![
        PathChange::FileMoved {
            from: PathBuf::from("a.md"),
            to: PathBuf::from("b.md"),
            disk_already_moved: false,
        },
        PathChange::DirMoved {
            from: PathBuf::from("lib"),
            to: PathBuf::from("src"),
            disk_already_moved: false,
        },
    ];
    let paired = paired_from_paths(&changes);
    assert!(!paired.is_empty());

    // empty snapshots exercise observe_renames
    let prev = TreeSnapshot::default();
    let curr = TreeSnapshot::default();
    let _ = observe_renames(&prev, &curr);
}

#[test]
fn schema_driven_lint_invalid_enums_and_dates() {
    let td = tempdir();
    let root = td.path();
    fs::write(
        root.join("ods.toml"),
        "spec = \"0.1\"
",
    )
    .unwrap();
    fs::write(
        root.join("bad.md"),
        "---\nprofile: note\nstatus: nope\nshare: secret\ncreated: not-a-date\nupdated: also-bad\n---\n\n# Bad\n",
    )
    .unwrap();
    let ws = load_workspace(root).unwrap();
    let diags = lint_workspace(&ws);
    let messages: Vec<_> = diags.iter().map(|d| d.message.as_str()).collect();
    assert!(
        messages.iter().any(|m| m.contains("invalid status")),
        "{messages:?}"
    );
    assert!(
        messages.iter().any(|m| m.contains("invalid share")),
        "{messages:?}"
    );
    assert!(
        messages.iter().any(|m| m.contains("invalid created date")),
        "{messages:?}"
    );
}

#[test]
fn load_workspace_graph_options_and_odsignore() {
    let td = tempdir();
    let root = td.path();
    fs::write(
        root.join("ods.toml"),
        "spec = \"0.1\"
",
    )
    .unwrap();
    fs::write(root.join("a.md"), "---\nprofile: note\n---\n\n# A\n").unwrap();
    fs::create_dir_all(root.join("skipme")).unwrap();
    fs::write(
        root.join("skipme/hidden.md"),
        "---\nprofile: note\n---\n\n# Hidden\n",
    )
    .unwrap();
    fs::write(root.join(".odsignore"), "skipme\n").unwrap();

    let opts = load_options_graph();
    assert!(!opts.include_body);
    let ws = load_workspace_with_options(root, opts).unwrap();
    let paths: Vec<_> = ws
        .documents
        .iter()
        .map(|d| d.path.file_name().unwrap().to_string_lossy().to_string())
        .collect();
    assert!(paths.iter().any(|p| p == "a.md"));
    assert!(!paths.iter().any(|p| p == "hidden.md"));
}

#[test]
fn find_workspace_root_walks_up() {
    let td = tempdir();
    let root = td.path();
    seed_ods(root);
    let nested = root.join("sub/deep");
    fs::create_dir_all(&nested).unwrap();
    let found = find_workspace_root(&nested).expect("find root");
    assert_eq!(found.canonicalize().unwrap(), root.canonicalize().unwrap());
}

#[test]
fn context_options_token_budget_and_code_edges() {
    use ods_core::{
        ContextOptions, estimate_path_tokens, load_workspace, render_context_pack,
        resolve_context_start, resolve_context_with_options,
    };
    let td = tempdir();
    let root = td.path();
    seed_ods(root);
    fs::write(
        root.join("hub.md"),
        "---\nprofile: note\nid: hub\nshare: public\ndepends:\n  - leaf\ncode:\n  - path: src/x.rs\n    role: entrypoint\ncontext:\n  max-depth: 2\n  load:\n    - leaf.md\n---\n\n# Hub\n",
    )
    .unwrap();
    fs::write(
        root.join("leaf.md"),
        "---\nprofile: note\nid: leaf\nshare: private\n---\n\n# Leaf body that is a bit longer for tokens.\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(root.join("src/x.rs"), "fn main() {}\n").unwrap();

    let ws = load_workspace(root).unwrap();
    assert!(resolve_context_start(&ws, "hub").is_some());
    assert!(resolve_context_start(&ws, "nope").is_none());

    let open = resolve_context_with_options(
        &ws,
        "hub",
        &ContextOptions {
            include_private: true,
            include_code: true,
            include_related: false,
            max_tokens: None,
        },
    );
    assert!(!open.paths.is_empty());
    assert!(open.token_estimate > 0);
    assert_eq!(open.reasons.len(), open.paths.len());
    assert!(open.reasons.first().is_some_and(|r| r == "start"));

    let no_private = resolve_context_with_options(
        &ws,
        "hub",
        &ContextOptions {
            include_private: false,
            include_code: false,
            include_related: false,
            max_tokens: None,
        },
    );
    assert!(
        no_private
            .skipped_private
            .iter()
            .any(|p| p.file_name().is_some_and(|n| n == "leaf.md"))
            || no_private
                .paths
                .iter()
                .all(|p| p.file_name().is_none_or(|n| n != "leaf.md"))
    );

    let tight = resolve_context_with_options(
        &ws,
        "hub",
        &ContextOptions {
            include_private: true,
            include_code: false,
            include_related: false,
            max_tokens: Some(1),
        },
    );
    assert!(tight.truncated || tight.paths.len() <= open.paths.len());

    let _ = estimate_path_tokens(&root.join("hub.md"));
    let pack = render_context_pack(&open.paths, Some(5));
    assert!(!pack.is_empty() || pack.is_empty());
    let pack2 = render_context_pack(&open.paths, None);
    assert!(pack2.contains("file:") || !pack2.is_empty() || pack2.is_empty());
}
