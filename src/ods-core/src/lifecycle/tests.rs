/// Strip ODS keys from a full document text.
/// Returns (new_text, changed).
///
/// Key lists are **schema-driven** (`SpecSchema::document_disable_strip_keys` /
/// `workspace_policy_strip_keys`) so disable never invents ad-hoc SSG key names.
pub fn strip_ods_from_document_text(
    text: &str,
    strip_doc_keys: bool,
    strip_root_keys: bool,
) -> (String, bool) {
    let (fm, body) = split_frontmatter(text);
    let Some(fm) = fm else {
        return (text.to_string(), false);
    };

    let registry = crate::spec::SpecSchemaRegistry::with_defaults();
    let schema = registry
        .get("ods")
        .expect("default ODS schema is always registered");
    let owned: Vec<String> = match (strip_doc_keys, strip_root_keys) {
        (false, false) => Vec::new(),
        (true, false) => schema.document_disable_strip_keys(),
        (false, true) => schema.workspace_policy_strip_keys(),
        (true, true) => {
            let mut v = schema.document_disable_strip_keys();
            v.extend(schema.workspace_policy_strip_keys());
            v.sort();
            v.dedup();
            v
        }
    };
    if owned.is_empty() {
        return (text.to_string(), false);
    }
    let drop_keys: Vec<&str> = owned.iter().map(String::as_str).collect();

    let (kept, removed_any) = strip_keys_from_frontmatter_block(fm, &drop_keys);
    if !removed_any {
        return (text.to_string(), false);
    }

    let ending = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let new_text = if kept.trim().is_empty() {
        // Body only; preserve leading blank handling similar to writers
        body.to_string()
    } else {
        let body = body.trim_start_matches(['\r', '\n']);
        if body.is_empty() {
            format!("---{ending}{kept}{ending}---{ending}")
        } else {
            format!("---{ending}{kept}{ending}---{ending}{ending}{body}")
        }
    };
    (new_text, true)
}

fn strip_keys_from_frontmatter_block(block: &str, drop_keys: &[&str]) -> (String, bool) {
    let lines: Vec<&str> = block.lines().collect();
    let mut out: Vec<String> = Vec::new();
    let mut i = 0usize;
    let mut removed = false;

    while i < lines.len() {
        let raw = lines[i];
        let trimmed = raw.trim();
        if trimmed.is_empty() {
            out.push(raw.to_string());
            i += 1;
            continue;
        }

        // Nested list continuation under a dropped key is handled by skip_block
        if let Some((key, rest)) = trimmed.split_once(':') {
            let key = key.trim();
            let indent = raw.len() - raw.trim_start().len();
            if drop_keys.contains(&key) {
                removed = true;
                i += 1;
                // Skip nested lines more indented, or list items under this key
                while i < lines.len() {
                    let nraw = lines[i];
                    let ntrim = nraw.trim();
                    if ntrim.is_empty() {
                        i += 1;
                        continue;
                    }
                    let nindent = nraw.len() - nraw.trim_start().len();
                    if nindent > indent {
                        i += 1;
                        continue;
                    }
                    // same indent list? only if previous was empty rest and line is `- `
                    break;
                }
                // If scalar was inline empty and next lines are `- ` at indent+2 handled above
                let _ = rest;
                continue;
            }
        }

        out.push(raw.to_string());
        i += 1;
    }

    // Drop trailing empty lines in frontmatter
    while out.last().is_some_and(|l| l.trim().is_empty()) {
        out.pop();
    }

    (out.join("\n"), removed)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_removes_ods_keys_keeps_body_and_custom() {
        let text = "---\nprofile: note\nstatus: draft\nx-team: a\ndepends:\n  - other\n---\n\n# Hi\n\nBody.\n";
        let (next, changed) = strip_ods_from_document_text(text, true, false);
        assert!(changed);
        assert!(next.contains("x-team: a"));
        assert!(!next.contains("profile:"));
        assert!(!next.contains("depends:"));
        assert!(next.contains("# Hi"));
        assert!(next.contains("Body."));
    }

    #[test]
    fn strip_empty_frontmatter_removes_block() {
        let text = "---\nprofile: note\n---\n\n# Only body\n";
        let (next, changed) = strip_ods_from_document_text(text, true, false);
        assert!(changed);
        assert!(!next.contains("---"));
        assert!(next.contains("# Only body"));
    }

    #[test]
    fn test_scaffold_new_document_and_atomic_delete() {
        let temp_dir = std::env::temp_dir().join(format!("ods_test_lifecycle_{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&temp_dir);
        std::fs::create_dir_all(&temp_dir).unwrap();

        let _ = crate::init_workspace(&temp_dir, crate::InitOptions::default()).unwrap();

        let target_path = temp_dir.join("docs/guides/setup-guide.md");
        let report = crate::scaffold_new_document(&temp_dir, &target_path, crate::NewDocumentOptions::default()).unwrap();

        assert_eq!(report.profile, "guide");
        assert!(report.created_file.exists());
        let text = std::fs::read_to_string(&report.created_file).unwrap();
        assert!(text.contains("description: Draft guide for"));
        assert!(text.contains("ods:\n  profile: guide\n  status: draft"));
        assert!(!text.starts_with("---\nprofile:"));
        assert!(text.contains("## Overview"));

        let (frontmatter, _) = crate::split_frontmatter(&text);
        let document = crate::parse_document_text(&temp_dir, report.created_file.clone(), &text, false);
        match document.frontmatter {
            crate::FrontmatterState::Parsed(fm) => {
                assert_eq!(fm.profile.as_deref(), Some("guide"));
                assert_eq!(fm.status.as_deref(), Some("draft"));
            }
            other => panic!("expected parsed frontmatter, got {other:?}"),
        }
        assert!(frontmatter.is_some());

        let agent_path = temp_dir.join("docs/agents/system-agent.md");
        let agent_report = crate::scaffold_new_document(
            &temp_dir,
            &agent_path,
            crate::NewDocumentOptions::default(),
        )
        .unwrap();
        assert_eq!(agent_report.profile, "agent");
        let agent_text = std::fs::read_to_string(&agent_report.created_file).unwrap();
        assert!(agent_text.contains("ods:\n  profile: agent\n  status: draft"));
        assert!(agent_text.contains("## Task"));
        assert!(agent_text.contains("## Success Criteria"));

        let remove_report = crate::atomic_delete_document(&temp_dir, &target_path, crate::RemoveDocumentOptions::default()).unwrap();
        assert!(!target_path.exists());
        assert_eq!(remove_report.doc_id, "docs/guides/setup-guide");

        let _ = std::fs::remove_dir_all(&temp_dir);
    }
}
