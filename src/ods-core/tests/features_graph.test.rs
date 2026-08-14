use ods_core::{
    LintLevel, canonicalize_workspace_document_refs, export_workspace_graph, lint_workspace,
    lint_workspace_with_ref_style, load_workspace, resolve_context,
};
use std::fs;

fn tempdir() -> tempfile::TempDir {
    tempfile::tempdir().expect("tempdir")
}

#[test]
fn graph_keys_and_context_work_end_to_end() {
    let dir = tempdir();
    fs::write(
        dir.path().join("ods.toml"),
        "spec = \"0.1\"\n",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("specs")).unwrap();
    fs::write(
        dir.path().join("impl.md"),
        "---\nprofile: note\nstatus: stable\nid: stable/impl\n---\n\n# Impl\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("related.md"),
        "---\nprofile: note\nstatus: draft\n---\n\n# Related\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("specs/feature.md"),
        "---\nprofile: feature\nstatus: draft\ndepends:\n  - stable/impl\nrelated:\n  - related\ncontext:\n  load:\n    - stable/impl\n  ignore:\n    - archive/old\n---\n\n# Feature\n\n## Goal\n\n## Scope\n\n## Requirements\n\n## Acceptance Criteria\n\n## Risks\n",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("archive")).unwrap();
    fs::write(
        dir.path().join("archive/old.md"),
        "---\nprofile: note\nstatus: archived\n---\n\n# Old\n",
    )
    .unwrap();
    /* indexes removed */
    let workspace = load_workspace(dir.path()).unwrap();
    let diags = lint_workspace(&workspace);
    assert!(diags.is_empty(), "{diags:?}");
    let context = resolve_context(&workspace, "specs/feature", true);
    assert!(
        context.iter().any(|p| p.ends_with("impl.md")),
        "{context:?}"
    );
    assert!(
        context.iter().any(|p| p.ends_with("feature.md")),
        "{context:?}"
    );
    let graph = export_workspace_graph(dir.path(), dir.path().join("graph.md"), false).unwrap();
    let body = fs::read_to_string(graph).unwrap();
    assert!(body.contains("stable/impl"), "{body}");
    assert!(body.contains("related"), "{body}");
}

#[test]
fn code_refs_are_in_context_and_export() {
    let dir = tempdir();
    fs::write(
        dir.path().join("ods.toml"),
        "spec = \"0.1\"\n",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("src/routes")).unwrap();
    fs::write(
        dir.path().join("src/routes/checkout.tsx"),
        "export function CheckoutRoute() {}\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("checkout.md"),
        "---\nprofile: note\nstatus: draft\ncode:\n  - path: src/routes/checkout.tsx\n    symbol: CheckoutRoute\n    role: entrypoint\n---\n\n# Checkout\n",
    )
    .unwrap();

    let workspace = load_workspace(dir.path()).unwrap();
    let context = resolve_context(&workspace, "checkout", true);
    assert!(
        context
            .iter()
            .any(|p| p.ends_with("src/routes/checkout.tsx")),
        "{context:?}"
    );

    let graph = export_workspace_graph(dir.path(), dir.path().join("graph.md"), false).unwrap();
    let body = fs::read_to_string(graph).unwrap();
    assert!(body.contains("src/routes/checkout.tsx"), "{body}");
    assert!(body.contains("entrypoint"), "{body}");
    assert!(body.contains("#CheckoutRoute"), "{body}");
}

#[test]
fn code_files_are_not_indexed_as_document_children() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path();
    fs::write(
        root.join("ods.toml"),
        "spec = \"0.1\"
",
    )
    .unwrap();
    fs::write(
        root.join("doc.md"),
        "---
profile: note
status: draft
code:
  - path: src/main.rs
    role: entrypoint
---

# Doc
",
    )
    .unwrap();
    fs::create_dir_all(root.join("src")).unwrap();
    fs::write(
        root.join("src/main.rs"),
        "fn main() {}
",
    )
    .unwrap();
    let workspace = load_workspace(root).unwrap();
    assert!(workspace.document_by_path(&root.join("doc.md")).is_some());
    assert!(
        workspace
            .document_by_path(&root.join("src/main.rs"))
            .is_none()
    );
    assert!(workspace.code_paths.iter().any(|p| p.ends_with("main.rs")));
}

