// Opt-in init / opt-out disable for ODS workspaces.
//
// Init is explicit (`ods init`). Disable strips ODS metadata and leaves prose intact.

use crate::adopt::{AdoptOptions, adopt_workspace};
use crate::config::{
    WorkspaceConfig, migrate_root_index_to_toml, ods_toml_enabled, ods_toml_path, write_ods_toml,
};
use crate::fs::{find_workspace_root, load_workspace};
use crate::parse::split_frontmatter;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// True when `root` has a valid `ods.toml`.
pub fn ods_enabled(root: impl AsRef<Path>) -> bool {
    ods_toml_enabled(root.as_ref())
}

/// Resolve whether ODS is enabled for a path (file or directory).
pub fn ods_enabled_for_path(path: impl AsRef<Path>) -> bool {
    find_workspace_root(path.as_ref())
        .map(ods_enabled)
        .unwrap_or(false)
}

/// Options for `disable_workspace`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DisableOptions {
    /// Apply changes (default false = dry-run).
    pub write: bool,
    /// Strip ODS frontmatter keys from documents (default true).
    pub strip_frontmatter: bool,
    /// Strip root policy keys spec/custom_profiles/ignore/packs (default true).
    pub strip_root_policy: bool,
    /// Delete non-root legacy index.md files (default false).
    pub remove_indexes: bool,
    /// Delete root index.md / ods.toml (default false; dangerous).
    pub remove_root_index: bool,
}

impl Default for DisableOptions {
    fn default() -> Self {
        Self {
            write: false,
            strip_frontmatter: true,
            strip_root_policy: true,
            remove_indexes: false,
            remove_root_index: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DisableReport {
    pub root: PathBuf,
    pub already_disabled: bool,
    pub would_edit: Vec<PathBuf>,
    pub edited: Vec<PathBuf>,
    pub would_delete: Vec<PathBuf>,
    pub deleted: Vec<PathBuf>,
    pub dry_run: bool,
}

/// Options for [`init_workspace`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InitOptions {
    /// Run adopt --write after ensuring root marker.
    pub adopt: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct InitReport {
    pub root: PathBuf,
    /// Created `ods.toml` or migrated from legacy root index.
    pub initialized: bool,
    /// Root already had valid `ods.toml`.
    pub already_initialized: bool,
    pub adopted: Vec<PathBuf>,
    /// Always empty — nested indexes are not generated.
    pub indexes: Vec<PathBuf>,
}

/// Ensure workspace has `ods.toml`, optionally adopt plain files.
///
/// Single opt-in path for `ods init` (replaces the former `enable` command).
pub fn init_workspace(root: impl AsRef<Path>, options: InitOptions) -> io::Result<InitReport> {
    let root = canonical_or_original(root.as_ref());
    fs::create_dir_all(&root)?;
    let mut report = InitReport {
        root: root.clone(),
        ..Default::default()
    };

    let toml = ods_toml_path(&root);
    if toml.is_file() {
        if let Ok(mut cfg) = crate::config::load_workspace_config(&root) {
            if cfg.spec.trim() != crate::model::current_ods_spec_version() {
                cfg.spec = crate::model::current_ods_spec_version().to_string();
                let _ = write_ods_toml(&root, &cfg);
                report.initialized = true;
            } else {
                report.already_initialized = true;
            }
        } else {
            report.already_initialized = true;
        }
    } else if let Some(_path) = migrate_root_index_to_toml(&root)? {
        report.initialized = true;
    } else {
        write_ods_toml(&root, &WorkspaceConfig::new_workspace())?;
        report.initialized = true;
    }

    let workspace = load_workspace(&root)?;
    let _workspace = if options.adopt {
        let adopt_report = adopt_workspace(&workspace, AdoptOptions { write: true })?;
        report.adopted = adopt_report.written;
        load_workspace(&root)?
    } else {
        workspace
    };
    report.indexes = Vec::new();
    Ok(report)
}

/// Dry-run or apply ODS disable / revert to plain Markdown metadata.
pub fn disable_workspace(
    root: impl AsRef<Path>,
    options: DisableOptions,
) -> io::Result<DisableReport> {
    let root = canonical_or_original(root.as_ref());
    let mut report = DisableReport {
        root: root.clone(),
        dry_run: !options.write,
        ..Default::default()
    };

    if !ods_enabled(&root) {
        // Still allow stripping if user pointed at a tree with frontmatter but no ods:
        // Prefer strict: already disabled when no ods: marker.
        report.already_disabled = true;
        return Ok(report);
    }

    let workspace = load_workspace(&root)?;
    let root_index = root.join("index.ods.md");
    let root_toml = ods_toml_path(&root);

    if (options.remove_root_index || options.strip_root_policy) && root_toml.is_file() {
        report.would_delete.push(root_toml.clone());
        if options.write {
            fs::remove_file(&root_toml)?;
            report.deleted.push(root_toml.clone());
        }
    }

    for document in &workspace.documents {
        let path = &document.path;
        let is_root_index = path == &root_index || path == &root.join("index.md");
        let is_index = path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n == "index.md" || n == "index.ods.md");

        if options.remove_indexes && is_index && !is_root_index {
            report.would_delete.push(path.clone());
            if options.write {
                fs::remove_file(path)?;
                report.deleted.push(path.clone());
            }
            continue;
        }

        if options.remove_root_index && is_root_index {
            report.would_delete.push(path.clone());
            if options.write {
                fs::remove_file(path)?;
                report.deleted.push(path.clone());
            }
            continue;
        }

        if !(options.strip_frontmatter || (is_root_index && options.strip_root_policy)) {
            continue;
        }

        let text = fs::read_to_string(path)?;
        let strip_doc = options.strip_frontmatter;
        let strip_root = is_root_index && options.strip_root_policy;
        let (next, changed) = strip_ods_from_document_text(&text, strip_doc, strip_root);
        if !changed {
            continue;
        }
        // Body must be unchanged
        let (_, body_before) = split_frontmatter(&text);
        let (_, body_after) = split_frontmatter(&next);
        let body_before = body_before.trim_start_matches(['\r', '\n']);
        let body_after = body_after.trim_start_matches(['\r', '\n']);
        if body_before != body_after {
            return Err(io::Error::other(crate::error::lifecycle_refuse_body_change(
                path.display(),
            )));
        }

        report.would_edit.push(path.clone());
        if options.write {
            fs::write(path, next)?;
            report.edited.push(path.clone());
        }
    }

    Ok(report)
}

fn canonical_or_original(path: &Path) -> PathBuf {
    path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
}
