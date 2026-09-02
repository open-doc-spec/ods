use crate::config::{WorkspaceConfig, load_workspace_config, ods_toml_enabled};
use crate::model::{Document, LoadOptions, Workspace};
use crate::parse::document_id;
use crate::pipeline::{discover_markdown_paths, parse_paths_parallel};
use crate::profiles::{
    load_profile_catalog, profile_catalog_roots_from_config, validate_custom_profile_paths,
    validate_custom_profile_placements,
};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Directory / file base names that tooling never treats as documentation content.
pub(crate) const DEFAULT_IGNORE_NAMES: &[&str] = &[
    "target",
    "node_modules",
    "dist",
    "build",
    ".artifacts",
    ".git",
    ".hg",
    ".svn",
    ".jj",
    "__pycache__",
    ".venv",
    "venv",
    "vendor",
];

/// Load options optimized for graph ops (lint, doctor, context): no body retention.
pub fn load_options_graph() -> LoadOptions {
    LoadOptions::default()
}

/// Load options when full markdown bodies must be retained in memory.
pub fn load_options_with_bodies() -> LoadOptions {
    LoadOptions {
        include_body: true,
        respect_gitignore: true,
    }
}

pub fn load_workspace(root: impl AsRef<Path>) -> io::Result<Workspace> {
    load_workspace_with_options(root, LoadOptions::default())
}

/// Functional pipeline: config → discover → parallel parse → rebuild_maps.
pub fn load_workspace_with_options(
    root: impl AsRef<Path>,
    options: LoadOptions,
) -> io::Result<Workspace> {
    let root = root
        .as_ref()
        .canonicalize()
        .unwrap_or_else(|_| root.as_ref().to_path_buf());

    let config = match load_workspace_config(&root) {
        Ok(config) => config,
        Err(err) if err.kind() == io::ErrorKind::NotFound => WorkspaceConfig::default(),
        Err(err) => return Err(err),
    };
    validate_custom_profile_paths(&root, &config)?;
    let gitignore = if options.respect_gitignore {
        load_gitignore_patterns(&root)
    } else {
        Vec::new()
    };

    let profile_roots = profile_catalog_roots_from_config(&root, &config);
    let profile_catalog = load_profile_catalog(&root, &profile_roots)?;
    let mut workspace_ignore = config.ignore.clone();
    workspace_ignore.extend(load_odsignore_patterns(&root));

    let paths = discover_markdown_paths(&root, &profile_roots, &gitignore, &workspace_ignore)?;

    let documents = parse_paths_parallel(&root, &paths, options.include_body)?;
    validate_custom_profile_placements(&root, &documents, &profile_roots, &config)?;

    let mut workspace = Workspace {
        root,
        config,
        documents,
        profiles: profile_catalog,
        profile_roots,
        by_id: HashMap::new(),
        by_path: HashMap::new(),
        children: HashMap::new(),
        resource_paths: HashSet::new(),
        code_paths: HashSet::new(),
        ignore: workspace_ignore,
        tag_index: std::collections::BTreeMap::new(),
        profile_catalog_paths: HashSet::new(),
        doc_dirs: HashSet::new(),
    };
    rebuild_indexes(&mut workspace);
    Ok(workspace)
}

/// Load ignore patterns from a `.odsignore` file if present at `root`.
pub fn load_odsignore_patterns(root: &Path) -> Vec<String> {
    let odsignore_path = root.join(".odsignore");
    let Ok(content) = fs::read_to_string(odsignore_path) else {
        return Vec::new();
    };
    content
        .lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(String::from)
        .collect()
}

/// Locate the ODS workspace root for a file path.
///
/// Nearest ancestor with a valid `ods.toml` (`spec` set).
pub fn find_workspace_root(path: impl AsRef<Path>) -> Option<PathBuf> {
    let path = path.as_ref();
    let abs = absolute_probe_path(path)?;
    let start = if abs.is_dir() {
        abs
    } else {
        abs.parent()
            .filter(|p| !p.as_os_str().is_empty())?
            .to_path_buf()
    };

    let mut current = start;

    loop {
        if current.as_os_str().is_empty() {
            break;
        }

        if ods_toml_enabled(&current) {
            return Some(current.canonicalize().unwrap_or(current));
        }

        if current.join(".git").exists() {
            break;
        }

        if !current.pop() || current.as_os_str().is_empty() {
            break;
        }
    }

    None
}

/// Make a probe path absolute without requiring it to exist on disk.
fn absolute_probe_path(path: &Path) -> Option<PathBuf> {
    if path.as_os_str().is_empty() {
        return std::env::current_dir().ok();
    }
    let joined = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir().ok()?.join(path)
    };
    Some(
        joined
            .canonicalize()
            .unwrap_or_else(|_| crate::fs::normalize_path(&joined)),
    )
}

/// Insert or replace a document and rebuild maps (incremental LSP path).
pub fn upsert_document(workspace: &mut Workspace, document: Document) {
    if let Some(idx) = workspace.by_path.get(&document.path).copied() {
        workspace.documents[idx] = document;
    } else if let Some(idx) = workspace
        .documents
        .iter()
        .position(|doc| doc.path == document.path)
    {
        workspace.documents[idx] = document;
    } else {
        workspace.documents.push(document);
    }
    rebuild_indexes(workspace);
}

/// Remove a document by path and rebuild maps.
pub fn remove_document(workspace: &mut Workspace, path: &Path) -> bool {
    let before = workspace.documents.len();
    workspace.documents.retain(|doc| doc.path != path);
    if workspace.documents.len() != before {
        rebuild_indexes(workspace);
        true
    } else {
        false
    }
}
