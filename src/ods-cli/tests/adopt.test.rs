use ods_test_support::temp_workspace;
use std::env;
use std::fs;
use std::process::{Command, Stdio};

#[test]
fn adopt_reports_missing_canonical_sections() {
    let root = temp_workspace();
    fs::write(root.join("ods.toml"), "spec = \"0.1\"\n").expect("root index");
    fs::write(
        root.join("feature.md"),
        "---\nprofile: feature\nstatus: draft\n---\n\n# Feature\n\n## Mission\n## Scope\n## Requirements\n## Acceptance Criteria\n## Risks\n",
    )
    .expect("feature");

    let bin = env!("CARGO_BIN_EXE_ods");
    let output = Command::new(bin)
        .current_dir(&root)
        .arg("adopt")
        .arg(&root)
        .stdin(Stdio::null())
        .output()
        .expect("run ods adopt");

    let stdout = String::from_utf8(output.stdout).expect("utf8");
    assert!(
        stdout.contains("missing expected section: Goal"),
        "{stdout}"
    );
}
