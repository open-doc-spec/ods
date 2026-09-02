//! Central catalog of end-user error / usage / diagnostic messages.
//!
//! # Contract (first-call stderr)
//!
//! ```text
//! error: <what failed — one short line>
//! Next: <what to do — one copy-pasteable line>
//! ```
//!
//! Usage errors (exit 2) use `usage:` instead of `error:`.
//! Optional `Hint:` lines may follow when they prevent a second failure.
//!
//! Stable ids (`not_ods_workspace`, …) are for docs/tests; default human
//! output is code-free unless `ODS_ERROR_CODES=1`.

use std::env;
use std::fmt::Display;
use std::path::Path;

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// Lifecycle stage for catalog organization and docs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorStage {
    Argv,
    Dispatch,
    Scope,
    Load,
    Resolve,
    Mutate,
    Service,
    Report,
}

/// Structured user message before rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserMsg {
    /// Stable id for docs/tests (snake_case).
    pub id: &'static str,
    pub stage: ErrorStage,
    /// Short summary without `error:` / `usage:` prefix.
    pub summary: String,
    /// Imperative next action (without `Next:` prefix).
    pub next: Option<String>,
    /// Optional secondary hints (without `Hint:` prefix).
    pub hints: Vec<String>,
}

impl UserMsg {
    pub fn new(id: &'static str, stage: ErrorStage, summary: impl Into<String>) -> Self {
        Self {
            id,
            stage,
            summary: summary.into(),
            next: None,
            hints: Vec::new(),
        }
    }

    pub fn next(mut self, next: impl Into<String>) -> Self {
        self.next = Some(next.into());
        self
    }

    pub fn hint(mut self, hint: impl Into<String>) -> Self {
        self.hints.push(hint.into());
        self
    }

    /// Full stderr body for a failure (exit 1).
    pub fn render_error(&self) -> String {
        render_lines(
            "error",
            &self.summary,
            self.next.as_deref(),
            &self.hints,
            self.id,
        )
    }

    /// Full stderr body for a usage error (exit 2).
    pub fn render_usage(&self) -> String {
        render_lines(
            "usage",
            &self.summary,
            self.next.as_deref(),
            &self.hints,
            self.id,
        )
    }
}

fn error_codes_enabled() -> bool {
    matches!(
        env::var("ODS_ERROR_CODES").as_deref(),
        Ok("1") | Ok("true") | Ok("yes")
    )
}

fn render_lines(
    kind: &str,
    summary: &str,
    next: Option<&str>,
    hints: &[String],
    id: &str,
) -> String {
    let mut out = format!("{kind}: {summary}");
    if error_codes_enabled() {
        out.push_str(&format!(" [{id}]"));
    }
    if let Some(n) = next {
        out.push_str("\nNext: ");
        out.push_str(n);
    }
    for h in hints {
        out.push_str("\nHint: ");
        out.push_str(h);
    }
    out
}

/// Convenience: render a failure from summary + next.
pub fn render_error(summary: impl AsRef<str>, next: Option<&str>) -> String {
    let mut msg = UserMsg::new("generic", ErrorStage::Report, summary.as_ref());
    if let Some(n) = next {
        msg = msg.next(n);
    }
    msg.render_error()
}

/// Convenience: render a usage line from summary + next.
pub fn render_usage(summary: impl AsRef<str>, next: Option<&str>) -> String {
    let mut msg = UserMsg::new("generic_usage", ErrorStage::Argv, summary.as_ref());
    if let Some(n) = next {
        msg = msg.next(n);
    }
    msg.render_usage()
}

// ===========================================================================
// Dispatch / argv
// ===========================================================================

pub fn unknown_command(cmd: &str, suggestion: Option<&str>) -> UserMsg {
    let mut msg = UserMsg::new(
        "unknown_command",
        ErrorStage::Dispatch,
        format!("unknown command '{cmd}'"),
    )
    .next("run `ods help` for the command list");
    if let Some(s) = suggestion {
        msg = msg.hint(format!("did you mean `{s}`?"));
    }
    msg
}

pub fn unknown_platform_command(cmd: &str) -> UserMsg {
    UserMsg::new(
        "unknown_platform_command",
        ErrorStage::Dispatch,
        format!("unknown platform command '{cmd}'"),
    )
    .next("run `ods help` for the command list")
}

pub fn unknown_ods_command(cmd: &str, suggestion: Option<&str>) -> UserMsg {
    unknown_command(cmd, suggestion)
}

pub fn okf_namespace_removed() -> UserMsg {
    UserMsg::new(
        "okf_namespace_removed",
        ErrorStage::Dispatch,
        "unknown command 'okf'",
    )
    .next("use the `--okf` flag (e.g. `ods lint --okf`, `ods init --okf`)")
    .hint("there is no `ods okf` namespace")
}

