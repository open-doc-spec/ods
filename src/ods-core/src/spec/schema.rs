//! Declarative spec key schemas — single source of truth for dialect keys.
//!
//! Parse keeps a typed `Frontmatter` model; lint / `ods schema` / dialect
//! registration consult this registry so adding or updating keys is a schema
//! change rather than N hardcoded match arms.

use crate::model::{Diagnostic, Frontmatter, Severity};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SpecKind {
    Ods,
    Okf,
    Skills,
    Custom(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyPlacement {
    /// Universal / domain keys at YAML top level (e.g. tags, description, OKF type).
    TopLevel,
    /// ODS engine keys nested under the `ods:` map.
    NestedEngineMap,
    /// Deprecated alias for [`WorkspaceConfigOnly`] (legacy name: root index).
    RootIndexOnly,
    /// Only valid in workspace `ods.toml` (not ordinary document frontmatter).
    WorkspaceConfigOnly,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyType {
    String,
    List,
    Map,
    /// String or list of strings (e.g. owner).
    StringOrList,
    Enum(Vec<String>),
    Timestamp,
}

#[derive(Debug, Clone)]
pub struct KeyDefinition {
    pub name: String,
    pub placement: KeyPlacement,
    pub key_type: KeyType,
    pub required: bool,
    pub description: String,
    /// Alternate spellings accepted by parsers (e.g. created_at → created).
    pub aliases: Vec<String>,
}

impl KeyDefinition {
    fn new(
        name: impl Into<String>,
        placement: KeyPlacement,
        key_type: KeyType,
        required: bool,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            placement,
            key_type,
            required,
            description: description.into(),
            aliases: Vec::new(),
        }
    }

    fn with_aliases(mut self, aliases: &[&str]) -> Self {
        self.aliases = aliases.iter().map(|s| (*s).to_string()).collect();
        self
    }
}

#[derive(Debug, Clone)]
pub struct SpecSchema {
    pub kind: SpecKind,
    pub version: String,
    pub keys: HashMap<String, KeyDefinition>,
    /// When true, unknown keys are preserved (ODS / OKF / Skills default).
    pub preserve_unknown: bool,
}

impl SpecSchema {
    pub fn new(kind: SpecKind, version: impl Into<String>) -> Self {
        Self {
            kind,
            version: version.into(),
            keys: HashMap::new(),
            preserve_unknown: true,
        }
    }

    pub fn add_key(&mut self, def: KeyDefinition) {
        self.keys.insert(def.name.clone(), def);
    }

    pub fn key_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.keys.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }

    pub fn keys_with_placement(&self, placement: KeyPlacement) -> Vec<&KeyDefinition> {
        let mut out: Vec<&KeyDefinition> = self
            .keys
            .values()
            .filter(|k| k.placement == placement)
            .collect();
        out.sort_by(|a, b| a.name.cmp(&b.name));
        out
    }

    /// Document frontmatter keys stripped by `ods disable` (never foreign SSG keys).
    ///
    /// Includes all ODS domain keys at the flat top level (2.0+).
    pub fn document_disable_strip_keys(&self) -> Vec<String> {
        const SKIP: &[&str] = &["$schema"];
        let mut out = vec!["ods".to_string()];
        for def in self.keys.values() {
            if def.placement == KeyPlacement::TopLevel && !SKIP.contains(&def.name.as_str()) {
                out.push(def.name.clone());
            }
        }
        out.sort();
        out.dedup();
        out
    }

    /// Workspace/root policy keys stripped by disable root-policy (ods.toml / legacy root).
    pub fn workspace_policy_strip_keys(&self) -> Vec<String> {
        /// Dialect lint pins sometimes left on legacy root indexes.
        const LEGACY_LINT_PINS: &[&str] = &["okf_lint", "okf-lint", "skills_lint", "skills-lint"];
        let mut out = vec!["ods".to_string()];
        for def in self.keys.values() {
            if matches!(
                def.placement,
                KeyPlacement::WorkspaceConfigOnly | KeyPlacement::RootIndexOnly
            ) {
                out.push(def.name.clone());
            }
        }
        for pin in LEGACY_LINT_PINS {
            out.push((*pin).to_string());
        }
        out.sort();
        out.dedup();
        out
    }

    /// Flat engine key names in canonical emit order (ODS 2.0+).
    pub fn canonical_engine_key_order(&self) -> Vec<&str> {
        const ORDER: &[&str] = &[
            "profile",
            "status",
            "id",
            "share",
            "depends",
            "related",
            "resources",
            "code",
            "load",
        ];
        ORDER
            .iter()
            .copied()
            .filter(|name| self.keys.contains_key(*name))
            .collect()
    }

    pub fn find_similar_key(&self, name: &str) -> Option<&KeyDefinition> {
        let name_lower = name.to_ascii_lowercase();
        let mut best: Option<(&KeyDefinition, usize)> = None;

        for key in self.keys.values() {
            let d = crate::error::edit_distance(&name_lower, &key.name.to_ascii_lowercase());
            if d > 0 && d <= 2 {
                match best {
                    Some((_, bd)) if d >= bd => {}
                    _ => best = Some((key, d)),
                }
            }

            for alias in &key.aliases {
                let da = crate::error::edit_distance(&name_lower, &alias.to_ascii_lowercase());
                if da > 0 && da <= 2 {
                    match best {
                        Some((_, bd)) if da >= bd => {}
                        _ => best = Some((key, da)),
                    }
                }
            }
        }
        best.map(|(k, _)| k)
    }
}

#[derive(Debug, Default, Clone)]
pub struct SpecSchemaRegistry {
    schemas: HashMap<String, SpecSchema>,
}

impl SpecSchemaRegistry {
    pub fn with_defaults() -> Self {
        let mut reg = Self::default();
        reg.register_ods_schema();
        reg.register_okf_schema();
        reg.register_skills_schema();
        reg
    }

    pub fn register_ods_schema(&mut self) {
        let mut schema = SpecSchema::new(SpecKind::Ods, "2.0");

        // Universal + ODS 2.0 engine keys (all flat top-level — no ods: wrapper).
        for def in [
            KeyDefinition::new(
                "$schema",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Optional JSON Schema URI for editor validation",
            ),
            KeyDefinition::new(
                "description",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "One-line summary for indexes and previews",
            ),
            KeyDefinition::new(
                "tags",
                KeyPlacement::TopLevel,
                KeyType::List,
                false,
                "Free-form taxonomy tags",
            ),
            KeyDefinition::new(
                "owner",
                KeyPlacement::TopLevel,
                KeyType::StringOrList,
                false,
                "Responsible person or team",
            ),
            KeyDefinition::new(
                "author",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Individual author or agent",
            ),
            KeyDefinition::new(
                "created",
                KeyPlacement::TopLevel,
                KeyType::Timestamp,
                false,
                "Optional created timestamp",
            )
            .with_aliases(&["created_at", "date"]),
            KeyDefinition::new(
                "updated",
                KeyPlacement::TopLevel,
                KeyType::Timestamp,
                false,
                "Optional updated timestamp",
            )
            .with_aliases(&["last_updated", "updated_at"]),
            KeyDefinition::new(
                "title",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Display title (OKF signal or TITLE-001 sync with H1)",
            ),
            KeyDefinition::new(
                "name",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Alias for title",
            ),
            KeyDefinition::new(
                "profile",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Document profile type",
            ),
            KeyDefinition::new(
                "status",
                KeyPlacement::TopLevel,
                KeyType::Enum(vec![
                    "draft".into(),
                    "stable".into(),
                    "deprecated".into(),
                    "archived".into(),
                ]),
                false,
                "Document lifecycle status",
            ),
            KeyDefinition::new(
                "id",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Explicit document identifier",
            ),
            KeyDefinition::new(
                "share",
                KeyPlacement::TopLevel,
                KeyType::Enum(vec!["public".into(), "org".into(), "private".into()]),
                false,
                "Document visibility level",
            ),
            KeyDefinition::new(
                "depends",
                KeyPlacement::TopLevel,
                KeyType::List,
                false,
                "Required Markdown dependency paths",
            ),
            KeyDefinition::new(
                "related",
                KeyPlacement::TopLevel,
                KeyType::List,
                false,
                "Soft lateral links (string paths or 2.1 predicates)",
            ),
            KeyDefinition::new(
                "code",
                KeyPlacement::TopLevel,
                KeyType::List,
                false,
                "Source file path strings (no line numbers)",
            ),
            KeyDefinition::new(
                "resources",
                KeyPlacement::TopLevel,
                KeyType::List,
                false,
                "Resource file references or URLs",
            ),
            KeyDefinition::new(
                "load",
                KeyPlacement::TopLevel,
                KeyType::List,
                false,
                "Non-Markdown fixtures injected into AI context",
            ),
            // ODS 2.1 Pareto ontology (gated by workspace spec >= 2.1)
            KeyDefinition::new(
                "entity",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Pareto ontology entity name (ENT-001/002)",
            ),
            KeyDefinition::new(
                "domain",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Pareto ontology business domain",
            ),
            KeyDefinition::new(
                "schema",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Path to JSON Schema on disk (ONT-001)",
            ),
        ] {
            schema.add_key(def);
        }

        // Workspace config only (ods.toml)
        for def in [
            KeyDefinition::new(
                "spec",
                KeyPlacement::WorkspaceConfigOnly,
                KeyType::Enum(vec![
                    "2.0".into(),
                    "2.0.0".into(),
                    "2.1".into(),
                    "2.1.0".into(),
                ]),
                true,
                "ODS spec version marker",
            ),
            KeyDefinition::new(
                "ods",
                KeyPlacement::WorkspaceConfigOnly,
                KeyType::String,
                false,
                "Legacy alias for spec (ods.toml)",
            ),
            KeyDefinition::new(
                "dialect",
                KeyPlacement::WorkspaceConfigOnly,
                KeyType::Enum(vec!["standard".into(), "strict".into()]),
                false,
                "Workspace lint enforcement mode",
            ),
            KeyDefinition::new(
                "custom-profiles",
                KeyPlacement::WorkspaceConfigOnly,
                KeyType::List,
                false,
                "Workspace custom profile schema definitions",
            ),
            KeyDefinition::new(
                "profiles",
                KeyPlacement::WorkspaceConfigOnly,
                KeyType::List,
                false,
                "Legacy profile catalog roots",
            ),
            KeyDefinition::new(
                "packs",
                KeyPlacement::WorkspaceConfigOnly,
                KeyType::List,
                false,
                "Imported ODS packs",
            ),
            KeyDefinition::new(
                "ignore",
                KeyPlacement::WorkspaceConfigOnly,
                KeyType::List,
                false,
                "Ignore path prefixes",
            ),
            KeyDefinition::new(
                "context",
                KeyPlacement::WorkspaceConfigOnly,
                KeyType::Map,
                false,
                "Context traversal defaults ([context] table)",
            ),
            KeyDefinition::new(
                "ontology",
                KeyPlacement::WorkspaceConfigOnly,
                KeyType::Map,
                false,
                "Pareto ontology defaults ([ontology] table, 2.1+)",
            ),
            KeyDefinition::new(
                "aliases",
                KeyPlacement::WorkspaceConfigOnly,
                KeyType::Map,
                false,
                "Section and path aliases ([aliases] table)",
            ),
            KeyDefinition::new(
                "okf",
                KeyPlacement::WorkspaceConfigOnly,
                KeyType::Map,
                false,
                "OKF bundle settings ([okf] table)",
            ),
            KeyDefinition::new(
                "specs",
                KeyPlacement::WorkspaceConfigOnly,
                KeyType::Map,
                false,
                "Multi-spec activation and lint config",
            ),
            KeyDefinition::new(
                "service",
                KeyPlacement::WorkspaceConfigOnly,
                KeyType::Map,
                false,
                "Engine extension: ods serve memory budget",
            ),
        ] {
            schema.add_key(def);
        }

        self.schemas.insert("ods".into(), schema);
    }

    pub fn register_okf_schema(&mut self) {
        let mut schema = SpecSchema::new(SpecKind::Okf, "0.2");

        for def in [
            KeyDefinition::new(
                "okf_version",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "OKF bundle version",
            ),
            KeyDefinition::new(
                "type",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Concept kind (routes/filters)",
            ),
            KeyDefinition::new(
                "title",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Display title",
            ),
            KeyDefinition::new(
                "name",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Alias for title",
            ),
            KeyDefinition::new(
                "description",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Short summary",
            ),
            KeyDefinition::new(
                "tags",
                KeyPlacement::TopLevel,
                KeyType::List,
                false,
                "Free-form facets",
            ),
            KeyDefinition::new(
                "resource",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Canonical URI of underlying asset",
            ),
            KeyDefinition::new(
                "sources",
                KeyPlacement::TopLevel,
                KeyType::List,
                false,
                "Provenance sources",
            ),
            KeyDefinition::new(
                "usage_window",
                KeyPlacement::TopLevel,
                KeyType::Map,
                false,
                "Temporal applicability { from, to }",
            ),
            KeyDefinition::new(
                "generated",
                KeyPlacement::TopLevel,
                KeyType::Map,
                false,
                "Who/what produced the content",
            ),
            KeyDefinition::new(
                "verified",
                KeyPlacement::TopLevel,
                KeyType::List,
                false,
                "Independent confirmations",
            ),
            KeyDefinition::new(
                "status",
                KeyPlacement::TopLevel,
                KeyType::Enum(vec!["draft".into(), "stable".into(), "deprecated".into()]),
                false,
                "OKF lifecycle (top-level)",
            ),
            KeyDefinition::new(
                "stale_after",
                KeyPlacement::TopLevel,
                KeyType::Timestamp,
                false,
                "Absolute freshness deadline",
            ),
            KeyDefinition::new(
                "runtime",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Attested computation runtime",
            ),
            KeyDefinition::new(
                "parameters",
                KeyPlacement::TopLevel,
                KeyType::List,
                false,
                "Named typed parameters",
            ),
            KeyDefinition::new(
                "computation",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Path to computation file",
            ),
            KeyDefinition::new(
                "executor",
                KeyPlacement::TopLevel,
                KeyType::Map,
                false,
                "How to run + receipt shape",
            ),
            KeyDefinition::new(
                "attester",
                KeyPlacement::TopLevel,
                KeyType::Map,
                false,
                "Deterministic check of a run receipt",
            ),
        ] {
            schema.add_key(def);
        }

        self.schemas.insert("okf".into(), schema);
    }

    pub fn register_skills_schema(&mut self) {
        let mut schema = SpecSchema::new(SpecKind::Skills, "1.0");

        for def in [
            KeyDefinition::new(
                "name",
                KeyPlacement::TopLevel,
                KeyType::String,
                true,
                "Skill identity (must match parent directory)",
            ),
            KeyDefinition::new(
                "description",
                KeyPlacement::TopLevel,
                KeyType::String,
                true,
                "What the skill does and when to use it",
            ),
            KeyDefinition::new(
                "license",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "License name or path",
            ),
            KeyDefinition::new(
                "compatibility",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Environment requirements",
            ),
            KeyDefinition::new(
                "metadata",
                KeyPlacement::TopLevel,
                KeyType::Map,
                false,
                "Extra structured labels",
            ),
            KeyDefinition::new(
                "allowed-tools",
                KeyPlacement::TopLevel,
                KeyType::String,
                false,
                "Experimental pre-approved tools list",
            ),
        ] {
            schema.add_key(def);
        }

        self.schemas.insert("skills".into(), schema);
    }

    pub fn register_custom_profile(&mut self, name: &str, required_keys: &[String]) {
        let mut schema = SpecSchema::new(SpecKind::Custom(name.into()), "1.0");

        for key_name in required_keys {
            schema.add_key(KeyDefinition::new(
                key_name.clone(),
                KeyPlacement::TopLevel,
                KeyType::String,
                true,
                format!("Required custom profile domain key '{key_name}'"),
            ));
        }

        self.schemas.insert(name.into(), schema);
    }

    pub fn get(&self, name: &str) -> Option<&SpecSchema> {
        self.schemas.get(name)
    }

    pub fn dialect_names(&self) -> Vec<&str> {
        let mut names: Vec<&str> = self.schemas.keys().map(String::as_str).collect();
        names.sort_unstable();
        names
    }
}

/// Lightweight schema issue before conversion to workspace `Diagnostic`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaIssue {
    pub severity: Severity,
    pub message: String,
}

