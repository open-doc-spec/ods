#[cfg(test)]
mod parse_v02_tests {
    use super::*;

    #[test]
    fn test_parse_share_and_packs() {
        let text = r#"---
profile: guide
status: stable
share: private
packs:
  - vendor/engineering-pack
  - ../shared-pack
---
# Test Doc
"#;
        let doc = parse_document_text(Path::new("/workspace"), PathBuf::from("/workspace/doc.md"), text, true);
        if let crate::model::FrontmatterState::Parsed(fm) = doc.frontmatter {
            assert_eq!(fm.share.as_deref(), Some("private"));
            assert_eq!(fm.packs, vec!["vendor/engineering-pack", "../shared-pack"]);
        } else {
            panic!("expected parsed frontmatter");
        }
    }

    #[test]
    fn test_parse_share_org() {
        let text = r#"---
profile: decision
status: stable
share: org
---
# Internal Decision
"#;
        let doc = parse_document_text(Path::new("/workspace"), PathBuf::from("/workspace/doc.md"), text, true);
        if let crate::model::FrontmatterState::Parsed(fm) = doc.frontmatter {
            assert_eq!(fm.share.as_deref(), Some("org"));
        } else {
            panic!("expected parsed frontmatter");
        }
    }

    #[test]
    fn test_parse_custom_keys_scalar_list_and_known_keys() {
        use crate::model::CustomValue;
        let text = r#"---
profile: note
status: draft
Team: Infra
stack: [rust, go]
labels:
  - a
  - b
nested:
  child: 1
depends:
  - other.md
---
# Body stays
"#;
        let doc = parse_document_text(
            Path::new("/workspace"),
            PathBuf::from("/workspace/doc.md"),
            text,
            true,
        );
        let crate::model::FrontmatterState::Parsed(fm) = doc.frontmatter else {
            panic!("expected parsed frontmatter");
        };
        // Known keys still first-class.
        assert_eq!(fm.profile.as_deref(), Some("note"));
        assert_eq!(fm.status.as_deref(), Some("draft"));
        assert_eq!(fm.depends, vec!["other.md".to_string()]);
        // Custom keys case-folded; last wins for duplicates is N/A here.
        assert_eq!(
            fm.custom_keys.get("team"),
            Some(&CustomValue::String("Infra".into()))
        );
        assert_eq!(
            fm.custom_keys.get("stack"),
            Some(&CustomValue::List(vec!["rust".into(), "go".into()]))
        );
        assert_eq!(
            fm.custom_keys.get("labels"),
            Some(&CustomValue::List(vec!["a".into(), "b".into()]))
        );
        // Nested maps are not queryable as strings, but remain present values.
        assert_eq!(fm.custom_keys.get("nested"), Some(&CustomValue::Opaque));
        assert!(doc.body.contains("Body stays"));
    }

    #[test]
    fn test_parse_custom_key_empty_and_duplicate_last_wins() {
        use crate::model::CustomValue;
        let text = r#"---
profile: note
emptykey:
team: first
team: second
---
# D
"#;
        let doc = parse_document_text(
            Path::new("/workspace"),
            PathBuf::from("/workspace/doc.md"),
            text,
            true,
        );
        let crate::model::FrontmatterState::Parsed(fm) = doc.frontmatter else {
            panic!("expected parsed");
        };
        assert_eq!(fm.custom_keys.get("emptykey"), Some(&CustomValue::Null));
        assert_eq!(
            fm.custom_keys.get("team"),
            Some(&CustomValue::String("second".into()))
        );
    }
}