pub fn forbidden_ods_flag() -> UserMsg {
    UserMsg::new(
        "forbidden_ods_flag",
        ErrorStage::Argv,
        "unknown flag: --ods",
    )
    .next("ODS is the default — use bare `ods <cmd>` (extra specs: `--okf` / `--skills`)")
}

pub fn missing_flag_value(flag: &str, example: &str) -> UserMsg {
    UserMsg::new(
        "missing_flag_value",
        ErrorStage::Argv,
        format!("missing value for {flag}"),
    )
    .next(format!("pass a value, e.g. {example}"))
}

pub fn unknown_flag(flag: &str, help_cmd: &str) -> UserMsg {
    UserMsg::new(
        "unknown_flag",
        ErrorStage::Argv,
        format!("unknown flag: {flag}"),
    )
    .next(format!("run `{help_cmd}` for supported flags"))
}

pub fn unknown_subcommand(cmd: &str, sub: &str, help: &str) -> UserMsg {
    UserMsg::new(
        "unknown_subcommand",
        ErrorStage::Argv,
        format!("unknown {cmd} subcommand: {sub}"),
    )
    .next(format!("run `{help}`"))
}

pub fn missing_context_id() -> UserMsg {
    UserMsg::new(
        "missing_context_id",
        ErrorStage::Argv,
        "missing document id",
    )
    .next("run `ods context <id-or-path>` (discover ids with `ods find <query>` or `--tag` / `--key` when unique)")
}

/// Context filter fallback matched more than one document.
pub fn context_filter_ambiguous(count: usize, sample_ids: &[String]) -> UserMsg {
    let preview = if sample_ids.len() > 8 {
        format!(
            "{}… (+{} more)",
            sample_ids[..8].join(", "),
            sample_ids.len() - 8
        )
    } else {
        sample_ids.join(", ")
    };
    UserMsg::new(
        "context_filter_ambiguous",
        ErrorStage::Resolve,
        format!("context filter matched {count} documents; need a unique target"),
    )
    .next(format!(
        "narrow with `ods find --tag` / `--key`, or pass an id: {preview}"
    ))
}

pub fn missing_required_arg(what: &str, usage_line: &str) -> UserMsg {
    UserMsg::new(
        "missing_required_arg",
        ErrorStage::Argv,
        format!("missing {what}"),
    )
    .next(format!("usage: {usage_line}"))
}

// ===========================================================================
// Scope (multi-spec)
// ===========================================================================

pub fn not_ods_workspace(hint_okf: bool, hint_skills: bool) -> UserMsg {
    let mut msg = UserMsg::new(
        "not_ods_workspace",
        ErrorStage::Scope,
        "not an ODS workspace (no root ods.toml with spec)",
    )
    .next("run `ods init` here to create an ODS workspace");
    if hint_okf {
        msg = msg.hint("OKF markers found — pass `--okf` (e.g. `ods lint --okf`)");
    }
    if hint_skills {
        msg = msg.hint("SKILL.md found — pass `--skills` (e.g. `ods lint --skills`)");
    }
    msg
}

pub fn not_okf_bundle() -> UserMsg {
    UserMsg::new(
        "not_okf_bundle",
        ErrorStage::Scope,
        "not an OKF bundle (no root index with okf_version)",
    )
    .next("run `ods init --okf` then retry with `--okf`")
}

pub fn no_skills_package() -> UserMsg {
    UserMsg::new(
        "no_skills_package",
        ErrorStage::Scope,
        "no Agent Skills package found (expected SKILL.md)",
    )
    .next("run `ods init --skills` then retry with `--skills`")
}

// ===========================================================================
// Load
// ===========================================================================

pub fn load_workspace_failed(root: impl AsRef<Path>, err: impl Display) -> UserMsg {
    let root = root.as_ref().display();
    UserMsg::new(
        "load_workspace_failed",
        ErrorStage::Load,
        format!("could not load workspace at '{root}': {err}"),
    )
    .next("check the path exists, or run `ods init` to create a workspace")
}

pub fn load_okf_bundle_failed(root: impl AsRef<Path>, err: impl Display) -> UserMsg {
    let root = root.as_ref().display();
    UserMsg::new(
        "load_okf_bundle_failed",
        ErrorStage::Load,
        format!("could not load OKF bundle at '{root}': {err}"),
    )
    .next("run `ods init --okf` or check the path")
}

pub fn root_index_missing() -> UserMsg {
    UserMsg::new(
        "root_index_missing",
        ErrorStage::Load,
        "missing ods.toml workspace marker",
    )
    .next("run `ods init` then retry")
}