impl SchemaIssue {
    pub fn to_diagnostic(self, path: PathBuf) -> Diagnostic {
        Diagnostic {
            path,
            severity: self.severity,
            message: self.message,
        }
    }
}

/// Validate known ODS engine enums and placement flags against the registry.
///
/// Graph/profile/pack rules stay outside the schema layer.
pub fn validate_ods_frontmatter(frontmatter: &Frontmatter) -> Vec<SchemaIssue> {
    let registry = SpecSchemaRegistry::with_defaults();
    let Some(schema) = registry.get("ods") else {
        return Vec::new();
    };
    let mut issues = Vec::new();

    if let Some(status) = &frontmatter.status
        && let Some(def) = schema.keys.get("status")
        && let KeyType::Enum(allowed) = &def.key_type
        && !allowed.iter().any(|v| v == status)
    {
        let hint = status_alias_hint(status);
        issues.push(SchemaIssue {
            severity: Severity::Error,
            message: crate::error::lint_invalid_status(status, hint),
        });
    }

    if let Some(share) = &frontmatter.share
        && let Some(def) = schema.keys.get("share")
        && let KeyType::Enum(allowed) = &def.key_type
        && !allowed.iter().any(|v| v == share)
    {
        issues.push(SchemaIssue {
            severity: Severity::Error,
            message: crate::error::lint_invalid_share(share),
        });
    }

    // `title` is a first-class ODS 2.0 key; TITLE-001/002 in spec20_rules check it
    // against the H1 rather than discouraging its use.

    for key_name in frontmatter.custom_keys.keys() {
        if let Some(similar) = schema.find_similar_key(key_name) {
            if similar
                .aliases
                .iter()
                .any(|a| a.eq_ignore_ascii_case(key_name))
            {
                issues.push(SchemaIssue {
                    severity: Severity::Warning,
                    message: crate::error::lint_legacy_alias_used(key_name, &similar.name),
                });
            } else {
                issues.push(SchemaIssue {
                    severity: Severity::Warning,
                    message: crate::error::lint_key_typo_suggestion(key_name, &similar.name),
                });
            }
        }
    }

    // tags_misplaced is reported by tags::lint_document_tags (single message).
    issues
}

