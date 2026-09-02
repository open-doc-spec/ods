use crate::model::{FrontmatterState, Workspace};
use std::collections::{BTreeSet, VecDeque};
use std::path::{Path, PathBuf};

/// Options for bounded context resolution.
#[derive(Clone, Debug, Default)]
pub struct ContextOptions {
    /// Include documents with `share: private`.
    pub include_private: bool,
    /// Walk `code:` edges (default false for AI context — source dumps burn tokens).
    pub include_code: bool,
    /// Walk soft `related` edges (default false — not structural prerequisites).
    pub include_related: bool,
    /// Soft token budget (bytes/4 heuristic). `None` = unlimited.
    pub max_tokens: Option<usize>,
}

/// Result of context resolution with diagnostics for agents/humans.
#[derive(Clone, Debug, Default)]
pub struct ContextResult {
    pub paths: Vec<PathBuf>,
    /// Why each path was included (same length/order as `paths`).
    pub reasons: Vec<String>,
    /// Document paths skipped because `share: private` (or org when filtered).
    pub skipped_private: Vec<PathBuf>,
    /// Estimated tokens of included paths (file size / 4).
    pub token_estimate: usize,
    /// True when `max_tokens` stopped expansion early.
    pub truncated: bool,
}

/// Resolve a bounded reading list (legacy API).
///
/// Code edges are **included** here for backward compatibility with existing
/// tests and graph tooling. Prefer [`resolve_context_with_options`] for AI use
/// (`include_code: false` by default).
pub fn resolve_context(workspace: &Workspace, query: &str, include_private: bool) -> Vec<PathBuf> {
    resolve_context_with_options(
        workspace,
        query,
        &ContextOptions {
            include_private,
            include_code: true,
            include_related: false,
            max_tokens: None,
        },
    )
    .paths
}

/// Resolve context with token budget, private-skip tracking, and optional code/related edges.
pub fn resolve_context_with_options(
    workspace: &Workspace,
    query: &str,
    options: &ContextOptions,
) -> ContextResult {
    let Some(start) = resolve_context_start(workspace, query) else {
        return ContextResult::default();
    };

    // queue: (path, depth, reason)
    let mut queue = VecDeque::from([(start.clone(), 0usize, String::from("start"))]);
    let mut visited = BTreeSet::<PathBuf>::new();
    let mut output = Vec::<PathBuf>::new();
    let mut reasons = Vec::<String>::new();
    let mut skipped_private = Vec::<PathBuf>::new();
    let mut token_estimate = 0usize;
    let mut truncated = false;
    let max_depth = context_depth(workspace, &start).unwrap_or(2);
    let ignore_rules = context_ignore_rules(workspace, &start);

    while let Some((path, depth, reason)) = queue.pop_front() {
        if is_ignored(&path, &workspace.root, &ignore_rules) {
            continue;
        }
        let is_private = workspace
            .document_by_path(&path)
            .and_then(|doc| frontmatter(doc))
            .is_some_and(|fm| fm.share.as_deref() == Some("private"));
        if !options.include_private && is_private {
            if visited.insert(path.clone()) {
                skipped_private.push(path);
            }
            continue;
        }
        if !visited.insert(path.clone()) {
            continue;
        }

        let file_tokens = estimate_path_tokens(&path);
        if let Some(budget) = options.max_tokens {
            if !output.is_empty() && token_estimate.saturating_add(file_tokens) > budget {
                truncated = true;
                continue;
            }
        }
        token_estimate = token_estimate.saturating_add(file_tokens);
        output.push(path.clone());
        reasons.push(reason);

        if depth >= max_depth {
            continue;
        }

        let Some(document) = workspace.document_by_path(&path) else {
            continue;
        };
        let Some(frontmatter) = frontmatter(document) else {
            continue;
        };

        let from_label = path.file_name().and_then(|n| n.to_str()).unwrap_or("doc");

        let resolve_ref = |reference: &String| -> Option<PathBuf> {
            if let Some(document_path) =
                crate::refs::document_ref_to_path(workspace, document, reference)
            {
                Some(document_path)
            } else if crate::refs::is_file_like_ref(reference) {
                let resource_path =
                    crate::fs::normalize_join(&document.directory, Path::new(reference));
                if resource_path.exists() {
                    Some(resource_path)
                } else {
                    None
                }
            } else {
                None
            }
        };

        let mut next: Vec<(PathBuf, String)> = Vec::new();

        for reference in &frontmatter.depends {
            if let Some(p) = resolve_ref(reference) {
                if !is_ignored(&p, &workspace.root, &ignore_rules) {
                    next.push((p, format!("depends hop {} from {from_label}", depth + 1)));
                }
            }
        }

        for reference in &frontmatter.load {
            if let Some(p) = resolve_ref(reference) {
                if !is_ignored(&p, &workspace.root, &ignore_rules) {
                    next.push((p, format!("load from {from_label}")));
                }
            }
        }

        if let Some(ctx) = &frontmatter.context {
            for reference in &ctx.load {
                if let Some(p) = resolve_ref(reference) {
                    if !is_ignored(&p, &workspace.root, &ignore_rules) {
                        next.push((p, format!("context.load from {from_label} (legacy)")));
                    }
                }
            }
        }

        if options.include_related {
            for reference in &frontmatter.related {
                if let Some(p) = resolve_ref(reference) {
                    if !is_ignored(&p, &workspace.root, &ignore_rules) {
                        next.push((p, format!("related from {from_label}")));
                    }
                }
            }
        }

        if options.include_code {
            for code in &frontmatter.code {
                let code_path = crate::fs::normalize_join(&document.directory, &code.path);
                if code_path.exists() && !is_ignored(&code_path, &workspace.root, &ignore_rules) {
                    next.push((code_path, format!("code from {from_label}")));
                }
            }
        }

        next.sort_by(|a, b| a.0.cmp(&b.0));
        queue.extend(
            next.into_iter()
                .map(|(path, reason)| (path, depth + 1, reason)),
        );
    }

    ContextResult {
        paths: output,
        reasons,
        skipped_private,
        token_estimate,
        truncated,
    }
}

