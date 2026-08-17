fn run_disable_command(args: &[String]) -> Result<ExitCode, CliError> {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_command_help("disable");
        return Ok(ExitCode::from(0));
    }
    let (root, _level, format) = parse_common_flags(args, 2)?;
    require_ods_workspace(&root)?;
    let write = args.iter().any(|a| a == "--write");
    let keep_frontmatter = args.iter().any(|a| a == "--keep-frontmatter");
    let remove_indexes = args.iter().any(|a| a == "--remove-indexes");
    let remove_root_index = args.iter().any(|a| a == "--remove-root-index");
    let options = DisableOptions {
        write,
        strip_frontmatter: !keep_frontmatter,
        strip_root_policy: true,
        remove_indexes,
        remove_root_index,
    };
    // Prefer ODS root if path is inside one
    let root = ods_core::find_workspace_root(&root).unwrap_or(root);
    let report =
        disable_workspace(&root, options).map_err(|err| fail_io("disable", err))?;
    match format {
        OutputFormat::Text => {
            if report.already_disabled {
                println!(
                    "ODS not enabled at {} (nothing to disable)",
                    report.root.display()
                );
            } else {
                let mode = if report.dry_run { "dry-run" } else { "wrote" };
                println!(
                    "ods disable ({mode}) root {} — would_edit={} edit={} would_delete={} delete={}",
                    report.root.display(),
                    report.would_edit.len(),
                    report.edited.len(),
                    report.would_delete.len(),
                    report.deleted.len()
                );
                for p in report
                    .would_edit
                    .iter()
                    .chain(report.edited.iter())
                    .take(30)
                {
                    if let Ok(rel) = p.strip_prefix(&report.root) {
                        println!("  edit {}", rel.display());
                    } else {
                        println!("  edit {}", p.display());
                    }
                }
                for p in report
                    .would_delete
                    .iter()
                    .chain(report.deleted.iter())
                    .take(20)
                {
                    if let Ok(rel) = p.strip_prefix(&report.root) {
                        println!("  delete {}", rel.display());
                    } else {
                        println!("  delete {}", p.display());
                    }
                }
                if report.dry_run {
                    println!(
                        "re-run with --write to apply; then remove ods from CI if needed"
                    );
                }
            }
        }
        OutputFormat::Json | OutputFormat::Sarif => {
            println!(
                r#"{{"root":{},"dry_run":{},"already_disabled":{},"would_edit":{},"edited":{},"would_delete":{},"deleted":{}}}"#,
                json_escape(&report.root.display().to_string()),
                report.dry_run,
                report.already_disabled,
                report.would_edit.len(),
                report.edited.len(),
                report.would_delete.len(),
                report.deleted.len()
            );
        }
    }
    Ok(ExitCode::from(0))
}

#[cfg(test)]
mod test_disable_command {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_disable_command_text_and_json() {
        let td = tempdir().unwrap();
        let root = td.path();

        // help
        let res = run_disable_command(&["ods".into(), "disable".into(), "--help".into()]);
        assert!(res.is_ok());

        // create valid workspace
        fs::write(root.join("ods.toml"), "spec = \"0.1\"\n").unwrap();
        fs::write(
            root.join("index.ods.md"),
            "---\nprofile: index\nods: 0.1\n---\n\n# Root\n",
        )
        .unwrap();

        // dry-run text
        let res = run_disable_command(&["ods".into(), "disable".into(), root.to_str().unwrap().into()]);
        assert!(res.is_ok());

        // dry-run json
        let res = run_disable_command(&[
            "ods".into(),
            "disable".into(),
            root.to_str().unwrap().into(),
            "--format".into(),
            "json".into(),
        ]);
        assert!(res.is_ok());

        // real disable with --write
        let res = run_disable_command(&[
            "ods".into(),
            "disable".into(),
            root.to_str().unwrap().into(),
            "--write".into(),
            "--keep-frontmatter".into(),
        ]);
        assert!(res.is_ok());
    }
}