fn status_alias_hint(status: &str) -> Option<&'static str> {
    match status.to_ascii_lowercase().as_str() {
        "wip" | "in-progress" | "in_progress" | "todo" | "working" | "dev" => Some("draft"),
        "done" | "complete" | "completed" | "ready" | "ga" | "released" => Some("stable"),
        "old" | "obsolete" | "sunset" => Some("deprecated"),
        "archive" | "archived" | "inactive" => Some("archived"),
        _ => None,
    }
}

fn key_type_to_json_schema(key_type: &KeyType, description: &str) -> Value {
    let mut obj = match key_type {
        KeyType::String | KeyType::Timestamp => json!({ "type": "string" }),
        KeyType::List => json!({
            "type": "array",
            "items": { "type": "string" }
        }),
        KeyType::Map => json!({ "type": "object" }),
        KeyType::StringOrList => json!({
            "oneOf": [
                { "type": "string" },
                { "type": "array", "items": { "type": "string" } }
            ]
        }),
        KeyType::Enum(values) => json!({
            "type": "string",
            "enum": values
        }),
    };
    if let Some(map) = obj.as_object_mut() {
        map.insert("description".into(), Value::String(description.into()));
    }
    obj
}

/// Emit draft-07 JSON Schema for the ODS 2.0 dialect from the registry.
pub fn generate_ods_json_schema() -> String {
    let registry = SpecSchemaRegistry::with_defaults();
    let schema = registry
        .get("ods")
        .expect("ods schema registered in with_defaults");

    let mut props = serde_json::Map::new();
    for def in schema.keys.values() {
        if def.placement == KeyPlacement::TopLevel {
            props.insert(
                def.name.clone(),
                key_type_to_json_schema(&def.key_type, &def.description),
            );
        }
    }

    let root = json!({
        "$schema": "http://json-schema.org/draft-07/schema#",
        "$id": "https://opendocspec.org/schemas/v2.0/ods.schema.json",
        "title": "Open Document Spec (ODS) 2.0 Frontmatter Schema",
        "description": "Flat top-level frontmatter keys only; no ods: wrapper. Generated from SpecSchemaRegistry.",
        "type": "object",
        "properties": props,
        "additionalProperties": true
    });

    serde_json::to_string_pretty(&root).unwrap_or_else(|_| "{}".into())
}

