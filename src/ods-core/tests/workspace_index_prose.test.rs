use ods_core::{
    AdoptOptions, adopt_workspace, lint_workspace, load_options_with_bodies, load_workspace,
    load_workspace_with_options, resolve_context,
};
use ods_test_support::temp_workspace;
use std::fs;

#[test]
fn test_body_link_validation() {
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Root\n\n- [doc.md](doc.md)\n",
    )
    .expect("root index");

    fs::write(
        temp.join("doc.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# Doc\n\n- [good link](index.md)\n- [bad link](missing.md)\n",
    )
    .expect("doc");

    let workspace =
        load_workspace_with_options(&temp, load_options_with_bodies()).expect("workspace");
    let diagnostics = lint_workspace(&workspace);

    let dangling_errors = diagnostics
        .iter()
        .filter(|d| d.message.contains("dangling markdown link in body"))
        .collect::<Vec<_>>();

    assert_eq!(dangling_errors.len(), 1);
    assert!(dangling_errors[0].message.contains("missing.md"));
}

#[test]
fn context_ignore_skips_matching_paths() {
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Root\n\n- [main.md](main.md)\n- [archive/](archive/index.md)\n",
    )
    .expect("root");
    fs::create_dir_all(temp.join("archive")).expect("archive");
    fs::write(
        temp.join("archive/index.md"),
        "---\nprofile: index\n---\n\n# Archive\n\n- [old.md](old.md)\n",
    )
    .expect("archive index");
    fs::write(
        temp.join("archive/old.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# Old\n",
    )
    .expect("old");
    fs::write(
        temp.join("main.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - archive/old\ncontext:\n  max-depth: 2\n  ignore:\n    - archive/\n---\n\n# Main\n",
    )
    .expect("main");

    let workspace = load_workspace(&temp).expect("workspace");
    let resolved = resolve_context(&workspace, "main", true);
    let names: Vec<_> = resolved
        .iter()
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_string))
        .collect();
    assert_eq!(names, vec!["main.md".to_string()]);
    assert!(!names.iter().any(|n| n == "old.md"));
}

#[test]
fn context_query_fallback_and_code_ref_tests() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
    )
    .unwrap();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/lib.rs"), "pub fn f() {}\n").unwrap();
    fs::write(
        dir.join("my-doc.md"),
        "---\nprofile: note\ncode:\n  - path: ./src/lib.rs\n    role: implementation\n---\n\n# Doc\n",
    )
    .unwrap();

    let ws = load_workspace(&dir).unwrap();

    assert!(ods_core::resolve_context(&ws, "nonexistent", true).is_empty());

    let res = ods_core::resolve_context(&ws, "my-doc", true);
    let my_doc = dir.join("my-doc.md");
    let my_doc_canon = my_doc.canonicalize().unwrap_or_else(|_| my_doc.clone());
    let lib_rs = dir.join("src/lib.rs");
    let lib_rs_canon = lib_rs.canonicalize().unwrap_or_else(|_| lib_rs.clone());

    assert!(res.contains(&my_doc) || res.contains(&my_doc_canon));
    assert!(res.contains(&lib_rs) || res.contains(&lib_rs_canon));
}

#[test]
fn depends_and_related_references_resolve_without_dangling() {
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Root\n\n- [product.md](product.md)\n- [pricing.md](pricing.md)\n- [service.md](service.md)\n",
    )
    .expect("root index");
    fs::write(
        temp.join("product.md"),
        "---\nprofile: product\nstatus: stable\ndepends:\n  - pricing\nrelated:\n  - service\n---\n\n# Product\n",
    )
    .expect("product");
    fs::write(
        temp.join("pricing.md"),
        "---\nprofile: decision\nstatus: stable\nid: pricing\n---\n\n# Pricing\n",
    )
    .expect("pricing");
    fs::write(
        temp.join("service.md"),
        "---\nprofile: feature\nstatus: stable\nid: service\n---\n\n# Service\n",
    )
    .expect("service");

    let workspace = load_workspace(&temp).expect("workspace");
    let path = temp.join("product.md");
    let diagnostics =
        ods_core::lint_document_in_workspace(&workspace, &path, ods_core::LintLevel::Full);
    let dangling: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.message.contains("dangling reference"))
        .collect();
    assert!(
        dangling.is_empty(),
        "unexpected dangling refs: {dangling:#?}"
    );
}

#[test]
fn adopt_write_adds_minimal_frontmatter() {
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Root\n\n- [plain.md](plain.md)\n",
    )
    .expect("root");
    fs::write(temp.join("plain.md"), "# Plain\n\nJust prose.\n").expect("plain");

    let workspace = load_workspace(&temp).expect("workspace");
    let report = adopt_workspace(&workspace, AdoptOptions { write: true }).expect("adopt");
    assert_eq!(report.written.len(), 1);
    let text = fs::read_to_string(temp.join("plain.md")).expect("read");
    assert!(text.starts_with("---\nprofile: note\nstatus: draft\n---\n"));
    assert!(text.contains("# Plain"));
}

#[test]
fn test_context_share_private_filtering() {
    let temp = temp_workspace();
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Root\n",
    )
    .expect("root");
    fs::write(
        temp.join("public.md"),
        "---\nprofile: note\nstatus: stable\ndepends:\n  - private-doc\n---\n\n# Public Doc\n",
    )
    .expect("public");
    fs::write(
        temp.join("private-doc.md"),
        "---\nprofile: note\nstatus: stable\nid: private-doc\nshare: private\n---\n\n# Private Doc\n",
    )
    .expect("private");

    let workspace = load_workspace(&temp).expect("workspace");

    let paths_excluded = resolve_context(&workspace, "public", false);
    let names_excluded: Vec<_> = paths_excluded
        .iter()
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_string))
        .collect();
    assert_eq!(names_excluded, vec!["public.md".to_string()]);

    let paths_included = resolve_context(&workspace, "public", true);
    let names_included: Vec<_> = paths_included
        .iter()
        .filter_map(|p| p.file_name().and_then(|n| n.to_str()).map(str::to_string))
        .collect();
    assert_eq!(
        names_included,
        vec!["public.md".to_string(), "private-doc.md".to_string()]
    );
}
