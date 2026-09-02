use ods_core::{LintLevel, lint_document_in_workspace, lint_workspace, load_workspace};
use ods_test_support::temp_workspace;
use std::fs;

fn write_root(temp: &std::path::Path, spec: &str) {
    fs::write(temp.join("ods.toml"), format!("spec = \"{spec}\"\n")).expect("ods.toml");
    fs::write(
        temp.join("index.md"),
        "---\nprofile: index\n---\n\n# Root\n",
    )
    .expect("index");
}

fn assert_has_message(diagnostics: &[ods_core::Diagnostic], needle: &str) {
    assert!(
        diagnostics.iter().any(|d| d.message.contains(needle)),
        "expected diagnostic containing `{needle}`; got: {diagnostics:#?}"
    );
}

#[test]
fn spec20_rejects_legacy_and_wrapper_keys() {
    let temp = temp_workspace();
    write_root(&temp, "2.0");
    fs::write(
        temp.join("bad.md"),
        "---\nprofile: note\nstatus: draft\nods:\n  profile: note\ncontext:\n  max-depth: 1\ncustom_profile:\n  name: x\nmemory: true\ninvariants: []\n---\n\n# Bad\n",
    )
    .expect("doc");

    let ws = load_workspace(&temp).expect("load");
    let diags = lint_document_in_workspace(&ws, &temp.join("bad.md"), LintLevel::Full);
    assert_has_message(&diags, "ods:");
    assert_has_message(&diags, "context");
    assert_has_message(&diags, "custom_profile");
    assert_has_message(&diags, "memory");
    assert_has_message(&diags, "invariants");
}

#[test]
fn spec20_rejects_code_object_form() {
    let temp = temp_workspace();
    write_root(&temp, "2.0");
    fs::write(
        temp.join("code.md"),
        "---\nprofile: note\nstatus: draft\ncode:\n  - path: ./src/lib.rs\n    role: implementation\n---\n\n# Code\n",
    )
    .expect("doc");
    fs::create_dir_all(temp.join("src")).expect("src");
    fs::write(temp.join("src/lib.rs"), "// lib\n").expect("lib");

    let ws = load_workspace(&temp).expect("load");
    let diags = lint_document_in_workspace(&ws, &temp.join("code.md"), LintLevel::Full);
    assert_has_message(&diags, "code");
}

#[test]
fn spec20_title_rules_use_title_not_name() {
    let temp = temp_workspace();
    write_root(&temp, "2.0");
    fs::write(
        temp.join("skill.md"),
        "---\nprofile: note\nstatus: draft\nname: slug\n---\n\n# Display Title\n",
    )
    .expect("doc");
    fs::write(
        temp.join("titled.md"),
        "---\nprofile: note\nstatus: draft\ntitle: Wrong\n---\n\n# Right Title\n",
    )
    .expect("titled");
    fs::write(
        temp.join("matched.md"),
        "---\nprofile: note\nstatus: draft\ntitle: Matched\n---\n\n# Matched\n",
    )
    .expect("matched");
    fs::write(
        temp.join("missing-h1.md"),
        "---\nprofile: note\nstatus: draft\ntitle: Alone\n---\n\nBody without headings.\n",
    )
    .expect("missing");

    let ws = load_workspace(&temp).expect("load");
    let skill = lint_document_in_workspace(&ws, &temp.join("skill.md"), LintLevel::Full);
    assert!(!skill.iter().any(|d| d.message.contains("TITLE-001")));

    let titled = lint_document_in_workspace(&ws, &temp.join("titled.md"), LintLevel::Full);
    assert_has_message(&titled, "TITLE-001");

    let matched = lint_document_in_workspace(&ws, &temp.join("matched.md"), LintLevel::Full);
    assert!(!matched.iter().any(|d| d.message.contains("TITLE-001")));

    let missing = lint_document_in_workspace(&ws, &temp.join("missing-h1.md"), LintLevel::Full);
    assert_has_message(&missing, "TITLE-002");
}

#[test]
fn spec20_okf_skips_title_lint() {
    let temp = temp_workspace();
    write_root(&temp, "2.0");
    fs::write(
        temp.join("okf.md"),
        "---\nprofile: note\nstatus: draft\ntype: idea\ntitle: Wrong\n---\n\n# Right\n",
    )
    .expect("okf");
    let ws = load_workspace(&temp).expect("load");
    let diags = lint_document_in_workspace(&ws, &temp.join("okf.md"), LintLevel::Full);
    assert!(!diags.iter().any(|d| d.message.contains("TITLE-001")));
}

#[test]
fn spec20_asset_load_paths_must_exist() {
    let temp = temp_workspace();
    write_root(&temp, "2.0");
    fs::write(
        temp.join("asset.md"),
        "---\nprofile: note\nstatus: draft\nload:\n  - missing.json\n---\n\n# Asset\n",
    )
    .expect("doc");

    let ws = load_workspace(&temp).expect("load");
    let diags = lint_document_in_workspace(&ws, &temp.join("asset.md"), LintLevel::Full);
    assert_has_message(&diags, "load");
}

#[test]
fn spec20_ontology_keys_gated_by_spec() {
    let temp = temp_workspace();
    write_root(&temp, "2.0");
    fs::write(
        temp.join("ont.md"),
        "---\nprofile: note\nstatus: draft\nentity: User\ndomain: billing\nschema: ./schema.json\nrelated:\n  - owns: index\n---\n\n# Ont\n",
    )
    .expect("doc");

    let ws = load_workspace(&temp).expect("load");
    let diags = lint_document_in_workspace(&ws, &temp.join("ont.md"), LintLevel::Full);
    assert_has_message(&diags, "entity");
    assert_has_message(&diags, "domain");
    assert_has_message(&diags, "schema");
    assert_has_message(&diags, "related (typed)");
}

#[test]
fn spec21_ontology_schema_and_entity_lint() {
    let temp = temp_workspace();
    write_root(&temp, "2.1");
    fs::write(
        temp.join("a.md"),
        "---\nprofile: note\nstatus: draft\nentity: Shared\ndomain: ops\nschema: ./missing.json\n---\n\n# A\n",
    )
    .expect("a");
    fs::write(
        temp.join("b.md"),
        "---\nprofile: note\nstatus: draft\nentity: Shared\n---\n\n# B\n",
    )
    .expect("b");
    fs::write(
        temp.join("target.md"),
        "---\nprofile: note\nstatus: draft\nid: target\n---\n\n# Target\n",
    )
    .expect("target");
    fs::write(
        temp.join("pred.md"),
        "---\nprofile: note\nstatus: draft\nrelated:\n  - is_a: target\n---\n\n# Pred\n",
    )
    .expect("pred");

    let ws = load_workspace(&temp).expect("load");
    let schema_diags = lint_document_in_workspace(&ws, &temp.join("a.md"), LintLevel::Full);
    assert_has_message(&schema_diags, "schema");

    let pred_diags = lint_document_in_workspace(&ws, &temp.join("pred.md"), LintLevel::Full);
    assert!(pred_diags.iter().all(|d| !d.message.contains("dangling")));

    let all = lint_workspace(&ws);
    assert_has_message(&all, "entity");
}
