use ods_core::{
    NewDocumentOptions, RemoveDocumentOptions, atomic_delete_document, document_id,
    scaffold_new_document,
};

fn run_new_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("new");
        return Ok(ExitCode::from(0));
    }
    if args.len() < 3 {
        return Err(usage_msg(ods_core::missing_required_arg(
            "path",
            "ods new <path> [--profile <p>] [--title \"<t>\"]",
        )));
    }

    let mut target_path = None;
    let mut profile = None;
    let mut title = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "--profile" | "-p" => {
                let v = args.get(i + 1).ok_or_else(|| usage_msg(ods_core::missing_flag_value("--profile", "`ods new path.md --profile note`")))?;
                profile = Some(v.clone());
                i += 2;
            }
            "--title" | "-t" => {
                let v = args.get(i + 1).ok_or_else(|| usage_msg(ods_core::missing_flag_value("--title", "`ods new path.md --title \"My Doc\"`")))?;
                title = Some(v.clone());
                i += 2;
            }
            other if !other.starts_with('-') => {
                if target_path.is_none() {
                    target_path = Some(PathBuf::from(other));
                }
                i += 1;
            }
            _ => i += 1,
        }
    }

    let Some(path) = target_path else {
        return Err(usage_msg(ods_core::missing_required_arg("path", "ods new <path> [--profile <p>]")));
    };

    let root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let report = scaffold_new_document(&root, &path, NewDocumentOptions { profile, title })
        .map_err(|e| fail_msg(ods_core::scaffold_failed(e)))?;

    println!(
        "created document {}\n  id: {}\n  profile: {}\n  indexes updated: {}",
        report.created_file.display(),
        report.doc_id,
        report.profile,
        report.updated_indexes.len()
    );

    Ok(ExitCode::from(0))
}

fn run_rm_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("rm");
        return Ok(ExitCode::from(0));
    }
    let (_, _level, format) = parse_common_flags(args, 2)?;
    let positionals = positional_args(args, 2);
    let dry_run = args.iter().any(|a| a == "--dry-run");

    let root_flag = parse_flag_val(args, "--root").map(PathBuf::from);
    let (root_dir, target_str) = if let Some(rf) = root_flag {
        let t = positionals.first().ok_or_else(|| usage_msg(ods_core::missing_required_arg("path-or-id", "ods rm --root <dir> <path-or-id>")))?;
        (rf, t.clone())
    } else if positionals.len() >= 2 && PathBuf::from(&positionals[0]).is_dir() {
        (PathBuf::from(&positionals[0]), positionals[1].clone())
    } else if !positionals.is_empty() {
        (env::current_dir().unwrap_or_else(|_| PathBuf::from(".")), positionals[0].clone())
    } else {
        return Err(usage_msg(ods_core::missing_required_arg("path-or-id", "ods rm [root] <path-or-id> [--dry-run]")));
    };

    let root = resolve_root_path(root_dir);
    require_ods_workspace(&root)?;
    let target = PathBuf::from(&target_str);

    if dry_run {
        match format {
            OutputFormat::Text => {
                println!("(dry-run) would delete document {} and scrub references across workspace {}", target.display(), root.display());
            }
            OutputFormat::Json | OutputFormat::Sarif => {
                println!(
                    r#"{{"dry_run":true,"target":{},"root":{}}}"#,
                    json_escape(&target.display().to_string()),
                    json_escape(&root.display().to_string())
                );
            }
        }
        return Ok(ExitCode::from(0));
    }

    let report = atomic_delete_document(&root, &target, RemoveDocumentOptions { scrub_dependencies: true })
        .map_err(|e| fail_msg(ods_core::io_failed("delete document", e)))?;

    match format {
        OutputFormat::Text => {
            println!(
                "deleted document {}\n  id: {}\n  cleaned graph references: {}\n  indexes updated: {}",
                report.deleted_file.display(),
                report.doc_id,
                report.cleaned_references_count,
                report.updated_indexes.len()
            );
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            println!(
                r#"{{"deleted_file":{},"doc_id":{},"cleaned_references_count":{},"updated_indexes_count":{}}}"#,
                json_escape(&report.deleted_file.display().to_string()),
                json_escape(&report.doc_id),
                report.cleaned_references_count,
                report.updated_indexes.len()
            );
        }
    }

    Ok(ExitCode::from(0))
}