#[test]
fn duplicate_ids_and_missing_refs_are_reported() {
    let dir = tempdir();
    fs::write(dir.path().join("ods.toml"), "spec = \"0.1\"\n").unwrap();
    fs::write(
        dir.path().join("a.md"),
        "---\nprofile: note\nid: same\nstatus: draft\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("b.md"),
        "---\nprofile: note\nid: same\nstatus: draft\ndepends:\n  - missing\n---\n\n# B\n",
    )
    .unwrap();
    let workspace = load_workspace(dir.path()).unwrap();
    let messages = lint_workspace(&workspace)
        .into_iter()
        .map(|d| d.message)
        .collect::<Vec<_>>();
    assert!(
        messages.iter().any(|m| m.contains("duplicate document id")),
        "{messages:?}"
    );
    assert!(
        messages.iter().any(|m| m.contains("dangling reference")),
        "{messages:?}"
    );
}

#[test]
fn markdown_document_refs_resolve_and_canonical_lint_warns_on_legacy_ids() {
    let dir = tempdir();
    fs::write(
        dir.path().join("ods.toml"),
        "spec = \"0.1\"\n",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("website")).unwrap();
    fs::write(
        dir.path().join("website/cart-checkout.md"),
        "---\nprofile: note\nstatus: stable\n---\n\n# Checkout\n",
    )
    .unwrap();
    fs::write(
        dir.path().join("feature.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - website/cart-checkout.md\nrelated:\n  - website/cart-checkout\ncontext:\n  load:\n    - website/cart-checkout.md\n---\n\n# Feature\n",
    )
    .unwrap();

    /* indexes removed */
    let workspace = load_workspace(dir.path()).unwrap();
    let diags = lint_workspace(&workspace);
    assert!(diags.is_empty(), "{diags:?}");
    let strict = lint_workspace_with_ref_style(&workspace, LintLevel::Full, true);
    assert!(
        strict
            .iter()
            .any(|diag| diag.message.contains("prefer website/cart-checkout.md")),
        "{strict:?}"
    );
    let context = resolve_context(&workspace, "feature", true);
    assert!(
        context
            .iter()
            .any(|path| path.ends_with("cart-checkout.md")),
        "{context:?}"
    );
}

#[test]
fn fmt_md_paths_rewrites_document_refs_only() {
    let dir = tempdir();
    fs::write(
        dir.path().join("ods.toml"),
        "spec = \"0.1\"\n",
    )
    .unwrap();
    fs::create_dir_all(dir.path().join("website")).unwrap();
    fs::create_dir_all(dir.path().join("resources")).unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(
        dir.path().join("website/cart-checkout.md"),
        "---\nprofile: note\nstatus: stable\n---\n\n# Checkout\n",
    )
    .unwrap();
    fs::write(dir.path().join("resources/users.csv"), "id\n1\n").unwrap();
    fs::write(dir.path().join("src/app.ts"), "export {}\n").unwrap();
    fs::write(
        dir.path().join("feature.md"),
        "---\nprofile: note\nstatus: draft\nid: stable/feature\ndepends:\n  - website/cart-checkout\nrelated:\n  - website/cart-checkout.md\nresources:\n  - path: resources/users.csv\ncode:\n  - path: src/app.ts\n    role: implementation\ncontext:\n  load:\n    - website/cart-checkout\n    - resources/users.csv\n  ignore:\n    - archive/\n---\n\n# Feature\n",
    )
    .unwrap();

    let changed = canonicalize_workspace_document_refs(dir.path()).unwrap();
    assert_eq!(changed.len(), 1, "{changed:?}");
    let body = fs::read_to_string(dir.path().join("feature.md")).unwrap();
    assert!(body.contains("  - website/cart-checkout.md"), "{body}");
    assert!(body.contains("id: stable/feature"), "{body}");
    assert!(body.contains("path: resources/users.csv"), "{body}");
    assert!(body.contains("path: src/app.ts"), "{body}");
    assert!(body.contains("    - resources/users.csv"), "{body}");
    assert!(body.contains("    - archive/"), "{body}");
}
