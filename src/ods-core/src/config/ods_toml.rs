//! `ods.toml` — sole workspace marker and policy home.

use crate::model::{SpecLintConfig, WorkspaceSpecsConfig, current_ods_spec_version};
use crate::parse::split_frontmatter;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// Path of the workspace config file under `root`.
#[must_use]
pub fn ods_toml_path(root: &Path) -> PathBuf {
    root.join("ods.toml")
}

/// Optional service / memory knobs (defaults keep `ods serve` lean).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceConfig {
    /// `auto` | `watch` | `poll` — default `poll` for low memory.
    #[serde(default = "default_service_mode")]
    pub mode: String,
    #[serde(default = "default_poll_secs")]
    pub poll_secs: u64,
    /// Soft RSS budget in megabytes for `ods serve` / `ods start`.
    #[serde(default = "default_max_rss_mb")]
    pub max_rss_mb: u64,
}

fn default_service_mode() -> String {
    "poll".into()
}
fn default_poll_secs() -> u64 {
    2
}
fn default_max_rss_mb() -> u64 {
    10
}

impl Default for ServiceConfig {
    fn default() -> Self {
        Self {
            mode: default_service_mode(),
            poll_secs: default_poll_secs(),
            max_rss_mb: default_max_rss_mb(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecsToml {
    #[serde(default)]
    pub okf: SpecEngineToml,
    #[serde(default)]
    pub skills: SpecEngineToml,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpecEngineToml {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_true")]
    pub check_keys: bool,
    #[serde(default)]
    pub ignore_keys: Vec<String>,
}

fn default_true() -> bool {
    true
}

impl Default for SpecEngineToml {
    fn default() -> Self {
        Self {
            enabled: false,
            check_keys: true,
            ignore_keys: Vec::new(),
        }
    }
}

/// Root workspace policy loaded from `ods.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Spec version string (e.g. `"0.1"`). Serde also accepts legacy key `ods`.
    #[serde(default, alias = "ods")]
    pub spec: String,
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default, alias = "custom-profiles")]
    pub custom_profiles: Vec<String>,
    #[serde(default)]
    pub packs: Vec<String>,
    #[serde(default)]
    pub specs: SpecsToml,
    #[serde(default)]
    pub service: ServiceConfig,
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            spec: current_ods_spec_version().to_string(),
            ignore: Vec::new(),
            custom_profiles: Vec::new(),
            packs: Vec::new(),
            specs: SpecsToml::default(),
            service: ServiceConfig::default(),
        }
    }
}

impl WorkspaceConfig {
    #[must_use]
    pub fn is_valid_marker(&self) -> bool {
        !self.spec.trim().is_empty()
    }

    #[must_use]
    pub fn to_workspace_specs(&self) -> WorkspaceSpecsConfig {
        WorkspaceSpecsConfig {
            okf: SpecLintConfig {
                enabled: self.specs.okf.enabled,
                check_keys: self.specs.okf.check_keys,
                ignore_keys: self.specs.okf.ignore_keys.iter().cloned().collect(),
            },
            skills: SpecLintConfig {
                enabled: self.specs.skills.enabled,
                check_keys: self.specs.skills.check_keys,
                ignore_keys: self.specs.skills.ignore_keys.iter().cloned().collect(),
            },
        }
    }

    /// Minimal config for a new workspace.
    #[must_use]
    pub fn new_workspace() -> Self {
        Self::default()
    }
}

/// Render `ods.toml` body (stable key order for small files).
#[must_use]
pub fn render_ods_toml(config: &WorkspaceConfig) -> String {
    // Prefer a readable hand-written shape over opaque serde dump for defaults.
    let mut out = String::new();
    out.push_str("# ODS workspace configuration\n");
    out.push_str(&format!("spec = \"{}\"\n", config.spec));
    if !config.ignore.is_empty() {
        out.push_str("\nignore = [\n");
        for p in &config.ignore {
            out.push_str(&format!("  \"{}\",\n", escape_toml_str(p)));
        }
        out.push_str("]\n");
    }
    if !config.custom_profiles.is_empty() {
        out.push_str("\ncustom_profiles = [\n");
        for p in &config.custom_profiles {
            out.push_str(&format!("  \"{}\",\n", escape_toml_str(p)));
        }
        out.push_str("]\n");
    }
    if !config.packs.is_empty() {
        out.push_str("\npacks = [\n");
        for p in &config.packs {
            out.push_str(&format!("  \"{}\",\n", escape_toml_str(p)));
        }
        out.push_str("]\n");
    }
    if config.specs.okf.enabled || config.specs.skills.enabled {
        if config.specs.okf.enabled {
            out.push_str("\n[specs.okf]\nenabled = true\n");
        }
        if config.specs.skills.enabled {
            out.push_str("\n[specs.skills]\nenabled = true\n");
        }
    }
    out.push_str("\n[service]\n");
    out.push_str(&format!("mode = \"{}\"\n", config.service.mode));
    out.push_str(&format!("poll_secs = {}\n", config.service.poll_secs));
    out.push_str(&format!("max_rss_mb = {}\n", config.service.max_rss_mb));
    out
}

