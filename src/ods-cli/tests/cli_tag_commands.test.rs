use ods_test_support::temp_workspace;
use std::fs;
use std::path::PathBuf;
use std::process::Command;

fn ods_bin() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_ods"))
}

#[test]
fn fmt_refs_md_paths_and_canonical_lint_flag() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", root])
            .output()
            .unwrap()
            .status
            .success()
    );
    fs::create_dir_all(dir.join("website")).unwrap();
    fs::write(
        dir.join("website/cart-checkout.md"),
        "---\nprofile: note\nstatus: stable\n---\n\n# Checkout\n",
    )
    .unwrap();
    fs::write(
        dir.join("feature.md"),
        "---\nprofile: note\nstatus: draft\ndepends:\n  - website/cart-checkout\n---\n\n# Feature\n",
    )
    .unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["lint", root])
            .output()
            .unwrap()
            .status
            .success()
    );

    let out = Command::new(ods_bin())
        .args(["lint", "--canonical-refs", root])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(
        stdout.contains("non-canonical document reference"),
        "{stdout}"
    );

    let out = Command::new(ods_bin())
        .args(["fmt", "--refs", "md-paths", root])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let body = fs::read_to_string(dir.join("feature.md")).unwrap();
    assert!(body.contains("  - website/cart-checkout.md"), "{body}");
}

#[test]
fn status_and_fmt_migrate_preserve_third_party_frontmatter_keys() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", root])
            .output()
            .unwrap()
            .status
            .success()
    );
    fs::write(
        dir.join("post.md"),
        "---\nlayout: post\nauthor: Alice\nhero_image: /img.png\ntags:\n  - rust\nprofile: note\nstatus: draft\n---\n\n# Post\n",
    )
    .unwrap();

    let out = Command::new(ods_bin())
        .current_dir(&dir)
        .args(["status", "post.md", "stable"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let after_status = fs::read_to_string(dir.join("post.md")).unwrap();
    assert!(after_status.contains("layout: post"), "{after_status}");
    assert!(after_status.contains("author: Alice"), "{after_status}");
    assert!(
        after_status.contains("hero_image: /img.png"),
        "{after_status}"
    );
    assert!(
        after_status.contains("status: stable") || after_status.contains("  status: stable"),
        "{after_status}"
    );

    let out = Command::new(ods_bin())
        .args(["fmt", "--migrate", root])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let after_fmt = fs::read_to_string(dir.join("post.md")).unwrap();
    assert!(after_fmt.contains("layout: post"), "{after_fmt}");
    assert!(after_fmt.contains("author: Alice"), "{after_fmt}");
    assert!(after_fmt.contains("hero_image: /img.png"), "{after_fmt}");
    assert!(
        after_fmt.contains("profile: note")
            && (after_fmt.contains("status: stable") || after_fmt.contains("status: draft")),
        "{after_fmt}"
    );
    assert!(!after_fmt.contains("ods:\n  profile:"), "{after_fmt}");

    let out = Command::new(ods_bin())
        .current_dir(&dir)
        .args(["tag", "rename", "rust", "systems", "--write"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let after_tag = fs::read_to_string(dir.join("post.md")).unwrap();
    assert!(after_tag.contains("layout: post"), "{after_tag}");
    assert!(after_tag.contains("author: Alice"), "{after_tag}");
    assert!(
        after_tag.contains("systems") || after_tag.contains("- systems"),
        "{after_tag}"
    );
}

#[test]
fn fmt_migrate_flag_hoists_nested_ods_to_flat_keys() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", root])
            .output()
            .unwrap()
            .status
            .success()
    );
    fs::write(
        dir.join("legacy.md"),
        "---\ndescription: Legacy doc\nods:\n  profile: guide\n  status: draft\n---\n\n# Legacy\n",
    )
    .unwrap();

    let out = Command::new(ods_bin())
        .args(["fmt", "--migrate", root])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("ods: key layout"), "{stdout}");

    let body = fs::read_to_string(dir.join("legacy.md")).unwrap();
    assert!(body.contains("profile: guide"), "{body}");
    assert!(body.contains("status: draft"), "{body}");
    assert!(!body.contains("ods:\n  profile:"), "{body}");
}

#[test]
fn fmt_without_migrate_flag_leaves_legacy_frontmatter_untouched() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", root])
            .output()
            .unwrap()
            .status
            .success()
    );
    let legacy = "---\ndescription: Legacy doc\nprofile: guide\nstatus: draft\n---\n\n# Legacy\n";
    fs::write(dir.join("legacy.md"), legacy).unwrap();

    let out = Command::new(ods_bin())
        .args(["fmt", root])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);

    let body = fs::read_to_string(dir.join("legacy.md")).unwrap();
    assert_eq!(body, legacy);
}

#[test]
fn fmt_migrate_is_idempotent_over_two_runs() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", root])
            .output()
            .unwrap()
            .status
            .success()
    );
    fs::write(
        dir.join("legacy.md"),
        "---\ndescription: Legacy doc\nprofile: guide\nstatus: draft\n---\n\n# Legacy\n",
    )
    .unwrap();

    assert!(
        Command::new(ods_bin())
            .args(["fmt", "--migrate", root])
            .output()
            .unwrap()
            .status
            .success()
    );
    let first_pass = fs::read_to_string(dir.join("legacy.md")).unwrap();

    let out = Command::new(ods_bin())
        .args(["fmt", "--migrate", root])
        .output()
        .unwrap();
    assert!(out.status.success(), "{:?}", out);
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("already clean"), "{stdout}");

    let second_pass = fs::read_to_string(dir.join("legacy.md")).unwrap();
    assert_eq!(first_pass, second_pass);
}

#[test]
fn fmt_migrate_skips_root_index() {
    let dir = temp_workspace();
    let root = dir.to_str().unwrap();
    assert!(
        Command::new(ods_bin())
            .args(["init", root])
            .output()
            .unwrap()
            .status
            .success()
    );
    let root_index_before = fs::read_to_string(dir.join("ods.toml")).unwrap();
    assert!(root_index_before.contains("spec"));

    assert!(
        Command::new(ods_bin())
            .args(["fmt", "--migrate", root])
            .output()
            .unwrap()
            .status
            .success()
    );

    let root_index_after = fs::read_to_string(dir.join("ods.toml")).unwrap();
    assert_eq!(root_index_before, root_index_after);
}