/// Rough token estimate: file bytes / 4 (same heuristic as bench).
pub fn estimate_path_tokens(path: &Path) -> usize {
    std::fs::metadata(path)
        .map(|m| (m.len() as usize) / 4)
        .unwrap_or(0)
}

/// Concatenate file bodies under a token budget for agent prompt packs.
pub fn render_context_pack(paths: &[PathBuf], max_tokens: Option<usize>) -> String {
    let mut out = String::new();
    let mut used = 0usize;
    for path in paths {
        let Ok(text) = std::fs::read_to_string(path) else {
            continue;
        };
        let header = format!("\n---\n# file: {}\n\n", path.display());
        let chunk_tokens = (header.len() + text.len()) / 4;
        if let Some(budget) = max_tokens {
            if used > 0 && used.saturating_add(chunk_tokens) > budget {
                out.push_str(&format!(
                    "\n---\n# truncated: remaining files omitted (budget ~{budget} tokens)\n"
                ));
                break;
            }
        }
        used = used.saturating_add(chunk_tokens);
        out.push_str(&header);
        out.push_str(&text);
        if !text.ends_with('\n') {
            out.push('\n');
        }
    }
    out
}

fn context_ignore_rules(workspace: &Workspace, path: &Path) -> Vec<String> {
    let mut rules = workspace.config.context.ignore.clone();
    let Some(document) = workspace.document_by_path(path) else {
        return rules;
    };
    let Some(frontmatter) = frontmatter(document) else {
        return rules;
    };
    if let Some(ctx) = &frontmatter.context {
        rules.extend(ctx.ignore.clone());
    }
    rules
}

fn is_ignored(path: &Path, root: &Path, rules: &[String]) -> bool {
    if rules.is_empty() {
        return false;
    }

    let relative = path
        .strip_prefix(root)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/");
    let relative = relative.trim_start_matches("./");

    rules.iter().any(|rule| {
        let rule = rule.trim().trim_end_matches('/');
        if rule.is_empty() {
            return false;
        }
        relative == rule
            || relative.starts_with(&format!("{rule}/"))
            || relative.contains(&format!("/{rule}/"))
            || relative.ends_with(&format!("/{rule}"))
    })
}

fn frontmatter(document: &crate::model::Document) -> Option<&crate::model::Frontmatter> {
    match &document.frontmatter {
        FrontmatterState::Parsed(frontmatter) => Some(frontmatter),
        _ => None,
    }
}

