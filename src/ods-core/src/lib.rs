#![forbid(unsafe_code)]

pub mod bench;
pub mod config;
pub mod error;
pub mod fs;
pub mod graph;
pub mod lifecycle;
pub mod lint;
pub mod memory;
pub mod model;
pub mod multi_spec;
pub mod mutate;
pub mod mv;
pub mod observe;
pub mod okf;
pub mod parse;
pub mod pipeline;
pub mod profiles;
pub mod share;
pub mod spec;
pub mod store;
pub mod tags;

// Compatibility paths used by internal `crate::refs` / `crate::export` style imports.
pub mod adopt {
    pub use crate::mutate::{AdoptOptions, AdoptReport, adopt_workspace};
}
pub mod context {
    pub use crate::graph::{
        ContextOptions, ContextResult, estimate_path_tokens, render_context_pack, resolve_context,
        resolve_context_start, resolve_context_with_options,
    };
}
pub mod export {
    pub use crate::graph::{export_workspace_graph, render_graph_json, render_graph_markdown};
}
pub mod refs {
    pub use crate::graph::{
        canonical_document_ref, canonical_document_ref_for_reference, document_ref_to_id,
        document_ref_to_path, is_file_like_ref, is_markdown_ref,
    };
}

pub use bench::{
    BenchRestoreReport, BenchRunReport, BenchStatsReport, BenchStripOptions, BenchStripReport,
    bench_calculate_stats, bench_restore_workspace, bench_run_simulation, bench_strip_workspace,
    compute_repo_hash, create_workspace_snapshot, get_backup_dir, list_workspace_snapshots,
    undo_latest_snapshot,
};

pub use fs::{
    ReadOptions, ReadResult, SectionOutline, find_workspace_root, load_options_graph,
    load_options_with_bodies, load_workspace, load_workspace_with_options, normalize_join,
    normalize_path, path_matches_workspace_ignore, read_document_content, rebuild_indexes,
    remove_document, upsert_document,
};
pub use graph::{
    ContextOptions, ContextResult, canonical_document_ref, canonical_document_ref_for_reference,
    document_ref_to_id, document_ref_to_path, estimate_path_tokens, export_workspace_graph,
    is_file_like_ref, is_markdown_ref, render_context_pack, render_graph_json,
    render_graph_markdown, resolve_context, resolve_context_start, resolve_context_with_options,
};
pub use lifecycle::{
    DisableOptions, DisableReport, InitOptions, InitReport, NewDocumentOptions, NewDocumentReport,
    RemoveDocumentOptions, RemoveDocumentReport, atomic_delete_document, disable_workspace,
    init_workspace, ods_enabled, ods_enabled_for_path, scaffold_new_document,
    strip_ods_from_document_text,
};
pub use memory::{DEFAULT_MAX_RSS_MB, current_rss_kb, rss_over_budget, strip_workspace_bodies};
pub use mutate::{AdoptOptions, AdoptReport, adopt_workspace};
pub use observe::{
    TreeSnapshot, WatchTree, observe_renames, paired_from_paths, scan_markdown_tree,
    scan_markdown_tree_with_code_paths,
};
pub use pipeline::{
    apply_document_removes, apply_document_upserts, discover_markdown_paths, parse_path,
    parse_paths_parallel, parse_pool_jobs,
};
pub mod path_util {
    pub use crate::fs::{normalize_join, normalize_path};
}
pub use config::{
    ServiceConfig, WorkspaceConfig, load_workspace_config, migrate_root_index_to_toml,
    ods_toml_enabled, ods_toml_path, render_ods_toml, write_ods_toml,
};
pub use lint::{
    known_profiles, lint_document_in_workspace, lint_workspace, lint_workspace_with_level,
    lint_workspace_with_ref_style, profile_section_labels, profile_sections, workspace_compliance,
};
pub use model::{
    CodeRef, CodeRole, ComplianceMode, CustomProfileDefinition, CustomValue, Diagnostic, Document,
    Frontmatter, FrontmatterState, LintLevel, LoadOptions, ProfileCatalog, ProfileConflict,
    ProfileDefinition, ResourceRef, Severity, SpecLintConfig, Workspace, WorkspaceCompliance,
    WorkspaceSpecsConfig, current_ods_spec_version, current_ods_version, is_spec_at_least,
    parse_spec_version,
};
pub use mv::{
    PathChange, PathChangeReport, apply_path_changes, canonicalize_workspace_document_refs,
    canonicalize_workspace_document_refs_with_workspace, classify_watch_events,
    compute_path_change_edits, heal_orphan_path_ids, migrate_frontmatter_to_canonical,
    migrate_workspace_frontmatter, migrate_workspace_frontmatter_with_workspace,
    move_document_and_rewrite_refs, move_document_and_rewrite_refs_report,
    normalize_frontmatter_body_spacing, normalize_workspace_frontmatter_spacing,
    normalize_workspace_frontmatter_spacing_with_workspace, reindex_workspace,
    rewrite_references_in_text, rewrite_refs_after_moves,
};
pub use parse::{
    document_id, extract_heading_groups, extract_headings, parse_document_text, split_frontmatter,
    split_markdown_link_target,
};
pub use profiles::{
    load_profile_catalog, profile_catalog_roots, profile_catalog_roots_from_config,
    render_profile_template, resolve_document_profile, standard_profile_catalog,
};
pub use share::{ShareLevel, ShareOptions, SharePublishReport, effective_share, publish_workspace};
pub use spec::{
    KeyDefinition, KeyPlacement, KeyType, SchemaIssue, SpecKind, SpecSchema, SpecSchemaRegistry,
    evaluate_document_key_query, evaluate_single_key_clause, filter_documents_by_keys,
    generate_ods_json_schema, get_document_key_values, validate_ods_frontmatter,
};
pub use store::{DocMeta, StorePatch, WorkspaceStore};
pub use tags::{
    TagRenameReport, builtin_tags, completion_tags, docs_with_all_tags, docs_with_any_tag,
    docs_with_tag, is_builtin_tag, normalize_tag, normalize_tag_list, observed_tags,
    rename_tag_in_workspace, rewrite_tags_in_text, tag_usage, tag_usage_with_builtins,
};