pub fn path_not_found(path: impl AsRef<Path>) -> UserMsg {
    let p = path.as_ref().display();
    UserMsg::new(
        "path_not_found",
        ErrorStage::Load,
        format!("path not found: {p}"),
    )
    .next("check the path spelling or create the file/directory")
}

pub fn io_failed(action: &str, err: impl Display) -> UserMsg {
    UserMsg::new("io_failed", ErrorStage::Load, format!("{action}: {err}"))
        .next("check permissions and that the path is writable")
}

pub fn home_dir_unresolved() -> UserMsg {
    UserMsg::new(
        "home_dir_unresolved",
        ErrorStage::Load,
        "could not resolve home directory",
    )
    .next("set HOME (or USERPROFILE on Windows) and retry")
}

// ===========================================================================
// Resolve
// ===========================================================================

pub fn document_not_found_context(query: &str) -> UserMsg {
    UserMsg::new(
        "document_not_found_context",
        ErrorStage::Resolve,
        format!("no document matched '{query}' in this workspace"),
    )
    .next(format!(
        "run `ods find {query}` or pass a path-shaped id (e.g. specs/ods/core)"
    ))
}

pub fn document_not_found(target: &str) -> UserMsg {
    UserMsg::new(
        "document_not_found",
        ErrorStage::Resolve,
        format!("document not found: {target}"),
    )
    .next("run `ods find <query>` or pass a path relative to the workspace root")
}

pub fn concept_not_found(id: &str) -> UserMsg {
    UserMsg::new(
        "concept_not_found",
        ErrorStage::Resolve,
        format!("OKF concept not found: {id}"),
    )
    .next("check the concept id in the bundle, or run `ods lint --okf`")
}

pub fn context_requires_ods_or_okf() -> UserMsg {
    UserMsg::new(
        "context_requires_ods_or_okf",
        ErrorStage::Scope,
        "context requires an ODS workspace",
    )
    .next("run `ods init`, or pass `--okf` for OKF concept context")
}

// ===========================================================================
// Mutate
// ===========================================================================

pub fn undo_no_snapshot() -> UserMsg {
    UserMsg::new(
        "undo_no_snapshot",
        ErrorStage::Mutate,
        "nothing to undo (no snapshot found)",
    )
    .next("snapshots are created by bulk writes (e.g. adopt --write, fmt); nothing to restore")
}

pub fn already_exists(path: impl AsRef<Path>) -> UserMsg {
    let p = path.as_ref().display();
    UserMsg::new(
        "already_exists",
        ErrorStage::Mutate,
        format!("already exists: {p}"),
    )
    .next("choose a different path, or remove the existing file first")
}

pub fn scaffold_failed(err: impl Display) -> UserMsg {
    UserMsg::new(
        "scaffold_failed",
        ErrorStage::Mutate,
        format!("failed to scaffold document: {err}"),
    )
    .next("check the path is writable and the parent directory exists")
}

// ===========================================================================
// Lint / report diagnostics (short; guide holds long explanations)
// ===========================================================================

pub fn lint_invalid_status(status: &str, hint: Option<&str>) -> String {
    if let Some(h) = hint {
        format!(
            "invalid status: {status} (did you mean `{h}`? allowed: draft|stable|deprecated|archived)"
        )
    } else {
        format!("invalid status: {status} (allowed: draft|stable|deprecated|archived)")
    }
}

pub fn lint_invalid_share(share: &str) -> String {
    format!("invalid share value: {share} (allowed: public|org|private)")
}

pub fn lint_title_discouraged() -> String {
    "frontmatter `title:` is discouraged for ODS docs — use the first `# H1` as the document title (value is preserved)".into()
}

pub fn lint_ods_wrapper_rejected() -> String {
    "TITLE-000: `ods:` wrapper is removed in ODS 2.0 — use flat top-level keys; run: ods fmt --migrate".into()
}

pub fn lint_legacy_key_rejected(key: &str) -> String {
    format!("legacy key `{key}` is removed in ODS 2.0 — run: ods doctor")
}

pub fn lint_title_h1_mismatch(title: &str, h1: &str) -> String {
    format!("TITLE-001: frontmatter title/name `{title}` does not match H1 `{h1}`")
}

pub fn lint_title_missing_h1() -> String {
    "TITLE-002: frontmatter title/name present but document has no `# H1` heading".into()
}

pub fn lint_missing_load_path(path: impl Display) -> String {
    format!("ASSET-004: load path does not exist: {path}")
}

pub fn lint_ontology_key_on_20(key: &str) -> String {
    format!("`{key}` requires spec >= 2.1 (or @ods/pack-pareto-ontology)")
}

