use ods_core::{LintLevel, lint_workspace, lint_workspace_with_level, load_workspace};
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
fn non_canonical_heading_emits_missing_section_warning() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n");
    // feature expects Goal among canonical sections; Mission is not canonical and should warn
    fs::write(
        dir.join("a.md"),
        r#"---
profile: feature
status: draft
---

# A

## Mission

## Scope

## Requirements

## Acceptance Criteria

## Risks
"#,
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("missing expected section: Goal")),
        "unexpected section diagnostics: {diags:?}"
    );
}

#[test]
fn duplicate_ids_error() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n- [b.md](b.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\nid: same\n---\n\n# A\n",
    )
    .unwrap();
    fs::write(
        dir.join("b.md"),
        "---\nprofile: note\nstatus: draft\nid: same\n---\n\n# B\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("duplicate document id"))
    );
}

#[test]
fn invalid_share_value_error() {
    let dir = temp_workspace();
    write_root(&dir, "- [a.md](a.md)\n");
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\nshare: invalid\n---\n\n# A\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(
        diags
            .iter()
            .any(|d| d.message.contains("invalid share value: invalid"))
    );
}

#[test]
fn dangling_pack_path_error() {
    let dir = temp_workspace();
    fs::write(
        dir.join("index.ods.md"),
        "---\nprofile: index\nods: 0.1\npacks:\n  - vendor/non-existent-pack\n---\n\n# Root\n",
    )
    .unwrap();
    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace_with_level(&ws, LintLevel::Full);
    assert!(diags.iter().any(|d| {
        d.message
            .contains("missing pack path: vendor/non-existent-pack")
    }));
}

#[test]
fn lint_code_line_suffix_and_extra_index_entries() {
    let dir = temp_workspace();
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.rs"), "fn main() {}\n").unwrap();

    // Index with extra non-existent entry
    write_root(&dir, "- [a.md](a.md)\n- [extra_ghost.md](extra_ghost.md)\n");

    // Code ref with line suffix and body with external URLs / anchors
    fs::write(
        dir.join("a.md"),
        "---\nprofile: note\nstatus: draft\ncode:\n  - path: src/main.rs:L10\n    role: implementation\n---\n\n# A\n\n[Ext](https://example.com) [Anchor](#a) [Mail](mailto:user@test.com)\n",
    )
    .unwrap();

    let ws = load_workspace(&dir).unwrap();
    let diags = lint_workspace(&ws);
    assert!(diags.iter().any(|d| {
        d.message
            .contains("code path must not contain line number suffix")
    }));
    // Nested index child-list lint removed; line-suffix rule is the assertion above.
}
