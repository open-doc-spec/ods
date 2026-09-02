

pub(super) fn lint_packs(
    _workspace: &Workspace,
    _document: &Document,
    _frontmatter: &crate::model::Frontmatter,
) -> Vec<Diagnostic> {
    // Pack paths are validated on workspace config in lint_root_config.
    Vec::new()
}

pub(super) fn lint_references(
    workspace: &Workspace,
    document: &Document,
    ids: &BTreeMap<String, Vec<&Document>>,
    frontmatter: &crate::model::Frontmatter,
    canonical_refs: bool,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();

    for reference in frontmatter
        .depends
        .iter()
        .chain(frontmatter.related_targets().iter())
    {
        if crate::refs::document_ref_to_id(workspace, document, reference).is_none()
            && !ids.contains_key(reference)
        {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Error,
                message: crate::error::lint_dangling_reference(reference),
            });
        } else if canonical_refs
            && !crate::refs::is_markdown_ref(reference)
            && let Some(canonical) =
                crate::refs::canonical_document_ref_for_reference(workspace, document, reference)
            && canonical != *reference
        {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Warning,
                message: crate::error::lint_non_canonical_ref(reference, &canonical),
            });
        }
    }

    for load in &frontmatter.load {
        let path = normalize_join(&document.directory, Path::new(load));
        if !path.exists() {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Error,
                message: crate::error::lint_missing_load_path(load),
            });
        }
    }

    if let Some(context) = &frontmatter.context {
        for load in &context.load {
            if crate::refs::document_ref_to_path(workspace, document, load).is_some() {
                if canonical_refs
                    && !crate::refs::is_markdown_ref(load)
                    && let Some(canonical) =
                        crate::refs::canonical_document_ref_for_reference(workspace, document, load)
                    && canonical != *load
                {
                    diagnostics.push(Diagnostic {
                        path: document.path.clone(),
                        severity: Severity::Warning,
                        message: crate::error::lint_non_canonical_context_ref(load, &canonical),
                    });
                }
            } else if is_resource_like(load) {
                let path = normalize_join(&document.directory, Path::new(load));
                if !path.exists() {
                    diagnostics.push(Diagnostic {
                        path: document.path.clone(),
                        severity: Severity::Error,
                        message: crate::error::lint_missing_context_resource(load),
                    });
                }
            } else if !ids.contains_key(&load.to_lowercase()) {
                diagnostics.push(Diagnostic {
                    path: document.path.clone(),
                    severity: Severity::Error,
                    message: crate::error::lint_dangling_context_reference(load),
                });
            }
        }

        for ignore in &context.ignore {
            let ignored = normalize_join(&document.directory, Path::new(ignore));
            if !ignored.exists() && !ids.contains_key(&ignore.to_lowercase()) {
                diagnostics.push(Diagnostic {
                    path: document.path.clone(),
                    severity: Severity::Warning,
                    message: crate::error::lint_context_ignore_not_found(ignore),
                });
            }
        }
    }

    diagnostics
}

pub(super) fn lint_ods_scope(
    _workspace: &Workspace,
    document: &Document,
    frontmatter: &crate::model::Frontmatter,
) -> Vec<Diagnostic> {
    if frontmatter.ods.is_none() {
        return Vec::new();
    }

    vec![Diagnostic {
        path: document.path.clone(),
        severity: Severity::Error,
        message: crate::error::lint_root_ods_scope_only(),
    }]
}