pub fn lint_unknown_related_predicate(predicate: &str) -> String {
    format!("ENUM-006: unknown related predicate `{predicate}`")
}

pub fn lint_entity_not_found(entity: &str) -> String {
    format!("ENT-001: entity `{entity}` has no definition document")
}

pub fn lint_duplicate_entity(entity: &str) -> String {
    format!("ENT-002: duplicate entity name `{entity}`")
}

pub fn lint_ontology_schema_missing(path: impl Display) -> String {
    format!("ONT-001: schema path does not exist: {path}")
}

pub fn lint_code_object_form_rejected() -> String {
    "CODE-002: code entries must be plain string paths in ODS 2.0".into()
}

pub fn lint_dangling_reference(reference: &str) -> String {
    format!("dangling reference: {reference}")
}

pub fn lint_dangling_context_reference(load: &str) -> String {
    format!("dangling context reference: {load}")
}

pub fn lint_depends_cycle(cycle: &str) -> String {
    format!("depends cycle detected: {cycle}")
}

pub fn lint_duplicate_document_id(id: &str) -> String {
    format!("duplicate document id: {id}")
}

pub fn lint_missing_resource(path: impl Display) -> String {
    format!("missing resource: {path}")
}

pub fn lint_missing_code_path(path: impl Display) -> String {
    format!("missing code path: {path}")
}

pub fn lint_frontmatter_parse(message: &str) -> String {
    format!("frontmatter parse error: {message}")
}

pub fn lint_unknown_profile(profile: &str) -> String {
    format!("unknown profile: {profile}")
}

pub fn lint_unknown_profile_with_sources(profile: &str, configured_paths: &[String]) -> String {
    let paths = if configured_paths.is_empty() {
        "(none)".to_string()
    } else {
        configured_paths.join(", ")
    };
    format!(
        "profile not found: {profile} (custom profile definitions are loaded only from paths declared by custom_profiles in ods.toml: {paths})"
    )
}

pub fn lint_missing_expected_section(section: &str) -> String {
    format!("missing expected section: {section}")
}

pub fn lint_tags_misplaced() -> String {
    "tags must be top-level frontmatter (not under ods:) so other tools can read them; run: ods fmt --migrate".into()
}

pub fn lint_index_stale_missing(missing: &str) -> String {
    format!("index missing children: {missing}")
}

pub fn lint_index_stale_extra(extra: &str) -> String {
    format!("index has extra entries: {extra}")
}

pub fn skills_no_package() -> String {
    "[skills] no SKILL.md package found (root or skills/*/)".into()
}

pub fn skills_body_too_long(body_lines: usize) -> String {
    format!(
        "[skills] SKILL.md body has {body_lines} lines; recommend under 500 (progressive disclosure)"
    )
}

pub fn skills_prefix(message: impl Into<String>) -> String {
    format!("[skills] {}", message.into())
}

pub fn skills_missing_name() -> String {
    skills_prefix("missing required frontmatter field: name")
}

pub fn skills_name_too_long(len: usize) -> String {
    skills_prefix(format!("name must be at most 64 characters (got {len})"))
}

pub fn skills_name_invalid() -> String {
    skills_prefix(
        "name must be lowercase alphanumeric and hyphens only, must not start/end with hyphen, and must not contain consecutive hyphens",
    )
}

pub fn skills_name_dir_mismatch(name: &str, dir: &str) -> String {
    skills_prefix(format!(
        "name `{name}` must match parent directory name `{dir}`"
    ))
}

pub fn skills_missing_description() -> String {
    skills_prefix("missing required frontmatter field: description")
}

pub fn skills_description_too_long(len: usize) -> String {
    skills_prefix(format!(
        "description must be at most 1024 characters (got {len})"
    ))
}

pub fn skills_compatibility_too_long(len: usize) -> String {
    skills_prefix(format!(
        "compatibility must be at most 500 characters (got {len})"
    ))
}

// --- OKF diagnostics ---

pub fn okf_version_mismatch(other: &str) -> String {
    format!("okf_version is {other:?}; engine targets 0.2")
}

pub fn okf_missing_version() -> String {
    "root index.md missing okf_version: \"0.2\"".into()
}

pub fn okf_missing_frontmatter() -> String {
    "OKF concept missing YAML frontmatter".into()
}

pub fn okf_invalid_frontmatter(err: &str) -> String {
    format!("invalid OKF frontmatter: {err}")
}

pub fn okf_missing_type() -> String {
    "missing required frontmatter field: type".into()
}

pub fn okf_attested_requires_runtime() -> String {
    "Attested Computation requires runtime".into()
}