fn context_depth(workspace: &Workspace, path: &Path) -> Option<usize> {
    let document = workspace.document_by_path(path)?;
    let frontmatter = frontmatter(document)?;
    // Per-document max_depth (legacy) overrides workspace default.
    if let Some(depth) = frontmatter.context.as_ref().and_then(|c| c.max_depth) {
        return Some(depth);
    }
    Some(workspace.config.context.default_max_depth)
}

/// Resolve a context query to a starting document path.
///
/// Accepts document ids (`specs/ods/core`), paths with or without `.md`, bare stems
/// (`core` when unique), and absolute paths under the workspace root.
pub fn resolve_context_start(workspace: &Workspace, query: &str) -> Option<PathBuf> {
    let raw = query.trim();
    if raw.is_empty() {
        return None;
    }
    let query_lc = raw.to_lowercase();
    let query_path = Path::new(raw);
    let id_query = query_lc
        .strip_suffix(".md")
        .unwrap_or(query_lc.as_str())
        .trim_end_matches('/')
        .to_string();

    // Exact id (path-shaped ids are stored lowercase without extension).
    if let Some(doc) = workspace.document_by_id(&id_query) {
        return Some(doc.path.clone());
    }

    // Absolute or workspace-relative filesystem path (with or without .md).
    let mut path_candidates = vec![query_path.to_path_buf(), workspace.root.join(query_path)];
    if query_path.extension().is_none() {
        let mut with_md = query_path.to_path_buf();
        with_md.set_extension("md");
        path_candidates.push(workspace.root.join(&with_md));
        path_candidates.push(with_md);
    }
    for candidate in path_candidates {
        let normalized = crate::fs::normalize_path(&candidate);
        if let Some(doc) = workspace.document_by_path(&normalized) {
            return Some(doc.path.clone());
        }
        if let Ok(canon) = normalized.canonicalize() {
            if let Some(doc) = workspace.document_by_path(&canon) {
                return Some(doc.path.clone());
            }
        }
    }

    // Absolute path under workspace → id form.
    if query_path.is_absolute() {
        if let Ok(rel) = query_path.strip_prefix(&workspace.root) {
            let rel_id = rel
                .with_extension("")
                .to_string_lossy()
                .replace('\\', "/")
                .to_lowercase();
            if let Some(doc) = workspace.document_by_id(&rel_id) {
                return Some(doc.path.clone());
            }
        }
        if let Ok(canon) = query_path.canonicalize() {
            if let Ok(rel) = canon.strip_prefix(&workspace.root) {
                let rel_id = rel
                    .with_extension("")
                    .to_string_lossy()
                    .replace('\\', "/")
                    .to_lowercase();
                if let Some(doc) = workspace.document_by_id(&rel_id) {
                    return Some(doc.path.clone());
                }
            }
        }
    }

    // Path suffix match (…/specs/ods/core.md) then unique file-stem match.
    if let Some(doc) = workspace.documents.iter().find(|doc| {
        doc.path.ends_with(query_path)
            || doc
                .path
                .to_string_lossy()
                .replace('\\', "/")
                .to_lowercase()
                .ends_with(&query_lc)
            || doc
                .path
                .to_string_lossy()
                .replace('\\', "/")
                .to_lowercase()
                .ends_with(&format!("{id_query}.md"))
    }) {
        return Some(doc.path.clone());
    }

    let stem_hits: Vec<_> = workspace
        .documents
        .iter()
        .filter(|doc| {
            doc.path
                .file_stem()
                .and_then(|s| s.to_str())
                .is_some_and(|s| s.eq_ignore_ascii_case(&id_query))
        })
        .map(|doc| doc.path.clone())
        .collect();
    if stem_hits.len() == 1 {
        return Some(stem_hits.into_iter().next().unwrap());
    }

    None
}

#[cfg(test)]
mod tests_context {
    use super::*;

    #[test]
    fn test_resolve_context_start_empty_and_missing() {
        let ws = Workspace::empty(PathBuf::from("/nonexistent_ws_root"));
        assert_eq!(resolve_context_start(&ws, ""), None);
        assert_eq!(resolve_context_start(&ws, "   "), None);
        assert_eq!(resolve_context_start(&ws, "missing_doc_xyz"), None);
    }

    #[test]
    fn test_is_ignored_edge_cases() {
        let root = Path::new("/workspace");
        let path = Path::new("/workspace/vendor/lib.md");

        assert!(is_ignored(path, root, &["vendor".to_string()]));
        assert!(!is_ignored(
            path,
            root,
            &["".to_string(), "other".to_string()]
        ));
    }
}