fn run_archive_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("archive");
        return Ok(ExitCode::from(0));
    }
    // Thin alias for `ods status <path-or-id> archived` (friendly archive wording).
    let target = args.get(2).map(String::as_str).unwrap_or("");
    if target.is_empty() || target.starts_with('-') {
        return Err(usage_msg(ods_core::missing_required_arg(
            "path-or-id",
            "ods archive <path-or-id>",
        )));
    }
    set_document_status_with_label(target, "archived", "archived document")
}

fn run_status_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("status");
        return Ok(ExitCode::from(0));
    }
    let positionals = positional_args(args, 2);
    match positionals.as_slice() {
        [path, status] => set_document_status_with_label(path, status, "set status on"),
        [_] => Err(usage_msg(ods_core::missing_required_arg(
            "status",
            "ods status <path-or-id> <draft|stable|deprecated|archived>",
        ))),
        _ => Err(usage_msg(ods_core::missing_required_arg(
            "path-or-id status",
            "ods status <path-or-id> <draft|stable|deprecated|archived>",
        ))),
    }
}

fn set_document_status_with_label(
    target_str: &str,
    status: &str,
    action_label: &str,
) -> Result<ExitCode, CliError> {
    if target_str.is_empty() {
        return Err(usage_msg(ods_core::missing_required_arg(
            "path-or-id",
            "ods status <path-or-id> <draft|stable|deprecated|archived>",
        )));
    }
    let status = status.trim().to_ascii_lowercase();
    const ALLOWED: &[&str] = &["draft", "stable", "deprecated", "archived"];
    if !ALLOWED.contains(&status.as_str()) {
        return Err(usage_msg(ods_core::invalid_choice(
            "status",
            &status,
            "draft|stable|deprecated|archived",
        )));
    }

    let target = PathBuf::from(target_str);
    let root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let workspace = load_workspace(&root).map_err(|e| fail_load(&root, e))?;

    let target_abs = if target.is_absolute() {
        target.clone()
    } else {
        root.join(&target)
    };
    let target_canon = target_abs.canonicalize().ok();
    let target_stem = target.file_stem().map(|s| s.to_string_lossy().to_lowercase());
    let target_id_str = target.to_string_lossy().to_lowercase();

    let doc = workspace
        .documents
        .iter()
        .find(|d| {
            let did = document_id(
                &root,
                &d.path,
                match &d.frontmatter {
                    FrontmatterState::Parsed(fm) => Some(fm),
                    _ => None,
                },
            );
            d.path == target_abs
                || target_canon.as_ref() == Some(&d.path)
                || (target_canon.is_some() && d.path.canonicalize().ok() == target_canon)
                || did == target_id_str
                || target_stem.as_deref() == Some(did.as_str())
        })
        .ok_or_else(|| fail_msg(ods_core::document_not_found(target_str)))?;

    let doc_id = document_id(
        &root,
        &doc.path,
        match &doc.frontmatter {
            FrontmatterState::Parsed(fm) => Some(fm),
            _ => None,
        },
    );

    let text =
        fs::read_to_string(&doc.path).map_err(|e| fail_msg(ods_core::io_failed("read file", e)))?;
    let (fm_opt, body) = ods_core::split_frontmatter(&text);

    let new_text = if let Some(fm) = fm_opt {
        let lines = set_frontmatter_status(fm, &status);
        format!("---\n{}\n---\n\n{}", lines.join("\n"), body.trim_start())
    } else {
        format!("---\nstatus: {status}\n---\n\n{}", text.trim_start())
    };

    fs::write(&doc.path, new_text)
        .map_err(|e| fail_msg(ods_core::io_failed("write file", e)))?;

    println!(
        "{action_label} {}\n  id: {}\n  status: {status}",
        doc.path.display(),
        doc_id
    );
    Ok(ExitCode::from(0))
}

