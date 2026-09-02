use ods_core::{CodeRole, FrontmatterState, parse_document_text, split_frontmatter};
use std::path::PathBuf;

#[test]
fn split_frontmatter_absent() {
    let (fm, body) = split_frontmatter("# Hi\n");
    assert!(fm.is_none());
    assert!(body.starts_with("# Hi"));
}

#[test]
fn split_frontmatter_present() {
    let text = "---\nprofile: note\n---\n\n# Title\n";
    let (fm, body) = split_frontmatter(text);
    assert_eq!(fm.unwrap().trim(), "profile: note");
    assert!(body.contains("# Title"));
}

#[test]
fn parse_resources_path_only_ignores_type() {
    let root = PathBuf::from("/ws");
    let path = root.join("doc.md");
    let text = r#"---
profile: note
status: draft
resources:
  - path: ./data.csv
    type: csv
  - path: ./pic.png
---

# Doc
"#;
    let doc = parse_document_text(&root, path, text, true);
    match doc.frontmatter {
        FrontmatterState::Parsed(fm) => {
            assert_eq!(fm.resources.len(), 2);
            assert!(fm.resources[0].path.ends_with("data.csv"));
        }
        other => panic!("expected parsed: {other:?}"),
    }
}

#[test]
fn parse_context_block() {
    let root = PathBuf::from("/ws");
    let text = r#"---
profile: note
status: draft
context:
  max-depth: 2
  load:
    - a/b
  ignore:
    - archive/
---

# Doc
"#;
    let doc = parse_document_text(&root, root.join("d.md"), text, true);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!("parse failed");
    };
    let ctx = fm.context.expect("context");
    assert_eq!(ctx.max_depth, Some(2));
    assert_eq!(ctx.load, vec!["a/b".to_string()]);
    assert_eq!(ctx.ignore, vec!["archive/".to_string()]);
}

#[test]
fn parse_context_ignores_legacy_max_depth_key() {
    let root = PathBuf::from("/ws");
    let text = r#"---
profile: note
context:
  max_depth: 2
---

# Doc
"#;
    let doc = parse_document_text(&root, root.join("d.md"), text, true);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!("parse failed");
    };
    assert_eq!(fm.context.expect("context").max_depth, None);
}

#[test]
fn parse_code_refs_with_fixed_roles() {
    let root = PathBuf::from("/ws");
    let text = r#"---
profile: feature
status: draft
code:
  - path: src/routes/login.tsx
    symbol: LoginRoute
    role: Entrypoint
  - path: src/auth/session.rs
    symbol: create_session
    role: implementation
  - path: src/auth/session.test.ts
    role: test
  - path: src/schema/user.ts
    role: schema
  - path: db/migrations/001.sql
    role: migration
  - path: src/flags.ts
    role: config
  - path: infra/main.tf
    role: infrastructure
  - path: .github/workflows/ci.yml
    role: pipeline
---

# Feature
"#;
    let doc = parse_document_text(&root, root.join("feature.md"), text, true);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!("parse failed");
    };
    assert_eq!(fm.code.len(), 8);
    assert_eq!(fm.code[0].role, CodeRole::Entrypoint);
    assert_eq!(fm.code[0].symbol.as_deref(), Some("LoginRoute"));
    assert_eq!(fm.code[1].role.as_str(), "implementation");
    assert_eq!(fm.code[7].role, CodeRole::Pipeline);
}

#[test]
fn parse_code_refs_reject_missing_path_missing_role_and_invalid_role() {
    let root = PathBuf::from("/ws");
    for (text, expected) in [
        (
            "---\ncode:\n  - role: implementation\n---\n\n# D\n",
            "code entry missing path",
        ),
        (
            "---\ncode:\n  - path: src/a.ts\n    role: controller\n---\n\n# D\n",
            "invalid code role: controller",
        ),
    ] {
        let doc = parse_document_text(&root, root.join("d.md"), text, true);
        match doc.frontmatter {
            FrontmatterState::Invalid(message) => assert!(
                message.contains(expected),
                "expected {expected}, got {message}"
            ),
            other => panic!("expected invalid frontmatter: {other:?}"),
        }
    }
}

#[test]
fn parse_custom_keys_map() {
    let root = PathBuf::from("/ws");
    let text = r#"---
profile: index
aliases:
  Goal:
    - Mission
    - Objective
---

# Root
"#;
    let doc = parse_document_text(&root, root.join("index.md"), text, true);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!("parse failed");
    };
    assert!(fm.custom_keys.contains_key("aliases"));
}

