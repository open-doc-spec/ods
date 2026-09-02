#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_basic() {
        assert_eq!(normalize_tag("  Billing ").as_deref(), Some("billing"));
        assert_eq!(normalize_tag("   "), None);
    }

    #[test]
    fn docs_with_all_tags_intersection() {
        use crate::model::{Document, Frontmatter, FrontmatterState, Workspace};
        use std::path::PathBuf;

        let mk = |path: &str, tags: &[&str]| Document {
            path: PathBuf::from(path),
            directory: PathBuf::from("."),
            body: String::new(),
            headings: Vec::new(),
            frontmatter: FrontmatterState::Parsed(Frontmatter {
                tags: tags.iter().map(|t| t.to_string()).collect(),
                ..Default::default()
            }),
        };
        let mut ws = Workspace::empty(PathBuf::from("."));
        ws.documents = vec![
            mk("a.md", &["auth", "billing"]),
            mk("b.md", &["auth"]),
            mk("c.md", &["billing", "oncall"]),
        ];
        for (i, doc) in ws.documents.iter().enumerate() {
            let id = crate::parse::document_id(
                &ws.root,
                &doc.path,
                match &doc.frontmatter {
                    FrontmatterState::Parsed(fm) => Some(fm),
                    _ => None,
                },
            );
            ws.by_id.insert(id.clone(), i);
            if let FrontmatterState::Parsed(fm) = &doc.frontmatter {
                for tag in &fm.tags {
                    ws.tag_index.entry(tag.clone()).or_default().push(id.clone());
                }
            }
        }

        assert!(docs_with_all_tags(&ws, &[]).is_empty());
        let only_auth = docs_with_all_tags(&ws, &["auth".into()]);
        assert_eq!(only_auth.len(), 2);
        let both = docs_with_all_tags(&ws, &["auth".into(), "billing".into()]);
        assert_eq!(both.len(), 1);
        let none = docs_with_all_tags(&ws, &["auth".into(), "missing".into()]);
        assert!(none.is_empty());
    }

    #[test]
    fn normalize_list_dedupes() {
        assert_eq!(
            normalize_tag_list(["Billing", "billing", "oncall", ""]),
            vec!["billing".to_string(), "oncall".to_string()]
        );
    }

    #[test]
    fn rewrite_list_tags() {
        let text = "---\nprofile: note\ntags:\n  - billing\n  - old-tag\n---\n\n# Doc\n";
        let out = rewrite_tags_in_text(text, "old-tag", "new-tag").unwrap();
        assert!(out.contains("- new-tag"), "{out}");
        assert!(!out.contains("old-tag"), "{out}");
    }

    #[test]
    fn rewrite_inline_tags_bracket() {
        let text = "---\ntags: [a, old, b]\n---\n\n# D\n";
        let out = rewrite_tags_in_text(text, "old", "new").unwrap();
        assert!(out.contains("new"), "{out}");
        assert!(!out.contains("old"), "{out}");
    }

    #[test]
    fn tag_suggestions_and_rewrite_edge_cases() {
        assert!(is_builtin_tag("oncall"));
        assert!(!is_builtin_tag("custom_tag_123"));
        assert!(!is_builtin_tag(""));

        // Single scalar inline
        let text_scalar = "---\ntags: old\n---\n";
        let out = rewrite_tags_in_text(text_scalar, "old", "new").unwrap();
        assert!(out.contains("tags: new"));

        // Unmatched inline bracket
        let text_unmatched = "---\ntags: [x, y]\n---\n";
        let out = rewrite_tags_in_text(text_unmatched, "old", "new").unwrap();
        assert_eq!(out, text_unmatched);

        // Quoted tag item and comments/empty line/next key under tags
        let text_complex = "---\ntags:\n  - \"old\"\n  - \nprofile: note\n---\n";
        let out = rewrite_tags_in_text(text_complex, "old", "new").unwrap();
        assert!(out.contains("- \"new\""));
    }

    #[test]
    fn tags_catalog_all_helpers_and_warnings_test() {
        let dir = ods_test_support::temp_workspace();
        std::fs::write(dir.join("index.md"), "---\nprofile: index\nods: 0.1\n---\n\n# R\n").unwrap();
        std::fs::write(
            dir.join("a.md"),
            "---\nprofile: note\ntags:\n  - \"tag with spaces\"\n  - draft\n  - feature\n---\n\n# A\n",
        )
        .unwrap();

        let ws = crate::fs::load_workspace(&dir).unwrap();

        assert_eq!(observed_tags(&ws), vec!["draft".to_string(), "feature".to_string(), "tag with spaces".to_string()]);
        assert!(!docs_with_any_tag(&ws, &["draft".to_string()]).is_empty());
        assert!(!tag_usage_with_builtins(&ws, true).is_empty());

        let doc_a = ws.document_by_path(&dir.join("a.md")).unwrap();
        let diags = lint_document_tags(doc_a, &ws);
        assert!(diags.iter().any(|d| d.message.contains("tag has spaces")));
        assert!(diags.iter().any(|d| d.message.contains("tag collides with status value")));
        assert!(diags.iter().any(|d| d.message.contains("tag collides with profile name")));
    }

    #[test]
    fn nested_tags_under_ods_are_misplaced_and_visible_until_migrate() {
        let dir = ods_test_support::temp_workspace();
        std::fs::write(
            dir.join("index.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
        )
        .unwrap();
        std::fs::write(
            dir.join("nested.md"),
            "---\nods:\n  profile: note\n  status: draft\n  tags:\n    - billing\n    - block\n---\n\n# Nested\n",
        )
        .unwrap();

        let ws = crate::fs::load_workspace(&dir).unwrap();
        let doc = ws.document_by_path(&dir.join("nested.md")).unwrap();
        let FrontmatterState::Parsed(fm) = &doc.frontmatter else {
            panic!("expected parsed frontmatter");
        };
        assert!(fm.tags_misplaced, "nested tags must set tags_misplaced");
        assert!(fm.tags.iter().any(|t| t == "billing"));
        assert!(fm.tags.iter().any(|t| t == "block"));
        // Temporary in-memory merge so find/tags can surface broken docs before repair.
        assert!(docs_with_tag(&ws, "billing").iter().any(|id| id.contains("nested")));

        let diags = lint_document_tags(doc, &ws);
        assert!(
            diags.iter().any(|d| d.message.contains("tags must be top-level")),
            "{diags:?}"
        );
    }

    #[test]
    fn migrate_hoists_nested_tags_to_root_without_dropping() {
        let text = "---\nods:\n  profile: note\n  status: draft\n  tags:\n    - billing\n    - block\n---\n\n# Doc\n";
        let out = crate::mv::migrate_frontmatter_to_canonical(text).expect("should hoist");
        assert!(out.contains("tags:\n  - billing\n  - block\n"), "{out}");
        assert!(out.contains("profile: note\nstatus: draft\n"), "{out}");
        assert!(!out.contains("ods:"), "ods: wrapper must be removed in 2.0 migrate: {out}");
        // Idempotent after hoist
        assert!(crate::mv::migrate_frontmatter_to_canonical(&out).is_none());
    }

    #[test]
    fn migrate_merges_root_and_nested_tags() {
        let text = "---\ntags:\n  - billing\nods:\n  profile: note\n  tags:\n    - block\n    - Billing\n---\n\n# Doc\n";
        let out = crate::mv::migrate_frontmatter_to_canonical(text).expect("should migrate");
        assert!(out.contains("tags:\n  - billing\n  - block\n"), "{out}");
        assert!(!out.contains("  tags:"), "no nested tags under ods: {out}");
    }
}