fn escape_toml_str(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

/// Write `ods.toml` at `root`.
pub fn write_ods_toml(root: &Path, config: &WorkspaceConfig) -> io::Result<PathBuf> {
    let path = ods_toml_path(root);
    fs::write(&path, render_ods_toml(config))?;
    Ok(path)
}

/// Load workspace config. Prefers `ods.toml`; falls back to legacy root `index.ods.md` policy keys.
pub fn load_workspace_config(root: &Path) -> io::Result<WorkspaceConfig> {
    let toml_path = ods_toml_path(root);
    if toml_path.is_file() {
        let text = fs::read_to_string(&toml_path)?;
        return parse_ods_toml(&text).map_err(|e| {
            io::Error::new(io::ErrorKind::InvalidData, format!("invalid ods.toml: {e}"))
        });
    }
    if let Some(cfg) = load_legacy_root_index_config(root) {
        return Ok(cfg);
    }
    Err(io::Error::new(
        io::ErrorKind::NotFound,
        "missing ods.toml (workspace marker)",
    ))
}

/// Parse TOML text into [`WorkspaceConfig`].
pub fn parse_ods_toml(text: &str) -> Result<WorkspaceConfig, toml::de::Error> {
    let mut cfg: WorkspaceConfig = toml::from_str(text)?;
    if cfg.spec.trim().is_empty() {
        cfg.spec = current_ods_spec_version().to_string();
    }
    Ok(cfg)
}

/// True when `root/ods.toml` exists and parses with a non-empty `spec`.
#[must_use]
pub fn ods_toml_enabled(root: &Path) -> bool {
    let path = ods_toml_path(root);
    if !path.is_file() {
        return false;
    }
    fs::read_to_string(&path)
        .ok()
        .and_then(|t| parse_ods_toml(&t).ok())
        .is_some_and(|c| c.is_valid_marker())
}

/// Migrate legacy root `index.ods.md` / `index.md` → `ods.toml`. Returns written path if any.
pub fn migrate_root_index_to_toml(root: &Path) -> io::Result<Option<PathBuf>> {
    if ods_toml_path(root).is_file() {
        return Ok(None);
    }
    let Some(cfg) = load_legacy_root_index_config(root) else {
        return Ok(None);
    };
    let path = write_ods_toml(root, &cfg)?;
    // Remove legacy root index marker files (nested indexes are not recreated).
    for name in ["index.ods.md", "index.md"] {
        let p = root.join(name);
        if p.is_file() {
            let _ = fs::remove_file(&p);
        }
    }
    Ok(Some(path))
}

fn load_legacy_root_index_config(root: &Path) -> Option<WorkspaceConfig> {
    for name in ["index.ods.md", "index.md"] {
        let path = root.join(name);
        if !path.is_file() {
            continue;
        }
        let text = fs::read_to_string(&path).ok()?;
        let (fm_block, _) = split_frontmatter(&text);
        let block = fm_block?;
        if !block.lines().any(|l| {
            let t = l.trim();
            t.starts_with("ods:") || t.split_once(':').is_some_and(|(k, _)| k.trim() == "ods")
        }) {
            continue;
        }
        // Reuse document parser for nested maps / lists.
        let doc = crate::parse::parse_document_text(root, path, &text, false);
        if let crate::model::FrontmatterState::Parsed(fm) = doc.frontmatter {
            let mut cfg = WorkspaceConfig {
                spec: fm
                    .ods
                    .unwrap_or_else(|| current_ods_spec_version().to_string()),
                ignore: fm.ignore,
                custom_profiles: fm.profiles,
                packs: fm.packs,
                specs: SpecsToml {
                    okf: SpecEngineToml {
                        enabled: fm.specs.okf.enabled,
                        check_keys: fm.specs.okf.check_keys,
                        ignore_keys: fm.specs.okf.ignore_keys.into_iter().collect(),
                    },
                    skills: SpecEngineToml {
                        enabled: fm.specs.skills.enabled,
                        check_keys: fm.specs.skills.check_keys,
                        ignore_keys: fm.specs.skills.ignore_keys.into_iter().collect(),
                    },
                },
                service: ServiceConfig::default(),
            };
            if cfg.spec.trim().is_empty() {
                cfg.spec = current_ods_spec_version().to_string();
            }
            return Some(cfg);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn round_trip_minimal() {
        let cfg = WorkspaceConfig::new_workspace();
        let text = render_ods_toml(&cfg);
        let parsed = parse_ods_toml(&text).unwrap();
        assert_eq!(parsed.spec, current_ods_spec_version());
        assert_eq!(parsed.service.max_rss_mb, 10);
    }

    #[test]
    fn write_and_load() {
        let td = tempdir().unwrap();
        let mut cfg = WorkspaceConfig::new_workspace();
        cfg.ignore = vec!["target".into(), "src".into()];
        write_ods_toml(td.path(), &cfg).unwrap();
        let loaded = load_workspace_config(td.path()).unwrap();
        assert_eq!(loaded.ignore, vec!["target", "src"]);
    }

    #[test]
    fn migrate_from_legacy_index() {
        let td = tempdir().unwrap();
        fs::write(
            td.path().join("index.ods.md"),
            "---\nprofile: index\nods: 0.1\nignore:\n  - foo\n---\n\n# R\n",
        )
        .unwrap();
        let path = migrate_root_index_to_toml(td.path()).unwrap().unwrap();
        assert!(path.ends_with("ods.toml"));
        assert!(!td.path().join("index.ods.md").exists());
        let cfg = load_workspace_config(td.path()).unwrap();
        assert_eq!(cfg.ignore, vec!["foo"]);
    }
}
