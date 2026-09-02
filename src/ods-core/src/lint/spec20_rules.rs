// ODS 2.0/2.1 spec lint rules (TITLE-001, ONT/ENT/ENUM/ASSET).

pub(super) fn lint_spec20_document(
    workspace: &Workspace,
    document: &Document,
    frontmatter: &Frontmatter,
) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let ontology = workspace.config.ontology_enabled();

    if frontmatter.ods_wrapper {
        diagnostics.push(Diagnostic {
            path: document.path.clone(),
            severity: Severity::Error,
            message: crate::error::lint_ods_wrapper_rejected(),
        });
    }

    if frontmatter.context.is_some() {
        diagnostics.push(Diagnostic {
            path: document.path.clone(),
            severity: Severity::Error,
            message: crate::error::lint_legacy_key_rejected("context"),
        });
    }

    if frontmatter.custom_profile.is_some() {
        diagnostics.push(Diagnostic {
            path: document.path.clone(),
            severity: Severity::Error,
            message: crate::error::lint_legacy_key_rejected("custom_profile"),
        });
    }

    if frontmatter.code_object_form {
        diagnostics.push(Diagnostic {
            path: document.path.clone(),
            severity: Severity::Error,
            message: crate::error::lint_code_object_form_rejected(),
        });
    }

    for key in ["memory", "invariants"] {
        if frontmatter.present_keys.contains(key) {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Error,
                message: crate::error::lint_legacy_key_rejected(key),
            });
        }
    }

    // TITLE-001 / TITLE-002 when no OKF signal (`title` only — `name` is a skill slug, not H1)
    let okf_signal = frontmatter.concept_type.is_some() || frontmatter.okf_version.is_some();
    if !okf_signal {
        if let Some(title) = frontmatter.title.as_deref() {
            let h1 = document.headings.first().map(String::as_str);
            match h1 {
                None => diagnostics.push(Diagnostic {
                    path: document.path.clone(),
                    severity: Severity::Error,
                    message: crate::error::lint_title_missing_h1(),
                }),
                Some(h1_text) if normalize_title(title) != normalize_title(h1_text) => {
                    diagnostics.push(Diagnostic {
                        path: document.path.clone(),
                        severity: Severity::Error,
                        message: crate::error::lint_title_h1_mismatch(title, h1_text),
                    });
                }
                _ => {}
            }
        }
    }

    // ASSET-004: load paths must exist
    for load_path in &frontmatter.load {
        let resolved = crate::fs::normalize_join(&document.directory, Path::new(load_path));
        if !resolved.exists() {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Error,
                message: crate::error::lint_missing_load_path(load_path),
            });
        }
    }

    // ONT-001: schema path (2.1+)
    if let Some(schema_path) = &frontmatter.schema_path {
        if ontology {
            let resolved = crate::fs::normalize_join(&document.directory, Path::new(schema_path));
            if !resolved.exists() {
                diagnostics.push(Diagnostic {
                    path: document.path.clone(),
                    severity: Severity::Error,
                    message: crate::error::lint_ontology_schema_missing(schema_path),
                });
            }
        } else {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Error,
                message: crate::error::lint_ontology_key_on_20("schema"),
            });
        }
    }

    for key in ["entity", "domain"] {
        let present = match key {
            "entity" => frontmatter.entity.is_some(),
            "domain" => frontmatter.domain.is_some(),
            _ => false,
        };
        if present && !ontology {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Error,
                message: crate::error::lint_ontology_key_on_20(key),
            });
        }
    }

    // ENUM-006: unknown related predicates
    const PARETO: &[&str] = &["is_a", "part_of", "owns", "governed_by", "maps_to"];
    for entry in &frontmatter.related_entries {
        match entry {
            RelatedEntry::Predicate { predicate, .. } if !PARETO.contains(&predicate.as_str()) => {
                diagnostics.push(Diagnostic {
                    path: document.path.clone(),
                    severity: Severity::Error,
                    message: crate::error::lint_unknown_related_predicate(predicate),
                });
            }
            RelatedEntry::Custom { .. } => {}
            RelatedEntry::Predicate { .. } | RelatedEntry::Path(_) => {}
        }
        if !ontology
            && !matches!(entry, RelatedEntry::Path(_))
        {
            diagnostics.push(Diagnostic {
                path: document.path.clone(),
                severity: Severity::Error,
                message: crate::error::lint_ontology_key_on_20("related (typed)"),
            });
        }
    }

    diagnostics
}

pub(super) fn lint_spec21_entities(workspace: &Workspace) -> Vec<Diagnostic> {
    if !workspace.config.ontology_enabled() {
        return Vec::new();
    }

    let mut by_entity: HashMap<String, Vec<PathBuf>> = HashMap::new();
    for doc in &workspace.documents {
        let FrontmatterState::Parsed(fm) = &doc.frontmatter else {
            continue;
        };
        if let Some(entity) = &fm.entity {
            by_entity
                .entry(entity.clone())
                .or_default()
                .push(doc.path.clone());
        }
    }

    let mut diagnostics = Vec::new();
    for (entity, paths) in &by_entity {
        if paths.len() > 1 {
            diagnostics.push(Diagnostic {
                path: paths[0].clone(),
                severity: Severity::Error,
                message: crate::error::lint_duplicate_entity(entity),
            });
        }
    }
    diagnostics
}

fn normalize_title(s: &str) -> String {
    s.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
}