/// Extract scalar string values for a frontmatter key from a document.
/// Known engine/universal fields take precedence; unknown keys use `custom_keys`
/// (keys stored lowercased at parse time).
pub fn get_document_key_values(doc: &crate::model::Document, key: &str) -> Vec<String> {
    use crate::model::FrontmatterState;
    let FrontmatterState::Parsed(fm) = &doc.frontmatter else {
        return Vec::new();
    };
    fm.key_query_values(key)
}

/// Evaluate if a document matches a single key clause (`key` or `key=val1,val2`).
/// Value match is **exact** (case-insensitive). Comma-separated values are OR.
pub fn evaluate_single_key_clause(doc: &crate::model::Document, clause: &str) -> bool {
    let trimmed = clause.trim();
    if trimmed.is_empty() {
        return true;
    }

    let (key, target_val) = match trimmed.split_once('=') {
        Some((k, v)) => (k.trim(), Some(v.trim())),
        None => (trimmed, None),
    };

    let doc_values = get_document_key_values(doc, key);

    match target_val {
        None => !doc_values.is_empty(),
        Some(target) => {
            let alternatives: Vec<String> = target
                .split(',')
                .map(|s| s.trim().to_lowercase())
                .filter(|s| !s.is_empty())
                .collect();

            if alternatives.is_empty() {
                return !doc_values.is_empty();
            }

            doc_values.iter().any(|dv| {
                let dv_lc = dv.to_lowercase();
                alternatives.contains(&dv_lc)
            })
        }
    }
}

