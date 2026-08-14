use ods_core::{
    LintLevel, Severity, current_ods_spec_version, lint_workspace, lint_workspace_with_level,
    load_workspace,
};
use ods_test_support::temp_workspace;
use std::fs;
use std::path::Path;

fn write_root(dir: impl AsRef<Path>, extra: &str) {
    let dir = dir.as_ref();
    fs::write(
        dir.join("index.md"),
        format!("---\nprofile: index\nods: 0.1\n---\n\n# Root\n\n{extra}"),
    )
    .unwrap();
}

#[test]
fn duplicate_tag_warns() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\ntags:\n  - billing\n  - Billing\n---\n\n# A\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    // Both normalize to billing → duplicate after normalize
    assert!(
        ws.documents
            .iter()
            .any(|d| matches!(&d.frontmatter, ods_core::FrontmatterState::Parsed(fm) if fm.tags == vec!["billing".to_string(), "billing".to_string()])),
        "expected normalized duplicate tags"
    );
    let diags = lint_workspace_with_level(&ws, LintLevel::Full);
    assert!(
        diags
            .iter()
            .any(|d| d.severity == Severity::Warning && d.message.contains("duplicate tag")),
        "{diags:?}"
    );
}

#[test]
fn tag_index_builds_from_workspace() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n- [b.md](b.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\ntags:\n  - oncall\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(
        dir.join("b.md"),
        "---\nprofile: note\nstatus: draft\ntags:\n  - oncall\n  - billing\n---\n\n# B\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    assert_eq!(ws.tag_index.get("oncall").map(|v| v.len()), Some(2));
    assert_eq!(ws.tag_index.get("billing").map(|v| v.len()), Some(1));
    let tags = ods_core::completion_tags(&ws);
    assert!(tags.iter().any(|t| t == "oncall"));
    assert!(tags.iter().any(|t| t == "security")); // builtin unused
}

#[test]
fn invalid_status_errors() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: WIP\n---\n\n# A\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(
        diags.iter().any(|d| d.message.contains("invalid status")),
        "{diags:?}"
    );
}

#[test]
fn stale_root_ods_version_errors() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.ods.md"),
        "---\nprofile: index\nods: draft-1\n---\n\n# Root\n\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace_with_level(&ws, LintLevel::Full);
    assert!(
        diags.iter().any(|d| {
            d.severity == Severity::Error
                && d.message.contains("root ods spec version mismatch")
                && d.message.contains(current_ods_spec_version())
        }),
        "{diags:?}"
    );
}

#[test]
fn unknown_profile_is_an_error() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: not-a-real-profile\nstatus: draft\n---\n\n# A\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(
        diags
            .iter()
            .any(|d| { d.severity == Severity::Error && d.message.contains("profile not found") })
    );
}

#[test]
fn missing_custom_profile_path_is_a_load_error() {
    let dir = temp_workspace();
    fs::write(
        dir.join("ods.toml"),
        "spec = \"0.1\"\ncustom_profiles = [\"docs/profiles/incident.md\"]\n",
    )
    .unwrap();

    let error = load_workspace(&dir).expect_err("missing configured profile path must fail");
    let message = error.to_string();
    assert!(
        message.contains("custom profile path not found"),
        "{message}"
    );
    assert!(message.contains("docs/profiles/incident.md"), "{message}");
    assert!(message.contains("custom_profiles"), "{message}");
}

#[test]
fn custom_profile_metadata_outside_registered_path_is_a_load_error() {
    let dir = temp_workspace();
    fs::create_dir_all(dir.join("docs/profiles")).unwrap();
    fs::write(
        dir.join("ods.toml"),
        "spec = \"0.1\"\ncustom_profiles = [\"docs/profiles/incident.md\"]\n",
    )
    .unwrap();
    fs::write(
        dir.join("docs/profiles/incident.md"),
        "---\nods:\n  custom_profile:\n    name: incident\n---\n\n# Incident\n",
    )
    .unwrap();
    fs::write(
        dir.join("ordinary.md"),
        "---\nods:\n  custom_profile:\n    name: wrong-place\n---\n\n# Ordinary\n",
    )
    .unwrap();

    let error = load_workspace(&dir).expect_err("unregistered profile metadata must fail");
    let message = error.to_string();
    assert!(message.contains("invalid ods.custom_profile"), "{message}");
    assert!(message.contains("ordinary.md"), "{message}");
    assert!(message.contains("docs/profiles/incident.md"), "{message}");
}