/// Set `status:` on flat top-level and nested `ods.status` keys (preserve indent).
fn set_frontmatter_status(fm: &str, status: &str) -> Vec<String> {
    let mut lines: Vec<String> = fm.lines().map(|s| s.to_string()).collect();
    let mut in_ods_map = false;
    let mut ods_child_indent: Option<usize> = None;
    let mut updated_nested = false;
    let mut updated_flat = false;
    let mut has_ods_map = false;

    for line in &lines {
        if line.trim() == "ods:" {
            has_ods_map = true;
            break;
        }
    }

    for line in &mut lines {
        let trimmed = line.trim();
        let indent = line.len() - line.trim_start().len();

        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        if trimmed == "ods:" {
            in_ods_map = true;
            ods_child_indent = None;
            continue;
        }

        // Scalar root marker `ods: 0.1` — not a nested map.
        if trimmed.starts_with("ods:") {
            in_ods_map = false;
            ods_child_indent = None;
            continue;
        }

        if in_ods_map {
            if ods_child_indent.is_none() {
                if indent > 0 {
                    ods_child_indent = Some(indent);
                } else {
                    in_ods_map = false;
                }
            }
            if let Some(ci) = ods_child_indent {
                if indent < ci {
                    in_ods_map = false;
                    ods_child_indent = None;
                } else if indent == ci && trimmed.starts_with("status:") {
                    *line = format!("{:indent$}status: {status}", "", indent = indent);
                    updated_nested = true;
                    continue;
                } else {
                    continue;
                }
            }
        }

        if !in_ods_map && indent == 0 && trimmed.starts_with("status:") {
            *line = format!("status: {status}");
            updated_flat = true;
        }
    }

    if !updated_nested && has_ods_map {
        if let Some(idx) = lines.iter().position(|l| l.trim() == "ods:") {
            lines.insert(idx + 1, format!("  status: {status}"));
            updated_nested = true;
        }
    }
    if !updated_nested && !updated_flat {
        lines.push(format!("status: {status}"));
    }

    lines
}