pub use okf::{
    ActorEvent, DateRange, OkfAuditClass, OkfAuditReport, OkfBundle, OkfDocument, OkfFrontmatter,
    OkfFrontmatterState, OkfInitOptions, OkfInitReport, OkfLintLevel, OkfParameter, OkfSource,
    OkfStatus, OkfTrustTier, ResourceRefFields, audit_okf_bundle, concept_id_for_path,
    current_okf_version, derive_trust_tier, export_okf_graph, fmt_okf_bundle, generate_okf_indexes,
    init_okf_bundle, lint_okf_bundle, lint_okf_bundle_with_config, lint_okf_bundle_with_level,
    load_okf_bundle, okf_context, okf_enabled, okf_indexes_are_current, okf_version_from_root,
    parse_okf_frontmatter_block, render_okf_audit_markdown,
};

pub use multi_spec::{
    ActiveEngines, Detected, ExtraSpecs, ScopeResolveError, SkillFrontmatter, SkillPackage,
    SkillsInitOptions, SkillsInitReport, detect_workspace, init_skill_package, lint_skill_package,
    lint_skill_package_with_config, load_root_specs_config, parse_extra_spec_flags,
    parse_skill_package, resolve_engines, resolve_engines_with_config, skill_package_roots,
    skills_enabled,
};

pub use error::{
    CATALOG_MESSAGE_IDS, ErrorStage, KNOWN_COMMANDS, UserMsg, already_exists, concept_not_found,
    context_filter_ambiguous, context_requires_ods_or_okf, document_not_found,
    document_not_found_context, forbidden_ods_flag, home_dir_unresolved, invalid_choice, io_failed,
    load_okf_bundle_failed, load_workspace_failed, missing_context_id, missing_flag_value,
    missing_required_arg, no_skills_package, not_ods_workspace, not_okf_bundle,
    okf_namespace_removed, path_not_found, render_error, render_usage, root_index_missing,
    scaffold_failed, service_failed, suggest_command, undo_no_snapshot, unknown_command,
    unknown_flag, unknown_ods_command, unknown_platform_command, unknown_subcommand, update_failed,
};