#[test]
fn required_keys_match_top_level_custom_frontmatter_values() {
    let dir = temp_workspace();
    fs::create_dir_all(dir.join("ods-profiles")).unwrap();
    fs::write(
        dir.join("ods.toml"),
        "spec = \"0.1\"\ncustom_profiles = [\"ods-profiles\"]\n",
    )
    .unwrap();
    fs::write(
        dir.join("ods-profiles/incident.md"),
        "---\nods:\n  custom_profile:\n    name: incident\n    required_keys:\n      - GitHub-Issue\n      - service\n    optional_keys:\n      - owner\n    forbidden_keys:\n      - title\n---\n\n# Incident\n",
    )
    .unwrap();
    fs::write(
        dir.join("valid.md"),
        "---\nprofile: incident\ngithub-issue:\n  provider: github\n  number: 123\nservice: checkout\n---\n\n# Valid\n",
    )
    .unwrap();
    fs::write(
        dir.join("missing.md"),
        "---\nprofile: incident\nservice: checkout\n---\n\n# Missing\n",
    )
    .unwrap();
    fs::write(
        dir.join("null.md"),
        "---\nprofile: incident\ngithub-issue: null\nservice: checkout\n---\n\n# Null\n",
    )
    .unwrap();
    fs::write(
        dir.join("quoted.md"),
        "---\nprofile: incident\ngithub-issue: \"null\"\nservice: checkout\n---\n\n# Quoted\n",
    )
    .unwrap();
    fs::write(
        dir.join("forbidden.md"),
        "---\nprofile: incident\ntitle: Legacy title\ngithub-issue: 456\nservice: checkout\n---\n\n# Forbidden\n",
    )
    .unwrap();

    let workspace = load_workspace(&dir).unwrap();
    let diagnostics = lint_workspace(&workspace);
    let missing = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.message.contains("missing required key"))
        .collect::<Vec<_>>();

    assert_eq!(missing.len(), 2, "{diagnostics:#?}");
    assert!(
        missing
            .iter()
            .any(|diagnostic| diagnostic.path.ends_with("missing.md"))
    );
    assert!(
        missing
            .iter()
            .any(|diagnostic| diagnostic.path.ends_with("null.md"))
    );
    assert!(
        !missing
            .iter()
            .any(|diagnostic| diagnostic.path.ends_with("valid.md"))
    );
    assert!(
        !missing
            .iter()
            .any(|diagnostic| diagnostic.path.ends_with("quoted.md"))
    );
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.path.ends_with("forbidden.md")
                && diagnostic.message.contains("forbidden key 'title'")),
        "{diagnostics:#?}"
    );
}

#[test]
fn required_keys_treat_empty_known_lists_as_non_null() {
    let dir = temp_workspace();
    fs::create_dir_all(dir.join("ods-profiles")).unwrap();
    fs::write(
        dir.join("ods.toml"),
        "spec = \"0.1\"\ncustom_profiles = [\"ods-profiles\"]\n",
    )
    .unwrap();
    fs::write(
        dir.join("ods-profiles/empty-list.md"),
        "---\nods:\n  custom_profile:\n    name: empty-list\n    required_keys:\n      - tags\n---\n\n# Empty List\n",
    )
    .unwrap();
    fs::write(
        dir.join("valid-empty-list.md"),
        "---\nprofile: empty-list\ntags: []\n---\n\n# Valid\n",
    )
    .unwrap();
    fs::write(
        dir.join("null-list.md"),
        "---\nprofile: empty-list\ntags: null\n---\n\n# Null\n",
    )
    .unwrap();

    let workspace = load_workspace(&dir).unwrap();
    let diagnostics = lint_workspace(&workspace);
    let missing = diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.message.contains("missing required key"))
        .collect::<Vec<_>>();

    assert_eq!(missing.len(), 1, "{diagnostics:#?}");
    assert!(missing[0].path.ends_with("null-list.md"));
}

#[test]
fn dangling_reference_errors() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - missing/doc\n---\n\n# A\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("dangling reference"))
    );
}