pub fn okf_generated_by_required() -> String {
    "generated.by is required when generated is present".into()
}

pub fn okf_verified_by_required(idx: usize) -> String {
    format!("verified[{idx}].by is required")
}

pub fn okf_sources_resource_required(idx: usize) -> String {
    format!("sources[{idx}].resource is required within a sources entry")
}

pub fn okf_stale_after_format(date: &str) -> String {
    format!("stale_after should be YYYY-MM-DD, got {date:?}")
}

pub fn okf_concept_stale(date: &str) -> String {
    format!("concept is stale (stale_after: {date})")
}

/// Self-update / install failure (CLI boundary wraps update module String errors).
pub fn update_failed(detail: impl Display) -> UserMsg {
    UserMsg::new(
        "update_failed",
        ErrorStage::Service,
        format!("update failed: {detail}"),
    )
    .next(
        "check network access to GitHub releases, or install from https://github.com/open-doc-spec/ods/releases",
    )
}

/// Wrap a service/OS operation failure with a Next line.
pub fn service_failed(action: &str, err: impl Display) -> UserMsg {
    UserMsg::new(
        "service_failed",
        ErrorStage::Service,
        format!("{action}: {err}"),
    )
    .next("check permissions, or run `ods start --status` / see guide 07 (daemon troubleshooting)")
}

// --- Lifecycle / engine io::Error payloads (also used as update detail lines) ---

pub fn lifecycle_document_exists(path: impl Display) -> String {
    format!("document already exists: {path}")
}

pub fn lifecycle_document_not_found(path: impl Display) -> String {
    format!("document not found: {path}")
}

pub fn lifecycle_refuse_body_change(path: impl Display) -> String {
    format!("refuse to change body of {path}")
}

/// Short action:detail line for internal `Result<_, String>` layers (update/install).
pub fn detail(action: &str, err: impl Display) -> String {
    format!("{action}: {err}")
}

pub fn update_unsupported_platform(os: &str, arch: &str) -> String {
    format!("unsupported platform {os}/{arch}; supported: Linux/macOS/Windows x86_64 and arm64")
}

pub fn update_asset_not_found(filename: &str, tag: &str) -> String {
    format!("asset {filename} not found on release {tag}")
}

pub fn update_checksums_not_found(tag: &str) -> String {
    format!("SHA256SUMS not found on release {tag}")
}

pub fn update_checksum_entry_missing(filename: &str) -> String {
    format!("no SHA256 entry for {filename} in release checksums")
}

pub fn update_checksum_mismatch(filename: &str, expected: &str, actual: &str) -> String {
    format!("checksum mismatch for {filename}\n  expected: {expected}\n  got:      {actual}")
}

pub fn update_archive_missing_binary(root: impl Display) -> String {
    format!("archive missing ods/ods under {root}")
}

/// Stable message ids used by docs/tests (snake_case). Keep in sync when adding builders.
pub const CATALOG_MESSAGE_IDS: &[&str] = &[
    "unknown_command",
    "unknown_platform_command",
    "okf_namespace_removed",
    "forbidden_ods_flag",
    "missing_flag_value",
    "unknown_flag",
    "unknown_subcommand",
    "missing_context_id",
    "missing_required_arg",
    "not_ods_workspace",
    "not_okf_bundle",
    "no_skills_package",
    "load_workspace_failed",
    "load_okf_bundle_failed",
    "root_index_missing",
    "path_not_found",
    "io_failed",
    "home_dir_unresolved",
    "document_not_found_context",
    "document_not_found",
    "context_filter_ambiguous",
    "concept_not_found",
    "context_requires_ods_or_okf",
    "undo_no_snapshot",
    "already_exists",
    "scaffold_failed",
    "invalid_choice",
    "update_failed",
    "service_failed",
    "git_unavailable",
];

pub fn lint_code_path_line_suffix(path: impl Display) -> String {
    format!("code path must not contain line number suffix: {path}")
}

pub fn lint_dangling_body_link(link: &str) -> String {
    format!("dangling markdown link in body: {link}")
}

pub fn lint_missing_pack_path(pack: &str) -> String {
    format!("missing pack path: {pack}")
}

pub fn lint_missing_context_resource(load: &str) -> String {
    format!("missing context resource: {load}")
}

pub fn lint_context_ignore_not_found(ignore: &str) -> String {
    format!("context ignore target not found: {ignore}")
}

pub fn lint_root_ods_scope_only() -> String {
    "workspace policy keys (spec, ignore, packs, specs) belong in root ods.toml, not document frontmatter".into()
}

pub fn lint_invalid_date(field: &str, value: &str) -> String {
    format!("invalid {field} date format: '{value}' (expected YYYY-MM-DD or ISO-8601)")
}

