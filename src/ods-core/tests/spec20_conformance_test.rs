//! Optional integration with ods-spec conformance fixtures (satellite checkout).

use ods_core::{lint_workspace, load_workspace};
use std::fs;
use std::path::{Path, PathBuf};

fn ods_spec_root() -> Option<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../ods-spec");
    root.join("tests/fixtures/2.0.0").is_dir().then_some(root)
}

fn fixture_docs(dir: &Path) -> Vec<PathBuf> {
    let mut out = Vec::new();
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().is_some_and(|e| e == "md") {
                out.push(path);
            }
        }
    }
    out.sort();
    out
}

#[test]
fn ods_spec_20_positive_fixtures_load_and_lint() {
    let Some(spec_root) = ods_spec_root() else {
        eprintln!("skip: ods-spec fixtures not found");
        return;
    };
    let fixtures = fixture_docs(&spec_root.join("tests/fixtures/2.0.0"));
    assert!(
        !fixtures.is_empty(),
        "expected 2.0.0 fixtures under ods-spec"
    );

    let td = tempfile::tempdir().unwrap();
    let ws = td.path();
    fs::write(ws.join("ods.toml"), "spec = \"2.0\"\n").unwrap();
    for f in &fixtures {
        let name = f.file_name().unwrap();
        fs::copy(f, ws.join(name)).unwrap();
    }

    let loaded = load_workspace(ws).expect("workspace loads");
    assert!(
        loaded.documents.len() >= fixtures.len(),
        "expected at least {} docs",
        fixtures.len()
    );
    let _ = lint_workspace(&loaded);
}

#[test]
fn ods_spec_21_positive_fixtures_load() {
    let Some(spec_root) = ods_spec_root() else {
        return;
    };
    let fixtures = fixture_docs(&spec_root.join("tests/fixtures/2.1.0"));
    if fixtures.is_empty() {
        return;
    }
    let td = tempfile::tempdir().unwrap();
    let ws = td.path();
    fs::write(ws.join("ods.toml"), "spec = \"2.1\"\n").unwrap();
    for f in &fixtures {
        let name = f.file_name().unwrap();
        fs::copy(f, ws.join(name)).unwrap();
    }
    let loaded = load_workspace(ws).expect("2.1 workspace loads");
    assert!(!loaded.documents.is_empty());
}
