use crate::model::{Document, FrontmatterState, Workspace};
use crate::parse::document_id;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Compact per-document metadata held in the incremental store (no body).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocMeta {
    pub path: PathBuf,
    pub id: String,
    pub profile: Option<String>,
    pub status: Option<String>,
    pub share: Option<String>,
    pub description: Option<String>,
    pub tags: Vec<String>,
    pub depends: Vec<String>,
    pub related: Vec<String>,
    pub mtime: Option<SystemTime>,
    pub content_hash: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StorePatch {
    Upsert(PathBuf),
    Remove(PathBuf),
}

/// In-memory FM-only index. Suitable for `ods serve` (target ≤10 MB RSS for typical trees).
#[derive(Debug, Clone, Default)]
pub struct WorkspaceStore {
    pub root: PathBuf,
    pub by_path: HashMap<PathBuf, DocMeta>,
    pub by_id: HashMap<String, PathBuf>,
    pub tag_index: BTreeMap<String, Vec<String>>,
}

impl WorkspaceStore {
    #[must_use]
    pub fn from_workspace(workspace: &Workspace) -> Self {
        let mut store = Self {
            root: workspace.root.clone(),
            ..Default::default()
        };
        for doc in &workspace.documents {
            if let Some(meta) = meta_from_document(workspace, doc) {
                store.upsert_meta(meta);
            }
        }
        store
    }

    pub fn upsert_meta(&mut self, meta: DocMeta) {
        if let Some(old) = self.by_path.get(&meta.path) {
            self.by_id.remove(&old.id.to_lowercase());
            for tag in &old.tags {
                if let Some(ids) = self.tag_index.get_mut(tag) {
                    ids.retain(|id| id != &old.id);
                }
            }
        }
        let id_key = meta.id.to_lowercase();
        self.by_id.insert(id_key, meta.path.clone());
        for tag in &meta.tags {
            self.tag_index
                .entry(tag.clone())
                .or_default()
                .push(meta.id.clone());
        }
        self.by_path.insert(meta.path.clone(), meta);
    }

    pub fn remove_path(&mut self, path: &Path) {
        if let Some(old) = self.by_path.remove(path) {
            self.by_id.remove(&old.id.to_lowercase());
            for tag in &old.tags {
                if let Some(ids) = self.tag_index.get_mut(tag) {
                    ids.retain(|id| id != &old.id);
                }
            }
        }
    }

    /// Apply one filesystem change: reparse only that path (FM only).
    pub fn apply_patch(&mut self, patch: StorePatch) -> io::Result<()> {
        match patch {
            StorePatch::Remove(path) => {
                self.remove_path(&path);
                Ok(())
            }
            StorePatch::Upsert(path) => {
                let doc = crate::pipeline::parse_path(&self.root, path.clone(), false)?;
                let meta = meta_from_document_standalone(&self.root, &doc);
                if let Some(meta) = meta {
                    self.upsert_meta(meta);
                } else {
                    self.remove_path(&path);
                }
                Ok(())
            }
        }
    }

    /// Rough heap estimate for RSS budgeting (bytes).
    #[must_use]
    pub fn estimate_bytes(&self) -> usize {
        let mut n = std::mem::size_of::<Self>();
        for (p, m) in &self.by_path {
            n += p.as_os_str().len() + m.id.len() + m.tags.iter().map(String::len).sum::<usize>();
            n += m.depends.iter().map(String::len).sum::<usize>();
            n += 256; // overhead
        }
        n
    }

    /// True when estimated store size is under `max_rss_mb` soft budget.
    #[must_use]
    pub fn within_rss_budget(&self, max_rss_mb: u64) -> bool {
        let budget = (max_rss_mb as usize).saturating_mul(1024 * 1024);
        // Store is only part of process RSS; leave headroom factor 4 for binary/runtime.
        self.estimate_bytes().saturating_mul(4) < budget
    }
}

fn meta_from_document(workspace: &Workspace, doc: &Document) -> Option<DocMeta> {
    meta_from_document_standalone(&workspace.root, doc)
}

fn meta_from_document_standalone(root: &Path, doc: &Document) -> Option<DocMeta> {
    let fm = match &doc.frontmatter {
        FrontmatterState::Parsed(fm) => Some(fm),
        FrontmatterState::Absent => None,
        FrontmatterState::Invalid(_) => None,
    };
    let id = document_id(root, &doc.path, fm);
    let mtime = fs::metadata(&doc.path).and_then(|m| m.modified()).ok();
    let content_hash = {
        let mut h = 0u64;
        for b in doc.path.to_string_lossy().bytes() {
            h = h.wrapping_mul(31).wrapping_add(u64::from(b));
        }
        if let Some(fm) = fm {
            for t in &fm.tags {
                for b in t.bytes() {
                    h = h.wrapping_mul(31).wrapping_add(u64::from(b));
                }
            }
        }
        h
    };
    Some(DocMeta {
        path: doc.path.clone(),
        id,
        profile: fm.and_then(|f| f.profile.clone()),
        status: fm.and_then(|f| f.status.clone()),
        share: fm.and_then(|f| f.share.clone()),
        description: fm.and_then(|f| f.description.clone()),
        tags: fm.map(|f| f.tags.clone()).unwrap_or_default(),
        depends: fm.map(|f| f.depends.clone()).unwrap_or_default(),
        related: fm.map(|f| f.related.clone()).unwrap_or_default(),
        mtime,
        content_hash,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::WorkspaceConfig;
    use crate::config::write_ods_toml;
    use crate::fs::load_workspace;
    use tempfile::tempdir;

    #[test]
    fn patch_upsert_and_remove() {
        let td = tempdir().unwrap();
        let root = td.path();
        write_ods_toml(root, &WorkspaceConfig::new_workspace()).unwrap();
        fs::write(
            root.join("a.md"),
            "---\nprofile: note\nstatus: draft\ntags: [alpha]\n---\n\n# A\n",
        )
        .unwrap();
        let ws = load_workspace(root).unwrap();
        let mut store = WorkspaceStore::from_workspace(&ws);
        let n0 = store.by_path.len();
        assert!(n0 >= 1);
        assert!(store.within_rss_budget(10));

        fs::write(
            root.join("b.md"),
            "---\nprofile: note\nstatus: draft\ntags: [beta]\n---\n\n# B\n",
        )
        .unwrap();
        store
            .apply_patch(StorePatch::Upsert(root.join("b.md")))
            .unwrap();
        assert!(store.by_path.len() > n0);
        store
            .apply_patch(StorePatch::Remove(root.join("a.md")))
            .unwrap();
        assert!(store.by_path.values().any(|m| m.path.ends_with("b.md")));
        assert!(!store.by_path.contains_key(&root.join("a.md")));
    }

    #[test]
    fn patch_many_files_stays_bounded() {
        let td = tempdir().unwrap();
        let root = td.path();
        write_ods_toml(root, &WorkspaceConfig::new_workspace()).unwrap();
        let ws = load_workspace(root).unwrap();
        let mut store = WorkspaceStore::from_workspace(&ws);
        for i in 0..100 {
            let path = root.join(format!("n{i}.md"));
            fs::write(
                &path,
                format!("---\nprofile: note\nstatus: draft\ntags: [t{i}]\n---\n\n# N{i}\n"),
            )
            .unwrap();
            store.apply_patch(StorePatch::Upsert(path)).unwrap();
        }
        assert_eq!(store.by_path.len(), 100);
        assert!(store.within_rss_budget(10));
        let bytes = store.estimate_bytes();
        // Meta-only: 100 small docs should stay well under 1 MiB estimated.
        assert!(bytes < 1_000_000, "estimate_bytes={bytes}");
    }
}
