use ods_core::{
    FrontmatterState, document_id, extract_heading_groups, extract_headings, parse_document_text,
    split_frontmatter, split_markdown_link_target,
};
use std::path::PathBuf;

#[test]
fn document_id_path_and_explicit() {
    let root = PathBuf::from("/ws");
    let path = root.join("features/login.md");
    let id = document_id(&root, &path, None);
    assert_eq!(id, "features/login");

    let text = "---\nid: stable-login\nprofile: note\n---\n\n# X\n";
    let doc = parse_document_text(&root, path.clone(), text, true);
    let FrontmatterState::Parsed(fm) = &doc.frontmatter else {
        panic!();
    };
    assert_eq!(document_id(&root, &path, Some(fm)), "stable-login");
}

#[test]
fn extract_headings_and_groups() {
    let body = "# T\n\n## Goal\n\n## Scope\n";
    assert_eq!(extract_headings(body), vec!["Goal", "Scope"]);
    let groups = extract_heading_groups(body);
    assert_eq!(groups[0], vec!["Goal"]);
}

#[test]
fn markdown_link_target() {
    assert_eq!(
        split_markdown_link_target("- [x](foo/bar.md)"),
        Some("foo/bar.md".to_string())
    );
    assert!(split_markdown_link_target("no link").is_none());
}

#[test]
fn status_normalized_lowercase() {
    let root = PathBuf::from("/ws");
    let text = "---\nprofile: Note\nstatus: Draft\n---\n\n# D\n";
    let doc = parse_document_text(&root, root.join("d.md"), text, true);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!();
    };
    assert_eq!(fm.profile.as_deref(), Some("note"));
    assert_eq!(fm.status.as_deref(), Some("draft"));
}

#[test]
fn include_body_false_still_has_headings() {
    let root = PathBuf::from("/ws");
    let text = "---\nprofile: note\n---\n\n# T\n\n## Overview\n";
    let doc = parse_document_text(&root, root.join("d.md"), text, false);
    assert!(doc.body.is_empty());
    assert_eq!(doc.headings, vec!["Overview"]);
}

#[test]
fn test_parse_pattern_b_nested_ods_map() {
    let root = PathBuf::from("/ws");
    let text = r#"---
description: Refund processing guide
tags:
  - billing
  - support
owner:
  - support-team
  - billing-ops

ods:
  profile: guide
  status: stable
  id: refund-flow
  share: public
  depends:
    - ../checkout/cart.md
  related:
    - ../policy/faq.md
  resources:
    - path: docs/flow.pdf
  code:
    - path: apps/web/src/refund.ts
      role: implementation
      symbol:
        - processRefund
        - validateRefund
  context:
    max-depth: 2
    load:
      - ../checkout/cart.md
    ignore:
      - archive/
---

# Refund Processing Guide
"#;
    let doc = parse_document_text(&root, root.join("refund.md"), text, true);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!("expected parsed frontmatter, got {:?}", doc.frontmatter);
    };

    assert_eq!(fm.description.as_deref(), Some("Refund processing guide"));
    assert_eq!(fm.tags, vec!["billing", "support"]);
    assert_eq!(fm.owner.as_deref(), Some("support-team, billing-ops"));
    assert_eq!(fm.profile.as_deref(), Some("guide"));
    assert_eq!(fm.status.as_deref(), Some("stable"));
    assert_eq!(fm.id.as_deref(), Some("refund-flow"));
    assert_eq!(fm.share.as_deref(), Some("public"));
    assert_eq!(fm.depends, vec!["../checkout/cart.md"]);
    assert_eq!(fm.related, vec!["../policy/faq.md"]);
    assert_eq!(fm.resources.len(), 1);
    assert_eq!(fm.code.len(), 1);
    assert_eq!(
        fm.code[0].symbol.as_deref(),
        Some("processRefund, validateRefund")
    );
    assert_eq!(fm.context.expect("context").max_depth, Some(2));
}

#[test]
fn nested_ods_block_tolerates_key_order() {
    let root = PathBuf::from("/ws");
    let text = "---\nods:\n  status: stable\n  depends:\n    - ../a.md\n  profile: guide\n  share: public\n---\n\n# Doc\n";
    let doc = parse_document_text(&root, root.join("doc.md"), text, false);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!("expected parsed frontmatter, got {:?}", doc.frontmatter);
    };
    assert_eq!(fm.profile.as_deref(), Some("guide"));
    assert_eq!(fm.status.as_deref(), Some("stable"));
    assert_eq!(fm.share.as_deref(), Some("public"));
    assert_eq!(fm.depends, vec!["../a.md"]);
}

#[test]
fn frontmatter_parser_exhaustive_coverage() {
    let td = tempfile::tempdir().unwrap();
    let root = td.path();
    let path = root.join("test.md");
    std::fs::write(&path, "---\nprofile: note\nowner:\n  - alice\n  - bob\nunknown_key: val\nods:\n  profile: note\n  unknown_sub: subval\n---\n\n# Body\n").unwrap();

    let doc = ods_core::parse::parse_document(root, path).unwrap();
    if let FrontmatterState::Parsed(fm) = doc.frontmatter {
        assert_eq!(fm.owner.as_deref(), Some("alice, bob"));
    } else {
        panic!("expected parsed frontmatter");
    }

    let (fm, _) = split_frontmatter("---extra\nprofile: note\n---\n");
    assert!(fm.is_none());

    let (fm, body) = split_frontmatter("---\nprofile: note\n");
    assert!(fm.is_some());
    assert!(body.is_empty());

    let (fm, body) = split_frontmatter("---\r\nprofile: note\r\n---\r\n");
    assert!(fm.is_some());
    assert!(body.is_empty());

    let doc_bad = parse_document_text(root, root.join("bad.md"), "---\nno_colon_line\n---\n", true);
    assert!(matches!(doc_bad.frontmatter, FrontmatterState::Invalid(_)));

    let doc_nested_bad = parse_document_text(
        root,
        root.join("nbad.md"),
        "---\nods:\n  profile: note\n\n  no_colon_inner\n---\n",
        true,
    );
    assert!(matches!(
        doc_nested_bad.frontmatter,
        FrontmatterState::Invalid(_)
    ));
}

#[test]
fn test_frontmatter_title_permitted() {
    let root = PathBuf::from("/ws");
    let text = "---\ntitle: Valid Title Key\nprofile: guide\n---\n\n# Document Header\n";
    let doc = parse_document_text(&root, root.join("doc.md"), text, true);
    match doc.frontmatter {
        FrontmatterState::Parsed(fm) => {
            assert_eq!(fm.title.as_deref(), Some("Valid Title Key"));
            assert_eq!(fm.profile.as_deref(), Some("guide"));
        }
        other => panic!("expected parsed frontmatter with title key, got {other:?}"),
    }
}
