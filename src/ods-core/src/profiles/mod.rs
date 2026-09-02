use crate::model::{Document, FrontmatterState, ProfileCatalog, ProfileDefinition};
use crate::parse::{extract_profile_section_groups, parse_document_text};
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Validate every explicitly configured custom-profile path before loading it.
///
/// `custom_profiles` is the source of truth for profile definitions. Missing or
/// non-Markdown entries must fail workspace loading instead of being silently
/// skipped.
pub fn validate_custom_profile_paths(
    root: &Path,
    config: &crate::config::WorkspaceConfig,
) -> io::Result<()> {
    let config_path = root.join("ods.toml");
    for declared in &config.custom_profiles {
        let path = root.join(declared);
        let metadata = fs::metadata(&path).map_err(|err| {
            io::Error::new(
                err.kind(),
                format!(
                    "custom profile path not found: {} (declared by custom_profiles in {})",
                    path.display(),
                    config_path.display()
                ),
            )
        })?;

        if !metadata.is_file() && !metadata.is_dir() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "custom profile path is not a file or directory: {} (declared by custom_profiles in {})",
                    path.display(),
                    config_path.display()
                ),
            ));
        }

        if metadata.is_file() && path.extension().and_then(|ext| ext.to_str()) != Some("md") {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "custom profile definition must be a Markdown file: {} (declared by custom_profiles in {})",
                    path.display(),
                    config_path.display()
                ),
            ));
        }
    }
    Ok(())
}

/// Reject profile-definition metadata in documents that are not registered
/// profile-definition files.
pub fn validate_custom_profile_placements(
    root: &Path,
    documents: &[Document],
    profile_roots: &[PathBuf],
    config: &crate::config::WorkspaceConfig,
) -> io::Result<()> {
    let config_path = root.join("ods.toml");
    let declared_paths = if config.custom_profiles.is_empty() {
        "(none)".to_string()
    } else {
        config.custom_profiles.join(", ")
    };

    for document in documents {
        let FrontmatterState::Parsed(frontmatter) = &document.frontmatter else {
            continue;
        };
        if frontmatter.custom_profile.is_some()
            && !is_registered_profile_path(&document.path, profile_roots)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "invalid ods.custom_profile in {}: the file is not registered in custom_profiles from {} (registered paths: {}). Define the profile at a registered path and use ods.profile in this document",
                    document.path.display(),
                    config_path.display(),
                    declared_paths
                ),
            ));
        }
    }
    Ok(())
}

fn is_registered_profile_path(path: &Path, profile_roots: &[PathBuf]) -> bool {
    let path = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    profile_roots.iter().any(|root| {
        let root = root.canonicalize().unwrap_or_else(|_| root.to_path_buf());
        path == root || path.strip_prefix(&root).is_ok()
    })
}

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

    let mut seen = HashSet::new();
    roots.retain(|root| seen.insert(root.clone()));
    roots
}

pub fn load_profile_catalog(root: &Path, roots: &[PathBuf]) -> io::Result<ProfileCatalog> {
    let mut catalog = standard_profile_catalog();

    for profile_root in roots {
        if !profile_root.exists() {
            return Err(io::Error::new(
                io::ErrorKind::NotFound,
                format!("custom profile path not found: {}", profile_root.display()),
            ));
        }

        let mut paths = Vec::new();
        if profile_root.is_file() {
            if profile_root.extension().and_then(|ext| ext.to_str()) != Some("md") {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "custom profile definition must be a Markdown file: {}",
                        profile_root.display()
                    ),
                ));
            }
            paths.push(profile_root.clone());
        } else {
            collect_markdown_paths(profile_root, &mut paths)?;
            paths.sort();
        }

        for path in paths {
            let text = fs::read_to_string(&path)?;
            let document = parse_document_text(root, path.clone(), &text, true);
            if let FrontmatterState::Invalid(message) = &document.frontmatter {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!(
                        "invalid custom profile definition at {}: {}",
                        path.display(),
                        message
                    ),
                ));
            }
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
    let sections = extract_profile_section_groups(&document.body);

    if let FrontmatterState::Parsed(frontmatter) = &document.frontmatter {
        if let Some(definition) = &frontmatter.custom_profile {
            if let Some(explicit_name) = &definition.name {
                name = explicit_name.clone();
            }
            required_keys.clone_from(&definition.required_keys);
            optional_keys.clone_from(&definition.optional_keys);
            forbidden_keys.clone_from(&definition.forbidden_keys);
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
                "Goal",
                "Task",
                "Scope",
                "Non-Scope",
                "Context",
                "Inputs",
                "Constraints",
                "Priority",
                "Steps",
                "Output",
                "Success Criteria",
                "Failure Modes",
                "Dependencies",
                "Assumptions",
                "Examples",
            ],
        ),
        profile(
            "skill",
            vec![
                "Purpose",
                "Capability",
                "Activation",
                "Scope",
                "Non-Scope",
                "Inputs",
                "Outputs",
                "Workflow",
                "Rules",
                "Priority",
                "Validation",
                "Eval",
                "Resources",
                "Tools",
                "Lifecycle",
                "Traceability",
            ],
        ),
        profile(
            "feature",
            vec![
                "Goal",
                "Scope",
                "Requirements",
                "Acceptance Criteria",
                "Risks",
            ],
        ),
        profile(
            "guide",
            vec!["Overview", "Prerequisites", "Steps", "Troubleshooting"],
        ),
        profile(
            "api",
            vec!["Overview", "Request", "Response", "Errors", "Examples"],
        ),
        profile(
            "architecture",
            vec!["Overview", "Components", "Data Flow", "Trade-offs"],
        ),
        profile(
            "decision",
            vec!["Context", "Decision", "Alternatives", "Consequences"],
        ),
        profile(
            "sop",
            vec![
                "Purpose",
                "Prerequisites",
                "Steps",
                "Validation",
                "Rollback",
            ],
        ),
        profile("policy", vec!["Purpose", "Scope", "Rules", "Exceptions"]),
        profile(
            "meeting",
            vec!["Attendees", "Agenda", "Decisions", "Action Items"],
        ),
        profile("faq", vec![]),
        profile(
            "checklist",
            vec!["Overview", "Items", "Verification", "Notes"],
        ),
        profile("index", vec![]),
    ]
}

fn profile(name: &str, sections: Vec<&str>) -> ProfileDefinition {
    ProfileDefinition {
        name: name.to_string(),
        sections: sections.into_iter().map(|s| vec![s.to_string()]).collect(),
        required_keys: vec![],
        optional_keys: vec![],
        forbidden_keys: vec![],
        source: PathBuf::from(format!("<builtin:{name}>")),
    }
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
    fn profiles_pack_and_edge_cases() {
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
            "---\nods:\n  custom_profile:\n    name: subprof\n---\n# Subprof\n\n## NewCanonical\n",
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
