fn lint_profile_sections_direct(
    document: &Document,
    workspace: &Workspace,
    profile: &str,
) -> Vec<Diagnostic> {
    let expected = profile_sections(workspace, profile);
    if expected.is_empty() {
        return Vec::new();
    }

    let headings = document
        .headings
        .iter()
        .map(|heading| normalize_heading(heading))
        .collect::<BTreeSet<_>>();

    expected
        .iter()
        .filter_map(|group| group.first())
        .filter(|canonical| {
            let normalized = normalize_heading(canonical);
            !headings.contains(&normalized)
        })
        .map(|canonical| Diagnostic {
            path: document.path.clone(),
            severity: Severity::Warning,
            message: crate::error::lint_missing_expected_section(canonical),
        })
        .collect()
}

fn lint_resources(document: &Document, frontmatter: &crate::model::Frontmatter) -> Vec<Diagnostic> {
    frontmatter
        .resources
        .iter()
        .filter_map(|resource| {
            let path = normalize_join(&document.directory, &resource.path);
            (!path.exists()).then(|| Diagnostic {
                path: document.path.clone(),
                severity: Severity::Error,
                message: crate::error::lint_missing_resource(resource.path.display()),
            })
        })
        .collect()
}

fn lint_code_refs(document: &Document, frontmatter: &crate::model::Frontmatter) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for code in &frontmatter.code {
        let path_str = code.path.to_string_lossy();
        if path_str.contains(":L") || path_str.contains(":line") {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Error,
                message: crate::error::lint_code_path_line_suffix(code.path.display()),
            });
            continue;
        }
        let path = normalize_join(&document.directory, &code.path);
        if !path.exists() {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Error,
                message: crate::error::lint_missing_code_path(code.path.display()),
            });
        }
    }
    diagnostics
}



/// Links from top-level markdown list items only (`- [label](target)`).
fn extract_markdown_links(body: &str) -> BTreeSet<String> {
    let mut links = BTreeSet::new();
    let mut in_code_block = false;
    for line in body.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("```") {
            in_code_block = !in_code_block;
            continue;
        }
        if !in_code_block
            && let Some(target) = split_markdown_link_target(line)
        {
            links.insert(target);
        }
    }
    links
}

fn lint_body_links(document: &Document) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let links = extract_markdown_links(&document.body);

    for link in links {
        if link.starts_with("http://")
            || link.starts_with("https://")
            || link.starts_with("mailto:")
            || link.starts_with("ws://")
            || link.starts_with("wss://")
        {
            continue;
        }

        if link.starts_with('#') {
            continue;
        }

        let path_part = link.split('#').next().unwrap_or(&link);
        if path_part.is_empty() {
            continue;
        }

        let decoded_path = path_part.replace("%20", " ");

        let target_path = normalize_join(&document.directory, Path::new(&decoded_path));
        if !target_path.exists() {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Error,
                message: crate::error::lint_dangling_body_link(&link),
            });
        }
    }

    diagnostics
}

fn normalize_heading(text: &str) -> String {
    text.chars()
        .filter(|ch| ch.is_ascii_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

#[cfg(test)]
mod test_lint_helpers {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_lint_helpers() {
        assert_eq!(normalize_heading("  ## Section Name!  "), "sectionname");

        let body = "- [Link](nonexistent.md)\n- [Web](https://example.com)\n";
        let links = extract_markdown_links(body);
        assert!(links.contains("nonexistent.md"));
        assert!(links.contains("https://example.com"));

        let doc = Document {
            path: PathBuf::from("/tmp/doc.md"),
            directory: PathBuf::from("/tmp"),
            body: body.to_string(),
            headings: vec!["# Section Name".into()],
            frontmatter: crate::model::FrontmatterState::Absent,
        };

        let diags = lint_body_links(&doc);
        assert!(diags.iter().any(|d| d.message.contains("dangling")));
    }
}
