use crate::model::{Document, FrontmatterState, ProfileCatalog, ProfileDefinition};
use crate::parse::{extract_heading_groups, parse_document_text};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub fn standard_profile_catalog() -> ProfileCatalog {
    let mut catalog = ProfileCatalog::default();
    for definition in standard_profile_definitions() {
        catalog
            .definitions
            .insert(definition.name.clone(), definition);
    }
    catalog
}

pub fn profile_catalog_roots(root: &Path, root_index: Option<&Document>) -> Vec<PathBuf> {
    if let Some(root_index) = root_index
        && let FrontmatterState::Parsed(frontmatter) = &root_index.frontmatter
    {
        let cfg = crate::config::WorkspaceConfig {
            custom_profiles: frontmatter.profiles.clone(),
            packs: frontmatter.packs.clone(),
            ..Default::default()
        };
        return profile_catalog_roots_from_config(root, &cfg);
    }
    if let Ok(cfg) = crate::config::load_workspace_config(root) {
        return profile_catalog_roots_from_config(root, &cfg);
    }
    Vec::new()
}

/// Profile catalog roots from `ods.toml` (`custom_profiles` + pack `ods-profiles`).
pub fn profile_catalog_roots_from_config(
    root: &Path,
    config: &crate::config::WorkspaceConfig,
) -> Vec<PathBuf> {
    let mut roots = Vec::new();

    roots.extend(config.custom_profiles.iter().map(|path| {
        let p = root.join(path);
        p.canonicalize().unwrap_or(p)
    }));

    for pack_ref in &config.packs {
        let pack_dir = root.join(pack_ref);
        let pack_dir = pack_dir.canonicalize().unwrap_or(pack_dir);
        let pack_profiles = pack_dir.join("ods-profiles");
        if pack_profiles.exists() {
            roots.push(pack_profiles);
        } else if pack_dir.exists() {
            roots.push(pack_dir);
        }
    }

    roots.sort();
    roots.dedup();
    roots
}

pub fn load_profile_catalog(root: &Path, roots: &[PathBuf]) -> io::Result<ProfileCatalog> {
    let mut catalog = standard_profile_catalog();

    for profile_root in roots {
        if !profile_root.exists() {
            continue;
        }

        let mut paths = Vec::new();
        if profile_root.is_file() {
            paths.push(profile_root.clone());
        } else {
            collect_markdown_paths(profile_root, &mut paths)?;
            paths.sort();
        }

        for path in paths {
            let text = fs::read_to_string(&path)?;
            let document = parse_document_text(root, path.clone(), &text, true);
            if let Some(definition) = profile_definition_from_document(&document) {
                if let Some(existing) = catalog.definitions.get(&definition.name) {
                    catalog.conflicts.push(crate::model::ProfileConflict {
                        name: definition.name.clone(),
                        kept: existing.source.clone(),
                        ignored: definition.source.clone(),
                    });
                } else {
                    catalog
                        .definitions
                        .insert(definition.name.clone(), definition);
                }
            }
        }
    }

    Ok(catalog)
}

fn collect_markdown_paths(dir: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
    if dir.is_file() {
        if dir.extension().is_some_and(|ext| ext == "md") {
            out.push(dir.to_path_buf());
        }
        return Ok(());
    }

    let mut entries = fs::read_dir(dir)?.collect::<Result<Vec<_>, _>>()?;
    entries.sort_by_key(|entry| entry.file_name());

    for entry in entries {
        let path = entry.path();
        let file_name = entry.file_name();
        let file_type = entry.file_type()?;

        if should_ignore_name(&file_name) {
            continue;
        }

        if file_type.is_dir() {
            collect_markdown_paths(&path, out)?;
        } else if file_type.is_file() && path.extension().is_some_and(|ext| ext == "md") {
            out.push(path);
        }
    }

    Ok(())
}

fn should_ignore_name(name: &std::ffi::OsStr) -> bool {
    let text = name.to_string_lossy();
    text.starts_with('.') || text == "target"
}