pub fn lint_missing_required_key(key: &str, profile: &str) -> String {
    format!("missing required key '{key}' for profile '{profile}'")
}

pub fn lint_forbidden_profile_key(key: &str, profile: &str) -> String {
    format!("forbidden key '{key}' is present for profile '{profile}'")
}

pub fn lint_duplicate_tag(tag: &str) -> String {
    format!("duplicate tag: {tag}")
}

pub fn lint_tag_has_spaces(tag: &str, suggested: &str) -> String {
    format!("tag has spaces: {tag} (prefer {suggested})")
}

pub fn lint_tag_collides_status(tag: &str) -> String {
    format!("tag collides with status value: {tag} (use status: field)")
}

pub fn lint_tag_collides_profile(tag: &str) -> String {
    format!("tag collides with profile name: {tag} (use profile: field)")
}

/// Invalid enumerated CLI value (mode, format, shell, …).
pub fn invalid_choice(flag: &str, value: &str, allowed: &str) -> UserMsg {
    UserMsg::new(
        "invalid_choice",
        ErrorStage::Argv,
        format!("invalid {flag} '{value}'"),
    )
    .next(format!("use one of: {allowed}"))
}

pub fn lint_root_version_mismatch(version: &str, expected: &str) -> String {
    format!("root ods spec version mismatch: {version} (expected {expected})")
}

pub fn lint_root_missing_ods_version(expected: &str) -> String {
    format!("ods.toml missing spec = \"{expected}\"")
}

pub fn lint_missing_root_index(expected: &str) -> String {
    format!("missing ods.toml with spec = \"{expected}\"")
}

pub fn lint_missing_ods_toml(expected: &str) -> String {
    format!("missing ods.toml with spec = \"{expected}\"")
}

pub fn lint_non_canonical_ref(reference: &str, canonical: &str) -> String {
    format!("non-canonical document reference: {reference} (prefer {canonical})")
}

pub fn lint_non_canonical_context_ref(load: &str, canonical: &str) -> String {
    format!("non-canonical context document reference: {load} (prefer {canonical})")
}

pub fn lint_duplicate_profile(name: &str, kept: impl Display, ignored: impl Display) -> String {
    format!("duplicate profile definition: {name} (kept {kept}, ignored {ignored})")
}

pub fn lint_key_typo_suggestion(typo: &str, suggestion: &str) -> String {
    format!("unknown frontmatter key '{typo}' (did you mean '{suggestion}'?)")
}

pub fn lint_legacy_alias_used(alias: &str, canonical: &str) -> String {
    format!("legacy key alias '{alias}' used (canonical key is '{canonical}')")
}

// ===========================================================================
// Command suggestion helper
// ===========================================================================

/// Known top-level commands for did-you-mean.
pub const KNOWN_COMMANDS: &[&str] = &[
    "help",
    "version",
    "update",
    "upgrade",
    "setup",
    "workspaces",
    "skill",
    "pack",
    "stats",
    "overview",
    "summary",
    "completion",
    "schema",
    "tree",
    "diff",
    "clean",
    "lsp",
    "lint",
    "index",
    "profile",
    "profiles",
    "tags",
    "find",
    "tag",
    "context",
    "graph",
    "mv",
    "fmt",
    "adopt",
    "new",
    "rm",
    "remove",
    "status",
    "archive",
    "init",
    "enable",
    "disable",
    "revert",
    "doctor",
    "sync",
    "logs",
    "watch",
    "serve",
    "export",
    "start",
    "stop",
    "share",
    "bench",
    "sandbox",
    "audit",
    "coverage",
    "undo",
    "agents",
];

/// Suggest closest known command (simple edit distance; max distance 2).
pub fn suggest_command(input: &str) -> Option<&'static str> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    let mut best: Option<(&'static str, usize)> = None;
    for &cmd in KNOWN_COMMANDS {
        let d = edit_distance(input, cmd);
        if d == 0 {
            return Some(cmd);
        }
        if d <= 2 {
            match best {
                Some((_, bd)) if d >= bd => {}
                _ => best = Some((cmd, d)),
            }
        }
    }
    best.map(|(c, _)| c)
}