#[test]
fn parse_ignore_and_profiles_lists() {
    let root = PathBuf::from("/ws");
    let text = r#"---
profile: index
ods: 0.1
profiles:
  - ods-profiles
ignore:
  - src
  - apps/web/
---

# Root
"#;
    let doc = parse_document_text(&root, root.join("index.md"), text, true);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!("parse failed");
    };
    assert_eq!(fm.ods.as_deref(), Some("0.1"));
    assert_eq!(fm.profiles, vec!["ods-profiles".to_string()]);
    assert_eq!(fm.ignore, vec!["src".to_string(), "apps/web".to_string()]);
}

#[test]
fn parse_created_and_updated_timestamps() {
    let root = PathBuf::from("/ws");
    let text = r#"---
profile: note
created: 2026-07-31
last_updated: 2026-07-31T17:00:00Z
---

# Doc
"#;
    let doc = parse_document_text(&root, root.join("doc.md"), text, true);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!("parse failed");
    };
    assert_eq!(fm.created.as_deref(), Some("2026-07-31"));
    assert_eq!(fm.updated.as_deref(), Some("2026-07-31T17:00:00Z"));
}

#[test]
fn parse_specs_frontmatter_block() {
    let root = PathBuf::from("/ws");
    let text = r#"---
ods: 0.1
specs:
  okf:
    enabled: true
    lint:
      check_keys: false
      ignore_keys:
        - runtime
        - sources
  skills:
    enabled: true
    lint:
      check_keys: true
---

# Root Index
"#;
    let doc = parse_document_text(&root, root.join("index.ods.md"), text, true);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!("parse failed");
    };
    assert!(fm.specs.okf.enabled);
    assert!(!fm.specs.okf.check_keys);
    assert!(fm.specs.okf.ignore_keys.contains("runtime"));
    assert!(fm.specs.okf.ignore_keys.contains("sources"));
    assert!(fm.specs.skills.enabled);
    assert!(fm.specs.skills.check_keys);
}

#[test]
fn test_hybrid_frontmatter_preservation_on_mutation() {
    let root = PathBuf::from("/ws");
    let text = r#"---
profile: note
status: draft
layout: post
author: Alice
tags:
  - rust
  - ods
---

# My Post
"#;
    let doc = parse_document_text(&root, root.join("post.md"), text, true);
    let FrontmatterState::Parsed(fm) = doc.frontmatter else {
        panic!("parse failed");
    };

    // Verify third-party custom keys parsed into custom_keys map
    assert!(fm.custom_keys.contains_key("layout"));
    assert_eq!(fm.author.as_deref(), Some("Alice"));
    assert_eq!(fm.status.as_deref(), Some("draft"));

    // Verify fuzzy typo diagnostic checks
    let issues = ods_core::validate_ods_frontmatter(&fm);
    // Hugo layout & author keys are preserved without non-matching typo false positives
    assert!(issues.iter().all(|i| !i.message.contains("layout")));
}

#[test]
fn tag_rename_and_spacing_preserve_third_party_keys() {
    let text = "---\nlayout: post\nauthor: Alice\ntags:\n  - rust\n  - ods\nods:\n  profile: note\n  status: draft\n---\n\n# My Post\n";
    let renamed = ods_core::rewrite_tags_in_text(text, "rust", "systems").expect("rewrite");
    assert!(renamed.contains("layout: post"), "{renamed}");
    assert!(renamed.contains("author: Alice"), "{renamed}");
    assert!(renamed.contains("- systems"), "{renamed}");
    assert!(!renamed.contains("- rust\n"), "{renamed}");

    let spaced = ods_core::normalize_frontmatter_body_spacing(text);
    assert!(spaced.contains("layout: post"), "{spaced}");
    assert!(spaced.contains("author: Alice"), "{spaced}");
    assert!(spaced.contains("tags:\n  - rust\n  - ods\n"), "{spaced}");
}

#[test]
fn strip_ods_keeps_third_party_keys() {
    let text = "---\nlayout: post\nx-team: eng\nprofile: note\nstatus: draft\ndepends:\n  - other\n---\n\n# Body\n";
    let (next, changed) = ods_core::strip_ods_from_document_text(text, true, false);
    assert!(changed);
    assert!(next.contains("layout: post"), "{next}");
    assert!(next.contains("x-team: eng"), "{next}");
    assert!(!next.contains("profile:"), "{next}");
    assert!(!next.contains("depends:"), "{next}");
    assert!(next.contains("# Body"), "{next}");
}
