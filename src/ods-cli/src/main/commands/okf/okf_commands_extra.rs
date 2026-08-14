

fn run_okf_adopt_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_okf_bundle(&root)?;
    let write = args.iter().any(|a| a == "--write");
    let bundle = ods_core::load_okf_bundle(&root).map_err(|e| fail_msg(ods_core::load_okf_bundle_failed(&root, e)))?;
    let mut changed = 0usize;
    for doc in &bundle.documents {
        if doc.is_reserved {
            continue;
        }
        if !matches!(doc.frontmatter, ods_core::OkfFrontmatterState::Absent) {
            continue;
        }
        let rel = doc.path.strip_prefix(&root).unwrap_or(&doc.path);
        if write {
            let stem = doc
                .path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("concept");
            let body = fs::read_to_string(&doc.path).map_err(|e| fail_io("okf", e))?;
            let drafted = format!("---\ntype: Reference\ntitle: {stem}\nstatus: draft\n---\n\n{body}");
            fs::write(&doc.path, drafted).map_err(|e| fail_io("okf", e))?;
            changed += 1;
            if matches!(format, OutputFormat::Text) {
                println!("adopted {}", rel.display());
            }
        } else if matches!(format, OutputFormat::Text) {
            println!("would adopt {}", rel.display());
            changed += 1;
        }
    }
    if matches!(format, OutputFormat::Text) {
        if write {
            println!("adopted {changed} file(s)");
        } else {
            println!("{changed} plain file(s) (pass --write to draft frontmatter)");
        }
    }
    Ok(ExitCode::from(0))
}

fn run_okf_index_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_okf_bundle(&root)?;
    let check = args.iter().any(|a| a == "--check");
    let bundle = ods_core::load_okf_bundle(&root).map_err(|e| fail_msg(ods_core::load_okf_bundle_failed(&root, e)))?;
    if check {
        let current =
            ods_core::okf_indexes_are_current(&bundle).map_err(|e| fail_io("okf", e))?;
        match format {
            OutputFormat::Text => {
                if current {
                    println!("okf indexes up to date");
                } else {
                    eprintln!("okf indexes out of date; run `ods index --okf`");
                }
            }
            OutputFormat::Json | OutputFormat::Sarif => {
                println!(r#"{{"current":{}}}"#, if current { "true" } else { "false" });
            }
        }
        return Ok(ExitCode::from(if current { 0 } else { 1 }));
    }
    let paths =
        ods_core::generate_okf_indexes(&bundle).map_err(|e| fail_io("okf", e))?;
    match format {
        OutputFormat::Text => {
            for p in &paths {
                println!("{}", p.display());
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            println!(r#"{{"written":{},"count":{}}}"#, paths.len(), paths.len());
        }
    }
    Ok(ExitCode::from(0))
}

fn run_okf_context_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_okf_bundle(&root)?;
    let positionals = positional_args(args, 2);
    let id = positionals
        .last()
        .cloned()
        .ok_or_else(|| usage_msg(ods_core::missing_context_id()))?;
    let bundle = ods_core::load_okf_bundle(&root)
        .map_err(|e| fail_msg(ods_core::load_okf_bundle_failed(&root, e)))?;
    let list = ods_core::okf_context(&bundle, &id);
    if list.is_empty() {
        return Err(fail_msg(ods_core::concept_not_found(&id)));
    }
    match format {
        OutputFormat::Text => {
            for p in &list {
                let rel = p.strip_prefix(&root).unwrap_or(p);
                println!("{}", rel.display());
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            let items: Vec<_> = list
                .iter()
                .map(|p| json_escape(&p.display().to_string()))
                .collect();
            println!(r#"{{"context":[{}]}}"#, items.join(","));
        }
    }
    Ok(ExitCode::from(0))
}

fn run_okf_export_command(args: &[String]) -> Result<ExitCode, CliError> {
    let mut out = None;
    let mut path = None;
    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--out" => {
                out = args.get(i + 1).map(PathBuf::from);
                i += 2;
            }
            "--format" => {
                i += 2;
            }
            other if other.starts_with("--out=") => {
                out = Some(PathBuf::from(&other["--out=".len()..]));
                i += 1;
            }
            other if !other.starts_with('-') => {
                path = Some(PathBuf::from(other));
                i += 1;
            }
            _ => i += 1,
        }
    }
    let root = resolve_root_path(
        path.unwrap_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from("."))),
    );
    require_okf_bundle(&root)?;
    let out = out.unwrap_or_else(|| root.join("okf-graph.md"));
    let bundle = ods_core::load_okf_bundle(&root).map_err(|e| fail_msg(ods_core::load_okf_bundle_failed(&root, e)))?;
    let written =
        ods_core::export_okf_graph(&bundle, &out).map_err(|e| fail_io("okf", e))?;
    println!("wrote {}", written.display());
    Ok(ExitCode::from(0))
}