fn profile_definition_from_document(document: &Document) -> Option<ProfileDefinition> {
    let mut name = profile_name_from_path(&document.path)?;
    let mut required_keys = Vec::new();
    let mut optional_keys = Vec::new();
    let mut forbidden_keys = Vec::new();
    let mut sections = extract_heading_groups(&document.body);

    if let FrontmatterState::Parsed(frontmatter) = &document.frontmatter {
        if let Some(definition) = &frontmatter.custom_profile {
            if let Some(explicit_name) = &definition.name {
                name = explicit_name.clone();
            }
            required_keys.clone_from(&definition.required_keys);
            optional_keys.clone_from(&definition.optional_keys);
            forbidden_keys.clone_from(&definition.forbidden_keys);
        }
        for (canonical, aliases) in &frontmatter.aliases {
            if let Some(group) = sections
                .iter_mut()
                .find(|group| group.first() == Some(canonical))
            {
                for alias in aliases {
                    if !group.contains(alias) {
                        group.push(alias.clone());
                    }
                }
            } else {
                let mut group = vec![canonical.clone()];
                for alias in aliases {
                    if !group.contains(alias) {
                        group.push(alias.clone());
                    }
                }
                sections.push(group);
            }
        }
    }

    Some(ProfileDefinition {
        name,
        sections,
        required_keys,
        optional_keys,
        forbidden_keys,
        source: document.path.clone(),
    })
}

fn profile_name_from_path(path: &Path) -> Option<String> {
    let stem = path.file_stem()?.to_str()?.to_string();
    if stem == "index" {
        path.parent()
            .and_then(|parent| parent.file_name())
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
            .or(Some(stem))
    } else {
        Some(stem)
    }
}

fn standard_profile_definitions() -> Vec<ProfileDefinition> {
    vec![
        profile("note", vec![]),
        profile(
            "agent",
            vec![
                section(&["Goal", "Objective", "Purpose"]),
                section(&["Task", "Work", "Assignment"]),
                section(&["Scope", "In Scope", "Boundaries"]),
                section(&["Non-Scope", "Out of Scope", "Exclusions"]),
                section(&["Context", "Background"]),
                section(&["Inputs", "Source Material"]),
                section(&["Constraints", "Rules", "Limits"]),
                section(&["Priority", "Order"]),
                section(&["Steps", "Workflow", "Procedure", "Process"]),
                section(&["Output", "Deliverable", "Result"]),
                section(&[
                    "Success Criteria",
                    "Acceptance Criteria",
                    "Done When",
                    "Definition of Done",
                ]),
                section(&["Failure Modes", "Risks", "Edge Cases", "Fallbacks"]),
                section(&["Dependencies", "Prerequisites", "Blockers"]),
                section(&["Assumptions", "Unknowns"]),
                section(&["Examples", "Sample", "Examples"]),
            ],
        ),
        profile(
            "feature",
            vec![
                section(&["Goal", "Objective", "Objectives", "Purpose"]),
                section(&["Scope", "In Scope", "Boundaries"]),
                section(&["Requirements", "Functional Requirements", "Needs"]),
                section(&[
                    "Acceptance Criteria",
                    "Acceptance",
                    "Success Criteria",
                    "Definition of Done",
                ]),
                section(&["Risks", "Risks and Mitigations", "Concerns"]),
            ],
        ),
        profile(
            "guide",
            vec![
                section(&["Overview", "Introduction", "Summary", "Background"]),
                section(&["Prerequisites", "Requirements", "Before You Begin"]),
                section(&["Steps", "Instructions", "Procedure", "Process"]),
                section(&["Troubleshooting", "Common Issues", "FAQ"]),
            ],
        ),
        profile(
            "api",
            vec![
                section(&["Overview", "Introduction", "Summary", "Background"]),
                section(&["Request"]),
                section(&["Response"]),
                section(&["Errors"]),
                section(&["Examples"]),
            ],
        ),
        profile(
            "architecture",
            vec![
                section(&["Overview", "Introduction", "Summary", "Background"]),
                section(&["Components"]),
                section(&["Data Flow"]),
                section(&["Trade-offs", "Tradeoffs", "Pros and Cons"]),
            ],
        ),
        profile(
            "decision",
            vec![
                section(&["Context", "Background"]),
                section(&["Decision"]),
                section(&["Alternatives", "Options", "Options Considered"]),
                section(&["Consequences", "Outcome", "Implications"]),
            ],
        ),
        profile(
            "sop",
            vec![
                section(&["Purpose"]),
                section(&["Prerequisites", "Requirements", "Before You Begin"]),
                section(&["Steps", "Instructions", "Procedure", "Process"]),
                section(&["Validation", "Verification", "Checks"]),
                section(&["Rollback", "Recovery", "Revert"]),
            ],
        ),
        profile(
            "policy",
            vec![
                section(&["Purpose"]),
                section(&["Scope"]),
                section(&["Rules", "Standards", "Requirements"]),
                section(&["Exceptions"]),
            ],
        ),
        profile(
            "meeting",
            vec![
                section(&["Attendees"]),
                section(&["Agenda"]),
                section(&["Decisions"]),
                section(&["Action Items", "Actions", "Next Steps", "TODO"]),
            ],
        ),
        profile("faq", vec![]),
        profile(
            "checklist",
            vec![
                section(&["Overview", "Purpose", "Introduction", "Summary"]),
                section(&["Items", "Checklist", "Tasks", "Steps"]),
                section(&[
                    "Verification",
                    "Done When",
                    "Acceptance",
                    "Definition of Done",
                    "Checks",
                ]),
                section(&["Notes", "Exceptions", "Caveats", "References"]),
            ],
        ),
        profile("index", vec![]),
    ]
}

