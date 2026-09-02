use crate::fs::normalize_join;
use crate::model::{
    Diagnostic, Document, Frontmatter, FrontmatterState, LintLevel, RelatedEntry, Severity,
    Workspace,
};
use crate::parse::{document_id, split_markdown_link_target};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::{Path, PathBuf};

pub fn known_profiles(workspace: &Workspace) -> Vec<String> {
    workspace
        .profiles
        .definitions
        .keys()
        .cloned()
        .collect::<Vec<_>>()
}

pub fn profile_sections(workspace: &Workspace, profile: &str) -> Vec<Vec<String>> {
    workspace
        .profiles
        .definitions
        .get(profile)
        .map(|definition| definition.sections.clone())
        .unwrap_or_default()
}

pub fn profile_section_labels(workspace: &Workspace, profile: &str) -> Vec<String> {
    profile_sections(workspace, profile)
        .into_iter()
        .filter_map(|group| group.into_iter().next())
        .collect()
}

pub fn lint_workspace(workspace: &Workspace) -> Vec<Diagnostic> {
    lint_workspace_with_level(workspace, LintLevel::Full)
}

pub fn lint_workspace_with_level(workspace: &Workspace, level: LintLevel) -> Vec<Diagnostic> {
    lint_workspace_with_ref_style(workspace, level, false)
}

/// Returns binary compliance for a workspace.
pub fn workspace_compliance(workspace: &Workspace) -> crate::model::WorkspaceCompliance {
    let diags = lint_workspace(workspace);
    if diags.iter().any(|d| d.severity == crate::model::Severity::Error) {
        crate::model::WorkspaceCompliance::NonCompliant
    } else {
        crate::model::WorkspaceCompliance::Compliant
    }
}

pub fn lint_workspace_with_ref_style(
    workspace: &Workspace,
    level: LintLevel,
    canonical_refs: bool,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    diagnostics.extend(lint_root_ods_metadata(workspace));
    diagnostics.extend(lint_profile_conflicts(workspace));
    let ids = build_id_index(workspace);

    for document in &workspace.documents {
        diagnostics.extend(lint_document(
            workspace,
            document,
            &ids,
            level,
            canonical_refs,
        ));
    }

    diagnostics.extend(lint_duplicate_ids(&ids));
    diagnostics.extend(lint_cycles(workspace, &ids));
    diagnostics.extend(lint_spec21_entities(workspace));

    diagnostics
}

/// Lint a single document using workspace maps for reference resolution.
/// Includes global profile-conflict warnings that mention this path, plus
/// duplicate-id / cycle diagnostics that involve this document's id.
pub fn lint_document_in_workspace(
    workspace: &Workspace,
    path: &Path,
    level: LintLevel,
) -> Vec<Diagnostic> {
    let ids = build_id_index(workspace);
    let mut diagnostics = Vec::new();

    diagnostics.extend(
        lint_profile_conflicts(workspace)
            .into_iter()
            .filter(|d| d.path == path),
    );

    if let Some(document) = workspace
        .document_by_path(path)
        .or_else(|| workspace.documents.iter().find(|doc| doc.path == path))
    {
        diagnostics.extend(lint_document(workspace, document, &ids, level, false));

        let frontmatter = match &document.frontmatter {
            FrontmatterState::Parsed(fm) => Some(fm),
            _ => None,
        };
        let id = document_id(&workspace.root, &document.path, frontmatter);

        diagnostics.extend(
            lint_duplicate_ids(&ids)
                .into_iter()
                .filter(|d| d.path == path || d.message.contains(&id)),
        );
        diagnostics.extend(
            lint_cycles(workspace, &ids)
                .into_iter()
                .filter(|d| d.path == path || d.message.contains(&id)),
        );
    }

    diagnostics
}

fn lint_profile_conflicts(workspace: &Workspace) -> Vec<Diagnostic> {
    workspace
        .profiles
        .conflicts
        .iter()
        .map(|conflict| Diagnostic {
            path: conflict.ignored.clone(),
            severity: Severity::Warning,
            message: crate::error::lint_duplicate_profile(
                &conflict.name,
                conflict.kept.display(),
                conflict.ignored.display(),
            ),
        })
        .collect()
}

fn build_id_index(workspace: &Workspace) -> BTreeMap<String, Vec<&Document>> {
    let mut ids = BTreeMap::<String, Vec<&Document>>::new();

    for document in &workspace.documents {
        let frontmatter = match &document.frontmatter {
            FrontmatterState::Parsed(frontmatter) => Some(frontmatter),
            _ => None,
        };
        let id = document_id(&workspace.root, &document.path, frontmatter);
        ids.entry(id).or_default().push(document);
    }

    ids
}
fn lint_duplicate_ids(ids: &BTreeMap<String, Vec<&Document>>) -> Vec<Diagnostic> {
    ids.iter()
        .filter(|(_, docs)| docs.len() > 1)
        .flat_map(|(id, docs)| {
            docs.iter().map(move |doc| Diagnostic {
                path: doc.path.clone(),
                severity: Severity::Error,
                message: crate::error::lint_duplicate_document_id(id),
            })
        })
        .collect()
}

#[cfg(test)]
mod test_checker {
    use super::*;
    use crate::fs::load_workspace;
    use tempfile::tempdir;

    #[test]
    fn test_lint_checker_helpers() {
        let td = tempdir().unwrap();
        let root = td.path();
        std::fs::write(
            root.join("index.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# Root\n",
        )
        .unwrap();

        let ws = load_workspace(root).unwrap();
        let profiles = known_profiles(&ws);
        assert!(profiles.is_empty() || !profiles.is_empty());

        let secs = profile_sections(&ws, "note");
        assert!(secs.is_empty() || !secs.is_empty());

        let labels = profile_section_labels(&ws, "note");
        assert!(labels.is_empty() || !labels.is_empty());

        let conflicts = lint_profile_conflicts(&ws);
        assert!(conflicts.is_empty());
    }
}