pub fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let (m, n) = (a.len(), b.len());
    if m == 0 {
        return n;
    }
    if n == 0 {
        return m;
    }
    // Banded for short CLI names
    let mut prev: Vec<usize> = (0..=n).collect();
    let mut cur = vec![0; n + 1];
    for i in 1..=m {
        cur[0] = i;
        for j in 1..=n {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[n]
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_error_shape() {
        let s = not_ods_workspace(false, false).render_error();
        assert!(s.starts_with("error: not an ODS workspace"), "{s}");
        assert!(s.contains("\nNext: run `ods init`"), "{s}");
        assert!(!s.contains("To fix:"), "{s}");
    }

    #[test]
    fn render_usage_shape() {
        let s = missing_context_id().render_usage();
        assert!(s.starts_with("usage: missing document id"), "{s}");
        assert!(s.contains("\nNext:"), "{s}");
    }

    #[test]
    fn not_ods_workspace_hints() {
        let s = not_ods_workspace(true, true).render_error();
        assert!(s.contains("Hint:"), "{s}");
        assert!(s.contains("--okf"), "{s}");
        assert!(s.contains("--skills"), "{s}");
    }

    #[test]
    fn forbidden_ods_flag_message() {
        let s = forbidden_ods_flag().render_error();
        assert!(s.contains("--ods"), "{s}");
        assert!(s.contains("Next:"), "{s}");
    }

    #[test]
    fn unknown_command_suggestion() {
        let s = unknown_command("lintt", suggest_command("lintt")).render_error();
        assert!(s.contains("unknown command 'lintt'"), "{s}");
        assert!(s.contains("did you mean `lint`?"), "{s}");
    }

    #[test]
    fn context_not_found_directive() {
        let s = document_not_found_context("oauth").render_error();
        assert!(s.contains("oauth"), "{s}");
        assert!(s.contains("ods find"), "{s}");
    }

    #[test]
    fn context_filter_ambiguous_lists_ids() {
        let ids: Vec<String> = (0..10).map(|i| format!("doc{i}")).collect();
        let s = context_filter_ambiguous(ids.len(), &ids).render_error();
        assert!(s.contains("matched 10"), "{s}");
        assert!(s.contains("doc0"), "{s}");
        assert!(s.contains("ods find") || s.contains("Next:"), "{s}");
        let small = context_filter_ambiguous(2, &["a".into(), "b".into()]).render_error();
        assert!(small.contains("a") && small.contains("b"), "{small}");
    }

    #[test]
    fn suggest_command_close() {
        assert_eq!(suggest_command("lint"), Some("lint"));
        assert_eq!(suggest_command("lintt"), Some("lint"));
        assert_eq!(suggest_command("zzzzzzz"), None);
    }

    #[test]
    fn load_workspace_failed_has_next() {
        let s = load_workspace_failed("/tmp/x", "No such file").render_error();
        assert!(s.contains("could not load workspace"), "{s}");
        assert!(s.contains("ods init"), "{s}");
    }

    #[test]
    fn root_marker_and_scope_messages_point_at_ods_toml() {
        let s = root_index_missing().render_error();
        assert!(s.contains("ods.toml"), "{s}");
        assert!(!s.contains("index.ods.md"), "{s}");
        assert!(s.contains("ods init"), "{s}");
        assert!(lint_root_ods_scope_only().contains("ods.toml"));
        assert!(!lint_root_ods_scope_only().contains("index.ods.md"));
    }

    #[test]
    fn catalog_builders_render_nonempty() {
        // Smoke-call lint/user builders so catalog stays covered as messages evolve.
        let user_msgs = [
            unknown_command("x", None).render_error(),
            unknown_command("lintt", Some("lint")).render_error(),
            unknown_platform_command("foo").render_error(),
            unknown_ods_command("bar", None).render_error(),
            okf_namespace_removed().render_error(),
            forbidden_ods_flag().render_error(),
            missing_flag_value("--out", "ods export --out x").render_error(),
            unknown_flag("--nope", "ods help").render_error(),
            unknown_subcommand("pack", "zzz", "ods pack list").render_error(),
            missing_context_id().render_error(),
            missing_required_arg("path", "ods new <path>").render_error(),
            not_ods_workspace(false, false).render_error(),
            not_ods_workspace(true, true).render_error(),
            not_okf_bundle().render_error(),
            no_skills_package().render_error(),
            path_not_found("/nope").render_error(),
            io_failed("write", "disk full").render_error(),
            home_dir_unresolved().render_error(),
            document_not_found_context("q").render_error(),
            document_not_found("doc").render_error(),
            concept_not_found("c").render_error(),
            context_requires_ods_or_okf().render_error(),
            undo_no_snapshot().render_error(),
            already_exists("/x").render_error(),
            scaffold_failed("boom").render_error(),
            invalid_choice("--format", "xml", "text|json").render_error(),
        ];
        for s in user_msgs {
            assert!(!s.is_empty(), "{s}");
            assert!(s.contains("error:") || s.contains("usage:"), "{s}");
        }

        let strings = [
            lint_invalid_status("nope", None),
            lint_invalid_status("drft", Some("draft")),
            lint_invalid_share("public"),
            lint_title_discouraged(),
            lint_dangling_reference("x"),
            lint_dangling_context_reference("y"),
            lint_depends_cycle("a -> b -> a"),
            lint_duplicate_document_id("id"),
            lint_missing_resource("a.csv"),
            lint_missing_code_path("src/a.rs"),
            lint_frontmatter_parse("bad"),
            lint_unknown_profile("zzz"),
            lint_missing_expected_section("Goal"),
            lint_tags_misplaced(),
            lint_index_stale_missing("a.md"),
            lint_index_stale_extra("b.md"),
            skills_no_package(),
            skills_body_too_long(999),
            skills_prefix("x"),
            skills_missing_name(),
            skills_name_too_long(99),
            skills_name_invalid(),
            skills_name_dir_mismatch("a", "b"),
            skills_missing_description(),
            skills_description_too_long(999),
            skills_compatibility_too_long(999),
            okf_version_mismatch("0.1"),
            okf_missing_version(),
            okf_missing_frontmatter(),
            okf_invalid_frontmatter("e"),
            okf_missing_type(),
            okf_attested_requires_runtime(),
            okf_generated_by_required(),
            okf_verified_by_required(1),
            okf_sources_resource_required(0),
            okf_stale_after_format("x"),
            okf_concept_stale("2020-01-01"),
            lifecycle_document_exists("a.md"),
            lifecycle_document_not_found("a.md"),
            lifecycle_refuse_body_change("a.md"),
            detail("act", "err"),
            update_unsupported_platform("haiku", "riscv"),
            update_asset_not_found("ods.tgz", "v1"),
            update_checksums_not_found("v1"),
            update_checksum_entry_missing("ods.tgz"),
            update_checksum_mismatch("ods.tgz", "aa", "bb"),
            update_archive_missing_binary("/tmp"),
            lint_code_path_line_suffix("x.rs"),
            lint_dangling_body_link("a.md"),
            lint_missing_pack_path("p"),
            lint_missing_context_resource("c"),
            lint_context_ignore_not_found("i"),
            lint_invalid_date("created", "nope"),
            lint_missing_required_key("k", "note"),
            lint_forbidden_profile_key("k", "note"),
            lint_duplicate_tag("t"),
            lint_tag_has_spaces("a b", "a-b"),
            lint_tag_collides_status("draft"),
            lint_tag_collides_profile("note"),
            lint_root_version_mismatch("0.0", "0.1"),
            lint_root_missing_ods_version("0.1"),
            lint_missing_root_index("0.1"),
            lint_missing_ods_toml("0.1"),
            lint_non_canonical_ref("a", "a.md"),
            lint_non_canonical_context_ref("a", "a.md"),
            lint_duplicate_profile("p", "a.md", "b.md"),
            lint_key_typo_suggestion("stauts", "status"),
            lint_legacy_alias_used("created_at", "created"),
        ];
        for s in strings {
            assert!(!s.is_empty(), "{s}");
        }
    }

    #[test]
    fn update_and_service_failed_shape() {
        let u = update_failed("network down").render_error();
        assert!(u.starts_with("error: update failed:"), "{u}");
        assert!(u.contains("Next:"), "{u}");
        let s = service_failed("start service", "permission denied").render_error();
        assert!(s.contains("start service"), "{s}");
        assert!(s.contains("Next:"), "{s}");
    }

    #[test]
    fn okf_and_skills_diagnostic_strings() {
        assert!(okf_missing_version().contains("okf_version"));
        assert!(okf_verified_by_required(0).contains("verified[0]"));
        assert!(skills_missing_name().starts_with("[skills]"));
        assert!(skills_name_dir_mismatch("a", "b").contains("parent directory"));
    }

    #[test]
    fn catalog_ids_unique_and_nonempty() {
        assert!(!CATALOG_MESSAGE_IDS.is_empty());
        let mut seen = std::collections::BTreeSet::new();
        for id in CATALOG_MESSAGE_IDS {
            assert!(!id.is_empty(), "empty id");
            assert!(seen.insert(*id), "duplicate catalog id: {id}");
            assert!(
                id.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "id must be snake_case: {id}"
            );
        }
    }

    #[test]
    fn lifecycle_and_update_detail_helpers() {
        assert!(lifecycle_document_exists("a.md").contains("already exists"));
        assert!(lifecycle_document_not_found("b.md").contains("not found"));
        assert!(detail("open", "eof").contains("open: eof"));
        assert!(update_asset_not_found("ods.tgz", "v1").contains("ods.tgz"));
        assert!(update_unsupported_platform("plan9", "x86").contains("plan9"));
    }
}