/// Evaluate key query expression on a document (supporting AND/OR and comma multi-keys).
pub fn evaluate_document_key_query(doc: &crate::model::Document, expr: &str) -> bool {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return true;
    }

    if trimmed.contains(" OR ") || trimmed.contains(" or ") {
        let parts: Vec<&str> = if trimmed.contains(" OR ") {
            trimmed.split(" OR ").collect()
        } else {
            trimmed.split(" or ").collect()
        };
        return parts
            .iter()
            .any(|part| evaluate_document_key_query(doc, part));
    }

    if trimmed.contains(" AND ") || trimmed.contains(" and ") {
        let parts: Vec<&str> = if trimmed.contains(" AND ") {
            trimmed.split(" AND ").collect()
        } else {
            trimmed.split(" and ").collect()
        };
        return parts
            .iter()
            .all(|part| evaluate_document_key_query(doc, part));
    }

    if trimmed.contains(',') && trimmed.chars().filter(|c| *c == '=').count() > 1 {
        let parts: Vec<&str> = trimmed.split(',').collect();
        return parts
            .iter()
            .all(|part| evaluate_single_key_clause(doc, part));
    }

    evaluate_single_key_clause(doc, trimmed)
}

/// Filter workspace document IDs matching key expressions.
pub fn filter_documents_by_keys(
    workspace: &crate::model::Workspace,
    key_exprs: &[String],
    key_match_or: bool,
) -> Vec<String> {
    if key_exprs.is_empty() {
        return workspace.by_id.keys().cloned().collect();
    }

    workspace
        .documents
        .iter()
        .filter_map(|doc| {
            let matches = if key_match_or {
                key_exprs
                    .iter()
                    .any(|expr| evaluate_document_key_query(doc, expr))
            } else {
                key_exprs
                    .iter()
                    .all(|expr| evaluate_document_key_query(doc, expr))
            };
            if matches {
                let fm = match &doc.frontmatter {
                    crate::model::FrontmatterState::Parsed(fm) => Some(fm),
                    _ => None,
                };
                let id = crate::parse::document_id(&workspace.root, &doc.path, fm);
                Some(id)
            } else {
                None
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::Frontmatter;

    #[test]
    fn test_schema_registry_defaults() {
        let registry = SpecSchemaRegistry::with_defaults();
        let ods = registry.get("ods").expect("ods schema registered");
        assert!(ods.keys.contains_key("profile"));
        assert!(ods.keys.contains_key("custom-profiles"));
        assert!(ods.keys.contains_key("tags"));
        assert!(ods.keys.contains_key("description"));
        assert!(ods.keys.contains_key("specs"));

        let okf = registry.get("okf").expect("okf schema registered");
        assert!(okf.keys.contains_key("okf_version"));
        assert!(okf.keys.contains_key("type"));
        assert!(okf.keys.contains_key("sources"));

        let skills = registry.get("skills").expect("skills schema registered");
        assert!(skills.keys.contains_key("name"));
        assert!(skills.keys.contains_key("description"));
        assert!(skills.keys.contains_key("allowed-tools"));
    }

    #[test]
    fn test_custom_profile_registration() {
        let mut registry = SpecSchemaRegistry::with_defaults();
        registry
            .register_custom_profile("api_endpoint", &["endpoint_url".into(), "service".into()]);

        let custom = registry
            .get("api_endpoint")
            .expect("custom schema registered");
        assert!(custom.keys.contains_key("endpoint_url"));
        assert!(custom.keys.contains_key("service"));
    }

    #[test]
    fn validate_rejects_invalid_status_and_share() {
        let fm = Frontmatter {
            status: Some("nope".into()),
            share: Some("secret".into()),
            ..Default::default()
        };
        let issues = validate_ods_frontmatter(&fm);
        assert!(
            issues.iter().any(|i| i.message.contains("invalid status")),
            "{issues:?}"
        );
        assert!(
            issues.iter().any(|i| i.message.contains("invalid share")),
            "{issues:?}"
        );
    }

    #[test]
    fn generate_json_schema_contains_engine_and_universal_keys() {
        let raw = generate_ods_json_schema();
        let v: Value = serde_json::from_str(&raw).expect("valid json");
        let props = v
            .get("properties")
            .and_then(|p| p.as_object())
            .expect("properties");
        assert!(props.contains_key("tags"));
        assert!(props.contains_key("description"));
        assert!(props.contains_key("load"));
        assert!(props.contains_key("profile"));
        assert!(!props.contains_key("ods"));
        assert!(raw.contains("SpecSchemaRegistry") || raw.contains("Open Document Spec"));
    }

    #[test]
    fn ods_engine_keys_are_flat_top_level() {
        let registry = SpecSchemaRegistry::with_defaults();
        let ods = registry.get("ods").unwrap();
        for key in [
            "profile",
            "status",
            "id",
            "share",
            "depends",
            "related",
            "code",
            "resources",
            "load",
        ] {
            let def = ods.keys.get(key).unwrap_or_else(|| panic!("missing {key}"));
            assert_eq!(def.placement, KeyPlacement::TopLevel, "{key}");
        }
        let order = ods.canonical_engine_key_order();
        assert_eq!(
            order,
            vec![
                "profile",
                "status",
                "id",
                "share",
                "depends",
                "related",
                "resources",
                "code",
                "load",
            ]
        );
    }

    #[test]
    fn disable_strip_key_lists_are_schema_driven_and_exclude_ssg() {
        let registry = SpecSchemaRegistry::with_defaults();
        let ods = registry.get("ods").unwrap();
        let doc = ods.document_disable_strip_keys();
        assert!(doc.contains(&"ods".into()));
        assert!(doc.contains(&"profile".into()));
        assert!(doc.contains(&"tags".into()));
        assert!(!doc.iter().any(|k| k == "layout" || k == "hero_image"));
        let root = ods.workspace_policy_strip_keys();
        assert!(root.contains(&"ignore".into()) || root.contains(&"custom-profiles".into()));
        assert!(root.contains(&"okf_lint".into()));
        assert!(!root.iter().any(|k| k == "layout"));
    }

    #[test]
    fn test_evaluate_document_key_query() {
        use crate::model::CustomValue;
        use std::collections::BTreeMap;

        let mut custom = BTreeMap::new();
        custom.insert("team".into(), CustomValue::String("infra".into()));
        custom.insert(
            "tier".into(),
            CustomValue::List(vec!["p0".into(), "p1".into()]),
        );

        let doc = crate::model::Document {
            path: std::path::PathBuf::from("doc.md"),
            directory: std::path::PathBuf::from("."),
            body: String::new(),
            headings: Vec::new(),
            frontmatter: crate::model::FrontmatterState::Parsed(crate::model::Frontmatter {
                status: Some("draft".into()),
                owner: Some("alice".into()),
                custom_keys: custom,
                ..Default::default()
            }),
        };

        assert!(evaluate_document_key_query(&doc, "status=draft"));
        assert!(evaluate_document_key_query(&doc, "status=draft,stable"));
        assert!(!evaluate_document_key_query(&doc, "status=stable"));
        // Exact match only — substring must not match.
        assert!(!evaluate_document_key_query(&doc, "status=dra"));
        assert!(evaluate_document_key_query(&doc, "team=infra"));
        assert!(evaluate_document_key_query(&doc, "TEAM=INFRA"));
        assert!(evaluate_document_key_query(&doc, "tier=p0"));
        assert!(evaluate_document_key_query(
            &doc,
            "status=draft AND owner=alice"
        ));
        assert!(evaluate_document_key_query(
            &doc,
            "status=stable OR team=infra"
        ));
        assert!(evaluate_document_key_query(
            &doc,
            "status=draft,owner=alice"
        ));
        assert!(!evaluate_document_key_query(&doc, "missing"));
        assert!(evaluate_document_key_query(&doc, "team"));
        assert!(!evaluate_document_key_query(&doc, "status=draft-review"));
    }

    #[test]
    fn filter_documents_by_keys_and_or() {
        use crate::model::{CustomValue, Document, Frontmatter, FrontmatterState, Workspace};
        use std::collections::BTreeMap;
        use std::path::PathBuf;

        let mut custom_a = BTreeMap::new();
        custom_a.insert("team".into(), CustomValue::String("infra".into()));
        let doc_a = Document {
            path: PathBuf::from("a.md"),
            directory: PathBuf::from("."),
            body: String::new(),
            headings: Vec::new(),
            frontmatter: FrontmatterState::Parsed(Frontmatter {
                status: Some("draft".into()),
                owner: Some("alice".into()),
                custom_keys: custom_a,
                ..Default::default()
            }),
        };
        let doc_b = Document {
            path: PathBuf::from("b.md"),
            directory: PathBuf::from("."),
            body: String::new(),
            headings: Vec::new(),
            frontmatter: FrontmatterState::Parsed(Frontmatter {
                status: Some("stable".into()),
                owner: Some("bob".into()),
                ..Default::default()
            }),
        };
        let mut ws = Workspace::empty(PathBuf::from("."));
        ws.documents = vec![doc_a, doc_b];
        ws.by_id.insert("a".into(), 0);
        ws.by_id.insert("b".into(), 1);

        let and_ids =
            filter_documents_by_keys(&ws, &["status=draft".into(), "owner=alice".into()], false);
        assert_eq!(and_ids, vec!["a".to_string()]);

        let or_ids =
            filter_documents_by_keys(&ws, &["status=draft".into(), "status=stable".into()], true);
        assert_eq!(or_ids.len(), 2);

        let none = filter_documents_by_keys(&ws, &["status=archived".into()], false);
        assert!(none.is_empty());
    }
}
