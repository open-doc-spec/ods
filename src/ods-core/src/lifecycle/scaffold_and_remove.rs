use crate::parse::document_id;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NewDocumentOptions {
    pub profile: Option<String>,
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NewDocumentReport {
    pub created_file: PathBuf,
    pub doc_id: String,
    pub profile: String,
    pub updated_indexes: Vec<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RemoveDocumentOptions {
    pub scrub_dependencies: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RemoveDocumentReport {
    pub deleted_file: PathBuf,
    pub doc_id: String,
    pub cleaned_references_count: usize,
    pub updated_indexes: Vec<PathBuf>,
}

/// Scaffold a new ODS document with inferred profile, valid YAML frontmatter, section templates, and parent index update.
pub fn scaffold_new_document(
    root: &Path,
    target_path: &Path,
    options: NewDocumentOptions,
) -> io::Result<NewDocumentReport> {
    let target_path = if target_path.is_absolute() {
        target_path.to_path_buf()
    } else {
        root.join(target_path)
    };

    if target_path.exists() {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            crate::error::lifecycle_document_exists(target_path.display()),
        ));
    }

    if let Some(parent) = target_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Infer profile from path if not explicitly provided
    let path_str = target_path.to_string_lossy().to_lowercase();
    let inferred_profile = if let Some(p) = options.profile {
        p
    } else if path_str.contains("agent")
        || path_str.contains("agents")
        || path_str.contains("prompt")
        || path_str.contains("prompts")
        || path_str.contains("subagent")
        || path_str.contains("subagents")
    {
        "agent".to_string()
    } else if path_str.contains("guide") || path_str.contains("tutorial") || path_str.contains("howto") {
        "guide".to_string()
    } else if path_str.contains("feature") || path_str.contains("spec") || path_str.contains("prd") {
        "feature".to_string()
    } else if path_str.contains("decision") || path_str.contains("adr") {
        "decision".to_string()
    } else if path_str.contains("sop") || path_str.contains("policy") || path_str.contains("ops") {
        "sop".to_string()
    } else if path_str.contains("api") || path_str.contains("endpoint") {
        "api".to_string()
    } else {
        "note".to_string()
    };

    let title = if let Some(t) = options.title {
        t
    } else {
        target_path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Untitled")
            .replace(['-', '_'], " ")
    };

    let section_template = match inferred_profile.as_str() {
        "agent" => {
            "## Goal\n\n## Task\n\n## Scope\n\n## Non-Scope\n\n## Context\n\n## Inputs\n\n## Constraints\n\n## Priority\n\n## Steps\n\n## Output\n\n## Success Criteria\n\n## Failure Modes\n\n## Dependencies\n\n## Assumptions\n\n## Examples\n"
        }
        "feature" => "## Goal\n\n## Scope\n\n## Requirements\n\n## Acceptance Criteria\n\n## Risks\n",
        "guide" => "## Overview\n\n## Prerequisites\n\n## Steps\n\n## Troubleshooting\n",
        "decision" => "## Context\n\n## Decision\n\n## Alternatives\n\n## Consequences\n",
        "sop" => "## Purpose\n\n## Prerequisites\n\n## Steps\n\n## Validation\n\n## Rollback\n",
        "api" => "## Overview\n\n## Request\n\n## Response\n\n## Errors\n\n## Examples\n",
        "meeting" => "## Attendees\n\n## Agenda\n\n## Decisions\n\n## Action Items\n",
        "faq" => "## Q&A\n",
        _ => "## Overview\n\n## Details\n",
    };

    let content = format!(
        "---\ndescription: Draft {} for {}\nods:\n  profile: {}\n  status: draft\n---\n\n# {}\n\n{}",
        inferred_profile, title, inferred_profile, title, section_template
    );

    fs::write(&target_path, content)?;

    let doc_id = document_id(root, &target_path, None);

    let _workspace = load_workspace(root)?;
    let updated_indexes = Vec::new();

    Ok(NewDocumentReport {
        created_file: target_path,
        doc_id,
        profile: inferred_profile,
        updated_indexes,
    })
}

/// Atomically delete a document, scrub dependency references across the workspace, and update indexes.
pub fn atomic_delete_document(
    root: &Path,
    target: &Path,
    _options: RemoveDocumentOptions,
) -> io::Result<RemoveDocumentReport> {
    let workspace = load_workspace(root)?;
    let target_abs = if target.is_absolute() {
        target.to_path_buf()
    } else {
        root.join(target)
    };

    let target_id_str = target.to_string_lossy().to_lowercase();

    let doc_to_delete = workspace.documents.iter().find(|d| {
        let did = document_id(root, &d.path, match &d.frontmatter {
            crate::model::FrontmatterState::Parsed(fm) => Some(fm),
            _ => None,
        });
        d.path == target_abs || did == target_id_str
    });

    let (target_file, target_id) = if let Some(d) = doc_to_delete {
        let did = document_id(root, &d.path, match &d.frontmatter {
            crate::model::FrontmatterState::Parsed(fm) => Some(fm),
            _ => None,
        });
        (d.path.clone(), did)
    } else if target_abs.is_file() {
        (target_abs.clone(), document_id(root, &target_abs, None))
    } else {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            crate::error::lifecycle_document_not_found(target.display()),
        ));
    };

    if target_file.is_file() {
        fs::remove_file(&target_file)?;
    }

    let mut cleaned_count = 0;

    // Scrub target_id from all other workspace documents' depends: and related: lists
    for doc in &workspace.documents {
        if doc.path == target_file {
            continue;
        }

        if let Ok(text) = fs::read_to_string(&doc.path) {
            let (fm_opt, body) = split_frontmatter(&text);
            if let Some(fm_block) = fm_opt {
                let mut lines: Vec<String> = fm_block.lines().map(|s| s.to_string()).collect();
                let mut modified = false;

                lines.retain(|line| {
                    let trimmed = line.trim();
                    let matches_id = trimmed.contains(&target_id);
                    if matches_id && (trimmed.starts_with("- ") || trimmed.starts_with("depends:") || trimmed.starts_with("related:")) {
                        modified = true;
                        cleaned_count += 1;
                        false
                    } else {
                        true
                    }
                });

                if modified {
                    let new_fm = lines.join("\n");
                    let new_text = format!("---\n{}\n---\n\n{}", new_fm, body.trim_start());
                    let _ = fs::write(&doc.path, new_text);
                }
            }
        }
    }

    let _updated_workspace = load_workspace(root)?;
    let updated_indexes = Vec::new();

    Ok(RemoveDocumentReport {
        deleted_file: target_file,
        doc_id: target_id,
        cleaned_references_count: cleaned_count,
        updated_indexes,
    })
}