fn profile(name: &str, sections: Vec<Vec<&str>>) -> ProfileDefinition {
    ProfileDefinition {
        name: name.to_string(),
        sections: sections
            .into_iter()
            .map(|group| group.into_iter().map(|value| value.to_string()).collect())
            .collect(),
        required_keys: vec![],
        optional_keys: vec![],
        forbidden_keys: vec![],
        source: PathBuf::from(format!("<builtin:{name}>")),
    }
}

fn section<'a>(values: &'a [&'a str]) -> Vec<&'a str> {
    values.to_vec()
}

pub fn render_profile_template(
    catalog: &ProfileCatalog,
    profile_name: &str,
    title: &str,
) -> Result<String, String> {
    let definition = catalog.definitions.get(profile_name).ok_or_else(|| {
        let mut available: Vec<String> = catalog.definitions.keys().cloned().collect();
        available.sort();
        format!(
            "unknown profile '{profile_name}'. Available profiles: {}",
            available.join(", ")
        )
    })?;

    let mut out = format!(
        "---\nods:\n  profile: {}\n  status: draft\n---\n\n# {}\n\n",
        definition.name, title
    );
    for group in &definition.sections {
        if let Some(canonical) = group.first() {
            out.push_str(&format!("## {}\n\n", canonical));
        }
    }
    Ok(out)
}

pub fn resolve_document_profile<'a>(doc: &'a Document, catalog: &'a ProfileCatalog) -> &'a str {
    // Tier 1: Explicit Frontmatter Profile
    if let FrontmatterState::Parsed(fm) = &doc.frontmatter {
        if let Some(ref p) = fm.profile {
            if catalog.definitions.contains_key(p.as_str()) {
                return p.as_str();
            }
        }
    }

    // Tier 2: Directory Path Convention Mapping
    if let Some(parent) = doc.path.parent() {
        let folder_name = parent
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_lowercase();
        match folder_name.as_str() {
            "agent" | "agents" | "prompt" | "prompts" | "subagent" | "subagents" => return "agent",
            "adrs" | "decisions" => return "decision",
            "features" | "prds" => return "feature",
            "apis" | "endpoints" => return "api",
            "sops" | "runbooks" => return "sop",
            "rfcs" => return "rfc",
            "guides" | "tutorials" => return "guide",
            "policies" => return "policy",
            _ => {}
        }
    }

    // Tier 3: Heading Signature Fuzzy Matching
    let normalized_headings: std::collections::HashSet<String> = doc
        .headings
        .iter()
        .map(|h| h.trim().to_lowercase())
        .collect();

    if !normalized_headings.is_empty() {
        let mut best_match: Option<(&str, usize)> = None;
        for (profile_name, def) in &catalog.definitions {
            if profile_name == "note" || profile_name == "index" {
                continue;
            }
            let mut matched_sections = 0;
            for group in &def.sections {
                if group
                    .iter()
                    .any(|sec| normalized_headings.contains(&sec.trim().to_lowercase()))
                {
                    matched_sections += 1;
                }
            }
            if matched_sections > 0 {
                if let Some((_, best_score)) = best_match {
                    if matched_sections > best_score {
                        best_match = Some((profile_name.as_str(), matched_sections));
                    }
                } else {
                    best_match = Some((profile_name.as_str(), matched_sections));
                }
            }
        }
        if let Some((best_profile, _)) = best_match {
            return best_profile;
        }
    }

    // Tier 4: Default Fallback
    "note"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_pack_and_alias_edge_cases() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path();
        let pack_dir = root.join("pack_without_profiles");
        fs::create_dir_all(&pack_dir).unwrap();

        let index_doc = parse_document_text(
            root,
            root.join("index.md"),
            "---\ncustom-profiles:\n  - ods-profiles/sub/subprof.md\npacks:\n  - pack_without_profiles\n---\n",
            true,
        );
        let roots = profile_catalog_roots(root, Some(&index_doc));
        let pack_dir_canon = pack_dir.canonicalize().unwrap_or_else(|_| pack_dir.clone());
        assert!(
            roots.contains(&pack_dir)
                || roots.contains(&pack_dir_canon)
                || roots.contains(&root.join("ods-profiles/sub/subprof.md"))
        );

        let prof_dir = root.join("ods-profiles").join("sub");
        fs::create_dir_all(&prof_dir).unwrap();
        fs::write(prof_dir.join(".hidden"), "ignored").unwrap();
        fs::write(
            prof_dir.join("subprof.md"),
            "---\naliases:\n  NewCanonical:\n    - AliasOne\n---\n# Subprof\n",
        )
        .unwrap();

        let cat = load_profile_catalog(root, &roots).unwrap();
        assert!(cat.definitions.contains_key("subprof"));

        // Test render_profile_template
        let templ = render_profile_template(&cat, "subprof", "My Sub Title").unwrap();
        assert!(templ.contains("# My Sub Title"));

        let templ_err = render_profile_template(&cat, "nonexistent", "Title").unwrap_err();
        assert!(templ_err.contains("unknown profile"));

        // Test resolve_document_profile Tier 2 folder conventions
        let mut doc = parse_document_text(root, root.join("adrs/001.md"), "# Decision\n", true);
        assert_eq!(resolve_document_profile(&doc, &cat), "decision");

        doc.path = root.join("features/001.md");
        assert_eq!(resolve_document_profile(&doc, &cat), "feature");

        doc.path = root.join("rfcs/001.md");
        assert_eq!(resolve_document_profile(&doc, &cat), "rfc");

        // Test resolve_document_profile Tier 3 heading signature matching
        let mut doc_headings = parse_document_text(
            root,
            root.join("misc/doc.md"),
            "---\nstatus: draft\n---\n\n## NewCanonical\n",
            true,
        );
        doc_headings.headings = vec!["NewCanonical".into()];
        assert_eq!(resolve_document_profile(&doc_headings, &cat), "subprof");
    }
}
