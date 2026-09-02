use ods_core::{
    AdoptOptions, adopt_workspace, known_profiles, load_workspace, standard_profile_catalog,
};
use ods_test_support::temp_workspace;
use std::fs;

#[test]
fn standard_catalog_includes_core_profiles() {
    let cat = standard_profile_catalog();
    for name in [
        "note",
        "agent",
        "feature",
        "guide",
        "decision",
        "policy",
        "meeting",
        "index",
        "faq",
        "checklist",
        "api",
        "architecture",
        "sop",
    ] {
        assert!(
            cat.definitions.contains_key(name),
            "missing standard profile {name}"
        );
    }
}

#[test]
fn adopt_infers_feature_from_headings() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.ods.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n\n- [f.md](f.md)\n",
    )
    .unwrap();
    fs::write(
        dir.join("f.md"),
        "# F\n\n## Goal\n\n## Requirements\n\n## Acceptance Criteria\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let report = adopt_workspace(&ws, AdoptOptions { write: true }).unwrap();
    assert_eq!(report.written.len(), 1);
    let text = fs::read_to_string(dir.join("f.md")).unwrap();
    assert!(text.contains("profile: feature"), "{text}");
    assert!(text.contains("status: draft"));
}

#[test]
fn adopt_infers_guide_and_policy() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.ods.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n\n- [g.md](g.md)\n- [p.md](p.md)\n",
    )
    .unwrap();
    fs::write(
        dir.join("g.md"),
        "# G\n\n## Prerequisites\n\n## Steps\n\n## Troubleshooting\n",
    )
    .unwrap();
    fs::write(
        dir.join("p.md"),
        "# P\n\n## Purpose\n\n## Rules\n\n## Exceptions\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    adopt_workspace(&ws, AdoptOptions { write: true }).unwrap();
    assert!(
        fs::read_to_string(dir.join("g.md"))
            .unwrap()
            .contains("profile: guide")
    );
    assert!(
        fs::read_to_string(dir.join("p.md"))
            .unwrap()
            .contains("profile: policy")
    );
}

#[test]
fn adopt_infers_agent_from_headings() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.ods.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n\n- [a.md](a.md)\n",
    )
    .unwrap();
    fs::write(
        dir.join("a.md"),
        "# A\n\n## Goal\n\n## Task\n\n## Scope\n\n## Success Criteria\n\n## Failure Modes\n\n## Assumptions\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    adopt_workspace(&ws, AdoptOptions { write: true }).unwrap();
    let text = fs::read_to_string(dir.join("a.md")).unwrap();
    assert!(text.contains("profile: agent"), "{text}");
}

#[test]
fn known_profiles_lists_standards() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.ods.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let names = known_profiles(&ws);
    assert!(names.iter().any(|n| n == "feature"));
    assert!(names.iter().any(|n| n == "guide"));
}

#[test]
fn adopt_all_remaining_profiles_and_invalid_frontmatter() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.ods.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# R\n",
    )
    .unwrap();
    // Invalid frontmatter file
    fs::write(dir.join("bad.md"), "---\nno_colon_here\n---\n# Bad\n").unwrap();
    // Decision profile
    fs::write(dir.join("d.md"), "# D\n\n## Decision\n").unwrap();
    // SOP profile
    fs::write(dir.join("sop.md"), "# S\n\n## Rollback\n").unwrap();
    // API profile
    fs::write(dir.join("api.md"), "# A\n\n## Endpoint\n").unwrap();
    // Meeting profile
    fs::write(dir.join("m.md"), "# M\n\n## Agenda\n").unwrap();
    // FAQ profile
    fs::write(dir.join("faq.md"), "# F\n\n## Questions\n").unwrap();

    let ws = load_workspace(&dir).unwrap();
    let report = adopt_workspace(&ws, AdoptOptions { write: true }).unwrap();
    let bad_path = dir.join("bad.md");
    let bad_canon = bad_path.canonicalize().unwrap_or_else(|_| bad_path.clone());
    assert!(report.skipped.contains(&bad_path) || report.skipped.contains(&bad_canon));

    assert!(
        fs::read_to_string(dir.join("d.md"))
            .unwrap()
            .contains("profile: decision")
    );
    assert!(
        fs::read_to_string(dir.join("sop.md"))
            .unwrap()
            .contains("profile: sop")
    );
    assert!(
        fs::read_to_string(dir.join("api.md"))
            .unwrap()
            .contains("profile: api")
    );
    assert!(
        fs::read_to_string(dir.join("m.md"))
            .unwrap()
            .contains("profile: meeting")
    );
    assert!(
        fs::read_to_string(dir.join("faq.md"))
            .unwrap()
            .contains("profile: faq")
    );
}

#[test]
fn adopt_preserves_existing_frontmatter_keys() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.ods.md"),
        "---\nprofile: index\nods: 0.1\n---\n\n# Root\n",
    )
    .unwrap();

    let doc_path = dir.join("existing.md");
    fs::write(
        &doc_path,
        "---\ntitle: Cache Strategy\nauthor: Alice\nsidebar_position: 2\n---\n# Overview\n\n## Prerequisites\n",
    )
    .unwrap();

    let ws = load_workspace(&dir).unwrap();
    let report = adopt_workspace(&ws, AdoptOptions { write: true }).unwrap();
    let doc_canon = doc_path.canonicalize().unwrap_or_else(|_| doc_path.clone());
    assert!(
        report.written.contains(&doc_path) || report.written.contains(&doc_canon),
        "expected report.written to contain {doc_path:?} or {doc_canon:?}, got {report:?}"
    );

    let content = fs::read_to_string(&doc_path).unwrap();
    assert!(content.contains("title: Cache Strategy"));
    assert!(content.contains("author: Alice"));
    assert!(content.contains("sidebar_position: 2"));
    assert!(content.contains("profile: guide"));
    assert!(content.contains("status: draft"));
    assert!(!content.contains("ods:\n  profile:"));
}
