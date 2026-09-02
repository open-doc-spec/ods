use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::path::PathBuf;

#[must_use]
pub fn current_ods_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[must_use]
pub fn current_ods_spec_version() -> &'static str {
    "2.0"
}

/// Parse workspace `spec` string into a comparable major.minor pair.
#[must_use]
pub fn parse_spec_version(spec: &str) -> (u32, u32) {
    let trimmed = spec.trim();
    let mut parts = trimmed.split('.');
    let major = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    let minor = parts.next().and_then(|s| s.parse().ok()).unwrap_or(0);
    (major, minor)
}

#[must_use]
pub fn is_spec_at_least(spec: &str, major: u32, minor: u32) -> bool {
    let (m, n) = parse_spec_version(spec);
    m > major || (m == major && n >= minor)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

/// Workspace compliance outcome (binary — no Level 0–3 ladder).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WorkspaceCompliance {
    Compliant,
    NonCompliant,
}

/// Lint always runs the full integrity rule set (binary compliance — no partial modes).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ComplianceMode {
    #[default]
    Full,
}

pub type LintLevel = ComplianceMode;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub path: PathBuf,
    pub severity: Severity,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResourceRef {
    /// Local path. Empty for a `url`-only entry (ODS 2.1 allows exactly one of the two).
    pub path: PathBuf,
    /// External URL, when the entry was written as a mapping with `url:`.
    pub url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CodeRole {
    Entrypoint,
    Implementation,
    Test,
    Schema,
    Migration,
    Config,
    Infrastructure,
    Pipeline,
}

impl CodeRole {
    #[must_use]
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_lowercase().as_str() {
            "entrypoint" => Some(Self::Entrypoint),
            "implementation" => Some(Self::Implementation),
            "test" => Some(Self::Test),
            "schema" => Some(Self::Schema),
            "migration" => Some(Self::Migration),
            "config" => Some(Self::Config),
            "infrastructure" => Some(Self::Infrastructure),
            "pipeline" => Some(Self::Pipeline),
            _ => None,
        }
    }

    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Entrypoint => "entrypoint",
            Self::Implementation => "implementation",
            Self::Test => "test",
            Self::Schema => "schema",
            Self::Migration => "migration",
            Self::Config => "config",
            Self::Infrastructure => "infrastructure",
            Self::Pipeline => "pipeline",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CodeRef {
    pub path: PathBuf,
    pub symbol: Option<String>,
    pub role: CodeRole,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextSpec {
    pub load: Vec<String>,
    pub ignore: Vec<String>,
    pub max_depth: Option<usize>,
}

/// ODS 2.1 typed `related` entry (plain path, Pareto predicate, or custom verb).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RelatedEntry {
    Path(String),
    Predicate { predicate: String, target: String },
    Custom { verb: String, target: String },
}

impl RelatedEntry {
    #[must_use]
    pub fn target_path(&self) -> Option<&str> {
        match self {
            Self::Path(p) | Self::Predicate { target: p, .. } | Self::Custom { target: p, .. } => {
                Some(p.as_str())
            }
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpecLintConfig {
    pub enabled: bool,
    pub check_keys: bool,
    pub ignore_keys: HashSet<String>,
}

impl Default for SpecLintConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            check_keys: true,
            ignore_keys: HashSet::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WorkspaceSpecsConfig {
    pub okf: SpecLintConfig,
    pub skills: SpecLintConfig,
}

/// Eq-safe custom frontmatter value (unknown top-level keys only).
///
/// Structured values are retained as opaque values so callers can distinguish
/// them from an explicit YAML null when checking whether a key is present.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum CustomValue {
    #[default]
    Null,
    String(String),
    List(Vec<String>),
    Opaque,
}

impl CustomValue {
    /// Scalar / list string values for query matching.
    pub fn as_query_strings(&self) -> Vec<String> {
        match self {
            CustomValue::Null => Vec::new(),
            CustomValue::String(s) => {
                if s.is_empty() {
                    Vec::new()
                } else {
                    vec![s.clone()]
                }
            }
            CustomValue::List(items) => items.iter().filter(|s| !s.is_empty()).cloned().collect(),
            CustomValue::Opaque => Vec::new(),
        }
    }

    /// Best-effort subfield extraction for dotted key queries (`key.subfield`).
    pub fn map_subfield_query_strings(&self, _subfield: &str) -> Vec<String> {
        match self {
            CustomValue::Null => Vec::new(),
            CustomValue::String(s) if s.is_empty() => Vec::new(),
            CustomValue::String(s) => vec![s.clone()],
            CustomValue::List(items) => items.iter().filter(|s| !s.is_empty()).cloned().collect(),
            CustomValue::Opaque => Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Frontmatter {
    pub profile: Option<String>,
    pub status: Option<String>,
    pub created: Option<String>,
    pub updated: Option<String>,
    pub share: Option<String>,
    pub description: Option<String>,
    pub id: Option<String>,
    pub profiles: Vec<String>,
    pub packs: Vec<String>,
    pub depends: Vec<String>,
    pub related: Vec<String>,
    /// ODS 2.1 typed `related` entries (predicates + custom verbs).
    pub related_entries: Vec<RelatedEntry>,
    pub resources: Vec<ResourceRef>,
    pub code: Vec<CodeRef>,
    /// Legacy 1.x nested `context:` block (migrate → top-level `load`).
    pub context: Option<ContextSpec>,
    /// ODS 2.0 top-level auxiliary files for AI context injection.
    pub load: Vec<String>,
    pub owner: Option<String>,
    pub author: Option<String>,
    pub tags: Vec<String>,
    /// True when `tags` was found nested under `ods:` (invalid placement).
    pub tags_misplaced: bool,
    /// True when a nested `ods:` map block was present (rejected in 2.0+).
    pub ods_wrapper: bool,
    pub ods: Option<String>,
    /// Workspace-relative path prefixes to exclude from scan (legacy root-index policy; prefer `ods.toml` `ignore`).
    pub ignore: Vec<String>,
    pub name: Option<String>,
    pub title: Option<String>,
    /// Legacy 1.x key (rejected in 2.0+).
    pub custom_profile: Option<CustomProfileDefinition>,
    pub specs: WorkspaceSpecsConfig,
    // ODS 2.1 Pareto ontology
    pub entity: Option<String>,
    pub domain: Option<String>,
    /// Frontmatter key `schema` — path to JSON Schema on disk (ONT-001).
    pub schema_path: Option<String>,
    // OKF v0.2 superset (flat top-level)
    pub concept_type: Option<String>,
    pub resource: Option<String>,
    pub okf_version: Option<String>,
    pub runtime: Option<String>,
    pub computation: Option<String>,
    pub stale_after: Option<String>,
    /// True when `code` used legacy object form with role/symbol (rejected in 2.0+).
    pub code_object_form: bool,
    /// Known top-level keys whose YAML value was explicitly non-null.
    pub non_null_keys: BTreeSet<String>,
    /// All top-level keys encountered, including keys with an explicit null.
    pub present_keys: BTreeSet<String>,
    /// Non-standard top-level frontmatter keys (custom profiles, domain metadata).
    pub custom_keys: BTreeMap<String, CustomValue>,
}

impl Frontmatter {
    /// All Markdown paths referenced in `related` (plain + typed targets).
    #[must_use]
    pub fn related_targets(&self) -> Vec<String> {
        let mut out: Vec<String> = self.related.clone();
        for entry in &self.related_entries {
            if let Some(t) = entry.target_path() {
                out.push(t.to_string());
            }
        }
        out
    }

    /// Scalar/list values for key-based discovery (`ods find --key`).
    #[must_use]
    pub fn key_query_values(&self, key: &str) -> Vec<String> {
        let norm = key.trim().to_lowercase();
        // Dot-path for map subfields (e.g. usage_window.from).
        if let Some((parent, child)) = norm.split_once('.') {
            if let Some(val) = self.custom_keys.get(parent) {
                return val.map_subfield_query_strings(child);
            }
            return match (parent, child) {
                ("usage_window", "from" | "to") => self
                    .custom_keys
                    .get("usage_window")
                    .map(|v| v.map_subfield_query_strings(child))
                    .unwrap_or_default(),
                ("generated", "by" | "at") => self
                    .custom_keys
                    .get("generated")
                    .map(|v| v.map_subfield_query_strings(child))
                    .unwrap_or_default(),
                _ => Vec::new(),
            };
        }
        match norm.as_str() {
            "profile" => self.profile.clone().into_iter().collect(),
            "status" => self.status.clone().into_iter().collect(),
            "owner" => self.owner.clone().into_iter().collect(),
            "author" => self.author.clone().into_iter().collect(),
            "share" => self.share.clone().into_iter().collect(),
            "description" => self.description.clone().into_iter().collect(),
            "id" => self.id.clone().into_iter().collect(),
            "title" => self
                .title
                .clone()
                .or_else(|| self.name.clone())
                .into_iter()
                .collect(),
            "name" => self.name.clone().into_iter().collect(),
            "created" | "created_at" | "date" => self.created.clone().into_iter().collect(),
            "updated" | "updated_at" | "last_updated" => self.updated.clone().into_iter().collect(),
            "tags" => self.tags.clone(),
            "depends" => self.depends.clone(),
            "related" => self.related_targets(),
            "load" => self.load.clone(),
            "entity" => self.entity.clone().into_iter().collect(),
            "domain" => self.domain.clone().into_iter().collect(),
            "schema" => self.schema_path.clone().into_iter().collect(),
            "type" => self.concept_type.clone().into_iter().collect(),
            "resource" => self.resource.clone().into_iter().collect(),
            "okf_version" => self.okf_version.clone().into_iter().collect(),
            "runtime" => self.runtime.clone().into_iter().collect(),
            "computation" => self.computation.clone().into_iter().collect(),
            "stale_after" => self.stale_after.clone().into_iter().collect(),
            "packs" => self.packs.clone(),
            "profiles" | "custom-profiles" => self.profiles.clone(),
            "code" => self
                .code
                .iter()
                .map(|c| c.path.to_string_lossy().to_string())
                .collect(),
            "resources" => self
                .resources
                .iter()
                .map(|r| r.path.to_string_lossy().to_string())
                .collect(),
            other => self
                .custom_keys
                .get(other)
                .map(|val| val.as_query_strings())
                .unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CustomProfileDefinition {
    pub name: Option<String>,
    pub required_keys: Vec<String>,
    pub optional_keys: Vec<String>,
    pub forbidden_keys: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(clippy::large_enum_variant)]
pub enum FrontmatterState {
    Absent,
    Parsed(Frontmatter),
    Invalid(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Document {
    pub path: PathBuf,
    pub directory: PathBuf,
    pub body: String,
    pub headings: Vec<String>,
    pub frontmatter: FrontmatterState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileDefinition {
    pub name: String,
    pub sections: Vec<Vec<String>>,
    pub required_keys: Vec<String>,
    pub optional_keys: Vec<String>,
    pub forbidden_keys: Vec<String>,
    pub source: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileConflict {
    pub name: String,
    pub kept: PathBuf,
    pub ignored: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProfileCatalog {
    pub definitions: BTreeMap<String, ProfileDefinition>,
    pub conflicts: Vec<ProfileConflict>,
}

/// Load options for workspace scanning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadOptions {
    /// When false, document bodies are left empty (headings still extracted). Faster for large trees.
    pub include_body: bool,
    /// When true, also apply patterns from the nearest `.gitignore` under the root.
    pub respect_gitignore: bool,
}

impl Default for LoadOptions {
    fn default() -> Self {
        Self {
            include_body: false,
            respect_gitignore: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Workspace {
    pub root: PathBuf,
    /// Policy from `ods.toml` (or legacy root index migration).
    pub config: crate::config::WorkspaceConfig,
    pub documents: Vec<Document>,
    pub profiles: ProfileCatalog,
    pub profile_roots: Vec<PathBuf>,
    /// Lowercase document id -> index into `documents`.
    pub by_id: HashMap<String, usize>,
    /// Absolute/normalized path -> index into `documents`.
    pub by_path: HashMap<PathBuf, usize>,
    /// Directory -> sorted relative child entry strings (for virtual browse).
    pub children: HashMap<PathBuf, Vec<String>>,
    /// Absolute paths of files declared as resources.
    pub resource_paths: HashSet<PathBuf>,
    /// Absolute paths of files declared as code references.
    pub code_paths: HashSet<PathBuf>,
    /// Workspace-relative path prefixes from `ods.toml` `ignore` (scan scope).
    pub ignore: Vec<String>,
    /// Normalized tag → document ids (observed project tags). Rebuilt with maps.
    pub tag_index: BTreeMap<String, Vec<String>>,
    /// Absolute directory roots of nested profile catalogs from `custom_profiles`.
    pub profile_catalog_paths: HashSet<PathBuf>,
    /// Every directory that is itself, or is an ancestor of, at least one
    /// non-ignored document. O(1)-checkable substitute for scanning
    /// `documents` per call — see `fs::scanner::directory_children_for`.
    pub doc_dirs: HashSet<PathBuf>,
}

impl Workspace {
    pub fn empty(root: PathBuf) -> Self {
        Self {
            root,
            config: crate::config::WorkspaceConfig::default(),
            documents: Vec::new(),
            profiles: ProfileCatalog::default(),
            profile_roots: Vec::new(),
            by_id: HashMap::new(),
            by_path: HashMap::new(),
            children: HashMap::new(),
            resource_paths: HashSet::new(),
            code_paths: HashSet::new(),
            ignore: Vec::new(),
            tag_index: BTreeMap::new(),
            profile_catalog_paths: HashSet::new(),
            doc_dirs: HashSet::new(),
        }
    }

    pub fn document_by_id(&self, id: &str) -> Option<&Document> {
        self.by_id
            .get(&id.to_lowercase())
            .and_then(|&idx| self.documents.get(idx))
    }

    pub fn document_by_path(&self, path: &std::path::Path) -> Option<&Document> {
        self.by_path
            .get(path)
            .or_else(|| {
                path.canonicalize()
                    .ok()
                    .as_ref()
                    .and_then(|p| self.by_path.get(p))
            })
            .and_then(|&idx| self.documents.get(idx))
    }
}

#[cfg(test)]
mod test_model {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_model_helpers() {
        assert_eq!(current_ods_spec_version(), "2.0");
        assert!(is_spec_at_least("2.1", 2, 1));
        assert!(is_spec_at_least("2.1.0", 2, 1));
        assert!(!is_spec_at_least("2.0", 2, 1));
        assert!(!current_ods_version().is_empty());

        assert_eq!(CodeRole::parse("entrypoint"), Some(CodeRole::Entrypoint));
        assert_eq!(
            CodeRole::parse("implementation"),
            Some(CodeRole::Implementation)
        );
        assert_eq!(CodeRole::parse("test"), Some(CodeRole::Test));
        assert_eq!(CodeRole::parse("schema"), Some(CodeRole::Schema));
        assert_eq!(CodeRole::parse("migration"), Some(CodeRole::Migration));
        assert_eq!(CodeRole::parse("config"), Some(CodeRole::Config));
        assert_eq!(
            CodeRole::parse("infrastructure"),
            Some(CodeRole::Infrastructure)
        );
        assert_eq!(CodeRole::parse("pipeline"), Some(CodeRole::Pipeline));
        assert_eq!(CodeRole::parse("invalid"), None);

        assert_eq!(CodeRole::Entrypoint.as_str(), "entrypoint");
        assert_eq!(CodeRole::Implementation.as_str(), "implementation");
        assert_eq!(CodeRole::Test.as_str(), "test");
        assert_eq!(CodeRole::Schema.as_str(), "schema");
        assert_eq!(CodeRole::Migration.as_str(), "migration");
        assert_eq!(CodeRole::Config.as_str(), "config");
        assert_eq!(CodeRole::Infrastructure.as_str(), "infrastructure");
        assert_eq!(CodeRole::Pipeline.as_str(), "pipeline");

        let ws = Workspace::empty(PathBuf::from("/tmp"));
        assert!(ws.document_by_id("nonexistent").is_none());
        assert!(ws.document_by_path(Path::new("/tmp/nonexistent")).is_none());
    }
}