/// Show background service logs under `~/.ods/logs/` (not a watch alias).
fn run_logs_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("logs");
        return Ok(ExitCode::from(0));
    }
    let follow = args.iter().any(|a| a == "-f" || a == "--follow");
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .unwrap_or_else(|_| ".".into());
    let log_dir = std::path::PathBuf::from(&home).join(".ods").join("logs");
    let log_path = log_dir.join("ods-serve.log");
    if !log_path.exists() {
        println!("no service logs found under {}", log_dir.display());
        println!("hint: start the background service with `ods start` (OS service writes ods-serve.log)");
        return Ok(ExitCode::from(0));
    }

    let print_once = || -> Result<(), CliError> {
        match std::fs::read_to_string(&log_path) {
            Ok(body) if body.is_empty() => {
                println!("{} is empty (service may not have emitted output yet)", log_path.display());
            }
            Ok(body) => print!("{body}"),
            Err(e) => {
                return Err(fail_msg(ods_core::io_failed("read log", e)));
            }
        }
        Ok(())
    };

    if !follow {
        print_once()?;
        return Ok(ExitCode::from(0));
    }

    println!("following {} (Ctrl+C to stop)...", log_path.display());
    let mut last_len = 0usize;
    loop {
        match std::fs::read_to_string(&log_path) {
            Ok(body) => {
                if body.len() > last_len {
                    print!("{}", &body[last_len..]);
                    last_len = body.len();
                } else if body.len() < last_len {
                    // truncated/rotated
                    print!("{body}");
                    last_len = body.len();
                }
            }
            Err(e) => {
                eprintln!("ods logs: read error: {e}");
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
}

#[cfg(test)]
mod test_lifecycle_helpers {
    use super::*;

    #[test]
    fn archive_status_flat_nested_and_insert() {
        let flat = "profile: note\nstatus: draft\n";
        let out = set_frontmatter_status(flat, "archived");
        assert!(out.iter().any(|l| l.trim() == "status: archived"), "{out:?}");

        let nested = "ods:\n  profile: note\n  status: draft\n";
        let out = set_frontmatter_status(nested, "archived");
        assert!(
            out.iter().any(|l| l.contains("status: archived")),
            "{out:?}"
        );

        let ods_map_no_status = "ods:\n  profile: note\nprofile: note\n";
        let out = set_frontmatter_status(ods_map_no_status, "archived");
        assert!(
            out.iter().any(|l| l.contains("status: archived")),
            "{out:?}"
        );

        let scalar_ods = "ods: 0.1\nprofile: note\n";
        let out = set_frontmatter_status(scalar_ods, "archived");
        assert!(
            out.iter().any(|l| l.contains("status: archived")),
            "{out:?}"
        );

        let empty = "profile: note\n";
        let out = set_frontmatter_status(empty, "archived");
        assert!(
            out.iter().any(|l| l.contains("status: archived")),
            "{out:?}"
        );
    }

    #[test]
    fn test_run_logs_command_smoke() {
        let res = run_logs_command(&["ods".into(), "logs".into()]);
        assert!(res.is_ok());
    }

    #[test]
    fn archive_status_flat_nested_and_insert_extended() {
        // comments / blanks
        let with_comments = "# c\n\nprofile: note\nstatus: draft\n";
        let out = set_frontmatter_status(with_comments, "archived");
        assert!(out.iter().any(|l| l.contains("archived")));

        let nested = "ods:\n  profile: note\n  status: draft\n";
        let out = set_frontmatter_status(nested, "stable");
        assert!(out.iter().any(|l| l.contains("status: stable")), "{out:?}");
    }

    #[test]
    fn new_rm_archive_command_usage_errors() {
        let err = run_new_command(&["ods".into(), "new".into()]).unwrap_err();
        assert!(err.message().contains("new") || err.message().contains("path"));

        let err = run_rm_command(&["ods".into(), "rm".into()]).unwrap_err();
        assert!(err.message().contains("rm") || err.message().contains("path"));

        let err = run_archive_command(&["ods".into(), "archive".into()]).unwrap_err();
        assert!(err.message().contains("archive") || err.message().contains("path"));

        let err = run_status_command(&["ods".into(), "status".into()]).unwrap_err();
        assert!(err.message().contains("status") || err.message().contains("path"));
    }

    #[test]
    fn test_run_new_rm_archive_restore_and_logs() {
        let td = tempfile::tempdir().unwrap();
        let root = td.path();
        fs::write(root.join("ods.toml"), "spec = \"0.1\"\n").unwrap();
        let index_path = root.join("index.ods.md");
        fs::write(
            &index_path,
            "---\nprofile: index\nods: 0.1\nchildren:\n  - doc.md\n---\n\n# Root\n",
        )
        .unwrap();

        let doc_path = root.join("doc.md");
        fs::write(
            &doc_path,
            "---\nprofile: note\nstatus: draft\n---\n\n# Doc\n",
        )
        .unwrap();

        // 1. run_new_command
        let new_doc = root.join("sub").join("feature.md");
        let res = run_new_command(&[
            "ods".into(),
            "new".into(),
            new_doc.to_str().unwrap().into(),
            "--title".into(),
            "My Feature".into(),
            "--profile".into(),
            "feature".into(),
        ]);
        assert!(res.is_ok());
        assert!(new_doc.exists());

        // dry run new
        let dry_doc = root.join("dry.md");
        let res = run_new_command(&[
            "ods".into(),
            "new".into(),
            dry_doc.to_str().unwrap().into(),
            "--dry-run".into(),
        ]);
        assert!(res.is_ok());
        assert!(dry_doc.exists());

        // 2. set_frontmatter_status helper
        let archived = set_frontmatter_status("profile: note\nstatus: draft\n", "archived");
        assert!(archived.iter().any(|l| l.contains("status: archived")));

        let stable = set_frontmatter_status("profile: note\nstatus: archived\n", "stable");
        assert!(stable.iter().any(|l| l.contains("status: stable")));


        // 4. run_rm_command
        let res = run_rm_command(&[
            "ods".into(),
            "rm".into(),
            root.to_str().unwrap().into(),
            new_doc.to_str().unwrap().into(),
        ]);
        assert!(res.is_ok());

        // rm missing args
        let err = run_rm_command(&["ods".into(), "rm".into()]).unwrap_err();
        assert!(err.message().contains("path-or-id"));

        // rm dry run text and json
        let res = run_rm_command(&[
            "ods".into(),
            "rm".into(),
            root.to_str().unwrap().into(),
            index_path.to_str().unwrap().into(),
            "--dry-run".into(),
        ]);
        assert!(res.is_ok());

        let res = run_rm_command(&[
            "ods".into(),
            "rm".into(),
            root.to_str().unwrap().into(),
            index_path.to_str().unwrap().into(),
            "--dry-run".into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());

        // 5. run_logs_command
        let res = run_logs_command(&["ods".into(), "logs".into(), root.to_str().unwrap().into()]);
        assert!(res.is_ok());
    }
}