fn run_okf_fmt_command(args: &[String]) -> Result<ExitCode, CliError> {
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_okf_bundle(&root)?;
    let bundle = ods_core::load_okf_bundle(&root).map_err(|e| fail_msg(ods_core::load_okf_bundle_failed(&root, e)))?;
    let changed = ods_core::fmt_okf_bundle(&bundle).map_err(|e| fail_io("okf", e))?;
    match format {
        OutputFormat::Text => {
            if changed.is_empty() {
                println!("okf frontmatter already clean");
            } else {
                println!("formatted {} file(s)", changed.len());
                for p in &changed {
                    let rel = p.strip_prefix(&root).unwrap_or(p);
                    println!("  {}", rel.display());
                }
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            println!(r#"{{"changed":{},"count":{}}}"#, changed.len(), changed.len());
        }
    }
    Ok(ExitCode::from(0))
}

include!("okf_watch.rs");

fn filter_audit_flags(args: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < args.len() {
        match args[i].as_str() {
            "--write-report" => {}
            "--report-path" => {
                i += 1;
            }
            "--fail-on" => {
                i += 1;
            }
            other if other.starts_with("--report-path=") => {}
            other => out.push(other.to_string()),
        }
        i += 1;
    }
    out
}

fn parse_report_path(args: &[String]) -> Option<PathBuf> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--report-path" {
            return args.get(i + 1).map(PathBuf::from);
        }
        if let Some(rest) = args[i].strip_prefix("--report-path=") {
            return Some(PathBuf::from(rest));
        }
        i += 1;
    }
    None
}

fn parse_fail_on(args: &[String]) -> Option<&'static str> {
    let mut i = 0;
    while i < args.len() {
        if args[i] == "--fail-on" {
            return match args.get(i + 1).map(String::as_str) {
                Some("plain") => Some("plain"),
                Some("invalid") => Some("invalid"),
                Some("any") => Some("any"),
                Some(_) => Some("?"),
                None => Some("?"),
            };
        }
        i += 1;
    }
    None
}

#[cfg(test)]
mod test_okf_extra {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    fn okf_root() -> (tempfile::TempDir, String) {
        let td = tempdir().unwrap();
        let root = td.path();
        fs::write(
            root.join("index.md"),
            "---\nokf_version: \"0.2\"\n---\n\n# OKF\n",
        )
        .unwrap();
        fs::write(root.join("plain.md"), "# plain concept\n").unwrap();
        fs::write(
            root.join("metric.md"),
            "---\ntype: Metric\ntitle: M\nstatus: draft\n---\n\n# Metric\n",
        )
        .unwrap();
        let path = root.to_str().unwrap().to_string();
        (td, path)
    }

    #[test]
    fn adopt_index_context_export_fmt_json_branches() {
        let (td, path) = okf_root();
        let out = td.path().join("okf-graph.md");
        let res = run_okf_adopt_command(&[
            "ods".into(),
            "adopt".into(),
            path.clone(),
            "--okf".into(),
        ]);
        assert!(res.is_ok());

        let res = run_okf_adopt_command(&[
            "ods".into(),
            "adopt".into(),
            path.clone(),
            "--okf".into(),
            "--write".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());

        let res = run_okf_index_command(&[
            "ods".into(),
            "index".into(),
            path.clone(),
            "--okf".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());

        let res = run_okf_index_command(&[
            "ods".into(),
            "index".into(),
            path.clone(),
            "--okf".into(),
            "--check".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());

        let res = run_okf_index_command(&[
            "ods".into(),
            "index".into(),
            path.clone(),
            "--okf".into(),
            "--check".into(),
        ]);
        assert!(res.is_ok());

        let res = run_okf_context_command(&[
            "ods".into(),
            "context".into(),
            "--root".into(),
            path.clone(),
            "metric".into(),
            "--okf".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());

        let res = run_okf_context_command(&[
            "ods".into(),
            "context".into(),
            "--root".into(),
            path.clone(),
            "metric".into(),
            "--okf".into(),
        ]);
        assert!(res.is_ok());

        let res = run_okf_context_command(&[
            "ods".into(),
            "context".into(),
            path.clone(),
            "missing-id".into(),
            "--okf".into(),
        ]);
        assert!(res.is_err());

        let res = run_okf_export_command(&[
            "ods".into(),
            "export".into(),
            path.clone(),
            "--okf".into(),
            "--out".into(),
            out.to_str().unwrap().into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());
        assert!(out.exists());

        let res = run_okf_fmt_command(&[
            "ods".into(),
            "fmt".into(),
            path.clone(),
            "--okf".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());

        let res = run_okf_fmt_command(&["ods".into(), "fmt".into(), path, "--okf".into()]);
        assert!(res.is_ok());
    }
}

